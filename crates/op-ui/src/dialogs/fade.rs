//! Edit › Fade (Shift+Cmd+F): the last filter, adjustment, fill or stroke
//! blended back toward the pixels before it. Laid out at the positions
//! measured on Photoshop 2026's dialog (291 × 144 pt, a classic AppKit
//! dialog); sizes are Photoshop points from the dialog's top-left corner.

use egui::{Align2, Color32, Key, Rect, Sense, Ui, vec2};
use op_core::BlendMode;

use super::{appkit, common};
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(291.0), pt(144.0));
/// The slider: its track, and where the thumb's tip runs from 0% to 100%.
const TRACK: (f32, f32, f32) = (18.5, 185.0, 68.0);
const THUMB: (f32, f32) = (21.5, 178.25);

pub enum Outcome {
    Open,
    Cancel,
    Apply,
}

pub struct FadeDialog {
    opacity: String,
    pub mode: BlendMode,
    pub preview: bool,
    first_frame: bool,
}

impl Default for FadeDialog {
    fn default() -> Self {
        Self {
            opacity: "100".into(),
            mode: BlendMode::Normal,
            preview: true,
            first_frame: true,
        }
    }
}

impl FadeDialog {
    /// The Opacity as 0–1, `None` while the field isn't a percentage.
    pub fn opacity(&self) -> Option<f32> {
        let v: f32 = self.opacity.trim().parse().ok()?;
        (0.0..=100.0).contains(&v).then_some(v / 100.0)
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("fade-dialog"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        self.first_frame = false;
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Fade", theme::dialog_bold(pt(13.0)));
        appkit::text(
            ui,
            at(10.0, 47.5),
            Align2::LEFT_CENTER,
            "Opacity:",
            appkit::TEXT,
        );
        appkit::field(
            ui,
            r(127.0, 39.5, 169.0, 56.5),
            &mut self.opacity,
            "fade-opacity",
            (0.0, 100.0),
            1.0,
            0,
            self.first_frame,
        );
        appkit::text(ui, at(174.0, 47.5), Align2::LEFT_CENTER, "%", appkit::TEXT);

        // The slider: dragging or clicking along it sets the opacity
        let (x0, x1, y) = TRACK;
        let hit = ui.interact(
            r(x0 - 4.0, y - 6.0, x1 + 4.0, y + 10.0),
            ui.id().with("fade-slider"),
            Sense::click_and_drag(),
        );
        if (hit.dragged() || hit.clicked())
            && let Some(p) = hit.interact_pointer_pos()
        {
            let (a, b) = (at(THUMB.0, 0.0).x, at(THUMB.1, 0.0).x);
            let t = ((p.x - a) / (b - a)).clamp(0.0, 1.0);
            self.opacity = format!("{:.0}", t * 100.0);
        }
        ui.painter()
            .rect_filled(r(x0, y, x1, y + 3.0), pt(1.5), Color32::from_gray(0x75));
        let t = self.opacity().unwrap_or(1.0);
        let cx = THUMB.0 + (THUMB.1 - THUMB.0) * t;
        appkit::pin(ui.painter(), at(cx, 65.5), appkit::Pin::White);

        appkit::text(
            ui,
            at(10.0, 95.5),
            Align2::LEFT_CENTER,
            "Mode:",
            appkit::TEXT,
        );
        let (entries, modes) = appkit::blend_modes(self.mode);
        if let Some(k) = appkit::popup(
            ui,
            r(51.0, 85.0, 184.5, 106.0),
            "fade-mode",
            self.mode.label(),
            &entries,
        ) {
            self.mode = modes[k].unwrap_or(self.mode);
        }

        let ok = appkit::button(
            ui,
            r(201.0, 38.5, 280.5, 64.5),
            "OK",
            true,
            self.opacity().is_some(),
        );
        let cancel = appkit::button(ui, r(201.0, 73.5, 280.5, 99.5), "Cancel", false, true);
        appkit::checkbox_with(
            ui,
            at(201.0, 118.5),
            (11.5, 10.0),
            "Preview",
            &mut self.preview,
            true,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter) && self.opacity().is_some() {
            return Outcome::Apply;
        }
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_must_be_a_percentage() {
        let mut d = FadeDialog::default();
        assert_eq!(d.opacity(), Some(1.0));
        d.opacity = "40".into();
        assert_eq!(d.opacity(), Some(0.4));
        d.opacity = "101".into();
        assert_eq!(d.opacity(), None);
    }
}
