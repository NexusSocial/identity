use std::str::FromStr;

use ascii::{AsciiChar, AsciiStr, AsciiString};

const MAX_LENGTH: u8 = 253;
const MIN_SEGMENT_LENGTH: u8 = 1;
const MAX_SEGMENT_LENGTH: u8 = 63;

const EMPTY_ASCII: &AsciiString = &AsciiString::new();

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
	#[error("the handle must be no longer than {MAX_LENGTH} characters")]
	TooLong,
	#[error("the handle does not have at least two . separated segments")]
	NotEnoughSegments,
	#[error(
		"one of the handle's segments was shorter than {MIN_SEGMENT_LENGTH} characters"
	)]
	SegmentTooShort,
	#[error(
		"one of the handle's segments was longer than {MAX_SEGMENT_LENGTH} characters"
	)]
	SegmentTooLong,

	#[error("the handle cannot start with a dot")]
	StartingDot,
	#[error("the handle cannot end with a dot")]
	EndingDot,
	#[error("the handle cannot start with a dot")]
	StartingHyphen,
	#[error("the handle cannot end with a dot")]
	EndingHyphen,

	#[error("the only allowed characters are letters, digits, and hyphens, {0} is not allowed")]
	InvalidCharacter(char),
	#[error("the last segment (the TLD) cannot start with a digit")]
	TldStartsWithDigit,
	#[error("the last segment (the TLD) is disallowed")]
	TldDisallowed,
}

/// Precondition: tld is lower case
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

impl TryFrom<LowercaseAscii> for Handle {
	type Error = InvalidHandle;

	fn try_from(s: LowercaseAscii) -> Result<Self, Self::Error> {
		if s.len() > MAX_LENGTH.into() {
			return Err(InvalidHandle::TooLong);
		}
		if s.len() == 0 {
			return Err(InvalidHandle::NotEnoughSegments);
		}

		// Wont panic because we already checked that it is a valid hostname, which means ascii.
		let first_char = s[0];
		let last_char = s.last().expect("already confirmed non-zero length");
		if first_char == b'.' {
			return Err(InvalidHandle::StartingDot);
		};
		if first_char == b'-' {
			return Err(InvalidHandle::StartingHyphen);
		}
		if last_char == b'.' {
			return Err(InvalidHandle::EndingDot);
		};
		if last_char == b'-' {
			return Err(InvalidHandle::EndingHyphen);
		}

		let mut segment_count = 0;
		let mut last_segment = EMPTY_ASCII.as_ref();
		for segment in s.split(AsciiChar::Dot) {
			segment_count += 1;
			last_segment = segment;
			if segment.len() > MAX_SEGMENT_LENGTH.into() {
				return Err(InvalidHandle::SegmentTooLong);
			}
			if segment.len() < MIN_SEGMENT_LENGTH.into() {
				return Err(InvalidHandle::SegmentTooShort);
			}

			for char in segment {
				if !char.is_ascii_alphanumeric() && *char != AsciiChar::Minus {
					return Err(InvalidHandle::InvalidCharacter(char.as_char()));
				}
			}
		}

		if segment_count < 2 {
			return Err(InvalidHandle::NotEnoughSegments);
		}
		if is_reserved_tld(&last_segment) {
			return Err(InvalidHandle::TldDisallowed);
		}
		if last_segment.as_bytes()[0].is_ascii_digit() {
			return Err(InvalidHandle::TldStartsWithDigit);
		}

		Ok(Self(s))
	}
}

impl FromStr for Handle {
	type Err = InvalidHandle;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let ascii = AsciiString::from_ascii(s).map_err(|err| {
			debug_assert_ne!(
				s.len(),
				0,
				"sanity check: an empty string would have been ascii"
			);
			let invalid_char = s
				.split_at(err.ascii_error().valid_up_to())
				.1
				.chars()
				.next()
				.unwrap();
			InvalidHandle::InvalidCharacter(invalid_char)
		})?;
		let lower = LowercaseAscii::from(ascii);
		Self::try_from(lower)
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
			"Bingus.Bongus.gov",
			"i-have-.some-hyphens.com",
			"l33t.h4x0r.com",
			//"苹果.com",
		] {
			let ascii = AsciiStr::from_ascii(h).unwrap();
			assert_eq!(
				h.parse::<Handle>(),
				Ok(Handle(LowercaseAscii::from(ascii))),
				"{h} failed",
			)
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
				Err(InvalidHandle::InvalidCharacter('_')),
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
		assert_eq!(s.parse::<Handle>(), Err(InvalidHandle::TooLong));
	}

	#[test]
	fn reject_too_few_segments() {
		assert_eq!(
			"foobar".parse::<Handle>(),
			Err(InvalidHandle::NotEnoughSegments)
		);
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
		assert_eq!(s.parse::<Handle>(), Err(InvalidHandle::SegmentTooLong));

		assert_eq!(
			"a..b".parse::<Handle>(),
			Err(InvalidHandle::SegmentTooShort)
		);
	}

	#[test]
	fn reject_starting_and_ending_dots_and_hyphens() {
		// Test starting dots
		assert_eq!(
			".example.com".parse::<Handle>(),
			Err(InvalidHandle::StartingDot)
		);
		// Test ending dots
		assert_eq!(
			"example.com.".parse::<Handle>(),
			Err(InvalidHandle::EndingDot)
		);
		// Test starting hyphens
		assert_eq!(
			"-example.com".parse::<Handle>(),
			Err(InvalidHandle::StartingHyphen)
		);
		// Test ending hyphens
		assert_eq!(
			"example.com-".parse::<Handle>(),
			Err(InvalidHandle::EndingHyphen)
		);
	}

	#[test]
	fn reject_tld_starting_with_digit() {
		assert_eq!(
			"example.1com".parse::<Handle>(),
			Err(InvalidHandle::TldStartsWithDigit)
		);
		assert_eq!(
			"foo.bar.123".parse::<Handle>(),
			Err(InvalidHandle::TldStartsWithDigit)
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
			Err(InvalidHandle::TldStartsWithDigit)
		);
		assert_eq!(
			"192.168.1.1".parse::<Handle>(),
			Err(InvalidHandle::TldStartsWithDigit)
		);

		// Test IPv6 addresses
		assert_eq!(
			"[::1]".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('['))
		);
		assert_eq!(
			"[2001:db8::1]".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('['))
		);
	}

	#[test]
	fn reject_special_chars() {
		// Unicode characters
		assert_eq!(
			"héllo.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('é'))
		);
		assert_eq!(
			"hello.cøm".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('ø'))
		);
		assert_eq!(
			"你好.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('你'))
		);

		// ASCII control codes
		assert_eq!(
			"hello\u{0000}.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('\u{0000}'))
		);
		assert_eq!(
			"test\u{001F}.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('\u{001F}'))
		);

		// Symbols
		assert_eq!(
			"hello!.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('!'))
		);
		assert_eq!(
			"hello@.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('@'))
		);
		assert_eq!(
			"hello$.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('$'))
		);
		assert_eq!(
			"hello%.com".parse::<Handle>(),
			Err(InvalidHandle::InvalidCharacter('%'))
		);
	}
}
