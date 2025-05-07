use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::DidKey;

pub const SHA256_HASH_LEN: usize = 32;

#[derive(
	Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeychainVersion(u8);
impl KeychainVersion {
	pub const V0: KeychainVersion = KeychainVersion(0);
}

#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct GenesisKeychain<S = Signature, K = DidKey> {
	/// Version.
	pub v: KeychainVersion,
	/// Index of vec corresponds to key in KeyEntries. Signatures are of
	/// CBOR-serialized Keychain with empty values for `sigs` and `children`.
	pub gsigs: Vec<S>,
	/// Key ID comes from index in vec.
	pub keys: Vec<K>,
}

impl PartialEq<Keychain> for GenesisKeychain {
	fn eq(&self, other: &Keychain) -> bool {
		other == self
	}
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidKeychainErr {
	#[error("keychain had extraneous keys")]
	TooManyKeys,
	#[error("keychain had too few keys")]
	TooFewKeys,
}

#[derive(Debug, thiserror::Error)]
pub enum TryIntoGenesisErr {
	#[error("keychain is malformed: {0}")]
	Invalid(#[from] InvalidKeychainErr),
	#[error("keychain has children so it is not convertible losslessly")]
	HasChildKeys,
}

impl TryFrom<Keychain> for GenesisKeychain {
	type Error = TryIntoGenesisErr;

	fn try_from(value: Keychain) -> Result<Self, Self::Error> {
		if !value.children.is_empty() {
			return Err(TryIntoGenesisErr::HasChildKeys);
		}

		// 1. Validate keys.len >= gsigs.len()
		// 2. validate all KeyIDs in children < keys.len()
		// 3. Validate all keys are DidKey prefix.
		// 4. Validate that root keys don't appear in children
		todo!("validate rest of keychain");
	}
}

/// The keychain that underpins the permissions of mutation of the document
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Keychain {
	/// Version.
	pub v: KeychainVersion,
	/// Genesis signatures. Index of vec corresponds to key in KeyEntries. Signatures
	/// are of CBOR-serialized Keychain with empty values for `gsigs` and with
	/// `children` not present.
	pub gsigs: Vec<Signature>,
	/// Key ID comes from index in vec.
	pub keys: Vec<DidKey>,
	/// Information about child keys.
	pub children: BTreeMap<KeyId, ChildKeyInfo>,
}

impl Keychain {
	/// KeyId corresponds to position in slice.
	pub fn root_keys(&self) -> &[DidKey] {
		&self.keys[..self.gsigs.len()]
	}

	/// Get a version of the keychain in the state it was at it's genesis.
	pub fn to_genesis(&self) -> GenesisKeychain {
		GenesisKeychain {
			v: self.v,
			gsigs: self.gsigs.clone(),
			keys: self.keys[..self.gsigs.len()].to_vec(),
		}
	}

	/// Compute the sha256 hash of the genesis state.
	/// ```text
	/// sha256(serialized_genesis_keychain)
	/// ```
	pub fn hash(&self) -> KeychainHash {
		let genesis = self.to_genesis();
		let mut hasher = Sha256::new();
		serde_ipld_dagcbor::to_writer(&mut hasher, &genesis).expect("infallible");

		KeychainHash(hasher.finalize().into())
	}
}

impl From<GenesisKeychain> for Keychain {
	fn from(value: GenesisKeychain) -> Self {
		Keychain {
			v: value.v,
			gsigs: value.gsigs,
			keys: value.keys,
			children: BTreeMap::new(),
		}
	}
}

impl PartialEq<GenesisKeychain> for Keychain {
	fn eq(&self, other: &GenesisKeychain) -> bool {
		self.v == other.v
			&& self.gsigs == other.gsigs
			&& self.keys == other.keys
			&& self.children.is_empty()
	}
}

pub struct KeychainHash(pub [u8; SHA256_HASH_LEN]);

impl KeychainHash {
	/// The raw bytes of the hash
	pub fn as_raw(&self) -> &[u8; SHA256_HASH_LEN] {
		&self.0
	}

	// pub fn multihash(&self) -> &[u8] {}

	// pub fn method_specific_id(&self) -> String {}
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ChildKeyInfo {
	pub parent: KeyId,
	pub revoked_by: Option<KeyId>,
	pub capabilities: KeyCapabilities,
	pub sig: Signature,
}

#[derive(
	Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd,
)]
#[serde(transparent)]
pub struct KeyId(pub u8);

/// Signature bytes
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Signature(pub Vec<u8>);

use bitflags::bitflags;

bitflags! {
	/// The capabilities of a key in the hierarchy. Root keys implicitly can
	/// do all of
	/// these.
	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
	#[serde(transparent)]
	pub struct KeyCapabilities: u8 {
		/// The key can enroll child keys
		const EnrollChildren = 0b00000001;
		/// Can revoke sibling keys (keys at same depth of hierarchy)
		const RevokeSibling = 0b00000010;
		/// Can edit the DID document
		const EditDoc = 0b00000100;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn genesis_keychain_and_keychain_without_children_equivalent() {
		let keys = vec![
			DidKey::from_base58_btc_encoded("foobar"),
			DidKey::from_base58_btc_encoded("baz"),
		];
		let gsigs = vec![
			Signature(vec![69; 8]),
			Signature(vec![0xDE, 0xAD, 0xBE, 0xEF]),
		];
		let v = KeychainVersion::V0;
		let genesis = GenesisKeychain {
			v,
			gsigs: gsigs.clone(),
			keys: keys.clone(),
		};
		let regular = Keychain {
			v,
			gsigs,
			keys,
			children: Default::default(),
		};

		assert_eq!(regular, genesis, "equality without conversion");
		assert_eq!(regular.to_genesis(), genesis, "equality after to_genesis");
		assert_eq!(
			GenesisKeychain::try_from(regular)
				.expect("conversion should always work for this example"),
			genesis
		)
	}

	#[test]
	#[ignore = "TODO"]
	fn test_hash_against_known_values() {
		todo!()
	}

	#[test]
	#[ignore = "TODO"]
	fn test_root_keys() {
		todo!()
	}
}
