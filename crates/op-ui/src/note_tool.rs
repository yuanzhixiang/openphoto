//! Note markers and an editable, staged Notes window.
use crate::{
    document_view::{to_doc, to_screen},
    state::DocState,
};
use egui::{Key, Modifiers, Pos2, Response, Ui};
#[derive(Clone, Copy, Debug)]
pub struct Drag {
    id: Option<u32>,
    original: Pos2,
    current: Pos2,
}
pub struct Edit {
    pub id: u32,
    pub text: String,
    pub author: String,
    pub color: [u8; 3],
}
fn open(state: &mut DocState, id: u32) {
    if let Some(n) = state.doc.notes.iter().find(|n| n.id == id) {
        state.note_selected = Some(id);
        state.note_edit = Some(Edit {
            id,
            text: n.text.clone(),
            author: n.author.clone(),
            color: n.color,
        });
    }
}
pub fn cancel(state: &mut DocState) {
    state.note_drag = None;
}
pub fn commit(state: &mut DocState) {
    let Some(edit) = state.note_edit.take() else {
        return;
    };
    if let Some(n) = state.doc.notes.iter_mut().find(|n| n.id == edit.id)
        && (n.text != edit.text || n.author != edit.author || n.color != edit.color)
    {
        n.text = edit.text;
        n.author = edit.author;
        n.color = edit.color;
        state.doc.mark_dirty();
        state.record("Edit Note");
    }
}
pub fn input(
    ui: &mut Ui,
    response: &Response,
    state: &mut DocState,
    author: &str,
    color: [u8; 3],
    ppp: f32,
) {
    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        state.note_drag = None;
        state.note_edit = None;
        return;
    }
    if response.contains_pointer()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = response.interact_pointer_pos()
    {
        let p = to_doc(state, p, ppp);
        if p.x >= 0.0 && p.y >= 0.0 && p.x < state.doc.width as f32 && p.y < state.doc.height as f32
        {
            commit(state);
            let id = state
                .doc
                .notes
                .iter()
                .rev()
                .find(|n| {
                    egui::pos2(n.x as f32, n.y as f32).distance(p) * state.view.zoom / ppp < 10.0
                })
                .map(|n| n.id);
            let original = id
                .and_then(|id| {
                    state
                        .doc
                        .notes
                        .iter()
                        .find(|n| n.id == id)
                        .map(|n| egui::pos2(n.x as f32, n.y as f32))
                })
                .unwrap_or(p);
            state.note_drag = Some(Drag {
                id,
                original,
                current: p,
            });
        }
    }
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        if let Some(d) = &mut state.note_drag {
            d.current = egui::pos2(
                p.x.round()
                    .clamp(0.0, state.doc.width.saturating_sub(1) as f32),
                p.y.round()
                    .clamp(0.0, state.doc.height.saturating_sub(1) as f32),
            );
        }
    }
    if ui.input(|i| i.pointer.primary_released())
        && let Some(d) = state.note_drag.take()
    {
        if let Some(id) = d.id {
            if d.current.distance(d.original) > 0.5
                && let Some(n) = state.doc.notes.iter_mut().find(|n| n.id == id)
            {
                n.x = d.current.x as u32;
                n.y = d.current.y as u32;
                state.doc.mark_dirty();
                state.record("Move Note");
            }
            open(state, id);
        } else if let Some(id) = op_core::annotation::create_note(
            &mut state.doc,
            d.current.x as u32,
            d.current.y as u32,
            author,
        ) {
            if let Some(n) = state.doc.notes.iter_mut().find(|n| n.id == id) {
                n.color = color;
            }
            state.record("New Note");
            open(state, id);
        }
    }
}
pub fn draw(ui: &mut Ui, state: &mut DocState, ppp: f32) {
    for n in &state.doc.notes {
        let p = state
            .note_drag
            .filter(|d| d.id == Some(n.id))
            .map_or(egui::pos2(n.x as f32, n.y as f32), |d| d.current);
        let p = to_screen(state, p, ppp);
        ui.painter().rect_filled(
            egui::Rect::from_center_size(p, egui::vec2(13.0, 16.0)),
            1,
            egui::Color32::from_rgb(n.color[0], n.color[1], n.color[2]),
        );
        for dy in [-4.0, 0.0, 4.0] {
            ui.painter().line_segment(
                [p + egui::vec2(-4.0, dy), p + egui::vec2(4.0, dy)],
                egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
            );
        }
    }
    let Some(edit) = &mut state.note_edit else {
        return;
    };
    let mut opened = true;
    let mut done = false;
    let mut delete = false;
    egui::Window::new("Notes")
        .id(ui.id().with("notes-editor"))
        .default_pos(egui::pos2(
            state.view.viewport.right() - 290.0,
            state.view.viewport.top() + 10.0,
        ))
        .default_width(260.0)
        .open(&mut opened)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Author:");
                ui.text_edit_singleline(&mut edit.author);
            });
            ui.color_edit_button_srgb(&mut edit.color);
            ui.add(
                egui::TextEdit::multiline(&mut edit.text)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY),
            );
            ui.horizontal(|ui| {
                done = ui.button("Done").clicked();
                delete = ui.button("Delete").clicked();
            });
        });
    if delete {
        let id = state.note_edit.take().unwrap().id;
        let count = state.doc.notes.len();
        state.doc.notes.retain(|n| n.id != id);
        if state.doc.notes.len() != count {
            state.doc.mark_dirty();
            state.record("Delete Note");
        }
    } else if done || !opened {
        commit(state);
    }
}

pub fn color(app: &crate::state::AppState) -> [u8; 3] {
    if let Some(edit) = app
        .active_doc
        .and_then(|id| app.docs.get(&id))
        .and_then(|s| s.note_edit.as_ref())
    {
        return edit.color;
    }
    app.tool_settings
        .get("note.color")
        .and_then(|v| {
            let c: Vec<_> = v.split(',').filter_map(|v| v.parse::<u8>().ok()).collect();
            c.try_into().ok()
        })
        .unwrap_or([255; 3])
}
pub fn toggle(state: &mut DocState) {
    if state.note_edit.is_some() {
        commit(state);
    } else if let Some(id) = state
        .note_selected
        .filter(|id| state.doc.notes.iter().any(|n| n.id == *id))
        .or_else(|| state.doc.notes.last().map(|n| n.id))
    {
        open(state, id);
    }
}
