//! The Horizontal Type tool (T) on the canvas.
//!
//! A click places the insertion point (the first line's baseline); typed
//! text shows on the document right away as a new layer in the foreground
//! color. Enter starts a new line, Backspace deletes, Escape cancels, and
//! Cmd+Enter (or a click elsewhere on the canvas) commits, as in Photoshop.

use egui::{Color32, Key, Modifiers, Pos2, Rect, Stroke, Ui};

use crate::document_view::{to_doc, to_screen};
use crate::state::{DocState, TextEdit, TypeOptions};

/// Text size in document pixels: points at the document's resolution.
fn size_px(state: &DocState, options: TypeOptions) -> f32 {
    options.size_pt * state.doc.resolution / 72.0
}

/// Handles the Type tool's input. Returns true when text was committed.
pub fn input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    mode: (bool, bool),
    options: TypeOptions,
    color: [u8; 3],
    ppp: f32,
) -> bool {
    let (mask, vertical) = mode;
    let mut committed = false;
    if state.text_edit.as_ref().is_some_and(|e| e.mask != mask) {
        committed = commit(state);
    }
    if let Some(edit) = &mut state.text_edit
        && edit.vertical != vertical
    {
        edit.vertical = vertical;
        edit.shown_options = None;
    }
    if response.clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        if state.text_edit.is_some() {
            // A click elsewhere ends the text being typed
            committed = commit(state);
        } else {
            state.text_edit = Some(TextEdit {
                origin: to_doc(state, p, ppp),
                text: String::new(),
                before: state.doc.snapshot(),
                shown: String::new(),
                mask,
                vertical,
                selection_op: {
                    let m = ui.input(|i| i.modifiers);
                    match (m.shift, m.alt) {
                        (true, true) => op_core::SelectionOp::Intersect,
                        (true, false) => op_core::SelectionOp::Add,
                        (false, true) => op_core::SelectionOp::Subtract,
                        _ => op_core::SelectionOp::Replace,
                    }
                },
                changed: false,
                shown_options: None,
            });
        }
    }
    if state.text_edit.is_none() {
        return committed;
    }

    // Keys typed into a text field (such as the size in the options bar)
    // are not for the text on the canvas
    if ui.ctx().egui_wants_keyboard_input() {
        preview(state, options, color);
        return committed;
    }
    // Typed characters and editing keys go into the text
    let (typed, backspace, newline, escape, commit_key) = ui.input_mut(|i| {
        let mut typed = String::new();
        i.events.retain(|e| match e {
            egui::Event::Text(t) => {
                typed.push_str(t);
                false
            }
            _ => true,
        });
        (
            typed,
            i.consume_key(Modifiers::NONE, Key::Backspace),
            i.consume_key(Modifiers::NONE, Key::Enter),
            i.consume_key(Modifiers::NONE, Key::Escape),
            i.consume_key(Modifiers::COMMAND, Key::Enter),
        )
    });
    if escape {
        cancel(state);
        return committed;
    }
    if commit_key {
        return commit(state) || committed;
    }
    if let Some(edit) = &mut state.text_edit {
        edit.text.push_str(&typed);
        if backspace {
            edit.text.pop();
        }
        if newline {
            edit.text.push('\n');
        }
    }
    preview(state, options, color);
    committed
}

/// Shows the current text on the document as a layer.
pub fn preview(state: &mut DocState, options: TypeOptions, color: [u8; 3]) {
    let size = size_px(state, options);
    let Some(edit) = &state.text_edit else {
        return;
    };
    if edit.text == edit.shown && edit.shown_options == Some((options, color)) {
        return;
    }
    let (text, origin, mask, op, vertical) = (
        edit.text.clone(),
        edit.origin,
        edit.mask,
        edit.selection_op,
        edit.vertical,
    );
    state.doc.restore(&edit.before);
    let changed = if mask {
        let previous = state.doc.selection().cloned();
        match op_core::text::coverage_oriented(
            options.font(),
            &text,
            size,
            (origin.x, origin.y),
            state.doc.width,
            state.doc.height,
            vertical,
        ) {
            Ok(coverage) => {
                let shape = op_core::Selection::from_mask(
                    state.doc.width,
                    state.doc.height,
                    coverage,
                    false,
                );
                let selection = op_core::Selection::combine(previous.as_ref(), shape, op);
                state.doc.set_selection(Some(selection));
                state.doc.selection() != previous.as_ref()
            }
            Err(_) => false,
        }
    } else {
        op_core::text::add_text_layer_oriented(
            &mut state.doc,
            options.font(),
            &text,
            size,
            (origin.x, origin.y),
            color,
            vertical,
        )
        .is_ok_and(|id| id.is_some())
    };
    if let Some(edit) = &mut state.text_edit {
        edit.shown = text;
        edit.shown_options = Some((options, color));
        edit.changed = changed;
    }
}

/// Keeps the text ("Type Tool" in the history). Empty text leaves nothing.
pub fn commit(state: &mut DocState) -> bool {
    let Some(edit) = state.text_edit.take() else {
        return false;
    };
    if edit.shown.trim().is_empty() || !edit.changed {
        state.doc.restore(&edit.before);
        return false;
    }
    state.record(if edit.mask { "Type Mask" } else { "Type Tool" });
    true
}

/// Drops the text being typed.
pub fn cancel(state: &mut DocState) {
    if let Some(edit) = state.text_edit.take() {
        state.doc.restore(&edit.before);
    }
}

/// Red editing overlay; the document pixels stay untouched.
pub fn draw_mask_overlay(ui: &Ui, state: &DocState, canvas: Rect, ppp: f32) {
    if !state.text_edit.as_ref().is_some_and(|e| e.mask) {
        return;
    }
    let painter = ui.painter_at(canvas);
    for y in 0..state.doc.height {
        let value = |x| 255 - state.doc.selection().map_or(0, |s| s.get(x, y));
        let mut x = 0;
        while x < state.doc.width {
            let v = value(x);
            let start = x;
            x += 1;
            while x < state.doc.width && value(x) == v {
                x += 1;
            }
            if v == 0 {
                continue;
            }
            let points = [(start, y), (x, y), (x, y + 1), (start, y + 1)]
                .map(|(a, b)| to_screen(state, Pos2::new(a as f32, b as f32), ppp));
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                Color32::from_rgba_unmultiplied(255, 0, 0, v / 2),
                Stroke::NONE,
            ));
        }
    }
}

/// The blinking caret after the last character.
pub fn draw_caret(ui: &Ui, state: &DocState, options: TypeOptions, canvas: Rect, ppp: f32) {
    let Some(edit) = &state.text_edit else {
        return;
    };
    let size = size_px(state, options);
    let Some(((x, y), line)) = op_core::text::caret_oriented(
        options.font(),
        &edit.text,
        size,
        (edit.origin.x, edit.origin.y),
        edit.vertical,
    ) else {
        return;
    };
    // Blink every half second
    let time = ui.input(|i| i.time);
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(500));
    if (time * 2.0) as i64 % 2 == 1 {
        return;
    }
    let (a, b) = if edit.vertical {
        (Pos2::new(x - line / 2., y), Pos2::new(x + line / 2., y))
    } else {
        (Pos2::new(x, y - line * 0.75), Pos2::new(x, y + line * 0.2))
    };
    let top = to_screen(state, a, ppp);
    let bottom = to_screen(state, b, ppp);
    // Black with a white core, visible on any color
    let painter = ui.painter_at(canvas);
    painter.line_segment([top, bottom], Stroke::new(3.0, Color32::BLACK));
    painter.line_segment([top, bottom], Stroke::new(1.0, Color32::WHITE));
}
