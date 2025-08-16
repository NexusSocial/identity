use std::{
	collections::{BTreeMap, HashMap, HashSet},
	fmt::{Display, Write},
	num::ParseIntError,
};

use base64::Engine;
use fluent_uri::Uri;
use pkarr::dns::{CharacterString, rdata::TXT};

use super::{
	b64_dec,
	vmethod::{ParseVerificationMethodErr, VerificationMethod},
	vrelationship::{ParseVerificationRelationshipErr, VerificationRelationship},
};

/// Everything in a did:pkarr's Did Document except the `id` field. A
/// `DidDocumentContents` can be mapped 1:1 to a DNS txt record, for use in PKARR.
///
/// The generics are simply to enable borrowed data, they can be `&str` or `String`.
/// See [fluent_uri] for more info.
#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct DidDocumentContents {
	/// "Also Known As". A list of alternative aliases for the user.
	/// <https://www.w3.org/TR/cid-1.0/#also-known-as>
	pub aka: Vec<Uri<String>>,
	/// The [VerificationMethod]s. Index in vec matches `vr`.
	pub vm: Vec<VerificationMethod>,
	/// The [VerificationRelationship]s. The index in the vec matches
	/// `vm`.
	pub vr: Vec<VerificationRelationship>,
}

impl DidDocumentContents {
	pub fn to_txt_record(&self) -> TXT<'static> {
		// Had to use fn instead of closure because no impl T in closures
		fn populate_txt_from_iter(
			sbuf: &mut String,
			txt: &mut TXT,
			key_prefix: &str,
			it: impl Iterator<Item = impl Display>,
		) {
			for (key_idx, v) in it.into_iter().enumerate() {
				sbuf.clear();
				write!(sbuf, "{key_prefix}{key_idx}={v}").unwrap();
				// We use the string buffer because CharacterString copies
				// causing us to unecessarily drop buffers just to reallocate them.
				let cs = CharacterString::new(sbuf.as_bytes())
					.expect("TODO: is this always infallbile?")
					.into_owned();
				txt.add_char_string(cs);
			}
		}

		let mut txt = TXT::new();
		let mut sbuf = String::new();
		populate_txt_from_iter(&mut sbuf, &mut txt, "aka", self.aka.iter());
		populate_txt_from_iter(&mut sbuf, &mut txt, "vm", self.vm.iter());

		// Populate vr attr
		{
			let vr_as_bytes: &[u8] = bytemuck::cast_slice(self.vr.as_slice());
			sbuf.clear();
			sbuf.push_str("vr=");
			base64::prelude::BASE64_URL_SAFE_NO_PAD
				.encode_string(vr_as_bytes, &mut sbuf);
			let cs = CharacterString::new(sbuf.as_bytes())
				.expect("TODO: is this always infallbile?")
				.into_owned();
			txt.add_char_string(cs);
		}

		debug_assert!(
			txt.clone().long_attributes().unwrap().keys().is_sorted(),
			"all keys should be alphabetically sorted"
		);

		txt
	}
}

#[derive(Debug, thiserror::Error)]
#[error("failed to extract information from dns txt record")]
pub enum FromTxtRecordErr {
	#[error("encountered too many attributes")]
	TooManyAttrs,
	#[error("failed to parse aka string")]
	AkaParseErr(#[from] ParseAlsoKnownAsErr),
	#[error("failed to parse vm string")]
	VmParseErr(#[from] ParseVerificationMethodErr),
	#[error("failed to parse vr string")]
	VrParseErr(#[from] ParseVerificationRelationshipErr),
	#[error("failed to assemble attrs into lists")]
	ListAssembly(#[from] ListAssemblyErr),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ParseAlsoKnownAsErr(#[from] fluent_uri::error::ParseError<String>);

impl TryFrom<TXT<'_>> for DidDocumentContents {
	type Error = FromTxtRecordErr;

	fn try_from(value: TXT<'_>) -> Result<Self, Self::Error> {
		Self::try_from(&value)
	}
}

impl TryFrom<&TXT<'_>> for DidDocumentContents {
	type Error = FromTxtRecordErr;

	fn try_from(value: &TXT<'_>) -> Result<Self, Self::Error> {
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
		let aka = aka.map_err(ParseAlsoKnownAsErr)?;

		let vm: Vec<String> = varlen.remove("vm").unwrap_or_default();
		let vm: Result<Vec<VerificationMethod>, _> =
			vm.into_iter().map(VerificationMethod::try_from).collect();
		let vm = vm?;

		let vr: String = singleton.remove("vr").unwrap_or_default();
		let vr: Vec<VerificationRelationship> = b64_dec(&vr)
			.map_err(ParseVerificationRelationshipErr::from)?
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

/// Pops any attrs in the format `key0=a`, `key1=b`, and turns them into `key=[a,b]`,
/// and moves them into `out_varlen`.
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

#[cfg(test)]
mod test {
	use super::*;
	use eyre::Context;

	fn b64_enc(data: &[u8]) -> String {
		base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(data)
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

		assert!(
			txt.clone().long_attributes().unwrap().keys().is_sorted(),
			"sanity: keys should be alphabetically sorted"
		);

		txt
	}

	#[test]
	fn test_txt_record_conversion() -> eyre::Result<()> {
		// Arrange
		let aka0 = "at://atproto.com";
		let vm0 = "did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw";
		let vr0 = VerificationRelationship::Authentication;
		let original_txt = make_txt_record([aka0], [vm0], &b64_enc(&[vr0.bits()]));
		let expected_doc = DidDocumentContents {
			aka: vec![Uri::parse(aka0).unwrap().to_owned()],
			vm: vec![vm0.parse().unwrap()],
			vr: vec![vr0],
		};

		// Sanity: expected TXT attributes
		{
			let attrs = dbg!(original_txt.attributes());
			assert_eq!(attrs["aka0"].as_deref(), Some(aka0));
			assert_eq!(attrs["vm0"].as_deref(), Some(vm0));
			assert_eq!(b64_dec(attrs["vr"].as_ref().unwrap()).unwrap(), &[0x1]);
		}
		// Sanity: expected DidDocumentContents
		{
			assert_eq!(expected_doc.aka, vec![aka0]);
			assert_eq!(expected_doc.vm, vec![vm0]);
			assert_eq!(expected_doc.vr, vec![vr0]);
		}

		// Act: txt -> doc
		let doc: DidDocumentContents = original_txt
			.clone()
			.try_into()
			.wrap_err("error while converting TXT record to DID document contents")?;

		// Assert: document matches inputs
		assert_eq!(doc.aka, [aka0]);
		assert_eq!(doc.vm, [vm0]);
		assert_eq!(doc.vr, [vr0]);
		assert_eq!(doc, expected_doc, "(txt -> doc) != expected_doc");
		assert_eq!(expected_doc, doc, "(txt -> doc) != expected_doc");

		// Act: doc -> txt
		let roundtripped_txt: TXT<'static> = doc.to_txt_record();

		// Assert: txt round tripped successfully
		assert_eq!(roundtripped_txt, original_txt);
		assert_eq!(roundtripped_txt.attributes(), original_txt.attributes());
		assert_eq!(
			roundtripped_txt.clone().long_attributes(),
			original_txt.clone().long_attributes()
		);
		assert_eq!(
			String::try_from(roundtripped_txt).unwrap(),
			String::try_from(original_txt).unwrap()
		);

		Ok(())
	}
}
