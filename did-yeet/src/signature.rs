use serde::{Deserialize, Serialize};

/// Signature bytes
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(transparent)]
pub struct Signature(pub Vec<u8>);

impl Signature {
	pub fn new(bytes: &[u8]) -> Self {
		Signature(Vec::from(bytes))
	}
}

impl From<Vec<u8>> for Signature {
	fn from(value: Vec<u8>) -> Self {
		Signature(value)
	}
}

#[cfg(test)]
mod tests {
	use color_eyre::{eyre::Context, Section};

	use super::*;

	#[test]
	fn test_serde() -> color_eyre::Result<()> {
		let _ = color_eyre::install();
		#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
		struct S {
			field: Vec<Signature>,
		}

		let expected_deserialized = S {
			field: vec![Signature::from(vec![0xDE, 0xAD, 0xBE, 0xEF])],
		};
		let expected_serialized = serde_json::json!({
			"field": vec![vec![0xDEu8, 0xADu8, 0xBEu8, 0xEFu8]],
		});

		assert_eq!(
			serde_json::from_value::<S>(expected_serialized.clone())
				.wrap_err("failed to deserialize")
				.with_note(|| format!("json was {expected_serialized:#?}"))?,
			expected_deserialized
		);
		assert_eq!(
			serde_json::to_value(expected_deserialized.clone())
				.wrap_err("failed to serialize")
				.with_note(|| format!("struct was {expected_deserialized:?}"))?,
			expected_serialized
		);

		Ok(())
	}
}
