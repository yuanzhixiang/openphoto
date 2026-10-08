//! Image › Adjustments › Color Lookup..., as Photoshop 2026 lays it out
//! (a UXP dialog, 519 × 186 pt, measured from a 2x capture): the 3DLUT
//! File, Abstract and Device Link radios with their pop-ups, Dither, and
//! OK, Cancel and Preview on the right.

use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Stroke, StrokeKind, vec2};

use super::{AdjustDialog, Outcome};
use crate::dialogs::{common, uxp};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(519.0), pt(186.0));

/// The settings' order (`Kind::params`).
pub const FILE: usize = 0;
pub const SOURCE: usize = 1;
pub const DITHER: usize = 2;

/// The radios' rows (their centers' y) and the pop-ups' tops.
const ROWS: [(f32, f32, &str, &str); 3] = [
    (59.75, 48.0, "3DLUT File", ""),
    (88.75, 77.0, "Abstract", "Load Abstract Profile..."),
    (117.75, 106.0, "Device Link", "Load DeviceLink Profile..."),
];

impl AdjustDialog {
    pub(super) fn color_lookup_ui(&mut self, ui: &mut egui::Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Color Lookup", theme::dialog_bold(pt(13.0)));
        let source = self.value(SOURCE).unwrap_or(0.0) as usize;
        let labels = self
            .extra
            .labels(self.kind, FILE, None)
            .unwrap_or_else(|| vec!["Load 3D LUT...".to_owned()]);
        let file = self.value(FILE).unwrap_or(0.0) as usize;
        let mut picked = None;
        for (k, (cy, top, name, load)) in ROWS.into_iter().enumerate() {
            // Abstract and Device Link apply ICC profiles, which need color
            // management: they're shown off
            let usable = k == 0;
            if usable {
                if uxp::radio(ui, at(25.75, cy), name, source == k) {
                    self.values[SOURCE] = k.to_string();
                }
            } else {
                dimmed_radio(ui, at(25.75, cy), name);
            }
            let rect = r(110.0, top, 377.5, top + 24.0);
            if usable {
                let shown = labels.get(file).cloned().unwrap_or_default();
                common::ps_dropdown(
                    ui,
                    rect,
                    "lookup-file",
                    |p, rect| {
                        p.text(
                            rect.left_center() + vec2(pt(10.0), pt(0.75)),
                            Align2::LEFT_CENTER,
                            &shown,
                            uxp::font(),
                            uxp::TEXT,
                        );
                    },
                    |ui| {
                        for (i, label) in labels.iter().enumerate() {
                            if ui.selectable_label(i == file && i > 0, label).clicked() {
                                picked = Some(i);
                            }
                            if i == 0 && labels.len() > 1 {
                                ui.separator();
                            }
                        }
                    },
                );
            } else {
                ui.painter().rect(
                    rect,
                    CornerRadius::same(pt(3.0) as u8),
                    Color32::TRANSPARENT,
                    Stroke::new(pt(1.0), Color32::from_gray(0x5e)),
                    StrokeKind::Inside,
                );
                ui.painter().text(
                    rect.left_center() + vec2(pt(10.0), pt(0.75)),
                    Align2::LEFT_CENTER,
                    load,
                    uxp::font(),
                    OFF,
                );
                crate::ps_icons::paint(
                    ui.painter(),
                    Pos2::new(rect.right() - pt(13.5), rect.center().y),
                    crate::ps_icons::Icon::DialogChevron,
                    OFF,
                    crate::theme::color::PANEL,
                );
            }
        }
        if let Some(i) = picked {
            self.values[SOURCE] = "0".into();
            self.pick(FILE, i);
        }
        let mut dither = self.value(DITHER) != Some(0.0);
        uxp::checkbox(ui, at(20.0, 148.0), "Dither", &mut dither);
        self.values[DITHER] = (dither as u8).to_string();

        // OK is always on (and focused), as in Photoshop: with nothing to
        // look up it just closes
        let pressed = uxp::buttons(ui, frame, None, true, true);
        uxp::preview(ui, at(390.0, 127.5), &mut self.preview);
        match pressed {
            Some(uxp::Button::Ok) => self.effect().map_or(Outcome::Cancel, Outcome::Apply),
            Some(uxp::Button::Cancel) => Outcome::Cancel,
            _ => Outcome::Open,
        }
    }
}

/// Text of a control that's off.
const OFF: Color32 = Color32::from_gray(0x80);

/// A radio button drawn off: a dim ring and dim label.
fn dimmed_radio(ui: &egui::Ui, center: Pos2, label: &str) {
    ui.painter().circle_stroke(
        center,
        pt(6.0),
        Stroke::new(pt(1.0), Color32::from_gray(0x6a)),
    );
    ui.painter().text(
        center + vec2(pt(15.5), pt(0.75)),
        Align2::LEFT_CENTER,
        label,
        uxp::font(),
        OFF,
    );
}
