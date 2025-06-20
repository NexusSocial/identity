//! [DidPkarr] and its error types.

use std::{fmt::Display, str::FromStr};

use crate::dids::Did;

use ed25519_dalek::SignatureError;
pub use ed25519_dalek::PUBLIC_KEY_LENGTH;

/// Represents a `did:pkarr`.
#[derive(Debug, Eq)]
pub struct DidPkarr {
	// We keep both representations becaue we need both often.
	key_repr: ed25519_dalek::VerifyingKey,
	str_repr: String,
}

impl PartialEq for DidPkarr {
	// slightly more efficient than default impl by only comparing key_expr
	fn eq(&self, other: &Self) -> bool {
		let is_pk_equal = self.key_repr == other.key_repr;
		debug_assert_eq!(is_pk_equal, (self.as_str() == other.as_str()));

		is_pk_equal
	}
}

#[derive(Debug, thiserror::Error)]
#[error("not a ed25519 public key")]
pub struct InvalidPubkeyErr(#[from] SignatureError);

impl DidPkarr {
	pub fn from_pubkey_bytes(
		public_key: &[u8; ed25519_dalek::PUBLIC_KEY_LENGTH],
	) -> Result<Self, InvalidPubkeyErr> {
		let key_repr = ed25519_dalek::VerifyingKey::from_bytes(public_key)?;
		// Unfortunate that we have to allocate so many times...
		let encoded = base32::encode(base32::Alphabet::Z, key_repr.as_bytes());
		let str_repr = format!("did:pkarr:{encoded}");

		Ok(Self { key_repr, str_repr })
	}

	pub fn as_str(&self) -> &str {
		&self.str_repr
	}

	pub fn as_pubkey(&self) -> &[u8; ed25519_dalek::PUBLIC_KEY_LENGTH] {
		self.key_repr.as_bytes()
	}

	// TODO: allow Did type to be referential, and turn this into `as_did()`
	pub fn to_did(&self) -> Did {
		self.str_repr.parse().unwrap()
	}
}

#[derive(Debug, thiserror::Error)]
#[error("not a valid did:pkarr")]
pub struct DidPkarrParseErr(#[from] DidPkarrParseErrInner);

/// Inner type ensures we can evolve exact reasons without breaking changes to API
#[derive(Debug, thiserror::Error)]
enum DidPkarrParseErrInner {
	#[error("did not start with `did:pkarr:`")]
	WrongPrefix,
	#[error("not base32-z encoded")]
	NotBase32zEncoded,
	#[error(
		"pubkey bytes were of length {0} but expected length {expected_len}",
		expected_len=ed25519_dalek::PUBLIC_KEY_LENGTH
	)]
	WrongLength(usize),
	#[error(transparent)]
	InvalidPubkey(#[from] SignatureError),
}

impl TryFrom<String> for DidPkarr {
	type Error = DidPkarrParseErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		use DidPkarrParseErrInner as Inner;
		let Some(suffix) = value.strip_prefix("did:pkarr:") else {
			return Err(Inner::WrongPrefix.into());
		};
		let Some(decoded) = base32::decode(base32::Alphabet::Z, suffix) else {
			return Err(Inner::NotBase32zEncoded.into());
		};
		let pubkey_bytes: &[u8; PUBLIC_KEY_LENGTH] = decoded
			.as_slice()
			.try_into()
			.map_err(|_| Inner::WrongLength(decoded.len()))?;
		let pubkey = ed25519_dalek::VerifyingKey::from_bytes(pubkey_bytes)
			.map_err(Inner::from)?;

		Ok(Self {
			key_repr: pubkey,
			str_repr: value,
		})
	}
}

impl FromStr for DidPkarr {
	type Err = DidPkarrParseErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.to_owned().try_into()
	}
}

impl Display for DidPkarr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::dids::test::ED25519_EXAMPLES;

	#[test]
	fn test_valid_pubkeys() {
		for pk in ED25519_EXAMPLES
			.iter()
			.map(|sk| sk.verifying_key().to_bytes())
		{
			let expected_str =
				format!("did:pkarr:{}", base32::encode(base32::Alphabet::Z, &pk));

			// test bytes -> did
			let did_from_bytes = DidPkarr::from_pubkey_bytes(&pk).unwrap();
			assert_eq!(did_from_bytes.as_pubkey(), &pk, "bytes repr didn't match");
			assert_eq!(
				did_from_bytes.as_str(),
				expected_str,
				"string repr didn't match"
			);
			assert_eq!(
				format!("{did_from_bytes}"),
				expected_str,
				"display didn't match"
			);
			assert_eq!(
				did_from_bytes.to_did(),
				Did::from_str(&expected_str).unwrap(),
				"did repr didn't match"
			);

			// test str -> did
			let did_from_str: DidPkarr = expected_str.parse().unwrap();
			assert_eq!(did_from_str, did_from_bytes);
		}
	}

	#[test]
	fn test_invalid_strings() {
		let suffix = base32::encode(
			base32::Alphabet::Z,
			ED25519_EXAMPLES[0].verifying_key().as_bytes(),
		);
		assert!(
			DidPkarr::from_str(&format!("did:pkarr:{suffix}")).is_ok(),
			"sanity"
		);

		DidPkarr::from_str("").unwrap_err();
		DidPkarr::from_str("").unwrap_err();
		DidPkarr::from_str("did:").unwrap_err();
		DidPkarr::from_str("did:pkarr").unwrap_err();
		DidPkarr::from_str("did:pkarr:").unwrap_err();
		DidPkarr::from_str(":pkarr").unwrap_err();
		DidPkarr::from_str(":pkarr:").unwrap_err();
		DidPkarr::from_str("pkarr:").unwrap_err();
		DidPkarr::from_str(&format!(":pkarr:{suffix}")).unwrap_err();
		DidPkarr::from_str(&format!("did:pkarr{suffix}")).unwrap_err();
		DidPkarr::from_str(&format!("did:pkarr:{suffix}:")).unwrap_err();
		DidPkarr::from_str(&format!("did:pkarr:{suffix}a")).unwrap_err();
		DidPkarr::from_str(&format!("did::{suffix}")).unwrap_err();
	}
}
