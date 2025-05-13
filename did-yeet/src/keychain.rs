use serde::{Deserialize, Serialize};

pub const SHA256_HASH_LEN: usize = 32;

pub struct GenesisHash(pub [u8; SHA256_HASH_LEN]);

impl GenesisHash {
	/// The raw bytes of the hash
	pub fn as_raw(&self) -> &[u8; SHA256_HASH_LEN] {
		&self.0
	}

	// pub fn multihash(&self) -> &[u8] {}

	// pub fn method_specific_id(&self) -> String {}
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
