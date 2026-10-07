//! View › Guides › New Guide Layout...: columns, rows and margins turned
//! into guides (`op_core::guide_layout`), previewed on the canvas. Sizes are
//! points from the dialog's top-left corner; the layout follows Photoshop's
//! arrangement and has not been measured against Photoshop 2026 yet.

use egui::{Align2, Key, Rect, Sense, Ui, vec2};
use op_core::guide_layout::GuideLayout;

use super::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(420.0), pt(330.0));
/// The presets: Custom, then column counts with 20 px gutters.
const PRESETS: [(&str, u32); 4] = [
    ("Custom", 0),
    ("8 Column", 8),
    ("12 Column", 12),
    ("4 Column", 4),
];

pub enum Outcome {
    Open,
    Cancel,
    /// The layout and whether the existing guides go first.
    Ok(GuideLayout, bool),
}

pub struct GuideLayoutDialog {
    columns: bool,
    rows: bool,
    margin: bool,
    pub center_columns: bool,
    pub clear: bool,
    pub preview: bool,
    /// Columns: number, width, gutter; rows: number, height, gutter;
    /// margins: top, left, bottom, right (as typed).
    fields: [String; 10],
    first_frame: bool,
}

impl Default for GuideLayoutDialog {
    /// Photoshop's: 8 columns with 20 px gutters, Preview on.
    fn default() -> Self {
        let mut fields: [String; 10] = Default::default();
        fields[0] = "8".into();
        fields[2] = "20 px".into();
        Self {
            columns: true,
            rows: false,
            margin: false,
            center_columns: false,
            clear: false,
            preview: true,
            fields,
            first_frame: true,
        }
    }
}

fn length(text: &str) -> Option<f32> {
    let t = text.trim().trim_end_matches("px").trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<f32>()
        .ok()
        .filter(|v| (0.0..=30_000.0).contains(v))
}

impl GuideLayoutDialog {
    /// The layout as set, if every field in use reads.
    pub fn layout(&self) -> Option<GuideLayout> {
        let f = &self.fields;
        let count = |t: &str| -> Option<u32> {
            let n: u32 = t.trim().parse().ok()?;
            (1..=1000).contains(&n).then_some(n)
        };
        let gutter = |t: &str| {
            if t.trim().is_empty() {
                Some(0.0)
            } else {
                length(t)
            }
        };
        let columns = if self.columns {
            Some((count(&f[0])?, length(&f[1]), gutter(&f[2])?))
        } else {
            None
        };
        let rows = if self.rows {
            Some((count(&f[3])?, length(&f[4]), gutter(&f[5])?))
        } else {
            None
        };
        let margin = if self.margin {
            let m = |t: &str| {
                if t.trim().is_empty() {
                    Some(0.0)
                } else {
                    length(t)
                }
            };
            Some([m(&f[6])?, m(&f[7])?, m(&f[8])?, m(&f[9])?])
        } else {
            None
        };
        Some(GuideLayout {
            columns,
            rows,
            margin,
            center_columns: self.center_columns,
        })
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("guide-layout"))
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
        common::frame(ui, frame, "New Guide Layout", theme::dialog_bold(pt(13.0)));
        let label = |ui: &Ui, x: f32, y: f32, text: &str| {
            appkit::text(ui, at(x, y), Align2::RIGHT_CENTER, text, appkit::TEXT);
        };

        // Preset and Target
        label(ui, 64.0, 56.5, "Preset:");
        let chosen = PRESETS
            .iter()
            .position(|(_, n)| {
                *n > 0
                    && self.columns
                    && self.fields[0].trim() == n.to_string()
                    && self.fields[1].trim().is_empty()
                    && length(&self.fields[2]) == Some(20.0)
            })
            .unwrap_or(0);
        if let Some(k) = appkit::popup(
            ui,
            r(70.0, 46.0, 250.0, 67.0),
            "guide-layout-preset",
            PRESETS[chosen].0,
            &appkit::choices(PRESETS.iter().map(|(n, _)| *n), chosen),
        ) && PRESETS[k].1 > 0
        {
            self.columns = true;
            self.fields[0] = PRESETS[k].1.to_string();
            self.fields[1].clear();
            self.fields[2] = "20 px".into();
        }
        label(ui, 64.0, 86.5, "Target:");
        appkit::popup(
            ui,
            r(70.0, 76.0, 250.0, 97.0),
            "guide-layout-target",
            "Canvas",
            &appkit::choices(["Canvas"], 0),
        );

        // Columns and Rows, each three fields
        let mut field = |ui: &mut Ui, k: usize, x: f32, y: f32, on: bool, first: bool| {
            if on {
                appkit::field(
                    ui,
                    r(x, y - 9.5, x + 60.0, y + 9.5),
                    &mut self.fields[k],
                    ("guide-layout-field", k),
                    (0.0, 30_000.0),
                    1.0,
                    0,
                    first,
                );
            } else {
                ui.painter().rect_stroke(
                    r(x, y - 9.5, x + 60.0, y + 9.5),
                    0,
                    egui::Stroke::new(pt(1.0), egui::Color32::from_gray(0x5d)),
                    egui::StrokeKind::Inside,
                );
            }
        };
        appkit::checkbox(ui, at(20.0, 112.0), "Columns", &mut self.columns);
        appkit::checkbox(ui, at(200.0, 112.0), "Rows", &mut self.rows);
        for (k, name) in ["Number", "Width", "Gutter"].into_iter().enumerate() {
            let y = 145.0 + k as f32 * 27.0;
            label(ui, 84.0, y, name);
            field(ui, k, 90.0, y, self.columns, self.first_frame && k == 0);
        }
        for (k, name) in ["Number", "Height", "Gutter"].into_iter().enumerate() {
            let y = 145.0 + k as f32 * 27.0;
            label(ui, 264.0, y, name);
            field(ui, 3 + k, 270.0, y, self.rows, false);
        }
        // Margins
        appkit::checkbox(ui, at(20.0, 222.0), "Margin", &mut self.margin);
        for (k, name) in ["Top", "Left", "Bottom", "Right"].into_iter().enumerate() {
            let x = 52.0 + k as f32 * 90.0;
            label(ui, x - 6.0, 255.0, name);
            field(ui, 6 + k, x, 255.0, self.margin, false);
        }
        appkit::checkbox(
            ui,
            at(20.0, 282.0),
            "Center Columns",
            &mut self.center_columns,
        );
        appkit::checkbox(
            ui,
            at(20.0, 306.0),
            "Clear Existing Guides",
            &mut self.clear,
        );

        let valid = self.layout().is_some();
        let ok = appkit::button(ui, r(330.0, 38.5, 400.0, 64.5), "OK", true, valid);
        let cancel = appkit::button(ui, r(330.0, 73.5, 400.0, 99.5), "Cancel", false, true);
        appkit::checkbox(ui, at(330.0, 112.0), "Preview", &mut self.preview);
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        let enter = !typing && ui.input(|i| i.key_pressed(Key::Enter));
        if let Some(layout) = self.layout()
            && (ok.clicked() || enter)
        {
            return Outcome::Ok(layout, self.clear);
        }
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_make_the_layout() {
        let mut d = GuideLayoutDialog::default();
        let layout = d.layout().unwrap();
        assert_eq!(layout.columns, Some((8, None, 20.0)));
        assert_eq!(layout.rows, None);
        d.rows = true;
        assert!(d.layout().is_none());
        d.fields[3] = "3".into();
        assert_eq!(d.layout().unwrap().rows, Some((3, None, 0.0)));
        d.fields[0] = "x".into();
        assert!(d.layout().is_none());
    }
}
