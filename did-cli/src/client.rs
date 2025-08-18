use bon::bon;
use did_common::did::Did;
use did_key::DidKey;
use did_pkarr::{DidPkarr, DidPkarrDocument, PkarrClientBlockingExt};
use ed25519_dalek::SigningKey;
use eyre::{Result, WrapErr as _, eyre};
use std::fmt::Debug;
use std::str::FromStr as _;
use std::sync::Arc;

use crate::DidMethodKind;
use crate::resolvers::{DidPkarrResolverBlocking, DidResolverBlocking};
use crate::{doc::DidDocument, resolvers::DidKeyResolver};

#[derive(Debug, Clone, derive_more::Deref)]
pub struct Client(Arc<ClientInner>);

#[bon]
impl Client {
	#[builder]
	pub fn new() -> Self {
		Self(Arc::new(ClientInner {
			key: DidKeyResolver,
			pkarr: DidPkarrResolverBlocking::builder()
				.client(did_pkarr::Client::builder().build().unwrap().as_blocking())
				.resolve_most_recent(true)
				.build(),
		}))
	}
}

// TODO: don't make this pub and stop using Deref
#[derive(Debug)]
pub struct ClientInner {
	key: DidKeyResolver,
	pkarr: DidPkarrResolverBlocking,
}

impl ClientInner {
	pub fn read(&self, did: &Did) -> Result<DidDocument> {
		let doc = match did.method() {
			"key" => {
				let did = DidKey::from_str(did.as_str()).wrap_err("invalid did:key")?;
				DidResolverBlocking::read(&self.key, &did)
					.wrap_err("failed to read did:key")?
			}
			"pkarr" => {
				let did =
					DidPkarr::from_str(did.as_str()).wrap_err("invalid did:pkarr")?;
				DidResolverBlocking::read(&self.pkarr, &did)
					.wrap_err("failed to read did:pkarr")?
			}
			method => return Err(eyre!("unsupported did method `{method}`")),
		};

		Ok(doc)
	}

	pub fn create(&self, method: DidMethodKind, priv_key: &SigningKey) -> Result<Did> {
		let pub_key = priv_key.verifying_key();
		let did = match method {
			DidMethodKind::Key => {
				let did = DidKey {
					multicodec: did_key::KnownMultikeys::Ed25519Pub.into(),
					pubkey: pub_key.as_bytes().to_vec(),
				};

				let did: Did = format!("{did}").parse().unwrap();

				did
			}
			DidMethodKind::Pkarr => {
				let doc = DidPkarrDocument::builder(pub_key.into()).finish();
				PkarrClientBlockingExt::publish(
					&self.pkarr.client,
					&doc,
					None,
					priv_key,
				)
				.wrap_err("failed to publish to pkarr")?;

				doc.did()
					.as_uri()
					.to_owned()
					.try_into()
					.expect("all PKARR dids are valid dids")
			}
		};

		Ok(did)
	}
}
