use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{signature::Signature, DidKey};

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

/// A view on a [`Keychain`] of its initial, "genesis" state.
#[derive(Debug, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct GenesisKeychain<'a> {
	/// Version.
	v: KeychainVersion,
	/// Index of vec corresponds to key in KeyEntries. Signatures are of
	/// CBOR-serialized Keychain with empty values for `sigs` and `children`.
	// #[serde(deserialize_with = "Signature::deserialize_zero_copy_slice")]
	#[serde(borrow)]
	gsigs: Cow<'a, [Signature<'a>]>,
	/// Key ID comes from index in vec.
	// #[serde(deserialize_with = "DidKey::deserialize_zero_copy_slice")]
	#[serde(borrow)]
	keys: Cow<'a, [DidKey<'a>]>,
}

impl GenesisKeychain<'_> {
	pub fn v(&self) -> KeychainVersion {
		self.v
	}

	pub fn gsigs(&self) -> &[Signature] {
		self.gsigs.as_ref()
	}

	pub fn keys(&self) -> &[DidKey] {
		self.keys.as_ref()
	}

	/// Compute the sha256 hash of the genesis state.
	/// ```text
	/// sha256(serialized_genesis_keychain)
	/// ```
	pub fn hash(&self) -> GenesisHash {
		let mut hasher = Sha256::new();
		serde_ipld_dagcbor::to_writer(&mut hasher, self).expect("infallible");

		GenesisHash(hasher.finalize().into())
	}
}

/// We do this manually instead of deriving it because the derived one doesn't support
/// diffrerent lifetimes
impl PartialEq<GenesisKeychain<'_>> for GenesisKeychain<'_> {
	fn eq(&self, other: &GenesisKeychain<'_>) -> bool {
		self.v == other.v && self.gsigs == other.gsigs && self.keys == other.keys
	}
}

impl PartialEq<Keychain<'_>> for GenesisKeychain<'_> {
	fn eq(&self, other: &Keychain<'_>) -> bool {
		other == self
	}
}

#[derive(Debug, thiserror::Error)]
pub enum TryIntoGenesisErr {
	#[error("keychain has children so it is not convertible losslessly")]
	HasChildKeys,
	#[error("keychain has `gsigs.len()` ({n_sigs}) and `keys.len()` ({n_keys}) but genesis keychains always have equal numbers of these")]
	NumSigsDontMatchNumKeys { n_sigs: usize, n_keys: usize },
}

/// The keychain that underpins the permissions of mutation of the document
// TODO: Turn all Cows into regular vecs.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Keychain<'a> {
	/// Version.
	v: KeychainVersion,
	/// Key ID comes from index in vec.
	#[serde(borrow)]
	keys: Vec<DidKey<'a>>,
	/// Signatures that enroll each key. Index of vec corresponds to key in `keys`.
	/// Genesis keys are signed by themselves, and child keys are signed by their
	/// parent key.
	/// Signatures are are of CBOR-serialized Keychain with empty values for `gsigs` and with
	/// `children` not present.
	#[serde(borrow)]
	sigs: Vec<Signature<'a>>,
	/// Information about child keys. `KeyId` is `index_in_vec + self.n_root_keys()`
	children: Vec<ChildKeyInfo>,
}

impl<'a> Keychain<'a> {
	/// KeyId corresponds to position in slice.
	pub fn root_keys(&self) -> &[DidKey] {
		&self.keys[..self.n_root_keys()]
	}

	#[inline]
	pub fn n_root_keys(&self) -> usize {
		self.keys.len() - self.children.len()
	}

	/// Get a version of the keychain in the state it was at it's genesis.
	///
	/// May panic if the keychain is invalid to begin with.
	pub fn as_genesis(&self) -> GenesisKeychain<'_> {
		GenesisKeychain {
			v: self.v,
			gsigs: Cow::Borrowed(&self.sigs[..self.n_root_keys()]),
			// TODO: Consider making this not panic
			keys: Cow::Borrowed(&self.keys[..self.n_root_keys()]),
		}
	}
}

impl PartialEq<GenesisKeychain<'_>> for Keychain<'_> {
	fn eq(&self, other: &GenesisKeychain<'_>) -> bool {
		self.v == other.v
			&& self.sigs.as_slice() == other.gsigs.as_ref()
			&& self.keys.as_slice() == other.keys.as_ref()
			&& self.children.is_empty()
	}
}

pub struct GenesisHash(pub [u8; SHA256_HASH_LEN]);

impl GenesisHash {
	/// The raw bytes of the hash
	pub fn as_raw(&self) -> &[u8; SHA256_HASH_LEN] {
		&self.0
	}

	// pub fn multihash(&self) -> &[u8] {}

	// pub fn method_specific_id(&self) -> String {}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ChildKeyInfo {
	pub parent: KeyId,
	pub revoked_by: Option<KeyId>,
	pub capabilities: KeyCapabilities,
}

#[derive(
	Debug,
	Copy,
	Clone,
	Eq,
	PartialEq,
	Hash,
	Serialize,
	Deserialize,
	Ord,
	PartialOrd,
	derive_more::Display,
)]
#[serde(transparent)]
pub struct KeyId(pub u8);

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
	use crate::signature::Signature;

	use super::*;

	#[test]
	fn genesis_keychain_and_keychain_without_children_equivalent() {
		let keys = vec![
			DidKey::from_base58_btc_encoded("foobar"),
			DidKey::from_base58_btc_encoded("baz"),
		];
		let sigs = vec![
			Signature(Cow::Owned(vec![69; 8])),
			Signature(Cow::Owned(vec![0xDE, 0xAD, 0xBE, 0xEF])),
		];
		let v = KeychainVersion::V0;
		let genesis: GenesisKeychain<'static> = GenesisKeychain {
			v,
			gsigs: Cow::Owned(sigs.clone()),
			keys: Cow::Owned(keys.clone()),
		};
		let regular: Keychain<'static> = Keychain {
			v,
			sigs,
			keys,
			children: Default::default(),
		};

		assert_eq!(regular, genesis, "equality without conversion");
		assert_eq!(regular.as_genesis(), genesis, "equality after to_genesis");
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
