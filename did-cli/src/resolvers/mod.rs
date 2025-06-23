use crate::{doc::DidDocument, StdError};
use std::{fmt::Debug, future::Future};

mod key;
mod pkarr;

pub use self::key::DidKeyResolver;
pub use self::pkarr::{DidPkarrResolver, DidPkarrResolverBlocking};

/// Blocking version of [`DidResolver`].
pub trait DidResolverBlocking: Debug + Send + Sync {
	type Error: StdError + Send + Sync + 'static;
	type Did;

	fn read(&self, did: &Self::Did) -> Result<DidDocument, Self::Error>;
}

pub trait DidResolver: Send + Sync + Debug {
	type Error: StdError + Send + Sync + 'static;
	type Did;

	fn read(
		&self,
		did: &Self::Did,
	) -> impl Future<Output = Result<DidDocument, Self::Error>> + Send;
}
