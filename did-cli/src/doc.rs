use std::collections::BTreeSet;

use crate::Uri;

use did_common::{did::Did, did_url::DidUrl};
use did_key::DidKey;

/// For simplicity we are more opinionated about how to normalize a DidDocument.
///
/// Instead of allowing the various verification relationships to directly embed
/// their veritifcation methods, we force them to instead reference
/// `verification_method`.
#[derive(Debug)]
pub struct DidDocument {
	pub id: Did,
	pub also_known_as: Vec<Uri>,
	pub verification_method: Vec<VerificationMethod>,
	pub authentication: BTreeSet<VerificationMethodReference>,
	pub assertion: BTreeSet<VerificationMethodReference>,
}

/// A reference to one of the `verification_method`s.
///
/// Innner number is the index of the verification method
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub struct VerificationMethodReference(pub u16);

/// For simplicity we are more opinionated about how to normalize a VerificationMethod.
///
/// We normalize them to always be an external reference to some other DidDocument's
/// verification method, or a `did:key`. This means that:
/// * Instead of directly exposing Multibase or JsonWebKey verification methods, these
///   are normalized to a did:key to simplify things.
/// * Directly embedding other verification methods are not supported, they must be
///   referenced externally.
#[derive(Debug)]
pub enum VerificationMethod {
	DidKey(DidKey),
	External(DidUrl),
}
