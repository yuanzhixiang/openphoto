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
    pub fn read(app: &mut AppState) -> Self {
        let style = app
            .setting("slice.style", "0")
            .parse::<usize>()
            .unwrap_or(0)
            .min(2);
        let default = if style == 1 { "1" } else { "64" };
        let mut size = |key| {
            let v = app.setting(key, default).parse::<f32>().unwrap_or(1.0);
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
