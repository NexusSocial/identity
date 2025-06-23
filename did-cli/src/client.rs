use did_common::did::Did;
use eyre::Result;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;

use crate::{doc::DidDocument, DidMethodBlocking, DynDidMethod, StdError};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct MethodId(&'static str);

impl Display for MethodId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "did:{}", self.0)
	}
}

#[derive(Debug, Clone, derive_more::Deref)]
pub struct Client(Arc<ClientInner>);

// TODO: don't make this pub and stop using Deref
#[derive(Debug)]
pub struct ClientInner {
	resolvers: HashMap<MethodId, DynDidMethod>,
}

impl ClientInner {
	pub async fn read(&self, did: &Did) -> Result<DidDocument> {
		todo!()
	}
}
