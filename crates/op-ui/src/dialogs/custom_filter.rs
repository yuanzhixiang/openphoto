//! Filter > Other > Custom...: Photoshop 2026's classic dialog, 634 × 282
//! pt: the preview pane on the left, a 5 × 5 grid of kernel fields with
//! Scale and Offset under it, and OK, Cancel, Load..., Save... and Preview
//! down the right. Sizes are Photoshop points from the dialog's top-left
//! corner.

use egui::{Align2, Pos2, Rect, Ui, vec2};
use op_core::filter::Filter;

use super::appkit;
use super::uxp::Button;
use crate::theme::pt;

pub const SIZE: egui::Vec2 = vec2(pt(634.0), pt(282.0));
/// The grid's columns (left edges) and rows (top edges); fields are 44 ×
/// 19.
const COLUMNS: [f32; 5] = [277.0, 329.0, 380.5, 432.5, 484.5];
const ROWS: [f32; 5] = [78.0, 105.0, 132.0, 159.0, 186.0];
const KERNEL_RANGE: (f32, f32) = (-999.0, 999.0);
const SCALE_RANGE: (f32, f32) = (1.0, 9999.0);
const OFFSET_RANGE: (f32, f32) = (-9999.0, 9999.0);

/// What a button asked for, beyond OK and Cancel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    Load,
    Save,
}

#[derive(Clone)]
pub struct Dialog {
    /// The kernel, rows top to bottom; an empty field counts as 0.
    pub cells: [String; 25],
    pub scale: String,
    /// Empty counts as 0.
    pub offset: String,
}

impl Default for Dialog {
    /// Photoshop's first kernel: a sharpening cross.
    fn default() -> Self {
        let mut cells: [String; 25] = Default::default();
        for (i, v) in [(7, "-1"), (11, "-1"), (12, "5"), (13, "-1"), (17, "-1")] {
            cells[i] = v.into();
        }
        Self {
            cells,
            scale: "1".into(),
            offset: String::new(),
        }
    }
}

/// An integer field's value within `range`; empty is `empty`.
fn parse(text: &str, range: (f32, f32), empty: Option<i16>) -> Option<i16> {
    let text = text.trim();
    if text.is_empty() {
        return empty;
    }
    let v: f32 = text.parse().ok()?;
    (v.fract() == 0.0 && (range.0..=range.1).contains(&v)).then_some(v as i16)
}

impl Dialog {
    pub fn filter(&self) -> Option<Filter> {
        let mut kernel = [0i16; 25];
        for (k, cell) in kernel.iter_mut().zip(&self.cells) {
            *k = parse(cell, KERNEL_RANGE, Some(0))?;
        }
        Some(Filter::Custom {
            kernel,
            scale: parse(&self.scale, SCALE_RANGE, None)?,
            offset: parse(&self.offset, OFFSET_RANGE, Some(0))?,
        })
    }

    /// The fields as typed: the 25 cells, Scale, Offset.
    pub fn settings(&self) -> Vec<String> {
        let mut v = self.cells.to_vec();
        v.push(self.scale.clone());
        v.push(self.offset.clone());
        v
    }

    pub fn restore(&mut self, values: &[String]) {
        if values.len() == 27 {
            for (cell, v) in self.cells.iter_mut().zip(values) {
                cell.clone_from(v);
            }
            self.scale.clone_from(&values[25]);
            self.offset.clone_from(&values[26]);
        }
    }

    /// Photoshop's .acf file: 27 big-endian 16-bit integers, the kernel
    /// row by row, then scale and offset. Zeros load as empty fields.
    pub fn to_acf(&self) -> Option<Vec<u8>> {
        let Filter::Custom {
            kernel,
            scale,
            offset,
        } = self.filter()?
        else {
            return None;
        };
        Some(
            kernel
                .iter()
                .chain([&scale, &offset])
                .flat_map(|v| v.to_be_bytes())
                .collect(),
        )
    }

    pub fn load_acf(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() < 54 {
            return false;
        }
        let v: Vec<i16> = bytes[..54]
            .chunks(2)
            .map(|b| i16::from_be_bytes([b[0], b[1]]))
            .collect();
        let text = |v: i16| if v == 0 { String::new() } else { v.to_string() };
        for (cell, &k) in self.cells.iter_mut().zip(&v) {
            *cell = text(k);
        }
        self.scale = v[25].max(1).to_string();
        self.offset = text(v[26]);
        true
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> (Option<Button>, Option<Request>) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));

        for (i, cell) in self.cells.iter_mut().enumerate() {
            let (x, y) = (COLUMNS[i % 5], ROWS[i / 5]);
            appkit::field(
                ui,
                r(x, y, x + 44.0, y + 19.0),
                cell,
                ("custom-cell", i),
                KERNEL_RANGE,
                1.0,
                0,
                first_frame && i == 0,
            );
        }
        for (label, right, x, value, id, range) in [
            (
                "Scale:",
                325.5,
                330.5,
                &mut self.scale,
                "custom-scale",
                SCALE_RANGE,
            ),
            (
                "Offset:",
                429.5,
                434.5,
                &mut self.offset,
                "custom-offset",
                OFFSET_RANGE,
            ),
        ] {
            appkit::text(
                ui,
                Pos2::new(at(right, 0.0).x, at(0.0, 222.5).y),
                Align2::RIGHT_CENTER,
                label,
                appkit::TEXT,
            );
            appkit::field(
                ui,
                r(x, 213.0, x + 44.0, 232.0),
                value,
                id,
                range,
                1.0,
                0,
                false,
            );
        }

        let valid = self.filter().is_some();
        let button = |ui: &mut Ui, y: f32, label: &str, default: bool, enabled: bool| {
            appkit::button(ui, r(543.5, y, 623.5, y + 26.0), label, default, enabled)
        };
        let ok = button(ui, 38.5, "OK", true, valid);
        let cancel = button(ui, 73.5, "Cancel", false, true);
        let load = button(ui, 143.5, "Load...", false, true);
        let save = button(ui, 178.5, "Save...", false, valid);
        appkit::checkbox(ui, at(543.0, 216.5), "Preview", preview);
        let request = if load.clicked() {
            Some(Request::Load)
        } else if save.clicked() {
            Some(Request::Save)
        } else {
            None
        };
        if cancel.clicked() {
            return (Some(Button::Cancel), request);
        }
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
        (
            ((ok.clicked() || enter) && valid).then_some(Button::Ok),
            request,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_with_photoshops_sharpening_cross() {
        let Some(Filter::Custom {
            kernel,
            scale,
            offset,
        }) = Dialog::default().filter()
        else {
            panic!()
        };
        assert_eq!(kernel.iter().map(|&k| k as i32).sum::<i32>(), 1);
        assert_eq!((kernel[12], kernel[7], kernel[0]), (5, -1, 0));
        assert_eq!((scale, offset), (1, 0));
    }

    #[test]
    fn fields_must_be_integers_in_range() {
        let mut d = Dialog::default();
        d.cells[0] = "1000".into();
        assert!(d.filter().is_none());
        d.cells[0] = "1.5".into();
        assert!(d.filter().is_none());
        d.cells[0] = "-999".into();
        assert!(d.filter().is_some());
        // Scale can't be empty or 0; Offset can be empty
        d.scale = String::new();
        assert!(d.filter().is_none());
        d.scale = "0".into();
        assert!(d.filter().is_none());
        d.scale = "9999".into();
        d.offset = "-9999".into();
        assert!(d.filter().is_some());
    }

    #[test]
    fn acf_files_round_trip() {
        let d = Dialog {
            scale: "4".into(),
            offset: "10".into(),
            ..Default::default()
        };
        let bytes = d.to_acf().unwrap();
        assert_eq!(bytes.len(), 54);
        // The center (12th value) is 5, big-endian
        assert_eq!(&bytes[24..26], &[0, 5]);
        assert_eq!(&bytes[14..16], &[0xff, 0xff]);
        let mut e = Dialog {
            cells: Default::default(),
            scale: "1".into(),
            offset: String::new(),
        };
        assert!(e.load_acf(&bytes));
        assert_eq!(e.filter(), d.filter());
        // Zeros come back as empty fields
        assert_eq!(e.cells[0], "");
        assert!(!e.load_acf(&bytes[..53]));
    }

    #[test]
    fn settings_round_trip() {
        let mut d = Dialog::default();
        d.cells[3] = "7".into();
        d.offset = "2".into();
        let mut e = Dialog::default();
        e.restore(&d.settings());
        assert_eq!(e.filter(), d.filter());
    }
}
