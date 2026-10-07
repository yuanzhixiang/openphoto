//! The "Save changes?" prompt shown when closing a document (or quitting)
//! with unsaved changes. Like Photoshop 2026's, it is a macOS alert (see
//! `alert.rs`): the caution icon, the question, and Save, Don't Save and
//! Cancel stacked full width. Enter means Save, Escape Cancel and Cmd+D
//! Don't Save.

use egui::{Key, Modifiers};

use super::alert::{self, Alert, Answer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveChoice {
    Save,
    DontSave,
    Cancel,
}

/// The prompt's alert for the document titled `title`.
pub fn alert(title: &str) -> Alert {
    Alert::choose(
        format!("Save changes to the OpenPhoto document “{title}” before closing?"),
        &["Save", "Don’t Save", "Cancel"],
    )
}

/// Shows the prompt for the document titled `title`; returns the answer
/// once given.
pub fn show(ctx: &egui::Context, title: &str) -> Option<SaveChoice> {
    // macOS's shortcut for "Don't Save"
    if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::D)) {
        return Some(SaveChoice::DontSave);
    }
    match alert::show(ctx, &mut alert(title))? {
        Answer::Choice(0) => Some(SaveChoice::Save),
        Answer::Choice(1) => Some(SaveChoice::DontSave),
        _ => Some(SaveChoice::Cancel),
    }
}
