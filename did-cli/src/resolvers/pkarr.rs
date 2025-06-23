use std::{collections::BTreeSet, str::FromStr as _};

use did_key::DidKey;
use did_pkarr::{DidPkarr, DidPkarrDocument, PkarrClientBlockingExt, PkarrClientExt};
use eyre::Context;

use crate::doc::{DidDocument, VerificationMethod};

use super::{DidResolver, DidResolverBlocking};

#[derive(Debug, bon::Builder)]
pub struct DidPkarrResolver {
	resolve_most_recent: bool,
	client: did_pkarr::Client,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DidPkarrReadErr(#[from] eyre::Report);

impl DidResolver for DidPkarrResolver {
	type Error = DidPkarrReadErr;
	type Did = DidPkarr;

	async fn read(
		&self,
		did: &Self::Did,
	) -> Result<crate::doc::DidDocument, Self::Error> {
		let pkarr_doc = if self.resolve_most_recent {
			PkarrClientExt::resolve_most_recent(&self.client, did).await
		} else {
			PkarrClientExt::resolve(&self.client, did).await
		}
		.wrap_err("failed to resolve pkarr")?;

		Ok(DidDocument::from(pkarr_doc))
	}
}

#[derive(Debug, bon::Builder)]
pub struct DidPkarrResolverBlocking {
	resolve_most_recent: bool,
	client: did_pkarr::ClientBlocking,
}

impl DidResolverBlocking for DidPkarrResolverBlocking {
	type Error = DidPkarrReadErr;
	type Did = DidPkarr;

	fn read(&self, did: &Self::Did) -> Result<crate::doc::DidDocument, Self::Error> {
		let pkarr_doc = if self.resolve_most_recent {
			PkarrClientBlockingExt::resolve_most_recent(&self.client, did)
		} else {
			PkarrClientBlockingExt::resolve(&self.client, did)
		}
		.wrap_err("failed to resolve pkarr")?;

		Ok(DidDocument::from(pkarr_doc))
	}
}

impl From<DidPkarrDocument> for DidDocument {
	fn from(pkarr_doc: DidPkarrDocument) -> Self {
		let mut authentication = BTreeSet::new();
		let mut assertion = BTreeSet::new();
		Self {
			id: crate::Uri::from(pkarr_doc.did()).try_into().unwrap(),
			also_known_as: pkarr_doc.also_known_as().cloned().collect(),
			verification_method: pkarr_doc
				.verification_methods()
				.enumerate()
				.map(|(idx, (vm, vr))| {
					use did_pkarr::doc::VerificationRelationship as VR;
					let vm_ref = crate::doc::VerificationMethodReference(idx as _);
					if vr.contains(VR::Authentication) {
						authentication.insert(vm_ref);
					}
					if vr.contains(VR::Assertion) {
						assertion.insert(vm_ref);
					}
					VerificationMethod::from(vm.to_owned())
				})
				.collect(),
			authentication,
			assertion,
		}
	}
}

impl From<did_pkarr::doc::VerificationMethod> for VerificationMethod {
	fn from(value: did_pkarr::doc::VerificationMethod) -> Self {
		match value {
			did_pkarr::doc::VerificationMethod::DidKey(did) => {
				Self::DidKey(DidKey::from_str(did.as_uri().as_str()).unwrap())
			}
			did_pkarr::doc::VerificationMethod::DidUrl(did) => {
				Self::External(crate::Uri::from(did).try_into().unwrap())
			}
		}
	}
}
