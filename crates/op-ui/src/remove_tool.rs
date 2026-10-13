//! Remove canvas session; offline synthesis is not native Photoshop parity.
use crate::{
    document_view::{to_doc, to_screen},
    state::{AppState, DocState},
};
use egui::{Key, Pos2, Ui};
use op_core::{Selection, paint::BrushTip, selection_brush::SelectionStroke};

#[derive(Clone, Copy)]
pub struct Options {
    pub size: f32,
    pub subtract: bool,
    pub pressure: Option<f32>,
    pub after: bool,
    pub all: bool,
    pub new_layer: bool,
    pub ai: usize,
}
pub fn options(app: &mut AppState) -> Options {
    let size = app
        .setting("remove.size", "50")
        .parse::<f32>()
        .ok()
        .filter(|n| n.is_finite())
        .unwrap_or(50.)
        .clamp(1., 5000.);
    Options {
        size,
        subtract: app.setting("remove.mode", "0") == "1",
        pressure: if app.flag("remove.pressure", true) {
            app.pen_pressure
        } else {
            None
        },
        after: app.flag("remove.after_stroke", true),
        all: app.flag("remove.all_layers", false),
        new_layer: app.flag("remove.new_layer", false),
        ai: app
            .setting("remove.ai_mode", "0")
            .parse::<usize>()
            .unwrap_or(0)
            .min(2),
    }
}
#[derive(Default)]
pub struct Session {
    pub area: Option<Selection>,
    stroke: Option<SelectionStroke>,
    before: Option<Selection>,
    apply_requested: bool,
    latched: Option<Options>,
    undo_marks: Vec<Option<Selection>>,
    redo_marks: Vec<Option<Selection>>,
}
impl Session {
    pub fn pending(&self) -> bool {
        self.stroke.is_some() || self.area.is_some()
    }
    pub fn can_undo(&self) -> bool {
        self.stroke.is_some() || !self.undo_marks.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        self.stroke.is_none() && !self.redo_marks.is_empty()
    }
}
pub fn undo_marks(state: &mut DocState) -> bool {
    let Some(s) = &mut state.remove_session else {
        return false;
    };
    if s.stroke.take().is_some() {
        s.area = s.before.take();
        s.latched = None;
        return true;
    }
    let Some(before) = s.undo_marks.pop() else {
        return false;
    };
    s.redo_marks.push(s.area.take());
    s.area = before;
    true
}
pub fn redo_marks(state: &mut DocState) -> bool {
    let Some(s) = &mut state.remove_session else {
        return false;
    };
    if s.stroke.is_some() {
        return false;
    }
    let Some(after) = s.redo_marks.pop() else {
        return false;
    };
    s.undo_marks.push(s.area.take());
    s.area = after;
    true
}
pub fn request_apply(state: &mut DocState) {
    if let Some(s) = &mut state.remove_session {
        s.apply_requested = true;
    }
}
pub fn reset(state: &mut DocState) {
    state.remove_session = None;
}
fn apply(state: &mut DocState, opts: Options) -> Option<String> {
    let area = state.remove_session.as_ref()?.area.as_ref()?.clone();
    if opts.ai == 1 {
        return Some("Generative AI removal requires a configured native/model backend; no replacement model is used.".into());
    }
    match op_core::remove::apply(&mut state.doc, &area, opts.all, opts.new_layer) {
        Ok(changed) => {
            state.remove_session = None;
            if changed {
                state.record("Remove Tool");
            }
            None
        }
        Err(e) => Some(e.message().into()),
    }
}
pub fn input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    opts: Options,
    ppp: f32,
) -> Option<String> {
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        if let Some(s) = &mut state.remove_session {
            if s.stroke.take().is_some() {
                s.area = s.before.take();
                s.latched = None;
            } else {
                state.remove_session = None;
            }
        }
        return None;
    }
    let point = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    if response.contains_pointer() && ui.input(|i| i.pointer.primary_pressed()) {
        if (state.doc.width as usize).saturating_mul(state.doc.height as usize) > 16_777_216 {
            return Some("The document exceeds the offline marking limit.".into());
        }
        let s = state.remove_session.get_or_insert_with(Session::default);
        s.before = s.area.clone();
        s.latched = Some(opts);
        s.stroke = Some(SelectionStroke::new(
            state.doc.width,
            state.doc.height,
            s.area.as_ref(),
            BrushTip {
                diameter: opts.size,
                hardness: 1.,
                aliased: false,
                square: false,
                angle: 0.,
                roundness: 1.,
                spacing: 0.25,
            },
            1.,
            opts.subtract != ui.input(|i| i.modifiers.alt),
        ));
    }
    if let Some(s) = &mut state.remove_session
        && let Some(stroke) = &mut s.stroke
    {
        if let Some(p) = point {
            stroke.add_point_with_pressure(p.x, p.y, opts.pressure.unwrap_or(1.));
        }
        s.area = stroke.selection();
        if !ui.input(|i| i.pointer.primary_down()) {
            s.stroke = None;
            let before = s.before.take();
            if s.area != before {
                s.undo_marks.push(before);
                s.redo_marks.clear();
                let cap = (16_777_216
                    / ((state.doc.width as usize) * (state.doc.height as usize)).max(1))
                .clamp(1, 32);
                if s.undo_marks.len() > cap {
                    s.undo_marks.remove(0);
                }
            }
            let latched = s.latched.take().unwrap_or(opts);
            if latched.after {
                return apply(state, latched);
            }
        } else {
            ui.ctx().request_repaint();
        }
    }
    let commit = ui.input(|i| i.key_pressed(Key::Enter))
        || state
            .remove_session
            .as_ref()
            .is_some_and(|s| s.apply_requested);
    if commit {
        if let Some(s) = &mut state.remove_session {
            s.apply_requested = false;
        }
        return apply(state, opts);
    }
    // Pointer-up after an empty subtract stroke leaves no pending session.
    if state
        .remove_session
        .as_ref()
        .is_some_and(|s| s.stroke.is_none() && s.area.is_none() && s.redo_marks.is_empty())
    {
        state.remove_session = None;
    }
    None
}
pub fn draw(ui: &Ui, state: &DocState, opts: Options, ppp: f32) {
    if let Some(p) = state.pointer {
        let center = to_screen(state, p, ppp);
        let radius = opts.size
            * opts
                .pressure
                .unwrap_or(1.)
                .clamp(0., 1.)
                .max(1. / opts.size)
            * state.view.zoom
            / ppp
            / 2.;
        ui.painter()
            .circle_stroke(center, radius, egui::Stroke::new(2., egui::Color32::BLACK));
        ui.painter()
            .circle_stroke(center, radius, egui::Stroke::new(1., egui::Color32::WHITE));
    }
    let Some(area) = state.remove_session.as_ref().and_then(|s| s.area.as_ref()) else {
        return;
    };
    let Some((x0, y0, x1, y1)) = area.bounds() else {
        return;
    };
    for y in y0..y1 {
        let mut x = x0;
        while x < x1 {
            let v = area.get(x, y);
            if v == 0 {
                x += 1;
                continue;
            }
            let start = x;
            x += 1;
            while x < x1 && area.get(x, y) == v {
                x += 1;
            }
            let points = [(start, y), (x, y), (x, y + 1), (start, y + 1)]
                .map(|(a, b)| to_screen(state, Pos2::new(a as f32, b as f32), ppp));
            ui.painter().add(egui::Shape::convex_polygon(
                points.to_vec(),
                egui::Color32::from_rgba_unmultiplied(229, 97, 171, v / 2),
                egui::Stroke::NONE,
            ));
        }
    }
}
