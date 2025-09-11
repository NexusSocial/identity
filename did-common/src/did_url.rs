use alloc::{borrow::ToOwned as _, string::String};
use core::{hash::Hash, num::NonZeroU16, ops::Range, str::FromStr};

use crate::uri::{NotAUriErr, Uri};

// TODO: Do we want to allow for borrowing strings?

/// A [Decentralized Identifier][did-syntax]. This is a globally unique identifier that can be
/// resolved to a DID Document.
///
/// Note that the definition of a DidUrl used by this crate is *more* strict than what
/// is allowed for by the actual spec. We *do not* allow a path or query params in
/// the url. Only the did, followed by an optional fragment, is allowed.
///
/// [did-syntax]: https://www.w3.org/TR/did-1.1/#did-syntax
#[derive(Debug, Eq, Clone)]
pub struct DidUrl {
	uri: Uri,
	// u16 instead of usize because identifiers cant be that long anyway
	pub(crate) method_specific_id: Range<NonZeroU16>,
}

impl DidUrl {
	pub fn as_str(&self) -> &str {
		self.uri.as_str()
	}

	#[cfg(feature = "uri")]
	pub fn as_uri(&self) -> &fluent_uri::Uri<String> {
		self.uri.as_fluent()
	}

	/// The method for this DID Url.
	///
	/// # Example
	/// ```
	/// # use did_common::did_url::DidUrl;
	/// # use std::str::FromStr;
	/// assert_eq!(
	///     DidUrl::from_str("did:example:foobar#baz").unwrap().method(),
	///     "example",
	/// );
	/// ```
	pub fn method(&self) -> &str {
		const START_IDX: usize = "did:".len() as _;
		let range =
			START_IDX..usize::from(u16::from(self.method_specific_id.start) - 1);
		&self.uri.as_str()[range]
	}

	/// The method-specific ID for this DID Url.
	///
	/// # Example
	/// ```
	/// # use did_common::did_url::DidUrl;
	/// # use std::str::FromStr;
	/// assert_eq!(
	///     DidUrl::from_str("did:example:foobar#baz").unwrap().method_specific_id(),
	///     "foobar",
	/// );
	/// ```
	pub fn method_specific_id(&self) -> &str {
		let range = usize::from(u16::from(self.method_specific_id.start))
			..usize::from(u16::from(self.method_specific_id.end));
		&self.uri.as_str()[range]
	}

	/// The fragment part of the url. Empty string if there is no fragment.
	///
	/// # Example
	/// ```
	/// # use did_common::did_url::DidUrl;
	/// # use std::str::FromStr;
	/// assert_eq!(
	///     DidUrl::from_str("did:example:foobar#baz").unwrap().fragment(),
	///     "baz",
	/// );
	/// assert_eq!(DidUrl::from_str("did:example:foobar").unwrap().fragment(), "");
	/// ```
	pub fn fragment(&self) -> &str {
		let range = usize::from(u16::from(self.method_specific_id.end)) + 1..;
		self.uri.as_str().get(range).unwrap_or_default()
	}
}

impl PartialOrd for DidUrl {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Hash for DidUrl {
	fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl AsRef<str> for DidUrl {
	fn as_ref(&self) -> &str {
		self.uri.as_str()
	}
}

impl Ord for DidUrl {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.uri.cmp(&other.uri)
	}
}

impl<T: AsRef<str>> PartialEq<T> for DidUrl {
	fn eq(&self, other: &T) -> bool {
		self.uri.as_str() == other.as_ref()
	}
}

/// NOTE: We do not consider the particular implementation of Debug/Display to be
/// covered by semver guarantees
#[derive(Debug, thiserror::Error)]
pub enum DidUrlParseErr {
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
	#[error("query params are not allowed")]
	ContainsQuery,
	#[error("paths are not allowed")]
	ContainsPath,
	#[error("cannot end with fragment specifier `#`")]
	CannotEndWithFragmentSpecifier,
	#[error("multiple fragment specifiers `#` encountered")]
	MultipleFragmentSpecifiers,
}

fn parse_method_specific_id(value: &str) -> Result<Range<NonZeroU16>, DidUrlParseErr> {
	if value.len() > u16::MAX.into() {
		return Err(DidUrlParseErr::TooLong);
	}

	// Validate and extract the DID Method
	let Some(suffix) = value.strip_prefix("did:") else {
		return Err(DidUrlParseErr::MissingPrefix);
	};
	let Some((method, method_specific_id)) = suffix.split_once(":") else {
		return Err(DidUrlParseErr::MissingMethod);
	};
	if method.is_empty() {
		return Err(DidUrlParseErr::MissingMethod);
	}

	// Validate and extract the method specific ID
	if method_specific_id.is_empty() || method_specific_id.as_bytes()[0] == b'#' {
		return Err(DidUrlParseErr::EmptyMethodSpecificId);
	}
	let method_specific_id_start =
		u16::try_from(value.len() - method_specific_id.len())
			.and_then(NonZeroU16::try_from)
			.expect("infallible: already checked size");
	let method_specific_id_end = method_specific_id
		.find('#')
		.map(|idx| {
			NonZeroU16::try_from(
				u16::try_from(idx)
					.expect("infallible: already checked size")
					.checked_add(method_specific_id_start.get())
					.unwrap(),
			)
			.expect("infallible: this should never be zero")
		})
		.unwrap_or_else(|| {
			NonZeroU16::try_from(
				u16::try_from(value.len()).expect("infallible: already checked size"),
			)
			.expect("infallible: we already checked nonzero string length")
		});
	let method_specific_id_range = method_specific_id_start..method_specific_id_end;

	if let Some(idx) = method_specific_id.find(['/', '?']) {
		let bad_char = method_specific_id.as_bytes()[idx];
		if bad_char == b'/' {
			return Err(DidUrlParseErr::ContainsPath);
		} else {
			return Err(DidUrlParseErr::ContainsQuery);
		}
	}

	// Validate fragment
	let frag_range = usize::from(u16::from(method_specific_id_end))..;
	let frag = value.get(frag_range).unwrap_or_default();
	if frag.len() == 1 {
		debug_assert_eq!(frag.as_bytes()[0], b'#', "sanity");
		return Err(DidUrlParseErr::CannotEndWithFragmentSpecifier);
	}
	if frag.len() > 1 && frag[1..].contains("#") {
		return Err(DidUrlParseErr::MultipleFragmentSpecifiers);
	}

	Ok(method_specific_id_range)
}

impl TryFrom<String> for DidUrl {
	type Error = DidUrlParseErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let method_specific_id = parse_method_specific_id(&value)?;
		let uri = Uri::try_from(value)?;
		debug_assert!(!uri.has_query());
		debug_assert!(!uri.has_fragment());

		Ok(Self {
			uri,
			method_specific_id,
		})
	}
}

#[cfg(feature = "uri")]
impl TryFrom<fluent_uri::Uri<String>> for DidUrl {
	type Error = DidUrlParseErr;

	fn try_from(uri: fluent_uri::Uri<String>) -> Result<Self, Self::Error> {
		let method_specific_id = parse_method_specific_id(uri.as_str())?;

		Ok(Self {
			uri: Uri::from(uri),
			method_specific_id,
		})
	}
}

impl FromStr for DidUrl {
	type Err = DidUrlParseErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let method_specific_id = parse_method_specific_id(s)?;
		let uri = Uri::try_from(s.to_owned())?;

		Ok(Self {
			uri,
			method_specific_id,
		})
	}
}

impl core::fmt::Display for DidUrl {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_reject_empty_frag() {
		assert!(matches!(
			DidUrl::from_str("did:example:foobar#"),
			Err(DidUrlParseErr::CannotEndWithFragmentSpecifier)
		))
	}

	#[test]
	fn test_reject_frag_with_extra_pound_sign() {
		assert!(matches!(
			DidUrl::from_str("did:example:foobar#yeet#yoot"),
			Err(DidUrlParseErr::MultipleFragmentSpecifiers)
		));
		assert!(matches!(
			DidUrl::from_str("did:example:foobar#yeet#"),
			Err(DidUrlParseErr::MultipleFragmentSpecifiers)
		));
	}
}
