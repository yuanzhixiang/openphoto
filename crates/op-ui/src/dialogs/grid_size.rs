//! Custom Grid Size (Warp's Grid › Custom...): the number of rows and
//! columns of the warp grid. Sizes are points from the dialog's top-left
//! corner; the layout follows the AppKit-style dialogs and has not been
//! measured against Photoshop 2026 yet.

use egui::{Align2, Key, Rect, Sense, Ui, vec2};

use super::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(300.0), pt(112.0));
/// The grid's rows and columns: 1–50 each.
const RANGE: (f32, f32) = (1.0, 50.0);

pub enum Outcome {
    Open,
    Cancel,
    /// Columns and rows.
    Ok(usize, usize),
}

pub struct GridSizeDialog {
    rows: String,
    columns: String,
    first_frame: bool,
}

fn parse(text: &str) -> Option<usize> {
    let v: f32 = text.trim().parse().ok()?;
    (RANGE.0..=RANGE.1)
        .contains(&v)
        .then_some(v.round() as usize)
}

impl GridSizeDialog {
    /// Opens with the current grid (columns, rows).
    pub fn new(columns: usize, rows: usize) -> Self {
        Self {
            rows: rows.to_string(),
            columns: columns.to_string(),
            first_frame: true,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("custom-grid-size"))
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
        common::frame(ui, frame, "Custom Grid Size", theme::dialog_bold(pt(13.0)));
        for (k, (label, y)) in [("Rows:", 47.5), ("Columns:", 81.5)]
            .into_iter()
            .enumerate()
        {
            appkit::text(ui, at(90.0, y), Align2::RIGHT_CENTER, label, appkit::TEXT);
            let (text, id) = if k == 0 {
                (&mut self.rows, "grid-rows")
            } else {
                (&mut self.columns, "grid-columns")
            };
            appkit::field(
                ui,
                r(96.0, y - 9.5, 150.0, y + 9.5),
                text,
                id,
                RANGE,
                1.0,
                0,
                self.first_frame && k == 0,
            );
        }
        let size = parse(&self.columns).zip(parse(&self.rows));
        let ok = appkit::button(ui, r(220.0, 38.5, 288.0, 64.5), "OK", true, size.is_some());
        let cancel = appkit::button(ui, r(220.0, 73.5, 288.0, 99.5), "Cancel", false, true);
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
        if let Some((columns, rows)) = size
            && (ok.clicked() || enter)
        {
            return Outcome::Ok(columns, rows);
        }
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_columns_in_range() {
        assert_eq!(parse("7"), Some(7));
        assert_eq!(parse("0"), None);
        assert_eq!(parse("51"), None);
        assert_eq!(parse("x"), None);
    }
}
