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
	pub fn is_known(&self) -> bool {
		(Self::V0.0..=Self::V0.0).contains(&self.0)
	}

	pub const V0: KeychainVersion = KeychainVersion(0);
}

/// A view on a [`Keychain`] of its initial, "genesis" state.
#[derive(Debug, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct GenesisKeychain<'a> {
	/// Version.
	v: KeychainVersion,
	/// Index of vec corresponds to key in KeyEntries. Signatures are of
	/// CBOR-serialized Keychain with empty values for `sigs` and `children`.
	#[serde(borrow)]
	sigs: Cow<'a, [Signature]>,
	/// Key ID comes from index in vec.
	#[serde(borrow)]
	keys: Cow<'a, [DidKey]>,
}

impl GenesisKeychain<'_> {
	pub fn v(&self) -> KeychainVersion {
		self.v
	}

	pub fn sigs(&self) -> &[Signature] {
		self.sigs.as_ref()
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
		self.v == other.v && self.sigs == other.sigs && self.keys == other.keys
	}
}

impl PartialEq<Keychain> for GenesisKeychain<'_> {
	fn eq(&self, other: &Keychain) -> bool {
		other == self
	}
}

#[derive(Debug, thiserror::Error)]
pub enum TryIntoGenesisErr {
	#[error("keychain has children so it is not convertible losslessly")]
	HasChildKeys,
	#[error("keychain has `sigs.len()` ({n_sigs}) and `keys.len()` ({n_keys}) but genesis keychains always have equal numbers of these")]
	NumSigsDontMatchNumKeys { n_sigs: usize, n_keys: usize },
}

/// The keychain that underpins the permissions of mutation of the document
// TODO: Turn all Cows into regular vecs.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Keychain {
	/// Version.
	v: KeychainVersion,
	/// Key ID comes from index in vec.
	keys: Vec<DidKey>,
	/// Signatures that enroll each key. Index of vec corresponds to key in `keys`.
	/// Genesis keys are signed by themselves, and child keys are signed by their
	/// parent key.
	/// Signatures are are of CBOR-serialized Keychain with empty values for `sigs` and with
	/// `children` not present.
	// TODO: Consider directly placing the signature next to the data it protects?
	sigs: Vec<Signature>,
	/// Information about child keys. `KeyId` is `index_in_vec + self.n_root_keys()`
	children: Vec<ChildKeyInfo>,
}

impl Keychain {
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
			sigs: Cow::Borrowed(&self.sigs[..self.n_root_keys()]),
			// TODO: Consider making this not panic
			keys: Cow::Borrowed(&self.keys[..self.n_root_keys()]),
		}
	}
}

impl PartialEq<GenesisKeychain<'_>> for Keychain {
	fn eq(&self, other: &GenesisKeychain<'_>) -> bool {
		self.v == other.v
			&& self.sigs.as_slice() == other.sigs.as_ref()
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
	/// The key id of this child key
	pub kid: KeyId,
	/// The parent key that enrolled this child key
	pub parent: KeyId,
	pub revoked_by: Option<KeyId>,
	pub caps: KeyCapabilities,
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

	/// The capabilities of a key in the hierarchy. Root keys implicitly can
	/// do all of
	/// these.
	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
	#[serde(transparent)]
	pub struct KeyRevocationReason: u8 {
		/// This key has been deleted and is no longer in use, for unspecified reasons.
		const UNSPECIFIED = 0b00000000;
		/// The private key was leaked or compromised, any past signatures can no longer
		/// be trusted. When a key is marked as compromised, all of its children should
		/// be treated as compromised too.
		const COMPROMISED = 0b00000001;
		/// The key was associated with a controller/entity that we have rescinded
		/// custody from. For example, maybe this key was controlled by a service
		/// provider we no longer wish to use.
		const RESCIND_CUSTODY = 0b00000010;
	}
}

#[cfg(test)]
mod tests {
	use std::{str::FromStr, sync::LazyLock};

	use ed25519_dalek::{Signer, VerifyingKey};
	use hex_literal::hex;

	use super::*;
	use crate::signature::Signature;

	#[test]
	fn genesis_keychain_and_keychain_without_children_equivalent() {
		let keys = vec![
			DidKey::from_base58_btc_encoded("foobar"),
			DidKey::from_base58_btc_encoded("baz"),
		];
		let sigs = vec![
			Signature(vec![69; 8]),
			Signature(vec![0xDE, 0xAD, 0xBE, 0xEF]),
		];
		let v = KeychainVersion::V0;
		let genesis: GenesisKeychain<'static> = GenesisKeychain {
			v,
			sigs: Cow::Owned(sigs.clone()),
			keys: Cow::Owned(keys.clone()),
		};
		let regular: Keychain = Keychain {
			v,
			sigs,
			keys,
			children: Default::default(),
		};

		assert_eq!(regular, genesis, "equality without conversion");
		assert_eq!(regular.as_genesis(), genesis, "equality after to_genesis");
	}

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

	// Example keys from https://w3c-ccg.github.io/did-key-spec/#ed25519
	static DID_KEY_ED25519_EXAMPLES: LazyLock<Vec<DidKey>> = LazyLock::new(|| {
		[
			"did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
			"did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
			"did:key:z6MknGc3ocHs3zdPiJbnaaqDi58NGb4pk1Sp9WxWufuXSdxf",
		]
		.into_iter()
		.map(|k| DidKey::from_str(k).expect("these are all valid did:key"))
		.collect()
	});

	fn keychain_dummy_sigs() -> (Keychain, Vec<ed25519_dalek::SigningKey>) {
		let did_keys: Vec<(DidKey, ed25519_dalek::SigningKey)> = ED25519_EXAMPLES
			.iter()
			.map(|private| {
				let did = DidKey::from_base58_btc_encoded(
					&bs58::encode(private.verifying_key().as_bytes()).into_string(),
				);
				(did, private.to_owned())
			})
			.collect();

		let keychain = Keychain {
			v: KeychainVersion::V0,
			keys: did_keys
				.iter()
				.map(|(did, _dalek)| did.to_owned())
				.collect(),
			sigs: did_keys
				.iter()
				.map(|(_did, dalek)| Signature(dalek.sign(b"dummy").to_vec()))
				.collect(),
			children: vec![],
		};

		(
			keychain,
			did_keys.into_iter().map(|(_did, dalek)| dalek).collect(),
		)
	}

	#[test]
	#[ignore = "TODO"]
	fn test_genesis_serialization() {
		todo!()
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
