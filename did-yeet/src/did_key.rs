use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize};

/// A did:key string. Does not perform base58 decoding or validate the public key.
///
/// See also the [did:key spec][spec].
///
/// # Example
///
/// ```
/// # use did_yeet::DidKey;
/// /// From did:key spec section 4.1
/// let did_key: DidKey = "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".try_into().unwrap();
/// ```
///
/// [spec]: https://w3c-ccg.github.io/did-key-spec
#[derive(Debug, Clone, Eq, Hash, derive_more::Display, Serialize)]
#[serde(transparent)]
pub struct DidKey<'a>(Cow<'a, str>);

impl<'a> DidKey<'a> {
	pub const PREFIX: &'static str = "did:key:z";

	/// Construct a `DidKey` from [base58-btc](bs58) encoded data.
	pub fn from_base58_btc_encoded(data: &str) -> Self {
		Self(Cow::Owned(format!("{}{data}", Self::PREFIX)))
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

	pub fn deserialize_zero_copy_slice<'de: 'a, D>(
		deserializer: D,
	) -> Result<Cow<'a, [Self]>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let cowslice: Vec<&'de str> = Deserialize::deserialize(deserializer)?;

		let result: Vec<DidKey> = cowslice
			.into_iter()
			.map(DidKey::try_from)
			.collect::<Result<_, _>>()
			.map_err(serde::de::Error::custom)?;

		Ok(Cow::Owned(result))
	}
}

impl<'a> From<DidKey<'a>> for Cow<'a, str> {
	fn from(value: DidKey<'a>) -> Self {
		value.0
	}
}

impl<'a> TryFrom<Cow<'a, str>> for DidKey<'a> {
	type Error = TryFromStrErr;

	fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
		if !value.starts_with(Self::PREFIX) {
			return Err(TryFromStrErr::WrongPrefix);
		}

		Ok(Self(value))
	}
}

impl<'a> TryFrom<&'a str> for DidKey<'a> {
	type Error = TryFromStrErr;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		Cow::Borrowed(value).try_into()
	}
}

impl TryFrom<String> for DidKey<'static> {
	type Error = TryFromStrErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Cow::<'_, str>::Owned(value).try_into()
	}
}

// The less efficient, non-zero-copy implementation.
// NOTE: Because we deserialize owned anyway, the return type lifetime is fully decoupled from the
// deserializer lifetime.
impl<'a, 'de> Deserialize<'de> for DidKey<'a> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		s.try_into().map_err(serde::de::Error::custom)
	}
}

impl<T: AsRef<str>> PartialEq<T> for DidKey<'_> {
	fn eq(&self, other: &T) -> bool {
		self.0 == other.as_ref()
	}
}

impl AsRef<str> for DidKey<'_> {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
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
		let borrowed: DidKey = raw
			.try_into()
			.expect("key is valid so parsing should succeed");
		let owned: DidKey = raw
			.to_owned()
			.try_into()
			.expect("key is valid so parsing should succeed");
		let cow: DidKey = Cow::Borrowed(raw)
			.try_into()
			.expect("key is valid so parsing should succeed");

		// compare to str
		assert_eq!(borrowed, raw);
		assert_eq!(owned, raw);
		assert_eq!(cow, raw);

		// compare to Self
		assert_eq!(borrowed, owned);
		assert_eq!(owned, cow);
		assert_eq!(cow, borrowed);
	}

	#[test]
	fn test_invalid_multibase_prefix_fails() {
		let bad = "did:key:q";
		let borrowed = DidKey::try_from(bad);
		let owned = DidKey::try_from(bad.to_owned());
		let cow = DidKey::try_from(Cow::Borrowed(bad));
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(borrowed, expected);
		assert_eq!(owned, expected);
		assert_eq!(cow, expected);
	}

	#[test]
	fn test_invalid_method_prefix_fails() {
		let bad = "did:foo:z";
		let borrowed = DidKey::try_from(bad);
		let owned = DidKey::try_from(bad.to_owned());
		let cow = DidKey::try_from(Cow::Borrowed(bad));
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(borrowed, expected);
		assert_eq!(owned, expected);
		assert_eq!(cow, expected);
	}

	#[test]
	fn test_empty_str_fails() {
		let bad = "";
		let borrowed = DidKey::try_from(bad);
		let owned = DidKey::try_from(bad.to_owned());
		let cow = DidKey::try_from(Cow::Borrowed(bad));
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(borrowed, expected);
		assert_eq!(owned, expected);
		assert_eq!(cow, expected);
	}

	#[test]
	fn test_display() {
		let key: DidKey = DidKey::try_from(GOOD_ED25519).unwrap();
		assert_eq!(GOOD_ED25519, key.0);
		assert_eq!(GOOD_ED25519, format!("{key}"));
	}

	#[test]
	fn test_serialize() {
		#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
		struct S {
			field: DidKey<'static>,
		}

		let original_deserialized = S {
			field: GOOD_ED25519.try_into().unwrap(),
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
