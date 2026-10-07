//! Filter › Sharpen › Smart Sharpen..., as Photoshop 2026 lays it out (685
//! pt wide; 291 tall with Shadows / Highlights folded, 502 open; measured
//! from 2x captures): the preview on the left; Preview, the gear menu,
//! Preset, Amount, Radius, Reduce Noise and Remove with its angle on the
//! right; Shadows and Highlights under them.

use std::sync::Mutex;

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};

use super::filter_layout::Scale;
use super::legacy::Row;
use super::{AdjustDialog, Outcome, ParamKind};
use crate::dialogs::{appkit, common};
use crate::theme::{self, pt};

pub const WIDTH: f32 = 685.0;
/// The height folded and open.
pub const HEIGHTS: (f32, f32) = (291.0, 502.0);

/// The settings' order (`Kind::params`).
pub const AMOUNT: usize = 0;
pub const NOISE: usize = 2;
pub const REMOVE: usize = 3;
pub const ANGLE: usize = 4;
/// Shadows' Fade Amount, Tonal Width, Radius; then Highlights' three.
pub const SHADOWS: usize = 5;
pub const HIGHLIGHTS: usize = 8;
pub const LEGACY: usize = 11;
pub const MORE_ACCURATE: usize = 12;

/// Presets saved this session (name, values).
static SAVED: Mutex<Vec<(String, Vec<String>)>> = Mutex::new(Vec::new());

/// Radius's slider, measured on Photoshop by dragging its pin: 1 sits at
/// 16 % of the way, 64 at the end.
const RADIUS_SCALE: Scale = Scale::Table(&[
    (0.1, 0.0),
    (1.0, 0.163),
    (2.1, 0.248),
    (8.1, 0.489),
    (21.7, 0.737),
    (64.0, 1.0),
]);

/// A row whose label and field are centered on `y`.
fn row(label: &'static str, y: f32, unit: &'static str) -> Row {
    Row {
        label,
        label_right: 372.5,
        field: [539.0, y - 8.5, 583.0, y + 8.5],
        unit: Some((unit, 585.5)),
        track: (386.5, 524.5, y - 7.0),
        scale: Scale::Linear,
    }
}

impl AdjustDialog {
    /// The dialog's size: taller with Shadows / Highlights open.
    pub(super) fn smart_sharpen_size(&self) -> egui::Vec2 {
        let h = if self.extra.ss_open {
            HEIGHTS.1
        } else {
            HEIGHTS.0
        };
        vec2(pt(WIDTH), pt(h))
    }

    pub(super) fn smart_sharpen_ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Smart Sharpen", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        let open = self.extra.ss_open;
        self.legacy_preview(ui, frame, if open { 471.5 } else { 257.5 });
        let buttons = self.legacy_buttons(ui, frame, 613.0);
        self.legacy_preview_check(ui, frame, (378.5, 45.5));
        self.gear_menu(ui, r(577.0, 43.0, 597.0, 60.0));

        // Preset
        appkit::text(
            ui,
            at(372.5, 83.5),
            Align2::RIGHT_CENTER,
            "Preset:",
            appkit::TEXT,
        );
        self.preset_popup(ui, r(379.0, 73.5, 579.0, 93.5));

        // Amount, Radius, and Reduce Noise (More Accurate in Legacy)
        self.legacy_row(ui, frame, AMOUNT, &row("Amount:", 116.5, "%"), true, true);
        self.legacy_row(
            ui,
            frame,
            AMOUNT + 1,
            &Row {
                scale: RADIUS_SCALE,
                ..row("Radius:", 143.5, "px")
            },
            true,
            false,
        );
        if self.value(LEGACY) == Some(1.0) {
            let mut on = self.value(MORE_ACCURATE) == Some(1.0);
            appkit::checkbox_with(
                ui,
                at(386.5, 164.5),
                (12.0, 9.5),
                "More Accurate",
                &mut on,
                true,
            );
            self.values[MORE_ACCURATE] = (on as u8).to_string();
        } else {
            self.legacy_row(
                ui,
                frame,
                NOISE,
                &row("Reduce Noise:", 170.5, "%"),
                true,
                false,
            );
        }

        // Remove, with Motion Blur's angle
        appkit::text(
            ui,
            at(372.5, 206.5),
            Align2::RIGHT_CENTER,
            "Remove:",
            appkit::TEXT,
        );
        let remove = self.value(REMOVE).unwrap_or(1.0) as usize;
        if let ParamKind::Choice(options) = self.kind.params()[REMOVE].kind
            && let Some(k) = appkit::popup(
                ui,
                r(379.0, 197.0, 480.5, 216.0),
                "ss-remove",
                options[remove.min(2)],
                &appkit::choices(options.iter().copied(), remove.min(2)),
            )
        {
            self.values[REMOVE] = k.to_string();
        }
        let motion = remove == 2;
        let field = r(489.0, 197.5, 533.0, 215.5);
        if motion {
            let p = &self.kind.params()[ANGLE];
            appkit::field(
                ui,
                field,
                &mut self.values[ANGLE],
                ("ss-angle", ANGLE),
                (p.min, p.max),
                1.0,
                0,
                false,
            );
            self.dial(ui, ANGLE, at(575.0, 206.5), pt(17.5), false);
        } else {
            painter.rect(
                field,
                0,
                Color32::from_gray(0x4d),
                Stroke::new(pt(1.0), Color32::from_gray(0x5d)),
                egui::StrokeKind::Inside,
            );
            appkit::text(
                ui,
                field.left_center() + vec2(pt(4.0), 0.0),
                Align2::LEFT_CENTER,
                &self.values[ANGLE],
                appkit::TEXT_OFF,
            );
            painter.circle_stroke(
                at(575.0, 206.5),
                pt(17.5),
                Stroke::new(pt(1.0), Color32::from_gray(0x80)),
            );
            painter.circle_filled(at(575.0, 206.5), pt(1.5), Color32::from_gray(0x80));
        }
        let ink = if motion {
            appkit::TEXT
        } else {
            appkit::TEXT_OFF
        };
        appkit::text(ui, at(541.0, 204.0), Align2::LEFT_CENTER, "°", ink);

        // Shadows / Highlights, folded or open
        let bold = theme::dialog_bold(pt(13.0));
        let header = r(285.0, 245.0, 598.5, 266.0);
        let (title, title_y) = if open {
            ("Shadows", 253.0)
        } else {
            ("Shadows / Highlights", 257.5)
        };
        if open {
            painter.line_segment(
                [at(285.5, 243.0), at(598.5, 243.0)],
                Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
            );
        }
        chevron(&painter, at(294.0, title_y), open);
        painter.text(
            at(304.5, title_y),
            Align2::LEFT_CENTER,
            title,
            bold.clone(),
            appkit::TEXT,
        );
        let fold = if open {
            r(285.0, 243.0, 598.5, 263.0)
        } else {
            header
        };
        if ui
            .interact(fold, ui.id().with("ss-fold"), Sense::click())
            .clicked()
        {
            self.extra.ss_open = !open;
            // The preview changes size
            self.pane = None;
        }
        if open {
            for (k, (label, unit)) in [
                ("Fade Amount:", "%"),
                ("Tonal Width:", "%"),
                ("Radius:", "px"),
            ]
            .into_iter()
            .enumerate()
            {
                self.legacy_row(
                    ui,
                    frame,
                    SHADOWS + k,
                    &row(label, 285.0 + 27.0 * k as f32, unit),
                    true,
                    false,
                );
            }
            painter.line_segment(
                [at(295.0, 356.5), at(598.5, 356.5)],
                Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
            );
            painter.text(
                at(295.0, 374.0),
                Align2::LEFT_CENTER,
                "Highlights",
                bold,
                appkit::TEXT,
            );
            for (k, (label, unit)) in [
                ("Fade Amount:", "%"),
                ("Tonal Width:", "%"),
                ("Radius:", "px"),
            ]
            .into_iter()
            .enumerate()
            {
                self.legacy_row(
                    ui,
                    frame,
                    HIGHLIGHTS + k,
                    &row(label, 399.0 + 27.0 * k as f32, unit),
                    true,
                    false,
                );
            }
        }
        self.legacy_outcome(ui, buttons)
    }

    /// The gear: Use Legacy (checked when on), Save Preset... (keeps the
    /// settings as "Preset N" for the session) and Delete Preset (the
    /// chosen saved one).
    fn gear_menu(&mut self, ui: &mut Ui, rect: Rect) {
        let response = ui.interact(rect, ui.id().with("ss-gear"), Sense::click());
        gear_icon(
            ui.painter(),
            rect.center() - vec2(pt(2.0), 0.0),
            response.hovered(),
        );
        let legacy = self.value(LEGACY) == Some(1.0);
        let mut pick = None;
        egui::Popup::menu(&response)
            .id(ui.id().with("ss-gear-menu"))
            .show(|ui| {
                let check = if legacy {
                    "✓ Use Legacy"
                } else {
                    "Use Legacy"
                };
                if ui.button(check).clicked() {
                    pick = Some(0);
                }
                if ui.button("Save Preset...").clicked() {
                    pick = Some(1);
                }
                if ui
                    .add_enabled(
                        self.extra.ss_preset.is_some(),
                        egui::Button::new("Delete Preset"),
                    )
                    .clicked()
                {
                    pick = Some(2);
                }
            });
        match pick {
            Some(0) => self.values[LEGACY] = (!legacy as u8).to_string(),
            Some(1) => {
                let mut saved = SAVED.lock().expect("presets");
                let name = format!("Preset {}", saved.len() + 1);
                saved.push((name.clone(), self.values.clone()));
                self.extra.ss_preset = Some(name);
            }
            Some(2) => {
                if let Some(name) = self.extra.ss_preset.take() {
                    SAVED.lock().expect("presets").retain(|(n, _)| *n != name);
                }
            }
            _ => {}
        }
    }

    /// Preset: Default (Photoshop's values), then the saved presets.
    fn preset_popup(&mut self, ui: &mut Ui, rect: Rect) {
        let saved = SAVED.lock().expect("presets").clone();
        let names: Vec<&str> = std::iter::once("Default")
            .chain(saved.iter().map(|(n, _)| n.as_str()))
            .collect();
        let chosen = self
            .extra
            .ss_preset
            .as_ref()
            .and_then(|n| saved.iter().position(|(m, _)| m == n))
            .map_or(0, |k| k + 1);
        if let Some(k) = appkit::popup(
            ui,
            rect,
            "ss-preset",
            names[chosen],
            &appkit::choices(names.iter().copied(), chosen),
        ) {
            if k == 0 {
                self.extra.ss_preset = None;
                // Everything but Use Legacy to Photoshop's
                for (i, p) in self.kind.params().iter().enumerate() {
                    if i != LEGACY {
                        self.values[i] = super::format(self.kind, p.default, p.decimals);
                    }
                }
            } else {
                let (name, values) = &saved[k - 1];
                self.values = values.clone();
                self.extra.ss_preset = Some(name.clone());
            }
        }
    }
}

/// A disclosure chevron: pointing right folded, down open.
fn chevron(painter: &egui::Painter, c: Pos2, open: bool) {
    let s = Stroke::new(pt(1.25), appkit::TEXT);
    let p = |x: f32, y: f32| c + vec2(pt(x), pt(y));
    let points = if open {
        vec![p(-4.0, -2.0), p(0.0, 2.0), p(4.0, -2.0)]
    } else {
        vec![p(-2.0, -4.0), p(2.0, 0.0), p(-2.0, 4.0)]
    };
    painter.add(egui::Shape::line(points, s));
}

/// The gear with its menu triangle.
fn gear_icon(painter: &egui::Painter, c: Pos2, hover: bool) {
    let ink = if hover {
        appkit::TEXT
    } else {
        Color32::from_gray(0xd0)
    };
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::TAU / 8.0;
        let d = vec2(a.cos(), a.sin());
        painter.line_segment(
            [c + d * pt(3.5), c + d * pt(6.0)],
            Stroke::new(pt(2.2), ink),
        );
    }
    painter.circle_filled(c, pt(4.5), ink);
    painter.circle_filled(c, pt(1.8), Color32::from_gray(0x53));
    let t = c + vec2(pt(9.0), pt(5.0));
    painter.add(egui::Shape::convex_polygon(
        vec![
            t + vec2(-pt(2.5), -pt(1.2)),
            t + vec2(pt(2.5), -pt(1.2)),
            t + vec2(0.0, pt(1.3)),
        ],
        ink,
        Stroke::NONE,
    ));
}
