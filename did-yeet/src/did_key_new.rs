use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct DidKey {
	pub multicodec: u32,
	pub pubkey: Vec<u8>,
}

impl DidKey {
	pub const PREFIX: &'static str = "did:key:z";

	pub fn from_str(s: &str) -> Result<Self, TryFromStrErr> {
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

impl Display for DidKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut out = String::new();
		let mut scratch = Vec::new();
		self.to_str(&mut scratch, &mut out);

		f.write_str(&out)
	}
}

#[derive(Debug, thiserror::Error, Eq, PartialEq, Clone)]
pub enum TryFromStrErr {
	#[error("string did not start with `did:key:z`")]
	WrongPrefix,
	#[error("string was not base58-btc encoded: {0}")]
	NotBase58Btc(#[from] bs58::decode::Error),
	#[error("failed to decode varint for multikey type: {0}")]
	Varint(#[from] unsigned_varint::decode::Error),
}

#[derive(
	Debug,
	Eq,
	PartialEq,
	Copy,
	Clone,
	Hash,
	Ord,
	PartialOrd,
	derive_more::Deref,
	derive_more::DerefMut,
	derive_more::From,
	derive_more::Into,
)]
#[repr(transparent)]
pub struct KnownMultikeys(pub u32);

impl KnownMultikeys {
	pub const ED25519_PUB: Self = Self(0xED);
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::did_key::tests::ED25519_EXAMPLES;

	#[test]
	fn test_ed25519_round_trip() {
		// Arrange
		struct Example {
			dalek: ed25519_dalek::VerifyingKey,
			serialized: String,
		}
		let serialized = [
			"did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw", // Test1
			"did:key:z6MkiaMbhXHNA4eJVCCj8dbzKzTgYDKf6crKgHVHid1F1WCT", // Test2
			"did:key:z6MkwSD8dBdqcXQzKJZQFPy2hh2izzxskndKCjdmC2dBpfME", // Test3
			"did:key:z6Mkh7U7jBwoMro3UeHmXes4tKtFbZhMRWejbtunbU4hhvjP", // Test1024
			"did:key:z6MkvLrkgkeeWeRwktZGShYPiB5YuPkhN2yi3MqMKZMFMgWr", // TestSha
		];
		let examples: Vec<Example> = ED25519_EXAMPLES
			.iter()
			.zip(serialized)
			.map(|(dalek, serialized)| {
				let dalek = dalek.verifying_key();

				Example {
					dalek,
					serialized: serialized.to_owned(),
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
}
