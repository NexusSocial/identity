use pdf_writer::types::{LineCapStyle, LineJoinStyle};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};

const W: f32 = 800.0;
const H: f32 = 1000.0;

#[derive(Debug)]
struct Rgb {
	r: f32,
	g: f32,
	b: f32,
}

impl Rgb {
	const fn new(hex: u32) -> Self {
		Self {
			r: ((hex >> 16) & 0xff) as f32 / 255.0,
			g: ((hex >> 8) & 0xff) as f32 / 255.0,
			b: (hex & 0xff) as f32 / 255.0,
		}
	}
}

// Palette
const WHITE: Rgb = Rgb::new(0xFFFFFF);
const OFF_BLACK: Rgb = Rgb::new(0x111111);
const DARK_RED: Rgb = Rgb::new(0x8f1f1f);
const RED: Rgb = Rgb::new(0xb42a2a);
const LIGHT_RED: Rgb = Rgb::new(0xfff5f5);
const MEDIUM_RED: Rgb = Rgb::new(0xf1b5b5);
const LIGHT_GREY: Rgb = Rgb::new(0xe2e6ee);
const GREY: Rgb = Rgb::new(0x8f8f8f);

const TITLE_FONT: Rgb = WHITE;
const TITLE_FILL: Rgb = RED;
const TITLE_STROKE: Rgb = DARK_RED;

const HELP_FONT: Rgb = OFF_BLACK;

const SECRET_TITLE_FONT: Rgb = RED;
const SECRET_FILL: Rgb = LIGHT_RED;
const SECRET_STROKE: Rgb = MEDIUM_RED;

const ROW_FONT: Rgb = OFF_BLACK;
const ROW_STROKE: Rgb = LIGHT_GREY;
const ROW_FILL: Rgb = WHITE;

const QR_FONT: Rgb = GREY;
const QR_STROKE: Rgb = LIGHT_GREY;
const QR_FILL: Rgb = WHITE;

const CHECKBOX_FONT: Rgb = OFF_BLACK;
const CHECKBOX_STROKE: Rgb = OFF_BLACK;

const CHECKMARK_STROKE: Rgb = OFF_BLACK;

const EXTRA_TITLE_FONT: Rgb = GREY;
const EXTRA_FONT: Rgb = GREY;

#[derive(Debug, Default)]
enum FontStyle {
	#[default]
	Regular,
	Mono,
	Bold,
}

impl From<FontStyle> for Name<'static> {
	fn from(value: FontStyle) -> Self {
		match value {
			FontStyle::Regular => Name(b"FR"),
			FontStyle::Mono => Name(b"FM"),
			FontStyle::Bold => Name(b"FB"),
		}
	}
}

#[derive(Debug)]
struct Pos {
	x: f32,
	y: f32,
}

#[bon::builder]
fn text(
	c: &mut Content,
	text: &str,
	style: FontStyle,
	size: f32,
	color: Rgb,
	pos: Pos,
) {
	c.begin_text()
		.set_fill_rgb(color.r, color.g, color.b)
		.set_font(style.into(), size)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, pos.x, pos.y])
		.show(Str(text.as_bytes()))
		.end_text();
}

trait ContentExt {
	fn my_text<'a, 'b>(
		&'a mut self,
		t: &'b str,
	) -> TextBuilder<'a, 'b, text_builder::SetText<text_builder::SetC>>;
}

impl ContentExt for Content {
	fn my_text<'a, 'b>(
		&'a mut self,
		t: &'b str,
	) -> TextBuilder<'a, 'b, text_builder::SetText<text_builder::SetC>> {
		text().c(self).text(t)
	}
}

fn main() -> std::io::Result<()> {
	// Object ids
	let catalog_r = Ref::new(1);
	let pages_r = Ref::new(2);
	let page_r = Ref::new(3);
	let font_bold_r = Ref::new(4);
	let font_reg_r = Ref::new(5);
	let font_mono_r = Ref::new(6);
	let contents_r = Ref::new(7);

	let mut pdf = Pdf::new();

	// Root + page tree
	pdf.catalog(catalog_r).pages(pages_r);
	pdf.pages(pages_r).kids([page_r]).count(1);

	// Page & resources (fonts only; images optional)
	pdf.page(page_r)
		.parent(pages_r)
		.media_box(Rect::new(0.0, 0.0, W, H))
		.contents(contents_r)
		.resources()
		.fonts()
		.pair(FontStyle::Bold.into(), font_bold_r)
		.pair(FontStyle::Regular.into(), font_reg_r)
		.pair(FontStyle::Mono.into(), font_mono_r)
		.finish()
		.finish();

	// Built-in Type1 fonts (no embedding)
	pdf.type1_font(font_bold_r)
		.base_font(Name(b"Helvetica-Bold"));
	pdf.type1_font(font_reg_r).base_font(Name(b"Helvetica"));
	pdf.type1_font(font_mono_r).base_font(Name(b"Courier"));

	// Draw everything into one content stream
	let mut c = Content::new();

	// ---------- Title pill ----------
	let x = 30.0;
	let y = y_rect(30.0, 56.0);
	c.set_line_join(LineJoinStyle::RoundJoin)
		.set_line_width(1.5)
		.set_fill_rgb(TITLE_FILL.r, TITLE_FILL.g, TITLE_FILL.b)
		.set_stroke_rgb(TITLE_STROKE.r, TITLE_STROKE.g, TITLE_STROKE.b);
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
	c.my_text(title_left)
		.color(TITLE_FONT)
		.style(FontStyle::Bold)
		.size(28.0)
		.pos(Pos { x: tx, y: cy })
		.call();
	tx += est_w_helv(title_left, 28.0);

	// regular part
	c.my_text(title_right)
		.color(TITLE_FONT)
		.style(FontStyle::Regular)
		.size(28.0)
		.pos(Pos { x: tx, y: cy })
		.call();

	// ---------- How-to paragraph ----------
	let howto1 = "Keep this sheet offline and never share it. Anyone with the phrase and the optional";
	let howto2 =
		"password can control your account. Print on durable paper and store securely.";

	c.my_text(howto1)
		.color(HELP_FONT)
		.style(FontStyle::Regular)
		.size(16.0)
		.pos(Pos {
			x: 40.0,
			y: y_text(130.0),
		})
		.call();

	c.my_text(howto2)
		.color(HELP_FONT)
		.style(FontStyle::Regular)
		.size(16.0)
		.pos(Pos {
			x: 40.0,
			y: y_text(150.0),
		})
		.call();

	// ---------- Outer red pill with caption ----------
	c.set_line_join(LineJoinStyle::RoundJoin)
		.set_line_width(1.5)
		.set_fill_rgb(SECRET_FILL.r, SECRET_FILL.g, SECRET_FILL.b)
		.set_stroke_rgb(SECRET_STROKE.r, SECRET_STROKE.g, SECRET_STROKE.b);
	let outer_y = y_rect(170.0, 240.0);
	rounded(&mut c, 30.0, outer_y, 740.0, 240.0, 24.0);
	c.close_fill_nonzero_and_stroke();

	// caption "Account Details (Secret)" centered at y=189
	c.begin_text()
		.set_fill_rgb(
			SECRET_TITLE_FONT.r,
			SECRET_TITLE_FONT.g,
			SECRET_TITLE_FONT.b,
		)
		.set_font(FontStyle::Bold.into(), 14.0);
	center_text(
		&mut c,
		FontStyle::Bold.into(),
		14.0,
		"Account Details (Secret)",
		400.0,
		y_text(189.0),
	);
	c.end_text();

	// ---------- Four row pill backgrounds ----------
	for y_top in [206.0, 256.0, 306.0, 356.0].into_iter() {
		let y = y_rect(y_top, 36.0);
		c.set_line_width(1.0)
			.set_fill_rgb(ROW_FILL.r, ROW_FILL.g, ROW_FILL.b)
			.set_stroke_rgb(ROW_STROKE.r, ROW_STROKE.g, ROW_STROKE.b);
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
			.set_fill_rgb(ROW_FONT.r, ROW_FONT.g, ROW_FONT.b)
			.set_font(FontStyle::Mono.into(), 14.0);
		center_text_mono_courier(&mut c, 14.0, text, 300.0, y_text(y_svg));
		c.end_text();
	}

	// ---------- QR placeholder ----------
	c.set_line_width(1.0)
		.set_fill_rgb(QR_FILL.r, QR_FILL.g, QR_FILL.b)
		.set_stroke_rgb(QR_STROKE.r, QR_STROKE.g, QR_STROKE.b);
	let qx = 580.0;
	let qy = y_rect(200.0, 180.0);
	rounded(&mut c, qx, qy, 180.0, 180.0, 12.0);
	c.close_fill_nonzero_and_stroke();

	c.begin_text()
		.set_fill_rgb(QR_FONT.r, QR_FONT.g, QR_FONT.b)
		.set_font(FontStyle::Regular.into(), 12.0);
	center_text(
		&mut c,
		FontStyle::Regular.into(),
		12.0,
		"QR code placeholder",
		670.0,
		y_text(392.0),
	);
	c.end_text();

	// ---------- Checkbox (checked) ----------
	// Square
	c.set_line_width(1.0)
		.set_stroke_rgb(CHECKBOX_STROKE.r, CHECKBOX_STROKE.g, CHECKBOX_STROKE.b)
		.rect(40.0, y_rect(440.0, 18.0), 18.0, 18.0)
		.stroke();

	// Check mark path: (44,449) -> (49,455) -> (59,443)
	let p0 = (44.0, y_text(449.0));
	let p1 = (49.0, y_text(455.0));
	let p2 = (59.0, y_text(443.0));
	c.set_line_width(3.0)
		.set_line_cap(LineCapStyle::RoundCap)
		.set_line_join(LineJoinStyle::RoundJoin)
		.set_stroke_rgb(CHECKMARK_STROKE.r, CHECKMARK_STROKE.g, CHECKMARK_STROKE.b)
		.move_to(p0.0, p0.1)
		.line_to(p1.0, p1.1)
		.line_to(p2.0, p2.1)
		.stroke();

	// Checkbox label
	c.begin_text()
		.set_fill_rgb(CHECKBOX_FONT.r, CHECKBOX_FONT.g, CHECKBOX_FONT.b)
		.set_font(FontStyle::Regular.into(), 14.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, 66.0, y_text(454.0)])
		.show(Str("Password protected".as_bytes()))
		.end_text();

	// ---------- Extra info for developers ----------
	c.begin_text()
		.set_fill_rgb(EXTRA_TITLE_FONT.r, EXTRA_TITLE_FONT.g, EXTRA_TITLE_FONT.b)
		.set_font(FontStyle::Bold.into(), 14.0)
		.set_text_matrix([1.0, 0.0, 0.0, 1.0, 40.0, y_text(485.0)])
		.show(Str("extra info for developers".as_bytes()))
		.end_text();

	c.begin_text()
	.set_fill_rgb(EXTRA_FONT.r, EXTRA_FONT.g, EXTRA_FONT.b)
	.set_font(FontStyle::Regular.into(), 10.0)
	.set_text_matrix([1.0, 0.0, 0.0, 1.0, 40.0, y_text(505.0)])
	.show(Str("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".as_bytes()))
	.end_text();

	// Finalize content stream and write file
	pdf.stream(contents_r, &c.finish().into_vec());
	std::fs::write("recovery_kit.pdf", pdf.finish())?;
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

fn rounded(c: &mut Content, x: f32, y: f32, w: f32, h: f32, r: f32) {
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
}

/// SVG baseline -> PDF baseline
fn y_text(y_svg: f32) -> f32 {
	H - y_svg
}

/// SVG top-left -> PDF bottom-left
fn y_rect(y_svg_top: f32, h: f32) -> f32 {
	H - y_svg_top - h
}
