use std::collections::{BTreeMap, BTreeSet};

use fluent_uri::Uri;

use crate::{
	DidPkarrDocument,
	doc::{
		VerificationMethod, VerificationRelationship, doc_contents::DidDocumentContents,
	},
};

pub struct DidPkarrDocumentBuilder {
	pubkey: pkarr::PublicKey,
	also_known_as: BTreeSet<Uri<String>>,
	verification_methods: BTreeMap<VerificationMethod, VerificationRelationship>,
}

impl DidPkarrDocumentBuilder {
	pub fn new(public_key: pkarr::PublicKey) -> Self {
		DidPkarrDocumentBuilder {
			pubkey: public_key,
			also_known_as: BTreeSet::new(),
			verification_methods: BTreeMap::new(),
		}
	}

	pub fn also_known_as(mut self, aka: Uri<String>) -> Self {
		self.also_known_as.insert(aka);
		self
	}

	/// Append to the set of verification methods
	///
	/// # Panics
	/// Panics if `verification_relationship` does not contain at least one known bit.
	pub fn verification_method(
		mut self,
		verification_method: VerificationMethod,
		verification_relationship: VerificationRelationship,
	) -> Self {
		let verification_relationship = VerificationRelationship::from_bits_truncate(
			verification_relationship.bits(),
		);
		assert!(
			!verification_relationship.is_empty(),
			"verification method had no known bits set"
		);
		self.verification_methods
			.insert(verification_method, verification_relationship);
		self
	}

	pub fn finish(self) -> DidPkarrDocument {
		let contents = DidDocumentContents {
			aka: self.also_known_as.into_iter().collect(),
			vr: self.verification_methods.values().copied().collect(),
			vm: self.verification_methods.into_keys().collect(),
		};

		DidPkarrDocument {
			id: self.pubkey,
			contents,
		}
	}
}

#[cfg(test)]
mod test {
	use std::{str::FromStr as _, sync::LazyLock};

	use fluent_uri::Uri;
	use pkarr::PublicKey;

	use crate::{
		DidPkarrDocument,
		dids::test::DID_KEY_EXAMPLES,
		doc::{
			VerificationMethod, VerificationRelationship,
			doc_contents::DidDocumentContents,
		},
	};

	static EXAMPLE_B32_PUBKEY: LazyLock<PublicKey> = LazyLock::new(|| {
		// Randomly generated from app.pkarr.org
		PublicKey::from_str("1teexmau1ocrtneff71mdcpadwxi6wx8dxn11j6oc9x9i6zrrhjo")
			.unwrap()
	});

	#[test]
	fn test_empty() {
		let doc = DidPkarrDocument::builder(EXAMPLE_B32_PUBKEY.clone()).finish();
		assert_eq!(
			doc,
			DidPkarrDocument {
				id: EXAMPLE_B32_PUBKEY.clone(),
				contents: DidDocumentContents {
					aka: Vec::new(),
					vm: Vec::new(),
					vr: Vec::new()
				}
			}
		);
	}

	#[test]
	fn test_aka() {
		let atproto: Uri<String> = "at://thebutlah.com".parse().unwrap();
		let steam: Uri<String> = "steam://thebutlah".parse().unwrap();
		let foobar: Uri<String> = "foobar://example/foo".parse().unwrap();
		let doc = DidPkarrDocument::builder(EXAMPLE_B32_PUBKEY.clone())
			.also_known_as(steam.clone()) // These are not in alphabetical order
			.also_known_as(atproto.clone())
			.also_known_as(foobar.clone())
			.finish();
		assert_eq!(
			doc,
			DidPkarrDocument {
				id: EXAMPLE_B32_PUBKEY.clone(),
				contents: DidDocumentContents {
					aka: vec![atproto, foobar, steam], // alphabetical order
					vm: Vec::new(),
					vr: Vec::new()
				}
			}
		);
	}

	#[test]
	fn test_verification_methods() {
		let m1 = (
			DID_KEY_EXAMPLES[0].parse().unwrap(),
			VerificationRelationship::all(),
		);
		let m2 = (
			DID_KEY_EXAMPLES[1].parse().unwrap(),
			VerificationRelationship::Authentication,
		);
		let m3 = (
			DID_KEY_EXAMPLES[2].parse().unwrap(),
			VerificationRelationship::Assertion,
		);
		let m4 = (
			DID_KEY_EXAMPLES[3].parse().unwrap(),
			VerificationRelationship::KeyAgreement,
		);
		let m5 = (
			DID_KEY_EXAMPLES[4].parse().unwrap(),
			VerificationRelationship::Assertion
				| VerificationRelationship::Authentication,
		);

		// Test just 1
		for (vm, vr) in [&m1, &m2, &m3, &m4, &m5] {
			assert_eq!(
				DidPkarrDocument::builder(EXAMPLE_B32_PUBKEY.clone())
					.verification_method(VerificationMethod::clone(vm), *vr)
					.finish(),
				DidPkarrDocument {
					id: EXAMPLE_B32_PUBKEY.clone(),
					contents: DidDocumentContents {
						aka: Vec::new(),
						vm: vec![vm.clone()],
						vr: vec![*vr],
					}
				}
			);
		}

		// Test multiple
		for slice in [m1, m2, m3, m4, m5].chunks_exact(2) {
			let a = slice[0].clone();
			let b = slice[1].clone();

			// sort the values
			let (vm, vr) = if a.0 < b.0 {
				(vec![a.0.clone(), b.0.clone()], vec![a.1, b.1])
			} else {
				(vec![b.0.clone(), a.0.clone()], vec![b.1, a.1])
			};
			assert_eq!(
				DidPkarrDocument::builder(EXAMPLE_B32_PUBKEY.clone())
					.verification_method(VerificationMethod::clone(&a.0), a.1)
					.verification_method(VerificationMethod::clone(&b.0), b.1)
					.finish(),
				DidPkarrDocument {
					id: EXAMPLE_B32_PUBKEY.clone(),
					contents: DidDocumentContents {
						aka: Vec::new(),
						vm,
						vr,
					}
				}
			);
		}
	}
}
