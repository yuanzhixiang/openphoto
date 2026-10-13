//! Slice creation and canvas overlays; no pixel editing.
use crate::document_view::{to_doc, to_screen};
use crate::state::{AppState, DocState};
use egui::{Key, Modifiers, Pos2, Rect, Response, Stroke, Ui};

#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    pub style: usize,
    pub width: f32,
    pub height: f32,
}
impl Options {
    pub fn read(app: &AppState) -> Self {
        let style = app
            .tool_settings
            .get("slice.style")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0)
            .min(2);
        let default = if style == 1 { 1.0 } else { 64.0 };
        let size = |key| {
            let v = app
                .tool_settings
                .get(key)
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(default);
            if v.is_finite() && v > 0.0 { v } else { 1.0 }
        };
        Self {
            style,
            width: size("slice.w"),
            height: size("slice.h"),
        }
    }
    fn rect(self, a: Pos2, b: Pos2) -> Rect {
        if self.style == 2 {
            return Rect::from_min_size(a, egui::vec2(self.width, self.height));
        }
        let mut b = b;
        if self.style == 1 {
            let delta = b - a;
            let ratio = self.width / self.height;
            if ratio.is_finite() && ratio > 0.0 {
                if delta.x.abs() > delta.y.abs() * ratio {
                    b.y = a.y + delta.x.abs() / ratio * delta.y.signum();
                } else {
                    b.x = a.x + delta.y.abs() * ratio * delta.x.signum();
                }
            }
        }
        Rect::from_two_pos(a, b)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Drag {
    pub start: Pos2,
    pub current: Pos2,
}

pub fn input(ui: &mut Ui, response: &Response, state: &mut DocState, options: Options, ppp: f32) {
    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        state.slice_drag = None;
        return;
    }
    if response.contains_pointer()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = response.interact_pointer_pos()
    {
        let p = to_doc(state, p, ppp);
        if p.x >= 0.0 && p.y >= 0.0 && p.x < state.doc.width as f32 && p.y < state.doc.height as f32
        {
            state.slice_drag = Some(Drag {
                start: p,
                current: p,
            });
        }
    }
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        if let Some(d) = &mut state.slice_drag {
            d.current = p;
        }
    }
    if ui.input(|i| i.pointer.primary_released())
        && let Some(d) = state.slice_drag.take()
    {
        let r = options.rect(d.start, d.current);
        let b = (
            r.left().round().max(0.0) as u32,
            r.top().round().max(0.0) as u32,
            r.right().round().max(0.0) as u32,
            r.bottom().round().max(0.0) as u32,
        );
        if let Some(id) = op_core::slice::create(&mut state.doc, b) {
            state.selected_slices = vec![id];
            state.record("New Slice");
        }
    }
}
pub fn draw(ui: &Ui, state: &DocState, options: Options, ppp: f32) {
    let painter = ui.painter();
    let outline = |r: Rect, selected: bool| {
        let r = Rect::from_two_pos(to_screen(state, r.min, ppp), to_screen(state, r.max, ppp));
        painter.rect_stroke(
            r,
            0,
            Stroke::new(
                if selected { 2.0 } else { 1.0 },
                egui::Color32::from_rgb(70, 150, 230),
            ),
            egui::StrokeKind::Inside,
        );
        r
    };
    for (i, s) in state.doc.slices.iter().enumerate() {
        let (x0, y0, x1, y1) = s.bounds;
        let r = outline(
            Rect::from_min_max(
                egui::pos2(x0 as f32, y0 as f32),
                egui::pos2(x1 as f32, y1 as f32),
            ),
            state.selected_slices.contains(&s.id),
        );
        if state.selected_slices.contains(&s.id) {
            for p in [
                r.min,
                egui::pos2(r.center().x, r.top()),
                r.right_top(),
                egui::pos2(r.right(), r.center().y),
                r.max,
                egui::pos2(r.center().x, r.bottom()),
                r.left_bottom(),
                egui::pos2(r.left(), r.center().y),
            ] {
                painter.rect_filled(
                    Rect::from_center_size(p, egui::vec2(5.0, 5.0)),
                    0,
                    egui::Color32::WHITE,
                );
            }
        }
        painter.text(
            r.min,
            egui::Align2::LEFT_TOP,
            format!("{:02}", i + 1),
            egui::FontId::monospace(11.0),
            egui::Color32::WHITE,
        );
    }
    if let Some(d) = state.slice_drag {
        outline(options.rect(d.start, d.current), true);
    }
}

pub struct SelectDrag {
    before: op_core::Snapshot,
    slices: Vec<op_core::slice::Slice>,
    start: Pos2,
    handle: Option<(u64, i8, i8)>,
}
pub fn cancel_select(state: &mut DocState) {
    if let Some(d) = state.slice_select_drag.take() {
        state.doc.restore(&d.before);
    }
}
fn handle(bounds: (u32, u32, u32, u32), p: Pos2, scale: f32) -> Option<(i8, i8)> {
    let (x0, y0, x1, y1) = bounds;
    let rect = Rect::from_min_max(
        egui::pos2(x0 as f32, y0 as f32),
        egui::pos2(x1 as f32, y1 as f32),
    );
    for (hx, hy) in [
        (-1, -1),
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
    ] {
        let q = rect.center()
            + egui::vec2(
                hx as f32 * rect.width() / 2.0,
                hy as f32 * rect.height() / 2.0,
            );
        if q.distance(p) * scale <= 8.0 {
            return Some((hx, hy));
        }
    }
    None
}
pub fn select_input(ui: &mut Ui, response: &Response, state: &mut DocState, ppp: f32) {
    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        cancel_select(state);
        return;
    }
    if state.slice_select_drag.is_none()
        && ui.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Delete)
                || i.consume_key(Modifiers::NONE, Key::Backspace)
        })
    {
        let count = state.doc.slices.len();
        state
            .doc
            .slices
            .retain(|s| !state.selected_slices.contains(&s.id));
        if state.doc.slices.len() != count {
            state.doc.mark_dirty();
            state.selected_slices.clear();
            state.record("Delete Slice");
        }
    }
    if response.contains_pointer()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = response.interact_pointer_pos()
    {
        let p = to_doc(state, p, ppp);
        let hit = state.doc.slices.iter().rev().find_map(|s| {
            let h = state
                .selected_slices
                .contains(&s.id)
                .then(|| handle(s.bounds, p, state.view.zoom / ppp))
                .flatten();
            let (x0, y0, x1, y1) = s.bounds;
            let inside =
                p.x >= x0 as f32 && p.x <= x1 as f32 && p.y >= y0 as f32 && p.y <= y1 as f32;
            (h.is_some() || inside).then_some((s.id, h))
        });
        if let Some((id, h)) = hit {
            let shift = ui.input(|i| i.modifiers.shift);
            if shift && state.selected_slices.contains(&id) && h.is_none() {
                state.selected_slices.retain(|&s| s != id);
            } else {
                if !state.selected_slices.contains(&id) {
                    if !shift {
                        state.selected_slices.clear();
                    }
                    state.selected_slices.push(id);
                }
                state.slice_select_drag = Some(SelectDrag {
                    before: state.doc.snapshot(),
                    slices: state.doc.slices.clone(),
                    start: p,
                    handle: h.map(|(x, y)| (id, x, y)),
                });
            }
        } else if !ui.input(|i| i.modifiers.shift) {
            state.selected_slices.clear();
        }
    }
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        if let Some(d) = &state.slice_select_drag {
            let delta = p - d.start;
            let dx = delta.x.round() as i64;
            let dy = delta.y.round() as i64;
            let mut slices = d.slices.clone();
            if let Some((id, hx, hy)) = d.handle {
                if let Some(s) = slices.iter_mut().find(|s| s.id == id) {
                    let (mut x0, mut y0, mut x1, mut y1) = s.bounds;
                    if hx < 0 {
                        x0 = (x0 as i64 + dx).clamp(0, x1 as i64 - 1) as u32;
                    }
                    if hx > 0 {
                        x1 = (x1 as i64 + dx).clamp(x0 as i64 + 1, state.doc.width as i64) as u32;
                    }
                    if hy < 0 {
                        y0 = (y0 as i64 + dy).clamp(0, y1 as i64 - 1) as u32;
                    }
                    if hy > 0 {
                        y1 = (y1 as i64 + dy).clamp(y0 as i64 + 1, state.doc.height as i64) as u32;
                    }
                    s.bounds = (x0, y0, x1, y1);
                }
            } else {
                let chosen: Vec<_> = slices
                    .iter()
                    .filter(|s| state.selected_slices.contains(&s.id))
                    .collect();
                let lo_x = chosen.iter().map(|s| s.bounds.0).min().unwrap_or(0) as i64;
                let lo_y = chosen.iter().map(|s| s.bounds.1).min().unwrap_or(0) as i64;
                let hi_x = chosen.iter().map(|s| s.bounds.2).max().unwrap_or(0) as i64;
                let hi_y = chosen.iter().map(|s| s.bounds.3).max().unwrap_or(0) as i64;
                let dx = dx.clamp(-lo_x, (state.doc.width as i64 - hi_x).max(-lo_x));
                let dy = dy.clamp(-lo_y, (state.doc.height as i64 - hi_y).max(-lo_y));
                for s in &mut slices {
                    if state.selected_slices.contains(&s.id) {
                        let (x0, y0, x1, y1) = s.bounds;
                        s.bounds = (
                            (x0 as i64 + dx) as u32,
                            (y0 as i64 + dy) as u32,
                            (x1 as i64 + dx) as u32,
                            (y1 as i64 + dy) as u32,
                        );
                    }
                }
            }
            if state.doc.slices != slices {
                state.doc.slices = slices;
                state.doc.mark_dirty();
            }
        }
    }
    if ui.input(|i| i.pointer.primary_released())
        && let Some(d) = state.slice_select_drag.take()
        && state.doc.slices != d.slices
    {
        state.record(if d.handle.is_some() {
            "Resize Slice"
        } else {
            "Move Slice"
        });
    }
}
