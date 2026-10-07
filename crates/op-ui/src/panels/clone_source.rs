//! Clone Source panel (Window › Clone Source): five source slots, the
//! active source's offset, its scale and angle, and the overlay options.

use egui::Ui;

use crate::state::AppState;

/// The panel's settings, shared by the documents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClonePanel {
    /// W and H in percent, linked by default.
    pub scale: (f32, f32),
    pub linked: bool,
    /// Degrees, counterclockwise.
    pub angle: f32,
    pub show_overlay: bool,
    /// The overlay's opacity, 0–1.
    pub opacity: f32,
    pub clipped: bool,
    pub auto_hide: bool,
    pub invert: bool,
}

impl Default for ClonePanel {
    fn default() -> Self {
        Self {
            scale: (100.0, 100.0),
            linked: true,
            angle: 0.0,
            show_overlay: true,
            opacity: 1.0,
            clipped: true,
            auto_hide: false,
            invert: false,
        }
    }
}

impl ClonePanel {
    /// The stroke's transform about `origin` (None when it changes nothing).
    pub fn transform(&self, origin: egui::Pos2) -> Option<op_core::paint::SourceTransform> {
        let unchanged = self.scale == (100.0, 100.0) && self.angle == 0.0;
        (!unchanged).then_some(op_core::paint::SourceTransform {
            origin: (origin.x, origin.y),
            scale: (self.scale.0 / 100.0, self.scale.1 / 100.0),
            angle: self.angle,
        })
    }
}

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let mut panel = app.clone_panel;
    // The source slots
    if let Some(state) = app.active() {
        ui.horizontal(|ui| {
            for k in 0..5 {
                let set = state.clone_slots[k].0.is_some()
                    || (k == state.clone_slot && state.clone_source.is_some());
                let label = if set {
                    format!("◉{}", k + 1)
                } else {
                    format!("○{}", k + 1)
                };
                if ui
                    .selectable_label(state.clone_slot == k, label)
                    .on_hover_text("Clone Source")
                    .clicked()
                {
                    state.choose_clone_slot(k);
                }
            }
        });
        // The active source's offset (once painting has fixed it)
        ui.horizontal(|ui| {
            ui.label("Offset:");
            let mut offset = state.clone_offset.unwrap_or_default();
            let enabled = state.clone_source.is_some();
            let x = ui.add_enabled(
                enabled,
                egui::DragValue::new(&mut offset.x)
                    .prefix("X: ")
                    .suffix(" px"),
            );
            let y = ui.add_enabled(
                enabled,
                egui::DragValue::new(&mut offset.y)
                    .prefix("Y: ")
                    .suffix(" px"),
            );
            if x.changed() || y.changed() {
                state.clone_offset = Some(egui::vec2(offset.x.round(), offset.y.round()));
            }
        });
    }
    ui.horizontal(|ui| {
        let (mut w, mut h) = panel.scale;
        let wr = ui.add(
            egui::DragValue::new(&mut w)
                .range(1.0..=10000.0)
                .prefix("W: ")
                .suffix("%"),
        );
        if ui
            .selectable_label(panel.linked, "🔗")
            .on_hover_text("Maintain aspect ratio")
            .clicked()
        {
            panel.linked = !panel.linked;
        }
        let hr = ui.add(
            egui::DragValue::new(&mut h)
                .range(1.0..=10000.0)
                .prefix("H: ")
                .suffix("%"),
        );
        if panel.linked {
            if wr.changed() {
                h = w;
            } else if hr.changed() {
                w = h;
            }
        }
        panel.scale = (w, h);
    });
    ui.horizontal(|ui| {
        ui.add(
            egui::DragValue::new(&mut panel.angle)
                .range(-360.0..=360.0)
                .suffix("°"),
        );
        if ui.button("Reset Transform").clicked() {
            panel.scale = (100.0, 100.0);
            panel.angle = 0.0;
        }
    });
    ui.separator();
    ui.checkbox(&mut panel.show_overlay, "Show Overlay");
    ui.add_enabled_ui(panel.show_overlay, |ui| {
        ui.horizontal(|ui| {
            let mut pct = panel.opacity * 100.0;
            if ui
                .add(
                    egui::DragValue::new(&mut pct)
                        .range(0.0..=100.0)
                        .prefix("Opacity: ")
                        .suffix("%"),
                )
                .changed()
            {
                panel.opacity = pct / 100.0;
            }
        });
        ui.checkbox(&mut panel.clipped, "Clipped");
        ui.checkbox(&mut panel.auto_hide, "Auto Hide");
        ui.checkbox(&mut panel.invert, "Invert");
    });
    app.clone_panel = panel;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_only_when_scaled_or_turned() {
        let mut p = ClonePanel::default();
        assert!(p.transform(egui::pos2(1.0, 2.0)).is_none());
        p.scale = (200.0, 50.0);
        let t = p.transform(egui::pos2(1.0, 2.0)).unwrap();
        assert_eq!((t.origin, t.scale, t.angle), ((1.0, 2.0), (2.0, 0.5), 0.0));
    }
}
