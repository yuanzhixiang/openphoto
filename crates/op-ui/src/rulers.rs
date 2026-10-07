//! Rulers, guides and the grid of the document window (View menu).
//!
//! Rulers run along the top and left of the canvas area, in pixels, with
//! the origin at the canvas's top-left corner. Dragging out of a ruler
//! makes a guide; with the Move tool (or Cmd held) guides can be dragged,
//! and dropping one outside the canvas area deletes it.

use egui::{Align2, Color32, CursorIcon, FontId, Pos2, Rect, Stroke, Ui, Vec2};
use op_core::Guide;

use crate::document_view::{to_doc, to_screen};
use crate::state::{DocState, GuideDrag, ViewOptions};
use crate::theme::{color, pt};

/// Thickness of the rulers.
pub const RULER: f32 = pt(16.0);
/// Photoshop's default guide color (Cyan).
const GUIDE: Color32 = Color32::from_rgb(0x4a, 0xff, 0xff);
const GRID: Color32 = Color32::from_rgba_premultiplied(0x50, 0x50, 0x50, 0x80);
const RULER_BG: Color32 = Color32::from_gray(0x3c);
const TICK: Color32 = Color32::from_gray(0xa8);
/// Guides closer than this (points) to the pointer can be grabbed.
const GRAB: f32 = pt(4.0);

/// Splits the document window's canvas area into the two rulers, their
/// corner square and the canvas left over.
pub fn layout(area: Rect) -> (Rect, Rect, Rect) {
    let top = Rect::from_min_max(
        Pos2::new(area.left() + RULER, area.top()),
        Pos2::new(area.right(), area.top() + RULER),
    );
    let left = Rect::from_min_max(
        Pos2::new(area.left(), area.top() + RULER),
        Pos2::new(area.left() + RULER, area.bottom()),
    );
    let canvas = Rect::from_min_max(Pos2::new(area.left() + RULER, area.top() + RULER), area.max);
    (top, left, canvas)
}

/// The labelled tick spacing in document pixels: the smallest "nice" step
/// at least `min_points` apart on screen.
fn major_step(zoom_pt: f32, min_points: f32) -> f32 {
    let mut base = 1.0;
    loop {
        for k in [1.0, 2.0, 5.0] {
            let step = base * k;
            if step * zoom_pt >= min_points {
                return step;
            }
        }
        base *= 10.0;
    }
}

/// Draws one ruler. `horizontal` is the top ruler; `pointer` marks the
/// pointer's position on it.
fn draw_ruler(
    ui: &Ui,
    state: &DocState,
    rect: Rect,
    horizontal: bool,
    pointer: Option<Pos2>,
    ppp: f32,
) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0, RULER_BG);
    let edge = if horizontal {
        [rect.left_bottom(), rect.right_bottom()]
    } else {
        [rect.right_top(), rect.right_bottom()]
    };
    painter.line_segment(edge, Stroke::new(1.0, color::SEPARATOR));
    // Points per document pixel
    let zoom_pt = state.view.zoom / ppp;
    let step = major_step(zoom_pt, pt(60.0));
    let minor = step / 10.0;
    let origin = to_screen(state, Pos2::ZERO, ppp);
    let (start, len, o) = if horizontal {
        (rect.left(), rect.width(), origin.x)
    } else {
        (rect.top(), rect.height(), origin.y)
    };
    let first = ((start - o) / zoom_pt / minor).floor() as i64;
    let last = ((start + len - o) / zoom_pt / minor).ceil() as i64;
    let font = FontId::proportional(pt(9.0));
    for i in first..=last {
        let v = i as f32 * minor;
        let s = o + v * zoom_pt;
        let tick = if i % 10 == 0 {
            RULER
        } else if i % 5 == 0 {
            RULER * 0.4
        } else {
            RULER * 0.2
        };
        let (a, b) = if horizontal {
            (
                Pos2::new(s, rect.bottom() - tick),
                Pos2::new(s, rect.bottom()),
            )
        } else {
            (
                Pos2::new(rect.right() - tick, s),
                Pos2::new(rect.right(), s),
            )
        };
        painter.line_segment([a, b], Stroke::new(1.0, TICK));
        if i % 10 == 0 {
            let label = format!("{}", v.round() as i64);
            if horizontal {
                painter.text(
                    Pos2::new(s + pt(2.0), rect.top() + pt(1.0)),
                    Align2::LEFT_TOP,
                    label,
                    font.clone(),
                    TICK,
                );
            } else {
                // Photoshop writes the left ruler's numbers digit by digit
                // down the ruler
                for (k, ch) in label.chars().enumerate() {
                    painter.text(
                        Pos2::new(rect.left() + pt(3.0), s + pt(2.0) + k as f32 * pt(8.0)),
                        Align2::LEFT_TOP,
                        ch,
                        font.clone(),
                        TICK,
                    );
                }
            }
        }
    }
    if let Some(p) = pointer {
        let dotted = Stroke::new(1.0, color::TEXT);
        if horizontal && rect.x_range().contains(p.x) {
            painter.line_segment(
                [Pos2::new(p.x, rect.top()), Pos2::new(p.x, rect.bottom())],
                dotted,
            );
        } else if !horizontal && rect.y_range().contains(p.y) {
            painter.line_segment(
                [Pos2::new(rect.left(), p.y), Pos2::new(rect.right(), p.y)],
                dotted,
            );
        }
    }
}

/// Draws both rulers and their corner.
pub fn draw_rulers(ui: &Ui, state: &DocState, area: Rect, pointer: Option<Pos2>, ppp: f32) {
    let (top, left, _) = layout(area);
    let corner = Rect::from_min_size(area.min, Vec2::splat(RULER));
    ui.painter().rect_filled(corner, 0, RULER_BG);
    draw_ruler(ui, state, top, true, pointer, ppp);
    draw_ruler(ui, state, left, false, pointer, ppp);
}

/// The grid over the image: a line every inch, with four subdivisions
/// (Photoshop's defaults), drawn when lines are at least 4 points apart.
pub fn draw_grid(ui: &Ui, state: &DocState, canvas: Rect, ppp: f32) {
    let painter = ui.painter_at(canvas);
    let (w, h) = (state.doc.width as f32, state.doc.height as f32);
    let step = state.doc.resolution / 4.0;
    let zoom_pt = state.view.zoom / ppp;
    if step * zoom_pt < pt(4.0) {
        return;
    }
    // Lines are mapped end to end, so they follow the view's rotation
    let line = |a: Pos2, b: Pos2| [to_screen(state, a, ppp), to_screen(state, b, ppp)];
    let mut i = 0;
    loop {
        let v = i as f32 * step;
        if v > w.max(h) {
            break;
        }
        let stroke = Stroke::new(
            1.0,
            if i % 4 == 0 {
                GRID.gamma_multiply(1.6)
            } else {
                GRID
            },
        );
        if v <= w {
            painter.line_segment(line(Pos2::new(v, 0.0), Pos2::new(v, h)), stroke);
        }
        if v <= h {
            painter.line_segment(line(Pos2::new(0.0, v), Pos2::new(w, v)), stroke);
        }
        i += 1;
    }
}

/// Two screen points on a guide, far apart enough to cross the canvas
/// area whatever the view's rotation.
fn guide_line(state: &DocState, g: Guide, ppp: f32) -> [Pos2; 2] {
    let far = 1.0e6;
    let (a, b) = if g.vertical {
        (Pos2::new(g.position, -far), Pos2::new(g.position, far))
    } else {
        (Pos2::new(-far, g.position), Pos2::new(far, g.position))
    };
    [to_screen(state, a, ppp), to_screen(state, b, ppp)]
}

/// Guides across the whole canvas area, plus the one being dragged.
pub fn draw_guides(ui: &Ui, state: &DocState, canvas: Rect, ppp: f32) {
    let painter = ui.painter_at(canvas);
    let dragged = state.guide_drag.map(|d| d.guide);
    let moving = state.guide_drag.and_then(|d| d.index);
    let guides = state
        .doc
        .guides
        .iter()
        .enumerate()
        .filter(|(i, _)| Some(*i) != moving)
        .map(|(_, g)| *g)
        .chain(dragged);
    for g in guides {
        painter.line_segment(guide_line(state, g, ppp), Stroke::new(1.0, GUIDE));
    }
}

/// The guide under the screen point `p`, if any.
pub fn guide_at(state: &DocState, p: Pos2, ppp: f32) -> Option<usize> {
    // The distance in document pixels, scaled to screen points
    let d = to_doc(state, p, ppp);
    let scale = state.view.zoom / ppp;
    state
        .doc
        .guides
        .iter()
        .position(|&g| (if g.vertical { d.x } else { d.y } - g.position).abs() * scale <= GRAB)
}

pub fn guide_cursor(state: &DocState, index: usize) -> CursorIcon {
    if state.doc.guides[index].vertical {
        CursorIcon::ResizeColumn
    } else {
        CursorIcon::ResizeRow
    }
}

/// Starts dragging a new guide out of a ruler (`vertical` for the left one).
pub fn start_new(state: &mut DocState, vertical: bool, p: Pos2, ppp: f32) {
    let d = to_doc(state, p, ppp);
    state.guide_drag = Some(GuideDrag {
        index: None,
        guide: Guide {
            vertical,
            position: if vertical { d.x } else { d.y },
        },
    });
}

/// Starts moving the guide at `index`.
pub fn start_move(state: &mut DocState, index: usize) {
    state.guide_drag = Some(GuideDrag {
        index: Some(index),
        guide: state.doc.guides[index],
    });
}

/// Follows the pointer while a guide is dragged; on release, adds, moves
/// or (when dropped outside the canvas area) deletes it, recording "New
/// Guide", "Move Guide" or "Delete Guide".
pub fn drag(ui: &Ui, state: &mut DocState, canvas: Rect, ppp: f32) {
    let Some(mut drag) = state.guide_drag else {
        return;
    };
    let pointer = ui.input(|i| i.pointer.interact_pos().or(i.pointer.latest_pos()));
    if let Some(p) = pointer {
        let d = to_doc(state, p, ppp);
        drag.guide.position = if drag.guide.vertical { d.x } else { d.y };
        state.guide_drag = Some(drag);
    }
    if ui.input(|i| i.pointer.primary_down()) {
        ui.ctx().request_repaint();
        return;
    }
    state.guide_drag = None;
    let inside = pointer.is_some_and(|p| canvas.contains(p));
    match (drag.index, inside) {
        (None, true) => {
            state.doc.guides.push(drag.guide);
            state.record("New Guide");
        }
        (Some(i), true) => {
            state.doc.guides[i] = drag.guide;
            state.record("Move Guide");
        }
        (Some(i), false) => {
            state.doc.guides.remove(i);
            state.record("Delete Guide");
        }
        (None, false) => {}
    }
}

impl ViewOptions {
    /// Guides are drawn when both Extras and Show > Guides are on.
    pub fn guides_visible(&self) -> bool {
        self.extras && self.guides
    }

    pub fn grid_visible(&self) -> bool {
        self.extras && self.grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruler_steps_are_round_numbers() {
        assert_eq!(major_step(1.0, 60.0), 100.0);
        assert_eq!(major_step(0.5, 60.0), 200.0);
        assert_eq!(major_step(4.0, 60.0), 20.0);
        assert_eq!(major_step(32.0, 60.0), 2.0);
    }
}
