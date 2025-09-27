use key_generator::Ascii;

fn main() {
	let phrase = key_generator::RecoveryPhrase::builder()
		.random()
		.password(Ascii::EMPTY)
		.build();
	let signing_key = phrase.to_key(Ascii::EMPTY, 0).expect("password is correct");
	println!("recovery phrase: {}", phrase.as_display());
	println!("recovery key: {:?}", signing_key.0);

	let exports = phrase.export("Basis");
	println!("pdf length: {}", exports.pdf_contents.len());
	println!("svg length: {}", exports.svg_contents.len());
}
