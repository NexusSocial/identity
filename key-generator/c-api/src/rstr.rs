#[repr(C)]
pub struct key_gen_RStr {
	pub data: *const u8,
	pub length: usize,
}

impl From<&'static str> for key_gen_RStr {
	fn from(value: &'static str) -> Self {
		Self {
			data: value.as_ptr(),
			length: value.len(),
		}
	}
}
