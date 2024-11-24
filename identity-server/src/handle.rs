pub struct Lowercase<S: AsRef<str>>(pub S);

/// These requirements come directly from ATProto:
/// https://atproto.com/specs/handle#handle-identifier-syntax
#[derive(Debug, thiserror::Error)]
pub enum InvalidHandle {
	#[error("the handle must be no longer than 253 characters")]
	TooLong,
	#[error("the handle does not have at least two . separated segments")]
	NotEnoughSegments,
	#[error("one of the handle's segments was shorter than 1 character")]
	SegmentTooShort,
	#[error("one of the handle's segments was longer than 63 characters")]
	SegmentTooLong,

	#[error("the handle cannot start with a dot")]
	StartingDot,
	#[error("the handle cannot end with a dot")]
	EndingDot,
	#[error("the handle cannot start with a dot")]
	StartingHyphen,
	#[error("the handle cannot end with a dot")]
	EndingHyphen,

	#[error("the only allowed characters are letters, digits, and hyphens")]
	InvalidCharacter,
	#[error("the last segment (the TLD) cannot start with a digit")]
	TldStartsWithDigit,
	#[error("the last segment (the TLD) is disallowed")]
	TldDisallowed,
	#[error(transparent)]
	InvalidHostname(#[from] url::ParseError),
	#[error("IP addresses are not allowed")]
	IpAddress,
}

fn is_reserved_tld<S: AsRef<str>>(tld: Lowercase<S>) -> bool {
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
	.any(|banned| banned == tld.0.as_ref())
}

pub struct Handle<S: AsRef<str>>(S);

impl<S: AsRef<str>> AsRef<str> for Handle<S> {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
	}
}

impl<S: AsRef<str>> TryFrom<Lowercase<S>> for Handle<S> {
	type Error = InvalidHandle;

	fn try_from(s: Lowercase<S>) -> Result<Self, Self::Error> {
		let handle_ref = s.0.as_ref();
		if handle_ref.len() > 253 {
			return Err(InvalidHandle::TooLong);
		}
		let url::Host::Domain(_) =
			url::Host::parse(handle_ref).map_err(InvalidHandle::InvalidHostname)?
		else {
			return Err(InvalidHandle::IpAddress);
		};
		debug_assert!(handle_ref.is_ascii(), "sanity check");

		// Wont panic because we already checked that it is a valid hostname, which means ascii.
		let first_char = *handle_ref.as_bytes().first().unwrap();
		let last_char = *handle_ref.as_bytes().last().unwrap();
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
		let mut last_segment = "";
		for segment in handle_ref.split(".") {
			segment_count += 1;
			last_segment = segment;
			if segment.len() > 63 {
				return Err(InvalidHandle::SegmentTooLong);
			}
			if segment.len() < 1 {
				return Err(InvalidHandle::SegmentTooShort);
			}

			for c in segment.bytes() {
				if !c.is_ascii_alphanumeric() || c != b'-' {
					return Err(InvalidHandle::InvalidCharacter);
				}
			}
		}

		if segment_count < 2 {
			return Err(InvalidHandle::NotEnoughSegments);
		}
		if is_reserved_tld(Lowercase(last_segment)) {
			return Err(InvalidHandle::TldDisallowed);
		}
		if last_segment.as_bytes()[0].is_ascii_digit() {
			return Err(InvalidHandle::TldStartsWithDigit);
		}

		Ok(Self(s.0))
	}
}
#[cfg(test)]
mod test {
	// TODO: Write tests
}
