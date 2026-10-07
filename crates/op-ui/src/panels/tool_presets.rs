//! Tool Presets panel (Window › Tool Presets): the saved tool presets,
//! of the current tool only or of all tools; a click picks one.

use egui::Ui;

use crate::state::AppState;

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let mut picked = None;
    let mut deleted = None;
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 48.0)
        .show(ui, |ui| {
            for (k, p) in app.tool_presets.iter().enumerate() {
                if app.tool_presets_current_only && p.tool != app.tool {
                    continue;
                }
                ui.horizontal(|ui| {
                    if ui.selectable_label(false, &p.name).clicked() {
                        picked = Some(k);
                    }
                    if ui
                        .small_button("🗑")
                        .on_hover_text("Delete Tool Preset")
                        .clicked()
                    {
                        deleted = Some(k);
                    }
                });
            }
            if app.tool_presets.is_empty() {
                ui.weak("No tool presets.");
            }
        });
    ui.separator();
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.tool_presets_current_only, "Current Tool Only");
        if ui.button("New Tool Preset...").clicked() {
            open_new(app);
        }
    });
    if let Some(k) = picked {
        let preset = app.tool_presets[k].clone();
        crate::tool_presets::apply(app, &preset);
    }
    if let Some(k) = deleted {
        app.tool_presets.remove(k);
    }
}

/// New Tool Preset...: the name dialog for the current tool.
pub fn open_new(app: &mut AppState) {
    let name = crate::tool_presets::suggested_name(app, app.tool);
    app.new_tool_preset = Some(crate::dialogs::new_preset::NewPresetDialog::new(
        "New Tool Preset",
        name,
    ));
}
