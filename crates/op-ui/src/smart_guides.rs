//! View › Show › Smart Guides: while the Move tool drags, magenta lines
//! show where the moved pixels' edges or center line up with the canvas's
//! or another layer's, and the move snaps to them.

use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use op_core::LayerId;

use crate::document_view::to_screen;
use crate::state::{DocState, ViewOptions};

/// Photoshop's default Smart Guides color (Magenta), as its screen shows it.
const COLOR: Color32 = Color32::from_rgb(0xeb, 0x59, 0xf7);

/// A box in document pixels: (x0, y0, x1, y1).
type Bounds = (f32, f32, f32, f32);

/// A move's material: the box being moved (at the start) and the boxes it
/// can line up with: the canvas, then the other visible layers.
#[derive(Clone, Debug)]
pub struct SmartGuides {
    moving: Bounds,
    others: Vec<Bounds>,
    /// The lines showing now, as document segments.
    pub lines: Vec<[Pos2; 2]>,
    /// The distance labels showing now: the gap from the moved box to its
    /// nearest neighbor on each side (a segment and its length).
    pub labels: Vec<([Pos2; 2], f32)>,
}

impl SmartGuides {
    /// For a Move drag: `None` when Smart Guides don't show (Show › Smart
    /// Guides or Extras off) or nothing can move.
    pub fn begin(state: &DocState, view: &ViewOptions, moving_layers: &[LayerId]) -> Option<Self> {
        if !(view.extras && view.smart_guides) {
            return None;
        }
        let doc = &state.doc;
        let moving = op_core::transform::bounds(doc).ok()?;
        let mut others = vec![(0.0, 0.0, doc.width as f32, doc.height as f32)];
        for layer in &doc.layers {
            if !layer.visible || layer.is_background || moving_layers.contains(&layer.id) {
                continue;
            }
            if let Some((x0, y0, x1, y1)) = layer.image().and_then(|i| i.content_bounds()) {
                others.push((x0 as f32, y0 as f32, x1 as f32, y1 as f32));
            }
        }
        Some(Self {
            moving,
            others,
            lines: Vec::new(),
            labels: Vec::new(),
        })
    }

    /// For a drag that places points (the marquees, the shape tools): the
    /// point lines up with the canvas's and every visible layer's edges
    /// and centers. `align` then takes the point as its offset.
    pub fn for_points(state: &DocState, view: &ViewOptions) -> Option<Self> {
        if !(view.extras && view.smart_guides) {
            return None;
        }
        let doc = &state.doc;
        let mut others = vec![(0.0, 0.0, doc.width as f32, doc.height as f32)];
        for layer in &doc.layers {
            if !layer.visible || layer.is_background {
                continue;
            }
            if let Some((x0, y0, x1, y1)) = layer.image().and_then(|i| i.content_bounds()) {
                others.push((x0 as f32, y0 as f32, x1 as f32, y1 as f32));
            }
        }
        Some(Self {
            moving: (0.0, 0.0, 0.0, 0.0),
            others,
            lines: Vec::new(),
            labels: Vec::new(),
        })
    }

    /// The offset `d` pulled so an edge or the center of the moved box
    /// lines up with one of another box's within `tolerance` (document
    /// pixels), per axis; the lines that line up are kept for drawing.
    /// `snapped` is the plain snapping's offset: on each axis whichever
    /// pull is smaller wins (the plain one when the guides pull nothing).
    pub fn align_with(&mut self, d: Vec2, snapped: Vec2, tolerance: f32) -> Vec2 {
        let smart = self.align(d, tolerance);
        let pick = |raw: f32, s: f32, p: f32| {
            let (ds, dp) = ((s - raw).abs(), (p - raw).abs());
            if ds > 0.0 && (dp == 0.0 || ds <= dp) {
                s
            } else {
                p
            }
        };
        let out = Vec2::new(pick(d.x, smart.x, snapped.x), pick(d.y, smart.y, snapped.y));
        // The lines for where it ends up
        self.align(out, 0.0);
        out
    }

    pub fn align(&mut self, d: Vec2, tolerance: f32) -> Vec2 {
        let (x0, y0, x1, y1) = self.moving;
        let mine_x = [x0, (x0 + x1) / 2.0, x1];
        let mine_y = [y0, (y0 + y1) / 2.0, y1];
        let theirs_x = |b: &Bounds| [b.0, (b.0 + b.2) / 2.0, b.2];
        let theirs_y = |b: &Bounds| [b.1, (b.1 + b.3) / 2.0, b.3];
        // The smallest pull on each axis that brings something into line
        let pull = |mine: [f32; 3], theirs: &dyn Fn(&Bounds) -> [f32; 3], d: f32| {
            let mut best: Option<f32> = None;
            for b in &self.others {
                for t in theirs(b) {
                    for m in mine {
                        let p = t - (m + d);
                        if p.abs() <= tolerance && best.is_none_or(|q| p.abs() < q.abs()) {
                            best = Some(p);
                        }
                    }
                }
            }
            best.map_or(d, |p| d + p)
        };
        let mut out = Vec2::new(pull(mine_x, &theirs_x, d.x), pull(mine_y, &theirs_y, d.y));
        // Equal spacing: between two other boxes, the place leaving the
        // same gap on both sides, if nearer than the other pulls
        let nearer =
            |eq: f32, edge: f32, raw: f32| edge == raw || (eq - raw).abs() < (edge - raw).abs();
        if let Some(x) = self.equal_spacing(d, tolerance, true)
            && nearer(x, out.x, d.x)
        {
            out.x = x;
        }
        if let Some(y) = self.equal_spacing(d, tolerance, false)
            && nearer(y, out.y, d.y)
        {
            out.y = y;
        }
        // Every line that now matches, spanning both boxes
        let moved = (x0 + out.x, y0 + out.y, x1 + out.x, y1 + out.y);
        let mut lines = Vec::new();
        for b in &self.others {
            for (m, t) in [moved.0, (moved.0 + moved.2) / 2.0, moved.2]
                .into_iter()
                .flat_map(|m| theirs_x(b).map(move |t| (m, t)))
            {
                if (m - t).abs() < 0.01 {
                    let (top, bottom) = (moved.1.min(b.1), moved.3.max(b.3));
                    lines.push([Pos2::new(t, top), Pos2::new(t, bottom)]);
                }
            }
            for (m, t) in [moved.1, (moved.1 + moved.3) / 2.0, moved.3]
                .into_iter()
                .flat_map(|m| theirs_y(b).map(move |t| (m, t)))
            {
                if (m - t).abs() < 0.01 {
                    let (left, right) = (moved.0.min(b.0), moved.2.max(b.2));
                    lines.push([Pos2::new(left, t), Pos2::new(right, t)]);
                }
            }
        }
        self.lines = lines;
        self.labels = self.gaps(moved);
        out
    }

    /// The offset along x (`horizontal`) or y that puts the moved box
    /// midway between a box on each side overlapping it across, if within
    /// `tolerance` of `d`.
    fn equal_spacing(&self, d: Vec2, tolerance: f32, horizontal: bool) -> Option<f32> {
        let (x0, y0, x1, y1) = self.moving;
        if x1 - x0 < 1e-3 && y1 - y0 < 1e-3 {
            return None;
        }
        let m = (x0 + d.x, y0 + d.y, x1 + d.x, y1 + d.y);
        let neighbors = &self.others[1.min(self.others.len())..];
        let mut best: Option<f32> = None;
        for a in neighbors {
            for b in neighbors {
                let (lo, hi, size, cur, start) = if horizontal {
                    let across = |q: &Bounds| q.1 < m.3 && q.3 > m.1;
                    if !(across(a) && across(b)) || a.2 > b.0 {
                        continue;
                    }
                    (a.2, b.0, x1 - x0, d.x, x0)
                } else {
                    let across = |q: &Bounds| q.0 < m.2 && q.2 > m.0;
                    if !(across(a) && across(b)) || a.3 > b.1 {
                        continue;
                    }
                    (a.3, b.1, y1 - y0, d.y, y0)
                };
                if hi - lo < size {
                    continue;
                }
                let target = (lo + hi - size) / 2.0 - start;
                if (target - cur).abs() <= tolerance
                    && best.is_none_or(|q| (target - cur).abs() < (q - cur).abs())
                {
                    best = Some(target);
                }
            }
        }
        best
    }

    /// The gaps between the moved box and the nearest other box on each
    /// side that overlaps it across (not the canvas; nothing for a point).
    fn gaps(&self, m: Bounds) -> Vec<([Pos2; 2], f32)> {
        if m.2 - m.0 < 1e-3 && m.3 - m.1 < 1e-3 {
            return Vec::new();
        }
        let neighbors = &self.others[1.min(self.others.len())..];
        let mut out = Vec::new();
        let across_y = |b: &Bounds| b.1 < m.3 && b.3 > m.1;
        let across_x = |b: &Bounds| b.0 < m.2 && b.2 > m.0;
        let mid_y = |b: &Bounds| (m.1.max(b.1) + m.3.min(b.3)) / 2.0;
        let mid_x = |b: &Bounds| (m.0.max(b.0) + m.2.min(b.2)) / 2.0;
        // Left, right, above, below: the nearest that doesn't overlap
        let left = neighbors
            .iter()
            .filter(|b| across_y(b) && b.2 <= m.0)
            .max_by(|a, b| a.2.total_cmp(&b.2));
        if let Some(b) = left {
            let y = mid_y(b);
            out.push(([Pos2::new(b.2, y), Pos2::new(m.0, y)], m.0 - b.2));
        }
        let right = neighbors
            .iter()
            .filter(|b| across_y(b) && b.0 >= m.2)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some(b) = right {
            let y = mid_y(b);
            out.push(([Pos2::new(m.2, y), Pos2::new(b.0, y)], b.0 - m.2));
        }
        let above = neighbors
            .iter()
            .filter(|b| across_x(b) && b.3 <= m.1)
            .max_by(|a, b| a.3.total_cmp(&b.3));
        if let Some(b) = above {
            let x = mid_x(b);
            out.push(([Pos2::new(x, b.3), Pos2::new(x, m.1)], m.1 - b.3));
        }
        let below = neighbors
            .iter()
            .filter(|b| across_x(b) && b.1 >= m.3)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some(b) = below {
            let x = mid_x(b);
            out.push(([Pos2::new(x, m.3), Pos2::new(x, b.1)], b.1 - m.3));
        }
        out
    }
}

/// ⌘ held over the canvas with the Move tool (and nothing dragged):
/// distances from the active layer's pixels (or the selection) to the
/// layer under `pointer`, or to the canvas's edges when there is none.
pub fn measure(state: &DocState, pointer: Pos2) -> Vec<([Pos2; 2], f32)> {
    let doc = &state.doc;
    let Ok(mine) = op_core::transform::bounds(doc) else {
        return Vec::new();
    };
    let (x, y) = (pointer.x.floor(), pointer.y.floor());
    let under = doc.layers.iter().rev().find_map(|l| {
        if !l.visible || l.is_background || Some(l.id) == doc.active_layer {
            return None;
        }
        let image = l.image()?;
        let inside = x >= 0.0 && y >= 0.0 && x < doc.width as f32 && y < doc.height as f32;
        (inside && image.pixel(x as u32, y as u32)[3] > 0)
            .then(|| image.content_bounds())
            .flatten()
            .map(|(x0, y0, x1, y1)| (x0 as f32, y0 as f32, x1 as f32, y1 as f32))
    });
    let canvas = (0.0, 0.0, doc.width as f32, doc.height as f32);
    match under {
        Some(other) => {
            let mut g = SmartGuides {
                moving: mine,
                others: vec![canvas, other],
                lines: Vec::new(),
                labels: Vec::new(),
            };
            g.align(Vec2::ZERO, 0.0);
            g.labels
        }
        None => {
            // To the canvas's four edges, from the middle of each side
            let (x0, y0, x1, y1) = mine;
            let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
            [
                ([Pos2::new(0.0, cy), Pos2::new(x0, cy)], x0),
                ([Pos2::new(x1, cy), Pos2::new(canvas.2, cy)], canvas.2 - x1),
                ([Pos2::new(cx, 0.0), Pos2::new(cx, y0)], y0),
                ([Pos2::new(cx, y1), Pos2::new(cx, canvas.3)], canvas.3 - y1),
            ]
            .into_iter()
            .filter(|(_, d)| *d > 0.0)
            .collect()
        }
    }
}

/// Draws the smart guides of the move under way: 1 pt magenta lines.
pub fn draw(ui: &Ui, state: &DocState, clip: Rect, ppp: f32) {
    let painter = ui.painter().with_clip_rect(clip);
    draw_labels(&painter, state, &state.measure, ppp);
    let Some(smart) = &state.smart_guides else {
        return;
    };
    for [a, b] in &smart.lines {
        painter.line_segment(
            [to_screen(state, *a, ppp), to_screen(state, *b, ppp)],
            Stroke::new(crate::theme::pt(1.0), COLOR),
        );
    }
    draw_labels(&painter, state, &smart.labels, ppp);
}

/// Distances: each gap's line and its length in a magenta tag.
fn draw_labels(painter: &egui::Painter, state: &DocState, labels: &[([Pos2; 2], f32)], ppp: f32) {
    for ([a, b], length) in labels {
        let (a, b) = (to_screen(state, *a, ppp), to_screen(state, *b, ppp));
        painter.line_segment([a, b], Stroke::new(crate::theme::pt(1.0), COLOR));
        let text = format!("{} px", length.round());
        let galley = painter.layout_no_wrap(
            text,
            egui::FontId::proportional(crate::theme::pt(10.0)),
            Color32::WHITE,
        );
        let mid = a + (b - a) / 2.0;
        let tag = Rect::from_center_size(
            mid,
            galley.size() + Vec2::new(crate::theme::pt(8.0), crate::theme::pt(4.0)),
        );
        painter.rect_filled(tag, crate::theme::pt(2.0), COLOR);
        painter.galley(tag.center() - galley.size() / 2.0, galley, Color32::WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guides() -> SmartGuides {
        SmartGuides {
            moving: (10.0, 10.0, 30.0, 20.0),
            others: vec![(0.0, 0.0, 100.0, 80.0), (60.0, 50.0, 90.0, 70.0)],
            lines: Vec::new(),
            labels: Vec::new(),
        }
    }

    #[test]
    fn edges_and_centers_line_up() {
        let mut g = guides();
        // The box's center (20) 2 px short of the canvas's (50) after 28
        assert_eq!(g.align(Vec2::new(28.0, 0.0), 4.0).x, 30.0);
        assert!(g.lines.iter().any(|l| l[0].x == 50.0 && l[1].x == 50.0));
        // Its right edge onto the other layer's left edge (60), and its
        // middle (15 + 37) onto that layer's top (50), the nearer of what
        // lines up
        let d = g.align(Vec2::new(31.0, 37.0), 4.0);
        assert_eq!(d, Vec2::new(30.0, 35.0));
        // The lines span both boxes
        assert!(
            g.lines
                .iter()
                .any(|l| l[0].y == 50.0 && l[0].x == 40.0 && l[1].x == 90.0)
        );
        // Nothing near: as dragged, no lines
        let d = g.align(Vec2::new(5.0, 5.5), 1.0);
        assert_eq!(d, Vec2::new(5.0, 5.5));
        assert!(g.lines.is_empty());
    }

    #[test]
    fn equal_spacing_between_two_boxes() {
        // Boxes at 0–20 and 80–100 (rows 0–10); a 20 wide box from 10
        let mut g = SmartGuides {
            moving: (10.0, 0.0, 30.0, 10.0),
            others: vec![
                (0.0, 0.0, 200.0, 200.0),
                (0.0, 0.0, 20.0, 10.0),
                (80.0, 0.0, 100.0, 10.0),
            ],
            lines: Vec::new(),
            labels: Vec::new(),
        };
        // Moved 28: 2 short of the middle (40–60), it goes there
        let d = g.align(Vec2::new(28.0, 0.0), 4.0);
        assert_eq!(d.x, 30.0);
        let gaps: Vec<f32> = g.labels.iter().map(|l| l.1).collect();
        assert_eq!(gaps, vec![20.0, 20.0]);
    }

    #[test]
    fn gaps_and_points() {
        let mut g = guides();
        // Moved to (20, 50)–(40, 60): 20 px left of the other layer
        g.align(Vec2::new(10.0, 40.0), 0.0);
        assert_eq!(g.labels.len(), 1);
        let ([a, b], length) = g.labels[0];
        assert_eq!((a.x, b.x, length), (40.0, 60.0, 20.0));
        // A point (a zero box) lines up but has no gaps
        let mut p = SmartGuides {
            moving: (0.0, 0.0, 0.0, 0.0),
            ..guides()
        };
        let d = p.align(Vec2::new(58.0, 20.0), 4.0);
        assert_eq!(d.x, 60.0);
        assert!(p.labels.is_empty() && !p.lines.is_empty());
    }
}
