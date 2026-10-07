//! The options Photoshop asks for after the save panel when the file is a
//! JPEG (JPEG Options: Matte, Quality, Format Options) or a PNG (PNG
//! Format Options: File Size). Sizes are points from the dialog's
//! top-left corner; the layouts follow Photoshop's arrangement and have
//! not been measured against Photoshop 2026 yet.

use egui::{Align2, Key, Rect, Sense, Stroke, Ui, vec2};
use op_io::{ExportOptions, PngSize};

use super::{appkit, common};
use crate::theme::{self, pt};

/// Which options the dialog asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Jpeg,
    Png,
}

/// JPEG's Matte choices with their colors (None flattens onto white;
/// Foreground and Background are filled in by `new`).
const MATTES: [&str; 6] = [
    "None",
    "Foreground",
    "Background",
    "White",
    "Black",
    "50% Gray",
];

/// Quality's named steps and the quality each picks.
const QUALITY_NAMES: [(&str, u8); 4] = [("Low", 3), ("Medium", 5), ("High", 8), ("Maximum", 10)];

/// JPEG's Format Options.
const FORMATS: [&str; 3] = [
    "Baseline (\"Standard\")",
    "Baseline Optimized",
    "Progressive",
];

pub enum Outcome {
    Open,
    Cancel,
    Ok(ExportOptions),
}

pub struct SaveOptionsDialog {
    pub format: Format,
    pub options: ExportOptions,
    /// The Matte chosen (index into `MATTES`) and the colors it can take.
    matte: usize,
    colors: ([u8; 3], [u8; 3]),
    quality: String,
    /// Baseline, Baseline Optimized or Progressive, and Progressive's
    /// scans (3–5).
    pub jpeg_format: usize,
    scans: usize,
    first_frame: bool,
}

/// The name of the step a quality falls in.
fn quality_name(q: u8) -> &'static str {
    match q {
        0..=4 => "Low",
        5..=7 => "Medium",
        8..=9 => "High",
        _ => "Maximum",
    }
}

impl SaveOptionsDialog {
    /// Opens with the last options used; `colors` are the foreground and
    /// background colors (Matte's Foreground and Background).
    pub fn new(format: Format, options: ExportOptions, colors: ([u8; 3], [u8; 3])) -> Self {
        // (white, the default, reads as None)
        let matte = match options.matte {
            [255, 255, 255] => 0,
            [0, 0, 0] => 4,
            [128, 128, 128] => 5,
            m if m == colors.0 => 1,
            m if m == colors.1 => 2,
            _ => 0,
        };
        Self {
            format,
            quality: options.jpeg_quality.to_string(),
            options,
            matte,
            colors,
            jpeg_format: 0,
            scans: 3,
            first_frame: true,
        }
    }

    pub fn size(&self) -> egui::Vec2 {
        match self.format {
            Format::Jpeg => vec2(pt(440.0), pt(318.0)),
            Format::Png => vec2(pt(400.0), pt(150.0)),
        }
    }

    fn matte_color(&self) -> [u8; 3] {
        match self.matte {
            1 => self.colors.0,
            2 => self.colors.1,
            4 => [0, 0, 0],
            5 => [128, 128, 128],
            _ => [255, 255, 255],
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("save-options"))
            .frame(egui::Frame::NONE)
            .backdrop_color(egui::Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(self.size(), Sense::hover());
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
        let title = match self.format {
            Format::Jpeg => "JPEG Options",
            Format::Png => "PNG Format Options",
        };
        common::frame(ui, frame, title, theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        let group = |ui: &Ui, title: &str, rect: Rect| {
            let label = appkit::label(ui, rect.min + vec2(pt(10.0), 0.0), title);
            appkit::group(
                &painter,
                rect,
                (label.left() - pt(4.0), label.right() + pt(4.0)),
            );
        };

        let valid = match self.format {
            Format::Png => {
                group(ui, "File Size", r(18.0, 45.0, 330.0, 138.0));
                for (k, (label, size)) in [
                    ("Large file size (fastest saving)", PngSize::Large),
                    ("Medium file size", PngSize::Medium),
                    ("Smallest file size (slowest saving)", PngSize::Smallest),
                ]
                .into_iter()
                .enumerate()
                {
                    let y = 68.0 + k as f32 * 24.0;
                    if appkit::radio(ui, at(36.0, y), label, self.options.png == size) {
                        self.options.png = size;
                    }
                }
                true
            }
            Format::Jpeg => self.jpeg_ui(ui, frame, &group),
        };

        let x0 = frame.width() / pt(1.0) - 88.0;
        let ok = appkit::button(ui, r(x0, 38.5, x0 + 70.0, 64.5), "OK", true, valid);
        let cancel = appkit::button(ui, r(x0, 73.5, x0 + 70.0, 99.5), "Cancel", false, true);
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        let enter = !typing && ui.input(|i| i.key_pressed(Key::Enter));
        if valid && (ok.clicked() || enter) {
            return Outcome::Ok(self.options);
        }
        Outcome::Open
    }

    /// JPEG Options: Matte, the Image Options group (Quality field, its
    /// named step and a slider), the Format Options group.
    fn jpeg_ui(&mut self, ui: &mut Ui, frame: Rect, group: &dyn Fn(&Ui, &str, Rect)) -> bool {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        appkit::text(
            ui,
            at(70.0, 56.5),
            Align2::RIGHT_CENTER,
            "Matte:",
            appkit::TEXT,
        );
        if let Some(k) = appkit::popup(
            ui,
            r(76.0, 46.0, 200.0, 67.0),
            "jpeg-matte",
            MATTES[self.matte],
            &appkit::choices(MATTES, self.matte),
        ) {
            self.matte = k;
            self.options.matte = self.matte_color();
        }

        // Image Options: Quality
        group(ui, "Image Options", r(18.0, 90.0, 340.0, 176.0));
        appkit::text(
            ui,
            at(84.0, 116.5),
            Align2::RIGHT_CENTER,
            "Quality:",
            appkit::TEXT,
        );
        let field = appkit::field(
            ui,
            r(90.0, 107.0, 130.0, 126.0),
            &mut self.quality,
            "jpeg-quality",
            (0.0, 12.0),
            1.0,
            0,
            self.first_frame,
        );
        let typed = self.quality.trim().parse::<u8>().ok().filter(|q| *q <= 12);
        if field.changed()
            && let Some(q) = typed
        {
            self.options.jpeg_quality = q;
        }
        let names: Vec<&str> = QUALITY_NAMES.iter().map(|(n, _)| *n).collect();
        let current = quality_name(self.options.jpeg_quality);
        let chosen = names.iter().position(|n| *n == current).unwrap_or(3);
        if let Some(k) = appkit::popup(
            ui,
            r(140.0, 106.0, 240.0, 127.0),
            "jpeg-quality-name",
            current,
            &appkit::choices(names.iter().copied(), chosen),
        ) {
            self.options.jpeg_quality = QUALITY_NAMES[k].1;
            self.quality = self.options.jpeg_quality.to_string();
        }
        // The slider: small file to large file
        let (x0, x1, y) = (34.0, 324.0, 142.0);
        let track = r(x0, y, x1, y + 3.0);
        ui.painter()
            .rect_filled(track, pt(1.5), egui::Color32::from_gray(0x75));
        let place = self.options.jpeg_quality as f32 / 12.0;
        let tip = egui::pos2(
            track.left() + track.width() * place,
            track.bottom() + pt(0.5),
        );
        appkit::pin(ui.painter(), tip, appkit::Pin::White);
        let hit = track.expand2(vec2(pt(6.0), pt(8.0)));
        let response = ui.interact(
            hit,
            ui.id().with("jpeg-quality-track"),
            Sense::click_and_drag(),
        );
        if (response.dragged() || response.clicked())
            && let Some(p) = response.interact_pointer_pos()
        {
            let q = (((p.x - track.left()) / track.width()).clamp(0.0, 1.0) * 12.0).round() as u8;
            self.options.jpeg_quality = q;
            self.quality = q.to_string();
        }
        let small = theme::dialog(pt(10.0));
        for (x, align, label) in [
            (x0, Align2::LEFT_CENTER, "small file"),
            (x1, Align2::RIGHT_CENTER, "large file"),
        ] {
            ui.painter()
                .text(at(x, 162.0), align, label, small.clone(), appkit::TEXT);
        }

        // Format Options
        group(ui, "Format Options", r(18.0, 196.0, 340.0, 306.0));
        for (k, label) in FORMATS.into_iter().enumerate() {
            let y = 219.0 + k as f32 * 24.0;
            if appkit::radio(ui, at(36.0, y), label, self.jpeg_format == k) {
                self.jpeg_format = k;
            }
        }
        appkit::text(
            ui,
            at(84.0, 291.5),
            Align2::RIGHT_CENTER,
            "Scans:",
            appkit::TEXT,
        );
        let scans: Vec<String> = (3..=5).map(|n| n.to_string()).collect();
        if self.jpeg_format == 2 {
            if let Some(k) = appkit::popup(
                ui,
                r(90.0, 281.0, 150.0, 302.0),
                "jpeg-scans",
                &scans[self.scans - 3],
                &appkit::choices(scans.iter().map(String::as_str), self.scans - 3),
            ) {
                self.scans = k + 3;
            }
        } else {
            let field = r(90.0, 281.0, 150.0, 302.0);
            ui.painter().rect_stroke(
                field,
                pt(4.0),
                Stroke::new(pt(1.0), egui::Color32::from_gray(0x5d)),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                field.left_center() + vec2(pt(8.0), 0.0),
                Align2::LEFT_CENTER,
                &scans[self.scans - 3],
                appkit::font(),
                appkit::TEXT_OFF,
            );
        }
        typed.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_names_and_mattes() {
        assert_eq!(quality_name(3), "Low");
        assert_eq!(quality_name(8), "High");
        assert_eq!(quality_name(12), "Maximum");
        let colors = ([10, 20, 30], [200, 210, 220]);
        let options = ExportOptions {
            matte: colors.1,
            ..Default::default()
        };
        let d = SaveOptionsDialog::new(Format::Jpeg, options, colors);
        assert_eq!(d.matte, 2);
        assert_eq!(d.matte_color(), colors.1);
    }
}
