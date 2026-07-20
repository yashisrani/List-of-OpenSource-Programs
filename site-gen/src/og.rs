//! Build-time Open Graph image generation.
//!
//! Renders a 1200x630 PNG per page so links shared on Twitter, LinkedIn,
//! WhatsApp, Discord or Slack show a real preview instead of a bare URL. Each
//! card is built as an SVG string and rasterized with `resvg`. The font is
//! bundled and loaded explicitly so a card renders identically on any build
//! machine, including Vercel, regardless of installed system fonts.

use std::path::Path;

use resvg::tiny_skia;
use resvg::usvg;

const FONT_SANS: &[u8] = include_bytes!("../fonts/Inter.ttf");

const W: u32 = 1200;
const H: u32 = 630;

// Card palette, matching the site's light theme.
const BG: &str = "#fdfcfa";
const INK: &str = "#1a1a18";
const INK_2: &str = "#55534d";
const INK_3: &str = "#6f6c66";

pub struct OgRenderer {
    fontdb: std::sync::Arc<usvg::fontdb::Database>,
}

/// Escape text for inclusion in SVG character data.
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            _ => o.push(c),
        }
    }
    o
}

/// Break `text` into lines of at most `max_chars`, up to `max_lines`.
/// Inter has no metrics available here, so this counts characters rather than
/// measuring; the headline sizes below leave enough slack that the estimate
/// holds. The last line is ellipsized if the text runs long.
fn wrap(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let candidate = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if candidate.chars().count() > max_chars && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            if lines.len() == max_lines {
                let last = lines.last_mut().unwrap();
                last.push_str("...");
                return lines;
            }
            cur = word.to_string();
        } else {
            cur = candidate;
        }
    }
    if !cur.is_empty() && lines.len() < max_lines {
        lines.push(cur);
    }
    lines
}

impl OgRenderer {
    pub fn new() -> Self {
        let mut db = usvg::fontdb::Database::new();
        db.load_font_data(FONT_SANS.to_vec());
        OgRenderer {
            fontdb: std::sync::Arc::new(db),
        }
    }

    fn render_svg(&self, svg: &str, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut opt = usvg::Options::default();
        opt.fontdb = self.fontdb.clone();
        let tree = usvg::Tree::from_str(svg, &opt)?;
        let mut pixmap = tiny_skia::Pixmap::new(W, H).ok_or("pixmap alloc failed")?;
        resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());
        pixmap.save_png(out)?;
        Ok(())
    }

    /// A page card: headline, one supporting line, and the site name.
    ///
    /// Share cards are seen at thumbnail size in a crowded feed, so this is
    /// deliberately three elements and nothing else. Stats, chips and rules all
    /// competed with the headline and lost.
    pub fn card(
        &self,
        out: &Path,
        headline: &str,
        sub: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let lines = wrap(headline, 24, 3);
        let size = match lines.len() {
            1 => 88.0,
            2 => 80.0,
            _ => 66.0,
        };
        let line_h = size * 1.14;

        // Center the headline block, then hang the sub-line below it.
        let block_h = line_h * lines.len() as f32;
        let start_y = 315.0 - block_h / 2.0 + size * 0.76;

        let mut head = String::new();
        for (i, l) in lines.iter().enumerate() {
            head.push_str(&format!(
                r#"<text x="80" y="{y:.1}" font-family="Inter" font-size="{size}" font-weight="700" fill="{INK}" letter-spacing="-2.5">{t}</text>
"#,
                y = start_y + line_h * i as f32,
                t = esc(l),
            ));
        }
        let sub_y = start_y + line_h * (lines.len() - 1) as f32 + 74.0;

        let svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">
<rect width="{W}" height="{H}" fill="{BG}"/>
{head}<text x="80" y="{sub_y:.1}" font-family="Inter" font-size="30" font-weight="400" fill="{INK_2}">{sub}</text>
<text x="80" y="556" font-family="Inter" font-size="22" font-weight="600" fill="{INK_3}" letter-spacing="-0.2">Open Source Programs</text>
</svg>"##,
            sub = esc(sub),
        );

        self.render_svg(&svg, out)
    }
}
