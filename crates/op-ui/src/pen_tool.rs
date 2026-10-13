//! Path-mode Pen canvas input. Geometry stays editable and pixel-independent.
use crate::{
    document_view::{to_doc, to_screen},
    state::DocState,
};
use egui::{Key, PointerButton, Pos2, Rect, Stroke, Ui};
use op_core::path::{Knot, Path, Subpath};
#[derive(Default)]
pub struct Session {
    pub continuation: Option<(usize, usize)>,
    drag: Option<Drag>,
    convert: Option<ConvertDrag>,
}
struct Drag {
    before: op_core::Snapshot,
    path: usize,
    sub: usize,
    knot: usize,
    closing: bool,
}
impl Session {
    pub fn busy(&self) -> bool {
        self.drag.is_some() || self.convert.is_some()
    }
}
pub fn input(ui: &Ui, response: &egui::Response, state: &mut DocState, ppp: f32) {
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        if let Some(drag) = state.pen_session.drag.take() {
            state.doc.restore(&drag.before);
        }
        state.pen_session.continuation = None;
        return;
    }
    if response.hovered()
        && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary))
        && let Some(p) = ui.input(|i| i.pointer.interact_pos())
    {
        let p = to_doc(state, p, ppp);
        let before = state.doc.snapshot();
        let continuing = state.pen_session.continuation.filter(|&(path, sub)| {
            state
                .doc
                .paths
                .get(path)
                .and_then(|p| p.subpaths.get(sub))
                .is_some_and(|s| !s.closed)
        });
        let (path, sub) = if let Some(existing) = continuing {
            existing
        } else {
            let path = state.doc.paths.len();
            state.doc.paths.push(Path {
                name: format!("Path {}", path + 1),
                subpaths: vec![Subpath::default()],
            });
            (path, 0)
        };
        let first = state.doc.paths[path].subpaths[sub]
            .knots
            .first()
            .map(|k| k.anchor);
        let closing = state.doc.paths[path].subpaths[sub].knots.len() > 1
            && first.is_some_and(|a| {
                to_screen(state, Pos2::new(a.0, a.1), ppp).distance(to_screen(state, p, ppp)) <= 6.
            });
        let knot = if closing {
            0
        } else {
            let s = &mut state.doc.paths[path].subpaths[sub];
            s.knots.push(Knot::corner((p.x, p.y)));
            s.knots.len() - 1
        };
        state.doc.active_path = Some(path);
        state.pen_session.continuation = Some((path, sub));
        state.pen_session.drag = Some(Drag {
            before,
            path,
            sub,
            knot,
            closing,
        });
    }
    let Some(d) = state.pen_session.drag.as_ref() else {
        return;
    };
    let (path, sub, knot, closing) = (d.path, d.sub, d.knot, d.closing);
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        let k = &mut state.doc.paths[path].subpaths[sub].knots[knot];
        let (mut dx, mut dy) = (p.x - k.anchor.0, p.y - k.anchor.1);
        if ui.input(|i| i.modifiers.shift) {
            let length = dx.hypot(dy);
            let step = std::f32::consts::FRAC_PI_4;
            let a = (dy.atan2(dx) / step).round() * step;
            dx = length * a.cos();
            dy = length * a.sin();
        }
        if dx.hypot(dy) > 0.5 {
            k.outgoing = (k.anchor.0 + dx, k.anchor.1 + dy);
            if !ui.input(|i| i.modifiers.alt) {
                k.incoming = (k.anchor.0 - dx, k.anchor.1 - dy);
                k.linked = true;
            } else {
                k.linked = false;
            }
        }
    }
    if !ui.input(|i| i.pointer.primary_down()) {
        state.pen_session.drag = None;
        if closing {
            state.doc.paths[path].subpaths[sub].closed = true;
            state.pen_session.continuation = None;
        }
        state.doc.mark_dirty();
        state.record("Pen Tool");
    }
}
pub fn draw(ui: &Ui, state: &DocState, canvas: Rect, ppp: f32) {
    let Some(path) = state.doc.active_path.and_then(|i| state.doc.paths.get(i)) else {
        return;
    };
    let painter = ui.painter_at(canvas);
    let blue = egui::Color32::from_rgb(45, 150, 255);
    for sub in &path.subpaths {
        let points = sub
            .flatten()
            .into_iter()
            .map(|p| to_screen(state, Pos2::new(p.0, p.1), ppp))
            .collect::<Vec<_>>();
        painter.add(egui::Shape::line(points, Stroke::new(1., blue)));
        for k in &sub.knots {
            let a = to_screen(state, Pos2::new(k.anchor.0, k.anchor.1), ppp);
            for p in [k.incoming, k.outgoing] {
                let b = to_screen(state, Pos2::new(p.0, p.1), ppp);
                if a.distance(b) > 0.5 {
                    painter.line_segment([a, b], Stroke::new(0.75, blue));
                    painter.circle_filled(b, 2.5, blue);
                }
            }
            painter.rect_filled(Rect::from_center_size(a, egui::vec2(5., 5.)), 0., blue);
        }
    }
}

/// Closest point on the active path, with a screen-space hit radius.
fn curve_hit(state: &DocState, pointer: Pos2, ppp: f32) -> Option<(usize, usize, usize, f32)> {
    let path = state.doc.active_path?;
    let geometry = state.doc.paths.get(path)?;
    let distance = |sub: &Subpath, segment: usize, t: f32| {
        let at = op_core::path::cubic(
            &sub.knots[segment],
            &sub.knots[(segment + 1) % sub.knots.len()],
            t,
        );
        to_screen(state, Pos2::new(at.0, at.1), ppp).distance_sq(pointer)
    };
    let mut best = None;
    let mut best_distance = 36.0;
    for (si, sub) in geometry.subpaths.iter().enumerate() {
        for segment in 0..sub.segments() {
            let sample = (0usize..=128)
                .min_by(|&a, &b| {
                    distance(sub, segment, a as f32 / 128.).total_cmp(&distance(
                        sub,
                        segment,
                        b as f32 / 128.,
                    ))
                })
                .unwrap();
            let mut lo = sample.saturating_sub(1) as f32 / 128.;
            let mut hi = (sample + 1).min(128) as f32 / 128.;
            for _ in 0..28 {
                let a = lo + (hi - lo) / 3.;
                let b = hi - (hi - lo) / 3.;
                if distance(sub, segment, a) < distance(sub, segment, b) {
                    hi = b;
                } else {
                    lo = a;
                }
            }
            let t = (lo + hi) / 2.;
            let d = distance(sub, segment, t);
            if d < best_distance && t > 0.0001 && t < 0.9999 {
                best_distance = d;
                best = Some((path, si, segment, t));
            }
        }
    }
    best
}
pub fn add_anchor(ui: &Ui, response: &egui::Response, state: &mut DocState, ppp: f32) {
    if !response.clicked() {
        return;
    }
    let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) else {
        return;
    };
    let Some((path, sub, segment, t)) = curve_hit(state, pointer, ppp) else {
        return;
    };
    if state.doc.paths[path].subpaths[sub]
        .insert(segment, t)
        .is_some()
    {
        state.doc.mark_dirty();
        state.record("Add Anchor Point");
    }
}

fn anchor_hit(state: &DocState, pointer: Pos2, ppp: f32) -> Option<(usize, usize, usize)> {
    let path = state.doc.active_path?;
    let geometry = state.doc.paths.get(path)?;
    let mut nearest = 36.;
    let mut hit = None;
    for (si, sub) in geometry.subpaths.iter().enumerate() {
        for (ki, knot) in sub.knots.iter().enumerate() {
            let d =
                to_screen(state, Pos2::new(knot.anchor.0, knot.anchor.1), ppp).distance_sq(pointer);
            if d < nearest {
                nearest = d;
                hit = Some((path, si, ki));
            }
        }
    }
    hit
}
pub fn delete_anchor(ui: &Ui, response: &egui::Response, state: &mut DocState, ppp: f32) {
    if !response.clicked() {
        return;
    }
    let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) else {
        return;
    };
    let Some((path, sub, knot)) = anchor_hit(state, pointer, ppp) else {
        return;
    };
    let geometry = &mut state.doc.paths[path];
    geometry.subpaths[sub].knots.remove(knot);
    if geometry.subpaths[sub].knots.is_empty() {
        geometry.subpaths.remove(sub);
    }
    if geometry.subpaths.is_empty() {
        state.doc.paths.remove(path);
        state.doc.active_path = None;
    }
    state.pen_session.continuation = None;
    state.doc.mark_dirty();
    state.record("Delete Anchor Point");
}

struct ConvertDrag {
    before: op_core::Snapshot,
    original: Knot,
    path: usize,
    sub: usize,
    knot: usize,
    handle: u8,
    start: Pos2,
    moved: bool,
}
fn handle_hit(state: &DocState, pointer: Pos2, ppp: f32) -> Option<(usize, usize, usize, u8)> {
    let path = state.doc.active_path?;
    let geometry = state.doc.paths.get(path)?;
    let mut nearest = 36.;
    let mut hit = None;
    for (si, sub) in geometry.subpaths.iter().enumerate() {
        for (ki, knot) in sub.knots.iter().enumerate() {
            for (handle, p) in [(1, knot.incoming), (2, knot.outgoing)] {
                if p == knot.anchor {
                    continue;
                }
                let d = to_screen(state, Pos2::new(p.0, p.1), ppp).distance_sq(pointer);
                if d < nearest {
                    nearest = d;
                    hit = Some((path, si, ki, handle));
                }
            }
        }
    }
    hit
}
pub fn cancel_inflight(state: &mut DocState) {
    if let Some(d) = state.pen_session.drag.take() {
        state.doc.restore(&d.before);
    }
    if let Some(d) = state.pen_session.convert.take() {
        state.doc.restore(&d.before);
    }
}
pub fn convert_point(ui: &Ui, response: &egui::Response, state: &mut DocState, ppp: f32) {
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        cancel_inflight(state);
        return;
    }
    if response.hovered()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = ui.input(|i| i.pointer.interact_pos())
    {
        let hit = anchor_hit(state, p, ppp)
            .map(|(a, b, c)| (a, b, c, 0))
            .or_else(|| handle_hit(state, p, ppp));
        if let Some((path, sub, knot, handle)) = hit {
            state.pen_session.convert = Some(ConvertDrag {
                before: state.doc.snapshot(),
                original: state.doc.paths[path].subpaths[sub].knots[knot].clone(),
                path,
                sub,
                knot,
                handle,
                start: p,
                moved: false,
            });
            state.pen_session.continuation = None;
        }
    }
    let Some(d) = state.pen_session.convert.as_ref() else {
        return;
    };
    let (path, sub, knot, handle, start) = (d.path, d.sub, d.knot, d.handle, d.start);
    if let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
        && (pointer.distance(start) > 2. || d.moved)
    {
        let p = to_doc(state, pointer, ppp);
        let k = &mut state.doc.paths[path].subpaths[sub].knots[knot];
        if handle == 0 {
            let (mut dx, mut dy) = (p.x - k.anchor.0, p.y - k.anchor.1);
            if ui.input(|i| i.modifiers.shift) {
                let step = std::f32::consts::FRAC_PI_4;
                let a = (dy.atan2(dx) / step).round() * step;
                let l = dx.hypot(dy);
                dx = l * a.cos();
                dy = l * a.sin();
            }
            k.outgoing = (k.anchor.0 + dx, k.anchor.1 + dy);
            k.incoming = (k.anchor.0 - dx, k.anchor.1 - dy);
            k.linked = true;
        } else {
            if handle == 1 {
                k.incoming = (p.x, p.y);
            } else {
                k.outgoing = (p.x, p.y);
            }
            k.linked = false;
        }
        state.pen_session.convert.as_mut().unwrap().moved = true;
    }
    if !ui.input(|i| i.pointer.primary_down()) {
        let d = state.pen_session.convert.take().unwrap();
        let k = &mut state.doc.paths[path].subpaths[sub].knots[knot];
        if !d.moved && handle == 0 {
            k.incoming = k.anchor;
            k.outgoing = k.anchor;
            k.linked = false;
        }
        if *k != d.original {
            state.doc.mark_dirty();
            state.record("Convert Point");
        }
    }
}
