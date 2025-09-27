#![no_std]
#![forbid(improper_ctypes)]
#![forbid(improper_ctypes_definitions)]

#[cfg(feature = "export-pdf")]
extern crate alloc;

mod rstr;

#[cfg(feature = "export-pdf")]
mod rvec;

#[cfg(feature = "export-pdf")]
use crate::rvec::key_gen_RVec as RVec;

use key_generator::{Ascii, RecoveryPhrase};

use crate::rstr::key_gen_RStr as RStr;

#[repr(C)]
pub struct key_gen_private_key {
	pub bytes: [u8; 32],
}

#[repr(C)]
pub struct key_gen_phrase {
	pub entropy: [u8; 32],
	pub words: [RStr; 24],
}

#[unsafe(no_mangle)]
pub extern "C" fn key_gen_make_phrase(entropy: &[u8; 32]) -> key_gen_phrase {
	let entropy = *entropy;
	let phrase = RecoveryPhrase::builder().entropy(entropy).build();
	let words: [RStr; 24] = phrase.to_words().map(RStr::from);

	key_gen_phrase { entropy, words }
}

#[unsafe(no_mangle)]
pub extern "C" fn key_gen_compute_key(
	phrase: &key_gen_phrase,
	account: u16,
) -> key_gen_private_key {
	let phrase = RecoveryPhrase::builder().entropy(phrase.entropy).build();
	let private_key = phrase.to_key(Ascii::EMPTY, account).expect("infallible");

	key_gen_private_key {
		bytes: private_key.0,
	}
}

#[cfg(feature = "export-pdf")]
#[repr(C)]
pub struct key_gen_exports {
	pub pdf_contents: RVec,
	pub svg_contents: RVec, // guaranteed to be utf8
}

/// # Safety
/// `app_name` must be null terminated.
#[cfg(feature = "export-pdf")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn key_gen_export(
	phrase: &key_gen_phrase,
	app_name: *const core::ffi::c_char,
) -> key_gen_exports {
	let phrase = RecoveryPhrase::builder().entropy(phrase.entropy).build();
	let app_name = unsafe { core::ffi::CStr::from_ptr(app_name) };
	let app_name = app_name.to_string_lossy();
	let exports = phrase.export(&app_name);

	key_gen_exports {
		pdf_contents: exports.pdf_contents.into(),
		svg_contents: exports.svg_contents.into_bytes().into(),
	}
}
