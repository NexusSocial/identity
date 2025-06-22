use alloc::string::String;
use core::str::FromStr;

use crate::did_url::{DidUrl, DidUrlParseErr};

// TODO: Do we want to allow for borrowing strings?

/// A [Decentralized Identifier][did-syntax]. This is a globally unique identifier that can be
/// resolved to a DID Document.
///
/// [did-syntax]: https://www.w3.org/TR/did-1.1/#did-syntax
#[derive(Debug, Eq, Clone)]
pub struct Did {
	// All DidUrls are Dids but not all Dids are DidUrls
	inner: DidUrl,
}

impl Did {
	pub fn as_str(&self) -> &str {
		let range = ..usize::from(u16::from(self.inner.method_specific_id.end));
		debug_assert_eq!(
			range,
			..self.inner.as_str().len(),
			"until we support views on a DidUrl, `inner` shouldnt contain a fragment"
		);
		&self.inner.as_str()[range]
	}

	#[cfg(feature = "uri")]
	pub fn as_uri(&self) -> &fluent_uri::Uri<String> {
		self.inner.as_uri()
	}

	pub fn as_did_url(&self) -> &DidUrl {
		&self.inner
	}

	/// The method for this did.
	///
	/// # Example
	/// ```
	/// # use did_common::did::Did;
	/// # use std::str::FromStr;
	/// assert_eq!(Did::from_str("did:example:foobar").unwrap().method(), "example");
	/// ```
	pub fn method(&self) -> &str {
		self.inner.method()
	}

	/// The method-specific for this did.
	///
	/// # Example
	/// ```
	/// # use did_common::did::Did;
	/// # use std::str::FromStr;
	/// assert_eq!(
	///     Did::from_str("did:example:foobar").unwrap().method_specific_id(),
	///     "foobar",
	/// );
	/// ```
	pub fn method_specific_id(&self) -> &str {
		self.inner.method_specific_id()
	}
}

impl PartialOrd for Did {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl AsRef<str> for Did {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Ord for Did {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.inner.cmp(&other.inner)
	}
}

impl<T: AsRef<str>> PartialEq<T> for Did {
	fn eq(&self, other: &T) -> bool {
		self.as_str() == other.as_ref()
	}
}
/// NOTE: We do not consider the particular implementation of Debug/Display to be
/// covered by semver guarantees
#[derive(Debug, thiserror::Error)]
pub enum DidParseErr {
	#[error(transparent)]
	DidUrl(#[from] DidUrlParseErr),
	#[error("contains a fragment but only DidUrls can have this")]
	ContainsFragment,
}

impl TryFrom<String> for Did {
	type Error = DidParseErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let inner = DidUrl::try_from(value)?;
		if !inner.fragment().is_empty() {
			return Err(DidParseErr::ContainsFragment);
		}

		Ok(Self { inner })
	}
}

#[cfg(feature = "uri")]
impl TryFrom<fluent_uri::Uri<String>> for Did {
	type Error = DidParseErr;

	fn try_from(value: fluent_uri::Uri<String>) -> Result<Self, Self::Error> {
		let inner = DidUrl::try_from(value)?;
		if !inner.fragment().is_empty() {
			return Err(DidParseErr::ContainsFragment);
		}

		Ok(Self { inner })
	}
}

impl FromStr for Did {
	type Err = DidParseErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let inner = DidUrl::from_str(s)?;
		if !inner.fragment().is_empty() {
			return Err(DidParseErr::ContainsFragment);
		}

		Ok(Self { inner })
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
			"did:example:123456%2Fencodedpath",
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
	fn test_reject_paths() {
		assert!(matches!(
			Did::from_str("did:example:123456/path"),
			Err(DidParseErr::DidUrl(DidUrlParseErr::ContainsPath))
		));
		assert!(matches!(
			Did::from_str("did:example:123/api?x=y"),
			Err(DidParseErr::DidUrl(DidUrlParseErr::ContainsPath))
		));
	}

	#[test]
	fn test_reject_queries() {
		// https://www.w3.org/TR/did-1.1/#example-3
		assert!(matches!(
			Did::from_str("did:example:123456?versionId=1"),
			Err(DidParseErr::DidUrl(DidUrlParseErr::ContainsQuery))
		));

		// https://www.w3.org/TR/did-1.1/#example-a-resource-external-to-a-did-document
		assert!(matches!(
			Did::from_str(
				"did:example:123?service=agent&relativeRef=/credentials%23degree"
			),
			Err(DidParseErr::DidUrl(DidUrlParseErr::ContainsQuery))
		));
	}

	#[test]
	fn test_reject_fragments() {
		// https://www.w3.org/TR/did-1.1/#example-a-unique-verification-method-in-a-did-document
		assert!(matches!(
			Did::from_str("did:example:123#public-key-0"),
			Err(DidParseErr::ContainsFragment)
		));
		assert!(matches!(
			Did::from_str("did:example:123#"),
			Err(DidParseErr::DidUrl(
				DidUrlParseErr::CannotEndWithFragmentSpecifier
			))
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
