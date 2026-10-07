//! Image Size's own Fit To presets: Save Preset... writes the dialog's
//! settings to an `.imz` file, the presets folder's files are listed in the
//! Fit To menu, Load Preset... applies a file, and Delete Preset... (its
//! sheet is here) removes one. The `.imz` layout is Photoshop 2026's, read
//! from files it saved, so presets move between the two.

use std::path::{Path, PathBuf};

use egui::{Align2, Key, Pos2, Rect, Sense, Ui, vec2};

use super::alert::{self, Alert, Answer};
use super::image_size::{ResolutionUnit, SizeUnit};
use super::{appkit, common};
use crate::theme::{self, pt};

/// A saved preset. Widths and heights are kept in the unit for pixels and
/// percent, in inches for the printed units.
#[derive(Clone, Debug, PartialEq)]
pub struct SizePreset {
    pub name: String,
    pub unit: SizeUnit,
    /// The pixels when it was saved.
    pub pixels: (u32, u32),
    pub width: f64,
    pub height: f64,
    /// Pixels per inch.
    pub resolution: f64,
    pub resolution_unit: ResolutionUnit,
    pub constrain: bool,
    pub resample: bool,
}

/// Photoshop's unit numbers (its preferences' order: pixels, inches,
/// centimeters, millimeters, points, picas, percent, then columns).
fn unit_code(unit: SizeUnit) -> u16 {
    match unit {
        SizeUnit::Pixels => 0,
        SizeUnit::Inches => 1,
        SizeUnit::Centimeters => 2,
        SizeUnit::Millimeters => 3,
        SizeUnit::Points => 4,
        SizeUnit::Picas => 5,
        SizeUnit::Percent => 6,
        SizeUnit::Columns => 7,
    }
}

fn unit_from(code: u16) -> Option<SizeUnit> {
    SizeUnit::ALL.into_iter().find(|&u| unit_code(u) == code)
}

/// The `.imz` bytes, big-endian: version 2, the width and height units,
/// the pixels, a reserved zero, width, height and resolution (f64), the
/// resolution unit (1 per inch, 2 per centimeter), then the chain and
/// Resample flags.
pub fn encode(p: &SizePreset) -> Vec<u8> {
    let mut out = Vec::with_capacity(46);
    out.extend(2u32.to_be_bytes());
    out.extend(unit_code(p.unit).to_be_bytes());
    out.extend(unit_code(p.unit).to_be_bytes());
    out.extend(p.pixels.0.to_be_bytes());
    out.extend(p.pixels.1.to_be_bytes());
    out.extend(0u16.to_be_bytes());
    out.extend(p.width.to_be_bytes());
    out.extend(p.height.to_be_bytes());
    out.extend(p.resolution.to_be_bytes());
    let res_unit: u16 = match p.resolution_unit {
        ResolutionUnit::PerInch => 1,
        ResolutionUnit::PerCentimeter => 2,
    };
    out.extend(res_unit.to_be_bytes());
    out.push(p.constrain as u8);
    out.push(p.resample as u8);
    out
}

pub fn decode(name: &str, bytes: &[u8]) -> Option<SizePreset> {
    if bytes.len() < 46 {
        return None;
    }
    let u16_at = |i: usize| u16::from_be_bytes([bytes[i], bytes[i + 1]]);
    let u32_at = |i: usize| u32::from_be_bytes(bytes[i..i + 4].try_into().expect("4 bytes"));
    let f64_at = |i: usize| f64::from_be_bytes(bytes[i..i + 8].try_into().expect("8 bytes"));
    if u32_at(0) != 2 {
        return None;
    }
    let resolution = f64_at(0x22);
    if !(resolution.is_finite() && resolution > 0.0) {
        return None;
    }
    Some(SizePreset {
        name: name.to_owned(),
        unit: unit_from(u16_at(4))?,
        pixels: (u32_at(8), u32_at(12)),
        width: f64_at(0x12),
        height: f64_at(0x1a),
        resolution,
        resolution_unit: if u16_at(0x2a) == 2 {
            ResolutionUnit::PerCentimeter
        } else {
            ResolutionUnit::PerInch
        },
        constrain: bytes[0x2c] != 0,
        resample: bytes[0x2d] != 0,
    })
}

/// Where Save Preset... puts presets and the menu lists them from.
pub fn folder() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library/Application Support/OpenPhoto/Presets")
            .join("Image Size"),
    )
}

/// The presets in `dir`, oldest first (the menu adds new ones at the end).
pub fn list(dir: &Path) -> Vec<(PathBuf, SizePreset)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("imz")))
        .filter_map(|path| {
            let preset = read(&path)?;
            let made = std::fs::metadata(&path)
                .and_then(|m| m.created().or_else(|_| m.modified()))
                .ok();
            Some((made, path, preset))
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    found.into_iter().map(|(_, path, p)| (path, p)).collect()
}

/// A preset file, named after the file.
pub fn read(path: &Path) -> Option<SizePreset> {
    let name = path.file_stem()?.to_string_lossy();
    decode(&name, &std::fs::read(path).ok()?)
}

pub fn write(path: &Path, preset: &SizePreset) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, encode(preset))
}

/// The Delete Preset sheet, measured on Photoshop 2026 (356 × 110 pt, 11 pt
/// right and 28 pt down from the Image Size window's corner): a menu of the
/// presets, Delete and Cancel; Delete asks again in an alert.
pub struct DeletePresetDialog {
    pub names: Vec<String>,
    pub chosen: usize,
    confirm: Option<Alert>,
}

pub const DELETE_SIZE: egui::Vec2 = vec2(pt(356.0), pt(110.0));
pub const DELETE_OFFSET: egui::Vec2 = vec2(pt(11.0), pt(28.0));

pub enum DeleteOutcome {
    Open,
    Cancel,
    /// The preset at this index, confirmed.
    Delete(usize),
}

impl DeletePresetDialog {
    pub fn new(names: Vec<String>) -> Self {
        Self {
            names,
            chosen: 0,
            confirm: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, corner: Pos2) -> DeleteOutcome {
        let mut outcome = DeleteOutcome::Open;
        egui::Modal::new(egui::Id::new("delete-size-preset"))
            .area(
                egui::Modal::default_area(egui::Id::new("delete-size-preset-area"))
                    .anchor(Align2::LEFT_TOP, (corner + DELETE_OFFSET).to_vec2()),
            )
            .frame(egui::Frame::NONE)
            .backdrop_color(egui::Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(DELETE_SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        if let Some(confirm) = &mut self.confirm {
            match alert::show(ctx, confirm) {
                Some(Answer::Ok { .. }) => return DeleteOutcome::Delete(self.chosen),
                Some(_) => self.confirm = None,
                None => {}
            }
            return DeleteOutcome::Open;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            outcome = DeleteOutcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> DeleteOutcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Delete Preset", theme::dialog_bold(pt(13.0)));
        appkit::text(
            ui,
            at(49.5, 48.5),
            Align2::RIGHT_CENTER,
            "Preset:",
            appkit::TEXT,
        );
        let name = self.names.get(self.chosen).cloned().unwrap_or_default();
        if let Some(k) = appkit::popup(
            ui,
            r(55.0, 38.0, 264.0, 59.0),
            "delete-size-preset-name",
            &name,
            &appkit::choices(self.names.iter().map(String::as_str), self.chosen),
        ) {
            self.chosen = k;
        }
        let delete = appkit::button(ui, r(279.5, 38.5, 345.5, 64.5), "Delete", true, true);
        let cancel = appkit::button(ui, r(279.5, 73.5, 345.5, 99.5), "Cancel", false, true);
        if cancel.clicked() {
            return DeleteOutcome::Cancel;
        }
        let enter = self.confirm.is_none()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
        if delete.clicked() || enter {
            let mut confirm =
                Alert::error(format!("Do you really want to delete the preset '{name}'?"));
            confirm.cancel = true;
            confirm.ok_label = "Yes";
            confirm.cancel_label = "No";
            self.confirm = Some(confirm);
        }
        DeleteOutcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Files Photoshop 2026 saved: 64 × 72 px at 72 ppi in inches, then in
    /// pixels at Pixels/Centimeter with the chain off.
    const INCHES: &str = "00000002000100010000004000000048 00003fec72b020c49ba63ff0000000000000405200000000000000010101";
    const PIXELS: &str = "00000002000000000000004000000048 00004050000000000000405200000000000040520000000000000002 0001";

    fn bytes(hex: &str) -> Vec<u8> {
        let hex: String = hex.split_whitespace().collect();
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn reads_photoshops_files_and_writes_them_back() {
        let p = decode("optest", &bytes(INCHES)).unwrap();
        assert_eq!(p.unit, SizeUnit::Inches);
        assert_eq!(p.pixels, (64, 72));
        assert!((p.width - 0.889).abs() < 1e-9 && p.height == 1.0);
        assert_eq!(p.resolution, 72.0);
        assert_eq!(p.resolution_unit, ResolutionUnit::PerInch);
        assert!(p.constrain && p.resample);
        assert_eq!(encode(&p), bytes(INCHES));

        let p = decode("pixels", &bytes(PIXELS)).unwrap();
        assert_eq!(p.unit, SizeUnit::Pixels);
        assert_eq!((p.width, p.height), (64.0, 72.0));
        assert_eq!(p.resolution_unit, ResolutionUnit::PerCentimeter);
        assert!(!p.constrain && p.resample);
        assert_eq!(encode(&p), bytes(PIXELS));

        assert!(decode("short", &[0, 0, 0, 2]).is_none());
    }

    #[test]
    fn the_folder_lists_presets_oldest_first() {
        let dir = std::env::temp_dir().join(format!("op-size-presets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut p = decode("a", &bytes(INCHES)).unwrap();
        write(&dir.join("first.imz"), &p).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        p.resolution = 300.0;
        write(&dir.join("another.imz"), &p).unwrap();
        std::fs::write(dir.join("notes.txt"), "x").unwrap();
        let names: Vec<_> = list(&dir).into_iter().map(|(_, p)| p.name).collect();
        assert_eq!(names, ["first", "another"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
