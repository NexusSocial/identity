//! Types related to the [`DidPkarrDocument`].

use std::str::FromStr as _;

use base64::Engine as _;
use doc_contents::{DidDocumentContents, FromTxtRecordErr};
use fluent_uri::Uri;
use pkarr::{
	dns::{rdata::RData, Name},
	Keypair, SignedPacket,
};

use crate::dids::Did;

pub(crate) mod doc_contents;
pub(crate) mod vmethod;
pub(crate) mod vrelationship;

pub use self::{vmethod::VerificationMethod, vrelationship::VerificationRelationship};

const TXT_DOMAIN: &str = "_did_pkarr.";

fn b64_dec(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
	base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(s)
}

/// The type returned when resolving a [DidPkarr](crate::DidPkarr) to its document.
#[derive(Debug, Eq, PartialEq)]
pub struct DidPkarrDocument {
	id: pkarr::PublicKey,
	contents: DidDocumentContents,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to convert to pkarr packet")]
pub struct ToPkarrErr(#[from] ToPkarrErrInner);

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
enum ToPkarrErrInner {
	#[error("signing key did not match verifying key")]
	KeyMismatch,
	#[error("failed to convert to pkarr SignedPacket")]
	ToPkarr(#[from] pkarr::errors::SignedPacketBuildError),
}

impl DidPkarrDocument {
	/// Get the DID associated with this DID Document.
	///
	/// # Performance
	/// This allocates every time.
	pub fn did(&self) -> Did {
		let s = self.id.to_z32();
		Did::from_str(&format!("did:pkarr:{s}")).expect("infallible")
	}

	pub fn also_known_as(&self) -> impl Iterator<Item = &Uri<String>> {
		self.contents.aka.iter()
	}

	pub fn verification_methods(
		&self,
	) -> impl Iterator<Item = (&VerificationMethod, VerificationRelationship)> {
		debug_assert_eq!(self.contents.vm.len(), self.contents.vr.len());
		self.contents
			.vm
			.iter()
			.zip(self.contents.vr.iter().copied())
	}

	pub fn to_pkarr_packet(
		&self,
		signing_key: &ed25519_dalek::SigningKey,
		ts: pkarr::Timestamp,
	) -> Result<pkarr::SignedPacket, ToPkarrErr> {
		let kp = Keypair::from_secret_key(signing_key.as_bytes());
		if signing_key.verifying_key() != *self.id.verifying_key() {
			return Err(ToPkarrErr::from(ToPkarrErrInner::KeyMismatch));
		}
		pkarr::SignedPacket::builder()
			.timestamp(ts)
			.txt(
				Name::new(TXT_DOMAIN).expect("infallible"),
				self.contents.to_txt_record(),
				0,
			)
			.sign(&kp)
			.map_err(ToPkarrErrInner::from)
			.map_err(ToPkarrErr::from)
	}
}

/// Error converting a [SignedPacket] to a [DidPkarrDocument].
#[derive(Debug, thiserror::Error)]
pub enum TryFromSignedPacketErr {
	#[error("missing a _did_pkarr TXT record")]
	NoDidPkarrTxtRecord,
	#[error("encountered more than one _did_pkarr record")]
	MultipleDidPkarrRecords,
	#[error(transparent)]
	FromTxtRecordErr(#[from] FromTxtRecordErr),
}

impl TryFrom<SignedPacket> for DidPkarrDocument {
	type Error = TryFromSignedPacketErr;

	fn try_from(value: SignedPacket) -> Result<Self, Self::Error> {
		let id = value.public_key();
		let mut it = value.resource_records(TXT_DOMAIN);
		let Some(record) = it.next() else {
			return Err(TryFromSignedPacketErr::NoDidPkarrTxtRecord);
		};
		let RData::TXT(ref txt) = record.rdata else {
			return Err(TryFromSignedPacketErr::NoDidPkarrTxtRecord);
		};
		if it.next().is_some() {
			return Err(TryFromSignedPacketErr::MultipleDidPkarrRecords);
		}
		let contents = DidDocumentContents::try_from(txt)?;

		Ok(Self { id, contents })
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::dids::test::{DID_KEY_EXAMPLES, ED25519_EXAMPLES};

	use pkarr::Timestamp;
	use std::time::SystemTime;

	fn dummy_doc(signing_key: &ed25519_dalek::SigningKey) -> DidPkarrDocument {
		DidPkarrDocument {
			id: signing_key.verifying_key().as_bytes().try_into().unwrap(),
			contents: DidDocumentContents {
				aka: vec!["at://thebutlah.com".parse().unwrap()],
				vm: DID_KEY_EXAMPLES
					.iter()
					.map(|k| k.parse().unwrap())
					.collect(),
				vr: DID_KEY_EXAMPLES
					.iter()
					.map(|_| VerificationRelationship::Authentication)
					.collect(),
			},
		}
	}

	#[test]
	fn test_from_signed_packet() {
		let signing_key = &ED25519_EXAMPLES[0];
		let expected_doc = dummy_doc(signing_key);
		let ts = Timestamp::from(SystemTime::UNIX_EPOCH);
		let signed = expected_doc
			.to_pkarr_packet(signing_key, ts)
			.expect("failed to serialize to pkarr");
		let deserialized_doc = DidPkarrDocument::try_from(signed)
			.expect("failed to deserialize from pkarr");
		assert_eq!(deserialized_doc, expected_doc);
	}

	#[test]
	fn test_protection_against_key_mismatch() {
		let s1 = ED25519_EXAMPLES[0].clone();
		let p1: pkarr::PublicKey = s1.verifying_key().as_bytes().try_into().unwrap();
		let s2 = ED25519_EXAMPLES[1].clone();
		let ts = Timestamp::from(std::time::SystemTime::UNIX_EPOCH);
		let doc_from_s1 = DidPkarrDocument {
			id: p1.clone(),
			contents: DidDocumentContents {
				aka: vec!["at://thebutlah.com".parse().unwrap()],
				vm: DID_KEY_EXAMPLES
					.iter()
					.map(|k| k.parse().unwrap())
					.collect(),
				vr: DID_KEY_EXAMPLES
					.iter()
					.map(|_| VerificationRelationship::Authentication)
					.collect(),
			},
		};

		assert_eq!(
			doc_from_s1.to_pkarr_packet(&s1, ts).unwrap().public_key(),
			p1
		);
		assert_eq!(
			doc_from_s1
				.to_pkarr_packet(&s2, ts)
				.expect_err("mismatched keys should error")
				.0,
			ToPkarrErrInner::KeyMismatch
		);
	}
}
