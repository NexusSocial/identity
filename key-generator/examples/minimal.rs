use key_generator::Ascii;

fn main() {
	let phrase = key_generator::RecoveryPhrase::builder()
		.entropy([13; 32])
		.password(Ascii::EMPTY)
		.build();
	let signing_key = phrase.to_key(Ascii::EMPTY, 0).expect("password is correct");
	println!("{}: {}", "recovery phrase", phrase.as_display());
	println!("{}: {:?}", "recovery key", signing_key);

	let exports = phrase.export("Basis");
	println!("{exports:?}");
}
