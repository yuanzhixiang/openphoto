//! Pop-up menus for dialog dropdowns. Photoshop's dialogs open native macOS
//! menus (light and translucent, the current item checked and placed over
//! the button). A dropdown asks for one with [`dropdown`]; the app opens it
//! natively after the frame (`menu.rs`, `NativeMenu::popup_menu`) and hands
//! the pick back with [`deliver`], which the dropdown returns on a later
//! frame. Without a native menu (other platforms, tests) the same entries
//! open as an egui menu.

use egui::{Context, Id, Rect, Response, Ui};

#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Item {
        label: String,
        checked: bool,
        enabled: bool,
        /// A key shown at the right ("⌥1"), as a menu shows key equivalents.
        shortcut: Option<String>,
    },
    Separator,
}

impl Entry {
    pub fn item(label: impl Into<String>, checked: bool) -> Self {
        Self::Item {
            label: label.into(),
            checked,
            enabled: true,
            shortcut: None,
        }
    }

    pub fn enabled(mut self, on: bool) -> Self {
        if let Self::Item { enabled, .. } = &mut self {
            *enabled = on;
        }
        self
    }

    pub fn shortcut(mut self, key: impl Into<String>) -> Self {
        if let Self::Item { shortcut, .. } = &mut self {
            *shortcut = Some(key.into());
        }
        self
    }
}

/// A menu to open natively: its entries and the button (egui points) the
/// checked entry is placed over.
#[derive(Clone, Debug)]
pub struct Request {
    pub id: Id,
    pub entries: Vec<Entry>,
    pub button: Rect,
}

impl Request {
    /// The entry placed over the button: the checked one, else the first.
    pub fn positioning(&self) -> usize {
        self.entries
            .iter()
            .position(|e| matches!(e, Entry::Item { checked: true, .. }))
            .unwrap_or(0)
    }
}

fn request_id() -> Id {
    Id::new("native-popup-request")
}

fn pick_id() -> Id {
    Id::new("native-popup-pick")
}

fn available_id() -> Id {
    Id::new("native-popup-available")
}

/// Whether menus open natively (set by the app once its window's view is
/// known).
pub fn set_available(ctx: &Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(available_id(), on));
}

pub fn available(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp(available_id()).unwrap_or(false))
}

/// The menu a dropdown asked for this frame, if any.
pub fn take_request(ctx: &Context) -> Option<Request> {
    ctx.data_mut(|d| {
        let request = d.get_temp::<Request>(request_id());
        d.remove::<Request>(request_id());
        request
    })
}

/// Hands entry `index` of the menu `id` back to its dropdown.
pub fn deliver(ctx: &Context, id: Id, index: usize) {
    ctx.data_mut(|d| d.insert_temp(pick_id(), (id, index)));
    ctx.request_repaint();
}

/// Asks for `entries` to open natively over `button`.
fn ask(ctx: &Context, id: Id, entries: &[Entry], button: Rect) {
    let request = Request {
        id,
        entries: entries.to_vec(),
        button,
    };
    ctx.data_mut(|d| d.insert_temp(request_id(), request));
}

/// Opens `entries` when `response` is clicked and returns the index of the
/// entry picked (separators count), once it is.
pub fn dropdown(ui: &mut Ui, response: &Response, id: Id, entries: &[Entry]) -> Option<usize> {
    let ctx = ui.ctx().clone();
    let picked = ctx.data_mut(|d| match d.get_temp::<(Id, usize)>(pick_id()) {
        Some((owner, index)) if owner == id => {
            d.remove::<(Id, usize)>(pick_id());
            Some(index)
        }
        _ => None,
    });
    if available(&ctx) {
        if response.clicked() {
            ask(&ctx, id, entries, response.rect);
        }
        return picked;
    }
    let mut chosen = picked;
    egui::Popup::menu(response).id(id.with("menu")).show(|ui| {
        for (i, entry) in entries.iter().enumerate() {
            match entry {
                Entry::Item {
                    label,
                    checked,
                    enabled,
                    shortcut,
                } => {
                    let mut button = egui::Button::selectable(*checked, label.as_str());
                    if let Some(key) = shortcut {
                        button = button.shortcut_text(key.as_str());
                    }
                    if ui.add_enabled(*enabled, button).clicked() {
                        chosen = Some(i);
                    }
                }
                Entry::Separator => {
                    ui.separator();
                }
            }
        }
    });
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_native_pick_comes_back_to_its_dropdown() {
        let ctx = Context::default();
        set_available(&ctx, true);
        let id = Id::new("units");
        let entries = [
            Entry::item("Inches", false),
            Entry::Separator,
            Entry::item("Pixels", true),
        ];
        let mut picked = None;
        let frame = |picked: &mut Option<usize>| {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                let response = ui.button("Units");
                *picked = dropdown(ui, &response, id, &entries);
            });
            output.textures_delta.clear();
        };
        ask(&ctx, id, &entries, Rect::ZERO);
        let request = take_request(&ctx).expect("asked for a menu");
        assert_eq!(request.entries.len(), 3);
        assert_eq!(request.positioning(), 2);
        // Another dropdown's pick isn't this one's
        deliver(&ctx, Id::new("other"), 0);
        frame(&mut picked);
        assert_eq!(picked, None);
        deliver(&ctx, id, 0);
        frame(&mut picked);
        assert_eq!(picked, Some(0));
        frame(&mut picked);
        assert_eq!(picked, None);
    }
}
