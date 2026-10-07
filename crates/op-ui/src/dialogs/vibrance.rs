//! Image > Adjustments > Vibrance... and Posterize...: Photoshop 2026's
//! small UXP dialogs (401 × 166 and 401 × 164 pt), one or two sliders.
//! Sizes are Photoshop points from the dialog's top-left corner.

use egui::{Rect, Ui, vec2};
use op_core::adjust::Adjustment;

use super::uxp;
use crate::theme::pt;

pub const VIBRANCE_SIZE: egui::Vec2 = vec2(pt(401.0), pt(166.0));
pub const POSTERIZE_SIZE: egui::Vec2 = vec2(pt(401.0), pt(164.0));
const AMOUNT: (f32, f32) = (-100.0, 100.0);
const LEVELS: (f32, f32) = (2.0, 255.0);

fn parse(text: &str, (min, max): (f32, f32)) -> Option<i32> {
    let v: f32 = text.trim().parse().ok()?;
    (min..=max).contains(&v).then_some(v.round() as i32)
}

#[derive(Clone)]
pub struct Vibrance {
    pub vibrance: String,
    pub saturation: String,
}

impl Default for Vibrance {
    fn default() -> Self {
        Self {
            vibrance: "0".into(),
            saturation: "0".into(),
        }
    }
}

impl Vibrance {
    pub fn adjustment(&self) -> Option<Adjustment> {
        Some(Adjustment::Vibrance {
            vibrance: parse(&self.vibrance, AMOUNT)?,
            saturation: parse(&self.saturation, AMOUNT)?,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> Option<uxp::Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        for (k, (name, text, y)) in [
            ("Vibrance", &mut self.vibrance, 48.0),
            ("Saturation", &mut self.saturation, 103.0),
        ]
        .into_iter()
        .enumerate()
        {
            uxp::label(ui, at(20.0, y + 12.5), name);
            uxp::number_box(
                ui,
                Rect::from_min_max(at(215.0, y), at(260.0, y + 24.0)),
                text,
                ("vibrance-field", k),
                AMOUNT,
                first_frame && k == 0,
            );
            let value = parse(text, AMOUNT).unwrap_or(0) as f32;
            let track = |_| uxp::TRACK;
            if let Some(v) = uxp::slider(
                ui,
                ("vibrance", k),
                at(20.0, 0.0).x,
                at(260.0, 0.0).x,
                at(0.0, y + 36.0).y,
                value,
                AMOUNT,
                &track,
            ) {
                *text = format!("{v}");
            }
        }
        uxp::preview(ui, at(272.0, 127.0), preview);
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), false)
    }
}

#[derive(Clone)]
pub struct Posterize {
    pub levels: String,
}

impl Default for Posterize {
    fn default() -> Self {
        Self { levels: "4".into() }
    }
}

impl Posterize {
    pub fn adjustment(&self) -> Option<Adjustment> {
        Some(Adjustment::Posterize(parse(&self.levels, LEVELS)? as u8))
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> Option<uxp::Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        uxp::label(ui, at(20.0, 60.5), "Levels");
        uxp::number_box(
            ui,
            Rect::from_min_max(at(220.0, 48.0), at(260.0, 72.0)),
            &mut self.levels,
            "posterize-field",
            LEVELS,
            first_frame,
        );
        let value = parse(&self.levels, LEVELS).unwrap_or(4) as f32;
        let track = |_| uxp::TRACK;
        if let Some(v) = uxp::linear_slider(
            ui,
            "posterize",
            at(20.0, 0.0).x,
            at(260.0, 0.0).x,
            at(0.0, 84.0).y,
            value,
            LEVELS,
            &track,
            1.0,
        ) {
            self.levels = format!("{v}");
        }
        uxp::preview(ui, at(272.0, 127.0), preview);
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_ranges() {
        let mut v = Vibrance::default();
        assert_eq!(
            v.adjustment(),
            Some(Adjustment::Vibrance {
                vibrance: 0,
                saturation: 0
            })
        );
        v.saturation = "-101".into();
        assert_eq!(v.adjustment(), None);
        let mut p = Posterize::default();
        assert_eq!(p.adjustment(), Some(Adjustment::Posterize(4)));
        p.levels = "1".into();
        assert_eq!(p.adjustment(), None);
    }
}
