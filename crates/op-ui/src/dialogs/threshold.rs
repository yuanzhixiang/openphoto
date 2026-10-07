//! Image > Adjustments > Threshold...: Photoshop 2026's classic dialog,
//! 381 × 191 pt: the level and the luminosity histogram with a pin under
//! it. Sizes are Photoshop points from the dialog's top-left corner.

use egui::{Align2, Color32, Pos2, Rect, Sense, Ui, vec2};
use op_core::adjust::Adjustment;

use super::appkit::{self, Pin};
use super::uxp::Button;
use crate::theme::pt;

pub const SIZE: egui::Vec2 = vec2(pt(381.0), pt(191.0));
const RANGE: (f32, f32) = (1.0, 255.0);
const HISTOGRAM: [f32; 4] = [15.0, 66.0, 270.0, 166.0];
/// Where levels 0 and 255 sit under the histogram.
const PIN_X: (f32, f32) = (14.25, 268.75);

#[derive(Clone)]
pub struct Dialog {
    pub level: String,
    histogram: [u64; 256],
}

impl Dialog {
    pub fn new(histogram: [u64; 256]) -> Self {
        Self {
            level: "128".into(),
            histogram,
        }
    }

    fn parse(&self) -> Option<u8> {
        let v: f32 = self.level.trim().parse().ok()?;
        (RANGE.0..=RANGE.1).contains(&v).then_some(v.round() as u8)
    }

    pub fn adjustment(&self) -> Option<Adjustment> {
        self.parse().map(Adjustment::Threshold)
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> Option<Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();

        appkit::text(
            ui,
            at(171.5, 47.25),
            Align2::RIGHT_CENTER,
            "Threshold Level:",
            appkit::TEXT,
        );
        appkit::field(
            ui,
            r(177.0, 38.0, 213.5, 57.0),
            &mut self.level,
            "threshold-level",
            RANGE,
            1.0,
            0,
            first_frame,
        );

        let hist = r(HISTOGRAM[0], HISTOGRAM[1], HISTOGRAM[2], HISTOGRAM[3]);
        painter.rect_filled(hist, 0, appkit::FIELD);
        let max = self.histogram.iter().copied().max().unwrap_or(0).max(1) as f32;
        let bar = hist.width() / 256.0;
        for (i, &n) in self.histogram.iter().enumerate() {
            if n > 0 {
                let h = n as f32 / max * hist.height();
                let x = hist.left() + i as f32 * bar;
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(x, hist.bottom() - h),
                        Pos2::new(x + bar, hist.bottom()),
                    ),
                    0,
                    Color32::from_gray(0xd0),
                );
            }
        }
        // The pin: drag it (or press along its row) to set the level
        let (x0, x1) = (at(PIN_X.0, 0.0).x, at(PIN_X.1, 0.0).x);
        let tip = at(0.0, 167.5).y;
        let row = Rect::from_min_max(
            Pos2::new(x0 - pt(6.0), tip),
            Pos2::new(x1 + pt(6.0), tip + pt(11.0)),
        );
        let response = ui.interact(row, ui.id().with("threshold-pin"), Sense::click_and_drag());
        if (response.dragged() || response.clicked())
            && let Some(p) = response.interact_pointer_pos()
        {
            let v = ((p.x - x0) / (x1 - x0) * 255.0)
                .round()
                .clamp(RANGE.0, RANGE.1);
            self.level = format!("{v}");
        }
        let level = self.parse().unwrap_or(128) as f32;
        appkit::pin(
            ui.painter(),
            Pos2::new(x0 + (x1 - x0) * level / 255.0, tip),
            Pin::White,
        );

        let ok = appkit::button(
            ui,
            r(290.0, 38.5, 370.5, 64.5),
            "OK",
            true,
            self.adjustment().is_some(),
        );
        let cancel = appkit::button(ui, r(290.0, 73.5, 370.5, 99.5), "Cancel", false, true);
        appkit::checkbox(ui, at(290.0, 118.0), "Preview", preview);
        if cancel.clicked() {
            return Some(Button::Cancel);
        }
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
        ((ok.clicked() || enter) && self.adjustment().is_some()).then_some(Button::Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_range() {
        let mut d = Dialog::new([0; 256]);
        assert_eq!(d.adjustment(), Some(Adjustment::Threshold(128)));
        d.level = "0".into();
        assert_eq!(d.adjustment(), None);
        d.level = "255".into();
        assert_eq!(d.adjustment(), Some(Adjustment::Threshold(255)));
    }
}
