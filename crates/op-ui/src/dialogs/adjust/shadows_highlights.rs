//! Image › Adjustments › Shadows/Highlights... in its short layout, as
//! Photoshop 2026 opens it (a UXP dialog, 449 × 236 pt, measured from a 2x
//! capture): Shadows and Highlights with their Spectrum sliders and
//! fields, "Show more options", OK, Cancel, Load... and Save... down the
//! right side, and Preview. Show more options switches to the layout with
//! every setting (`filter_layout::SHADOWS_HIGHLIGHTS`).

use egui::{Align2, Rect, vec2};

use super::{AdjustDialog, Outcome};
use crate::dialogs::{common, preset_files, uxp};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(449.0), pt(236.0));

/// The settings' order (`Kind::params`): the two amounts, and Show More
/// Options last.
const SHADOWS: usize = 0;
const HIGHLIGHTS: usize = 3;
pub const MORE: usize = 10;

/// Labels right-aligned here; the track from 77.25 to 236.5 (its thumb
/// 83.75–230); the fields 245–308.
const LABEL_RIGHT: f32 = 68.5;
const TRACK: (f32, f32) = (77.25, 236.5);
const FIELD: (f32, f32) = (245.0, 308.0);

impl AdjustDialog {
    /// Whether Shadows/Highlights shows its short layout.
    pub(super) fn shadows_highlights_short(&self) -> bool {
        self.value(MORE) != Some(1.0)
    }

    pub(super) fn shadows_highlights_ui(&mut self, ui: &mut egui::Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(
            ui,
            frame,
            "Shadows/Highlights",
            theme::dialog_bold(pt(13.0)),
        );
        let painter = ui.painter().clone();
        // Row: the setting, its label, the field's top, the track's line
        for (k, (i, name, top, line)) in [
            (SHADOWS, "Shadows", 48.0, 59.75),
            (HIGHLIGHTS, "Highlights", 84.0, 95.75),
        ]
        .into_iter()
        .enumerate()
        {
            painter.text(
                at(LABEL_RIGHT, line + 0.5 + 0.75),
                Align2::RIGHT_CENTER,
                name,
                uxp::font(),
                uxp::TEXT,
            );
            let field = r(FIELD.0, top, FIELD.1, top + 24.0);
            // Photoshop's field holds the percent sign ("35%")
            if self.first_frame && !self.values[i].ends_with('%') {
                self.values[i].push('%');
            }
            uxp::number_box(
                ui,
                field,
                &mut self.values[i],
                ("sh-field", k),
                (0.0, 100.0),
                self.first_frame && k == 0,
            );
            // The percent sign after a typed number without one
            let text = self.values[i].clone();
            if !text.contains('%') {
                let shown = painter.layout_no_wrap(text, uxp::font(), uxp::TEXT);
                painter.text(
                    field.left_center() + vec2(pt(11.0) + shown.size().x, pt(0.75)),
                    Align2::LEFT_CENTER,
                    "%",
                    uxp::font(),
                    uxp::TEXT,
                );
            }
            let value = self.value(i).unwrap_or(0.0);
            let track = |_| uxp::TRACK;
            if let Some(v) = uxp::linear_slider(
                ui,
                ("sh-slider", k),
                at(TRACK.0, 0.0).x,
                at(TRACK.1, 0.0).x,
                at(0.0, line).y,
                value,
                (0.0, 100.0),
                &track,
                1.0,
            ) {
                self.values[i] = format!("{}%", v.round());
            }
        }

        // Show more options: the layout with every setting
        let mut more = false;
        uxp::checkbox(ui, at(20.0, 126.0), "Show more options", &mut more);
        if more {
            self.values[MORE] = "1".into();
        }

        // OK, Cancel, Load..., Save... (the column 20 pt from the right)
        let pressed = uxp::buttons(ui, frame, None, self.effect().is_some(), false);
        let bold = theme::uxp_bold(pt(12.0));
        let x = frame.right() - pt(129.0);
        let row = |i: f32| {
            Rect::from_min_size(
                egui::Pos2::new(x, frame.top() + pt(48.0 + 36.0 * i)),
                vec2(pt(109.0), pt(24.0)),
            )
        };
        let load = common::ps_button_with(ui, row(2.0), "Load...", false, true, bold.clone());
        let save = common::ps_button_with(ui, row(3.0), "Save...", false, true, bold);
        if load.clicked()
            && let Some((_, bytes)) = preset_files::SHADOWS_HIGHLIGHTS.load()
        {
            self.load_shadows_highlights(&bytes);
        }
        if save.clicked()
            && let Some(bytes) = self.shadows_highlights_bytes()
        {
            preset_files::SHADOWS_HIGHLIGHTS.save(&bytes);
        }
        uxp::preview(ui, at(320.0, 198.0), &mut self.preview);
        match pressed {
            Some(uxp::Button::Ok) => self.effect().map_or(Outcome::Open, Outcome::Apply),
            Some(uxp::Button::Cancel) => Outcome::Cancel,
            _ => Outcome::Open,
        }
    }

    /// Load... and Save...'s file: the ten settings (`preset_files`).
    fn shadows_highlights_bytes(&self) -> Option<Vec<u8>> {
        let mut v = [0f32; 10];
        for (i, x) in v.iter_mut().enumerate() {
            *x = self.value(i)?;
        }
        Some(preset_files::encode_shadows_highlights(v))
    }

    fn load_shadows_highlights(&mut self, bytes: &[u8]) {
        if let Some(v) = preset_files::decode_shadows_highlights(bytes) {
            for (i, x) in v.into_iter().enumerate() {
                let decimals = self.kind.params()[i].decimals;
                self.values[i] = format!("{x:.decimals$}");
            }
        }
    }
}
