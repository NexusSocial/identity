//! did:pkarrm - [Public Key Addressable Resource Record][pkarr]s using Multiformats based [Decentralized Identifiers][did].
//!
//! [did]: https://www.w3.org/TR/did-1.0/
//! [pkarr]: https://github.com/Pubky/pkarr

mod resolver;

pub const ED25519_PUB_LEN: usize = 32;

#[derive(Debug, Eq, PartialEq, Clone)]
struct DidPkarrm {
	pubkey: Pubkey,
}

impl DidPkarrm {
	pub fn as_pubkey(&self) -> &Pubkey {
		&self.pubkey
	}
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Pubkey(pub [u8; ED25519_PUB_LEN]);
