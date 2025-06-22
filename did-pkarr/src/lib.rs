//! `did:pkarr` - [PKARR][pkarr] based [Decentralized Identifiers][did]
//!
//! [did]: https://www.w3.org/TR/did-1.1/
//! [pkarr]: https://github.com/Pubky/pkarr

pub mod did_pkarr;
pub mod doc;
#[cfg(any(feature = "dht", feature = "http"))]
pub mod io;

pub use crate::did_pkarr::DidPkarr;
pub use crate::doc::DidPkarrDocument;

pub use pkarr;

#[cfg(any(feature = "dht", feature = "http"))]
pub use crate::io::{Client, PkarrClientBlockingExt, PkarrClientExt};
