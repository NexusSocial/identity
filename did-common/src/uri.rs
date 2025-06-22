//! Module to abstract over whether we use fluent_uri or just a string.
//! Limits the scope of cfg gated code to this module as much as possble

use alloc::string::String;

#[cfg(feature = "uri")]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[repr(transparent)]
pub(crate) struct Uri(fluent_uri::Uri<String>);
#[cfg(not(feature = "uri"))]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[repr(transparent)]
pub(crate) struct Uri(String);

impl Uri {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}

	pub fn has_query(&self) -> bool {
		#[cfg(feature = "uri")]
		let result = self.0.has_query();
		#[cfg(not(feature = "uri"))]
		let result = self.0.contains('?');

		result
	}

	pub fn has_fragment(&self) -> bool {
		#[cfg(feature = "uri")]
		let result = self.0.has_fragment();
		#[cfg(not(feature = "uri"))]
		let result = self.0.contains('#');

		result
	}

	#[cfg(feature = "uri")]
	pub fn as_fluent(&self) -> &fluent_uri::Uri<String> {
		&self.0
	}
}

impl AsRef<str> for Uri {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
	}
}

#[cfg(feature = "uri")]
impl From<Uri> for fluent_uri::Uri<String> {
	fn from(value: Uri) -> Self {
		value.0
	}
}

#[cfg(feature = "uri")]
impl From<fluent_uri::Uri<String>> for Uri {
	fn from(value: fluent_uri::Uri<String>) -> Self {
		Uri(value)
	}
}

impl From<Uri> for String {
	fn from(value: Uri) -> Self {
		#[cfg_attr(not(feature = "uri"), expect(clippy::useless_conversion))]
		value.0.into()
	}
}

/// NOTE: We do not consider the particular implementation of Debug/Display to be
/// covered by semver guarantees
#[derive(Debug, thiserror::Error)]
#[error("not a uri")]
pub struct NotAUriErr(
	#[cfg(feature = "uri")]
	#[from]
	fluent_uri::error::ParseError<String>,
	#[cfg(not(feature = "uri"))]
	#[from]
	ParseError,
);

/// Only used when not using uri feature.
#[cfg_attr(feature = "uri", expect(dead_code))]
#[derive(Debug, thiserror::Error)]
enum ParseError {
	#[error("value was not ascii")]
	NotAscii,
}

impl TryFrom<String> for Uri {
	type Error = NotAUriErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		#[cfg(feature = "uri")]
		let uri = fluent_uri::Uri::parse(value).map_err(NotAUriErr::from)?;
		#[cfg(not(feature = "uri"))]
		let uri = {
			if !value.is_ascii() {
				return Err(NotAUriErr::from(ParseError::NotAscii));
			}

			value
		};

		Ok(Self(uri))
	}
}
