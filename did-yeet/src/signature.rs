use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize};

/// Signature bytes
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
// NOTE: We intentionally don't borrow to make the common case (owned deserialization) simple
pub struct Signature<'a>(pub Cow<'a, [u8]>);

impl<'a> Signature<'a> {
	// pub fn deserialize_zero_copy<'de: 'a, D>(deserializer: D) -> Result<Self, D::Error>
	// where
	// 	D: Deserializer<'de>,
	// {
	// 	let s: &[u8] = Deserialize::deserialize(deserializer)?;
	//
	// 	Signature::try_from(s).map_err(serde::de::Error::custom)
	// }

	pub fn deserialize_zero_copy_slice<'de: 'a, D>(
		deserializer: D,
	) -> Result<Cow<'a, [Self]>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let borrows: Vec<&'de [u8]> = Deserialize::deserialize(deserializer)?;

		let result: Vec<Signature> = borrows.into_iter().map(Signature::from).collect();

		Ok(Cow::Owned(result))
	}
}

impl<'a> From<&'a [u8]> for Signature<'a> {
	fn from(value: &'a [u8]) -> Self {
		Signature(Cow::Borrowed(value))
	}
}

impl PartialEq<Signature<'_>> for Signature<'_> {
	fn eq(&self, other: &Signature<'_>) -> bool {
		self.0 == other.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_static_deserialize() {
		#[derive(Debug, Serialize, Deserialize)]
		struct S {
			field: Vec<Signature<'static>>,
		}
	}
}
