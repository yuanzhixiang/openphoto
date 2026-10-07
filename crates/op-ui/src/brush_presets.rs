//! Brush presets and the brush picker's popup: Size, Hardness, the tip's
//! angle and roundness, and Photoshop's General Brushes.

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};

use crate::state::PaintOptions;
use op_core::paint::Pressure;

/// A brush preset: its name, size, hardness and what pressure controls.
pub struct Preset {
    pub name: &'static str,
    pub size: f32,
    pub hardness: f32,
    pub pressure: Pressure,
}

const fn preset(
    name: &'static str,
    hardness: f32,
    size: bool,
    opacity: bool,
    flow: bool,
) -> Preset {
    Preset {
        name,
        size: 30.0,
        hardness,
        pressure: Pressure {
            size,
            opacity,
            flow,
        },
    }
}

/// Photoshop 2026's General Brushes folder.
pub const GENERAL: [Preset; 8] = [
    preset("Soft Round", 0.0, false, false, false),
    preset("Hard Round", 1.0, false, false, false),
    preset("Soft Round Pressure Size", 0.0, true, false, false),
    preset("Hard Round Pressure Size", 1.0, true, false, false),
    preset("Soft Round Pressure Opacity", 0.0, false, true, false),
    preset("Hard Round Pressure Opacity", 1.0, false, true, false),
    preset(
        "Soft Round Pressure Opacity and Flow",
        0.0,
        false,
        true,
        true,
    ),
    preset(
        "Hard Round Pressure Opacity and Flow",
        1.0,
        false,
        true,
        true,
    ),
];

/// Picking preset `k`: its size, hardness and dynamics, a round tip.
pub fn apply(options: &mut PaintOptions, k: usize) {
    let Some(p) = GENERAL.get(k) else {
        return;
    };
    options.size = p.size;
    options.hardness = p.hardness;
    options.pressure = p.pressure;
    options.angle = 0.0;
    options.roundness = 1.0;
    options.spacing = 0.25;
    options.preset = Some(k);
}

/// The popup's controls for `options` (`hardness` false for the Pencil):
/// Size, Hardness, the angle and roundness dial with its fields, and the
/// presets. Changing the tip by hand keeps the preset's dynamics.
pub fn picker_ui(ui: &mut Ui, options: &mut PaintOptions, hardness: bool) {
    ui.set_min_width(260.0);
    ui.spacing_mut().slider_width = 180.0;
    ui.horizontal(|ui| {
        ui.label("Size:");
        ui.add(
            egui::Slider::new(&mut options.size, 1.0..=PaintOptions::MAX_SIZE)
                .logarithmic(true)
                .max_decimals(0)
                .suffix(" px"),
        );
    });
    if hardness {
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
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        tip_dial(ui, options);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Angle:");
                ui.add(
                    egui::DragValue::new(&mut options.angle)
                        .range(-180.0..=180.0)
                        .max_decimals(0)
                        .suffix("°"),
                );
            });
            ui.horizontal(|ui| {
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
        });
    });
    ui.separator();
    ui.label("General Brushes");
    for (k, p) in GENERAL.iter().enumerate() {
        let chosen = options.preset == Some(k);
        let row = ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(28.0, 22.0), Sense::hover());
            thumbnail(ui, rect, p);
            ui.selectable_label(chosen, p.name)
        });
        if row.inner.clicked() {
            apply(options, k);
        }
    }
}

/// The tip's preview: an ellipse turned by the angle, with handles at its
/// ends; dragging sets the angle, dragging a handle the roundness.
fn tip_dial(ui: &mut Ui, options: &mut PaintOptions) {
    let (rect, response) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::drag());
    let c = rect.center();
    let painter = ui.painter_at(rect);
    painter.circle_stroke(c, 28.0, Stroke::new(1.0, Color32::from_gray(0x80)));
    let (s, co) = options.angle.to_radians().sin_cos();
    let along = vec2(co, -s);
    let across = vec2(s, co);
    let points: Vec<Pos2> = (0..48)
        .map(|k| {
            let t = k as f32 / 48.0 * std::f32::consts::TAU;
            c + along * (t.cos() * 24.0) + across * (t.sin() * 24.0 * options.roundness)
        })
        .collect();
    painter.add(egui::Shape::convex_polygon(
        points,
        Color32::from_gray(0x70),
        Stroke::new(1.0, Color32::WHITE),
    ));
    painter.line_segment(
        [c - along * 28.0, c + along * 28.0],
        Stroke::new(1.0, Color32::WHITE),
    );
    if response.dragged()
        && let Some(p) = response.interact_pointer_pos()
    {
        let d = p - c;
        let near_axis = (d.normalized().dot(across)).abs() > 0.7;
        if near_axis && d.length() < 28.0 {
            options.roundness = (d.dot(across).abs() / 24.0).clamp(0.01, 1.0);
        } else {
            options.angle = (-d.y).atan2(d.x).to_degrees().round();
        }
    }
}

/// A preset's thumbnail: a dot showing its hardness.
fn thumbnail(ui: &Ui, rect: Rect, p: &Preset) {
    let c = rect.center();
    let painter = ui.painter();
    let rings = 8;
    for k in 0..rings {
        let t = 1.0 - k as f32 / rings as f32;
        let alpha = if t <= p.hardness.max(0.05) {
            1.0
        } else {
            ((1.0 - t) / (1.0 - p.hardness.max(0.05))).clamp(0.0, 1.0)
        };
        painter.circle_filled(
            c,
            9.0 * t,
            Color32::from_gray(0xf0).gamma_multiply(alpha.powf(1.5)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_size_hardness_and_dynamics() {
        let mut o = PaintOptions {
            angle: 30.0,
            roundness: 0.5,
            ..crate::state::AppState::default().brush
        };
        apply(&mut o, 3);
        assert_eq!(o.size, 30.0);
        assert_eq!(o.hardness, 1.0);
        assert!(o.pressure.size && !o.pressure.opacity);
        assert_eq!((o.angle, o.roundness), (0.0, 1.0));
        assert_eq!(o.preset, Some(3));
        assert_eq!(GENERAL[7].name, "Hard Round Pressure Opacity and Flow");
    }
}
