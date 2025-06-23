#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::{
	fmt::{Debug, Display},
	str::FromStr,
};

/// A parsed did:key. Does not perform validate the public key.
///
/// See also the [did:key spec][spec].
///
/// # Example
///
/// ```
/// # use did_key::DidKey;
/// /// From did:key spec section 4.1
/// let did_key: DidKey = "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".parse().unwrap();
/// ```
///
/// [spec]: https://w3c-ccg.github.io/did-key-spec
#[derive(Eq, PartialEq, Clone, Hash)]
pub struct DidKey {
	pub multicodec: u32,
	pub pubkey: Vec<u8>,
}

impl DidKey {
	pub const PREFIX: &'static str = "did:key:z";

	/// Encodes as a string. Result written into `out`. `scratch` is used as temporary
	/// scratch space, making it possible to reuse allocations.
	pub fn to_str(&self, scratch: &mut Vec<u8>, out: &mut String) {
		scratch.clear();
		out.clear();
		out.push_str(Self::PREFIX);

		{
			let mut buf = unsigned_varint::encode::u32_buffer();
			let encoded_varint =
				unsigned_varint::encode::u32(self.multicodec, &mut buf);
			scratch.extend(encoded_varint);
		}
		scratch.extend(&self.pubkey);

		bs58::encode::EncodeBuilder::new(scratch, bs58::Alphabet::BITCOIN)
			.onto(out)
			.expect("infallible");
	}
}

#[derive(Debug, thiserror::Error, Eq, PartialEq, Clone)]
pub enum TryFromStrErr {
	#[error("string did not start with `did:key:z`")]
	WrongPrefix,
	#[error("string was not base58-btc encoded: {0}")]
	NotBase58Btc(#[from] bs58::decode::Error),
	#[error("failed to decode varint for pubkey type: {0}")]
	Varint(#[from] unsigned_varint::decode::Error),
}

impl FromStr for DidKey {
	type Err = TryFromStrErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let Some(suffix) = s.strip_prefix(Self::PREFIX) else {
			return Err(TryFromStrErr::WrongPrefix);
		};
		let decoded = bs58::decode(suffix).into_vec()?;
		let (multicodec, pubkey) = unsigned_varint::decode::u32(&decoded)?;

		// PERF: maybe we can reuse the buffer of `decoded` to be more efficient than
		// a clone
		Ok(Self {
			multicodec,
			pubkey: pubkey.to_owned(),
		})
	}
}

impl Debug for DidKey {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let s = self.pubkey.as_slice();
		let e = s.len();
		let mut closure = |pubkey| {
			f.debug_struct(core::any::type_name::<Self>())
				.field("multicodec", &self.multicodec)
				.field("pubkey", &pubkey)
				.finish()
		};
		if e > 8 {
			closure(format_args!(
				"{:x}{:x}{:x}{:x}...{:x}{:x}{:x}{:x}",
				s[0],
				s[1],
				s[2],
				s[3],
				s[e - 4],
				s[e - 3],
				s[e - 2],
				s[e - 1],
			))
		} else {
			closure(format_args!("{:x?}", s))
		}
	}
}

impl Display for DidKey {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let mut out = String::new();
		let mut scratch = Vec::new();
		self.to_str(&mut scratch, &mut out);

		f.write_str(&out)
	}
}

#[cfg(any(feature = "serde", test))]
mod serde_impls {
	use super::*;

	use alloc::format;

	use serde::{Deserialize, Serialize};

	impl<'de> Deserialize<'de> for DidKey {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			let s = String::deserialize(deserializer)?;
			s.parse().map_err(serde::de::Error::custom)
		}
	}

	impl Serialize for DidKey {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			serializer.serialize_str(&format!("{self}"))
		}
	}
}

/// Helpful list of multicodec values for various public key types.
/// See the [multicodec table][multicodec] for more values
///
/// [multicodec]: https://github.com/multiformats/multicodec/blob/master/table.csv
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
#[repr(u32)]
#[non_exhaustive]
pub enum KnownMultikeys {
	Ed25519Pub = 0xED,
}

impl From<KnownMultikeys> for u32 {
	fn from(value: KnownMultikeys) -> Self {
		value as u32
	}
}

// TODO: use a macro to avoid repeating the numerical literal
impl TryFrom<u32> for KnownMultikeys {
	type Error = ();

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		Ok(match value {
			0xED => Self::Ed25519Pub,
			_ => return Err(()),
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;

	use color_eyre::eyre::WrapErr as _;
	use hex_literal::hex;
	use serde::{Deserialize, Serialize};
	use std::sync::LazyLock;

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

	const DID_KEY_EXAMPLES: &[&str] = &[
		"did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw", // Test1
		"did:key:z6MkiaMbhXHNA4eJVCCj8dbzKzTgYDKf6crKgHVHid1F1WCT", // Test2
		"did:key:z6MkwSD8dBdqcXQzKJZQFPy2hh2izzxskndKCjdmC2dBpfME", // Test3
		"did:key:z6Mkh7U7jBwoMro3UeHmXes4tKtFbZhMRWejbtunbU4hhvjP", // Test1024
		"did:key:z6MkvLrkgkeeWeRwktZGShYPiB5YuPkhN2yi3MqMKZMFMgWr", // TestSha
	];

	#[test]
	fn test_ed25519_round_trip() {
		// Arrange
		struct Example {
			dalek: ed25519_dalek::VerifyingKey,
			serialized: String,
		}
		let examples: Vec<Example> = ED25519_EXAMPLES
			.iter()
			.zip(DID_KEY_EXAMPLES)
			.map(|(dalek, serialized)| {
				let dalek = dalek.verifying_key();

				Example {
					dalek,
					serialized: serialized.to_string(),
				}
			})
			.collect();

		// Act + Assert
		let mut sbuf = String::new();
		let mut scratch = Vec::new();
		for Example { dalek, serialized } in examples {
			let deserialized = DidKey::from_str(&serialized)
				.expect("all are valid keys so they should deserialize");

			assert_eq!(
				deserialized,
				DidKey {
					multicodec: 0xED,
					pubkey: dalek.as_bytes().to_vec()
				},
				"deserialization didn't match expected value"
			);

			deserialized.to_str(&mut scratch, &mut sbuf);
			assert_eq!(
				sbuf, serialized,
				"serialization via `to_str` didn't match expected value"
			);
			assert_eq!(
				format!("{deserialized}"),
				serialized,
				"serialization via `Display` didn't match expected value"
			);
		}
	}

	#[test]
	fn test_invalid_multibase_prefix_fails() {
		let bad = "did:key:q";
		let parsed = DidKey::from_str(bad);
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(parsed, expected);
	}

	#[test]
	fn test_invalid_method_prefix_fails() {
		let bad = "did:foo:z";
		let parsed = DidKey::from_str(bad);
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(parsed, expected);
	}

	#[test]
	fn test_empty_str_fails() {
		let bad = "";
		let parsed = DidKey::from_str(bad);
		let expected = Err(TryFromStrErr::WrongPrefix);

		assert_eq!(parsed, expected);
	}

	#[test]
	fn test_serialize() -> color_eyre::Result<()> {
		let _ = color_eyre::install();
		#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
		struct S {
			field: DidKey,
		}

		let original_deserialized = S {
			field: DID_KEY_EXAMPLES[0].parse().unwrap(),
		};
		let original_serialized = serde_json::json!({
			"field": DID_KEY_EXAMPLES[0],
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
