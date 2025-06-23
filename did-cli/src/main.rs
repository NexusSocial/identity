use clap::{Parser, Subcommand};
use color_eyre::Result;
use did_common::did::Did;
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
}

#[derive(Debug, Parser)]
struct CreateCmd;

impl CreateCmd {
	fn run(self) -> Result<()> {
		todo!()
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
