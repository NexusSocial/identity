#![no_std]
#![forbid(unsafe_code)]

use core::{fmt, str::FromStr};

use bip39::{Language, Mnemonic};
use rand_core::CryptoRng;

const ED25519_SIGNING_KEY_BYTES: usize = 32;

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Ed25519SigningKey(pub [u8; ED25519_SIGNING_KEY_BYTES]);

impl fmt::Debug for Ed25519SigningKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Ed25519SigningKey").finish_non_exhaustive()
	}
}

/// Wrapper struct, because for god knows what reason, [`Mnemonic`] implements
/// Debug, making it easy to leak the secret.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct MnemonicWrapper(Mnemonic);

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

/// A BIP39 alphabet recovery phrase, 256 bits in entropy.
///
/// Supports multiple languages.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RecoveryPhrase {
	phrase: MnemonicWrapper,
	password_protected: bool,
}

impl RecoveryPhrase {
	pub fn from_phrase(phrase: &str) -> Result<Self, bip39::Error> {
		let phrase: MnemonicWrapper = Mnemonic::from_str(phrase)?.into();

		Ok(Self {
			phrase,
			password_protected: false,
		})
	}

	/// NOTE: Most BIP39 compatible wallets only support english. Consider if
	/// localization is actually important for the user.
	pub fn generate(lang: Language, mut rand: impl CryptoRng) -> Self {
		let mut entropy = [0u8; 32];
		rand.fill_bytes(&mut entropy);
		rand.fill_bytes(&mut entropy);
		rand.fill_bytes(&mut entropy);
		rand.fill_bytes(&mut entropy);

		Self::generate_from_entropy(lang, &entropy)
	}

	pub fn generate_from_entropy(lang: Language, entropy: &[u8; 32]) -> Self {
		let phrase: MnemonicWrapper = Mnemonic::from_entropy_in(lang, entropy)
			.expect("should be infallible as we generated 256 bits")
			.into();

		Self {
			phrase,
			password_protected: false,
		}
	}

	pub fn password_protected(&mut self) -> &mut bool {
		&mut self.password_protected
	}

	pub fn as_words(&self) -> impl Iterator<Item = &'static str> + Clone + '_ {
		self.phrase.0.words()
	}

	pub fn is_password_protected(&self) -> bool {
		self.password_protected
	}

	/// Computes the ed25519 signing key from the recovery phrase + password. Set
	/// password to empty string if no password is expected.
	pub fn to_ed25519(
		&self,
		password: &str,
	) -> Result<Ed25519SigningKey, PasswordError> {
		let seed = self.to_seed(password)?;
		// TODO: what the hell are indexes
		let signing_key: [u8; ED25519_SIGNING_KEY_BYTES] =
			slip10_ed25519::derive_ed25519_private_key(&seed, &[0]);

		Ok(Ed25519SigningKey(signing_key))
	}

	/// Helper function to generate the seed from the mnemonic + password. Set password
	/// to empty string if no password is desired.
	fn to_seed(&self, password: &str) -> Result<[u8; 64], PasswordError> {
		match (self.password_protected, password.is_empty()) {
			(true, false) | (false, true) => (),
			(true, true) => return Err(PasswordError::ExpectedPassword),
			(false, false) => return Err(PasswordError::UnexpectedPassword),
		}

		Ok(self.phrase.0.to_seed(password))
	}
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
	#[error("the phrase is password protected but no password was provided")]
	ExpectedPassword,
	#[error("the phrase is not password protected but a password was provided")]
	UnexpectedPassword,
}

#[cfg(test)]
mod test {
	use super::*;
	use hex_literal::hex;
	use rand::rngs::StdRng;
	use rand_core::SeedableRng;

	struct Example {
		entropy: [u8; 32],
		phrase: &'static str, // 24 words
		password: &'static str,
		seed_with_password: [u8; 64],
		seed_empty_password: [u8; 64],
	}

	// Generated from https://iancoleman.io/bip39/
	const KNOWN_PHRASES: &[Example] = &[Example {
		entropy: hex!(
			"71bac318678fd69a3f51fc225a968f04003bcc37235473ccb95aad0a14f495c7"
		),
		phrase: "immune stock ship someone word escape wool display car start phrase amount admit toward symptom hedgehog inherit grape find foam pattern kid finish toast",
		password: "foobar",
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

		let phrase = RecoveryPhrase::generate(Language::English, &mut rng);
		assert_eq!(phrase.as_words().count(), 24);
		assert!(
			phrase
				.as_words()
				.all(|w| Language::English.find_word(w).is_some())
		);
	}

	#[test]
	fn test_known_phrases() {
		for e in KNOWN_PHRASES {
			let mut phrase_from_entropy =
				RecoveryPhrase::generate_from_entropy(Language::English, &e.entropy);
			let expected_iter = e.phrase.split(" ");
			assert_eq!(expected_iter.clone().count(), 24);
			assert_eq!(phrase_from_entropy.as_words().count(), 24);
			for (a, b) in phrase_from_entropy.as_words().zip(expected_iter) {
				assert_eq!(a, b);
			}
			let phrase_from_phrase = RecoveryPhrase::from_phrase(e.phrase).unwrap();
			assert_eq!(phrase_from_phrase, phrase_from_entropy);

			assert_eq!(
				phrase_from_entropy.to_seed("").unwrap(),
				e.seed_empty_password
			);
			*phrase_from_entropy.password_protected() = true;
			assert_eq!(
				phrase_from_entropy.to_seed(e.password).unwrap(),
				e.seed_with_password
			);
		}
	}

	// TODO: test to_ed25519
}
