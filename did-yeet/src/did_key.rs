use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A did:key string. Does not perform base58 decoding or validate the public key.
///
/// See also the [did:key spec][spec].
///
/// # Example
///
/// ```
/// /// From did:key spec section 4.1
/// let did_key: DidKey = "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".parse().unwrap();
/// assert_eq!()
/// ```
///
/// [spec]: https://w3c-ccg.github.io/did-key-spec
#[derive(Debug, Clone, Eq, Hash, derive_more::Display, Serialize)]
#[serde(transparent)]
pub struct DidKey(String);

impl DidKey {
	pub const PREFIX: &str = "did:key:z";

	/// Construct a `DidKey` from [base58-btc](bs58) encoded data.
	pub fn from_base58_btc_encoded(data: &str) -> Self {
		Self(format!("{}{data}", Self::PREFIX))
	}

	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl From<DidKey> for String {
	fn from(value: DidKey) -> Self {
		value.0
	}
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
		if !s.starts_with(Self::PREFIX) {
			return Err(TryFromStrErr::WrongPrefix);
		}

		Ok(Self(s.to_owned()))
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

impl AsRef<str> for DidKey {
	fn as_ref(&self) -> &str {
		self.0.as_str()
	}
}

#[derive(Debug, thiserror::Error, Eq, PartialEq, Clone)]
pub enum TryFromStrErr {
	#[error("string did not start with `did:key:z`")]
	WrongPrefix,
}

#[cfg(test)]
mod tests {
	use super::*;

	const GOOD_ED25519: &str =
		"did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp";

	#[test]
	fn test_round_trip() {
		let raw = GOOD_ED25519;
		let parsed: DidKey =
			raw.parse().expect("key is valid so parsing should succeed");
		let try_from: DidKey = raw
			.to_owned()
			.try_into()
			.expect("key is valid so parsing should succeed");

		// all three values equal
		assert_eq!(parsed, try_from);
		assert_eq!(parsed, raw);
		assert_eq!(try_from, raw);

		assert_eq!(raw, parsed.as_str());
		assert_eq!(raw, parsed.as_ref());

		assert_eq!(raw, String::from(parsed));
	}

	#[test]
	fn test_invalid_multibase_prefix_fails() {
		let bad = "did:key:q";
		assert_eq!(bad.parse::<DidKey>(), Err(TryFromStrErr::WrongPrefix));
		assert_eq!(
			DidKey::try_from(bad.to_owned()),
			Err(TryFromStrErr::WrongPrefix)
		);
	}

	#[test]
	fn test_invalid_method_prefix_fails() {
		let bad = "did:foo:z";
		assert_eq!(bad.parse::<DidKey>(), Err(TryFromStrErr::WrongPrefix));
		assert_eq!(
			DidKey::try_from(bad.to_owned()),
			Err(TryFromStrErr::WrongPrefix)
		);
	}

	#[test]
	fn test_empty_str_fails() {
		let bad = "";
		assert_eq!(bad.parse::<DidKey>(), Err(TryFromStrErr::WrongPrefix));
		assert_eq!(
			DidKey::try_from(bad.to_owned()),
			Err(TryFromStrErr::WrongPrefix)
		);
	}

	#[test]
	fn test_display() {
		let key: DidKey = GOOD_ED25519.parse().unwrap();
		assert_eq!(GOOD_ED25519, key.0);
		assert_eq!(GOOD_ED25519, format!("{key}"));
	}

	#[test]
	fn test_serialize() {
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

		let deserialized: S =
			serde_json::from_value(original_serialized.clone()).unwrap();
		assert_eq!(
			deserialized, original_deserialized,
			"deserialized should match expected value"
		);

		let serialized = serde_json::to_value(deserialized).unwrap();
		assert_eq!(
			serialized, original_serialized,
			"serialized should match expected value"
		);
	}
}
