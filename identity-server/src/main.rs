use std::path::PathBuf;

use clap::Parser as _;
use color_eyre::eyre::{ensure, Context, OptionExt, Result};
use futures::FutureExt;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use identity_server::{
	config::{Config, DatabaseConfig},
	jwks_provider::JwksProvider,
	spawn_http_redirect_server, spawn_http_server, spawn_https_server, MigratedDbPool,
};

const GOOGLE_CLIENT_ID_DOCS_URL: &str = "https://developers.google.com/identity/gsi/web/guides/get-google-api-clientid#get_your_google_api_client_id";

#[derive(clap::Parser, Debug)]
struct Cli {
	#[clap(long, env)]
	config: PathBuf,
}

/// Convenient container to manager all tasks that need to be monitored and reaped.
#[derive(Debug)]
struct Tasks {
	http: Option<(JoinHandle<Result<()>>, oneshot::Sender<()>)>,
	https: Option<(JoinHandle<Result<()>>, oneshot::Sender<()>)>,
}

impl Tasks {
	/// Runs all tasks
	async fn run(self) -> Result<()> {
		let tasks_fut = async move {
			match self {
				Tasks {
					http: None,
					https: None,
				} => unreachable!("at least one server type should be running"),
				Tasks {
					http: Some((http_handle, _http_kill)),
					https: None,
				} => http_handle
					.await
					.wrap_err("HTTP server panicked")?
					.wrap_err("HTTP server exited abnormally"),
				Tasks {
					http: None,
					https: Some((https_handle, _https_kill)),
				} => https_handle
					.await
					.wrap_err("HTTPS server panicked")?
					.wrap_err("HTTPS server exited abnormally"),
				Tasks {
					http: Some((http_handle, _http_kill)),
					https: Some((https_handle, _https_kill)),
				} => {
					let ((), ()) = tokio::try_join!(
						http_handle.map(|r| r
							.wrap_err("HTTP server panicked")?
							.wrap_err("HTTP server exited abnormally")),
						https_handle.map(|r| r
							.wrap_err("HTTPS server panicked")?
							.wrap_err("HTTPS server exited abnormally"))
					)?;
					Ok(())
				}
			}
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

	async fn spawn(config_file: Config, router: axum::Router) -> Result<Self> {
		ensure!(
			config_file.http.is_some() || config_file.https.is_some(),
			"at least one of the `http` or `https` sections in config.toml must \
            be provided"
		);
		Ok(if let Some(https_cfg) = config_file.https {
			let (https_task, https_kill_signal, https_port) =
				spawn_https_server(https_cfg, router)
					.await
					.wrap_err("failed to spawn http server")?;
			let https = Some((https_task, https_kill_signal));

			let http = if let Some(http_cfg) = config_file.http {
				let (http_task, http_kill_signal, _http_port) =
					spawn_http_redirect_server(http_cfg.port, https_port)
						.await
						.wrap_err("failed to spawn http redirect server")?;
				Some((http_task, http_kill_signal))
			} else {
				None
			};

			Tasks { http, https }
		} else {
			let http_cfg = config_file.http.expect(
				"infallible: we already know that it is Some, since https is None",
			);
			let (http_task, http_kill_signal, _http_port) =
				spawn_http_server(http_cfg, router)
					.await
					.wrap_err("failed to spawn http server")?;

			Tasks {
				https: None,
				http: Some((http_task, http_kill_signal)),
			}
		})
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let cli = Cli::parse();

	let config_file = tokio::fs::read_to_string(cli.config)
		.await
		.wrap_err("failed to read config file")?;
	let config_file: Config =
		config_file.parse().wrap_err("config file was invalid")?;

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

	Tasks::spawn(config_file, router)
		.await
		.wrap_err("failed to spawn tasks")?
		.run()
		.await
}
