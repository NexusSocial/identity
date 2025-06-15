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

	/// Gets the method in `did:<method>:<method-specific-id>`
	pub fn method(&self) -> &str {
		&self.uri.as_str()[self.method.clone()]
	}

	/// Gets the method specific identifier in `did:<method>:<method-specific-id>`
	/// TODO: Currently reports fragment but this is wrong.
	pub fn method_specific_id(&self) -> &str {
		let suffix = (self.method.end + 1)..;
		&self.uri.as_str()[suffix]
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
	#[error("method specific id was empty")]
	EmptyMethodSpecificId,
}

impl TryFrom<Uri<String>> for Did {
	type Error = DidFromUriErr;

	fn try_from(value: Uri<String>) -> Result<Self, Self::Error> {
		if value.scheme().as_str() != PREFIX || value.authority().is_some() {
			return Err(DidFromUriErr::WrongPrefix);
		}

		let Some((method, id)) = value.path().split_once(':') else {
			return Err(DidFromUriErr::MissingMethod);
		};
		if id.is_empty() {
			return Err(DidFromUriErr::EmptyMethodSpecificId);
		}

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
pub(crate) mod test {
	use super::*;

	// From https://datatracker.ietf.org/doc/html/rfc8032#section-7.1
	pub(crate) const DID_KEY_EXAMPLES: &[&str] = &[
		"did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw", // Test1
		"did:key:z6MkiaMbhXHNA4eJVCCj8dbzKzTgYDKf6crKgHVHid1F1WCT", // Test2
		"did:key:z6MkwSD8dBdqcXQzKJZQFPy2hh2izzxskndKCjdmC2dBpfME", // Test3
		"did:key:z6Mkh7U7jBwoMro3UeHmXes4tKtFbZhMRWejbtunbU4hhvjP", // Test1024
		"did:key:z6MkvLrkgkeeWeRwktZGShYPiB5YuPkhN2yi3MqMKZMFMgWr", // TestSha
	];

	// From: https://w3c-ccg.github.io/did-method-web/#example-example-web-method-dids
	pub(crate) const DID_WEB_EXAMPLES: &[&str] = &[
		"did:web:w3c-ccg.github.io",
		"did:web:w3c-ccg.github.io:user:alice",
		"did:web:example.com%3A3000",
	];

	#[test]
	fn test_invalid_prefix_fails() {
		let negative_cases = ["di:not:valid", "did:nomethod"];
		for e in negative_cases {
			assert!(Did::from_str(e).is_err(), "failed example {e}")
		}
	}

	#[test]
	fn test_method_specific_parts() {
		for e in DID_KEY_EXAMPLES {
			let did = Did::from_str(e).expect(e);
			assert_eq!(did.method(), "key", "method was incorrect");
			assert_eq!(
				did.method_specific_id(),
				e.strip_prefix("did:key:").unwrap(),
				"method specific id was incorrect"
			)
		}
	}
}
