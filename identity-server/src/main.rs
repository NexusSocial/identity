use std::{
	io::IsTerminal as _,
	path::{Path, PathBuf},
};

use clap::Parser as _;
use color_eyre::{
	eyre::{bail, Context, OptionExt, Result},
	Section as _,
};
use futures::FutureExt;
use tokio::task::JoinHandle;
use tokio::{io::AsyncWriteExt as _, sync::oneshot};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use identity_server::{
	config::{
		Config, DatabaseConfig, TlsConfig, ValidationError, DEFAULT_CONFIG_CONTENTS,
	},
	jwks_provider::JwksProvider,
	spawn_http_server, spawn_https_server, MigratedDbPool,
};

const GOOGLE_CLIENT_ID_DOCS_URL: &str = "https://developers.google.com/identity/gsi/web/guides/get-google-api-clientid#get_your_google_api_client_id";

async fn load_config(cfg_path: &Path) -> Result<Config> {
	tokio::fs::read_to_string(cfg_path)
		.await
		.wrap_err("failed to read config file")
		.inspect(|cfg| debug!(contents = cfg, "config file contents"))
		.and_then(|cfg| cfg.parse().wrap_err("failed to parse config file"))
		.and_then(|cfg: Config| {
			cfg.validate().map(|()| cfg).or_else(|err| {
				let suggestion = match err {
					ValidationError::UnspecifiedDomain => {
						"try adding your domain in the `http.tls.domains` list"
					}
				};
				Err(err)
					.wrap_err("config file was invalid")
					.suggestion(suggestion)
			})
		})
		.with_note(|| format!("Config file path: {}", cfg_path.display()))
}

#[derive(clap::Parser, Debug)]
#[clap(version)]
struct Cli {
	#[clap(subcommand)]
	command: Commands,
}

#[derive(clap::Parser, Debug)]
enum Commands {
	Serve(ServeArgs),
	DefaultConfig(DefaultConfigArgs),
}

/// Runs the server
#[derive(clap::Parser, Debug)]
struct ServeArgs {
	#[clap(long, env)]
	config: PathBuf,
}

impl ServeArgs {
	async fn run(self) -> Result<()> {
		let cli = self;
		let config_file = load_config(&cli.config).await?;

		let db_pool = {
			let DatabaseConfig::Sqlite { ref db_file } = config_file.database;
			let connect_opts = sqlx::sqlite::SqliteConnectOptions::new()
				.create_if_missing(true)
				.filename(db_file);
			let pool_opts = sqlx::sqlite::SqlitePoolOptions::new();
			let pool = pool_opts
				.connect_with(connect_opts.clone())
				.await
				.wrap_err_with(|| {
					format!(
						"failed to connect to database with path {}",
						connect_opts.get_filename().display()
					)
				})?;
			MigratedDbPool::new(pool)
				.await
				.wrap_err("failed to migrate db pool")?
		};
		let reqwest_client = reqwest::Client::new();

		let v1_cfg = identity_server::v1::RouterConfig {
			uuid_provider: Default::default(),
			db_pool,
		};
		let oauth_cfg = identity_server::oauth::OAuthConfig {
			google_client_id: config_file
				.third_party
				.google
				.clone()
				.ok_or_eyre(format!(
					"currently, setting up google is required. Please follow the \
                instructions at {GOOGLE_CLIENT_ID_DOCS_URL} and fill in the \
                `third_party.google.oauth2_client_id` field in the config.toml",
				))?
				.oauth2_client_id,
			google_jwks_provider: JwksProvider::google(reqwest_client.clone()),
		};
		let router = identity_server::RouterConfig {
			v1: v1_cfg,
			oauth: oauth_cfg,
		}
		.build()
		.await
		.wrap_err("failed to build router")?;

		let cache_dir = config_file.cache.dir();
		debug!("using cache dir {}", cache_dir.display());
		// .join(if cli.prod_tls { "prod" } else { "dev" });
		tokio::fs::create_dir_all(&cache_dir)
			.await
			.wrap_err("failed to create cache directory for certs")?;

		Tasks::spawn(config_file, router)
			.await
			.wrap_err("failed to spawn tasks")?
			.run()
			.await
	}
}

/// Echoes the default config to stdout
#[derive(clap::Parser, Debug)]
struct DefaultConfigArgs {}

impl DefaultConfigArgs {
	async fn run(self) -> Result<()> {
		tokio::io::stdout()
			.write_all(DEFAULT_CONFIG_CONTENTS.as_bytes())
			.await
			.expect("should never fail");
		Ok(())
	}
}

/// Convenient container to manager all tasks that need to be monitored and reaped.
#[derive(Debug)]
struct Tasks {
	http: (JoinHandle<Result<()>>, oneshot::Sender<()>),
}

impl Tasks {
	/// Spawns all subtasks
	async fn spawn(config_file: Config, router: axum::Router) -> Result<Self> {
		let (http_task, http_kill_signal) =
			if matches!(config_file.http.tls, TlsConfig::Disable) {
				let tuple = spawn_http_server(config_file.http, router)
					.await
					.wrap_err("failed to spawn http server")?;
				(tuple.0, tuple.1)
			} else {
				let tuple = spawn_https_server(config_file, router)
					.await
					.wrap_err("failed to spawn https server")?;
				(tuple.0, tuple.1)
			};

		Ok(Tasks {
			http: (http_task, http_kill_signal),
		})
	}

	/// Runs all tasks
	async fn run(self) -> Result<()> {
		let tasks_fut = async move {
			let Tasks {
				http: (http_handle, _http_kill),
			} = self;
			http_handle
				.await
				.wrap_err("HTTP server panicked")?
				.wrap_err("HTTP server exited abnormally")
		};

		let kill_fut = tokio::signal::ctrl_c().map(|r| {
			info!("detected ctrl-c, shutting down...");
			r.wrap_err("error getting ctrl-c signal")
		});

		tokio::select! {
			result = kill_fut => result,
			result = tasks_fut => result,
		}
	}
}

fn is_root() -> bool {
	#[cfg(unix)]
	let result = rustix::process::getuid().is_root();
	#[cfg(windows)]
	let result = false;
	result
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	if is_root() {
		bail!("You should only run this program as a non-root user");
	}

	if !std::io::stdout().is_terminal() {
		debug!("We don't appear to be in a terminal");
	}

	let cli = Cli::parse();
	match cli.command {
		Commands::Serve(args) => args.run().await,
		Commands::DefaultConfig(args) => args.run().await,
	}
}
