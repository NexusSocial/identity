use font8x8::{BASIC_FONTS, UnicodeFonts};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

const GLYPH_W: u32 = 8;
const GLYPH_H: u32 = 8;
const SPACING: u32 = 1; // 1px between characters
const PAD: u32 = 4; // border padding

fn main() -> Result<(), Box<dyn Error>> {
	let text = "hello world";

	let chars: Vec<char> = text.chars().collect();
	let width = PAD * 2 + (chars.len() as u32) * (GLYPH_W + SPACING) - SPACING;
	let height = PAD * 2 + GLYPH_H;

	// Grayscale buffer (0 = black, 255 = white)
	let mut img = vec![0u8; (width * height) as usize];

	for (i, ch) in chars.iter().enumerate() {
		// Look up an 8x8 glyph; unknown chars render as blanks.
		if let Some(glyph) = BASIC_FONTS.get(*ch) {
			let x0 = PAD + (i as u32) * (GLYPH_W + SPACING);
			let y0 = PAD;

			for (row_idx, row_bits) in glyph.iter().enumerate() {
				for col in 0..8 {
					// In font8x8, bit 0 is the left-most pixel.
					if (row_bits >> col) & 1 == 1 {
						let x = x0 + col as u32;
						let y = y0 + row_idx as u32;
						img[(y * width + x) as usize] = 255; // white pixel
					}
				}
			}
		}
	}

	// Encode as PNG (8-bit grayscale)
	let file = File::create("hello_font8x8.png")?;
	let w = BufWriter::new(file);
	let mut encoder = png::Encoder::new(w, width, height);
	encoder.set_color(png::ColorType::Grayscale);
	encoder.set_depth(png::BitDepth::Eight);
	let mut writer = encoder.write_header()?;
	writer.write_image_data(&img)?;
	println!("Wrote hello_font8x8.png ({}x{})", width, height);
	Ok(())
}
