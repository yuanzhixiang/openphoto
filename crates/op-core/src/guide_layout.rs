//! View › Guides › New Guide Layout...: columns, rows and margins as
//! guides.

use crate::document::Guide;

/// A guide layout. Lengths are in pixels; a `None` width or height splits
/// what is left evenly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GuideLayout {
    /// Columns: number, width, gutter (when on).
    pub columns: Option<(u32, Option<f32>, f32)>,
    /// Rows: number, height, gutter (when on).
    pub rows: Option<(u32, Option<f32>, f32)>,
    /// Margins: top, left, bottom, right (when on).
    pub margin: Option<[f32; 4]>,
    /// Columns of a set width sit in the middle of the space instead of
    /// at its left.
    pub center_columns: bool,
}

impl Default for GuideLayout {
    /// Photoshop's: 8 columns, 20 px gutters, no rows or margins.
    fn default() -> Self {
        Self {
            columns: Some((8, None, 20.0)),
            rows: None,
            margin: None,
            center_columns: false,
        }
    }
}

/// The edges of `n` spans with `gap` between them in `from`–`to`, each
/// `size` long (or the space shared evenly), starting at the start (or
/// centered).
fn spans(from: f32, to: f32, (n, size, gap): (u32, Option<f32>, f32), center: bool) -> Vec<f32> {
    if n == 0 {
        return Vec::new();
    }
    let space = to - from;
    let gaps = gap * (n - 1) as f32;
    let size = size.unwrap_or((space - gaps) / n as f32).max(0.0);
    let total = size * n as f32 + gaps;
    let start = if center {
        from + (space - total) / 2.0
    } else {
        from
    };
    let mut out = Vec::new();
    for k in 0..n {
        let a = start + k as f32 * (size + gap);
        out.push(a);
        out.push(a + size);
    }
    out
}

impl GuideLayout {
    /// The guides for a `width` × `height` canvas: the margins, then each
    /// column's and row's edges (once where a gutter is 0).
    pub fn guides(&self, width: f32, height: f32) -> Vec<Guide> {
        let [top, left, bottom, right] = self.margin.unwrap_or([0.0; 4]);
        let (x0, x1, y0, y1) = (left, width - right, top, height - bottom);
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        if self.margin.is_some() {
            xs.extend([x0, x1]);
            ys.extend([y0, y1]);
        }
        if let Some(c) = self.columns {
            xs.extend(spans(x0, x1, c, self.center_columns));
        }
        if let Some(r) = self.rows {
            ys.extend(spans(y0, y1, r, false));
        }
        let mut out = Vec::new();
        let mut add = |vertical: bool, v: f32| {
            // (to Photoshop's 1/32 px, once each)
            let v = (v * 32.0).round() / 32.0;
            if !out
                .iter()
                .any(|g: &Guide| g.vertical == vertical && (g.position - v).abs() < 1e-3)
            {
                out.push(Guide {
                    vertical,
                    position: v,
                });
            }
        };
        for x in xs {
            add(true, x);
        }
        for y in ys {
            add(false, y);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn positions(guides: &[Guide], vertical: bool) -> Vec<f32> {
        guides
            .iter()
            .filter(|g| g.vertical == vertical)
            .map(|g| g.position)
            .collect()
    }

    #[test]
    fn columns_rows_and_margins() {
        // 4 columns, 10 px gutters across 430 px: 100 px each
        let layout = GuideLayout {
            columns: Some((4, None, 10.0)),
            ..Default::default()
        };
        let g = layout.guides(430.0, 100.0);
        assert_eq!(
            positions(&g, true),
            vec![0.0, 100.0, 110.0, 210.0, 220.0, 320.0, 330.0, 430.0]
        );
        // No gutter: shared edges once; rows and margins
        let layout = GuideLayout {
            columns: Some((2, None, 0.0)),
            rows: Some((2, None, 0.0)),
            margin: Some([10.0, 20.0, 10.0, 20.0]),
            center_columns: false,
        };
        let g = layout.guides(240.0, 120.0);
        assert_eq!(positions(&g, true), vec![20.0, 220.0, 120.0]);
        assert_eq!(positions(&g, false), vec![10.0, 110.0, 60.0]);
        // Set widths, centered
        let layout = GuideLayout {
            columns: Some((2, Some(50.0), 10.0)),
            center_columns: true,
            ..Default::default()
        };
        assert_eq!(
            positions(&layout.guides(210.0, 10.0), true),
            vec![50.0, 100.0, 110.0, 160.0]
        );
    }
}
