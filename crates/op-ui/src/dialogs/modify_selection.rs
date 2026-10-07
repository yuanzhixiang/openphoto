//! Select > Modify: Border..., Smooth..., Expand..., Contract... and
//! Feather... (Shift+F6). One value in pixels each, plus "Apply effect at
//! canvas bounds", laid out at the positions measured on Photoshop 2026's
//! dialogs (295 × 128 pt, UXP). Sizes are in Photoshop points from the
//! dialog's top-left corner.

use egui::{Color32, Key, Rect, Sense, Ui, vec2};
use op_core::Selection;

use super::{common, uxp};
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(295.0), pt(128.0));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModifyKind {
    Border,
    Smooth,
    Expand,
    Contract,
    Feather,
}

impl ModifyKind {
    fn title(self) -> &'static str {
        match self {
            Self::Border => "Border Selection",
            Self::Smooth => "Smooth Selection",
            Self::Expand => "Expand Selection",
            Self::Contract => "Contract Selection",
            Self::Feather => "Feather Selection",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Border => "Width",
            Self::Smooth => "Sample Radius",
            Self::Expand => "Expand By",
            Self::Contract => "Contract By",
            Self::Feather => "Feather Radius",
        }
    }

    /// The history name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Border => "Border",
            Self::Smooth => "Smooth",
            Self::Expand => "Expand",
            Self::Contract => "Contract",
            Self::Feather => "Feather",
        }
    }

    fn range(self) -> (f32, f32) {
        match self {
            Self::Border => (1.0, 200.0),
            Self::Smooth | Self::Expand | Self::Contract => (1.0, 500.0),
            Self::Feather => (0.1, 1000.0),
        }
    }

    /// The modified selection.
    pub fn apply(self, s: &Selection, value: f32, at_bounds: bool) -> Selection {
        match self {
            Self::Border => s.border(value),
            Self::Smooth => s.smooth(value.round() as u32),
            Self::Expand => s.expand(value.round()),
            Self::Contract => s.contract(value.round(), at_bounds),
            Self::Feather => s.feather(value),
        }
    }
}

pub enum Outcome {
    Open,
    Cancel,
    Apply { value: f32, at_bounds: bool },
}

pub struct ModifyDialog {
    pub kind: ModifyKind,
    value: String,
    at_bounds: bool,
    first_frame: bool,
}

impl ModifyDialog {
    pub fn new(kind: ModifyKind) -> Self {
        Self {
            kind,
            value: "1".into(),
            at_bounds: false,
            first_frame: true,
        }
    }

    fn value(&self) -> Option<f32> {
        let v: f32 = self.value.trim().parse().ok()?;
        let (lo, hi) = self.kind.range();
        (lo..=hi).contains(&v).then_some(v)
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("modify-selection"))
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
        common::frame(ui, frame, self.kind.title(), theme::dialog_bold(pt(13.0)));
        // The field follows its label, 9 pt after it
        let label = uxp::label(ui, at(20.5, 60.0), self.kind.label());
        let field_x = label.right() + pt(9.0);
        let field = Rect::from_min_max(
            egui::pos2(field_x, at(0.0, 48.0).y),
            egui::pos2(field_x + pt(36.0), at(0.0, 72.0).y),
        );
        common::text_field(
            ui,
            field,
            &mut self.value,
            "modify-value",
            uxp::font(),
            pt(11.5),
            self.first_frame,
        );
        uxp::label(ui, field.right_center() + vec2(pt(5.5), 0.0), "pixels");
        common::ps_checkbox(
            ui,
            at(20.0, 90.0),
            "Apply effect at canvas bounds",
            &mut self.at_bounds,
            true,
        );

        let value = self.value();
        let ok = common::ps_button(
            ui,
            r(205.0, 48.0, 275.0, 72.0),
            "OK",
            true,
            value.is_some(),
            true,
        );
        let cancel = common::ps_button(
            ui,
            r(205.0, 84.0, 275.0, 108.0),
            "Cancel",
            false,
            true,
            true,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter)
            && let Some(value) = value
        {
            return Outcome::Apply {
                value,
                at_bounds: self.at_bounds,
            };
        }
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        let mut d = ModifyDialog::new(ModifyKind::Border);
        assert_eq!(d.value(), Some(1.0));
        d.value = "201".into();
        assert_eq!(d.value(), None);
        let mut f = ModifyDialog::new(ModifyKind::Feather);
        f.value = "0.5".into();
        assert_eq!(f.value(), Some(0.5));
    }
}
