//! Layer Comps panel (Window › Layer Comps): the document's layer comps,
//! Last Document State first; applying one puts its layers back.

use egui::Ui;
use op_core::layer_comps::{self, LayerComp};

use crate::state::{AppState, DocState};

/// The New Layer Comp dialog's fields.
#[derive(Clone, Debug, PartialEq)]
pub struct NewComp {
    pub name: String,
    pub visibility: bool,
    pub position: bool,
    pub appearance: bool,
    pub comment: String,
}

/// Applies comp `k` (None: Last Document State), recording "Apply Layer
/// Comp"; the layers before the first comp applied (since Last Document
/// State was last applied) are kept as Last Document State.
pub fn apply(state: &mut DocState, k: Option<usize>) {
    match k {
        Some(k) => {
            let Some(comp) = state.layer_comps.get(k).cloned() else {
                return;
            };
            if state.last_document_state.is_none() {
                state.last_document_state = Some(layer_comps::record(&state.doc));
            }
            layer_comps::apply(&mut state.doc, &comp);
            state.comp_applied = Some(k);
        }
        None => {
            let Some(layers) = state.last_document_state.take() else {
                return;
            };
            let all = LayerComp {
                name: String::new(),
                comment: String::new(),
                visibility: true,
                position: true,
                appearance: true,
                layers,
            };
            layer_comps::apply(&mut state.doc, &all);
            state.comp_applied = None;
        }
    }
    state.record("Apply Layer Comp");
}

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let Some(state) = app.active() else {
        return;
    };
    let mut apply_k: Option<Option<usize>> = None;
    let current = state.comp_applied;
    if ui
        .selectable_label(current.is_none(), "Last Document State")
        .clicked()
    {
        apply_k = Some(None);
    }
    for (k, comp) in state.layer_comps.iter().enumerate() {
        let r = ui.selectable_label(current == Some(k), &comp.name);
        let r = if comp.comment.is_empty() {
            r
        } else {
            r.on_hover_text(&comp.comment)
        };
        if r.clicked() {
            apply_k = Some(Some(k));
        }
    }
    ui.separator();
    let n = state.layer_comps.len();
    let mut new = false;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(n > 0, egui::Button::new("◀"))
            .on_hover_text("Apply Previous Selected Layer Comp")
            .clicked()
        {
            apply_k = Some(Some(current.map_or(n - 1, |c| (c + n - 1) % n)));
        }
        if ui
            .add_enabled(n > 0, egui::Button::new("▶"))
            .on_hover_text("Apply Next Selected Layer Comp")
            .clicked()
        {
            apply_k = Some(Some(current.map_or(0, |c| (c + 1) % n)));
        }
        if ui
            .add_enabled(current.is_some(), egui::Button::new("Update"))
            .clicked()
            && let Some(c) = current
        {
            state.layer_comps[c].layers = layer_comps::record(&state.doc);
        }
        if ui.button("New...").clicked() {
            new = true;
        }
        if ui
            .add_enabled(current.is_some(), egui::Button::new("Delete"))
            .clicked()
            && let Some(c) = current
        {
            state.layer_comps.remove(c);
            state.comp_applied = None;
        }
    });
    if let Some(k) = apply_k {
        apply(state, k);
    }
    if new {
        let name = format!("Layer Comp {}", n + 1);
        app.new_layer_comp = Some(NewComp {
            name,
            visibility: true,
            position: true,
            appearance: true,
            comment: String::new(),
        });
    }
}

/// The New Layer Comp dialog: OK records the layers as a new comp.
pub fn new_comp_dialog(ctx: &egui::Context, app: &mut AppState) {
    let Some(mut fields) = app.new_layer_comp.take() else {
        return;
    };
    let mut done = None;
    egui::Modal::new(egui::Id::new("new-layer-comp")).show(ctx, |ui| {
        ui.set_min_width(300.0);
        ui.heading("New Layer Comp");
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut fields.name);
        });
        ui.label("Apply To Layers:");
        ui.checkbox(&mut fields.visibility, "Visibility");
        ui.checkbox(&mut fields.position, "Position");
        ui.checkbox(&mut fields.appearance, "Appearance (Layer Style)");
        ui.label("Comment:");
        ui.text_edit_multiline(&mut fields.comment);
        ui.horizontal(|ui| {
            if ui.button("OK").clicked() {
                done = Some(true);
            }
            if ui.button("Cancel").clicked() {
                done = Some(false);
            }
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        done = Some(false);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !ctx.egui_wants_keyboard_input() {
        done = Some(true);
    }
    match done {
        Some(true) => {
            if let Some(state) = app.active() {
                let comp = LayerComp {
                    name: fields.name,
                    comment: fields.comment,
                    visibility: fields.visibility,
                    position: fields.position,
                    appearance: fields.appearance,
                    layers: layer_comps::record(&state.doc),
                };
                state.layer_comps.push(comp);
                state.comp_applied = Some(state.layer_comps.len() - 1);
            }
        }
        Some(false) => {}
        None => app.new_layer_comp = Some(fields),
    }
}
