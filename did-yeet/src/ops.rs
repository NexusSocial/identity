//! Keychain operations.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::keychain::{KeyCapabilities, KeyRevocationReason};

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
