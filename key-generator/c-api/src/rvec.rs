use alloc::vec::Vec;

/// An owned Vec<u8>
#[repr(C)]
pub struct key_gen_RVec {
	pub data: *mut u8,
	pub len: usize,
	pub capacity: usize,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn key_gen_RVec_drop(v: key_gen_RVec) {
	let _ = unsafe { Vec::from_raw_parts(v.data, v.len, v.capacity) };
}

impl From<Vec<u8>> for key_gen_RVec {
	fn from(value: Vec<u8>) -> Self {
		let len = value.len();
		let capacity = value.capacity();
		let data: &'static mut [u8] = value.leak();

		Self {
			data: data.as_mut_ptr(),
			len,
			capacity,
		}
	}
}
