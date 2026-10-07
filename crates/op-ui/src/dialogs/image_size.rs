//! Image > Image Size (Alt+Cmd+I), laid out at the positions measured on
//! Photoshop 2026's dialog (665 × 358 pt, a resizable macOS window): a
//! 100% preview on the left; on the right the memory size, Dimensions,
//! Fit To, Width and Height with their units and the chain, Resolution,
//! Resample and its method. Sizes are in Photoshop points from the
//! dialog's top-left corner. The numbers live in [`Model`].

use std::sync::Arc;

use egui::{
    Align2, Color32, CornerRadius, Key, Modifiers, Pos2, Rect, Sense, Shape, Stroke, StrokeKind,
    Ui, vec2,
};
use op_core::image_ops::Resample;

use super::auto_resolution::{AutoResolutionDialog, Outcome as AutoOutcome};
use super::canvas_size::MAX_DIMENSION;
use super::common;
use super::size_presets::{self, DeleteOutcome, DeletePresetDialog, SizePreset};
use crate::native_popup::{self, Entry};
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(665.0), pt(358.0));
const LABEL: Color32 = Color32::from_gray(0xd6);
const LINK: Color32 = Color32::from_rgb(0x5e, 0x9e, 0xee);
const BOX_FILL: Color32 = Color32::from_gray(0x38);
const BOX_EDGE: Color32 = Color32::from_gray(0x63);
const BRACKET: Color32 = Color32::from_gray(0x82);
/// Labels end here.
const LABEL_RIGHT: f32 = 423.5;

/// Units for Width and Height (and, but Columns, the Dimensions line).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeUnit {
    Percent,
    Pixels,
    Inches,
    Centimeters,
    Millimeters,
    Points,
    Picas,
    Columns,
}

impl SizeUnit {
    pub const ALL: [Self; 8] = [
        Self::Percent,
        Self::Pixels,
        Self::Inches,
        Self::Centimeters,
        Self::Millimeters,
        Self::Points,
        Self::Picas,
        Self::Columns,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Percent => "Percent",
            Self::Pixels => "Pixels",
            Self::Inches => "Inches",
            Self::Centimeters => "Centimeters",
            Self::Millimeters => "Millimeters",
            Self::Points => "Points",
            Self::Picas => "Picas",
            Self::Columns => "Columns",
        }
    }

    fn abbreviation(self) -> &'static str {
        match self {
            Self::Percent => "%",
            Self::Pixels => "px",
            Self::Inches => "in",
            Self::Centimeters => "cm",
            Self::Millimeters => "mm",
            Self::Points => "pt",
            Self::Picas => "pica",
            Self::Columns => "col",
        }
    }

    /// Units per inch, for the units tied to the resolution.
    fn per_inch(self) -> Option<f64> {
        match self {
            Self::Inches => Some(1.0),
            Self::Centimeters => Some(2.54),
            Self::Millimeters => Some(25.4),
            Self::Points => Some(72.0),
            Self::Picas => Some(6.0),
            _ => None,
        }
    }

    /// A printed size: it depends on the resolution.
    fn physical(self) -> bool {
        !matches!(self, Self::Percent | Self::Pixels)
    }
}

/// Resolution units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolutionUnit {
    PerInch,
    PerCentimeter,
}

impl ResolutionUnit {
    pub const ALL: [Self; 2] = [Self::PerInch, Self::PerCentimeter];

    pub fn label(self) -> &'static str {
        match self {
            Self::PerInch => "Pixels/Inch",
            Self::PerCentimeter => "Pixels/Centimeter",
        }
    }
}

/// Fit To's presets, in Photoshop 2026's menu (groups between separators).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FitTo {
    Original,
    /// The resolution from the Auto Resolution sheet.
    AutoResolution,
    /// Width × height, a unit and the resolution (ppi).
    Preset(&'static str, f64, f64, SizeUnit, f64),
    /// One of the presets saved with Save Preset..., by its place in the
    /// menu.
    User(usize),
    Custom,
}

pub const FIT_PRESETS: [&[FitTo]; 3] = [
    &[
        FitTo::Preset(
            "960 x 640 px 144 ppi",
            960.0,
            640.0,
            SizeUnit::Pixels,
            144.0,
        ),
        FitTo::Preset(
            "1024 x 768 px 72 ppi",
            1024.0,
            768.0,
            SizeUnit::Pixels,
            72.0,
        ),
        FitTo::Preset(
            "1136 x 640 px 144 ppi",
            1136.0,
            640.0,
            SizeUnit::Pixels,
            144.0,
        ),
        FitTo::Preset(
            "1366 x 768 px 72 ppi",
            1366.0,
            768.0,
            SizeUnit::Pixels,
            72.0,
        ),
    ],
    &[
        FitTo::Preset(
            "A4 210 x 297 mm 300 dpi",
            210.0,
            297.0,
            SizeUnit::Millimeters,
            300.0,
        ),
        FitTo::Preset(
            "A6 105 x 148 mm 300 dpi",
            105.0,
            148.0,
            SizeUnit::Millimeters,
            300.0,
        ),
        FitTo::Preset(
            "Legal 8.5 x 14 in 300 dpi",
            8.5,
            14.0,
            SizeUnit::Inches,
            300.0,
        ),
        FitTo::Preset(
            "Letter 8.5 x 11 in 300 dpi",
            8.5,
            11.0,
            SizeUnit::Inches,
            300.0,
        ),
    ],
    &[
        FitTo::Preset("4 x 6 in 300 dpi", 4.0, 6.0, SizeUnit::Inches, 300.0),
        FitTo::Preset("5 x 7 in 300 dpi", 5.0, 7.0, SizeUnit::Inches, 300.0),
        FitTo::Preset("8 x 10 in 300 dpi", 8.0, 10.0, SizeUnit::Inches, 300.0),
        FitTo::Preset("11 x 14 in 300 dpi", 11.0, 14.0, SizeUnit::Inches, 300.0),
    ],
];

impl FitTo {
    pub fn label(self) -> &'static str {
        match self {
            Self::Original => "Original Size",
            Self::AutoResolution => "Auto Resolution...",
            Self::Preset(name, ..) => name,
            Self::User(_) | Self::Custom => "Custom",
        }
    }
}

/// The dialog's numbers: the new size in pixels and the resolution, shown
/// in the chosen units. Photoshop's rules:
/// - with Resample on, a size edit changes the pixels (and with the chain,
///   both sides); a resolution edit keeps a printed size (inches...) and
///   changes the pixels, or keeps the pixels when they're shown in pixels
///   or percent;
/// - with Resample off the pixels stay the document's: a printed size edit
///   changes the resolution, the chain is always on and Pixels and Percent
///   can't be edited.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub original: (u32, u32),
    pub original_resolution: f64,
    pub width: f64,
    pub height: f64,
    /// Pixels per inch.
    pub resolution: f64,
    pub unit: SizeUnit,
    pub resolution_unit: ResolutionUnit,
    pub dimensions_unit: SizeUnit,
    pub constrain: bool,
    pub resample: bool,
    pub method: Resample,
    pub fit: FitTo,
    pub scale_styles: bool,
    /// Preserve Details' Reduce Noise, 0–100 %.
    pub reduce_noise: f64,
}

/// Column width and gutter in points (Photoshop's defaults).
const COLUMN: f64 = 180.0;
const GUTTER: f64 = 12.0;

impl Model {
    pub fn new(width: u32, height: u32, resolution: f32) -> Self {
        Self {
            original: (width, height),
            original_resolution: resolution as f64,
            width: width as f64,
            height: height as f64,
            resolution: resolution as f64,
            unit: SizeUnit::Inches,
            resolution_unit: ResolutionUnit::PerInch,
            dimensions_unit: SizeUnit::Pixels,
            constrain: true,
            resample: true,
            method: Resample::Automatic,
            fit: FitTo::Original,
            scale_styles: true,
            reduce_noise: 0.0,
        }
    }

    /// `px` (of a side `original` pixels long) in `unit`.
    pub fn in_unit(&self, px: f64, original: u32, unit: SizeUnit) -> f64 {
        match unit {
            SizeUnit::Percent => px / original as f64 * 100.0,
            SizeUnit::Pixels => px,
            SizeUnit::Columns => (px / self.resolution * 72.0 + GUTTER) / (COLUMN + GUTTER),
            u => px / self.resolution * u.per_inch().expect("physical"),
        }
    }

    fn to_pixels(&self, value: f64, original: u32, resolution: f64) -> f64 {
        match self.unit {
            SizeUnit::Percent => value / 100.0 * original as f64,
            SizeUnit::Pixels => value,
            SizeUnit::Columns => (value * (COLUMN + GUTTER) - GUTTER) / 72.0 * resolution,
            u => value / u.per_inch().expect("physical") * resolution,
        }
    }

    /// Whether the size fields can be edited.
    pub fn size_editable(&self) -> bool {
        self.resample || self.unit.physical()
    }

    pub fn set_width(&mut self, value: f64) {
        self.set_side(value, true);
    }

    pub fn set_height(&mut self, value: f64) {
        self.set_side(value, false);
    }

    fn set_side(&mut self, value: f64, width: bool) {
        if value.is_nan() || value <= 0.0 || !self.size_editable() {
            return;
        }
        self.fit = FitTo::Custom;
        let (w0, h0) = (self.original.0 as f64, self.original.1 as f64);
        let original = if width {
            self.original.0
        } else {
            self.original.1
        };
        if self.resample {
            let px = self.to_pixels(value, original, self.resolution);
            if width {
                self.width = px;
                if self.constrain {
                    self.height = px * h0 / w0;
                }
            } else {
                self.height = px;
                if self.constrain {
                    self.width = px * w0 / h0;
                }
            }
        } else {
            // The pixels stay: the printed size sets the resolution
            let side = if width { self.width } else { self.height };
            let one = self.to_pixels(value, original, 1.0);
            if one > 0.0 {
                self.resolution = side / one;
            }
        }
    }

    /// `value` in the resolution unit.
    pub fn set_resolution(&mut self, value: f64) {
        if value.is_nan() || value <= 0.0 {
            return;
        }
        self.fit = FitTo::Custom;
        let ppi = match self.resolution_unit {
            ResolutionUnit::PerInch => value,
            ResolutionUnit::PerCentimeter => value * 2.54,
        };
        if self.resample && self.unit.physical() {
            // The printed size stays, so the pixels follow
            let k = ppi / self.resolution;
            self.width *= k;
            self.height *= k;
        }
        self.resolution = ppi;
    }

    pub fn resolution_in_unit(&self) -> f64 {
        match self.resolution_unit {
            ResolutionUnit::PerInch => self.resolution,
            ResolutionUnit::PerCentimeter => self.resolution / 2.54,
        }
    }

    pub fn set_resample(&mut self, on: bool) {
        self.resample = on;
        if !on {
            // Without resampling the pixels can't change
            self.width = self.original.0 as f64;
            self.height = self.original.1 as f64;
            self.constrain = true;
        }
    }

    pub fn set_constrain(&mut self, on: bool) {
        if !self.resample {
            return;
        }
        self.constrain = on;
        if on {
            let (w0, h0) = (self.original.0 as f64, self.original.1 as f64);
            self.height = self.width * h0 / w0;
        }
    }

    /// Fit To: Original Size puts everything back; a preset fits the
    /// image in its box (turned to the image's orientation), keeping the
    /// proportions, at its resolution and in its unit.
    pub fn fit_to(&mut self, fit: FitTo) {
        match fit {
            FitTo::Original => {
                let (w, h) = self.original;
                self.width = w as f64;
                self.height = h as f64;
                self.resolution = self.original_resolution;
            }
            FitTo::Preset(_, a, b, unit, ppi) => {
                self.resample = true;
                self.constrain = true;
                self.unit = unit;
                self.resolution = ppi;
                let (w0, h0) = (self.original.0 as f64, self.original.1 as f64);
                let (bw, bh) = if (w0 >= h0) == (a >= b) {
                    (a, b)
                } else {
                    (b, a)
                };
                let (bw, bh) = (
                    self.to_pixels(bw, self.original.0, ppi),
                    self.to_pixels(bh, self.original.1, ppi),
                );
                self.fit_box(bw, bh);
            }
            FitTo::AutoResolution | FitTo::User(_) | FitTo::Custom => {}
        }
        self.fit = fit;
    }

    /// Fits the image in a `bw` × `bh` pixel box, keeping its proportions.
    fn fit_box(&mut self, bw: f64, bh: f64) {
        let (w0, h0) = (self.original.0 as f64, self.original.1 as f64);
        let k = (bw / w0).min(bh / h0);
        self.width = w0 * k;
        self.height = h0 * k;
    }

    /// The settings as a preset named `name` (Save Preset...).
    pub fn to_preset(&self, name: &str) -> SizePreset {
        let (width, height) = match self.unit {
            SizeUnit::Pixels => (self.width.round(), self.height.round()),
            SizeUnit::Percent => (
                self.in_unit(self.width, self.original.0, SizeUnit::Percent),
                self.in_unit(self.height, self.original.1, SizeUnit::Percent),
            ),
            _ => (self.width / self.resolution, self.height / self.resolution),
        };
        SizePreset {
            name: name.to_owned(),
            unit: self.unit,
            pixels: (self.width.round() as u32, self.height.round() as u32),
            width,
            height,
            resolution: self.resolution,
            resolution_unit: self.resolution_unit,
            constrain: self.constrain,
            resample: self.resample,
        }
    }

    /// Applies a saved preset: its units, chain, Resample and resolution;
    /// with Resample on, its size (with the chain on the image is fitted in
    /// that size, keeping its proportions, as the built-in presets do).
    pub fn apply_preset(&mut self, p: &SizePreset, fit: FitTo) {
        self.unit = p.unit;
        self.resolution_unit = p.resolution_unit;
        self.resample = p.resample;
        self.resolution = p.resolution;
        if p.resample {
            self.constrain = p.constrain;
            let (w0, h0) = (self.original.0 as f64, self.original.1 as f64);
            let (w, h) = match p.unit {
                SizeUnit::Pixels => (p.width, p.height),
                SizeUnit::Percent => (w0 * p.width / 100.0, h0 * p.height / 100.0),
                _ => (p.width * p.resolution, p.height * p.resolution),
            };
            if p.constrain {
                self.fit_box(w, h);
            } else {
                self.width = w;
                self.height = h;
            }
        } else {
            self.constrain = true;
            self.width = self.original.0 as f64;
            self.height = self.original.1 as f64;
        }
        self.fit = fit;
    }

    /// Auto Resolution's OK: the new resolution keeps the printed size, so
    /// with Resample on the pixels follow (64 × 72 px at 72 ppi set to 200
    /// ppi is 178 × 200 px, as in Photoshop).
    pub fn auto_resolution(&mut self, ppi: f64) {
        if self.resample {
            let k = ppi / self.resolution;
            self.width *= k;
            self.height *= k;
        }
        self.resolution = ppi;
        self.fit = FitTo::AutoResolution;
    }

    /// The result: whole pixels within Photoshop's limits, and the
    /// resolution in pixels per inch.
    pub fn result(&self) -> Option<(u32, u32, f32)> {
        let side = |v: f64| {
            let v = v.round();
            (1.0..=MAX_DIMENSION as f64)
                .contains(&v)
                .then_some(v as u32)
        };
        let ok = (1.0..=10_000.0).contains(&self.resolution);
        Some((
            side(self.width)?,
            side(self.height)?,
            ok.then_some(self.resolution as f32)?,
        ))
    }

    /// "1.70M", or "425.4K (was 1.70M)" once it changes.
    pub fn memory_text(&self) -> String {
        let mem = |w: f64, h: f64| {
            let bytes = w.round() * h.round() * 3.0;
            if bytes >= 1024.0 * 1024.0 {
                format!("{:.2}M", bytes / 1024.0 / 1024.0)
            } else {
                format!("{:.1}K", bytes / 1024.0)
            }
        };
        let now = mem(self.width, self.height);
        let was = mem(self.original.0 as f64, self.original.1 as f64);
        if now == was {
            now
        } else {
            format!("{now} (was {was})")
        }
    }

    /// "734 px  ×  811 px" in the Dimensions unit.
    pub fn dimensions_text(&self) -> (String, String) {
        let u = self.dimensions_unit;
        let side = |px: f64, original: u32| {
            let v = self.in_unit(px, original, u);
            if u == SizeUnit::Percent {
                format!("{}%", number(v))
            } else {
                format!("{} {}", number(v), u.abbreviation())
            }
        };
        (
            side(self.width.round(), self.original.0),
            side(self.height.round(), self.original.1),
        )
    }

    /// The text a size field shows.
    pub fn field_text(&self, px: f64, original: u32) -> String {
        if self.unit == SizeUnit::Pixels {
            return format!("{}", px.round());
        }
        number(self.in_unit(px, original, self.unit))
    }
}

/// Up to three decimals, without trailing zeros: 734 / 72 in → "10.194".
fn number(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_string() }
}

pub enum Outcome {
    Open,
    Cancel,
    Apply {
        width: u32,
        height: u32,
        resolution: f32,
        /// `None` when Resample is off (only the resolution changes).
        resample: Option<Resample>,
        /// Preserve Details' Reduce Noise, 0–1.
        reduce_noise: f32,
    },
}

/// The document's merged pixels for the preview.
pub struct Preview {
    pub rgba: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

pub struct ImageSizeDialog {
    pub model: Model,
    width_text: String,
    height_text: String,
    resolution_text: String,
    noise_text: String,
    preview: Option<Preview>,
    /// The image point at the preview's center.
    center: (f32, f32),
    texture: Option<(egui::TextureHandle, (i64, i64))>,
    first_frame: bool,
    /// The Auto Resolution sheet, while it's open.
    pub auto: Option<AutoResolutionDialog>,
    corner: Pos2,
    /// Saved presets (the folder, and its files with their settings).
    folder: Option<std::path::PathBuf>,
    pub presets: Vec<(std::path::PathBuf, SizePreset)>,
    /// The Delete Preset sheet, while it's open.
    pub delete: Option<DeletePresetDialog>,
    /// How much larger than its smallest size the window has been dragged
    /// (Photoshop points); the app keeps it for the next opening.
    pub extra: egui::Vec2,
    /// The window's top-left corner: it stays put while resizing.
    origin: Option<Pos2>,
}

/// What a Fit To menu entry does.
#[derive(Clone, Copy, Debug, PartialEq)]
enum FitAction {
    Fit(FitTo),
    Auto,
    Load,
    Save,
    Delete,
    Custom,
    Nothing,
}

impl ImageSizeDialog {
    pub fn new(width: u32, height: u32, resolution: f32) -> Self {
        let model = Model::new(width, height, resolution);
        let mut dialog = Self {
            model,
            width_text: String::new(),
            height_text: String::new(),
            resolution_text: String::new(),
            noise_text: "0".into(),
            preview: None,
            center: (width as f32 / 2.0, height as f32 / 2.0),
            texture: None,
            first_frame: true,
            auto: None,
            corner: Pos2::ZERO,
            folder: None,
            presets: Vec::new(),
            delete: None,
            extra: egui::Vec2::ZERO,
            origin: None,
        };
        dialog.set_preset_folder(size_presets::folder());
        dialog.refresh(None);
        dialog
    }

    /// Opens at the size it was last left at.
    pub fn with_extra(mut self, extra: egui::Vec2) -> Self {
        self.extra = extra.max(egui::Vec2::ZERO);
        self
    }

    pub fn with_preview(mut self, preview: Preview) -> Self {
        self.preview = Some(preview);
        self
    }

    /// Where presets are saved and listed from (rereads the list).
    pub fn set_preset_folder(&mut self, folder: Option<std::path::PathBuf>) {
        self.presets = folder
            .as_deref()
            .map(size_presets::list)
            .unwrap_or_default();
        self.folder = folder;
    }

    /// Fit To's text: a saved preset shows its name.
    pub fn fit_label(&self) -> String {
        match self.model.fit {
            FitTo::User(i) => self
                .presets
                .get(i)
                .map_or("Custom".into(), |(_, p)| p.name.clone()),
            fit => fit.label().into(),
        }
    }

    /// Save Preset...'s file: writes the settings there and, when it's in
    /// the presets folder, picks it in Fit To.
    pub fn save_preset_to(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        let path = if path.extension().is_none() {
            path.with_extension("imz")
        } else {
            path.to_path_buf()
        };
        let name = path
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        size_presets::write(&path, &self.model.to_preset(&name))?;
        self.set_preset_folder(self.folder.clone());
        if let Some(i) = self.presets.iter().position(|(p, _)| *p == path) {
            self.model.fit = FitTo::User(i);
        }
        Ok(())
    }

    /// Load Preset...'s file: applies its settings (Fit To shows Custom
    /// unless the file is one of the listed presets).
    pub fn load_preset_from(&mut self, path: &std::path::Path) -> bool {
        let Some(preset) = size_presets::read(path) else {
            return false;
        };
        let fit = self
            .presets
            .iter()
            .position(|(p, _)| p == path)
            .map_or(FitTo::Custom, FitTo::User);
        self.model.apply_preset(&preset, fit);
        self.refresh(None);
        true
    }

    /// Fit To's menu: Original Size and Auto Resolution..., the built-in
    /// groups, the saved presets, the preset commands and Custom.
    fn fit_menu(&self) -> Vec<(Entry, FitAction)> {
        let fit = self.model.fit;
        let item = |label: &str, f: FitTo| (Entry::item(label, fit == f), FitAction::Fit(f));
        let mut menu = vec![
            item(FitTo::Original.label(), FitTo::Original),
            (
                Entry::item(FitTo::AutoResolution.label(), fit == FitTo::AutoResolution),
                FitAction::Auto,
            ),
        ];
        let separator = (Entry::Separator, FitAction::Nothing);
        for group in FIT_PRESETS {
            menu.push(separator.clone());
            menu.extend(group.iter().map(|&p| item(p.label(), p)));
        }
        if !self.presets.is_empty() {
            menu.push(separator.clone());
            for (i, (_, p)) in self.presets.iter().enumerate() {
                menu.push(item(&p.name, FitTo::User(i)));
            }
        }
        menu.push(separator.clone());
        menu.push((Entry::item("Load Preset...", false), FitAction::Load));
        menu.push((Entry::item("Save Preset...", false), FitAction::Save));
        menu.push((
            Entry::item("Delete Preset...", false).enabled(!self.presets.is_empty()),
            FitAction::Delete,
        ));
        menu.push(separator);
        menu.push((
            Entry::item("Custom", fit == FitTo::Custom),
            FitAction::Custom,
        ));
        menu
    }

    /// Rewrites the fields from the model, except the one being edited.
    fn refresh(&mut self, editing: Option<u8>) {
        let m = &self.model;
        if editing != Some(0) {
            self.width_text = m.field_text(m.width, m.original.0);
        }
        if editing != Some(1) {
            self.height_text = m.field_text(m.height, m.original.1);
        }
        if editing != Some(2) {
            self.resolution_text = number(m.resolution_in_unit());
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        let size = SIZE + self.extra * pt(1.0);
        let origin = *self
            .origin
            .get_or_insert_with(|| ctx.content_rect().center() - size / 2.0);
        egui::Modal::new(egui::Id::new("image-size"))
            .area(
                egui::Modal::default_area(egui::Id::new("image-size-area"))
                    .anchor(Align2::LEFT_TOP, origin.to_vec2()),
            )
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                self.corner = rect.min;
                outcome = self.ui(ui, rect);
                self.resize_ui(ui, rect);
            });
        self.first_frame = false;
        // A sheet takes the keys while it's open
        if let Some(delete) = &mut self.delete {
            match delete.show(ctx, Some(self.corner)) {
                DeleteOutcome::Open => {}
                DeleteOutcome::Cancel => self.delete = None,
                DeleteOutcome::Delete(i) => {
                    self.delete = None;
                    if let Some((path, _)) = self.presets.get(i) {
                        let _ = std::fs::remove_file(path);
                    }
                    if self.model.fit == FitTo::User(i) {
                        self.model.fit = FitTo::Custom;
                    }
                    self.set_preset_folder(self.folder.clone());
                }
            }
            return Outcome::Open;
        }
        if let Some(auto) = &mut self.auto {
            match auto.show(ctx, self.corner) {
                AutoOutcome::Open => {}
                AutoOutcome::Cancel => self.auto = None,
                AutoOutcome::Apply(ppi) => {
                    self.auto = None;
                    self.model.auto_resolution(ppi);
                    self.refresh(None);
                }
            }
            return Outcome::Open;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    /// Dragging the window's right or bottom edge or its corner resizes it
    /// (never below its first size): the preview grows, the controls keep
    /// to the right and the buttons to the bottom, as in Photoshop.
    fn resize_ui(&mut self, ui: &mut Ui, frame: Rect) {
        let grip = pt(4.0);
        let handles = [
            (
                Rect::from_min_max(
                    Pos2::new(frame.right() - grip, frame.top() + common::TITLE_BAR),
                    Pos2::new(frame.right() + grip, frame.bottom() - grip),
                ),
                vec2(1.0, 0.0),
                egui::CursorIcon::ResizeHorizontal,
            ),
            (
                Rect::from_min_max(
                    Pos2::new(frame.left(), frame.bottom() - grip),
                    Pos2::new(frame.right() - grip, frame.bottom() + grip),
                ),
                vec2(0.0, 1.0),
                egui::CursorIcon::ResizeVertical,
            ),
            (
                Rect::from_center_size(frame.right_bottom(), vec2(grip * 2.0, grip * 2.0)),
                vec2(1.0, 1.0),
                egui::CursorIcon::ResizeNwSe,
            ),
        ];
        for (k, (rect, axes, cursor)) in handles.into_iter().enumerate() {
            let response = ui
                .interact(rect, ui.id().with(("image-size-resize", k)), Sense::drag())
                .on_hover_cursor(cursor);
            if response.dragged() {
                let delta = response.drag_delta() * axes / pt(1.0);
                self.extra = (self.extra + delta).max(egui::Vec2::ZERO);
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let (grow_w, grow_h) = (self.extra.x, self.extra.y);
        // The controls keep to the right edge
        let column = frame.translate(vec2(pt(grow_w), 0.0));
        let at = |x: f32, y: f32| column.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let font = theme::dialog(pt(12.0));
        common::frame(ui, frame, "Image Size", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        traffic_lights(&painter, frame);
        self.preview_ui(
            ui,
            Rect::from_min_max(
                frame.min + vec2(pt(10.0), pt(38.0)),
                frame.min + vec2(pt(288.0 + grow_w), pt(316.0 + grow_h)),
            ),
        );

        let label = |text: &str, cy: f32| {
            painter.text(
                at(LABEL_RIGHT, cy),
                Align2::RIGHT_CENTER,
                text,
                font.clone(),
                LABEL,
            );
        };

        // Memory size and the gear's menu
        label("Image Size:", 50.25);
        painter.text(
            at(431.5, 49.75),
            Align2::LEFT_CENTER,
            self.model.memory_text(),
            font.clone(),
            LABEL,
        );
        let gear = r(633.5, 41.5, 654.5, 58.5);
        let response = ui.interact(gear, ui.id().with("gear"), Sense::click());
        gear_icon(&painter, gear.center());
        let styles = [Entry::item("Scale Styles", self.model.scale_styles)];
        if native_popup::dropdown(ui, &response, ui.id().with("gear-menu"), &styles).is_some() {
            self.model.scale_styles = !self.model.scale_styles;
        }

        // Dimensions, with the unit chosen from the box's menu
        label("Dimensions:", 79.75);
        let dims_box = r(432.0, 70.5, 450.0, 88.0);
        let response = ui.interact(dims_box, ui.id().with("dims-unit"), Sense::click());
        painter.rect(
            dims_box,
            CornerRadius::same(pt(2.0) as u8),
            theme::color::FIELD,
            Stroke::new(pt(1.0), Color32::from_gray(0x66)),
            StrokeKind::Inside,
        );
        // A wide, thin chevron
        let c = dims_box.center();
        painter.add(Shape::line(
            vec![
                c + vec2(pt(-5.75), pt(-2.75)),
                c + vec2(0.0, pt(3.0)),
                c + vec2(pt(5.75), pt(-2.75)),
            ],
            Stroke::new(pt(1.0), Color32::from_gray(0xdd)),
        ));
        let units: Vec<Entry> = SizeUnit::ALL[..7]
            .iter()
            .map(|&u| Entry::item(u.label(), u == self.model.dimensions_unit))
            .collect();
        if let Some(k) = native_popup::dropdown(ui, &response, ui.id().with("dims-menu"), &units) {
            self.model.dimensions_unit = SizeUnit::ALL[k];
        }
        let (dw, dh) = self.model.dimensions_text();
        let value = painter.layout_no_wrap(dw, font.clone(), LABEL);
        let x = at(459.0, 0.0).x;
        let y = at(0.0, 79.25).y;
        let w = value.size().x;
        painter.galley(Pos2::new(x, y - value.size().y / 2.0), value, LABEL);
        // A larger "×", its ink 8.5 pt clear of both sides
        let times = painter.layout_no_wrap("×".into(), theme::dialog(pt(14.0)), LABEL);
        let tx = x + w + pt(6.0);
        let tw = times.size().x;
        painter.galley(Pos2::new(tx, y - times.size().y / 2.0), times, LABEL);
        painter.text(
            Pos2::new(tx + tw + pt(5.5), y),
            Align2::LEFT_CENTER,
            dh,
            font.clone(),
            LABEL,
        );

        // Fit To
        label("Fit To:", 107.5);
        let menu = self.fit_menu();
        let entries: Vec<Entry> = menu.iter().map(|(e, _)| e.clone()).collect();
        let picked = common::field_popup(
            ui,
            r(428.0, 97.0, 655.0, 118.0),
            "fit-to",
            &self.fit_label(),
            true,
            &entries,
        );
        match picked.map(|k| menu[k].1) {
            Some(FitAction::Fit(FitTo::User(i))) => {
                if let Some((_, preset)) = self.presets.get(i).cloned() {
                    self.model.apply_preset(&preset, FitTo::User(i));
                    self.refresh(None);
                }
            }
            Some(FitAction::Fit(fit)) => {
                self.model.fit_to(fit);
                self.refresh(None);
            }
            Some(FitAction::Auto) => self.auto = Some(AutoResolutionDialog::new()),
            Some(FitAction::Custom) => self.model.fit = FitTo::Custom,
            Some(FitAction::Delete) => {
                let names = self.presets.iter().map(|(_, p)| p.name.clone()).collect();
                self.delete = Some(DeletePresetDialog::new(names));
            }
            // macOS's own panels, as Photoshop's
            Some(FitAction::Save) => {
                let mut panel = rfd::FileDialog::new()
                    .set_title("Save settings in:")
                    .set_file_name("Untitled.imz")
                    .add_filter("Image Size", &["imz"]);
                if let Some(dir) = &self.folder {
                    let _ = std::fs::create_dir_all(dir);
                    panel = panel.set_directory(dir);
                }
                if let Some(path) = panel.save_file() {
                    let _ = self.save_preset_to(&path);
                }
            }
            Some(FitAction::Load) => {
                let mut panel = rfd::FileDialog::new().add_filter("Image Size", &["imz"]);
                if let Some(dir) = &self.folder {
                    panel = panel.set_directory(dir);
                }
                if let Some(path) = panel.pick_file() {
                    self.load_preset_from(&path);
                }
            }
            Some(FitAction::Nothing) | None => {}
        }

        // Width and Height, the chain between them, and their unit
        let editable = self.model.size_editable();
        let rows = [
            (136.5, 127.0, 126.0, "Width:"),
            (165.5, 156.0, 155.0, "Height:"),
        ];
        for (k, &(cy, fy, dy, text)) in rows.iter().enumerate() {
            label(text, cy);
            let field = r(427.5, fy, 503.0, fy + 19.0);
            let buffer = if k == 0 {
                &mut self.width_text
            } else {
                &mut self.height_text
            };
            let response = if editable {
                common::text_field(
                    ui,
                    field,
                    buffer,
                    ("image-size", k),
                    font.clone(),
                    pt(4.5),
                    self.first_frame && k == 0,
                )
            } else {
                disabled_field(&painter, field, buffer);
                ui.interact(field, ui.id().with(("image-size", k)), Sense::hover())
            };
            if response.changed()
                && let Ok(v) = buffer.trim().parse::<f64>()
            {
                if k == 0 {
                    self.model.set_width(v);
                } else {
                    self.model.set_height(v);
                }
                self.refresh(Some(k as u8));
            }
            let units: Vec<Entry> = SizeUnit::ALL
                .iter()
                .map(|&u| Entry::item(u.label(), u == self.model.unit))
                .collect();
            if let Some(i) = common::field_popup(
                ui,
                r(511.0, dy, 655.0, dy + 21.0),
                if k == 0 { "width-unit" } else { "height-unit" },
                self.model.unit.label(),
                true,
                &units,
            ) {
                self.model.unit = SizeUnit::ALL[i];
                self.refresh(None);
            }
        }
        self.chain_ui(ui, column);

        // Resolution
        label("Resolution:", 194.75);
        let response = common::text_field(
            ui,
            r(427.5, 185.0, 503.0, 204.0),
            &mut self.resolution_text,
            "image-resolution",
            font.clone(),
            pt(4.5),
            false,
        );
        if response.changed()
            && let Ok(v) = self.resolution_text.trim().parse::<f64>()
        {
            self.model.set_resolution(v);
            self.refresh(Some(2));
        }
        let units: Vec<Entry> = ResolutionUnit::ALL
            .iter()
            .map(|&u| Entry::item(u.label(), u == self.model.resolution_unit))
            .collect();
        if let Some(i) = common::field_popup(
            ui,
            r(511.0, 184.0, 655.0, 205.0),
            "resolution-unit",
            self.model.resolution_unit.label(),
            true,
            &units,
        ) {
            self.model.resolution_unit = ResolutionUnit::ALL[i];
            self.refresh(None);
        }

        // Resample and its method (Alt+1 ... Alt+8 pick one)
        let mut resample = self.model.resample;
        common::ps_checkbox_with(
            ui,
            at(334.5, 217.5),
            "Resample:",
            &mut resample,
            true,
            font.clone(),
            0.0,
        );
        if resample != self.model.resample {
            self.model.set_resample(resample);
            self.refresh(None);
        }
        let mut method = self.model.method;
        let resampling = self.model.resample;
        ui.input_mut(|i| {
            if !resampling {
                return;
            }
            for (k, key) in [
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Num4,
                Key::Num5,
                Key::Num6,
                Key::Num7,
                Key::Num8,
            ]
            .into_iter()
            .enumerate()
            {
                if i.consume_key(Modifiers::ALT, key) {
                    method = Resample::ALL[k];
                }
            }
        });
        let mut methods = Vec::new();
        let mut picks = Vec::new();
        for (k, m) in Resample::ALL.into_iter().enumerate() {
            methods
                .push(Entry::item(m.label(), m == method).shortcut(format!("\u{2325}{}", k + 1)));
            picks.push(Some(m));
            if m.separator_after() {
                methods.push(Entry::Separator);
                picks.push(None);
            }
        }
        if let Some(Some(m)) = common::field_popup(
            ui,
            r(428.0, 213.0, 655.0, 234.0),
            "resample",
            method.label(),
            self.model.resample,
            &methods,
        )
        .map(|k| picks[k])
        {
            method = m;
        }
        self.model.method = method;
        if self.model.resample
            && matches!(
                method,
                Resample::PreserveDetails | Resample::PreserveDetails2
            )
        {
            self.reduce_noise_ui(ui, column);
        }

        // Generative Upscale lives in Adobe's cloud: the link is shown but
        // leads nowhere here
        painter.text(
            at(328.5, 276.5),
            Align2::LEFT_CENTER,
            "Create a new, larger document with more detail",
            font.clone(),
            LABEL,
        );
        let link = painter.layout_no_wrap("Open in Generative Upscale...".into(), font, LINK);
        let link_min = at(328.0, 287.5);
        let underline_y = at(0.0, 301.25).y;
        painter.line_segment(
            [
                Pos2::new(link_min.x, underline_y),
                Pos2::new(link_min.x + link.size().x, underline_y),
            ],
            Stroke::new(pt(1.0), LINK),
        );
        painter.galley(link_min, link, LINK);

        let values = self.model.result();
        let cancel = common::ps_button(
            ui,
            r(329.0, 311.5 + grow_h, 486.5, 337.5 + grow_h),
            "Cancel",
            false,
            true,
            false,
        );
        let ok = common::ps_button(
            ui,
            r(496.5, 311.5 + grow_h, 654.0, 337.5 + grow_h),
            "OK",
            true,
            values.is_some(),
            false,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter =
            self.auto.is_none() && self.delete.is_none() && ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter)
            && let Some((width, height, resolution)) = values
        {
            return Outcome::Apply {
                width,
                height,
                resolution,
                resample: self.model.resample.then_some(self.model.method),
                reduce_noise: self.model.reduce_noise as f32 / 100.0,
            };
        }
        Outcome::Open
    }

    /// Preserve Details' Reduce Noise row (Photoshop 2026): the label, a
    /// slider with a white pin on a 3 pt `#757575` track, the percentage.
    fn reduce_noise_ui(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let painter = ui.painter().clone();
        painter.text(
            at(LABEL_RIGHT, 251.5),
            Align2::RIGHT_CENTER,
            "Reduce Noise:",
            theme::dialog(pt(12.0)),
            LABEL,
        );
        const TRACK: (f32, f32) = (431.5, 569.5);
        // The pin's tip runs from the track's start to 5.5 pt before its end
        let span = TRACK.1 - 5.5 - TRACK.0;
        let track = Rect::from_min_max(at(TRACK.0, 248.5), at(TRACK.1, 251.5));
        painter.rect_filled(track, 0.0, Color32::from_gray(0x75));
        let hit = Rect::from_min_max(at(TRACK.0 - 6.0, 243.0), at(TRACK.1, 258.0));
        let response = ui.interact(hit, ui.id().with("reduce-noise"), Sense::click_and_drag());
        if let Some(p) = response.interact_pointer_pos()
            && (response.dragged() || response.clicked())
        {
            let t = ((p.x - at(TRACK.0, 0.0).x) / pt(span)).clamp(0.0, 1.0);
            self.model.reduce_noise = (t * 100.0).round() as f64;
            self.noise_text = format!("{}", self.model.reduce_noise);
        }
        let x = TRACK.0 + span * self.model.reduce_noise as f32 / 100.0;
        super::appkit::pin(&painter, at(x, 245.5), super::appkit::Pin::White);
        let field = Rect::from_min_max(at(580.5, 241.5), at(641.0, 260.5));
        let response = common::text_field(
            ui,
            field,
            &mut self.noise_text,
            "reduce-noise-field",
            theme::dialog(pt(12.0)),
            pt(4.5),
            false,
        );
        if response.changed()
            && let Ok(v) = self.noise_text.trim().parse::<f64>()
        {
            self.model.reduce_noise = v.clamp(0.0, 100.0);
        }
        painter.text(
            at(643.5, 251.5),
            Align2::LEFT_CENTER,
            "%",
            theme::dialog(pt(12.0)),
            LABEL,
        );
    }

    /// The chain button between Width and Height with its bracket.
    fn chain_ui(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let painter = ui.painter().clone();
        let stroke = Stroke::new(pt(1.0), BRACKET);
        // Top: right to the Width label, down to the button
        painter.line_segment([at(355.75, 135.75), at(373.0, 135.75)], stroke);
        painter.line_segment([at(356.25, 135.25), at(356.25, 140.0)], stroke);
        painter.line_segment([at(356.25, 162.0), at(356.25, 167.25)], stroke);
        painter.line_segment([at(355.75, 166.75), at(369.0, 166.75)], stroke);
        let button = Rect::from_min_max(at(349.0, 142.0), at(365.0, 160.0));
        let sense = if self.model.resample {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(button, ui.id().with("constrain"), sense);
        painter.rect(
            button,
            CornerRadius::same(pt(3.0) as u8),
            BOX_FILL,
            Stroke::new(pt(1.0), BOX_EDGE),
            StrokeKind::Inside,
        );
        chain_icon(
            &painter,
            button.center() + vec2(pt(0.5), pt(0.5)),
            self.model.constrain,
        );
        if response.clicked() {
            self.model.set_constrain(!self.model.constrain);
            self.refresh(None);
        }
    }

    /// The document at 100% (one image pixel per device pixel), centered;
    /// dragging pans it.
    fn preview_ui(&mut self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter().clone();
        painter.rect_filled(rect, 0, Color32::from_gray(0x42));
        let inner = rect.shrink(pt(1.0));
        painter.rect_filled(inner, 0, Color32::WHITE);
        let Some(preview) = &self.preview else {
            return;
        };
        let ppp = ui.ctx().pixels_per_point();
        let (pw, ph) = (
            (inner.width() * ppp).round() as i64,
            (inner.height() * ppp).round() as i64,
        );
        let response = ui.interact(inner, ui.id().with("preview"), Sense::drag());
        if response.dragged() {
            let d = response.drag_delta() * ppp;
            self.center.0 -= d.x;
            self.center.1 -= d.y;
        }
        // Keep the image over the preview where it can be
        let (iw, ih) = (preview.width as f32, preview.height as f32);
        let clamp = |c: f32, image: f32, view: f32| {
            if image <= view {
                image / 2.0
            } else {
                c.clamp(view / 2.0, image - view / 2.0)
            }
        };
        self.center = (
            clamp(self.center.0, iw, pw as f32),
            clamp(self.center.1, ih, ph as f32),
        );
        let x0 = (self.center.0 - pw as f32 / 2.0).round() as i64;
        let y0 = (self.center.1 - ph as f32 / 2.0).round() as i64;
        let stale = self.texture.as_ref().is_none_or(|(_, at)| *at != (x0, y0));
        if stale {
            let mut pixels = vec![0u8; (pw * ph * 4) as usize];
            for y in 0..ph {
                for x in 0..pw {
                    let (sx, sy) = (x0 + x, y0 + y);
                    let o = ((y * pw + x) * 4) as usize;
                    // Transparency shows Photoshop's checkerboard
                    let check = if ((x / 8) + (y / 8)) % 2 == 0 {
                        255
                    } else {
                        204
                    };
                    let mut px = [check, check, check, 255];
                    if sx >= 0 && sy >= 0 && sx < iw as i64 && sy < ih as i64 {
                        let i = ((sy * iw as i64 + sx) * 4) as usize;
                        let s = &preview.rgba[i..i + 4];
                        let a = s[3] as u32;
                        for c in 0..3 {
                            px[c] =
                                ((s[c] as u32 * a + px[c] as u32 * (255 - a) + 127) / 255) as u8;
                        }
                    } else {
                        px = [255; 4];
                    }
                    pixels[o..o + 4].copy_from_slice(&px);
                }
            }
            let image =
                egui::ColorImage::from_rgba_unmultiplied([pw as usize, ph as usize], &pixels);
            let texture =
                ui.ctx()
                    .load_texture("image-size-preview", image, egui::TextureOptions::NEAREST);
            self.texture = Some((texture, (x0, y0)));
        }
        if let Some((texture, _)) = &self.texture {
            painter.image(
                texture.id(),
                inner,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
}

/// A disabled size field: Pixels and Percent with Resample off.
fn disabled_field(painter: &egui::Painter, rect: Rect, text: &str) {
    painter.rect(
        rect,
        0,
        Color32::from_gray(0x4e),
        Stroke::new(pt(1.0), Color32::from_gray(0x5e)),
        StrokeKind::Inside,
    );
    painter.text(
        rect.left_center() + vec2(pt(4.5), 0.0),
        Align2::LEFT_CENTER,
        text,
        theme::dialog(pt(12.0)),
        Color32::from_gray(0x88),
    );
}

/// The window's close, minimize and zoom buttons: a modal window, so only
/// zoom (it can be resized) is in color.
fn traffic_lights(painter: &egui::Painter, frame: Rect) {
    for (x, fill, edge) in [
        (14.0, Color32::from_gray(0xba), Color32::from_gray(0x99)),
        (34.0, Color32::from_gray(0xba), Color32::from_gray(0x99)),
        (
            54.0,
            Color32::from_rgb(0x61, 0xc5, 0x54),
            Color32::from_rgb(0x4e, 0x9a, 0x45),
        ),
    ] {
        let c = frame.min + vec2(pt(x), pt(14.0));
        painter.circle(c, pt(6.0), fill, Stroke::new(pt(0.5), edge));
    }
}

/// The gear with its small menu triangle, traced from Photoshop 2026.
fn gear_icon(painter: &egui::Painter, center: Pos2) {
    let c = center + vec2(pt(-2.5), pt(-0.5));
    let color = Color32::from_gray(0xd6);
    for k in 0..8 {
        let a = std::f32::consts::TAU * k as f32 / 8.0;
        let dir = vec2(a.cos(), a.sin());
        painter.add(Shape::line_segment(
            [c + dir * pt(3.0), c + dir * pt(5.25)],
            Stroke::new(pt(2.5), color),
        ));
    }
    painter.circle(c, pt(3.0), color, Stroke::NONE);
    painter.circle_filled(c, pt(1.6), Color32::from_gray(0x53));
    let t = c + vec2(pt(7.5), pt(4.5));
    painter.add(Shape::convex_polygon(
        vec![
            t + vec2(pt(-1.5), 0.0),
            t + vec2(pt(1.5), 0.0),
            t + vec2(0.0, pt(1.5)),
        ],
        color,
        Stroke::NONE,
    ));
}

/// Photoshop's chain, traced at 2x: two links open toward each other
/// with a bar through them; the bar is gone when the proportions aren't
/// linked.
fn chain_icon(painter: &egui::Painter, center: Pos2, linked: bool) {
    let color = Color32::from_gray(0xd7);
    let stroke = Stroke::new(pt(1.0), color);
    // The top link, from its inner bottom-left round to its bottom-right
    let top: Vec<(f32, f32)> = std::iter::once((-1.75, -1.0))
        .chain(std::iter::once((-3.0, -2.75)))
        .chain((0..=12).map(|k| {
            let a = std::f32::consts::PI * (1.0 + k as f32 / 12.0);
            (3.0 * a.cos(), -4.5 + 3.0 * a.sin())
        }))
        .chain([(3.0, -2.75), (1.75, -1.0)])
        .collect();
    for flip in [1.0f32, -1.0] {
        let points = top
            .iter()
            .map(|&(x, y)| center + vec2(pt(x), pt(if flip > 0.0 { y } else { -1.0 - y })))
            .collect();
        painter.add(Shape::line(points, stroke));
    }
    if linked {
        painter.line_segment(
            [center + vec2(0.0, pt(-3.75)), center + vec2(0.0, pt(2.25))],
            Stroke::new(pt(1.5), color),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 734 × 811 at 72 ppi, as in the Photoshop 2026 capture.
    fn model() -> Model {
        Model::new(734, 811, 72.0)
    }

    #[test]
    fn opens_like_photoshop() {
        let m = model();
        assert_eq!(m.field_text(m.width, 734), "10.194");
        assert_eq!(m.field_text(m.height, 811), "11.264");
        assert_eq!(m.memory_text(), "1.70M");
        assert_eq!(
            m.dimensions_text(),
            ("734 px".to_string(), "811 px".to_string())
        );
        assert_eq!(m.method, Resample::Automatic);
        assert_eq!(m.result(), Some((734, 811, 72.0)));
    }

    #[test]
    fn the_chain_and_units() {
        let mut m = model();
        m.unit = SizeUnit::Pixels;
        m.set_width(367.0);
        assert_eq!(m.result(), Some((367, 406, 72.0)));
        assert_eq!(m.memory_text(), "436.5K (was 1.70M)");
        m.unit = SizeUnit::Percent;
        assert_eq!(m.field_text(m.width, 734), "50");
        m.set_constrain(false);
        m.set_height(100.0);
        assert_eq!(m.result(), Some((367, 811, 72.0)));
        // Centimeters at 72 ppi: 2.54 cm is 72 px
        m.unit = SizeUnit::Centimeters;
        m.set_width(2.54);
        assert_eq!(m.result().unwrap().0, 72);
    }

    #[test]
    fn resolution_keeps_the_printed_size_or_the_pixels() {
        // Inches: 144 ppi doubles the pixels
        let mut m = model();
        m.set_resolution(144.0);
        assert_eq!(m.result(), Some((1468, 1622, 144.0)));
        // Pixels: the pixels stay
        let mut m = model();
        m.unit = SizeUnit::Pixels;
        m.set_resolution(144.0);
        assert_eq!(m.result(), Some((734, 811, 144.0)));
        // Resample off: a printed size sets the resolution
        let mut m = model();
        m.set_resample(false);
        m.set_width(734.0 / 144.0);
        let (w, h, r) = m.result().unwrap();
        assert_eq!((w, h), (734, 811));
        assert!((r - 144.0).abs() < 0.01);
        // ...and pixels can't be typed
        m.unit = SizeUnit::Pixels;
        assert!(!m.size_editable());
        m.set_width(10.0);
        assert_eq!(m.result().unwrap().0, 734);
        // Pixels per centimeter
        let mut m = model();
        m.resolution_unit = ResolutionUnit::PerCentimeter;
        assert_eq!(number(m.resolution_in_unit()), "28.346");
    }

    #[test]
    fn fit_to_presets() {
        // 1024 × 768 turned upright for a portrait image: 768 × 1024 box
        let mut m = model();
        m.fit_to(FIT_PRESETS[0][1]);
        assert_eq!(m.result(), Some((768, 849, 72.0)));
        assert_eq!(m.unit, SizeUnit::Pixels);
        // 4 × 6 in at 300 dpi: 1200 × 1800 box
        m.fit_to(FIT_PRESETS[2][0]);
        assert_eq!(m.result(), Some((1200, 1326, 300.0)));
        assert_eq!(m.fit.label(), "4 x 6 in 300 dpi");
        // Typing makes it Custom; Original Size puts it back
        m.set_width(2.0);
        assert_eq!(m.fit, FitTo::Custom);
        m.fit_to(FitTo::Original);
        assert_eq!(m.result(), Some((734, 811, 72.0)));
    }

    #[test]
    fn numbers_trim_zeros() {
        assert_eq!(number(10.0), "10");
        assert_eq!(number(10.1944), "10.194");
        assert_eq!(number(0.5), "0.5");
    }
}
