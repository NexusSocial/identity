//! did:pkarr - [Public Key Addressable Resource Record][pkarr] based [Decentralized Identifiers][did].
//!
//! [did]: https://www.w3.org/TR/did-1.0/
//! [pkarr]: https://github.com/Pubky/pkarr
//!
//! # Feature Flags
#![doc = document_features::document_features!()]

#[cfg(feature = "io")]
pub use did_pkarr_io as io;
