use std::fmt::Debug;
use std::future::Future;

pub mod client;
pub mod doc;

/// Blocking version of [`DidMethod`].
pub trait DidMethodBlocking: Debug + Send + Sync {}

pub trait DidMethod: Send + Sync + Debug {
	fn read(&self) -> impl Future<Output = ()> + Send;
}

/// Alternative to `Box<dyn DidMethod>` since async fn in trait are not dyn-compatible.
#[derive(Debug)]
pub enum DynDidMethod {}

impl DidMethod for DynDidMethod {
	async fn read(&self) {
		todo!()
	}
}
