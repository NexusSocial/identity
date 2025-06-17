//! Types related to DIDs

mod did;
mod did_pkarr;

pub use self::did::*;
pub use self::did_pkarr::*;

#[cfg(test)]
pub(crate) mod test {
	use hex_literal::hex;
	use std::sync::LazyLock;

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

	// From https://datatracker.ietf.org/doc/html/rfc8032#section-7.1
	pub(crate) static ED25519_EXAMPLES: LazyLock<Vec<ed25519_dalek::SigningKey>> =
		LazyLock::new(|| {
			let test1 = (
				hex!(
					"9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
				),
				hex!(
					"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
				),
			);
			let test2 = (
				hex!(
					"4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb"
				),
				hex!(
					"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c"
				),
			);
			let test3 = (
				hex!(
					"c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7"
				),
				hex!(
					"fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025"
				),
			);
			let test1024 = (
				hex!(
					"f5e5767cf153319517630f226876b86c8160cc583bc013744c6bf255f5cc0ee5"
				),
				hex!(
					"278117fc144c72340f67d0f2316e8386ceffbf2b2428c9c51fef7c597f1d426e"
				),
			);
			let test_sha = (
				hex!(
					"833fe62409237b9d62ec77587520911e9a759cec1d19755b7da901b96dca3d42"
				),
				hex!(
					"ec172b93ad5e563bf4932c70e1245034c35467ef2efd4d64ebf819683467e2bf"
				),
			);
			[test1, test2, test3, test1024, test_sha]
				.into_iter()
				.map(|(private, public)| {
					let private = ed25519_dalek::SigningKey::from_bytes(&private);
					assert_eq!(private.verifying_key().as_bytes(), &public);
					private
				})
				.collect()
		});
}
