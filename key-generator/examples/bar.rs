use std::fs;
use std::sync::Arc;
use std::sync::LazyLock;

const FONTDB: LazyLock<usvg::fontdb::Database> = std::sync::LazyLock::new(|| {
	let mut db = usvg::fontdb::Database::new();
	db.load_system_fonts();

	db
});

const TEMPLATE: &str = r###"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="800" height="1000" viewBox="0 0 800 1000"
     font-family="Arial, Helvetica, sans-serif" text-rendering="geometricPrecision" role="img" aria-label="BIP-39 Recovery Kit">
  <desc>Basis Recovery Kit, containing </desc>

  <!-- Title pill (dark red) -->
  <rect x="30" y="30" width="740" height="56" rx="28" ry="28" fill="#b42a2a" stroke="#8f1f1f" stroke-width="1.5"/>
  <text x="400" y="66" font-size="28" text-anchor="middle" fill="#ffffff">
    <tspan font-weight="bold">Basis</tspan>
    <tspan> Recovery Kit</tspan>
  </text>

  <!-- How-to paragraph -->
  <text x="40" y="110" font-size="16" fill="#111">
    <tspan x="40" dy="20">Keep this sheet offline and never share it. Anyone with the phrase and the optional</tspan>
    <tspan x="40" dy="20">password can control your account. Print on durable paper and store securely.</tspan>
  </text>

  <!-- OUTER RED PILL (wraps words + QR) -->
  <rect x="30" y="170" width="740" height="240" rx="24" ry="24" fill="#fff5f5" stroke="#f1b5b5" stroke-width="1.5"/>
  <text x="400" y="189" font-size="14" font-weight="bold" fill="#b44" text-anchor="middle">Account Details (Secret)</text>

  <!-- Row pill backgrounds (white) -->
  <rect x="60" y="206" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>
  <rect x="60" y="256" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>
  <rect x="60" y="306" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>
  <rect x="60" y="356" width="500" height="36" rx="18" ry="18" fill="#ffffff" stroke="#e2e6ee"/>

  <!-- Four centered rows, six words each, hyphen-separated (monospaced) -->
  <text x="300" y="228" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
    word01-word02-word03-word04-word05-word06
  </text>
  <text x="300" y="278" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
    word07-word08-word09-word10-word11-word12
  </text>
  <text x="300" y="328" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
    word13-word14-word15-word16-word17-word18
  </text>
  <text x="300" y="378" font-size="14" fill="#000" text-anchor="middle" font-family="Courier New, monospace">
    word19-word20-word21-word22-word23-word24
  </text>

  <!-- QR code area (white) -->
  <rect x="580" y="200" width="180" height="180" rx="12" ry="12" fill="#ffffff" stroke="#d7dbe3"/>
  <image x="590" y="210" width="160" height="160"
         href="data:image/png;base64,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" />
  <text x="670" y="392" font-size="12" fill="#777" text-anchor="middle">QR code placeholder</text>

  <!-- Checkbox (checked by default) -->
  <rect x="40" y="440" width="18" height="18" rx="3" ry="3" fill="none" stroke="#333"/>
  <path d="M44 449 l5 6 l10 -12" fill="none" stroke="#111" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
  <text x="66" y="454" font-size="14" fill="#111">Password protected</text>

  <!-- Extra info for developers -->
  <text x="40" y="485" font-size="14" font-weight="bold">extra info for developers</text>
  <text x="40" y="505" font-size="10" fill="#777">
    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
  </text>
</svg>
"###;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// ------- Convert SVG -> PDF with usvg + svg2pdf -------
	// Parse with usvg
	let opt = usvg::Options {
		fontdb: Arc::new(FONTDB.clone()),
		..Default::default()
	};
	// Load system fonts to render text properly
	// opt.fontdb.load_system_fonts();

	// `to_ref()` passes a lightweight reference into the parser (current usvg API)
	let tree = usvg::Tree::from_str(&TEMPLATE, &opt)
		.map_err(|e| format!("SVG parse error: {e:?}"))?;
	let svg = tree.to_string(&usvg::WriteOptions::default());

	// Convert to PDF
	// svg2pdf typically takes (&usvg::Tree, &mut Vec<u8>, Options)
	let pdf_bytes = svg2pdf::to_pdf(
		&tree,
		svg2pdf::ConversionOptions::default(),
		svg2pdf::PageOptions::default(),
	)
	.unwrap();

	// Write outputs
	fs::write("recovery_kit.svg", svg.as_bytes())?;
	fs::write("recovery_kit.pdf", &pdf_bytes)?;

	println!("Wrote recovery_kit.svg and recovery_kit.pdf");
	Ok(())
}
