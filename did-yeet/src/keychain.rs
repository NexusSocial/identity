use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use sha2::{Digest, Sha256};

pub const SHA256_HASH_LEN: usize = 32;

#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct GenesisKeychain {
	/// Version.
	pub v: u8,
	/// Index of vec corresponds to key in KeyEntries. Signatures are of
	/// CBOR-serialized Keychain with empty values for `sigs` and `children`.
	pub gsigs: Vec<Signature>,
	/// Key ID comes from index in vec.
	pub keys: Vec<Key>,
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

/// The keychain that underpins the permissions of mutation of the document
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Keychain {
	/// Version.
	pub v: u8,
	/// Genesis signatures. Index of vec corresponds to key in KeyEntries. Signatures
	/// are of CBOR-serialized Keychain with empty values for `gsigs` and with
	/// `children` not present.
	pub gsigs: Vec<Signature>,
	/// Key ID comes from index in vec.
	pub keys: Vec<Key>,
	/// Information about child keys.
	pub children: BTreeMap<KeyId, ChildKeyInfo>,
}

impl Keychain {
	/// KeyId corresponds to position in slice.
	pub fn root_keys(&self) -> &[Key] {
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
	/// ```ignore
	/// sha256(serialized_genesis_keychain)
	/// ```
	pub fn hash(&self) -> KeychainHash {
		let genesis = self.to_genesis();
		let mut hasher = Sha256::new();
		serde_ipld_dagcbor::to_writer(&mut hasher, &genesis).expect("infallible");

		KeychainHash(hasher.finalize().into())
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

/// did:key
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Key(pub String);

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
