use bon::bon;
use did_common::did::Did;
use did_key::DidKey;
use did_pkarr::DidPkarr;
use eyre::{eyre, Result, WrapErr as _};
use std::fmt::Debug;
use std::str::FromStr as _;
use std::sync::Arc;

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
}
