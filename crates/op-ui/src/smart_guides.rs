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
        let out = Vec2::new(pull(mine_x, &theirs_x, d.x), pull(mine_y, &theirs_y, d.y));
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
        out
    }
}

/// Draws the smart guides of the move under way: 1 pt magenta lines.
pub fn draw(ui: &Ui, state: &DocState, clip: Rect, ppp: f32) {
    let Some(smart) = &state.smart_guides else {
        return;
    };
    let painter = ui.painter().with_clip_rect(clip);
    for [a, b] in &smart.lines {
        painter.line_segment(
            [to_screen(state, *a, ppp), to_screen(state, *b, ppp)],
            Stroke::new(crate::theme::pt(1.0), COLOR),
        );
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
}
