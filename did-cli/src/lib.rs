use core::convert::Infallible;
pub use core::error::Error as StdError;
use core::fmt::Debug;
use core::future::Future;

use did_key::DidKey;

pub mod client;
pub mod doc;

type Uri<T = String> = fluent_uri::Uri<T>;

/// Blocking version of [`DidMethod`].
pub trait DidMethodBlocking: Debug + Send + Sync {
	type Error: StdError + Send + Sync + 'static;

	fn read(&self) -> Result<(), Self::Error>;
}

pub trait DidMethod: Send + Sync + Debug {
	type Error: StdError + Send + Sync + 'static;

	fn read(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// Alternative to `Box<dyn DidMethod>` since async fn in trait are not dyn-compatible.
#[derive(Debug)]
pub enum DynDidMethod {
	Key(DidKey),
	// Pkarr(did_pkarr::DidPkarr),
}

#[derive(Debug, thiserror::Error)]
pub enum DynDidReadErr {}

impl DidMethod for DynDidMethod {
	type Error = DynDidReadErr;

	async fn read(&self) -> Result<(), Self::Error> {
		match self {
			Self::Key(m) => Ok(DidMethod::read(m).await.expect("infallible")),
			// Self::Pkarr(m) => DidMethod::read(m).await,
		}
	}
}

impl DidMethodBlocking for DynDidMethod {
	type Error = DynDidReadErr;

	fn read(&self) -> Result<(), Self::Error> {
		todo!()
	}
}

impl DidMethodBlocking for DidKey {
	type Error = Infallible;

	fn read(&self) -> Result<(), Self::Error> {
		todo!()
	}
}

impl DidMethod for DidKey {
	type Error = Infallible;

	async fn read(&self) -> Result<(), Self::Error> {
		<Self as DidMethodBlocking>::read(self)
	}
}
