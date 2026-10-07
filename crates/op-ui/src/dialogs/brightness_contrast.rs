//! Image > Adjustments > Brightness/Contrast...: Photoshop 2026's UXP
//! dialog, 401 × 212 pt. Sizes are Photoshop points from the dialog's
//! top-left corner.

use egui::{Rect, Ui, vec2};
use op_core::adjust::Adjustment;

use super::uxp;
use crate::theme::pt;

pub const SIZE: egui::Vec2 = vec2(pt(401.0), pt(212.0));
const BRIGHTNESS: (f32, f32) = (-150.0, 150.0);
const CONTRAST: (f32, f32) = (-50.0, 100.0);

#[derive(Clone)]
pub struct Dialog {
    pub brightness: String,
    pub contrast: String,
    pub legacy: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            brightness: "0".into(),
            contrast: "0".into(),
            legacy: false,
        }
    }
}

fn parse(text: &str, (min, max): (f32, f32)) -> Option<f32> {
    let v: f32 = text.trim().parse().ok()?;
    (min..=max).contains(&v).then_some(v.round())
}

impl Dialog {
    pub fn adjustment(&self) -> Option<Adjustment> {
        Some(Adjustment::BrightnessContrast {
            brightness: parse(&self.brightness, BRIGHTNESS)? as i32,
            contrast: parse(&self.contrast, CONTRAST)? as i32,
            legacy: self.legacy,
        })
    }

    /// Auto: Photoshop's automatic values for the image.
    pub fn auto(&mut self, (brightness, contrast): (i32, i32)) {
        self.brightness = brightness.to_string();
        self.contrast = contrast.to_string();
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
        for (i, (name, text, range, y)) in [
            ("Brightness", &mut self.brightness, BRIGHTNESS, 48.0),
            ("Contrast", &mut self.contrast, CONTRAST, 103.0),
        ]
        .into_iter()
        .enumerate()
        {
            uxp::label(ui, at(20.0, y + 12.5), name);
            uxp::number_box(
                ui,
                r(203.0, y, 260.0, y + 24.0),
                text,
                ("bc-field", i),
                range,
                first_frame && i == 0,
            );
            let value = parse(text, range).unwrap_or(0.0);
            let track = |_| uxp::TRACK;
            if let Some(v) = uxp::slider(
                ui,
                ("bc", i),
                at(20.0, 0.0).x,
                at(260.0, 0.0).x,
                at(0.0, y + 36.0).y,
                value,
                range,
                &track,
            ) {
                *text = format!("{v}");
            }
        }
        uxp::checkbox(ui, at(20.0, 164.0), "Use Legacy", &mut self.legacy);
        uxp::preview(ui, at(272.0, 174.0), preview);
        uxp::buttons(
            ui,
            frame,
            Some(("Auto", true)),
            self.adjustment().is_some(),
            false,
        )
    }
}
