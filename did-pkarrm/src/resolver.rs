use std::{
	collections::{BTreeMap, HashMap, HashSet},
	num::ParseIntError,
	str::FromStr,
};

use bitflags::bitflags;
use fluent_uri::Uri;
use pkarr::dns::rdata::TXT;

/// A verification method most typically is a public key (via `did:key`), or a Did Url
/// that links to a verification method in a different Did Document.
#[derive(Debug, Eq, PartialEq, Clone)]
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
		let uri: Uri<String> = Uri::try_from(s.to_owned())?;
		let did = Did::try_from(uri)?;
		Ok(Self::from(did))
	}
}

impl TryFrom<String> for VerificationMethod {
	type Error = ParseVerificationMethodErr;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let uri: Uri<String> = Uri::try_from(value)?;
		let did = Did::try_from(uri)?;

		Ok(Self::from(did))
	}
}

impl From<Did> for VerificationMethod {
	fn from(value: Did) -> Self {
		let (prefix, _suffix) = value
			.0
			.path()
			.split_once(':')
			.expect("already checked for did: prefix");

		if prefix == "key" {
			Self::DidKey(value)
		} else {
			Self::DidUrl(value)
		}
	}
}

impl<T: AsRef<str>> PartialEq<T> for VerificationMethod {
	fn eq(&self, other: &T) -> bool {
		let did = match self {
			VerificationMethod::DidKey(did) => did,
			VerificationMethod::DidUrl(did) => did,
		};

		did == other
	}
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Did(Uri<String>);

impl Did {
	pub fn as_uri(&self) -> &Uri<String> {
		&self.0
	}
}

#[derive(Debug, thiserror::Error)]
pub enum DidFromUriErr {
	#[error("did not start with `did:`")]
	WrongPrefix,
}

impl TryFrom<Uri<String>> for Did {
	type Error = DidFromUriErr;

	fn try_from(value: Uri<String>) -> Result<Self, Self::Error> {
		if value.scheme().as_str() == "did" && value.authority().is_none() {
			Ok(Self(value))
		} else {
			Err(DidFromUriErr::WrongPrefix)
		}
	}
}

impl<T: AsRef<str>> PartialEq<T> for Did {
	fn eq(&self, other: &T) -> bool {
		self.0 == other.as_ref()
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ParseVerificationMethodErr {
	#[error("not a uri")]
	NotAUri(#[from] fluent_uri::error::ParseError<String>),
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
	#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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

	fn from_str(_s: &str) -> Result<Self, Self::Err> {
		todo!()
	}
}

/// Everything in a did:pkarrm's Did Document except the `id` field. A
/// `DidDocumentContents` can be mapped 1:1 to a DNS txt record, for use in PKARR.
///
/// The generics are simply to enable borrowed data, they can be `&str` or `String`.
/// See [fluent_uri] for more info.
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
	#[error("failed to extract fields from attributes")]
	AttrsToFields(#[from] AttrsToFieldsErr),
	#[error("failed to parse aka string")]
	AkaParseErr(#[from] fluent_uri::error::ParseError<String>),
	#[error("failed to parse vm string")]
	VmParseErr(#[from] ParseVerificationMethodErr),
	#[error("failed to parse vr string")]
	VrParseErr(#[from] ParseVerificationRelationshipErr),
	#[error("failed to decode verification relationship using base32z")]
	VrNotB32z,
	#[error("failed to assemble attrs into lists")]
	ListAssembly(#[from] ListAssemblyErr),
}

impl TryFrom<TXT<'_>> for DidDocumentContents {
	type Error = FromTxtRecordErr;

	fn try_from(value: TXT<'_>) -> Result<Self, Self::Error> {
		let mut attrs = value.attributes();
		if attrs.len() >= usize::from(u8::MAX) {
			return Err(FromTxtRecordErr::TooManyAttrs);
		}

		let mut novalue = HashSet::new();
		let mut singleton = HashMap::new();
		let mut varlen = HashMap::new();
		assemble_into_lists(&mut attrs, &mut novalue, &mut singleton, &mut varlen)?;

		let aka: Vec<String> = varlen.remove("aka").unwrap_or_default();
		let aka: Result<Vec<Uri<String>>, _> =
			aka.into_iter().map(Uri::try_from).collect();
		let aka = aka?;

		let vm: Vec<String> = varlen.remove("vm").unwrap_or_default();
		let vm: Result<Vec<VerificationMethod>, _> =
			vm.into_iter().map(VerificationMethod::try_from).collect();
		let vm = vm?;

		let vr: String = singleton.remove("vr").unwrap_or_default();
		let vr: Vec<VerificationRelationship> = b32z_decode(&vr)
			.map_err(|()| FromTxtRecordErr::VrNotB32z)?
			.into_iter()
			.map(VerificationRelationship::from_bits_truncate)
			.collect();

		Ok(Self { aka, vm, vr })
	}
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ListAssemblyErr {
	#[error("key suffix could not be parsed into a u8")]
	KeySuffixNotU8(#[from] ParseIntError),
	#[error("skipped an index for the keys")]
	SkippedIndex,
	#[error("index was encountered twice for same key")]
	DuplicateIndex,
}

/// Pops any attrs in the format key0=a, key1=b, and turns them into key=[a,b], and
/// moves them into `out_varlen`.
///
/// Attrs in the form key=a are moved into `out_singleton`.
///
/// Attrs with no value are moved into `out_novalue`.
///
/// `attrs` should be fully drained at the end of it, but with its original capacity.
///
/// `scratch` is just scratch space, to avoid needing to reallocate.
fn assemble_into_lists(
	attrs: &mut HashMap<String, Option<String>>,
	out_novalue: &mut HashSet<String>,
	out_singleton: &mut HashMap<String, String>,
	out_varlen: &mut HashMap<String, Vec<String>>,
) -> Result<(), ListAssemblyErr> {
	out_novalue.clear();
	out_singleton.clear();
	out_varlen.clear();

	// Hopefully this steals the buffer to reuse it
	let mut out_varlen_wip: HashMap<String, BTreeMap<u8, String>> =
		std::mem::take(out_varlen)
			.into_keys()
			.map(|k| (k, BTreeMap::new()))
			.collect();

	for (mut k, v) in attrs.drain() {
		let Some(v) = v else {
			out_novalue.insert(k);
			continue;
		};
		let (prefix, suffix_num) = split_off_number(&k)?;
		let Some(suffix_num) = suffix_num else {
			out_singleton.insert(k, v);
			continue;
		};
		k.truncate(prefix.len()); // truncate to only prefix
		let values = out_varlen_wip.entry(k).or_default();
		let already_exists = values.insert(suffix_num, v).is_some();
		if already_exists {
			return Err(ListAssemblyErr::DuplicateIndex);
		}
	}

	// now collapse the varlen
	let out_varlen_wip: Result<HashMap<String, Vec<String>>, ListAssemblyErr> =
		out_varlen_wip
			.into_iter()
			.map(|(k, bt)| {
				let mut vec: Vec<String> = Vec::new();
				for (i, v) in bt {
					if usize::from(i) != vec.len() {
						return Err(ListAssemblyErr::SkippedIndex);
					}
					vec.push(v);
				}
				Ok((k, vec))
			})
			.collect();
	*out_varlen = out_varlen_wip?;

	Ok(())
}

fn split_off_number(s: &str) -> Result<(&str, Option<u8>), std::num::ParseIntError> {
	let Some(first_digit) = s.find(|ch: char| ch.is_ascii_digit()) else {
		return Ok((s, None));
	};
	let (prefix, suffix) = s.split_at(first_digit);
	let num: u8 = suffix.parse()?;

	Ok((prefix, Some(num)))
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Field {
	Aka,
	Vm,
	Vr,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseKeyErr {
	#[error("key suffix was not a u8")]
	SuffixNotANumber(#[from] std::num::ParseIntError),
	#[error("unknown attribute key")]
	UnknownKey,
}

#[derive(Debug, thiserror::Error)]
pub enum AttrsToFieldsErr {
	#[error("attribute value was empty string or not present")]
	EmptyVal,
	#[error("failed to parse attribute key")]
	ParseKey(#[from] ParseKeyErr),
	#[error("skipped an index for field {0:?}")]
	SkippedAttrIdx(Field),
	#[error("value for field {0:?} was not base32-z")]
	ValueNotBase32(Field),
}

fn b32z_decode(s: &str) -> Result<Vec<u8>, ()> {
	base32::decode(base32::Alphabet::Z, s).ok_or(())
}

#[cfg(test)]
mod test {
	use eyre::Context;

	use super::*;

	fn b32z(data: &[u8]) -> String {
		base32::encode(base32::Alphabet::Z, data)
	}

	fn make_txt_record<'a, AKA, VM>(aka: AKA, vm: VM, vr: &str) -> TXT<'static>
	where
		AKA: IntoIterator<Item = &'a str>,
		VM: IntoIterator<Item = &'a str>,
	{
		let mut txt = TXT::new();
		for (i, s) in aka.into_iter().enumerate() {
			let cs = format!("aka{i}={s}").try_into().unwrap();
			txt.add_char_string(cs);
		}
		for (i, s) in vm.into_iter().enumerate() {
			let cs = format!("vm{i}={s}").try_into().unwrap();
			txt.add_char_string(cs);
		}
		let cs = format!("vr={vr}").try_into().unwrap();
		txt.add_char_string(cs);

		txt
	}

	#[test]
	fn test_txt_record_conversion() -> eyre::Result<()> {
		// Arrange
		let aka0 = "at://atproto.com";
		let vm0 = "did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw";
		let vr0 = VerificationRelationship::Authentication;
		let txt = make_txt_record([aka0], [vm0], &b32z(&[vr0.bits()]));

		// Sanity checks
		let attrs = dbg!(txt.attributes());
		assert_eq!(attrs["aka0"].as_deref(), Some(aka0));
		assert_eq!(attrs["vm0"].as_deref(), Some(vm0));
		assert_eq!(b32z_decode(attrs["vr"].as_ref().unwrap()).unwrap(), &[0x1]);

		// Act
		let doc: DidDocumentContents = txt
			.try_into()
			.wrap_err("error while converting TXT record to DID document contents")?;

		// Assert
		assert_eq!(doc.aka, [aka0]);
		assert_eq!(doc.vm, [vm0]);
		assert_eq!(doc.vr, [vr0]);

		Ok(())
	}
}
