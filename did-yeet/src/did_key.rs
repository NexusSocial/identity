use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A did:key string. Does not perform base58 decoding or validate the public key.
///
/// See also the [did:key spec][spec].
///
/// # Example
///
/// ```
/// # use did_yeet::DidKey;
/// /// From did:key spec section 4.1
/// let did_key: DidKey = "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".parse().unwrap();
/// ```
///
/// [spec]: https://w3c-ccg.github.io/did-key-spec
#[derive(
	Debug, Clone, Eq, Hash, derive_more::Display, derive_more::AsRef, Serialize,
)]
#[as_ref(str)]
#[serde(transparent)]
pub struct DidKey(String);

impl DidKey {
	pub const PREFIX: &'static str = "did:key:z";

	/// Construct a `DidKey` from [base58-btc](bs58) encoded data.
	pub fn from_base58_btc_encoded(data: &str) -> Self {
		Self(format!("{}{data}", Self::PREFIX))
	}

	pub fn as_str(&self) -> &str {
		self.0.as_ref()
	}

	// pub fn deserialize_zero_copy<'de: 'a, D>(deserializer: D) -> Result<Self, D::Error>
	// where
	// 	D: Deserializer<'de>,
	// {
	// 	let s: &str = Deserialize::deserialize(deserializer)?;
	//
	// 	DidKey::try_from(s).map_err(serde::de::Error::custom)
	// }
}

impl TryFrom<String> for DidKey {
	type Error = TryFromStrErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		if !value.starts_with(Self::PREFIX) {
			return Err(TryFromStrErr::WrongPrefix);
		}

		Ok(Self(value))
	}
}

impl FromStr for DidKey {
	type Err = TryFromStrErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.to_owned().try_into()
	}
}

impl<'de> Deserialize<'de> for DidKey {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		s.try_into().map_err(serde::de::Error::custom)
	}
}

impl<T: AsRef<str>> PartialEq<T> for DidKey {
	fn eq(&self, other: &T) -> bool {
		self.0 == other.as_ref()
	}
}

#[derive(Debug, thiserror::Error, Eq, PartialEq, Clone)]
pub enum TryFromStrErr {
	#[error("string did not start with `did:key:z`")]
	WrongPrefix,
}

#[cfg(test)]
pub(crate) mod tests {
	use std::sync::LazyLock;

	use color_eyre::eyre::Context;
	use hex_literal::hex;

	use super::*;

	// From https://datatracker.ietf.org/doc/html/rfc8032#section-7.1
	pub static ED25519_EXAMPLES: LazyLock<Vec<ed25519_dalek::SigningKey>> =
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
	fn test_ed25519_examples() {
		let _examples = &*ED25519_EXAMPLES;
	}

	const GOOD_ED25519: &str =
		"did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp";

	#[test]
	fn test_round_trip() {
		let raw = GOOD_ED25519;
		let parsed: DidKey =
			raw.parse().expect("key is valid so parsing should succeed");
		let tried: DidKey = raw
			.to_owned()
			.try_into()
			.expect("key is valid so parsing should succeed");

		// compare to str
		assert_eq!(parsed, raw);
		assert_eq!(tried, raw);

		// compare to Self
		assert_eq!(parsed, tried);
	}

	#[test]
	fn test_invalid_multibase_prefix_fails() {
		let bad = "did:key:q";
		let parsed = DidKey::from_str(bad);
		let tried = DidKey::try_from(bad.to_owned());
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(parsed, expected);
		assert_eq!(tried, expected);
	}

	#[test]
	fn test_invalid_method_prefix_fails() {
		let bad = "did:foo:z";
		let parsed = DidKey::from_str(bad);
		let tried = DidKey::try_from(bad.to_owned());
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(parsed, expected);
		assert_eq!(tried, expected);
	}

	#[test]
	fn test_empty_str_fails() {
		let bad = "";
		let parsed = DidKey::from_str(bad);
		let tried = DidKey::try_from(bad.to_owned());
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(parsed, expected);
		assert_eq!(tried, expected);
	}

	#[test]
	fn test_display() {
		let key: DidKey = DidKey::from_str(GOOD_ED25519).unwrap();
		assert_eq!(GOOD_ED25519, key.0);
		assert_eq!(GOOD_ED25519, format!("{key}"));
	}

	#[test]
	fn test_serialize() -> color_eyre::Result<()> {
		let _ = color_eyre::install();
		#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
		struct S {
			field: DidKey,
		}

		let original_deserialized = S {
			field: GOOD_ED25519.parse().unwrap(),
		};
		let original_serialized = serde_json::json!({
			"field": GOOD_ED25519,
		});

		let deserialized: S = serde_json::from_value(original_serialized.clone())
			.wrap_err("failed to deserialize")?;
		assert_eq!(
			deserialized, original_deserialized,
			"deserialized should match expected value"
		);

		let serialized =
			serde_json::to_value(deserialized).wrap_err("failed to serialize")?;
		assert_eq!(
			serialized, original_serialized,
			"serialized should match expected value"
		);

		Ok(())
	}
}
