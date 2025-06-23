pub use core::error::Error as StdError;
use core::fmt::Debug;

use did_key::DidKey;

pub mod client;
pub mod doc;
pub mod resolvers;

type Uri<T = String> = fluent_uri::Uri<T>;

/// Alternative to `Box<dyn DidMethod>` since async fn in trait are not dyn-compatible.
#[derive(Debug)]
pub enum DynDidMethod {
	Key(DidKey),
	// Pkarr(did_pkarr::DidPkarr),
}
