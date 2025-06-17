//! `did:pkarr` - [Decentralized Identifiers] using
//! [Public Key Addressable Resource Record][pkarr]s.
//!
//! [did]: https://www.w3.org/TR/did-1.0/
//! [pkarr]: https://github.com/Pubky/pkarr

pub mod dids;
pub mod doc;
#[cfg(any(feature = "dht", feature = "http"))]
pub mod io;

pub use crate::dids::DidPkarr;
pub use crate::doc::DidPkarrDocument;

pub use pkarr;

#[cfg(any(feature = "dht", feature = "http"))]
pub use crate::io::PkarrClientExt;
