use std::{collections::BTreeSet, convert::Infallible};

use did_common::did::Did;
use did_key::DidKey;

use crate::doc::{DidDocument, VerificationMethod, VerificationMethodReference};

use super::{DidResolver, DidResolverBlocking};

#[derive(Debug)]
pub struct DidKeyResolver;

impl DidResolverBlocking for DidKeyResolver {
	type Error = Infallible;
	type Did = DidKey;

	fn read(&self, did_key: &Self::Did) -> Result<DidDocument, Self::Error> {
		let did = Did::try_from(did_key.to_string()).unwrap();
		Ok(DidDocument {
			id: did.clone(),
			also_known_as: vec![],
			verification_method: vec![VerificationMethod::DidKey(did_key.clone())],
			authentication: BTreeSet::from([0].map(VerificationMethodReference)),
			assertion: BTreeSet::from([0].map(VerificationMethodReference)),
		})
	}
}

impl DidResolver for DidKeyResolver {
	type Error = Infallible;
	type Did = DidKey;

	async fn read(&self, did: &Self::Did) -> Result<DidDocument, Self::Error> {
		DidResolverBlocking::read(self, did)
	}
}
