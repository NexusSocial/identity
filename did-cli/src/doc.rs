use std::{collections::BTreeSet, fmt::Display};

use crate::Uri;

use did_common::{did::Did, did_url::DidUrl};

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
	pub authentication: BTreeSet<DidUrlFragment>,
	pub assertion: BTreeSet<DidUrlFragment>,
}

/// The `#foobar` suffix of a DidUrl.
#[derive(Debug)]
pub struct DidUrlFragment(String);

impl Display for DidUrlFragment {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "#{}", self.0)
	}
}

/// For simplicity wer are more opinionated about how to normalize a VerificationMethod.
///
/// We normalize them to always be external reference to some other DidDocument's
/// verification method, or a `did:key`. This means that:
/// * Instead of directly exposing Multibase or JsonWebKey verification methods, these
/// are normalized to a did:key to simplify things.
/// * Directly embedding other verification methods are not supported.
#[derive(Debug)]
pub enum VerificationMethod {
	DidKey(Did),
	Reference(DidUrl),
}
