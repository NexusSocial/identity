//! Keychain operations.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod newtypes {
	use super::*;

	#[derive(
		Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
	)]
	pub struct Did(String);

	#[derive(
		Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
	)]
	pub struct DidUrl(String);

	#[derive(
		Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
	)]
	pub struct Signature(Vec<u8>);

	#[derive(
		Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
	)]
	pub struct Hash(String);

	/// Seconds since unix epoch
	#[derive(
		Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
	)]
	pub struct UnixEpoch(pub u64);
}
use self::newtypes::*;

/// Enrolls a group of keys under a parent key.
#[derive(
	Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct Enroll {
	// Only genesis keys can have `None` as the parent, because they are the only keys
	// at the root of the keychain.
	pub parent: Option<DidUrl>,
	pub dids: BTreeMap<Did, DidEnrollment>,
}

#[derive(
	Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct DidEnrollment {
	pub caps: KeyCapabilities,
}

#[derive(
	Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub enum Operation {
	Enroll(Enroll),
	Revoke(Revoke),
}

impl Operation {
	/// Serializes the payload for signing
	pub fn serialize_for_signing(&self, vec: &mut Vec<u8>) {
		vec.clear();
		serde_ipld_dagcbor::to_writer(vec, self).expect("serialization is infallible")
	}
}

#[derive(
	Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct Revoke {
	pub dids: BTreeMap<Did, DidRevocation>,
}

#[derive(
	Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct DidRevocation {
	pub reason: KeyRevocationReason,
	/// Any signatures issued on this date or after should be considered invalid.
	pub sigs_invalid_on: UnixEpoch,
}

#[derive(
	Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct Operations(BTreeMap<Hash, OperationEntry>);

#[derive(
	Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
struct OperationEntry {
	pub op: Operation,
	/// Signs the hash of the serialized operation.
	pub sig: Signature,
}

use bitflags::bitflags;

bitflags! {
	/// The capabilities of a key in the hierarchy. Root keys implicitly can
	/// do all of
	/// these.
	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
	#[serde(transparent)]
	pub struct KeyCapabilities: u8 {
		/// The key can enroll child keys
		const EnrollChildren = 0b00000001;
		/// Can revoke sibling keys (keys at same depth of hierarchy)
		const RevokeSibling = 0b00000010;
		/// Can edit the DID document
		const EditDoc = 0b00000100;
	}

	/// The reason keys were revoked
	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
	#[serde(transparent)]
	pub struct KeyRevocationReason: u8 {
		/// This key has been deleted and is no longer in use, for unspecified reasons.
		const UNSPECIFIED = 0b00000000;
		/// The private key was leaked or compromised, any past signatures can no longer
		/// be trusted.
		const COMPROMISED = 0b00000001;
		/// The key was associated with a controller/entity that we have rescinded
		/// custody from. For example, maybe this key was controlled by a service
		/// provider we no longer wish to use.
		const RESCIND_CUSTODY = 0b00000010;
	}
}

#[cfg(test)]
mod test {
	use crate::{did_key::tests::ED25519_EXAMPLES, DidKey};

	use super::*;

	#[test]
	fn test_serialize_enroll_genesis() {
		let keys: Vec<DidKey> = ED25519_EXAMPLES
			.iter()
			.map(|key| {
				DidKey::from_base58_btc_encoded(
					&bs58::encode(ED25519_EXAMPLES[0].verifying_key().as_bytes())
						.into_string(),
				)
			})
			.collect();
		let enroll = Enroll {
			parent: None,
			dids: BTreeMap::from([]),
		};
	}
}
