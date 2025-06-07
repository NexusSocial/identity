use bitflags::bitflags;
use fluent_uri::Uri;
use pkarr::{dns::rdata::TXT, SignedPacket};

/// A verification method most typically is a public key (via `did:key`), or a Did Url
/// that links to a verification method in a different Did Document.
pub enum VerificationMethod<T = String> {
	/// A `did:key`. This does not include the fragment suffix, to save space.
	DidKey(Uri<T>),
	/// A reference to a verification method in a remote Did Document. Any method other
	/// than `did:key` can be used.
	///
	/// DidUrls allow the use of verification methods that are controlled by third
	/// parties or with alternative did methods such as did:web. By referencing external
	/// Dids, users can use more convenient third party services while retaining their
	/// ability for credible exit.
	DidUrl(Uri<T>),
}

bitflags! {
	/// Verification relationships are represented as a bitset(*).
	///
	/// # What is a verification relationship?
	///
	/// A verification relationship dictates how a particular [`VerificationMethod`].
	/// Can be used.
	///
	/// See also:
	/// - <https://www.w3.org/TR/did-1.1/#verification-relationships>
	/// - <https://www.w3.org/TR/cid-1.0/#verification-relationships>
	///
	/// # (*) A note about varint encoding
	///
	/// [Varint encoding](https://github.com/multiformats/unsigned-varint) is used by
	/// multiformats to represent variable-size integers. We use varints for the
	/// `VerificationRelationship` to make it more likely that the syntax for did:pkarrm
	/// will continue to be valid even if did-core adds more verification relationships
	/// increasing the overall number to more than 8 (the maximum number of bits in a
	/// byte). Instead of preemptively using a u16 or u32, we simply use a varint.
	///
	/// However as of right now, did:pkarrm *only* supports three verification
	/// relationships, meaning only the lowest 3 bits could ever be set. Varint encoding
	/// is a no-op for all bytes `<128` because their most significant bit is not set.
	/// This means that even though the verification relationship is *specified* as a
	/// varint, this implementation of did:pkarrm can disregard this and just directly
	/// encode as a u8 bitflags.
	pub struct VerificationRelationship: u8 {
		/// <https://www.w3.org/TR/cid-1.0/#authentication>
		const Authentication = (1 << 0);
		/// <https://www.w3.org/TR/cid-1.0/#assertion>
		const Assertion = (1 << 1);
		/// <https://www.w3.org/TR/cid-1.0/#key-agreement>
		const KeyAgreement = (1 << 2);
	}
}

/// Everything in a did:pkarrm's Did Document except the `id` field. A
/// `DidDocumentContents` can be mapped 1:1 to a DNS txt record, for use in PKARR.
///
/// The generics are simply to enable borrowed data, they can be `&str` or `String`.
/// See [fluent_uri](fluent_uri) for more info.
pub struct DidDocumentContents<T1 = String, T2 = String> {
	/// "Also Known As". A list of alternative aliases for the user.
	/// <https://www.w3.org/TR/cid-1.0/#also-known-as>
	pub aka: Vec<Uri<T1>>,
	/// The [VerificationMethod]s. Index in vec matches `vr`.
	pub vm: Vec<VerificationMethod<T2>>,
	/// The [VerificationRelationship]s. The index in the vec matches
	/// `vm`.
	pub vr: Vec<VerificationRelationship>,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to extract information from dns txt record")]
pub struct FromTxtRecordErr;

impl<'a> TryFrom<TXT<'a>> for DidDocumentContents<&'a str, &'a str> {
	type Error = FromTxtRecordErr;

	fn try_from(value: TXT<'a>) -> Result<Self, Self::Error> {
		let attrs = value.attributes();
		for (k, v) in attrs.iter() {}

		let result = Self {
			aka: todo!(),
			vm: todo!(),
			vr: todo!(),
		};

		todo!()
	}
}

enum Field {
	Aka,
	Vm,
	Vr,
}

struct FieldIdx(u8);

#[derive(Debug, thiserror::Error)]
enum ParseFieldErr {
	#[error("unknown field name")]
	UnknownField,
	#[error("field suffix was not a u8: {0}")]
	SuffixNotANumber(#[from] std::num::ParseIntError),
	#[error("expected a suffix after the field name but none was provided")]
	SuffixMissing,
}

fn parse_field_from_key(key: &str) -> Result<(Field, FieldIdx), ParseFieldErr> {
	let (field, suffix) = if let Some(suffix) = key.strip_prefix("aka") {
		(Field::Aka, suffix)
	} else if let Some(suffix) = key.strip_prefix("vm") {
		(Field::Vm, suffix)
	} else if let Some(suffix) = key.strip_prefix("vr") {
		(Field::Vr, suffix)
	} else {
		return Err(ParseFieldErr::UnknownField);
	};

	let idx: u8 = match field {
		Field::Aka => suffix.parse()?,
		Field::Vm => todo!(),
		Field::Vr => todo!(),
	};

	todo!()
}
