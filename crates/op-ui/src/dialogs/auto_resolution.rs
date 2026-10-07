//! Image Size › Fit To › Auto Resolution...: the resolution for printing at
//! a halftone screen frequency. Laid out at the positions measured on
//! Photoshop 2026's sheet (348 × 135 pt, 15 pt in from the Image Size
//! window's top-left corner); sizes are in Photoshop points from its
//! top-left corner.

use std::sync::Mutex;

use egui::{Align2, Key, Pos2, Rect, Sense, Ui, vec2};

use super::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(348.0), pt(135.0));
/// From the Image Size window's corner.
pub const OFFSET: egui::Vec2 = vec2(pt(15.0), pt(15.0));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenUnit {
    LinesPerInch,
    LinesPerCentimeter,
}

impl ScreenUnit {
    pub fn label(self) -> &'static str {
        match self {
            Self::LinesPerInch => "Lines/Inch",
            Self::LinesPerCentimeter => "Lines/Centimeter",
        }
    }
}

/// Draft prints at the screen frequency, Good at 1.5 ×, Best at 2 ×.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Draft,
    Good,
    Best,
}

impl Quality {
    pub const ALL: [Self; 3] = [Self::Draft, Self::Good, Self::Best];

    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Good => "Good",
            Self::Best => "Best",
        }
    }

    fn factor(self) -> f64 {
        match self {
            Self::Draft => 1.0,
            Self::Good => 1.5,
            Self::Best => 2.0,
        }
    }
}

/// The sheet's settings: the screen frequency as typed (in `unit`) and the
/// quality. Photoshop keeps the last ones confirmed with OK.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub screen: f64,
    pub unit: ScreenUnit,
    pub quality: Quality,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            screen: 133.0,
            unit: ScreenUnit::LinesPerInch,
            quality: Quality::Good,
        }
    }
}

impl Settings {
    /// The screen frequency in lines per inch.
    pub fn lines_per_inch(&self) -> f64 {
        match self.unit {
            ScreenUnit::LinesPerInch => self.screen,
            ScreenUnit::LinesPerCentimeter => self.screen * 2.54,
        }
    }

    /// The resolution (pixels per inch, whole, halves rounded up): 133
    /// lines/inch at Good is 200 ppi, at Best 266, at Draft 133.
    pub fn resolution(&self) -> f64 {
        (self.lines_per_inch() * self.quality.factor()).round()
    }

    /// Changing the unit converts the number (133 lines/inch is "52.362"
    /// lines/centimeter, which back in inches is "132.999").
    pub fn set_unit(&mut self, unit: ScreenUnit) {
        if unit != self.unit {
            self.screen = match unit {
                ScreenUnit::LinesPerCentimeter => self.screen / 2.54,
                ScreenUnit::LinesPerInch => self.screen * 2.54,
            };
            self.screen = (self.screen * 1000.0).round() / 1000.0;
            self.unit = unit;
        }
    }
}

static LAST: Mutex<Option<Settings>> = Mutex::new(None);

/// The settings the sheet opens with: the last confirmed ones.
pub fn last() -> Settings {
    LAST.lock().ok().and_then(|s| *s).unwrap_or_default()
}

fn remember(settings: Settings) {
    if let Ok(mut s) = LAST.lock() {
        *s = Some(settings);
    }
}

pub enum Outcome {
    Open,
    Cancel,
    /// The resolution to use, in pixels per inch.
    Apply(f64),
}

pub struct AutoResolutionDialog {
    pub settings: Settings,
    screen_text: String,
    first_frame: bool,
}

impl Default for AutoResolutionDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoResolutionDialog {
    pub fn new() -> Self {
        let settings = last();
        Self {
            screen_text: number(settings.screen),
            settings,
            first_frame: true,
        }
    }

    /// Shows the sheet over the Image Size window whose corner is `corner`.
    pub fn show(&mut self, ctx: &egui::Context, corner: Pos2) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("auto-resolution"))
            .area(
                egui::Modal::default_area(egui::Id::new("auto-resolution-area"))
                    .anchor(Align2::LEFT_TOP, (corner + OFFSET).to_vec2()),
            )
            .frame(egui::Frame::NONE)
            .backdrop_color(egui::Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        self.first_frame = false;
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Auto Resolution", theme::dialog_bold(pt(13.0)));

        appkit::text(
            ui,
            at(61.5, 48.5),
            Align2::RIGHT_CENTER,
            "Screen:",
            appkit::TEXT,
        );
        let field = appkit::field(
            ui,
            r(66.5, 39.0, 133.5, 58.0),
            &mut self.screen_text,
            "auto-resolution-screen",
            (1.0, 9999.0),
            1.0,
            0,
            self.first_frame,
        );
        if field.changed()
            && let Ok(v) = self.screen_text.trim().parse::<f64>()
        {
            self.settings.screen = v;
        }
        let units = [ScreenUnit::LinesPerInch, ScreenUnit::LinesPerCentimeter];
        let chosen = units
            .iter()
            .position(|&u| u == self.settings.unit)
            .unwrap_or(0);
        if let Some(u) = appkit::popup(
            ui,
            r(141.5, 38.0, 262.5, 59.0),
            "auto-resolution-unit",
            self.settings.unit.label(),
            &appkit::choices(units.iter().map(|u| u.label()), chosen),
        )
        .map(|k| units[k])
        {
            self.settings.set_unit(u);
            self.screen_text = number(self.settings.screen);
        }

        // Quality, in a group box titled over its top edge
        let title = appkit::label(ui, at(30.5, 74.5), "Quality");
        appkit::group(
            ui.painter(),
            r(10.5, 75.5, 262.5, 124.5),
            (title.left() - pt(5.0), title.right() + pt(5.0)),
        );
        for (q, x) in Quality::ALL.into_iter().zip([27.5, 108.25, 190.75]) {
            let chosen = self.settings.quality == q;
            if appkit::radio_with(ui, at(x, 107.0), q.label(), chosen, (15.5, appkit::font())) {
                self.settings.quality = q;
            }
        }

        let valid = self
            .screen_text
            .trim()
            .parse::<f64>()
            .is_ok_and(|v| v > 0.0 && v.is_finite());
        let ok = appkit::button(ui, r(278.0, 38.5, 338.5, 64.5), "OK", true, valid);
        let cancel = appkit::button(ui, r(278.0, 73.5, 338.5, 99.5), "Cancel", false, true);
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
        if valid && (ok.clicked() || enter) {
            remember(self.settings);
            return Outcome::Apply(self.settings.resolution());
        }
        Outcome::Open
    }
}

/// Up to three decimals, without trailing zeros.
fn number(v: f64) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolutions_match_photoshop() {
        // Photoshop 2026: 133 lines/inch is 200 ppi at Good, 266 at Best;
        // 50 at Draft is 50
        let mut s = Settings::default();
        assert_eq!(s.resolution(), 200.0);
        s.quality = Quality::Best;
        assert_eq!(s.resolution(), 266.0);
        s.quality = Quality::Draft;
        s.screen = 50.0;
        assert_eq!(s.resolution(), 50.0);
    }

    #[test]
    fn units_convert_like_photoshop() {
        let mut s = Settings::default();
        s.set_unit(ScreenUnit::LinesPerCentimeter);
        assert_eq!(number(s.screen), "52.362");
        s.quality = Quality::Best;
        assert_eq!(s.resolution(), 266.0);
        s.set_unit(ScreenUnit::LinesPerInch);
        assert_eq!(number(s.screen), "132.999");
    }
}
