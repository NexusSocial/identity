//! Types associated with the DidDocument data model exposed by this crate.

use base64::Engine as _;

pub mod did;
pub mod doc_contents;
pub mod vmethod;
pub mod vrelationship;

fn b64_dec(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
	base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(s)
}
