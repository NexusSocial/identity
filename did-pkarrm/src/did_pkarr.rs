//! [DidPkarr] and its error types.

use std::{fmt::Display, str::FromStr};

use crate::data_model::did::Did;

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
		"pubkey bytes were of length {0} but expected length {}",
		ed25519_dalek::PUBLIC_KEY_LENGTH
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
	use hex_literal::hex;
	use std::{str::FromStr as _, sync::LazyLock};

	use crate::{data_model::did::Did, DidPkarr};

	// From https://datatracker.ietf.org/doc/html/rfc8032#section-7.1
	static ED25519_EXAMPLES: LazyLock<Vec<ed25519_dalek::SigningKey>> =
		LazyLock::new(|| {
			let test1 = (
				hex!(
					"9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
				),
				hex!(
					"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
				),
			);
			let test2 = (
				hex!(
					"4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb"
				),
				hex!(
					"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c"
				),
			);
			let test3 = (
				hex!(
					"c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7"
				),
				hex!(
					"fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025"
				),
			);
			let test1024 = (
				hex!(
					"f5e5767cf153319517630f226876b86c8160cc583bc013744c6bf255f5cc0ee5"
				),
				hex!(
					"278117fc144c72340f67d0f2316e8386ceffbf2b2428c9c51fef7c597f1d426e"
				),
			);
			let test_sha = (
				hex!(
					"833fe62409237b9d62ec77587520911e9a759cec1d19755b7da901b96dca3d42"
				),
				hex!(
					"ec172b93ad5e563bf4932c70e1245034c35467ef2efd4d64ebf819683467e2bf"
				),
			);
			[test1, test2, test3, test1024, test_sha]
				.into_iter()
				.map(|(private, public)| {
					let private = ed25519_dalek::SigningKey::from_bytes(&private);
					assert_eq!(private.verifying_key().as_bytes(), &public);
					private
				})
				.collect()
		});

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
