use pdf_writer::types::{LineCapStyle, LineJoinStyle};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str, TextStr};

const W: f32 = 800.0;
const H: f32 = 1000.0;

fn main() -> std::io::Result<()> {
	// Object ids
	let catalog = Ref::new(1);
	let pages = Ref::new(2);
	let page = Ref::new(3);
	let font_bold = Ref::new(4);
	let font_reg = Ref::new(5);
	let font_mono = Ref::new(6);
	let contents = Ref::new(7);

	let mut pdf = Pdf::new();

	// Root + page tree
	pdf.catalog(catalog).pages(pages);
	pdf.pages(pages).kids([page]).count(1);

	// Page & resources (fonts only; images optional)
	pdf.page(page)
		.parent(pages)
		.media_box(Rect::new(0.0, 0.0, W, H))
		.contents(contents)
		.resources()
		.fonts()
		.pair(Name(b"FB"), font_bold)
		.pair(Name(b"FR"), font_reg)
		.pair(Name(b"FM"), font_mono)
		.finish()
		.finish();

	// Built-in Type1 fonts (no embedding)
	pdf.type1_font(font_bold).base_font(Name(b"Helvetica-Bold"));
	pdf.type1_font(font_reg).base_font(Name(b"Helvetica"));
	pdf.type1_font(font_mono).base_font(Name(b"Courier"));

	// Draw everything into one content stream
	let mut c = Content::new();

	// --- helpers for color + coords ---
	let rgb = |hex: u32| -> (f32, f32, f32) {
		(
			((hex >> 16) & 0xff) as f32 / 255.0,
			((hex >> 8) & 0xff) as f32 / 255.0,
			(hex & 0xff) as f32 / 255.0,
		)
	};
	let y_text = |y_svg: f32| -> f32 { H - y_svg }; // SVG baseline -> PDF baseline
	let y_rect = |y_svg_top: f32, h: f32| -> f32 { H - y_svg_top - h }; // SVG top-left -> PDF bottom-left

	// Rounded rectangle path
	let mut rounded = |c: &mut Content, x: f32, y: f32, w: f32, h: f32, r: f32| {
		let k = 0.552_284_75_f32; // circle-to-bezier kappa
		let ox = r * k;
		let oy = r * k;

		c.move_to(x + r, y);
		c.line_to(x + w - r, y);
		c.cubic_to(x + w - r + ox, y, x + w, y + r - oy, x + w, y + r);
		c.line_to(x + w, y + h - r);
		c.cubic_to(
			x + w,
			y + h - r + oy,
			x + w - r + ox,
			y + h,
			x + w - r,
			y + h,
		);
		c.line_to(x + r, y + h);
		c.cubic_to(x + r - ox, y + h, x, y + h - r + oy, x, y + h - r);
		c.line_to(x, y + r);
		c.cubic_to(x, y + r - oy, x + r - ox, y, x + r, y);
		c.close_path();
	};

	// ---------- Title pill ----------
	let (fill_dr, fill_dg, fill_db) = rgb(0xb42a2a); // dark red
	let (stroke_dr, stroke_dg, stroke_db) = rgb(0x8f1f1f);
	let x = 30.0;
	let y = y_rect(30.0, 56.0);
	c.set_line_join(LineJoinStyle::RoundJoin)
		.set_line_width(1.5)
		.set_fill_rgb(fill_dr, fill_dg, fill_db)
		.set_stroke_rgb(stroke_dr, stroke_dg, stroke_db);
	rounded(&mut c, x, y, 740.0, 56.0, 28.0);
	c.close_fill_nonzero_and_stroke();

	// Title text: "Basis" (bold) + " Recovery Kit" (regular), centered at x=400, y=66
	let cx = 400.0;
	let cy = y_text(66.0);
	let title_left = "Basis";
	let title_right = " Recovery Kit";

	// crude width estimate for Helvetica-ish centering (good enough for this layout)
	let est_w_helv =
		|s: &str, size: f32| -> f32 { s.chars().count() as f32 * size * 0.52 };
	let total_w = est_w_helv(title_left, 28.0) + est_w_helv(title_right, 28.0);
	let mut tx = cx - total_w / 2.0;

	// bold part
	c.begin_text()
		.set_fill_rgb(1.0, 1.0, 1.0)
		.set_font(Name(b"FB"), 28.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, tx, cy])
		.show(Str(title_left.as_bytes()))
		.end_text();
	tx += est_w_helv(title_left, 28.0);

	// regular part
	c.begin_text()
		.set_fill_rgb(1.0, 1.0, 1.0)
		.set_font(Name(b"FR"), 28.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, tx, cy])
		.show(Str(title_right.as_bytes()))
		.end_text();

	// ---------- How-to paragraph ----------
	let howto1 = "Keep this sheet offline and never share it. Anyone with the phrase and the optional";
	let howto2 =
		"password can control your account. Print on durable paper and store securely.";

	c.begin_text()
		.set_fill_rgb(0.066, 0.066, 0.066)
		.set_font(Name(b"FR"), 16.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, 40.0, y_text(130.0)])
		.show(Str(howto1.as_bytes()))
		.end_text();

	c.begin_text()
		.set_fill_rgb(0.066, 0.066, 0.066)
		.set_font(Name(b"FR"), 16.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, 40.0, y_text(150.0)])
		.show(Str(howto2.as_bytes()))
		.end_text();

	// ---------- Outer red pill with caption ----------
	let (outer_fill_r, outer_fill_g, outer_fill_b) = rgb(0xfff5f5);
	let (outer_stroke_r, outer_stroke_g, outer_stroke_b) = rgb(0xf1b5b5);
	c.set_line_join(LineJoinStyle::RoundJoin)
		.set_line_width(1.5)
		.set_fill_rgb(outer_fill_r, outer_fill_g, outer_fill_b)
		.set_stroke_rgb(outer_stroke_r, outer_stroke_g, outer_stroke_b);
	let outer_y = y_rect(170.0, 240.0);
	rounded(&mut c, 30.0, outer_y, 740.0, 240.0, 24.0);
	c.close_fill_nonzero_and_stroke();

	// caption "Account Details (Secret)" centered at y=189
	let (cap_r, cap_g, cap_b) = rgb(0xbb4444); // #b44-ish
	c.begin_text()
		.set_fill_rgb(cap_r, cap_g, cap_b)
		.set_font(Name(b"FB"), 14.0);
	center_text(
		&mut c,
		Name(b"FB"),
		14.0,
		"Account Details (Secret)",
		400.0,
		y_text(189.0),
	);
	c.end_text();

	// ---------- Four row pill backgrounds ----------
	let (row_stroke_r, row_stroke_g, row_stroke_b) = rgb(0xe2e6ee);
	for (i, y_top) in [206.0, 256.0, 306.0, 356.0].into_iter().enumerate() {
		let y = y_rect(y_top, 36.0);
		c.set_line_width(1.0)
			.set_fill_rgb(1.0, 1.0, 1.0)
			.set_stroke_rgb(row_stroke_r, row_stroke_g, row_stroke_b);
		rounded(&mut c, 60.0, y, 500.0, 36.0, 18.0);
		c.close_fill_nonzero_and_stroke();
	}

	// ---------- Monospaced centered word rows ----------
	let rows = [
		"word01-word02-word03-word04-word05-word06",
		"word07-word08-word09-word10-word11-word12",
		"word13-word14-word15-word16-word17-word18",
		"word19-word20-word21-word22-word23-word24",
	];
	let ys = [228.0, 278.0, 328.0, 378.0];
	for (text, y_svg) in rows.iter().zip(ys) {
		c.begin_text()
			.set_fill_rgb(0.0, 0.0, 0.0)
			.set_font(Name(b"FM"), 14.0);
		center_text_mono_courier(&mut c, 14.0, text, 300.0, y_text(y_svg));
		c.end_text();
	}

	// ---------- QR placeholder ----------
	let (qr_stroke_r, qr_stroke_g, qr_stroke_b) = rgb(0xd7dbe3);
	c.set_line_width(1.0)
		.set_fill_rgb(1.0, 1.0, 1.0)
		.set_stroke_rgb(qr_stroke_r, qr_stroke_g, qr_stroke_b);
	let qx = 580.0;
	let qy = y_rect(200.0, 180.0);
	rounded(&mut c, qx, qy, 180.0, 180.0, 12.0);
	c.close_fill_nonzero_and_stroke();

	c.begin_text()
		.set_fill_rgb(0.47, 0.47, 0.47)
		.set_font(Name(b"FR"), 12.0);
	center_text(
		&mut c,
		Name(b"FR"),
		12.0,
		"QR code placeholder",
		670.0,
		y_text(392.0),
	);
	c.end_text();

	// ---------- Checkbox (checked) ----------
	// Square
	c.set_line_width(1.0)
		.set_stroke_rgb(0.2, 0.2, 0.2)
		.rect(40.0, y_rect(440.0, 18.0), 18.0, 18.0)
		.stroke();

	// Check mark path: (44,449) -> (49,455) -> (59,443)
	let p0 = (44.0, y_text(449.0));
	let p1 = (49.0, y_text(455.0));
	let p2 = (59.0, y_text(443.0));
	c.set_line_width(3.0)
		.set_line_cap(LineCapStyle::RoundCap)
		.set_line_join(LineJoinStyle::RoundJoin)
		.set_stroke_rgb(0.066, 0.066, 0.066)
		.move_to(p0.0, p0.1)
		.line_to(p1.0, p1.1)
		.line_to(p2.0, p2.1)
		.stroke();

	// Checkbox label
	c.begin_text()
		.set_fill_rgb(0.066, 0.066, 0.066)
		.set_font(Name(b"FR"), 14.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, 66.0, y_text(454.0)])
		.show(Str("Password protected".as_bytes()))
		.end_text();

	// ---------- Extra info for developers ----------
	c.begin_text()
		.set_fill_rgb(0.0, 0.0, 0.0)
		.set_font(Name(b"FB"), 14.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, 40.0, y_text(485.0)])
		.show(Str("extra info for developers".as_bytes()))
		.end_text();

	c.begin_text()
        .set_fill_rgb(0.47, 0.47, 0.47)
        .set_font(Name(b"FR"), 10.0)
        .set_text_matrix([1.0, 0.0, 0.0, 1.0, 40.0, y_text(505.0)])
        .show(Str("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".as_bytes()))
        .end_text();

	// Finalize content stream and write file
	pdf.stream(contents, &c.finish().into_vec());
	std::fs::write("target/basis-recovery.pdf", pdf.finish())?;
	Ok(())
}

// Centered text helper for proportional fonts (rough estimate; good enough here).
fn center_text(
	c: &mut Content,
	font: Name<'_>,
	size: f32,
	text: &str,
	cx: f32,
	cy: f32,
) {
	let est_w = text.chars().count() as f32 * size * 0.52;
	let x = cx - est_w / 2.0;
	c.set_font(font, size)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, x, cy])
		.show(Str(text.as_bytes()));
}

// Accurate centering for Courier (monospaced: width = 0.6 * font_size per glyph)
fn center_text_mono_courier(c: &mut Content, size: f32, text: &str, cx: f32, cy: f32) {
	let w = text.chars().count() as f32 * size * 0.6;
	let x = cx - w / 2.0;
	c.set_text_matrix([1.0, 0.0, 0.0, 1.0, x, cy])
		.show(Str(text.as_bytes()));
}
