//! File operations and global shortcuts.

use std::path::PathBuf;

use egui::Key;
use op_core::{Color, DocId, Document};

use crate::commands::Command;
use crate::state::AppState;

pub fn open_paths(app: &mut AppState, paths: Vec<PathBuf>) {
    for path in paths {
        match op_io::open(&path) {
            Ok(doc) => {
                let id = doc.id;
                app.add_document(doc, "Open");
                if let Some(state) = app.docs.get_mut(&id) {
                    state.path = Some(path);
                }
            }
            Err(e) => {
                log::error!("{}: {e}", path.display());
                app.alert = Some(format!("Could not open “{}”: {e}", path.display()));
            }
        }
    }
}

pub fn open_dialog(app: &mut AppState) {
    let paths = rfd::FileDialog::new()
        .add_filter("Images", op_io::OPEN_EXTENSIONS)
        .pick_files();
    if let Some(paths) = paths {
        open_paths(app, paths);
    }
}

/// Edit > Cut, Copy, Copy Merged, Paste and Paste in Place.
pub fn clipboard(command: Command, app: &mut AppState, ppp: f32) {
    if app.typing {
        // A text field has focus: the menu item acts on its text
        let event = match command {
            Command::Cut => Some(egui::Event::Cut),
            Command::Copy | Command::CopyMerged => Some(egui::Event::Copy),
            _ => app.clipboard.text().map(egui::Event::Paste),
        };
        app.forward_events.extend(event);
        return;
    }
    let [r, g, b, _] = app.background.to_rgba8();
    let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id)) else {
        return;
    };
    let (name, result) = match command {
        Command::Cut => ("Cut", op_core::clipboard::cut(&mut state.doc, [r, g, b])),
        Command::Copy => ("Copy", op_core::clipboard::copy(&state.doc)),
        Command::CopyMerged => ("Copy Merged", op_core::clipboard::copy_merged(&state.doc)),
        _ => {
            let Some(clip) = app.clipboard.get() else {
                return;
            };
            let (w, h) = (state.doc.width, state.doc.height);
            let visible = crate::document_view::visible_rect(state, ppp);
            let into = matches!(command, Command::PasteInto | Command::PasteOutside);
            // Paste Into / Outside: centered on the selection, masked by it
            let at = match state
                .doc
                .selection()
                .and_then(|s| s.bounds())
                .filter(|_| into)
            {
                Some((x0, y0, x1, y1)) => (
                    ((x0 + x1) as i64 - clip.width as i64) / 2,
                    ((y0 + y1) as i64 - clip.height as i64) / 2,
                ),
                None => {
                    let in_place = command == Command::PasteInPlace;
                    op_core::clipboard::placement(&clip, w, h, visible, in_place)
                }
            };
            let selection = state.doc.selection().cloned();
            op_core::clipboard::paste(&mut state.doc, &clip, at);
            if into && let Some(s) = selection {
                // The mask comes from the selection the paste replaced
                state.doc.set_selection(Some(s));
                let kind = if command == Command::PasteInto {
                    op_core::layer_ops::NewMask::RevealSelection
                } else {
                    op_core::layer_ops::NewMask::HideSelection
                };
                op_core::layer_ops::add_mask(&mut state.doc, kind);
                state.doc.set_selection(None);
                state.record(if command == Command::PasteInto {
                    "Paste Into"
                } else {
                    "Paste Outside"
                });
                return;
            }
            state.record("Paste");
            return;
        }
    };
    match result {
        Ok(clip) => {
            if command == Command::Cut {
                state.record("Cut");
            }
            app.clipboard.set(clip);
        }
        Err(e) => app.alert = Some(e.message(name)),
    }
}

/// File > New...: opens the dialog, named after the next untitled document
/// and sized like the clipboard's image if there is one.
pub fn new_dialog(app: &mut AppState) {
    let name = format!("Untitled-{}", app.untitled_counter + 1);
    let size = app.clipboard.get().map(|c| (c.width, c.height));
    let [r, g, b, _] = app.background.to_rgba8();
    app.new_document_dialog = Some(
        crate::dialogs::NewDocumentDialog::new(
            name,
            size,
            app.new_document_recent.clone(),
            app.new_document_saved.clone(),
            app.new_document_welcome_closed,
        )
        .with_background(egui::Color32::from_rgb(r, g, b)),
    );
}

/// Creates the document the New dialog describes. A transparent background
/// gives a regular "Layer 1" instead of a background layer, as in Photoshop.
pub fn create_document(
    app: &mut AppState,
    name: String,
    (width, height): (u32, u32),
    resolution: f32,
    contents: crate::dialogs::NewContents,
) {
    use crate::dialogs::NewContents;
    app.untitled_counter += 1;
    let fill = match contents {
        NewContents::White => Color::WHITE,
        NewContents::Black => Color::BLACK,
        NewContents::BackgroundColor => app.background,
        NewContents::Transparent => Color::from_rgba8([0; 4]),
    };
    let mut doc = Document::new_with_background(name, width, height, fill);
    if contents == NewContents::Transparent {
        let layer = &mut doc.layers[0];
        layer.is_background = false;
        layer.name = "Layer 1".into();
    }
    doc.resolution = resolution;
    app.add_document(doc, "New");
}

/// The document OpenPhoto starts with when no file is given.
pub fn new_document(app: &mut AppState) {
    app.untitled_counter += 1;
    let doc = Document::new_with_background(
        format!("Untitled-{}", app.untitled_counter),
        1920,
        1080,
        Color::WHITE,
    );
    app.add_document(doc, "New");
}

pub fn export_dialog(app: &mut AppState) {
    let Some(state) = app.active() else {
        return;
    };
    let stem = state
        .doc
        .title
        .rsplit_once('.')
        .map_or(state.doc.title.as_str(), |(s, _)| s)
        .to_owned();
    let path = rfd::FileDialog::new()
        .set_file_name(format!("{stem}.png"))
        .add_filter("PNG", &["png"])
        .add_filter("JPEG", &["jpg", "jpeg"])
        .save_file();
    if let Some(path) = path
        && let Err(e) = op_io::export_composite(&state.doc, &path)
    {
        app.alert = Some(format!("Could not export: {e}"));
    }
}

/// Whether File > Save can write the document back to its own file: a
/// Photoshop document always can; a flat format only holds one layer.
fn saves_in_place(state: &crate::state::DocState) -> bool {
    let Some(path) = &state.path else {
        return false;
    };
    let psd = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("psd"));
    psd || state.doc.layers.len() == 1
}

fn file_stem(title: &str) -> &str {
    title.rsplit_once('.').map_or(title, |(s, _)| s)
}

/// File > Save. Documents without a file, or whose layers their file can't
/// hold, go through Save As. Returns whether the document was saved.
pub fn save(app: &mut AppState, id: DocId) -> bool {
    let Some(state) = app.docs.get_mut(&id) else {
        return false;
    };
    if !saves_in_place(state) {
        return save_as(app, id, false);
    }
    let path = state.path.clone().expect("checked");
    match op_io::save(&state.doc, &path) {
        Ok(()) => {
            state.mark_saved();
            true
        }
        Err(e) => {
            app.alert = Some(format!("Could not save “{}”: {e}", path.display()));
            false
        }
    }
}

/// File > Save As... and Save a Copy.... The document takes the new file
/// as its own unless `copy`, or unless the chosen format can't hold its
/// layers (then the file is a flattened copy, as Photoshop's "As a Copy").
pub fn save_as(app: &mut AppState, id: DocId, copy: bool) -> bool {
    let Some(state) = app.docs.get_mut(&id) else {
        return false;
    };
    let mut dialog =
        rfd::FileDialog::new().set_file_name(format!("{}.psd", file_stem(&state.doc.title)));
    if let Some(dir) = state.path.as_ref().and_then(|p| p.parent()) {
        dialog = dialog.set_directory(dir);
    }
    for (name, extensions) in op_io::SAVE_FORMATS {
        dialog = dialog.add_filter(*name, extensions);
    }
    let Some(path) = dialog.save_file() else {
        return false;
    };
    save_to(app, id, path, copy)
}

/// Writes the document to `path` and updates its file and saved state as
/// [`save_as`] describes.
pub fn save_to(app: &mut AppState, id: DocId, path: PathBuf, copy: bool) -> bool {
    let Some(state) = app.docs.get_mut(&id) else {
        return false;
    };
    if let Err(e) = op_io::save(&state.doc, &path) {
        app.alert = Some(format!("Could not save “{}”: {e}", path.display()));
        return false;
    }
    let previous = state.path.replace(path.clone());
    if copy || !saves_in_place(state) {
        state.path = previous;
        return true;
    }
    if let Some(name) = path.file_name() {
        state.doc.title = name.to_string_lossy().into_owned();
    }
    state.mark_saved();
    true
}

/// File > Revert: reloads the file, recorded as a "Revert" history state.
pub fn revert(app: &mut AppState) {
    let Some(state) = app.active() else {
        return;
    };
    let Some(path) = state.path.clone() else {
        return;
    };
    match op_io::open(&path) {
        Ok(disk) => {
            state.doc.restore(&disk.snapshot());
            state.record("Revert");
            state.mark_saved();
        }
        Err(e) => app.alert = Some(format!("Could not open “{}”: {e}", path.display())),
    }
}

/// Closes `ids` in order; a document with unsaved changes first asks
/// whether to save it (see [`continue_closing`]).
pub fn request_close(app: &mut AppState, ids: Vec<DocId>) {
    app.close_queue = ids;
    continue_closing(app);
}

/// Works through the close queue until a document needs to ask about its
/// unsaved changes, or the queue is empty (then quits, if quitting).
pub fn continue_closing(app: &mut AppState) {
    while let Some(&id) = app.close_queue.first() {
        match app.docs.get(&id) {
            Some(state) if state.is_dirty() => {
                app.active_doc = Some(id);
                app.save_prompt = Some(id);
                return;
            }
            Some(_) => app.close_document(id),
            None => {}
        }
        app.close_queue.remove(0);
    }
    if app.quit_after_close {
        app.quit_after_close = false;
        app.quit_approved = true;
    }
}

/// The answer to "Save changes?" for the document at the front of the queue.
pub fn answer_save_prompt(app: &mut AppState, answer: crate::dialogs::SaveChoice) {
    use crate::dialogs::SaveChoice;
    let Some(id) = app.save_prompt.take() else {
        return;
    };
    let close = match answer {
        SaveChoice::Save => save(app, id),
        SaveChoice::DontSave => true,
        SaveChoice::Cancel => false,
    };
    if close {
        app.close_document(id);
        app.close_queue.retain(|d| *d != id);
        continue_closing(app);
    } else {
        // Cancel (or a cancelled Save As) stops closing and quitting
        app.close_queue.clear();
        app.quit_after_close = false;
    }
}

pub fn close_active(app: &mut AppState) {
    if let Some(id) = app.active_doc {
        request_close(app, vec![id]);
    }
}

/// File > Close All.
pub fn close_all(app: &mut AppState) {
    request_close(app, app.doc_order.clone());
}

/// File > Close Others: closes every document except the active one.
pub fn close_others(app: &mut AppState) {
    let active = app.active_doc;
    let others = app
        .doc_order
        .iter()
        .copied()
        .filter(|id| Some(*id) != active)
        .collect();
    request_close(app, others);
}

/// Quitting: closes every document (asking about unsaved changes); the app
/// quits once all are closed.
pub fn quit(app: &mut AppState) {
    app.quit_after_close = true;
    request_close(app, app.doc_order.clone());
}

/// Single-key tool and color shortcuts. Modifier shortcuts are commands; see
/// [`crate::commands`].
pub fn handle_tool_keys(ctx: &egui::Context, app: &mut AppState) {
    if app.modal_open() {
        return;
    }
    // Single-key shortcuts are ignored while a text field has focus
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let keys: Vec<(Key, bool)> = ctx.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } if !modifiers.command && !modifiers.ctrl && !modifiers.alt => {
                    Some((*key, modifiers.shift))
                }
                _ => None,
            })
            .collect()
    });
    for (key, shift) in keys {
        if paint_key(app, key, shift) || nudge_key(app, key, shift) {
            continue;
        }
        match key {
            Key::D if !shift => crate::toolbar::reset_colors(app),
            Key::Q if !shift => {
                if let Some(state) = app.active() {
                    crate::toolbar::toggle_quick_mask(state);
                }
            }
            Key::X if !shift => crate::toolbar::swap_colors(app),
            _ => {
                let name = key.name();
                if name.len() == 1 {
                    let slots = app.tool_slots.clone();
                    let key = name.chars().next().unwrap();
                    if let Some(tool) =
                        op_tools::tool_for_key(key, shift, app.tool, |slot| slots[slot])
                    {
                        app.select_tool(tool);
                    }
                }
            }
        }
    }
}

/// Painting-tool keys, as in Photoshop: `[`/`]` change the size, Shift+`[`/`]`
/// the hardness in 25% steps, number keys the opacity (1 = 10% … 0 = 100%)
/// and Shift+number the flow. Returns whether the key was used.
fn paint_key(app: &mut AppState, key: Key, shift: bool) -> bool {
    let tool = app.tool;
    let Some(opts) = app.paint_options(tool) else {
        return false;
    };
    let digit = match key {
        Key::Num0 => Some(10),
        Key::Num1 => Some(1),
        Key::Num2 => Some(2),
        Key::Num3 => Some(3),
        Key::Num4 => Some(4),
        Key::Num5 => Some(5),
        Key::Num6 => Some(6),
        Key::Num7 => Some(7),
        Key::Num8 => Some(8),
        Key::Num9 => Some(9),
        _ => None,
    };
    match (key, shift) {
        (Key::OpenBracket, false) => {
            let step = crate::state::PaintOptions::size_step(opts.size - 0.5);
            opts.size = (opts.size - step).max(1.0);
        }
        (Key::CloseBracket, false) => {
            let step = crate::state::PaintOptions::size_step(opts.size);
            opts.size = (opts.size + step).min(crate::state::PaintOptions::MAX_SIZE);
        }
        (Key::OpenBracket, true) => opts.hardness = (opts.hardness - 0.25).max(0.0),
        (Key::CloseBracket, true) => opts.hardness = (opts.hardness + 0.25).min(1.0),
        _ => match digit {
            Some(d) if shift && tool != op_tools::Tool::Pencil => opts.flow = d as f32 / 10.0,
            Some(d) => opts.opacity = d as f32 / 10.0,
            None => return false,
        },
    }
    true
}

/// Arrow keys nudge with the Move tool (1 pixel, 10 with Shift).
fn nudge_key(app: &mut AppState, key: Key, shift: bool) -> bool {
    if app.tool != op_tools::Tool::Move {
        return false;
    }
    let step = if shift { 10 } else { 1 };
    let (dx, dy) = match key {
        Key::ArrowLeft => (-step, 0),
        Key::ArrowRight => (step, 0),
        Key::ArrowUp => (0, -step),
        Key::ArrowDown => (0, step),
        _ => return false,
    };
    let [r, g, b, _] = app.background.to_rgba8();
    if let Some(state) = app.active()
        && let Err(message) = crate::document_view::nudge(state, dx, dy, [r, g, b])
    {
        app.alert = Some(message);
    }
    true
}

/// Layer > Duplicate Layer... confirmed: copies the active layer, named
/// `name`, into the current document, another open one (which stays in the
/// background, as in Photoshop) or a new one (which opens and becomes
/// active). Recorded as "Duplicate Layer" in the document that gets it.
pub fn duplicate_layer(app: &mut AppState, name: &str, to: crate::dialogs::Destination) {
    use crate::dialogs::Destination;
    use op_core::layer_ops;
    let Some(source_id) = app.active_doc else {
        return;
    };
    match to {
        Destination::Document(target) if target == source_id => {
            if let Some(state) = app.active()
                && layer_ops::duplicate_named(&mut state.doc, name).is_some()
            {
                state.record("Duplicate Layer");
            }
        }
        Destination::Document(target) => {
            if let [Some(source), Some(state)] = app.docs.get_disjoint_mut([&source_id, &target])
                && layer_ops::duplicate_into(&source.doc, &mut state.doc, name).is_some()
            {
                state.record("Duplicate Layer");
            }
        }
        Destination::New(title) => {
            let Some(source) = app.docs.get(&source_id) else {
                return;
            };
            if let Some(doc) = layer_ops::duplicate_to_new(&source.doc, &title, name) {
                app.untitled_counter += 1;
                app.add_document(doc, "Duplicate Layer");
            }
        }
    }
}
