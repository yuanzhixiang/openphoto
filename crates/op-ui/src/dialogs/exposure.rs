//! Image > Adjustments > Exposure...: Photoshop 2026's UXP dialog,
//! 401 × 257 pt. Sizes are Photoshop points from the dialog's top-left
//! corner.

use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use op_core::adjust::Adjustment;

use super::{common, uxp};
use crate::ps_icons::{self, Icon};
use crate::theme::{color, pt};

pub const SIZE: egui::Vec2 = vec2(pt(401.0), pt(257.0));
const EXPOSURE: (f32, f32) = (-20.0, 20.0);
const OFFSET: (f32, f32) = (-0.5, 0.5);
const GAMMA: (f32, f32) = (0.01, 9.99);

#[derive(Clone)]
pub struct Dialog {
    /// Exposure, offset and gamma as typed (Photoshop shows gamma 1 as
    /// "+1").
    pub values: [String; 3],
    /// Which eyedropper is chosen (Photoshop starts on the white point).
    eyedropper: usize,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            values: ["0", "0", "+1"].map(String::from),
            eyedropper: 2,
        }
    }
}

fn parse(text: &str, (min, max): (f32, f32)) -> Option<f32> {
    let v: f32 = text.trim().parse().ok()?;
    (min..=max).contains(&v).then_some(v)
}

/// A number with up to `decimals` places, trailing zeros dropped.
fn trimmed(v: f32, decimals: usize) -> String {
    let s = format!("{v:.decimals$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    if s == "-0" { "0".into() } else { s }
}

impl Dialog {
    pub fn adjustment(&self) -> Option<Adjustment> {
        Some(Adjustment::Exposure {
            exposure: parse(&self.values[0], EXPOSURE)?,
            offset: parse(&self.values[1], OFFSET)?,
            gamma: parse(&self.values[2], GAMMA)?,
        })
    }

    /// The dialog showing Photoshop's preset `k`.
    fn preset_dialog(k: usize) -> Self {
        let [exposure, offset, gamma] = super::adjust_presets::EXPOSURE[k].1;
        let signed = |v: f32| {
            let t = trimmed(v, 2);
            if v > 0.0 { format!("+{t}") } else { t }
        };
        Self {
            values: [signed(exposure), trimmed(offset, 4), signed(gamma)],
            ..Self::default()
        }
    }

    /// The settings as an Exposure preset file (None while a field is
    /// invalid).
    fn encoded(&self) -> Option<Vec<u8>> {
        Some(super::preset_files::encode_exposure([
            parse(&self.values[0], EXPOSURE)?,
            parse(&self.values[1], OFFSET)?,
            parse(&self.values[2], GAMMA)?,
        ]))
    }

    /// A preset file's settings into the dialog, shown as Photoshop's
    /// presets are.
    fn load_preset(&mut self, bytes: &[u8]) {
        if let Some([exposure, offset, gamma]) = super::preset_files::decode_exposure(bytes) {
            let signed = |v: f32| {
                let t = trimmed(v, 2);
                if v > 0.0 { format!("+{t}") } else { t }
            };
            self.values = [signed(exposure), trimmed(offset, 4), signed(gamma)];
        }
    }

    fn preset(&self) -> &'static str {
        if self.adjustment() == Self::default().adjustment() {
            return "Default";
        }
        (0..super::adjust_presets::EXPOSURE.len())
            .find(|&k| self.adjustment() == Self::preset_dialog(k).adjustment())
            .map_or("Custom", |k| super::adjust_presets::EXPOSURE[k].0)
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> Option<uxp::Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();

        uxp::label(ui, at(20.0, 61.0), "Preset");
        use super::preset_files as files;
        let encoded = self.encoded();
        let saved_name = files::shown(files::EXPOSURE, encoded.as_deref());
        let preset = saved_name.as_deref().unwrap_or(self.preset());
        let saved = files::EXPOSURE.saved();
        let mut reset = false;
        let mut picked = None;
        let mut picked_saved = None;
        common::ps_dropdown(
            ui,
            r(56.0, 48.5, 231.0, 73.5),
            "exposure-preset",
            |p, rect| {
                p.text(
                    rect.left_center() + vec2(pt(9.0), pt(0.75)),
                    Align2::LEFT_CENTER,
                    preset,
                    uxp::font(),
                    uxp::TEXT,
                );
            },
            |ui| {
                if ui.button("Default").clicked() {
                    reset = true;
                }
                ui.add_enabled(false, egui::Button::new("Custom"));
                ui.separator();
                for (k, (name, _)) in super::adjust_presets::EXPOSURE.iter().enumerate() {
                    if ui.selectable_label(preset == *name, *name).clicked() {
                        picked = Some(k);
                    }
                }
                // The saved presets
                if !saved.is_empty() {
                    ui.separator();
                }
                for (k, (name, _)) in saved.iter().enumerate() {
                    if ui.selectable_label(preset == name, name).clicked() {
                        picked_saved = Some(k);
                    }
                }
            },
        );
        if reset {
            *self = Self::default();
        }
        if let Some(k) = picked {
            *self = Self::preset_dialog(k);
        }
        ps_icons::paint(
            &painter,
            at(248.0, 61.0),
            Icon::PresetMenu,
            uxp::TEXT,
            color::PANEL,
        );
        if let Some(bytes) =
            picked_saved.and_then(|k| files::load_saved(files::EXPOSURE, &saved[k]))
        {
            self.load_preset(&bytes);
        }
        let gear = Rect::from_center_size(at(248.0, 61.0), vec2(pt(18.0), pt(18.0)));
        if let Some(bytes) = files::gear(ui, gear, "exposure-gear", files::EXPOSURE, encoded) {
            self.load_preset(&bytes);
        }

        // Label, top, range, decimals, slider step
        type Row = (&'static str, f32, (f32, f32), usize, f32);
        let rows: [Row; 3] = [
            ("Exposure", 84.0, EXPOSURE, 2, 0.01),
            ("Offset", 139.0, OFFSET, 4, 0.0001),
            ("Gamma Correction", 194.0, GAMMA, 2, 0.01),
        ];
        for (k, (name, y, range, decimals, step)) in rows.into_iter().enumerate() {
            let text = &mut self.values[k];
            uxp::label(ui, at(20.0, y + 13.0), name);
            uxp::number_box(
                ui,
                r(201.0, y, 260.0, y + 24.0),
                text,
                ("exposure-field", k),
                (range.0, range.1),
                first_frame && k == 0,
            );
            let value = parse(text, range).unwrap_or(if k == 2 { 1.0 } else { 0.0 });
            let track = |_| uxp::TRACK;
            let changed = if k == 2 {
                uxp::linear_slider(
                    ui,
                    ("exposure", k),
                    at(20.0, 0.0).x,
                    at(260.0, 0.0).x,
                    at(0.0, y + 36.0).y,
                    value,
                    range,
                    &track,
                    step,
                )
            } else {
                uxp::slider_stepped(
                    ui,
                    ("exposure", k),
                    at(20.0, 0.0).x,
                    at(260.0, 0.0).x,
                    at(0.0, y + 36.0).y,
                    value,
                    range,
                    &track,
                    step,
                )
            };
            if let Some(v) = changed {
                let shown = trimmed(v, decimals);
                *text = if k == 2 { format!("+{shown}") } else { shown };
            }
        }

        // Set Black, Gray and White Point eyedroppers (not available here)
        for (k, (x, icon)) in [
            (283.5, Icon::EyedropperBlack),
            (319.5, Icon::EyedropperGray),
            (356.0, Icon::EyedropperWhite),
        ]
        .into_iter()
        .enumerate()
        {
            let rect = Rect::from_center_size(at(x, 133.0), vec2(pt(24.0), pt(24.0)));
            if ui
                .interact(rect, ui.id().with(("exposure-dropper", k)), Sense::click())
                .clicked()
            {
                self.eyedropper = k;
            }
            let chosen = self.eyedropper == k;
            if chosen {
                painter.rect(
                    rect,
                    pt(4.0),
                    Color32::from_gray(0xd8),
                    Stroke::NONE,
                    StrokeKind::Inside,
                );
            }
            let (tint, bg) = if chosen {
                (Color32::from_gray(0x32), Color32::from_gray(0xd8))
            } else {
                (uxp::TEXT, color::PANEL)
            };
            ps_icons::paint_scaled(&painter, rect.center(), icon, tint, bg, 0.9);
        }
        uxp::preview(ui, at(272.0, 162.0), preview);
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_files_round_trip() {
        let d = Dialog::preset_dialog(0);
        let mut e = Dialog::default();
        e.load_preset(&d.encoded().unwrap());
        assert_eq!(e.values, d.values);
    }

    #[test]
    fn photoshops_presets() {
        let d = Dialog::preset_dialog(2);
        assert_eq!(d.values, ["+1", "0", "+1"].map(String::from));
        assert_eq!(d.preset(), "Plus 1.0");
        assert_eq!(Dialog::preset_dialog(1).preset(), "Minus 2.0");
    }

    #[test]
    fn defaults_and_numbers() {
        let d = Dialog::default();
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::Exposure {
                exposure: 0.0,
                offset: 0.0,
                gamma: 1.0
            })
        );
        assert_eq!(d.preset(), "Default");
        assert_eq!(
            (trimmed(1.50, 2), trimmed(-0.0, 2), trimmed(0.0123, 4)),
            ("1.5".into(), "0".into(), "0.0123".into())
        );
    }
}
