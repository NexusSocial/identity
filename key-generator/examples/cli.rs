use clap::Parser;
use color_eyre::Result;
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use owo_colors::OwoColorize;

fn main() -> Result<()> {
	color_eyre::install()?;

	let args = Args::parse();

	match args {
		Args::New(c) => c.run(),
	}
}

#[derive(Debug, Parser)]
#[command(version, about)]
enum Args {
	New(NewCmd),
}

/// Generates a new account
#[derive(Debug, Parser)]
struct NewCmd {
	#[arg(long, default_value = "")]
	password: String,

	/// The account number to inspect
	#[arg(long, default_value_t = 0)]
	account: u16,

	#[arg(long)]
	print_private_key: bool,
	#[arg(long)]
	print_phrase: bool,

	/// Use base58-btc encoding
	#[arg(long, group = "enc")]
	b58: bool,
	/// Use base64 url-safe-no-pad encoding
	#[arg(long, group = "enc")]
	b64: bool,
	/// Use z-base-32 encoding
	#[arg(long, group = "enc")]
	b32: bool,
	/// Use hexadecimal encoding (the default)
	#[arg(long, group = "enc", default_value_t = true)]
	hex: bool,
}

impl NewCmd {
	fn run(self) -> Result<()> {
		let encode = if self.b58 {
			encode_b58
		} else if self.b64 {
			encode_b64
		} else if self.b32 {
			encode_b32
		} else {
			encode_hex
		};
		let phrase = key_generator::RecoveryPhrase::builder()
			.random()
			.password(&self.password)
			.build();
		let mut signing_key: SigningKey =
			phrase.to_key("", 0).expect("password is correct").into();

		if self.print_private_key {
			let private_key_encoded = encode(signing_key.as_bytes());
			println!("{}: {private_key_encoded}", "private key".bold().red());
		}

		if self.print_phrase {
			println!(
				"{}: {}",
				"recovery phrase".bold().red(),
				phrase.as_display()
			)
		}

		println!(
			"{}: {}",
			"password protection".bold().green(),
			phrase.is_password_protected()
		);

		let public_key = signing_key.verifying_key();
		let public_key_encoded = encode(public_key.as_bytes());
		println!("{}: {public_key_encoded}", "public key".bold().green());

		const EXAMPLE_MESSAGE: &str = "example message";
		let signature = signing_key.sign(EXAMPLE_MESSAGE.as_bytes());
		public_key
			.verify_strict(EXAMPLE_MESSAGE.as_bytes(), &signature)
			.expect("sanity: should always be valid");
		let encoded_sig = encode(&signature.to_vec());
		println!(
			"{} for \"{}\": {encoded_sig}",
			"signature".bold().green(),
			EXAMPLE_MESSAGE.italic()
		);

		Ok(())
	}
}

fn encode_hex(b: &dyn AsRef<[u8]>) -> String {
	hex::encode_upper(b)
}

fn encode_b58(b: &dyn AsRef<[u8]>) -> String {
	bs58::encode(b)
		.with_alphabet(bs58::Alphabet::BITCOIN)
		.into_string()
}

fn encode_b64(b: &dyn AsRef<[u8]>) -> String {
	use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

	BASE64_URL_SAFE_NO_PAD.encode(b)
}

fn encode_b32(b: &dyn AsRef<[u8]>) -> String {
	base32::encode(base32::Alphabet::Z, b.as_ref())
}
