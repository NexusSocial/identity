use clap::{Parser, Subcommand};
use color_eyre::{Result, Section};
use did_cli::DidMethodKind;
use did_common::did::Did;
use ed25519_dalek::SigningKey;
use eyre::{eyre, Context};
use tracing::info;
use tracing_subscriber::{
	layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter,
};

fn main() -> Result<()> {
	color_eyre::install()?;
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let args = Args::parse();
	info!("starting");
	match args.subcommands {
		Subcommands::Create(cmd) => cmd.run(),
		Subcommands::Read(cmd) => cmd.run(),
		Subcommands::Update(cmd) => cmd.run(),
	}
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
	#[command(subcommand)]
	subcommands: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
	Create(CreateCmd),
	Read(ReadCmd),
	Update(UpdateCmd),
}

#[derive(Debug, Parser)]
struct CreateCmd {
	kind: String,
}

impl CreateCmd {
	fn run(self) -> Result<()> {
		let s = self.kind.trim();
		let method = if s == "did:key" {
			DidMethodKind::Key
		} else if s == "did:pkarr" {
			DidMethodKind::Pkarr
		} else {
			return Err(eyre!("unknown did method"))
				.suggestion("try did:key or did:pkarr");
		};
		let client = did_cli::client::Client::builder().build();
		let mut rand = rand::thread_rng();
		let priv_key = SigningKey::generate(&mut rand);
		let did = client
			.create(method, &priv_key)
			.wrap_err("failed to create did")?;
		println!("{did}");

		Ok(())
	}
}

#[derive(Debug, Parser)]
struct ReadCmd {
	did: Did,
}

impl ReadCmd {
	fn run(self) -> Result<()> {
		let client = did_cli::client::Client::builder().build();
		let doc = client.read(&self.did)?;
		println!("{doc:#?}");

		Ok(())
	}
}

#[derive(Debug, Parser)]
struct UpdateCmd {
	did: Did,
}

impl UpdateCmd {
	fn run(self) -> Result<()> {
		todo!()
	}
}
