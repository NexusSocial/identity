use std::str::FromStr;

use ascii::{AsciiStr, AsciiString};

#[allow(unused)]
const MAX_LENGTH: u8 = 253;
#[allow(unused)]
const MAX_SEGMENT_LENGTH: u8 = 63;

/// Any ascii-lowercase string
#[derive(Debug, Eq, PartialEq, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct LowercaseAscii(AsciiString);

impl TryFrom<String> for LowercaseAscii {
	type Error = ascii::FromAsciiError<String>;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Ok(Self::from(AsciiString::from_ascii(value)?))
	}
}

impl FromStr for LowercaseAscii {
	type Err = ascii::AsAsciiStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(AsciiStr::from_ascii(s)?))
	}
}

impl From<&AsciiStr> for LowercaseAscii {
	fn from(value: &AsciiStr) -> Self {
		LowercaseAscii(value.to_ascii_lowercase())
	}
}

impl From<AsciiString> for LowercaseAscii {
	fn from(mut value: AsciiString) -> Self {
		value.make_ascii_lowercase();
		Self(value)
	}
}

impl AsRef<str> for LowercaseAscii {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
	}
}

impl AsRef<[u8]> for LowercaseAscii {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

/// These requirements come directly from ATProto:
/// https://atproto.com/specs/handle#handle-identifier-syntax
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum InvalidHandle {
	#[error("failed idna conversion")]
	NotADomain,

	#[error("missing or invalid top-level domain")]
	TldInvalid,

	#[error("tld is reserved")]
	TldReserved,
}

/// Precondition: tld is lower case
/// See https://atproto.com/specs/handle#additional-non-syntax-restrictions
fn is_reserved_tld(tld: &AsciiStr) -> bool {
	debug_assert_eq!(tld, tld.to_ascii_lowercase(), "check precondition");
	[
		"alt",
		"arpa",
		"example",
		"internal",
		"invalid",
		"local",
		"localhost",
		"onion",
		"test",
	]
	.into_iter()
	.any(|banned| banned == tld)
}

#[derive(Debug, Eq, PartialEq, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct Handle(LowercaseAscii);

impl AsRef<str> for Handle {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
	}
}

impl AsRef<[u8]> for Handle {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

impl FromStr for Handle {
	type Err = InvalidHandle;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// PERF: This could use unsafe and be faster
		let ascii = AsciiString::from_ascii(
			idna::domain_to_ascii_strict(s).map_err(|_| InvalidHandle::NotADomain)?,
		)
		.unwrap();
		let Some(tld_idx) = ascii.as_str().rfind('.') else {
			return Err(InvalidHandle::TldInvalid);
		};
		if ascii[tld_idx + 1].is_ascii_digit() {
			return Err(InvalidHandle::TldInvalid);
		}
		let tld = &ascii[tld_idx + 1..];
		if is_reserved_tld(tld) {
			return Err(InvalidHandle::TldReserved);
		}

		debug_assert_eq!(ascii, ascii.to_ascii_lowercase());
		Ok(Self(LowercaseAscii(ascii)))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn happy_cases() {
		for h in [
			"foobar.example.com",
			"bow.ties.are.cool",
			"u.wu",
			"i-have.some-hyphens.com",
			"l33t.h4x0r.com",
		] {
			let ascii = AsciiString::from_ascii(h).unwrap();
			assert_eq!(
				h.parse::<Handle>(),
				Ok(Handle(LowercaseAscii::from(ascii))),
				"{h} failed",
			)
		}
	}

	#[test]
	fn upper_case_normalizes_to_lower_case() {
		for h in ["Bingus.Bongus.gov", "bInGuS.BOnGuS"] {
			assert_eq!(
				h.parse::<Handle>(),
				Ok(Handle(LowercaseAscii(
					h.to_ascii_lowercase().parse().unwrap()
				))),
				"{h} failed",
			);
		}
	}

	#[test]
	fn subdomains_that_start_with_underscore_shouldnt_parse() {
		for h in [
			"_foobar.example.com",
			"_foo._bar.example.com",
			"_foo.bar.example.com",
			"foo._bar.example.com",
		] {
			assert_eq!(
				h.parse::<Handle>(),
				Err(InvalidHandle::NotADomain),
				"{h} failed",
			)
		}
	}

	#[test]
	fn reject_too_long() {
		let mut s = String::from("a");
		s.push_str(&".a".repeat((MAX_LENGTH - 1) as usize / 2));
		assert_eq!(s.len(), MAX_LENGTH as usize, "sanity check");
		assert_eq!(
			s.parse::<Handle>(),
			Ok(Handle(LowercaseAscii(s.parse().unwrap()))),
			"maximum possible length works"
		);
		s.push('a');
		assert_eq!(s.parse::<Handle>(), Err(InvalidHandle::NotADomain));
	}

	#[test]
	fn reject_too_few_segments() {
		assert_eq!("foobar".parse::<Handle>(), Err(InvalidHandle::TldInvalid));
	}

	#[test]
	fn reject_incorrect_segment_lengths() {
		let mut s = String::from("a.");
		s.push_str(&"a".repeat(MAX_SEGMENT_LENGTH.into()));
		assert_eq!(
			s.parse::<Handle>(),
			Ok(Handle(LowercaseAscii(s.parse().unwrap())))
		);
		s.push('a');
		assert_eq!(s.parse::<Handle>(), Err(InvalidHandle::NotADomain));

		assert_eq!("a..b".parse::<Handle>(), Err(InvalidHandle::NotADomain));
	}

	#[test]
	fn reject_starting_and_ending_dots_and_hyphens() {
		// Test starting dots
		assert_eq!(
			".example.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
		// Test ending dots
		assert_eq!(
			"example.com.".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
		// Test starting hyphens
		assert_eq!(
			"-example.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
		// Test ending hyphens
		assert_eq!(
			"example.com-".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
	}

	#[test]
	fn reject_tld_starting_with_digit() {
		assert_eq!(
			"example.1com".parse::<Handle>(),
			Err(InvalidHandle::TldInvalid)
		);
		assert_eq!(
			"foo.bar.123".parse::<Handle>(),
			Err(InvalidHandle::TldInvalid)
		);
		// Ensure digits are allowed in other positions of TLD
		assert_eq!(
			"example.c0m".parse::<Handle>(),
			Ok(Handle(LowercaseAscii("example.c0m".parse().unwrap())))
		);
	}

	#[test]
	fn reject_ip_addresses() {
		// Test IPv4 addresses
		assert_eq!(
			"127.0.0.1".parse::<Handle>(),
			Err(InvalidHandle::TldInvalid)
		);
		assert_eq!(
			"192.168.1.1".parse::<Handle>(),
			Err(InvalidHandle::TldInvalid)
		);

		// Test IPv6 addresses
		assert_eq!("[::1]".parse::<Handle>(), Err(InvalidHandle::NotADomain));
		assert_eq!(
			"[2001:db8::1]".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
	}

	#[test]
	fn accept_utf8_as_punycode() {
		// Unicode characters
		assert_eq!(
			"héllo.com".parse::<Handle>(),
			Ok(Handle(LowercaseAscii("xn--hllo-bpa.com".parse().unwrap())))
		);
		assert_eq!(
			"hello.cøm".parse::<Handle>(),
			Ok(Handle(LowercaseAscii("hello.xn--cm-lka".parse().unwrap())))
		);
		assert_eq!(
			"你好.com".parse::<Handle>(),
			Ok(Handle(LowercaseAscii("xn--6qq79v.com".parse().unwrap())))
		);

		// ASCII control codes
		assert_eq!(
			"hello\u{0000}.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
		assert_eq!(
			"test\u{001F}.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);

		// Symbols
		assert_eq!(
			"hello!.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
		assert_eq!(
			"hello@.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
		assert_eq!(
			"hello$.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
		assert_eq!(
			"hello%.com".parse::<Handle>(),
			Err(InvalidHandle::NotADomain)
		);
	}

	#[test]
	fn unicode_based_domains_convert_via_punycode() {
		assert_eq!(
			"苹果.com".parse::<Handle>(),
			Ok(Handle(LowercaseAscii("xn--gtvz22d.com".parse().unwrap())))
		);
	}

	#[test]
	fn reject_reserved_tlds() {
		for tld in [
			"alt",
			"arpa",
			"example",
			"internal",
			"invalid",
			"local",
			"localhost",
			"onion",
			"test",
		] {
			assert_eq!(
				format!("foo.{tld}").parse::<Handle>(),
				Err(InvalidHandle::TldReserved),
				"faild on {tld}"
			)
		}
	}
}
