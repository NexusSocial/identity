use alloc::{string::String, vec::Vec};
use pdf_writer::{
	types::{LineCapStyle, LineJoinStyle},
	{Content, Finish, Name, Pdf, Rect as PdfRect, Ref, Str},
};

// const TEMPLATE: &str = r###"<?xml version="1.0" encoding="UTF-8"?>
// <svg xmlns="http://www.w3.org/2000/svg"
//      width="800" height="1000" viewBox="0 0 800 1000"
//      font-family="Arial, Helvetica, sans-serif" text-rendering="geometricPrecision" role="img" aria-label="BIP-39 Recovery Kit">
//   <desc>Basis Recovery Kit, containing </desc>
//
//   <!-- Title pill (dark red) -->
//   <rect x="30" y="30" width="740" height="56" rx="28" ry="28" fill="#b42a2a" stroke="#8f1f1f" stroke-width="1.5"/>
//   <text x="400" y="66" font-size="28" text-anchor="middle" fill="#ffffff">
//     <tspan font-weight="bold">Basis</tspan>
//     <tspan> Recovery Kit</tspan>
//   </text>
//
//   <!-- How-to paragraph -->
//   <text x="40" y="110" font-size="16" fill="#111">
//     <tspan x="40" dy="20">Keep this sheet offline and never share it. Anyone with the phrase and the optional</tspan>
//     <tspan x="40" dy="20">password can control your account. Print on durable paper and store securely.</tspan>
//   </text>
//
//   <!-- OUTER RED PILL (wraps words + QR) -->
//   <rect x="30" y="170" width="740" height="240" rx="24" ry="24" fill="#fff5f5" stroke="#f1b5b5" stroke-width="1.5"/>
//   <text x="400" y="189" font-size="14" font-weight="bold" fill="#b44" text-anchor="middle">Account Details (Secret)</text>
//
//   <!-- Row pill backgrounds (white) -->
//   <rect x="60" y="206" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>
//   <rect x="60" y="256" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>
//   <rect x="60" y="306" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>
//   <rect x="60" y="356" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>
//
//   <!-- Four centered rows, six words each, hyphen-separated (monospaced) -->
//   <text x="300" y="228" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
//     word01-word02-word03-word04-word05-word06
//   </text>
//   <text x="300" y="278" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
//     word07-word08-word09-word10-word11-word12
//   </text>
//   <text x="300" y="328" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
//     word13-word14-word15-word16-word17-word18
//   </text>
//   <text x="300" y="378" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
//     word19-word20-word21-word22-word23-word24
//   </text>
//
//   <!-- QR code area (white) -->
//   <rect x="580" y="200" width="180" height="180" rx="12" ry="12" fill="#ffffff" stroke="#d7dbe3"/>
//   <image x="590" y="210" width="160" height="160"
//          href="data:image/png;base64,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" />
//   <text x="670" y="392" font-size="12" fill="#777" text-anchor="middle">QR code placeholder</text>
//
//   <!-- Checkbox (checked by default) -->
//   <rect x="40" y="440" width="18" height="18" rx="3" ry="3" fill="none" stroke="#333"/>
//   <path d="M44 449 l5 6 l10 -12" fill="none" stroke="#111" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
//   <text x="66" y="454" font-size="14" fill="#111">Password protected</text>
//
//   <!-- Extra info for developers -->
//   <text x="40" y="485" font-size="14" font-weight="bold">extra info for developers</text>
//   <text x="40" y="505" font-size="10" fill="#777">
//     Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
//   </text>
// </svg>
// "###;

use crate::PHRASE_LEN;

pub(crate) struct PdfGenerator<'a, 'b> {
	pub(crate) words: [&'a str; crate::PHRASE_LEN],
	pub(crate) app_name: &'b str,
	pub(crate) password: bool,
}

#[derive(Debug)]
pub struct Exports {
	pub pdf_contents: Vec<u8>,
	pub svg_contents: String,
}

impl PdfGenerator<'_, '_> {
	pub(crate) fn build(self) -> Exports {
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
		let title_left = self.app_name.trim();
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
		let howto2 = "password can control your account. Print on durable paper and store securely.";

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
		let n_words_row = PHRASE_LEN / 4;
		assert_eq!(n_words_row * 4, PHRASE_LEN, "sanity: always true for 24");
		let rows = self.words.chunks(n_words_row);
		let ys = [228.0, 278.0, 328.0, 378.0];
		for (text, y_svg) in rows.zip(ys) {
			let size = 14.;
			let text = text.join("-");
			let text_width = est_text_width(&text, size, FontStyle::Mono);
			c.my_text_2(Text {
				text: &text,
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
			is_checked: self.password,
		});

		// Checkbox label
		c.my_text_2(Text {
			text: "Password protected?",
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

		let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
		c.my_text_2(Text {
			text,
			style: FontStyle::Regular,
			size: 10.,
			color: EXTRA_FONT,
			pos: Vec2 { x: 40., y: 505. },
		});

		pdf.stream(contents_r, &c.finish().into_vec());

		let pdf_contents = pdf.finish();
		let svg_contents = String::new(); // TODO

		Exports {
			pdf_contents,
			svg_contents,
		}
	}
}

const W: f32 = 800.0;
const H: f32 = 1000.0;

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

#[derive(Default, Clone, Copy)]
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

#[derive(Clone, Copy)]
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
	#[inline(never)]
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
	#[inline(never)]
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
	#[inline(never)]
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
