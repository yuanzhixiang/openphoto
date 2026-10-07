//! The dialog naming a new preset (Crop tool › New Crop Preset...), laid out
//! as Photoshop 2026's (448 × 110 pt): a Name field, OK and Cancel. Sizes
//! are in Photoshop points from its top-left corner.

use egui::{Align2, Key, Rect, Sense, Ui, vec2};

use super::{appkit, common};
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(448.0), pt(110.0));

pub enum Outcome {
    Open,
    Cancel,
    /// The name typed (not empty).
    Ok(String),
}

pub struct NewPresetDialog {
    pub title: &'static str,
    pub name: String,
    first_frame: bool,
}

impl NewPresetDialog {
    /// Opens titled `title`, with `name` suggested and selected.
    pub fn new(title: &'static str, name: String) -> Self {
        Self {
            title,
            name,
            first_frame: true,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new(("new-preset", self.title)))
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
        common::frame(ui, frame, self.title, theme::dialog_bold(pt(13.0)));
        appkit::text(
            ui,
            at(55.5, 47.5),
            Align2::RIGHT_CENTER,
            "Name:",
            appkit::TEXT,
        );
        ui.painter().rect(
            r(60.0, 38.0, 362.5, 57.0),
            0,
            appkit::FIELD,
            egui::Stroke::new(pt(1.0), egui::Color32::from_gray(0x5e)),
            egui::StrokeKind::Inside,
        );
        common::text_field(
            ui,
            r(60.0, 38.0, 362.5, 57.0),
            &mut self.name,
            "new-preset-name",
            appkit::font(),
            pt(4.0),
            self.first_frame,
        );
        let valid = !self.name.trim().is_empty();
        let ok = appkit::button(ui, r(377.5, 38.5, 438.5, 64.5), "OK", true, valid);
        let cancel = appkit::button(ui, r(377.5, 73.5, 438.5, 99.5), "Cancel", false, true);
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
        if valid && (ok.clicked() || enter) {
            return Outcome::Ok(self.name.trim().to_owned());
        }
        Outcome::Open
    }
}
