//! Filter › Stylize › Oil Paint..., as Photoshop 2026 lays it out (324 ×
//! 651 pt, measured from a 2x capture): the preview at the top left, OK,
//! Cancel and Preview beside it, then the Brush group's four sliders and
//! the Lighting group (its checkbox in the title) with the angle dial and
//! Shine.

use egui::{Rect, vec2};

use super::filter_layout::Scale;
use super::legacy::Row;
use super::{AdjustDialog, Outcome};
use crate::dialogs::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(324.0), pt(651.0));

/// The settings' order (`Kind::params`).
pub const BRISTLE: usize = 3;
pub const LIGHTING: usize = 4;
pub const ANGLE: usize = 5;
pub const SHINE: usize = 6;

/// A row on line `y`: the label right-aligned at 104, the field 110–169,
/// the track 31–293 17.5 pt below the line.
fn row(label: &'static str, y: f32) -> Row {
    Row {
        label,
        label_right: 104.0,
        field: [110.0, y - 8.5, 169.0, y + 8.5],
        unit: None,
        track: (31.0, 293.0, y + 17.5),
        scale: Scale::Linear,
    }
}

impl AdjustDialog {
    pub(super) fn oil_paint_ui(&mut self, ui: &mut egui::Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Oil Paint", theme::dialog_bold(pt(13.0)));
        self.legacy_preview(ui, frame, 261.5);
        let buttons = self.legacy_buttons(ui, frame, 232.0);
        self.legacy_preview_check(ui, frame, (233.5, 119.5));

        // Brush
        let brush = r(11.0, 296.0, 299.0, 506.0);
        let title = |ui: &egui::Ui, x: f32, rect: Rect, gap: f32, text: &str| {
            appkit::group(ui.painter(), rect, (at(x - 5.0, 0.0).x, at(x + gap, 0.0).x));
            if !text.is_empty() {
                appkit::label(ui, egui::Pos2::new(at(x, 0.0).x, rect.top()), text);
            }
        };
        title(ui, 30.5, brush, 40.0, "Brush");
        for (k, (label, y)) in [
            ("Stylization:", 319.5),
            ("Cleanliness:", 368.5),
            ("Scale:", 417.5),
            ("Bristle Detail:", 466.5),
        ]
        .into_iter()
        .enumerate()
        {
            self.legacy_row(ui, frame, k, &row(label, y), true, k == 0);
        }

        // Lighting: its checkbox in the group's title turns the rest on
        let lighting = r(11.0, 532.5, 299.0, 640.0);
        title(ui, 26.0, lighting, 66.0, "");
        let mut on = self.value(LIGHTING) == Some(1.0);
        appkit::checkbox_with(ui, at(30.0, 526.5), (12.0, 10.0), "Lighting", &mut on, true);
        self.values[LIGHTING] = (on as u8).to_string();
        let angle = Row {
            field: [110.0, 556.5, 154.0, 573.5],
            ..row("Angle:", 565.0)
        };
        if on {
            let p = &self.kind.params()[ANGLE];
            appkit::field(
                ui,
                Rect::from_min_max(
                    at(angle.field[0], angle.field[1]),
                    at(angle.field[2], angle.field[3]),
                ),
                &mut self.values[ANGLE],
                ("oil-angle", ANGLE),
                (p.min, p.max),
                1.0,
                0,
                false,
            );
            self.dial(ui, ANGLE, at(191.0, 565.0), pt(18.0), false);
        } else {
            let f = Rect::from_min_max(at(110.0, 556.5), at(154.0, 573.5));
            ui.painter().rect(
                f,
                0,
                egui::Color32::from_gray(0x4d),
                egui::Stroke::new(pt(1.0), egui::Color32::from_gray(0x5d)),
                egui::StrokeKind::Inside,
            );
            appkit::text(
                ui,
                f.left_center() + vec2(pt(4.0), 0.0),
                egui::Align2::LEFT_CENTER,
                &self.values[ANGLE],
                appkit::TEXT_OFF,
            );
            ui.painter().circle_stroke(
                at(191.0, 565.0),
                pt(18.0),
                egui::Stroke::new(pt(1.0), egui::Color32::from_gray(0x80)),
            );
        }
        let ink = if on { appkit::TEXT } else { appkit::TEXT_OFF };
        appkit::text(
            ui,
            at(104.0, 565.0),
            egui::Align2::RIGHT_CENTER,
            "Angle:",
            ink,
        );
        appkit::text(ui, at(160.0, 562.0), egui::Align2::LEFT_CENTER, "°", ink);
        self.legacy_row(ui, frame, SHINE, &row("Shine:", 599.5), on, false);
        self.legacy_outcome(ui, buttons)
    }
}
