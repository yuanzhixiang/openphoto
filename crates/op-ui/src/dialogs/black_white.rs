//! Image > Adjustments > Black & White... (⌥⇧⌘B): Photoshop 2026's UXP
//! dialog, 401 × 568 pt: six color weights and an optional tint. Sizes are
//! Photoshop points from the dialog's top-left corner.

use egui::{Align2, Color32, CornerRadius, Rect, Stroke, StrokeKind, Ui, vec2};
use op_core::adjust::Adjustment;

use super::{common, uxp};
use crate::ps_icons::{self, Icon};
use crate::theme::{color, pt};

pub const SIZE: egui::Vec2 = vec2(pt(401.0), pt(568.0));
const WEIGHT: (f32, f32) = (-200.0, 300.0);
const HUE: (f32, f32) = (0.0, 360.0);
const SATURATION: (f32, f32) = (0.0, 100.0);
/// Photoshop's Default preset.
const DEFAULTS: [i32; 6] = [40, 60, 40, 60, 20, 80];
const NAMES: [&str; 6] = [
    "Reds:",
    "Yellows:",
    "Greens:",
    "Cyans:",
    "Blues:",
    "Magentas:",
];
/// The color each track passes through at its middle (between black at
/// −200% and white at 300%).
const COLORS: [[u8; 3]; 6] = [
    [234, 51, 35],
    [255, 255, 85],
    [117, 251, 76],
    [117, 251, 253],
    [0, 0, 245],
    [234, 51, 247],
];

#[derive(Clone)]
pub struct Dialog {
    pub weights: [String; 6],
    pub tint: bool,
    /// Tint hue (degrees) and saturation (percent).
    pub hue: String,
    pub saturation: String,
    focus: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            weights: DEFAULTS.map(|w| w.to_string()),
            tint: false,
            hue: "42".into(),
            saturation: "20".into(),
            focus: false,
        }
    }
}

fn parse(text: &str, (min, max): (f32, f32)) -> Option<i32> {
    let v: f32 = text
        .trim()
        .trim_end_matches(['%', '°'])
        .trim()
        .parse()
        .ok()?;
    (min..=max).contains(&v).then_some(v.round() as i32)
}

/// The tint color for a hue and saturation: HSB at Photoshop's brightness
/// of 88.2% (hue 42°, saturation 20% gives (225, 211, 180)).
pub fn tint_color(hue: i32, saturation: i32) -> [u8; 3] {
    let v = 0.882f32;
    let s = saturation as f32 / 100.0;
    let h = (hue as f32).rem_euclid(360.0) / 60.0;
    let f = h.fract();
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let [r, g, b] = match h as usize {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    };
    [r, g, b].map(|c| (c * 255.0).round() as u8)
}

impl Dialog {
    pub fn adjustment(&self) -> Option<Adjustment> {
        let mut weights = [0; 6];
        for (w, t) in weights.iter_mut().zip(&self.weights) {
            *w = parse(t, WEIGHT)?;
        }
        let tint = if self.tint {
            Some(tint_color(
                parse(&self.hue, HUE)?,
                parse(&self.saturation, SATURATION)?,
            ))
        } else {
            None
        };
        Some(Adjustment::BlackWhite { weights, tint })
    }

    fn preset(&self) -> &'static str {
        let untouched = self
            .weights
            .iter()
            .zip(DEFAULTS)
            .all(|(t, d)| parse(t, WEIGHT) == Some(d))
            && !self.tint;
        if untouched {
            return "Default";
        }
        super::adjust_presets::BLACK_WHITE
            .iter()
            .find(|(_, w)| {
                !self.tint
                    && self
                        .weights
                        .iter()
                        .zip(w)
                        .all(|(t, d)| parse(t, WEIGHT) == Some(*d))
            })
            .map_or("Custom", |(name, _)| name)
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

        uxp::label(ui, at(20.0, 61.0), "Preset:");
        let preset = self.preset();
        let mut reset = false;
        let mut picked = None;
        common::ps_dropdown(
            ui,
            r(59.0, 48.5, 231.0, 73.5),
            "bw-preset",
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
                for (k, (name, _)) in super::adjust_presets::BLACK_WHITE.iter().enumerate() {
                    if ui.selectable_label(preset == *name, *name).clicked() {
                        picked = Some(k);
                    }
                }
            },
        );
        if let Some(k) = picked {
            *self = Self::default();
            self.weights = super::adjust_presets::BLACK_WHITE[k]
                .1
                .map(|w| w.to_string());
        }
        if reset {
            *self = Self::default();
            self.focus = true;
        }
        ps_icons::paint(
            &painter,
            at(248.0, 61.0),
            Icon::PresetMenu,
            uxp::TEXT,
            color::PANEL,
        );

        let focus = first_frame || std::mem::take(&mut self.focus);
        let percent = |ui: &Ui, field: Rect, text: &str, suffix: &str, tint: Color32| {
            if !text.contains(suffix) {
                let shown = ui
                    .painter()
                    .layout_no_wrap(text.to_string(), uxp::font(), tint);
                ui.painter().text(
                    field.left_center() + vec2(pt(11.0) + shown.size().x, pt(0.75)),
                    Align2::LEFT_CENTER,
                    suffix,
                    uxp::font(),
                    tint,
                );
            }
        };
        for k in 0..6 {
            let y = 84.0 + 55.0 * k as f32;
            uxp::label(ui, at(20.0, y + 13.0), NAMES[k]);
            let field = r(209.0, y, 260.0, y + 24.0);
            uxp::number_box(
                ui,
                field,
                &mut self.weights[k],
                ("bw-field", k),
                WEIGHT,
                focus && k == 0,
            );
            percent(ui, field, &self.weights[k], "%", uxp::TEXT);
            let value = parse(&self.weights[k], WEIGHT).unwrap_or(DEFAULTS[k]) as f32;
            let [cr, cg, cb] = COLORS[k];
            let colors = uxp::three(
                Color32::BLACK,
                Color32::from_rgb(cr, cg, cb),
                Color32::WHITE,
            );
            if let Some(v) = uxp::linear_slider(
                ui,
                ("bw", k),
                at(20.0, 0.0).x,
                at(260.0, 0.0).x,
                at(0.0, y + 36.0).y,
                value,
                WEIGHT,
                &colors,
                1.0,
            ) {
                self.weights[k] = format!("{v}");
            }
        }

        // Tint: its swatch, then hue and saturation (dimmed while off)
        uxp::checkbox(ui, at(20.0, 420.5), "Tint", &mut self.tint);
        let hue = parse(&self.hue, HUE).unwrap_or(42);
        let saturation = parse(&self.saturation, SATURATION).unwrap_or(20);
        let [tr, tg, tb] = tint_color(hue, saturation);
        let swatch_fill = if self.tint {
            Color32::from_rgb(tr, tg, tb)
        } else {
            Color32::from_rgb(0x75, 0x73, 0x6c)
        };
        painter.rect(
            r(72.5, 414.0, 120.0, 437.5),
            CornerRadius::same(pt(4.0) as u8),
            swatch_fill,
            Stroke::new(pt(1.0), Color32::from_gray(0x8a)),
            StrokeKind::Inside,
        );
        let dim = |c: Color32| {
            if self.tint {
                c
            } else {
                c.lerp_to_gamma(color::PANEL, 0.6)
            }
        };
        for (k, (name, y, range, suffix)) in [
            ("Hue", 450.0, HUE, "°"),
            ("Saturation", 505.0, SATURATION, "%"),
        ]
        .into_iter()
        .enumerate()
        {
            painter.text(
                at(20.0, y + 13.75),
                Align2::LEFT_CENTER,
                name,
                uxp::font(),
                dim(uxp::TEXT),
            );
            let field = r(209.0, y, 260.0, y + 24.0);
            let text = if k == 0 {
                &mut self.hue
            } else {
                &mut self.saturation
            };
            if self.tint {
                uxp::number_box(ui, field, text, ("bw-tint", k), range, false);
            } else {
                painter.rect_filled(field, pt(3.0), Color32::from_gray(0x4c));
                painter.text(
                    field.left_center() + vec2(pt(11.0), pt(0.75)),
                    Align2::LEFT_CENTER,
                    text.as_str(),
                    uxp::font(),
                    dim(uxp::TEXT),
                );
            }
            percent(ui, field, text, suffix, dim(uxp::TEXT));
            let value = parse(text, range).unwrap_or(0) as f32;
            let hue_track = |t: f32| {
                let [r, g, b] = tint_color((t * 360.0) as i32, 45);
                dim(Color32::from_rgb(r, g, b))
            };
            let sat_track = |t: f32| {
                dim(Color32::from_gray(0x76).lerp_to_gamma(Color32::from_rgb(tr, tg, tb), t))
            };
            let colors: &dyn Fn(f32) -> Color32 = if k == 0 { &hue_track } else { &sat_track };
            if self.tint
                && let Some(v) = uxp::linear_slider(
                    ui,
                    ("bw-tint", k),
                    at(20.0, 0.0).x,
                    at(260.0, 0.0).x,
                    at(0.0, y + 36.0).y,
                    value,
                    range,
                    colors,
                    1.0,
                )
            {
                *text = format!("{v}");
            } else if !self.tint {
                uxp::draw_slider(
                    ui,
                    at(20.0, 0.0).x,
                    at(260.0, 0.0).x,
                    at(0.0, y + 36.0).y,
                    value,
                    range,
                    colors,
                    Color32::from_gray(0xd0).lerp_to_gamma(color::PANEL, 0.85),
                );
            }
        }

        uxp::preview(ui, at(272.0, 163.0), preview);
        // Auto (Photoshop picks weights from the image) is not available here
        uxp::buttons(
            ui,
            frame,
            Some(("Auto", false)),
            self.adjustment().is_some(),
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photoshops_presets() {
        let mut d = Dialog {
            weights: super::super::adjust_presets::BLACK_WHITE[0]
                .1
                .map(|w| w.to_string()),
            ..Default::default()
        };
        assert_eq!(d.preset(), "Blue Filter");
        d.tint = true;
        assert_eq!(d.preset(), "Custom");
    }

    #[test]
    fn weights_and_tint() {
        let mut d = Dialog::default();
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::BlackWhite {
                weights: DEFAULTS,
                tint: None
            })
        );
        assert_eq!(tint_color(42, 20), [225, 211, 180]);
        d.tint = true;
        d.weights[4] = "-50%".into();
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::BlackWhite {
                weights: [40, 60, 40, 60, -50, 80],
                tint: Some([225, 211, 180])
            })
        );
        assert_eq!(d.preset(), "Custom");
        d.hue = "361".into();
        assert_eq!(d.adjustment(), None);
    }
}
