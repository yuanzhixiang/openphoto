//! The Type tool's text, rasterized: glyph outlines from a font file
//! (through `skrifa`), flattened and filled with the non-zero winding rule,
//! anti-aliased.

use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, MetadataProvider};

use crate::document::Document;
use crate::layer::{Layer, LayerId};
use crate::tile::TiledImage;

/// A glyph outline flattened into line segments, in document pixels.
#[derive(Default)]
struct Flattener {
    /// Edges (x0, y0, x1, y1).
    edges: Vec<[f32; 4]>,
    start: (f32, f32),
    last: (f32, f32),
    /// Glyph origin on the canvas (baseline), and the y flip.
    origin: (f32, f32),
}

impl Flattener {
    fn point(&self, x: f32, y: f32) -> (f32, f32) {
        // Font units go up; the canvas goes down
        (self.origin.0 + x, self.origin.1 - y)
    }

    fn edge_to(&mut self, p: (f32, f32)) {
        if p != self.last {
            self.edges.push([self.last.0, self.last.1, p.0, p.1]);
        }
        self.last = p;
    }
}

impl OutlinePen for Flattener {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.start = p;
        self.last = p;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.edge_to(p);
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (p0, c, p1) = (self.last, self.point(cx, cy), self.point(x, y));
        const N: usize = 8;
        for k in 1..=N {
            let t = k as f32 / N as f32;
            let u = 1.0 - t;
            self.edge_to((
                u * u * p0.0 + 2.0 * u * t * c.0 + t * t * p1.0,
                u * u * p0.1 + 2.0 * u * t * c.1 + t * t * p1.1,
            ));
        }
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let (p0, c0, c1, p1) = (
            self.last,
            self.point(cx0, cy0),
            self.point(cx1, cy1),
            self.point(x, y),
        );
        const N: usize = 12;
        for k in 1..=N {
            let t = k as f32 / N as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.edge_to((
                a * p0.0 + b * c0.0 + c * c1.0 + d * p1.0,
                a * p0.1 + b * c0.1 + c * c1.1 + d * p1.1,
            ));
        }
    }

    fn close(&mut self) {
        let start = self.start;
        self.edge_to(start);
    }
}

/// Fills the edges with the non-zero winding rule onto a `width` ×
/// `height` coverage buffer (4 sample rows per pixel, exact horizontal
/// overlap).
fn fill(edges: &[[f32; 4]], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let mut out = vec![0u8; w * height as usize];
    if edges.is_empty() {
        return out;
    }
    let y_min = edges
        .iter()
        .map(|e| e[1].min(e[3]))
        .fold(f32::MAX, f32::min)
        .floor()
        .max(0.0) as u32;
    let y_max = (edges
        .iter()
        .map(|e| e[1].max(e[3]))
        .fold(f32::MIN, f32::max)
        .ceil()
        .max(0.0) as u32)
        .min(height);
    const N: u32 = 4;
    let mut coverage = vec![0f32; w];
    let mut crossings: Vec<(f32, i32)> = Vec::new();
    for y in y_min..y_max {
        coverage.fill(0.0);
        for sub in 0..N {
            let sy = y as f32 + (sub as f32 + 0.5) / N as f32;
            crossings.clear();
            for e in edges {
                let (y0, y1) = (e[1], e[3]);
                if (y0 <= sy) != (y1 <= sy) {
                    let x = e[0] + (sy - y0) / (y1 - y0) * (e[2] - e[0]);
                    crossings.push((x, if y1 > y0 { 1 } else { -1 }));
                }
            }
            crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut winding = 0;
            for pair in crossings.windows(2) {
                winding += pair[0].1;
                if winding == 0 {
                    continue;
                }
                let (a, b) = (
                    pair[0].0.clamp(0.0, width as f32),
                    pair[1].0.clamp(0.0, width as f32),
                );
                let mut x = a.floor() as usize;
                while (x as f32) < b && x < w {
                    let overlap = b.min(x as f32 + 1.0) - a.max(x as f32);
                    coverage[x] += overlap.max(0.0) / N as f32;
                    x += 1;
                }
            }
        }
        let row = &mut out[y as usize * w..(y as usize + 1) * w];
        for (o, c) in row.iter_mut().zip(&coverage) {
            *o = (c.min(1.0) * 255.0).round() as u8;
        }
    }
    out
}

/// Why text can't be set.
#[derive(Debug, PartialEq, Eq)]
pub struct FontError;

/// Line height for `size`: the font's ascent − descent + line gap, the
/// distance Photoshop's "Auto" leading approximates.
fn line_height(font: &FontRef, size: f32) -> f32 {
    let m = font.metrics(Size::new(size), LocationRef::default());
    let h = m.ascent - m.descent + m.leading;
    if h > 0.0 { h } else { size * 1.2 }
}

/// Coverage of `text` set in the font at `size` pixels, the first line's
/// baseline starting at `origin`, on a `width` × `height` canvas. Lines
/// break at '\n'; characters the font lacks are skipped.
pub fn coverage(
    font_data: &[u8],
    text: &str,
    size: f32,
    origin: (f32, f32),
    width: u32,
    height: u32,
) -> Result<Vec<u8>, FontError> {
    coverage_oriented(font_data, text, size, origin, width, height, false)
}

/// Upright vertical glyph cells; newline starts a column to the left.
pub fn coverage_oriented(
    font_data: &[u8],
    text: &str,
    size: f32,
    origin: (f32, f32),
    width: u32,
    height: u32,
    vertical: bool,
) -> Result<Vec<u8>, FontError> {
    let font = FontRef::new(font_data).map_err(|_| FontError)?;
    let charmap = font.charmap();
    let outlines = font.outline_glyphs();
    let metrics = font.glyph_metrics(Size::new(size), LocationRef::default());
    let mut pen = Flattener::default();
    let line = line_height(&font, size);
    for (i, text_line) in text.split('\n').enumerate() {
        let mut x = origin.0;
        let baseline = origin.1 + i as f32 * line;
        let ascent = font.metrics(Size::new(size), LocationRef::default()).ascent;

        for (row, gid) in text_line
            .chars()
            .filter_map(|ch| charmap.map(ch))
            .enumerate()
        {
            if let Some(glyph) = outlines.get(gid) {
                pen.origin = if vertical {
                    (
                        origin.0 - i as f32 * line - metrics.advance_width(gid).unwrap_or(0.) / 2.,
                        origin.1 + row as f32 * size + ascent,
                    )
                } else {
                    (x, baseline)
                };
                let settings = DrawSettings::unhinted(Size::new(size), LocationRef::default());
                let _ = glyph.draw(settings, &mut pen);
            }
            x += metrics.advance_width(gid).unwrap_or(0.0);
        }
    }
    Ok(fill(&pen.edges, width, height))
}

/// The point where a text's caret sits after its last character, and the
/// line height (for drawing the caret).
pub fn caret(
    font_data: &[u8],
    text: &str,
    size: f32,
    origin: (f32, f32),
) -> Option<((f32, f32), f32)> {
    caret_oriented(font_data, text, size, origin, false)
}
pub fn caret_oriented(
    font_data: &[u8],
    text: &str,
    size: f32,
    origin: (f32, f32),
    vertical: bool,
) -> Option<((f32, f32), f32)> {
    let font = FontRef::new(font_data).ok()?;
    let charmap = font.charmap();
    let metrics = font.glyph_metrics(Size::new(size), LocationRef::default());
    let line = line_height(&font, size);
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.last().copied().unwrap_or("");
    let x = last
        .chars()
        .filter_map(|ch| charmap.map(ch))
        .map(|gid| metrics.advance_width(gid).unwrap_or(0.0))
        .sum::<f32>();
    let p = if vertical {
        (
            origin.0 - (lines.len() - 1) as f32 * line,
            origin.1 + last.chars().filter(|&c| charmap.map(c).is_some()).count() as f32 * size,
        )
    } else {
        (origin.0 + x, origin.1 + (lines.len() - 1) as f32 * line)
    };
    Some((p, line))
}

/// Adds a layer with `text` in `color` above the active layer, named after
/// the text like Photoshop's type layers (its first line, up to 30
/// characters). Returns `None` for text without visible glyphs.
pub fn add_text_layer(
    doc: &mut Document,
    font_data: &[u8],
    text: &str,
    size: f32,
    origin: (f32, f32),
    color: [u8; 3],
) -> Result<Option<LayerId>, FontError> {
    add_text_layer_oriented(doc, font_data, text, size, origin, color, false)
}
pub fn add_text_layer_oriented(
    doc: &mut Document,
    font_data: &[u8],
    text: &str,
    size: f32,
    origin: (f32, f32),
    color: [u8; 3],
    vertical: bool,
) -> Result<Option<LayerId>, FontError> {
    let (w, h) = (doc.width, doc.height);
    let cov = coverage_oriented(font_data, text, size, origin, w, h, vertical)?;
    if cov.iter().all(|&v| v == 0) {
        return Ok(None);
    }
    let mut image = TiledImage::new(w, h);
    for (i, &v) in cov.iter().enumerate() {
        if v > 0 {
            image.set_pixel(
                i as u32 % w,
                i as u32 / w,
                [color[0], color[1], color[2], v],
            );
        }
    }
    let name: String = text.lines().next().unwrap_or("").chars().take(30).collect();
    let id = doc.new_layer_id();
    doc.insert_above_active(Layer::raster(id, name, image));
    Ok(Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    const FONT: &[u8] = include_bytes!("../../op-ui/assets/fonts/SourceSans3-Regular.ttf");

    #[test]
    fn letters_cover_pixels_above_the_baseline() {
        let cov = coverage(FONT, "I", 40.0, (10.0, 50.0), 60, 60).unwrap();
        let at = |x: usize, y: usize| cov[y * 60 + x];
        // The I's stem: solid between the cap height and the baseline
        let stem = (10..30).find(|&x| at(x, 40) == 255).expect("a solid stem");
        assert_eq!(at(stem, 30), 255);
        assert_eq!(at(stem, 55), 0, "nothing below the baseline");
        assert_eq!(at(stem, 5), 0, "nothing above the cap height");
    }

    #[test]
    fn holes_stay_open_and_lines_stack() {
        // An "O" is hollow in the middle
        let cov = coverage(FONT, "O", 60.0, (5.0, 70.0), 80, 80).unwrap();
        let row: Vec<u8> = (0..80).map(|x| cov[48 * 80 + x]).collect();
        let first = row.iter().position(|&v| v == 255).unwrap();
        let last = row.iter().rposition(|&v| v == 255).unwrap();
        assert!(row[(first + last) / 2] == 0, "{row:?}");
        // A second line sits one line height lower
        let (_, line) = caret(FONT, "a\nb", 20.0, (0.0, 20.0)).unwrap();
        let (end, _) = caret(FONT, "a\nb", 20.0, (0.0, 20.0)).unwrap();
        assert!((end.1 - (20.0 + line)).abs() < 1e-3);
        assert!(end.0 > 0.0);
    }

    #[test]
    fn text_layers_are_named_after_the_text() {
        let mut doc = Document::new_with_background("t", 200, 60, Color::WHITE);
        let id = add_text_layer(
            &mut doc,
            FONT,
            "Hello\nworld",
            20.0,
            (5.0, 25.0),
            [255, 0, 0],
        )
        .unwrap()
        .unwrap();
        assert_eq!(doc.layer(id).unwrap().name, "Hello");
        assert_eq!(
            add_text_layer(&mut doc, FONT, "   ", 20.0, (5.0, 25.0), [0; 3]),
            Ok(None)
        );
        assert!(add_text_layer(&mut doc, b"not a font", "x", 20.0, (0.0, 0.0), [0; 3]).is_err());
    }
}
