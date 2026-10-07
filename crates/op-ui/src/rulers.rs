//! Rulers, guides and the grid of the document window (View menu).
//!
//! Rulers run along the top and left of the canvas area, in pixels, with
//! the origin at the canvas's top-left corner. Dragging out of a ruler
//! makes a guide; with the Move tool (or Cmd held) guides can be dragged,
//! and dropping one outside the canvas area deletes it.

use egui::{Align2, Color32, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use op_core::Guide;

use crate::document_view::{to_doc, to_screen};
use crate::state::{DocState, GuideDrag, ViewOptions};
use crate::theme::pt;

/// Thickness of the rulers.
pub const RULER: f32 = pt(19.0);
/// Photoshop's default guide color (Cyan).
const GUIDE: Color32 = Color32::from_rgb(0x4a, 0xff, 0xff);
/// A selected guide's color.
const SELECTED_GUIDE: Color32 = Color32::from_rgb(0x2e, 0x7c, 0xf6);
const GRID: Color32 = Color32::from_rgba_premultiplied(0x50, 0x50, 0x50, 0x80);
const RULER_BG: Color32 = Color32::from_gray(0x47);
const TICK: Color32 = Color32::from_gray(0x66);
const LABEL: Color32 = Color32::from_gray(0x9e);
const POINTER: Color32 = Color32::from_gray(0xdc);
/// Ticks: labelled ones at least this far apart on screen, the smallest
/// at least this far (measured in Photoshop 2026 across its zoom levels).
const MAJOR_MIN: f32 = pt(30.0);
const MINOR_MIN: f32 = pt(4.75);
/// Guides closer than this (points) to the pointer can be grabbed.
const GRAB: f32 = pt(4.0);

/// The rulers' unit (Photoshop's Preferences › Units & Rulers, also picked
/// by right-clicking a ruler).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RulerUnit {
    #[default]
    Pixels,
    Inches,
    Centimeters,
    Millimeters,
    Points,
    Picas,
    Percent,
}

impl RulerUnit {
    /// In the menu's order.
    pub const ALL: [Self; 7] = [
        Self::Pixels,
        Self::Inches,
        Self::Centimeters,
        Self::Millimeters,
        Self::Points,
        Self::Picas,
        Self::Percent,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Pixels => "Pixels",
            Self::Inches => "Inches",
            Self::Centimeters => "Centimeters",
            Self::Millimeters => "Millimeters",
            Self::Points => "Points",
            Self::Picas => "Picas",
            Self::Percent => "Percent",
        }
    }

    /// Document pixels per unit along a ruler (percent is of the
    /// document's width on the top ruler, of its height on the left one).
    pub fn pixels(self, doc: &op_core::Document, horizontal: bool) -> f32 {
        let res = doc.resolution;
        match self {
            Self::Pixels => 1.0,
            Self::Inches => res,
            Self::Centimeters => res / 2.54,
            Self::Millimeters => res / 25.4,
            Self::Points => res / 72.0,
            Self::Picas => res / 6.0,
            Self::Percent => {
                let side = if horizontal { doc.width } else { doc.height };
                side as f32 / 100.0
            }
        }
    }

    /// The tick spacings the ruler can use, in units, ascending: round
    /// numbers, halves of an inch down to sixteenths, and whole picas
    /// (points by the pica) as Photoshop's rulers show them.
    fn ladder(self) -> Vec<f64> {
        let decimal = || (-3..8).flat_map(|e| [1.0, 2.0, 5.0].map(|k| k * 10f64.powi(e)));
        match self {
            Self::Inches => [0.0625, 0.125, 0.25, 0.5]
                .into_iter()
                .chain(decimal().filter(|&v| v >= 1.0))
                .collect(),
            Self::Picas | Self::Points => {
                let scale = if self == Self::Points { 12.0 } else { 1.0 };
                [
                    0.5, 1.0, 3.0, 6.0, 12.0, 24.0, 60.0, 120.0, 240.0, 600.0, 1200.0, 2400.0,
                    6000.0,
                ]
                .into_iter()
                .map(|v| v * scale)
                .collect()
            }
            _ => decimal().collect(),
        }
    }
}

/// A ruler's ticks, in units: labelled ones every `major`, the smallest
/// every `minor`, medium ones every `medium` (when there is a step in
/// between that fits).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ticks {
    pub major: f64,
    pub medium: Option<f64>,
    pub minor: f64,
}

/// Picks the ticks for `unit` at `points_per_unit` on screen: the major
/// step is the smallest on the ladder at least `MAJOR_MIN` apart; the minor
/// the smallest dividing it at least `MINOR_MIN` apart; the medium the
/// largest between them dividing the major and divided by the minor.
pub fn ticks(unit: RulerUnit, points_per_unit: f32) -> Ticks {
    let ladder = unit.ladder();
    let scale = points_per_unit as f64;
    let divides = |a: f64, b: f64| {
        let q = a / b;
        (q - q.round()).abs() < 1e-6
    };
    let major = ladder
        .iter()
        .copied()
        .find(|&v| v * scale >= MAJOR_MIN as f64)
        .unwrap_or(*ladder.last().expect("ladders aren't empty"));
    let minor = ladder
        .iter()
        .copied()
        .find(|&v| v <= major && v * scale >= MINOR_MIN as f64 && divides(major, v))
        .unwrap_or(major);
    let medium = ladder
        .iter()
        .copied()
        .rfind(|&v| v > minor && v < major && divides(major, v) && divides(v, minor));
    Ticks {
        major,
        medium,
        minor,
    }
}

/// A ruler number: the distance from the origin, without a sign (as
/// Photoshop writes them), whole or with up to three decimals.
fn ruler_label(v: f64) -> String {
    let v = v.abs();
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
}

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

/// The corner square where the rulers meet (dragged to move their origin).
pub fn corner(area: Rect) -> Rect {
    Rect::from_min_size(area.min, Vec2::splat(RULER))
}

/// Draws one ruler. `horizontal` is the top ruler; `pointer` marks the
/// pointer's position on it. Measured on Photoshop 2026: 19 pt, `#474747`;
/// ticks one physical pixel wide in `#666666`, standing on the canvas side
/// (major ones the full width, medium 4 pt, minor 2 pt); numbers in
/// `#9e9e9e`, after their tick on the top ruler and digit by digit below
/// it on the left one.
fn draw_ruler(
    ui: &Ui,
    state: &DocState,
    unit: RulerUnit,
    rect: Rect,
    horizontal: bool,
    pointer: Option<Pos2>,
    ppp: f32,
) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0, RULER_BG);
    let unit_px = unit.pixels(&state.doc, horizontal) as f64;
    // Points per unit along the ruler (the rulers follow the image upright)
    let zoom_pt = (state.view.zoom / ppp) as f64;
    let t = ticks(unit, (unit_px * zoom_pt) as f32);
    let origin = to_screen(state, state.ruler_origin, ppp);
    let (start, len, o) = if horizontal {
        (rect.left(), rect.width(), origin.x)
    } else {
        (rect.top(), rect.height(), origin.y)
    };
    let per_unit = unit_px * zoom_pt;
    let first = (((start - o) as f64 / per_unit) / t.minor).floor() as i64;
    let last = (((start + len - o) as f64 / per_unit) / t.minor).ceil() as i64;
    let font = FontId::proportional(pt(10.5));
    let px = 1.0 / ppp;
    let on = |a: f64, b: f64| {
        let q = a / b;
        (q - q.round()).abs() < 1e-6
    };
    for i in first..=last {
        let v = i as f64 * t.minor;
        // One physical pixel, on the pixel the position falls in
        let s = ((o as f64 + v * per_unit) as f32 * ppp).floor() / ppp;
        let major = on(v, t.major);
        let tick = if major {
            RULER
        } else if t.medium.is_some_and(|m| on(v, m)) {
            pt(4.0)
        } else {
            pt(2.0)
        };
        let line = if horizontal {
            Rect::from_min_max(
                Pos2::new(s, rect.bottom() - tick),
                Pos2::new(s + px, rect.bottom()),
            )
        } else {
            Rect::from_min_max(
                Pos2::new(rect.right() - tick, s),
                Pos2::new(rect.right(), s + px),
            )
        };
        painter.rect_filled(line, 0, TICK);
        if major {
            let label = ruler_label(v);
            if horizontal {
                // Adobe Clean's digits are wider than Source Sans 3's: the
                // tracking keeps the numbers as long as Photoshop's
                let mut job = egui::text::LayoutJob::default();
                job.append(
                    &label,
                    0.0,
                    egui::TextFormat {
                        font_id: font.clone(),
                        color: LABEL,
                        extra_letter_spacing: pt(1.1),
                        ..Default::default()
                    },
                );
                let galley = painter.layout_job(job);
                let y = rect.top() + pt(11.25) - galley.size().y / 2.0;
                painter.galley(Pos2::new(s + pt(1.5), y), galley, LABEL);
            } else {
                // Photoshop writes the left ruler's numbers digit by digit
                // down the ruler
                for (k, ch) in label.chars().enumerate() {
                    painter.text(
                        Pos2::new(rect.left() + pt(9.5), s + pt(5.75) + k as f32 * pt(9.75)),
                        Align2::CENTER_CENTER,
                        ch,
                        font.clone(),
                        LABEL,
                    );
                }
            }
        }
    }
    // The pointer's position: a short light line
    if let Some(p) = pointer {
        let stroke = Stroke::new(pt(1.0), POINTER);
        if horizontal && rect.x_range().contains(p.x) {
            painter.line_segment(
                [
                    Pos2::new(p.x, rect.top() + pt(1.5)),
                    Pos2::new(p.x, rect.top() + pt(13.5)),
                ],
                stroke,
            );
        } else if !horizontal && rect.y_range().contains(p.y) {
            painter.line_segment(
                [
                    Pos2::new(rect.left() + pt(1.5), p.y),
                    Pos2::new(rect.left() + pt(13.5), p.y),
                ],
                stroke,
            );
        }
    }
}

/// Draws both rulers and their corner, and the crosshair while the origin
/// is dragged out of the corner.
pub fn draw_rulers(
    ui: &Ui,
    state: &DocState,
    unit: RulerUnit,
    area: Rect,
    pointer: Option<Pos2>,
    ppp: f32,
) {
    let (top, left, canvas) = layout(area);
    ui.painter().rect_filled(corner(area), 0, RULER_BG);
    draw_ruler(ui, state, unit, top, true, pointer, ppp);
    draw_ruler(ui, state, unit, left, false, pointer, ppp);
    if let Some(p) = state.origin_drag {
        let painter = ui.painter_at(canvas);
        let stroke = Stroke::new(1.0, POINTER);
        painter.line_segment(
            [
                Pos2::new(p.x, canvas.top()),
                Pos2::new(p.x, canvas.bottom()),
            ],
            stroke,
        );
        painter.line_segment(
            [
                Pos2::new(canvas.left(), p.y),
                Pos2::new(canvas.right(), p.y),
            ],
            stroke,
        );
    }
}

/// Dragging out of the corner moves the rulers' origin to where it is let
/// go (snapping like other drags); double-clicking the corner puts it back
/// at the image's top-left corner. Not recorded in the history.
pub fn origin_input(ui: &Ui, state: &mut DocState, view: &ViewOptions, area: Rect, ppp: f32) {
    let r = ui.interact(
        corner(area),
        ui.id().with("ruler-corner"),
        Sense::click_and_drag(),
    );
    if r.double_clicked() {
        state.ruler_origin = Pos2::ZERO;
    }
    if r.drag_started() {
        crate::snap::begin(state, view, false);
    }
    if r.dragged()
        && let Some(p) = r.interact_pointer_pos()
    {
        state.origin_drag = Some(p);
        ui.ctx().request_repaint();
    }
    if r.drag_stopped()
        && let Some(p) = state.origin_drag.take()
    {
        let d = to_doc(state, p, ppp);
        state.ruler_origin = crate::snap::point(ui, state, d, ppp);
    }
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
    let stroke = |k: i64| {
        Stroke::new(
            1.0,
            if k.rem_euclid(4) == 0 {
                GRID.gamma_multiply(1.6)
            } else {
                GRID
            },
        )
    };
    let origin = state.ruler_origin;
    for (k, x) in grid_lines(origin.x, step, w) {
        painter.line_segment(line(Pos2::new(x, 0.0), Pos2::new(x, h)), stroke(k));
    }
    for (k, y) in grid_lines(origin.y, step, h) {
        painter.line_segment(line(Pos2::new(0.0, y), Pos2::new(w, y)), stroke(k));
    }
}

/// The grid's lines across `0..=size`: every `step` from the rulers'
/// origin `o` (Photoshop's grid starts at the origin), numbered from it.
pub fn grid_lines(o: f32, step: f32, size: f32) -> impl Iterator<Item = (i64, f32)> {
    let first = ((0.0 - o) / step).ceil() as i64;
    let last = ((size - o) / step).floor() as i64;
    (first..=last).map(move |k| (k, o + k as f32 * step))
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
    let selected = state.guide_selection();
    let guides = state
        .doc
        .guides
        .iter()
        .enumerate()
        .filter(|(i, _)| Some(*i) != moving)
        .map(|(i, g)| (*g, selected.contains(&i)))
        .chain(dragged.map(|g| (g, false)));
    for (g, chosen) in guides {
        // A selected guide shows in the selection color; others in their
        // own color, if they have one
        let color = if chosen {
            SELECTED_GUIDE
        } else {
            g.color
                .map_or(GUIDE, |[r, g, b]| Color32::from_rgb(r, g, b))
        };
        painter.line_segment(guide_line(state, g, ppp), Stroke::new(1.0, color));
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
            color: None,
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
        // View › Snap: the guide comes to the targets' lines (not to the
        // guide being moved)
        let raw = to_doc(state, p, ppp);
        let d = match (&state.snap, ui.input(|i| i.modifiers.ctrl)) {
            (Some(t), false) => {
                let mut t = t.clone();
                if let Some(i) = drag.index {
                    let own = state.doc.guides[i];
                    let lines = if own.vertical { &mut t.xs } else { &mut t.ys };
                    if let Some(k) = lines.iter().position(|&v| v == own.position) {
                        lines.remove(k);
                    }
                }
                t.point(raw, crate::snap::tolerance(state, ppp))
            }
            _ => raw,
        };
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
        self.extras && self.guides && self.canvas_guides
    }

    pub fn grid_visible(&self) -> bool {
        self.extras && self.grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Photoshop 2026's rulers, read at its zoom levels on a 72 ppi
    /// image (points per unit = zoom / 2 on its Retina display).
    #[test]
    fn ticks_match_photoshops() {
        let px = |zoom: f32| ticks(RulerUnit::Pixels, pt(zoom / 2.0));
        let t = |major, medium, minor| Ticks {
            major,
            medium,
            minor,
        };
        assert_eq!(px(0.25), t(500.0, Some(100.0), 50.0));
        assert_eq!(px(1.0 / 3.0), t(200.0, Some(100.0), 50.0));
        assert_eq!(px(0.5), t(200.0, Some(100.0), 20.0));
        assert_eq!(px(2.0 / 3.0), t(100.0, None, 20.0));
        assert_eq!(px(1.0), t(100.0, Some(50.0), 10.0));
        assert_eq!(px(2.0), t(50.0, Some(10.0), 5.0));
        assert_eq!(px(3.5417), t(20.0, Some(10.0), 5.0));
        assert_eq!(px(4.0), t(20.0, Some(10.0), 5.0));
        assert_eq!(px(8.0), t(10.0, None, 2.0));
        // The other units at 100% (72 ppi: an inch is 36 points on screen)
        assert_eq!(ticks(RulerUnit::Inches, pt(36.0)), t(1.0, Some(0.5), 0.25));
        let cm = 36.0 / 2.54;
        assert_eq!(
            ticks(RulerUnit::Centimeters, pt(cm)),
            t(5.0, Some(1.0), 0.5)
        );
        assert_eq!(
            ticks(RulerUnit::Millimeters, pt(cm / 10.0)),
            t(50.0, Some(10.0), 5.0)
        );
        assert_eq!(ticks(RulerUnit::Points, pt(0.5)), t(72.0, Some(36.0), 12.0));
        assert_eq!(ticks(RulerUnit::Picas, pt(6.0)), t(6.0, Some(3.0), 1.0));
        // Percent of a 64 px wide image
        assert_eq!(ticks(RulerUnit::Percent, pt(0.32)), t(100.0, None, 20.0));
    }

    #[test]
    fn labels_drop_the_sign() {
        assert_eq!(ruler_label(-100.0), "100");
        assert_eq!(ruler_label(0.5), "0.5");
        assert_eq!(ruler_label(1.0000000001), "1");
    }

    #[test]
    fn the_grid_starts_at_the_origin() {
        let lines: Vec<_> = grid_lines(30.0, 18.0, 72.0).collect();
        assert_eq!(lines, [(-1, 12.0), (0, 30.0), (1, 48.0), (2, 66.0)]);
    }
}
