//! The Brushes panel (the brush presets) and the Brush Settings panel (the
//! current painting tool's tip shape, and what the pen's pressure
//! controls).

use egui::Ui;

use crate::state::AppState;

/// Brushes panel: the size, and Photoshop's General Brushes.
pub fn brushes(ui: &mut Ui, app: &mut AppState) {
    let tool = app.tool;
    let Some(options) = app.paint_options(tool) else {
        ui.label("Brushes need a painting tool.");
        return;
    };
    ui.horizontal(|ui| {
        ui.label("Size:");
        ui.add(
            egui::Slider::new(
                &mut options.size,
                1.0..=crate::state::PaintOptions::MAX_SIZE,
            )
            .logarithmic(true)
            .max_decimals(0)
            .suffix(" px"),
        );
    });
    ui.separator();
    ui.label("General Brushes");
    for (k, p) in crate::brush_presets::GENERAL.iter().enumerate() {
        if ui
            .selectable_label(options.preset == Some(k), p.name)
            .clicked()
        {
            crate::brush_presets::apply(options, k);
        }
    }
}

/// Brush Settings panel: Brush Tip Shape (size, angle, roundness,
/// hardness, spacing), and Shape Dynamics' and Transfer's pen pressure.
pub fn brush_settings(ui: &mut Ui, app: &mut AppState) {
    let tool = app.tool;
    let Some(options) = app.paint_options(tool) else {
        ui.label("Brush Settings need a painting tool.");
        return;
    };
    ui.strong("Brush Tip Shape");
    ui.horizontal(|ui| {
        ui.label("Size:");
        ui.add(
            egui::Slider::new(
                &mut options.size,
                1.0..=crate::state::PaintOptions::MAX_SIZE,
            )
            .logarithmic(true)
            .max_decimals(0)
            .suffix(" px"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Angle:");
        ui.add(
            egui::DragValue::new(&mut options.angle)
                .range(-180.0..=180.0)
                .suffix("°"),
        );
        ui.label("Roundness:");
        let mut pct = options.roundness * 100.0;
        if ui
            .add(
                egui::DragValue::new(&mut pct)
                    .range(1.0..=100.0)
                    .max_decimals(0)
                    .suffix("%"),
            )
            .changed()
        {
            options.roundness = pct / 100.0;
        }
    });
    ui.horizontal(|ui| {
        ui.label("Hardness:");
        let mut pct = options.hardness * 100.0;
        if ui
            .add(
                egui::Slider::new(&mut pct, 0.0..=100.0)
                    .max_decimals(0)
                    .suffix("%"),
            )
            .changed()
        {
            options.hardness = pct / 100.0;
        }
    });
    ui.horizontal(|ui| {
        ui.label("Spacing:");
        let mut pct = options.spacing * 100.0;
        if ui
            .add(
                egui::Slider::new(&mut pct, 1.0..=1000.0)
                    .logarithmic(true)
                    .max_decimals(0)
                    .suffix("%"),
            )
            .changed()
        {
            options.spacing = pct / 100.0;
        }
    });
    ui.separator();
    ui.checkbox(
        &mut options.pressure.size,
        "Shape Dynamics: Size Jitter — Pen Pressure",
    );
    ui.checkbox(
        &mut options.pressure.opacity,
        "Transfer: Opacity Jitter — Pen Pressure",
    );
    ui.checkbox(
        &mut options.pressure.flow,
        "Transfer: Flow Jitter — Pen Pressure",
    );
}
