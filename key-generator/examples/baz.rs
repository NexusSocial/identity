use pdf_writer::types::{LineCapStyle, LineJoinStyle, TextAlign, TextRenderingMode};
use pdf_writer::{Content, Finish, Name, Pdf, Rect as PdfRect, Ref, Str};

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

// Elements
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

#[derive(Debug, Default, Clone, Copy)]
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

/// Position
#[derive(Debug, Clone, Copy)]
struct Vec2 {
	x: f32,
	y: f32,
}

impl From<(f32, f32)> for Vec2 {
	fn from(value: (f32, f32)) -> Self {
		Self {
			x: value.0,
			y: value.1,
		}
	}
}

struct Rect {
	pos: Vec2,
	size: Vec2,
}

impl Rect {
	fn new(pos: impl Into<Vec2>, size: impl Into<Vec2>) -> Self {
		Self {
			pos: pos.into(),
			size: size.into(),
		}
	}
}

/// Uses SVG coordinate system, 0,0 top left
struct Pill {
	weight: f32,
	fill: Rgb,
	stroke: Rgb,
	rect: Rect,
	radius: f32,
}

impl Pill {
	fn draw(&self, c: &mut Content) {
		c.set_line_join(LineJoinStyle::RoundJoin)
			.set_line_width(self.weight)
			.set_fill_rgb(self.fill.r, self.fill.g, self.fill.b)
			.set_stroke_rgb(self.stroke.r, self.stroke.g, self.stroke.b);
		rounded(
			c,
			self.rect.pos.x,
			H - self.rect.pos.y - self.rect.size.y,
			self.rect.size.x,
			self.rect.size.y,
			self.radius,
		);
		c.close_fill_nonzero_and_stroke();
	}
}

struct Text<'a> {
	text: &'a str,
	style: FontStyle,
	size: f32,
	color: Rgb,
	pos: Vec2,
	// align: TextAlign,
}

impl Text<'_> {
	fn draw(&self, c: &mut Content) {
		c.begin_text()
			.set_fill_rgb(self.color.r, self.color.g, self.color.b)
			.set_font(self.style.into(), self.size)
			.set_text_matrix([1.0, 0.0, 0.0, 1.0, self.pos.x, H - self.pos.y])
			.show(Str(self.text.as_bytes()))
			.end_text();
	}
}

struct Checkbox {
	pos: Vec2,
	size: f32,
	is_checked: bool,
}

impl Checkbox {
	fn draw(&self, c: &mut Content) {
		// Square
		c.my_pill(Pill {
			weight: 1.,
			fill: WHITE,
			stroke: CHECKBOX_STROKE,
			rect: Rect::new(self.pos, (self.size, self.size)),
			radius: self.size * 0.1,
		});
		// c.set_line_width(1.0)
		// 	.set_stroke_rgb(CHECKBOX_STROKE.r, CHECKBOX_STROKE.g, CHECKBOX_STROKE.b)
		// 	.rect(40.0, y_rect(440.0, 18.0), 18.0, 18.0)
		// 	.stroke();

		if !self.is_checked {
			return;
		}

		// Check mark path: (44,449) -> (49,455) -> (59,443)
		let p0 = (
			self.pos.x + self.size * 0.2,
			H - (self.pos.y + self.size * 0.5),
		);
		let p1 = (
			self.pos.x + self.size * 0.5,
			H - (self.pos.y + self.size * 0.8),
		);
		let p2 = (
			self.pos.x + self.size * 1.1,
			H - (self.pos.y + self.size * 0.1),
		);
		c.set_line_width(3.0)
			.set_line_cap(LineCapStyle::RoundCap)
			.set_line_join(LineJoinStyle::RoundJoin)
			.set_stroke_rgb(CHECKMARK_STROKE.r, CHECKMARK_STROKE.g, CHECKMARK_STROKE.b)
			.move_to(p0.0, p0.1)
			.line_to(p1.0, p1.1)
			.line_to(p2.0, p2.1)
			.stroke();
	}
}

trait ContentExt {
	fn my_text_2(&mut self, text: Text);

	fn my_pill(&mut self, pill: Pill);

	fn my_checkbox(&mut self, cbox: Checkbox);
}

impl ContentExt for Content {
	fn my_pill(&mut self, pill: Pill) {
		pill.draw(self);
	}

	fn my_text_2(&mut self, text: Text) {
		text.draw(self);
	}

	fn my_checkbox(&mut self, cbox: Checkbox) {
		cbox.draw(self);
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
		.media_box(PdfRect::new(0.0, 0.0, W, H))
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
	let padding = 30.;
	let radius = 28.0;
	c.my_pill(Pill {
		weight: 1.5,
		fill: TITLE_FILL,
		stroke: TITLE_STROKE,
		rect: Rect::new((padding, padding), (W - 2. * padding, radius * 2.)),
		radius,
	});

	let cx = W / 2.;
	let cy = 66.0;
	let title_left = "Basis";
	let title_right = " Recovery Kit";

	let bold_width = est_text_width(title_left, 28.0, FontStyle::Bold);
	let normal_width = est_text_width(title_right, 28.0, FontStyle::Regular);
	let total_width = bold_width + normal_width;
	let mut tx = cx - total_width / 2.0;

	// bold part
	c.my_text_2(Text {
		text: title_left,
		color: TITLE_FONT,
		style: FontStyle::Bold,
		size: 28.0,
		pos: Vec2 { x: tx, y: cy },
	});
	tx += bold_width;

	// regular part
	c.my_text_2(Text {
		text: title_right,
		color: TITLE_FONT,
		style: FontStyle::Regular,
		size: 28.0,
		pos: Vec2 { x: tx, y: cy },
	});

	// ---------- How-to paragraph ----------
	let howto1 = "Keep this sheet offline and never share it. Anyone with the phrase and the optional";
	let howto2 =
		"password can control your account. Print on durable paper and store securely.";

	c.my_text_2(Text {
		text: howto1,
		color: HELP_FONT,
		style: FontStyle::Regular,
		size: 16.0,
		pos: Vec2 { x: 40.0, y: 130.0 },
	});

	c.my_text_2(Text {
		text: howto2,
		color: HELP_FONT,
		style: FontStyle::Regular,
		size: 16.0,
		pos: Vec2 { x: 40.0, y: 150.0 },
	});

	// ---------- Outer red pill with caption ----------
	c.my_pill(Pill {
		weight: 1.5,
		fill: SECRET_FILL,
		stroke: SECRET_STROKE,
		rect: Rect::new((30., 170.), (740., 240.)),
		radius: 24.,
	});

	let size = 14.;
	let text = "Account Details (Secret)";
	let text_width = est_text_width(text, size, FontStyle::Bold);
	c.my_text_2(Text {
		text,
		style: FontStyle::Bold,
		size,
		color: SECRET_TITLE_FONT,
		pos: Vec2 {
			x: (W - text_width) / 2.,
			y: 189.,
		},
	});

	// ---------- Four row pill backgrounds ----------
	for y_top in [206.0, 256.0, 306.0, 356.0].into_iter() {
		c.my_pill(Pill {
			weight: 1.,
			fill: ROW_FILL,
			stroke: ROW_STROKE,
			rect: Rect::new((60., y_top), (500., 36.)),
			radius: 18.,
		});
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
		let size = 14.;
		let text_width = est_text_width(text, size, FontStyle::Mono);
		c.my_text_2(Text {
			text,
			style: FontStyle::Mono,
			size,
			color: ROW_FONT,
			pos: Vec2 {
				x: 300. - text_width / 2.,
				y: y_svg,
			},
		});
	}

	// ---------- QR placeholder ----------
	let qr_pos = Vec2 { x: 580., y: 200. };
	let qr_size = Vec2 { x: 180., y: 180. };
	c.my_pill(Pill {
		weight: 1.,
		fill: QR_FILL,
		stroke: QR_STROKE,
		rect: Rect::new(qr_pos, qr_size),
		radius: 12.,
	});

	let text = "QR code placeholder";
	let size = 12.;
	let text_width = est_text_width(text, size, FontStyle::Regular);
	c.my_text_2(Text {
		text,
		style: FontStyle::Regular,
		size,
		color: QR_FONT,
		pos: Vec2 {
			x: 670. - text_width / 2.,
			y: 392.,
		},
	});

	// ---------- Checkbox ----------

	// Checkbox
	c.my_checkbox(Checkbox {
		pos: Vec2 { x: 40., y: 440. },
		size: 18.,
		is_checked: true,
	});

	// Checkbox label
	c.my_text_2(Text {
		text: "Password protected",
		style: FontStyle::Regular,
		size: 14.,
		color: CHECKBOX_FONT,
		pos: Vec2 { x: 66., y: 454. },
	});

	// ---------- Extra info for developers ----------
	c.my_text_2(Text {
		text: "extra info for developers",
		style: FontStyle::Bold,
		size: 14.,
		color: EXTRA_TITLE_FONT,
		pos: Vec2 { x: 40., y: 485. },
	});

	c.my_text_2(Text {
		text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
		style: FontStyle::Regular,
		size: 10.,
		color: EXTRA_FONT,
		pos: Vec2 { x: 40., y: 505. },
	});

	// Finalize content stream and write file
	pdf.stream(contents_r, &c.finish().into_vec());
	std::fs::write("recovery_kit.pdf", pdf.finish())?;
	Ok(())
}

// Uses native PDF coordinate system, 0,0 bottom left
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

// crude width estimate for Helvetica-ish centering (good enough for this layout)
fn est_text_width(s: &str, size: f32, style: FontStyle) -> f32 {
	let multiplier = match style {
		FontStyle::Regular => 0.52,
		FontStyle::Bold => 0.52,
		FontStyle::Mono => 0.6,
	};
	s.chars().count() as f32 * size * multiplier
}
