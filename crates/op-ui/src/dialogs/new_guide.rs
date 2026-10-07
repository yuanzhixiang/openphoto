//! View > Guides > New Guide...: Orientation (Horizontal or Vertical),
//! Position and Color, laid out at the positions measured on Photoshop
//! 2026's "New guide" dialog (390 × 188 pt, a UXP dialog). Sizes are in
//! Photoshop points from the dialog's top-left corner.

use egui::{Align2, Color32, CornerRadius, Key, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use op_core::Guide;

use super::{common, uxp};
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(390.0), pt(188.0));
/// Photoshop's guide colors, Cyan first (the default).
const COLORS: [(&str, [u8; 3]); 9] = [
    ("Cyan", [0x85, 0xfb, 0xfd]),
    ("Light Blue", [0x4a, 0x9c, 0xff]),
    ("Light Red", [0xff, 0x6e, 0x6e]),
    ("Green", [0x4a, 0xff, 0x4a]),
    ("Medium Blue", [0x31, 0x31, 0xff]),
    ("Yellow", [0xff, 0xff, 0x4a]),
    ("Magenta", [0xff, 0x4a, 0xff]),
    ("Gray", [0x80, 0x80, 0x80]),
    ("Black", [0, 0, 0]),
];

pub enum Outcome {
    Open,
    Cancel,
    Apply(Guide),
}

pub struct NewGuideDialog {
    vertical: bool,
    position: String,
    /// Edit Selected Guides...: the guide being edited (its index).
    pub editing: Option<usize>,
    /// An index into [`COLORS`]; guides are drawn in one color for now.
    color: usize,
}

impl Default for NewGuideDialog {
    /// Photoshop starts with a horizontal guide at 0.
    fn default() -> Self {
        Self {
            vertical: false,
            position: "0 px".into(),
            editing: None,
            color: 0,
        }
    }
}

impl NewGuideDialog {
    /// View › Guides › Edit Selected Guides...: the dialog filled in with
    /// guide `index`, titled "Edit Guide"; its OK changes that guide.
    pub fn editing(guide: Guide, index: usize) -> Self {
        Self {
            vertical: guide.vertical,
            position: format!("{} px", (guide.position * 32.0).round() / 32.0),
            editing: Some(index),
            color: 0,
        }
    }

    fn guide(&self) -> Option<Guide> {
        let position: f32 = self
            .position
            .trim()
            .trim_end_matches("px")
            .trim()
            .parse()
            .ok()?;
        (position.abs() <= 30_000.0).then_some(Guide {
            vertical: self.vertical,
            position,
        })
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("new-guide"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let title = if self.editing.is_some() {
            "Edit Guide"
        } else {
            "New guide"
        };
        common::frame(ui, frame, title, theme::dialog_bold(pt(13.0)));
        ui.painter().text(
            at(20.0, 57.0),
            Align2::LEFT_CENTER,
            "Orientation",
            theme::uxp_bold(pt(12.0)),
            uxp::TEXT,
        );
        for (vertical, label, x) in [(false, "Horizontal", 26.0), (true, "Vertical", 108.0)] {
            if uxp::radio(ui, at(x, 84.0), label, self.vertical == vertical) {
                self.vertical = vertical;
            }
        }
        let right_label = |ui: &Ui, cy: f32, text: &str| {
            ui.painter().text(
                at(59.5, cy),
                Align2::RIGHT_CENTER,
                text,
                uxp::font(),
                uxp::TEXT,
            );
        };
        right_label(ui, 120.5, "Position");
        common::text_field(
            ui,
            r(68.0, 108.0, 161.0, 132.0),
            &mut self.position,
            "guide-position",
            uxp::font(),
            pt(11.5),
            false,
        );

        right_label(ui, 156.5, "Color");
        let (name, [cr, cg, cb]) = COLORS[self.color];
        let mut chosen = self.color;
        common::ps_dropdown(
            ui,
            r(68.0, 144.0, 228.0, 168.0),
            "guide-color",
            |painter, rect| {
                let swatch = Rect::from_min_size(
                    rect.min + vec2(pt(7.0), pt(4.0)),
                    vec2(pt(16.0), pt(16.0)),
                );
                painter.rect(
                    swatch,
                    CornerRadius::same(pt(2.0) as u8),
                    Color32::from_rgb(cr, cg, cb),
                    Stroke::new(pt(1.0), Color32::from_gray(0xa9)),
                    StrokeKind::Inside,
                );
                painter.text(
                    rect.left_center() + vec2(pt(30.5), pt(0.75)),
                    Align2::LEFT_CENTER,
                    name,
                    uxp::font(),
                    uxp::TEXT,
                );
            },
            |ui| {
                for (k, (label, _)) in COLORS.iter().enumerate() {
                    ui.selectable_value(&mut chosen, k, *label);
                }
            },
        );
        self.color = chosen;
        // The custom color's swatch
        ui.painter().rect(
            r(240.0, 144.0, 288.0, 168.0),
            CornerRadius::same(pt(3.0) as u8),
            Color32::WHITE,
            Stroke::new(pt(1.0), Color32::from_gray(0xa9)),
            StrokeKind::Inside,
        );

        // No field has the focus: OK does, as in Photoshop
        let guide = self.guide();
        let ok = common::ps_focused_button(
            ui,
            r(300.0, 48.0, 370.0, 72.0),
            "OK",
            theme::uxp_bold(pt(12.0)),
        );
        let cancel = common::ps_button(
            ui,
            r(300.0, 84.0, 370.0, 108.0),
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
            && let Some(guide) = guide
        {
            return Outcome::Apply(guide);
        }
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_accepts_a_px_suffix() {
        let mut d = NewGuideDialog {
            position: "120 px".into(),
            vertical: true,
            ..Default::default()
        };
        assert_eq!(
            d.guide(),
            Some(Guide {
                vertical: true,
                position: 120.0
            })
        );
        d.position = "abc".into();
        assert_eq!(d.guide(), None);
    }
}
