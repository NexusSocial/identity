use bitflags::bitflags;

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
	#[derive(Debug, Eq, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
	#[repr(C)]
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
	#[error("failed to decode verification relationship using base32z")]
	VrNotB64(#[from] base64::DecodeError),
}
