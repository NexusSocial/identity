//! did:pkarrm - [Public Key Addressable Resource Record][pkarr]s using Multiformats based [Decentralized Identifiers][did].
//!
//! [did]: https://www.w3.org/TR/did-1.0/
//! [pkarr]: https://github.com/Pubky/pkarr

pub mod data_model;
mod did_pkarr;

pub use crate::data_model::DidPkarrDocument;
pub use crate::did_pkarr::DidPkarr;

/// Error types for the crate
pub mod errors {
	pub use crate::did_pkarr::{DidPkarrParseErr, InvalidPubkeyErr};
}
