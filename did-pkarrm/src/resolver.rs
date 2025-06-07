use std::{collections::HashMap, str::FromStr};

use bitflags::bitflags;
use fluent_uri::Uri;
use pkarr::{dns::rdata::TXT, SignedPacket};

/// A verification method most typically is a public key (via `did:key`), or a Did Url
/// that links to a verification method in a different Did Document.
pub enum VerificationMethod {
	/// A `did:key`. This does not include the fragment suffix, to save space.
	DidKey(Did),
	/// A reference to a verification method in a remote Did Document. Any method other
	/// than `did:key` can be used.
	///
	/// DidUrls allow the use of verification methods that are controlled by third
	/// parties or with alternative did methods such as did:web. By referencing external
	/// Dids, users can use more convenient third party services while retaining their
	/// ability for credible exit.
	DidUrl(Did),
}

impl FromStr for VerificationMethod {
	type Err = ParseVerificationMethodErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let uri: Uri<String> = Uri::from_str(s)?;
		let did = Did::try_from(uri)?;
		Ok(Self::from(did))
	}
}

impl From<Did> for VerificationMethod {
	fn from(value: Did) -> Self {
		todo!()
	}
}

pub struct Did(Uri<String>);

#[derive(Debug, thiserror::Error)]
pub enum DidFromUriErr {
	#[error("did not start with did:")]
	WrongPrefix,
}

impl TryFrom<Uri<String>> for Did {
	type Error = DidFromUriErr;

	fn try_from(value: Uri<String>) -> Result<Self, Self::Error> {
		todo!()
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ParseVerificationMethodErr {
	#[error("not a uri: {0}")]
	NotAUri(#[from] fluent_uri::error::ParseError),
	#[error("did not start with did:")]
	NotADid(#[from] DidFromUriErr),
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

#[derive(Debug, thiserror::Error)]
pub enum ParseVerificationRelationshipErr {
	//TODO
}

impl FromStr for VerificationRelationship {
	type Err = ParseVerificationRelationshipErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		todo!()
	}
}

/// Everything in a did:pkarrm's Did Document except the `id` field. A
/// `DidDocumentContents` can be mapped 1:1 to a DNS txt record, for use in PKARR.
///
/// The generics are simply to enable borrowed data, they can be `&str` or `String`.
/// See [fluent_uri](fluent_uri) for more info.
pub struct DidDocumentContents {
	/// "Also Known As". A list of alternative aliases for the user.
	/// <https://www.w3.org/TR/cid-1.0/#also-known-as>
	pub aka: Vec<Uri<String>>,
	/// The [VerificationMethod]s. Index in vec matches `vr`.
	pub vm: Vec<VerificationMethod>,
	/// The [VerificationRelationship]s. The index in the vec matches
	/// `vm`.
	pub vr: Vec<VerificationRelationship>,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to extract information from dns txt record")]
pub enum FromTxtRecordErr {
	#[error("encountered too many attributes")]
	TooManyAttrs,
	#[error("failed to extract fields from attributes: {0}")]
	AttrsToFields(#[from] AttrsToFieldsErr),
	#[error("failed to parse aka string: {0}")]
	AkaParseErr(fluent_uri::error::ParseError),
	#[error("failed to parse vm string: {0}")]
	VmParseErr(#[from] ParseVerificationMethodErr),
	#[error("failed to parse vr string: {0}")]
	VrParseErr(#[from] ParseVerificationRelationshipErr),
}

impl TryFrom<TXT<'_>> for DidDocumentContents {
	type Error = FromTxtRecordErr;

	fn try_from(value: TXT<'_>) -> Result<Self, Self::Error> {
		let attrs = value.attributes();
		if attrs.len() >= usize::from(u8::MAX) {
			return Err(FromTxtRecordErr::TooManyAttrs);
		}
		let fields = attrs_to_fields(&attrs)?;

		let aka: Result<Vec<Uri<String>>, _> = fields
			.aka
			.into_iter()
			.map(|s: &str| {
				Uri::from_str(s).map_err(|e| FromTxtRecordErr::AkaParseErr(e))
			})
			.collect();
		let aka = aka?;
		let vm: Result<Vec<VerificationMethod>, _> = fields
			.vm
			.into_iter()
			.map(|s: &str| {
				VerificationMethod::from_str(s)
					.map_err(|e| FromTxtRecordErr::VmParseErr(e))
			})
			.collect();
		let vm = vm?;
		let vr: Result<Vec<VerificationRelationship>, _> = fields
			.vr
			.into_iter()
			.map(|s: &str| {
				VerificationRelationship::from_str(s)
					.map_err(|e| FromTxtRecordErr::VrParseErr(e))
			})
			.collect();
		let vr = vr?;

		Ok(Self { aka, vm, vr })
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Field {
	Aka,
	Vm,
	Vr,
}

/// The index in the field's vec
struct FieldIdx(u8);

#[derive(Debug, thiserror::Error)]
pub enum ParseKeyErr {
	#[error("key suffix was not a u8: {0}")]
	SuffixNotANumber(#[from] std::num::ParseIntError),
	#[error("unknown attribute key")]
	UnknownKey,
}

fn parse_field_from_key(key: &str) -> Result<(Field, FieldIdx), ParseKeyErr> {
	let (field, suffix) = if let Some(suffix) = key.strip_prefix("aka") {
		(Field::Aka, suffix)
	} else if let Some(suffix) = key.strip_prefix("vm") {
		(Field::Vm, suffix)
	} else if let Some(suffix) = key.strip_prefix("vr") {
		(Field::Vr, suffix)
	} else {
		return Err(ParseKeyErr::UnknownKey);
	};

	let idx: u8 = suffix.parse()?;
	Ok((field, FieldIdx(idx)))
}

/// The sum of a arithmetic sequence of numbers:
/// `0, 1, 2, .. N`
///
/// It is equal to:
/// `len(sequence) * (min(sequence) + max(sequence)) / 2`
/// aka `(N + 1)*N/2`
fn arithmetic_series(max: u8) -> u16 {
	// because we start at 0, we add 1 to the max to get the length of the sequence
	let count = u16::from(max) + 1;
	count
		.checked_mul(u16::from(max))
		.unwrap()
		.checked_div(2)
		.unwrap()
}

#[derive(Default)]
struct Fields<T> {
	aka: T,
	vm: T,
	vr: T,
}

#[derive(Debug, thiserror::Error)]
pub enum AttrsToFieldsErr {
	#[error("attribute value was empty string or not present")]
	EmptyVal,
	#[error("failed to parse attribute key: {0}")]
	ParseKey(#[from] ParseKeyErr),
	#[error("skipped an index for field {0:?}")]
	SkippedAttrIdx(Field),
}

// Get ready for the most overengineered code ever...
fn attrs_to_fields(
	attrs: &HashMap<String, Option<String>>,
) -> Result<Fields<Vec<&str>>, AttrsToFieldsErr> {
	// We can detect gaps without allocating by ensuring that the sum of all keys
	// is equal to len(seq)*max(seq)/2 and that min(seq) == 0.
	//
	// (this is just the formula for a finite arithmetic series)
	#[derive(Default)]
	struct Stats {
		max: u8,
		min: u8,
		sum: u16,
		count: u8,
	}
	impl Stats {
		fn was_index_skipped(&self) -> bool {
			self.min != 0
				|| self.count != (self.max + 1)
				|| self.sum != arithmetic_series(self.max)
		}
	}

	let mut fstats: Fields<Stats> = Default::default();
	let mut fvalues: Fields<Vec<&str>> = Default::default();
	for (k, v) in attrs.iter() {
		let v = v.as_deref().unwrap_or_default();
		if v.is_empty() {
			return Err(AttrsToFieldsErr::EmptyVal);
		}

		let (field, idx) = parse_field_from_key(k)?;
		let (stats, values) = match field {
			Field::Aka => (&mut fstats.aka, &mut fvalues.aka),
			Field::Vm => (&mut fstats.vm, &mut fvalues.vm),
			Field::Vr => (&mut fstats.vr, &mut fvalues.vr),
		};
		stats.max = u8::max(stats.max, idx.0);
		stats.min = u8::min(stats.min, idx.0);
		stats.sum += u16::from(idx.0);
		stats.count += 1;
		values.resize(stats.count.into(), ""); // guarantees next command wont fail
		values[usize::from(idx.0)] = v;
	}

	if fstats.aka.was_index_skipped() {
		return Err(AttrsToFieldsErr::SkippedAttrIdx(Field::Aka));
	}
	if fstats.vm.was_index_skipped() {
		return Err(AttrsToFieldsErr::SkippedAttrIdx(Field::Vm));
	}
	if fstats.vr.was_index_skipped() {
		return Err(AttrsToFieldsErr::SkippedAttrIdx(Field::Vr));
	}
	// sanity: we already ensured there are no holes
	debug_assert!(!fvalues.aka.iter().any(|s| s.is_empty()), "sanity");
	debug_assert!(!fvalues.vm.iter().any(|s| s.is_empty()), "sanity");
	debug_assert!(!fvalues.vr.iter().any(|s| s.is_empty()), "sanity");

	Ok(fvalues)
}
