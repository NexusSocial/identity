use uuid::Uuid;

// PERF: stop allocating, uuids are a known fixed length to begin with.
pub fn uuid_to_did(did_hostname: &str, uuid: &Uuid) -> String {
	format!("did:web:{did_hostname}:v1:{}", uuid.as_hyphenated())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_known_examples() {
		let examples = [
			(
				"did.socialvr.net",
				Uuid::from_u128(1337u128),
				"did:web:did.socialvr.net:v1:00000000-0000-0000-0000-000000000539",
			),
			(
				"socialvr.net",
				Uuid::from_u128(0u128),
				"did:web:socialvr.net:v1:00000000-0000-0000-0000-000000000000",
			),
			(
				"foo.bar.baz.bingus",
				Uuid::from_u128(u128::MAX),
				"did:web:foo.bar.baz.bingus:v1:ffffffff-ffff-ffff-ffff-ffffffffffff",
			),
		];

		for example @ (hostname, uuid, did) in examples {
			assert_eq!(
				uuid_to_did(hostname, &uuid),
				did,
				"example failed {example:?}"
			);
		}
	}
}
