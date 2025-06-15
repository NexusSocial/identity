use std::{fmt::Display, ops::Range, str::FromStr};

use fluent_uri::Uri;

const PREFIX: &str = "did";

/// A Decentralized Identitifer. This is essentially just a uri which can be represented
/// as a string. All DIDs have the form `did:<method>:<method-specific-id>`
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Did {
	uri: Uri<String>,
	method: Range<usize>,
}

impl Did {
	pub fn as_uri(&self) -> &Uri<String> {
		&self.uri
	}
}

impl Display for Did {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_uri())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum DidFromUriErr {
	#[error("did not start with `{PREFIX}:`")]
	WrongPrefix,
	#[error("missing method specific identifier")]
	MissingMethod,
}

impl TryFrom<Uri<String>> for Did {
	type Error = DidFromUriErr;

	fn try_from(value: Uri<String>) -> Result<Self, Self::Error> {
		if value.scheme().as_str() != PREFIX || value.authority().is_some() {
			return Err(DidFromUriErr::WrongPrefix);
		}

		let Some((method, _id)) = value.path().split_once(':') else {
			return Err(DidFromUriErr::MissingMethod);
		};

		let start = "did:".len();
		let method_range = start..(start + method.len());

		debug_assert_eq!(
			value.as_str().get(method_range.clone()),
			Some(method.as_str())
		);

		Ok(Self {
			uri: value,
			method: method_range,
		})
	}
}

#[derive(Debug, thiserror::Error)]
pub enum DidParseErr {
	#[error("not a uri")]
	NotAUri(#[from] fluent_uri::error::ParseError),
	#[error("uri is not a valid DID")]
	UriIsInvalid(#[from] DidFromUriErr),
}

impl FromStr for Did {
	type Err = DidParseErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let uri = Uri::parse(s)?;
		Ok(Did::try_from(uri.to_owned())?)
	}
}

impl<T: AsRef<str>> PartialEq<T> for Did {
	fn eq(&self, other: &T) -> bool {
		self.uri == other.as_ref()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_invalid_prefix_fails() {
		let negative_examples = ["di:not:valid", "did:nomethod"];
		for e in negative_examples {
			assert!(Did::from_str(e).is_err(), "failed example {e}")
		}
	}
}
