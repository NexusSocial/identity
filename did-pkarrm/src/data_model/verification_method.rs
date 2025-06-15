use std::{fmt::Display, str::FromStr};

use fluent_uri::Uri;

use super::did::{Did, DidFromUriErr};

/// A verification method most typically is a public key (via `did:key`), or a Did Url
/// that links to a verification method in a different Did Document.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum VerificationMethod {
	/// A `did:key`. This does not include the fragment suffix, to save space.
	DidKey(Did),
	/// A reference to a verification method in a remote Did Document. Any method other
	/// than `did:key` can be used.
	///
	/// DidUrls allow the use of verification methods that are controlled by third
	/// parties or with alternative did methods such as did:web. By referencing external
	/// Dids, users can use more convenient third party services while retaining their
	/// ability for credible exit.
	DidUrl(Did),
}

impl VerificationMethod {
	pub fn as_did(&self) -> &Did {
		match self {
			VerificationMethod::DidKey(did) => did,
			VerificationMethod::DidUrl(did) => did,
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ParseVerificationMethodErr {
	#[error("not a uri")]
	NotAUri(#[from] fluent_uri::error::ParseError<String>),
	#[error("did not start with did:")]
	NotADid(#[from] DidFromUriErr),
}

impl FromStr for VerificationMethod {
	type Err = ParseVerificationMethodErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let uri: Uri<String> = Uri::try_from(s.to_owned())?;
		let did = Did::try_from(uri)?;
		Ok(Self::from(did))
	}
}

impl TryFrom<String> for VerificationMethod {
	type Error = ParseVerificationMethodErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let uri: Uri<String> = Uri::try_from(value)?;
		let did = Did::try_from(uri)?;

		Ok(Self::from(did))
	}
}

impl From<Did> for VerificationMethod {
	fn from(value: Did) -> Self {
		let (prefix, _suffix) = value
			.as_uri()
			.path()
			.split_once(':')
			.expect("already checked for did: prefix");

		if prefix == "key" {
			Self::DidKey(value)
		} else {
			Self::DidUrl(value)
		}
	}
}

impl<T: AsRef<str>> PartialEq<T> for VerificationMethod {
	fn eq(&self, other: &T) -> bool {
		self.as_did() == other
	}
}

impl Display for VerificationMethod {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_did())
	}
}
