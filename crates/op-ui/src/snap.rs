//! View › Snap: points and moved boxes pulled to guides, grid lines, layer
//! edges and the document's bounds when they come within a few screen
//! points of them.

use egui::{Pos2, Vec2};
use op_core::LayerId;

use crate::state::{DocState, ViewOptions};

/// How close, in screen points, something has to come to snap.
pub const DISTANCE: f32 = 8.0;

/// What a drag can snap to, gathered when it starts (layer bounds are not
/// cheap to find every frame).
#[derive(Clone, Debug, Default)]
pub struct Targets {
    /// Vertical lines (document x) and horizontal lines (document y).
    pub xs: Vec<f32>,
    pub ys: Vec<f32>,
    /// For the Move tool: the box being moved, as it was at the start.
    pub moving: Option<(f32, f32, f32, f32)>,
}

impl Targets {
    /// The targets View › Snap To turns on: guides and the grid while they
    /// show, the edges of visible layers other than `skip`, the document's
    /// edges. `None` when Snap is off.
    pub fn new(state: &DocState, view: &ViewOptions, skip: &[LayerId]) -> Option<Self> {
        if !view.snap {
            return None;
        }
        let doc = &state.doc;
        let (w, h) = (doc.width as f32, doc.height as f32);
        let mut t = Self::default();
        if view.snap_guides && view.guides_visible() {
            for g in &doc.guides {
                if g.vertical {
                    t.xs.push(g.position);
                } else {
                    t.ys.push(g.position);
                }
            }
        }
        if view.snap_grid && view.grid_visible() {
            // The grid's lines and subdivisions (see rulers::draw_grid)
            let step = doc.resolution / 4.0;
            let mut v = 0.0;
            while step > 0.0 && v <= w.max(h) {
                if v <= w {
                    t.xs.push(v);
                }
                if v <= h {
                    t.ys.push(v);
                }
                v += step;
            }
        }
        if view.snap_layers {
            for layer in &doc.layers {
                if !layer.visible || layer.is_background || skip.contains(&layer.id) {
                    continue;
                }
                if let Some((x0, y0, x1, y1)) = layer.image().and_then(|i| i.content_bounds()) {
                    t.xs.extend([x0 as f32, x1 as f32]);
                    t.ys.extend([y0 as f32, y1 as f32]);
                }
            }
        }
        if view.snap_bounds {
            t.xs.extend([0.0, w]);
            t.ys.extend([0.0, h]);
        }
        Some(t)
    }

    /// `p` with each coordinate pulled to the nearest target within
    /// `tolerance` (document pixels).
    pub fn point(&self, p: Pos2, tolerance: f32) -> Pos2 {
        Pos2::new(
            nearest(&self.xs, p.x, tolerance).unwrap_or(p.x),
            nearest(&self.ys, p.y, tolerance).unwrap_or(p.y),
        )
    }

    /// The Move tool's offset adjusted so an edge or the center of the
    /// moved box lands on a target, when one is within `tolerance`.
    pub fn offset(&self, d: Vec2, tolerance: f32) -> Vec2 {
        let Some((x0, y0, x1, y1)) = self.moving else {
            return d;
        };
        let pull = |targets: &[f32], edges: [f32; 3], d: f32| {
            edges
                .iter()
                .filter_map(|&e| nearest(targets, e + d, tolerance).map(|t| t - e))
                .min_by(|a, b| (a - d).abs().total_cmp(&(b - d).abs()))
                .unwrap_or(d)
        };
        Vec2::new(
            pull(&self.xs, [x0, x1, (x0 + x1) / 2.0], d.x),
            pull(&self.ys, [y0, y1, (y0 + y1) / 2.0], d.y),
        )
    }
}

fn nearest(targets: &[f32], v: f32, tolerance: f32) -> Option<f32> {
    targets
        .iter()
        .copied()
        .filter(|t| (t - v).abs() <= tolerance)
        .min_by(|a, b| (a - v).abs().total_cmp(&(b - v).abs()))
}

/// The snap distance in document pixels at the current zoom.
pub fn tolerance(state: &DocState, ppp: f32) -> f32 {
    DISTANCE * ppp / state.view.zoom
}

/// Starts snapping for a drag: the Move tool leaves out the layers it
/// moves and remembers the box it moves.
pub fn begin(state: &mut DocState, view: &ViewOptions, moving: bool) {
    let skip = if moving {
        let mut ids = state.doc.selected_layers();
        for id in ids.clone() {
            ids.extend(op_core::link::linked_with(&state.doc, id));
        }
        ids
    } else {
        Vec::new()
    };
    state.snap = Targets::new(state, view, &skip).map(|mut t| {
        if moving {
            t.moving = op_core::transform::bounds(&state.doc).ok();
        }
        t
    });
}

/// A document point pulled to the drag's targets (Control held: as is,
/// as in Photoshop).
pub fn point(ui: &egui::Ui, state: &DocState, p: Pos2, ppp: f32) -> Pos2 {
    match &state.snap {
        Some(t) if !ui.input(|i| i.modifiers.ctrl) => t.point(p, tolerance(state, ppp)),
        _ => p,
    }
}

/// The Move tool's offset pulled to the drag's targets.
pub fn offset(ui: &egui::Ui, state: &DocState, d: Vec2, ppp: f32) -> Vec2 {
    match &state.snap {
        Some(t) if !ui.input(|i| i.modifiers.ctrl) => t.offset(d, tolerance(state, ppp)),
        _ => d,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_and_boxes_snap_within_the_distance() {
        let t = Targets {
            xs: vec![0.0, 100.0],
            ys: vec![50.0],
            moving: Some((10.0, 10.0, 30.0, 20.0)),
        };
        assert_eq!(t.point(Pos2::new(97.0, 54.0), 4.0), Pos2::new(100.0, 50.0));
        assert_eq!(t.point(Pos2::new(90.0, 60.0), 4.0), Pos2::new(90.0, 60.0));
        // The box's right edge (30) moved by 68 is 2 short of 100
        assert_eq!(t.offset(Vec2::new(68.0, 0.0), 4.0).x, 70.0);
        // Its center (15) moved by 37 is 2 past 50
        assert_eq!(t.offset(Vec2::new(0.0, 37.0), 4.0).y, 35.0);
        assert_eq!(t.offset(Vec2::new(50.0, 0.0), 4.0).x, 50.0);
    }
}
