//! Ruler measurements are view metadata, independent of pixel/history edits.
use crate::{
    document_view::{to_doc, to_screen},
    state::DocState,
};
use egui::{Key, Modifiers, Pos2, Response, Ui};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    pub a: Pos2,
    pub b: Pos2,
    pub c: Option<Pos2>,
}
impl Measurement {
    pub fn readout(self, label: &str) -> Option<String> {
        let v = self.b - self.a;
        let angle = if let Some(c) = self.c {
            let d = c - self.a;
            let a = (d.y.atan2(d.x) - v.y.atan2(v.x))
                .to_degrees()
                .rem_euclid(360.0);
            a.min(360.0 - a)
        } else {
            v.y.atan2(v.x).to_degrees()
        };
        Some(match label {
            "X: 0.00" => format!("X: {:.2}", self.a.x),
            "Y: 0.00" => format!("Y: {:.2}", self.a.y),
            "W: 0.00" => format!("W: {:.2}", v.x),
            "H: 0.00" => format!("H: {:.2}", v.y),
            "A: 0.0°" => format!("A: {angle:.1}°"),
            "L1: 0.00" => format!("L1: {:.2}", v.length()),
            "L2:" => self.c.map_or_else(
                || "L2:".into(),
                |c| format!("L2: {:.2}", c.distance(self.a)),
            ),
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Drag {
    before: Option<Measurement>,
    start: Pos2,
    base: Measurement,
    kind: u8,
}
pub fn cancel(state: &mut DocState) {
    if let Some(d) = state.measure_drag.take() {
        state.measurement = d.before;
    }
}
fn line_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let v = b - a;
    let t = if v.length_sq() > 0.0 {
        ((p - a).dot(v) / v.length_sq()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.distance(a + v * t)
}
pub fn input(ui: &mut Ui, response: &Response, state: &mut DocState, ppp: f32) {
    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        cancel(state);
        return;
    }
    if response.double_clicked() {
        state.measurement = None;
        state.measure_drag = None;
        return;
    }
    if response.contains_pointer()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = response.interact_pointer_pos()
    {
        let p = to_doc(state, p, ppp);
        let before = state.measurement;
        let mut base = Measurement {
            a: p,
            b: p,
            c: None,
        };
        let mut kind = 1;
        if let Some(m) = before {
            let radius = 8.0 * ppp / state.view.zoom;
            if ui.input(|i| i.modifiers.alt)
                && (p.distance(m.a) <= radius || p.distance(m.b) <= radius)
            {
                base = if p.distance(m.b) < p.distance(m.a) {
                    Measurement {
                        a: m.b,
                        b: m.a,
                        c: Some(p),
                    }
                } else {
                    Measurement { c: Some(p), ..m }
                };
                kind = 2;
            } else if p.distance(m.a) <= radius {
                base = m;
                kind = 0;
            } else if p.distance(m.b) <= radius {
                base = m;
                kind = 1;
            } else if m.c.is_some_and(|c| p.distance(c) <= radius) {
                base = m;
                kind = 2;
            } else if line_distance(p, m.a, m.b) <= radius
                || m.c.is_some_and(|c| line_distance(p, m.a, c) <= radius)
            {
                base = m;
                kind = 3;
            }
        }
        state.measure_drag = Some(Drag {
            before,
            start: p,
            base,
            kind,
        });
        state.measurement = Some(base);
    }
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        if let Some(d) = state.measure_drag {
            let mut m = d.base;
            if d.kind == 3 {
                let delta = p - d.start;
                m.a += delta;
                m.b += delta;
                m.c = m.c.map(|c| c + delta);
            } else {
                let anchor = if d.kind == 0 { m.b } else { m.a };
                let mut p = p;
                if ui.input(|i| i.modifiers.shift) {
                    let (dx, dy) =
                        op_core::move_tool::constrain_offset(p.x - anchor.x, p.y - anchor.y);
                    p = anchor + egui::vec2(dx, dy);
                }
                match d.kind {
                    0 => m.a = p,
                    1 => m.b = p,
                    _ => m.c = Some(p),
                }
            }
            state.measurement = Some(m);
        }
    }
    if ui.input(|i| i.pointer.primary_released())
        && state.measure_drag.take().is_some()
        && state.measurement.is_some_and(|m| m.a.distance(m.b) < 0.5)
    {
        state.measurement = None;
    }
}
pub fn draw(ui: &Ui, state: &DocState, ppp: f32) {
    let Some(m) = state.measurement else {
        return;
    };
    let a = to_screen(state, m.a, ppp);
    let b = to_screen(state, m.b, ppp);
    let mut lines = vec![(a, b)];
    if let Some(c) = m.c {
        lines.push((a, to_screen(state, c, ppp)));
    }
    for (a, b) in lines {
        ui.painter()
            .line_segment([a, b], egui::Stroke::new(2.0, egui::Color32::BLACK));
        ui.painter()
            .line_segment([a, b], egui::Stroke::new(1.0, egui::Color32::WHITE));
        for p in [a, b] {
            ui.painter().line_segment(
                [p - egui::vec2(5.0, 0.0), p + egui::vec2(5.0, 0.0)],
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
            ui.painter().line_segment(
                [p - egui::vec2(0.0, 5.0), p + egui::vec2(0.0, 5.0)],
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
        }
    }
}
pub fn straighten(app: &mut crate::state::AppState) {
    let bg = app.background.to_rgba8();
    let Some(state) = app.active() else {
        return;
    };
    let Some(m) = state.measurement else {
        return;
    };
    let v = m.b - m.a;
    if v.length() < 0.5 {
        return;
    }
    let mut angle = v.y.atan2(v.x);
    while angle > std::f32::consts::FRAC_PI_4 {
        angle -= std::f32::consts::FRAC_PI_2;
    }
    while angle < -std::f32::consts::FRAC_PI_4 {
        angle += std::f32::consts::FRAC_PI_2;
    }
    if angle.abs() < 1e-6 {
        return;
    }
    let Ok((x0, y0, x1, y1)) = op_core::transform::bounds(&state.doc) else {
        return;
    };
    let map = op_core::transform::Affine::around(
        ((x0 + x1) / 2.0, (y0 + y1) / 2.0),
        1.0,
        1.0,
        -angle,
        (0.0, 0.0),
    );
    if op_core::transform::transform(&mut state.doc, map, [bg[0], bg[1], bg[2]]).is_ok() {
        state.record("Straighten Layer");
        state.measurement = None;
    }
}
