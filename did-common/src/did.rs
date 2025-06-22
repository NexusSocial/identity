use alloc::{borrow::ToOwned as _, string::String};
use core::{ops::RangeFrom, str::FromStr};

use crate::uri::{NotAUriErr, Uri};

// TODO: Do we want to allow for borrowing strings?

/// A [Decentralized Identifier][did-syntax]. This is a globally unique identifier that can be
/// resolved to a DID Document.
///
/// [did-syntax]: https://www.w3.org/TR/did-1.1/#did-syntax
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Did {
	uri: Uri,
	// u16 instead of usize because identifiers cant be that long anyway
	method_specific_id: RangeFrom<u16>,
}

impl Did {
	pub fn as_str(&self) -> &str {
		self.uri.as_str()
	}

	#[cfg(feature = "uri")]
	pub fn as_uri(&self) -> &fluent_uri::Uri<String> {
		self.uri.as_fluent()
	}

	/// The method for this did.
	///
	/// ```
	/// # use did_common::did::Did;
	/// # use std::str::FromStr;
	/// assert_eq!(Did::from_str("did:example:foobar").unwrap().method(), "example");
	/// ```
	pub fn method(&self) -> &str {
		const START_IDX: usize = "did:".len() as _;
		let method_range = START_IDX..usize::from(self.method_specific_id.start - 1);
		&self.uri.as_str()[method_range]
	}

	pub fn method_specific_id(&self) -> &str {
		&self.uri.as_str()[usize::from(self.method_specific_id.start)..]
	}
}

impl PartialOrd for Did {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl AsRef<str> for Did {
	fn as_ref(&self) -> &str {
		self.uri.as_str()
	}
}

impl Ord for Did {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.uri.cmp(&other.uri)
	}
}

impl PartialEq<str> for Did {
	fn eq(&self, other: &str) -> bool {
		self.uri.as_str() == other
	}
}

impl PartialEq<String> for Did {
	fn eq(&self, other: &String) -> bool {
		self.uri.as_str() == other
	}
}

/// NOTE: We do not consider the particular implementation of Debug/Display to be
/// covered by semver guarantees
#[derive(Debug, thiserror::Error)]
pub enum DidParseErr {
	#[error("string was too long")]
	TooLong,
	#[error("missing `did:` prefix")]
	MissingPrefix,
	#[error("missing DID method")]
	MissingMethod,
	#[error("method specific ID was invalid")]
	EmptyMethodSpecificId,
	#[error(transparent)]
	NotAUri(#[from] NotAUriErr),
	#[error("did contains query params but only DidUrls can have this")]
	ContainsQuery,
	#[error("did contains a fragment but only DidUrls can have this")]
	ContainsFragment,
}

fn parse_method_specific_id(value: &str) -> Result<RangeFrom<u16>, DidParseErr> {
	if value.len() > u16::MAX.into() {
		return Err(DidParseErr::TooLong);
	}
	let Some(suffix) = value.strip_prefix("did:") else {
		return Err(DidParseErr::MissingPrefix);
	};
	let Some((method, method_specific_id)) = suffix.split_once(":") else {
		return Err(DidParseErr::MissingMethod);
	};
	if method.is_empty() {
		return Err(DidParseErr::MissingMethod);
	}
	if method_specific_id.is_empty() {
		return Err(DidParseErr::EmptyMethodSpecificId);
	}

	let meth_specific_id_start = u16::try_from(value.len() - method_specific_id.len())
		.expect("infallible: already checked size");

	Ok(meth_specific_id_start..)
}

impl TryFrom<String> for Did {
	type Error = DidParseErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let method_specific_id = parse_method_specific_id(&value)?;
		let uri = Uri::try_from(value)?;
		if uri.has_query() {
			return Err(DidParseErr::ContainsQuery);
		}
		if uri.has_fragment() {
			return Err(DidParseErr::ContainsFragment);
		}

		Ok(Self {
			uri,
			method_specific_id,
		})
	}
}

impl FromStr for Did {
	type Err = DidParseErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let method_specific_id = parse_method_specific_id(s)?;
		let uri = Uri::try_from(s.to_owned())?;
		if uri.has_query() {
			return Err(DidParseErr::ContainsQuery);
		}
		if uri.has_fragment() {
			return Err(DidParseErr::ContainsFragment);
		}

		Ok(Self {
			uri,
			method_specific_id,
		})
	}
}

impl core::fmt::Display for Did {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_key_works() {
		// from my crate
		const DID_KEY_EXAMPLES: &[&str] = &[
			"did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw", // Test1
			"did:key:z6MkiaMbhXHNA4eJVCCj8dbzKzTgYDKf6crKgHVHid1F1WCT", // Test2
			"did:key:z6MkwSD8dBdqcXQzKJZQFPy2hh2izzxskndKCjdmC2dBpfME", // Test3
			"did:key:z6Mkh7U7jBwoMro3UeHmXes4tKtFbZhMRWejbtunbU4hhvjP", // Test1024
			"did:key:z6MkvLrkgkeeWeRwktZGShYPiB5YuPkhN2yi3MqMKZMFMgWr", // TestSha
		];

		for e in DID_KEY_EXAMPLES {
			let did = Did::from_str(e).expect(e);
			assert_eq!(did.method(), "key");
			assert_eq!(
				did.method_specific_id(),
				e.strip_prefix("did:key:").unwrap()
			);
		}
	}

	#[test]
	fn test_example_works() {
		const DID_EXAMPLE_EXAMPLES: &[&str] = &[
			"did:example:foobar:1337",
			"did:example:1337",
			"did:example:123456/path",
		];

		for e in DID_EXAMPLE_EXAMPLES {
			let did = Did::from_str(e).expect(e);
			assert_eq!(did.method(), "example");
			assert_eq!(
				did.method_specific_id(),
				e.strip_prefix("did:example:").unwrap()
			);
		}
	}

	#[test]
	fn test_web_works() {
		const DID_WEB_EXAMPLES: &[&str] = &[
			"did:web:example.com",
			"did:web:w3c-ccg.github.io",
			"did:web:w3c-ccg.github.io:user:alice",
			"did:web:example.com%3A3000",
		];

		for e in DID_WEB_EXAMPLES {
			let did = Did::from_str(e).expect(e);
			assert_eq!(did.method(), "web");
			assert_eq!(
				did.method_specific_id(),
				e.strip_prefix("did:web:").unwrap()
			);
		}
	}

	#[test]
	fn test_pkarr_works() {
		// generated randomly from https://app.pkarr.org/
		const DID_PKARR_EXAMPLES: &[&str] = &[
			"did:pkarr:9p8exgmze7p67d76j9r8mq848hjxfus14hne56ygidpdhqmqnxmy",
			"did:pkarr:qj9oh4q9qpa5399g6gxjec9r1ijjp1nnbogrojq1uah7a5czmf8y",
			"did:pkarr:3wmzzqhzpywzqtbgeh4df5rtb6yxpexdz5aqcxa9wtgy4r8adzoo",
		];

		for e in DID_PKARR_EXAMPLES {
			let did = Did::from_str(e).expect(e);
			assert_eq!(did.method(), "pkarr");
			assert_eq!(
				did.method_specific_id(),
				e.strip_prefix("did:pkarr:").unwrap()
			);
		}
	}

	#[test]
	fn test_reject_queries() {
		// https://www.w3.org/TR/did-1.1/#example-3
		assert!(matches!(
			Did::from_str("did:example:123456?versionId=1"),
			Err(DidParseErr::ContainsQuery)
		));

		// For api stability we should continue to return the query error instead of fragment
		// https://www.w3.org/TR/did-1.1/#example-a-resource-external-to-a-did-document
		assert!(matches!(
			Did::from_str(
				"did:example:123?service=agent&relativeRef=/credentials%23degree"
			),
			Err(DidParseErr::ContainsQuery)
		));
	}

	#[test]
	fn test_reject_fragments() {
		// https://www.w3.org/TR/did-1.1/#example-a-unique-verification-method-in-a-did-document
		assert!(matches!(
			Did::from_str("did:example:123#public-key-0"),
			Err(DidParseErr::ContainsFragment)
		));
	}

	#[test]
	fn test_accept_pct_encoded_pound() {
		// See https://github.com/w3c/did/issues/898
		// I am *choosing* to not treat pct encoded fragments as fragments since its a
		// bit weird
		assert_eq!(
			Did::from_str("did:example:123%23public-key-0")
				.unwrap()
				.method_specific_id(),
			"123%23public-key-0"
		);
	}
}
