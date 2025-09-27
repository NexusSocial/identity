#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "export-pdf")]
extern crate alloc;

#[cfg(feature = "export-pdf")]
mod exports;

use core::{fmt, ops::Deref};

use bip39::{Language, Mnemonic};
use bon::bon;
use hmac::{Hmac, Mac};
use rand_core::CryptoRng;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const ED25519_SIGNING_KEY_BYTES: usize = 32;
const SHA256_BYTES: usize = 32;
const ENTROPY_BYTES: usize = 32;
const PHRASE_LEN: usize = 24;
const SEED_BYTES: usize = 64;
const PURPOSE: u32 = 1778203272 >> 1; // Randomly generated
const COIN_TYPE: u32 = 1648924679 >> 1; // Randomly generated

// TODO: Dalek only impls clone. Maybe we should not implement these?
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Ed25519SigningKey(pub [u8; ED25519_SIGNING_KEY_BYTES]);

impl fmt::Debug for Ed25519SigningKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Ed25519SigningKey").finish_non_exhaustive()
	}
}

#[cfg(feature = "dalek")]
impl From<Ed25519SigningKey> for ed25519_dalek::SigningKey {
	fn from(value: Ed25519SigningKey) -> Self {
		ed25519_dalek::SigningKey::from_bytes(&value.0)
	}
}

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Seed([u8; SEED_BYTES]);

impl fmt::Debug for Seed {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Seed").finish_non_exhaustive()
	}
}

/// Wrapper struct, because for god knows what reason, [`Mnemonic`] implements
/// Debug, making it easy to leak the secret.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct MnemonicWrapper(Mnemonic);

impl MnemonicWrapper {
	fn generate_from_entropy(lang: Language, entropy: &[u8; ENTROPY_BYTES]) -> Self {
		Mnemonic::from_entropy_in(lang, entropy)
			.expect("should be infallible as we generated 256 bits")
			.into()
	}

	fn to_entropy(&self) -> [u8; ENTROPY_BYTES] {
		let (array, len) = self.0.to_entropy_array();

		array[0..len].try_into().expect("infallible")
	}

	fn as_display(&self) -> &(impl core::fmt::Display + core::fmt::Debug) {
		&self.0
	}
}

impl fmt::Debug for MnemonicWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Mnemonic")
			.field("lang", &self.0.language())
			.finish_non_exhaustive()
	}
}

impl From<Mnemonic> for MnemonicWrapper {
	fn from(value: Mnemonic) -> Self {
		Self(value)
	}
}

/// 32-bit error detection via truncated `HMAC-Sha256(key=mnemonic, data=passprhase)`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct PassphraseHmac(u32);

impl PassphraseHmac {
	fn new(mnemonic: &MnemonicWrapper, pass: &str) -> Self {
		let h: [u8; SHA256_BYTES] = HmacSha256::new_from_slice(&mnemonic.to_entropy())
			.expect("hmac can take key of any size")
			.chain_update(pass)
			.finalize()
			.into_bytes()
			.into();

		Self(u32::from_le_bytes(h[0..4].try_into().expect("infallible")))
	}
}

/// A BIP39 alphabet recovery phrase, 256 bits in entropy.
///
/// Supports multiple languages.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RecoveryPhrase {
	phrase: MnemonicWrapper,
	// Hash only present if we are password protected.
	passphrase_hmac: Option<PassphraseHmac>,
}

#[derive(Debug, thiserror::Error, Copy, Clone)]
#[error("was not ascii")]
pub struct AsciiErr;

#[derive(Default, Copy, Clone)]
pub struct Ascii<'a>(&'a str);

impl<'a> Ascii<'a> {
	pub const EMPTY: Self = Ascii("");

	pub const fn try_from_const(s: &'a str) -> Result<Self, AsciiErr> {
		if !s.is_ascii() {
			return Err(AsciiErr);
		}

		Ok(Self(s))
	}
}

impl Deref for Ascii<'_> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<'a> TryFrom<&'a str> for Ascii<'a> {
	type Error = AsciiErr;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		Self::try_from_const(value)
	}
}

#[bon]
impl RecoveryPhrase {
	#[builder]
	pub fn new(
		/// NOTE: Most BIP39 compatible wallets only support english. Consider if
		/// localization is actually important for the user.
		#[builder(default = Language::English)]
		language: Language,
		entropy: [u8; ENTROPY_BYTES],
		#[builder(default)] password: Ascii<'_>,
	) -> Self {
		let phrase = MnemonicWrapper::generate_from_entropy(language, &entropy);
		let passphrase_hmac = if password.0.is_empty() {
			None
		} else {
			Some(PassphraseHmac::new(&phrase, password.0))
		};

		RecoveryPhrase {
			phrase,
			passphrase_hmac,
		}
	}

	pub fn to_words(&self) -> [&'static str; PHRASE_LEN] {
		let mut buf = [""; PHRASE_LEN];
		let mut i = 0;
		for w in self.phrase.0.words() {
			buf[i] = w;
			i += 1;
		}
		assert_eq!(PHRASE_LEN, i, "sanity");

		buf
	}

	pub fn is_password_protected(&self) -> bool {
		self.passphrase_hmac.is_some()
	}

	/// Computes the ed25519 signing key from the recovery phrase + password. Set
	/// password to empty string if no password is expected. Use `0` for the default
	/// account.
	pub fn to_key(
		&self,
		password: Ascii<'_>,
		account: u16,
	) -> Result<Ed25519SigningKey, PasswordError> {
		let seed = self.to_seed(password)?;
		let signing_key: [u8; ED25519_SIGNING_KEY_BYTES] =
			slip10_ed25519::derive_ed25519_private_key(
				&seed.0,
				&[PURPOSE, COIN_TYPE, account.into()],
			);

		Ok(Ed25519SigningKey(signing_key))
	}

	/// Returns a type that can be displayed. Be careful not to leak this anywhere
	/// as it is supposed to be kept secret.
	pub fn as_display(&self) -> &(impl core::fmt::Display + core::fmt::Debug) {
		self.phrase.as_display()
	}

	#[cfg(feature = "export-pdf")]
	pub fn export(&self, app_name: &str) -> crate::exports::Exports {
		crate::exports::PdfGenerator {
			words: self.to_words(),
			app_name,
			password: self.is_password_protected(),
		}
		.build()
	}

	/// Helper function to generate the seed from the mnemonic + password. Set password
	/// to empty string if no password is desired.
	fn to_seed(&self, password: Ascii) -> Result<Seed, PasswordError> {
		let is_password_protected = self.passphrase_hmac.is_some();
		match (is_password_protected, password.0.is_empty()) {
			(false, true) => (),
			(true, true) => return Err(PasswordError::ExpectedPassword),
			(false, false) => return Err(PasswordError::UnexpectedPassword),
			(true, false) => {
				let Some(ref expected_hmac) = self.passphrase_hmac else {
					unreachable!()
				};
				let candidate_hmac = PassphraseHmac::new(&self.phrase, password.0);
				if &candidate_hmac != expected_hmac {
					return Err(PasswordError::IncorrectPassword);
				}
			}
		}

		Ok(Seed(self.phrase.0.to_seed_normalized(password.0)))
	}
}

use recovery_phrase_builder::{IsUnset, SetEntropy, SetLanguage, State};

impl<'a, S: State> RecoveryPhraseBuilder<'a, S> {
	#[cfg(feature = "os-rng")]
	pub fn random(self) -> RecoveryPhraseBuilder<'a, SetEntropy<S>>
	where
		S::Entropy: IsUnset,
	{
		let mut rng = rand_core::UnwrapErr(rand_core::OsRng);
		self.from_rng(&mut rng)
	}

	pub fn from_rng(
		self,
		value: &mut impl CryptoRng,
	) -> RecoveryPhraseBuilder<'a, SetEntropy<S>>
	where
		S::Entropy: IsUnset,
	{
		let mut entropy = [0; ENTROPY_BYTES];
		value.fill_bytes(&mut entropy);
		self.entropy(entropy)
	}

	pub fn from_phrase(
		self,
		phrase: Ascii,
	) -> Result<RecoveryPhraseBuilder<'a, SetLanguage<SetEntropy<S>>>, bip39::Error>
	where
		S::Entropy: IsUnset,
		S::Language: IsUnset,
	{
		let m = MnemonicWrapper::from(Mnemonic::parse_in_normalized(
			Language::English,
			phrase.0,
		)?);
		Ok(self.entropy(m.to_entropy()).language(m.0.language()))
	}
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, thiserror::Error)]
pub enum PasswordError {
	#[error("the phrase is password protected but no password was provided")]
	ExpectedPassword,
	#[error("the phrase is not password protected but a password was provided")]
	UnexpectedPassword,
	#[error("the password was incorrect")]
	IncorrectPassword,
}

#[cfg(test)]
mod test {
	use super::*;
	use hex_literal::hex;
	use rand::rngs::StdRng;
	use rand_core::SeedableRng;

	struct Example {
		entropy: [u8; ENTROPY_BYTES],
		phrase: Ascii<'static>, // `PHRASE_LEN` words
		password: Ascii<'static>,
		seed_with_password: [u8; SEED_BYTES],
		seed_empty_password: [u8; SEED_BYTES],
	}

	const fn u<T: Copy, E: Copy>(r: Result<T, E>) -> T {
		let Ok(t) = r else {
			panic!("failed to unwrap");
		};

		t
	}

	// Generated from https://iancoleman.io/bip39/
	const KNOWN_PHRASES: &[Example] = &[Example {
		entropy: hex!(
			"71bac318678fd69a3f51fc225a968f04003bcc37235473ccb95aad0a14f495c7"
		),
		phrase: u(Ascii::try_from_const(
			"immune stock ship someone word escape wool display car start phrase amount admit toward symptom hedgehog inherit grape find foam pattern kid finish toast",
		)),
		password: u(Ascii::try_from_const("foobar")),
		seed_with_password: hex!(
			"4b557b4918eccf77831c4771d8a222307cf11755c614f7623976cbe5ee8e0d2262a526ff1f0818d1ddf4e7f8526af68ea1ff980f8dc47529aa4ae8d43316974d"
		),
		seed_empty_password: hex!(
			"32d9c45e00f69a944b1d76262d78c2c8b559f8ce73f4b04238c30514de2d7e208348403ade7d24081ad251f1bdad97f3b245a446374db0888444637f36632367"
		),
	}];

	#[test]
	fn test_generate_runs() {
		let mut rng = StdRng::seed_from_u64(1337);

		let phrase = RecoveryPhrase::builder()
			.language(Language::English)
			.from_rng(&mut rng)
			.build();
		assert_eq!(phrase.to_words().len(), PHRASE_LEN);
		assert!(
			phrase
				.to_words()
				.into_iter()
				.all(|w| Language::English.find_word(w).is_some())
		);
	}

	#[test]
	fn test_known_phrases_password() {
		for e in KNOWN_PHRASES {
			let phrase_from_entropy = RecoveryPhrase::builder()
				.password(e.password)
				.language(Language::English)
				.entropy(e.entropy)
				.build();

			let expected_iter = e.phrase.split(" ");
			assert_eq!(expected_iter.clone().count(), PHRASE_LEN);
			assert_eq!(phrase_from_entropy.to_words().len(), PHRASE_LEN);
			for (a, b) in phrase_from_entropy
				.to_words()
				.into_iter()
				.zip(expected_iter)
			{
				assert_eq!(a, b);
			}

			let phrase_from_phrase = RecoveryPhrase::builder()
				.password(e.password)
				.from_phrase(e.phrase)
				.unwrap()
				.build();
			assert_eq!(phrase_from_phrase, phrase_from_entropy);

			assert_eq!(
				phrase_from_entropy.to_seed(Ascii::EMPTY),
				Err(PasswordError::ExpectedPassword),
			);
			assert_eq!(
				phrase_from_entropy.to_key(Ascii::EMPTY, 0),
				Err(PasswordError::ExpectedPassword),
			);

			assert_eq!(
				phrase_from_entropy.to_seed("non-empty".try_into().unwrap()),
				Err(PasswordError::IncorrectPassword)
			);
			assert_eq!(
				phrase_from_entropy.to_key("non-empty".try_into().unwrap(), 0),
				Err(PasswordError::IncorrectPassword)
			);

			assert_eq!(
				phrase_from_entropy.to_seed(e.password).unwrap().0,
				e.seed_with_password
			);
			assert!(phrase_from_entropy.to_key(e.password, 0).is_ok());
		}
	}

	#[test]
	fn test_known_phrases_no_password() {
		for e in KNOWN_PHRASES {
			let phrase_from_entropy = RecoveryPhrase::builder()
				.language(Language::English)
				.entropy(e.entropy)
				.build();

			let expected_iter = e.phrase.split(" ");
			assert_eq!(expected_iter.clone().count(), PHRASE_LEN);
			assert_eq!(phrase_from_entropy.to_words().len(), PHRASE_LEN);
			for (a, b) in phrase_from_entropy
				.to_words()
				.into_iter()
				.zip(expected_iter)
			{
				assert_eq!(a, b);
			}

			let phrase_from_phrase = RecoveryPhrase::builder()
				.from_phrase(e.phrase)
				.unwrap()
				.build();
			assert_eq!(phrase_from_phrase, phrase_from_entropy);

			assert_eq!(
				phrase_from_entropy.to_seed("").unwrap().0,
				e.seed_empty_password
			);
			assert!(phrase_from_entropy.to_key("", 0).is_ok());

			assert_eq!(
				phrase_from_entropy.to_seed("non-empty"),
				Err(PasswordError::UnexpectedPassword)
			);
			assert_eq!(
				phrase_from_entropy.to_key("non-empty", 0),
				Err(PasswordError::UnexpectedPassword)
			);

			assert_eq!(
				phrase_from_entropy.to_seed(e.password),
				Err(PasswordError::UnexpectedPassword)
			);
			assert_eq!(
				phrase_from_entropy.to_key(e.password, 0),
				Err(PasswordError::UnexpectedPassword)
			);
		}
	}
}
