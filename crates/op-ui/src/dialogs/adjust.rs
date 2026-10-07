//! Dialogs for adjustments and filters with settings: Image > Adjustments >
//! Threshold..., Posterize..., Levels..., Hue/Saturation..., Exposure...,
//! and the Filter menu's Gaussian Blur..., Box Blur..., Unsharp Mask...,
//! Add Noise..., Median..., Minimum..., Maximum..., High Pass..., Offset...
//! and Mosaic....
//!
//! Each setting is a parameter: a number with a range, a text field and
//! (mostly) a slider; a choice shown as radio buttons; or a checkbox. Laid
//! out after Photoshop's dialogs; sizes are in Photoshop points. With
//! Preview on, the document shows the result while the dialog is open.

use egui::{Align2, Color32, Key, Pos2, Rect, Sense, Stroke, Ui, vec2};
use op_core::adjust::Adjustment;
use op_core::filter::{
    DiffuseMode, Filter, MezzotintType, OffsetFill, RippleSize, SpherizeMode, TilesFill, WindMethod,
};

use super::{
    appkit, black_white, brightness_contrast, channel_mixer, color_balance, common, curves,
    custom_filter, distort, exposure, filter_layout, gradient_map, hue_saturation, levels,
    photo_filter, plain_filter, selective_color, threshold, uxp, vibrance,
};
use crate::theme::{self, pt};

/// What a dialog applies: an adjustment or a filter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Adjustment(Adjustment),
    Filter(Filter),
}

impl Effect {
    pub fn name(self) -> &'static str {
        match self {
            Self::Adjustment(a) => a.name(),
            Self::Filter(f) => f.name(),
        }
    }

    /// Applies to the active layer; `background` is the background color
    /// (used by Offset on the background layer).
    pub fn apply(
        self,
        doc: &mut op_core::Document,
        background: [u8; 3],
    ) -> Result<(), op_core::fill::FillError> {
        match self {
            Self::Adjustment(a) => op_core::adjust::apply(doc, a),
            Self::Filter(f) => op_core::filter::apply(doc, f, background),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ParamKind {
    Number,
    /// Radio buttons; the value is the chosen index.
    Choice(&'static [&'static str]),
    /// A checkbox; the value is 0 or 1.
    Check,
}

/// One setting.
struct Param {
    label: &'static str,
    min: f32,
    max: f32,
    default: f32,
    decimals: usize,
    kind: ParamKind,
}

const fn param(label: &'static str, min: f32, max: f32, default: f32, decimals: usize) -> Param {
    Param {
        label,
        min,
        max,
        default,
        decimals,
        kind: ParamKind::Number,
    }
}

const fn choice(label: &'static str, options: &'static [&'static str], default: usize) -> Param {
    Param {
        label,
        min: 0.0,
        max: (options.len() - 1) as f32,
        default: default as f32,
        decimals: 0,
        kind: ParamKind::Choice(options),
    }
}

const fn check(label: &'static str, default: bool) -> Param {
    Param {
        label,
        min: 0.0,
        max: 1.0,
        default: default as u8 as f32,
        decimals: 0,
        kind: ParamKind::Check,
    }
}

const GAUSSIAN_BLUR: &[Param] = &[param("Radius (pixels):", 0.1, 1000.0, 1.0, 1)];
const BOX_BLUR: &[Param] = &[param("Radius (pixels):", 1.0, 2000.0, 1.0, 0)];
const UNSHARP_MASK: &[Param] = &[
    param("Amount (%):", 1.0, 500.0, 50.0, 0),
    param("Radius (pixels):", 0.1, 1000.0, 1.0, 1),
    param("Threshold (levels):", 0.0, 255.0, 0.0, 0),
];
const ADD_NOISE: &[Param] = &[
    param("Amount (%):", 0.1, 400.0, 12.5, 2),
    choice("Distribution", &["Uniform", "Gaussian"], 0),
    check("Monochromatic", false),
];
const RADIUS: &[Param] = &[param("Radius (pixels):", 1.0, 500.0, 1.0, 0)];
/// Minimum and Maximum: a fractional radius and Preserve.
const RANK_RADIUS: &[Param] = &[
    param("Radius (pixels):", 0.2, 500.0, 1.0, 1),
    choice("Preserve", &["Squareness", "Roundness"], 0),
];
const MOTION_BLUR: &[Param] = &[
    param("Angle (°):", -360.0, 360.0, 0.0, 0),
    param("Distance (pixels):", 1.0, 2000.0, 10.0, 0),
];
const TWIRL: &[Param] = &[param("Angle (°):", -999.0, 999.0, 50.0, 0)];
const CRYSTALLIZE: &[Param] = &[param("Cell Size", 3.0, 300.0, 10.0, 0)];
const POINTILLIZE: &[Param] = &[param("Cell Size", 3.0, 300.0, 5.0, 0)];
const RIPPLE: &[Param] = &[
    param("Amount (%):", -999.0, 999.0, 100.0, 0),
    choice("Size", &["Small", "Medium", "Large"], 1),
];
const MEZZOTINT: &[Param] = &[choice(
    "Type",
    &[
        "Fine Dots",
        "Medium Dots",
        "Grainy Dots",
        "Coarse Dots",
        "Short Lines",
        "Medium Lines",
        "Long Lines",
        "Short Strokes",
        "Medium Strokes",
        "Long Strokes",
    ],
    0,
)];
const TILES: &[Param] = &[
    param("Number Of Tiles:", 1.0, 99.0, 10.0, 0),
    param("Maximum Offset (%):", 1.0, 99.0, 10.0, 0),
    choice(
        "Fill Empty Area With",
        &[
            "Background Color",
            "Foreground Color",
            "Inverse Image",
            "Unaltered Image",
        ],
        0,
    ),
];
const COLOR_HALFTONE: &[Param] = &[
    param("Max. Radius (pixels):", 4.0, 127.0, 8.0, 0),
    param("Channel 1:", -360.0, 360.0, 108.0, 0),
    param("Channel 2:", -360.0, 360.0, 162.0, 0),
    param("Channel 3:", -360.0, 360.0, 90.0, 0),
    param("Channel 4:", -360.0, 360.0, 45.0, 0),
];
const DIFFUSE: &[Param] = &[choice(
    "Mode",
    &["Normal", "Darken Only", "Lighten Only", "Anisotropic"],
    0,
)];
const PINCH: &[Param] = &[param("Amount (%):", -100.0, 100.0, 50.0, 0)];
const SPHERIZE: &[Param] = &[
    param("Amount (%):", -100.0, 100.0, 100.0, 0),
    choice("Mode", &["Normal", "Horizontal only", "Vertical only"], 0),
];
const POLAR: &[Param] = &[choice(
    "Options",
    &["Rectangular to Polar", "Polar to Rectangular"],
    0,
)];
const EMBOSS: &[Param] = &[
    param("Angle (°):", -180.0, 180.0, 135.0, 0),
    param("Height (pixels):", 1.0, 100.0, 3.0, 0),
    param("Amount (%):", 1.0, 500.0, 100.0, 0),
];
const HIGH_PASS: &[Param] = &[param("Radius (pixels):", 0.1, 1000.0, 10.0, 1)];
const OFFSET: &[Param] = &[
    param("Horizontal (pixels right):", -30000.0, 30000.0, 0.0, 0),
    param("Vertical (pixels down):", -30000.0, 30000.0, 0.0, 0),
    choice(
        "Undefined Areas",
        &["Set to Transparent", "Repeat Edge Pixels", "Wrap Around"],
        0,
    ),
];
const MOSAIC: &[Param] = &[param("Cell Size (square):", 2.0, 200.0, 10.0, 0)];
const SURFACE_BLUR: &[Param] = &[
    param("Radius (pixels):", 1.0, 100.0, 5.0, 0),
    param("Threshold (levels):", 2.0, 255.0, 15.0, 0),
];
const TRACE_CONTOUR: &[Param] = &[
    param("Level:", 0.0, 255.0, 128.0, 0),
    choice("Edge", &["Lower", "Upper"], 1),
];
const WIND: &[Param] = &[
    choice("Method", &["Wind", "Blast", "Stagger"], 0),
    choice("Direction", &["From the Right", "From the Left"], 0),
];
const DUST_AND_SCRATCHES: &[Param] = &[
    param("Radius (pixels):", 1.0, 500.0, 1.0, 0),
    param("Threshold (levels):", 0.0, 255.0, 0.0, 0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Threshold,
    Posterize,
    Levels,
    Curves,
    HueSaturation,
    Exposure,
    BrightnessContrast,
    ColorBalance,
    BlackWhite,
    Vibrance,
    ChannelMixer,
    SelectiveColor,
    PhotoFilter,
    GradientMap,
    GaussianBlur,
    BoxBlur,
    UnsharpMask,
    AddNoise,
    Median,
    Minimum,
    Maximum,
    HighPass,
    Offset,
    Mosaic,
    MotionBlur,
    Emboss,
    Twirl,
    Crystallize,
    Pointillize,
    Diffuse,
    Ripple,
    Mezzotint,
    Tiles,
    ColorHalftone,
    Pinch,
    Spherize,
    PolarCoordinates,
    SurfaceBlur,
    DustAndScratches,
    /// Filter > Other > Custom...
    Custom,
    TraceContour,
    Wind,
}

impl Kind {
    fn title(self) -> &'static str {
        match self {
            Self::Threshold => "Threshold",
            Self::Posterize => "Posterize",
            Self::Levels => "Levels",
            Self::Curves => "Curves",
            Self::HueSaturation => "Hue/Saturation",
            Self::Exposure => "Exposure",
            Self::BrightnessContrast => "Brightness/Contrast",
            Self::ColorBalance => "Color Balance",
            Self::BlackWhite => "Black and White",
            Self::Vibrance => "Vibrance",
            Self::ChannelMixer => "Channel Mixer",
            Self::SelectiveColor => "Selective Color",
            Self::PhotoFilter => "Photo Filter",
            Self::GradientMap => "Gradient Map",
            Self::GaussianBlur => "Gaussian Blur",
            Self::BoxBlur => "Box Blur",
            Self::UnsharpMask => "Unsharp Mask",
            Self::AddNoise => "Add Noise",
            Self::Median => "Median",
            Self::Minimum => "Minimum",
            Self::Maximum => "Maximum",
            Self::HighPass => "High Pass",
            Self::Offset => "Offset",
            Self::Mosaic => "Mosaic",
            Self::MotionBlur => "Motion Blur",
            Self::Emboss => "Emboss",
            Self::Twirl => "Twirl",
            Self::Crystallize => "Crystallize",
            Self::Pointillize => "Pointillize",
            Self::Diffuse => "Diffuse",
            Self::Ripple => "Ripple",
            Self::Mezzotint => "Mezzotint",
            Self::Tiles => "Tiles",
            Self::ColorHalftone => "Color Halftone",
            Self::Pinch => "Pinch",
            Self::Spherize => "Spherize",
            Self::PolarCoordinates => "Polar Coordinates",
            Self::SurfaceBlur => "Surface Blur",
            Self::DustAndScratches => "Dust & Scratches",
            Self::Custom => "Custom",
            Self::TraceContour => "Trace Contour",
            Self::Wind => "Wind",
        }
    }

    fn params(self) -> &'static [Param] {
        match self {
            // Their own dialogs keep their settings
            Self::Levels
            | Self::Curves
            | Self::HueSaturation
            | Self::BrightnessContrast
            | Self::ColorBalance
            | Self::ChannelMixer
            | Self::SelectiveColor
            | Self::Vibrance
            | Self::Posterize
            | Self::Exposure
            | Self::PhotoFilter
            | Self::BlackWhite
            | Self::Threshold
            | Self::GradientMap
            | Self::Custom => &[],
            Self::GaussianBlur => GAUSSIAN_BLUR,
            Self::BoxBlur => BOX_BLUR,
            Self::UnsharpMask => UNSHARP_MASK,
            Self::AddNoise => ADD_NOISE,
            Self::Median => RADIUS,
            Self::Minimum | Self::Maximum => RANK_RADIUS,
            Self::HighPass => HIGH_PASS,
            Self::Offset => OFFSET,
            Self::Mosaic => MOSAIC,
            Self::MotionBlur => MOTION_BLUR,
            Self::Emboss => EMBOSS,
            Self::Twirl => TWIRL,
            Self::Crystallize => CRYSTALLIZE,
            Self::Pointillize => POINTILLIZE,
            Self::Diffuse => DIFFUSE,
            Self::Ripple => RIPPLE,
            Self::Mezzotint => MEZZOTINT,
            Self::Tiles => TILES,
            Self::ColorHalftone => COLOR_HALFTONE,
            Self::Pinch => PINCH,
            Self::Spherize => SPHERIZE,
            Self::PolarCoordinates => POLAR,
            Self::SurfaceBlur => SURFACE_BLUR,
            Self::DustAndScratches => DUST_AND_SCRATCHES,
            Self::TraceContour => TRACE_CONTOUR,
            Self::Wind => WIND,
        }
    }

    /// The classic Photoshop layout of a filter's dialog, where rebuilt.
    fn layout(self) -> Option<&'static filter_layout::Layout> {
        use filter_layout as l;
        Some(match self {
            Self::GaussianBlur => l::GAUSSIAN_BLUR,
            Self::HighPass => l::HIGH_PASS,
            Self::BoxBlur => l::BOX_BLUR,
            Self::Median => l::MEDIAN,
            Self::Minimum => l::MINIMUM,
            Self::Maximum => l::MAXIMUM,
            Self::UnsharpMask => l::UNSHARP_MASK,
            Self::AddNoise => l::ADD_NOISE,
            Self::Mosaic => l::MOSAIC,
            Self::MotionBlur => l::MOTION_BLUR,
            Self::Emboss => l::EMBOSS,
            Self::SurfaceBlur => l::SURFACE_BLUR,
            Self::DustAndScratches => l::DUST_AND_SCRATCHES,
            Self::Offset => l::OFFSET,
            Self::TraceContour => l::TRACE_CONTOUR,
            Self::Diffuse => l::DIFFUSE,
            _ => return None,
        })
    }

    /// The small dialogs without a preview.
    fn plain(self) -> Option<&'static plain_filter::Layout> {
        Some(match self {
            Self::Tiles => &plain_filter::TILES,
            Self::ColorHalftone => &plain_filter::COLOR_HALFTONE,
            _ => return None,
        })
    }

    /// The Distort filters' plug-in style layout.
    fn distort(self) -> Option<&'static distort::Layout> {
        Some(match self {
            Self::Twirl => &distort::TWIRL,
            Self::Crystallize | Self::Pointillize => &distort::CELL_SIZE,
            Self::Ripple => &distort::RIPPLE,
            Self::Mezzotint => &distort::MEZZOTINT,
            Self::Pinch => &distort::PINCH,
            Self::Spherize => &distort::SPHERIZE,
            Self::PolarCoordinates => &distort::POLAR,
            Self::Wind => &distort::WIND,
            _ => return None,
        })
    }

    /// A filter dialog's size, from its classic or plug-in style layout
    /// (the adjustments' own dialogs have their own sizes).
    fn size(self) -> egui::Vec2 {
        let (w, h) = self
            .layout()
            .map(|l| l.size)
            .or(self.distort().map(|l| l.size))
            .or(self.plain().map(|l| l.size))
            .expect("every filter dialog has a layout");
        vec2(pt(w), pt(h))
    }
}

fn text_width(ui: &Ui, text: &str) -> f32 {
    theme::tracked_galley(ui.painter(), text, appkit::font(), appkit::TEXT)
        .size()
        .x
        / pt(1.0)
}

/// Custom's Load...: a Photoshop .acf kernel file.
fn load_kernel(d: &mut custom_filter::Dialog) {
    let path = rfd::FileDialog::new()
        .add_filter("Custom Filter", &["acf"])
        .pick_file();
    if let Some(bytes) = path.and_then(|p| std::fs::read(p).ok()) {
        d.load_acf(&bytes);
    }
}

/// Custom's Save...: the kernel as a Photoshop .acf file.
fn save_kernel(d: &custom_filter::Dialog) {
    let Some(bytes) = d.to_acf() else {
        return;
    };
    let path = rfd::FileDialog::new()
        .add_filter("Custom Filter", &["acf"])
        .set_file_name("Untitled.acf")
        .save_file();
    if let Some(path) = path {
        let _ = std::fs::write(path, bytes);
    }
}

/// The zoom controls under the preview pane: zoom out (dimmed at 100%),
/// the zoom, zoom in.
fn zoom_controls(ui: &Ui, y: f32, left: f32) {
    let painter = ui.painter();
    let x = |v: f32| left + pt(v);
    appkit::text(
        ui,
        Pos2::new(x(113.75), y),
        Align2::CENTER_CENTER,
        "100%",
        appkit::TEXT,
    );
    for (cx, plus, tint) in [
        (57.0, false, Color32::from_gray(0x80)),
        (171.75, true, appkit::TEXT),
    ] {
        let c = Pos2::new(x(cx) - pt(1.5), y - pt(1.5));
        let stroke = Stroke::new(pt(1.0), tint);
        painter.circle_stroke(c, pt(4.5), stroke);
        painter.line_segment(
            [c + vec2(pt(3.3), pt(3.3)), c + vec2(pt(6.5), pt(6.5))],
            Stroke::new(pt(1.5), tint),
        );
        painter.line_segment([c - vec2(pt(2.2), 0.0), c + vec2(pt(2.2), 0.0)], stroke);
        if plus {
            painter.line_segment([c - vec2(0.0, pt(2.2)), c + vec2(0.0, pt(2.2))], stroke);
        }
    }
}

pub enum Outcome {
    Open,
    Cancel,
    Apply(Effect),
}

pub struct AdjustDialog {
    pub kind: Kind,
    values: Vec<String>,
    pub preview: bool,
    /// Histogram of the pixels being adjusted (Threshold and Levels).
    histogram: [u64; 256],
    first_frame: bool,
    /// The effect the document currently previews, if any.
    pub previewing: Option<Effect>,
    /// The document before any preview, restored on Cancel.
    pub before: op_core::Snapshot,
    /// The dialogs rebuilt after Photoshop 2026, which keep their own
    /// settings.
    custom: Option<Custom>,
    /// The classic filter dialogs' preview pane image (the document as
    /// previewed), set by the app.
    pub pane: Option<egui::TextureHandle>,
    /// The Distort dialogs' diagram and the settings it was drawn for.
    diagram: Option<(Vec<String>, egui::TextureHandle)>,
}

/// A dialog with its own layout and settings.
enum Custom {
    BrightnessContrast(brightness_contrast::Dialog),
    ColorBalance(color_balance::Dialog),
    HueSaturation(Box<hue_saturation::Dialog>),
    Levels(Box<levels::Dialog>),
    Curves(Box<curves::Dialog>),
    ChannelMixer(Box<channel_mixer::Dialog>),
    SelectiveColor(Box<selective_color::Dialog>),
    Vibrance(vibrance::Vibrance),
    Posterize(vibrance::Posterize),
    Exposure(exposure::Dialog),
    PhotoFilter(photo_filter::Dialog),
    BlackWhite(Box<black_white::Dialog>),
    Threshold(Box<threshold::Dialog>),
    GradientMap(gradient_map::Dialog),
    Kernel(Box<custom_filter::Dialog>),
}

/// Some of Photoshop's fields drop trailing zeros (Add Noise shows 12.5,
/// Minimum shows 1) where others keep them (Gaussian Blur shows 1.0).
fn format(kind: Kind, v: f32, decimals: usize) -> String {
    let s = format!("{v:.decimals$}");
    let trims = matches!(kind, Kind::AddNoise | Kind::Minimum | Kind::Maximum);
    if trims && s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

impl AdjustDialog {
    pub fn new(kind: Kind, histogram: [u64; 256], before: op_core::Snapshot) -> Self {
        Self {
            kind,
            values: kind
                .params()
                .iter()
                .map(|p| format(kind, p.default, p.decimals))
                .collect(),
            // The dialogs without a preview don't preview on the document
            preview: kind.plain().is_none(),
            histogram,
            first_frame: true,
            previewing: None,
            before,
            pane: None,
            diagram: None,
            custom: match kind {
                Kind::BrightnessContrast => Some(Custom::BrightnessContrast(Default::default())),
                Kind::ColorBalance => Some(Custom::ColorBalance(Default::default())),
                Kind::HueSaturation => Some(Custom::HueSaturation(Box::new(
                    hue_saturation::Dialog::new(0),
                ))),
                Kind::Levels => Some(Custom::Levels(Box::new(levels::Dialog::new([[0; 256]; 3])))),
                Kind::ChannelMixer => Some(Custom::ChannelMixer(Default::default())),
                Kind::Vibrance => Some(Custom::Vibrance(Default::default())),
                Kind::Posterize => Some(Custom::Posterize(Default::default())),
                Kind::Exposure => Some(Custom::Exposure(Default::default())),
                Kind::PhotoFilter => Some(Custom::PhotoFilter(Default::default())),
                Kind::BlackWhite => Some(Custom::BlackWhite(Default::default())),
                Kind::Threshold => Some(Custom::Threshold(Box::new(threshold::Dialog::new(
                    histogram,
                )))),
                Kind::GradientMap => Some(Custom::GradientMap(gradient_map::Dialog::new((
                    [0; 3], [255; 3],
                )))),
                Kind::SelectiveColor => Some(Custom::SelectiveColor(Default::default())),
                Kind::Custom => Some(Custom::Kernel(Default::default())),
                Kind::Curves => Some(Custom::Curves(Box::new(curves::Dialog::new([[0; 256]; 3])))),
                _ => None,
            },
        }
    }

    /// Levels' (and Curves') per-channel histograms.
    pub fn set_channel_histograms(&mut self, histograms: [[u64; 256]; 3]) {
        match &mut self.custom {
            Some(Custom::Levels(d)) => **d = levels::Dialog::new(histograms),
            Some(Custom::Curves(d)) => **d = curves::Dialog::new(histograms),
            _ => {}
        }
    }

    /// The filter dialogs' settings as typed, to open with next time
    /// (Photoshop's filter dialogs remember their last values); `None` for
    /// the adjustments' own dialogs.
    pub fn settings(&self) -> Option<Vec<String>> {
        match &self.custom {
            None => Some(self.values.clone()),
            Some(Custom::Kernel(d)) => Some(d.settings()),
            Some(_) => None,
        }
    }

    /// Puts back settings from [`settings`](Self::settings).
    pub fn restore(&mut self, values: &[String]) {
        match &mut self.custom {
            None if values.len() == self.values.len() => self.values = values.to_vec(),
            Some(Custom::Kernel(d)) => d.restore(values),
            _ => {}
        }
    }

    /// Whether this dialog shows the classic preview pane.
    pub fn wants_pane(&self) -> bool {
        self.kind == Kind::Custom
            || self.kind.distort().is_some()
            || self.kind.layout().is_some_and(|l| l.pane)
    }

    /// Gradient Map's two colors (the foreground and background colors).
    pub fn set_gradient_colors(&mut self, colors: ([u8; 3], [u8; 3])) {
        if let Some(Custom::GradientMap(d)) = &mut self.custom {
            d.colors = colors;
        }
    }

    /// Hue/Saturation's Colorize starts from this hue.
    pub fn set_colorize_hue(&mut self, hue: i32) {
        if let Some(Custom::HueSaturation(d)) = &mut self.custom {
            d.colorize_values[0] = hue.to_string();
        }
    }

    fn value(&self, i: usize) -> Option<f32> {
        let p = &self.kind.params()[i];
        let v: f32 = self.values[i].trim().parse().ok()?;
        (p.min..=p.max).contains(&v).then_some(v)
    }

    fn set(&mut self, i: usize, v: f32) {
        let p = &self.kind.params()[i];
        self.values[i] = format(self.kind, v.clamp(p.min, p.max), p.decimals);
    }

    /// The effect as currently set, if every value is valid.
    pub fn effect(&self) -> Option<Effect> {
        match &self.custom {
            Some(Custom::BrightnessContrast(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::ColorBalance(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::HueSaturation(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::Levels(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::Curves(d)) => return Some(Effect::Adjustment(d.adjustment())),
            Some(Custom::ChannelMixer(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::SelectiveColor(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::Vibrance(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::Posterize(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::Exposure(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::PhotoFilter(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::BlackWhite(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::Threshold(d)) => return d.adjustment().map(Effect::Adjustment),
            Some(Custom::GradientMap(d)) => return Some(Effect::Adjustment(d.adjustment())),
            Some(Custom::Kernel(d)) => return d.filter().map(Effect::Filter),
            None => {}
        }
        let v: Vec<f32> = (0..self.values.len())
            .map(|i| self.value(i))
            .collect::<Option<_>>()?;
        let filter = match self.kind {
            Kind::GaussianBlur => Filter::GaussianBlur { radius: v[0] },
            Kind::BoxBlur => Filter::BoxBlur {
                radius: v[0] as u32,
            },
            Kind::UnsharpMask => Filter::UnsharpMask {
                amount: v[0],
                radius: v[1],
                threshold: v[2] as u8,
            },
            Kind::AddNoise => Filter::AddNoise {
                amount: v[0],
                gaussian: v[1] == 1.0,
                monochromatic: v[2] == 1.0,
            },
            Kind::Median => Filter::Median {
                radius: v[0] as u32,
            },
            Kind::Minimum => Filter::Minimum {
                radius: v[0],
                round: v[1] == 1.0,
            },
            Kind::Maximum => Filter::Maximum {
                radius: v[0],
                round: v[1] == 1.0,
            },
            Kind::HighPass => Filter::HighPass { radius: v[0] },
            Kind::Offset => Filter::Offset {
                dx: v[0] as i32,
                dy: v[1] as i32,
                fill: [
                    OffsetFill::Background,
                    OffsetFill::RepeatEdges,
                    OffsetFill::Wrap,
                ][v[2] as usize],
            },
            Kind::Mosaic => Filter::Mosaic { cell: v[0] as u32 },
            Kind::MotionBlur => Filter::MotionBlur {
                angle: v[0] as i32,
                distance: v[1] as u32,
            },
            Kind::Twirl => Filter::Twirl { angle: v[0] as i32 },
            Kind::Crystallize => Filter::Crystallize { cell: v[0] as u32 },
            Kind::Ripple => Filter::Ripple {
                amount: v[0] as i32,
                size: [RippleSize::Small, RippleSize::Medium, RippleSize::Large][v[1] as usize],
            },
            Kind::Tiles => Filter::Tiles {
                count: v[0] as u32,
                offset: v[1] as u32,
                fill: [
                    TilesFill::Background,
                    TilesFill::Foreground,
                    TilesFill::Inverse,
                    TilesFill::Unaltered,
                ][v[2] as usize],
                // Set from the app's foreground color when applied
                foreground: [0; 3],
            },
            Kind::ColorHalftone => Filter::ColorHalftone {
                radius: v[0] as u32,
                angles: [v[1] as i32, v[2] as i32, v[3] as i32, v[4] as i32],
            },
            Kind::Mezzotint => Filter::Mezzotint {
                kind: MezzotintType::ALL[(v[0] as usize).min(9)],
            },
            Kind::Pointillize => Filter::Pointillize { cell: v[0] as u32 },
            Kind::Diffuse => Filter::Diffuse {
                mode: [
                    DiffuseMode::Normal,
                    DiffuseMode::DarkenOnly,
                    DiffuseMode::LightenOnly,
                    DiffuseMode::Anisotropic,
                ][v[0] as usize],
            },
            Kind::Pinch => Filter::Pinch {
                amount: v[0] as i32,
            },
            Kind::Spherize => Filter::Spherize {
                amount: v[0] as i32,
                mode: [
                    SpherizeMode::Normal,
                    SpherizeMode::HorizontalOnly,
                    SpherizeMode::VerticalOnly,
                ][v[1] as usize],
            },
            Kind::PolarCoordinates => Filter::PolarCoordinates {
                to_polar: v[0] == 0.0,
            },
            Kind::Emboss => Filter::Emboss {
                angle: v[0] as i32,
                height: v[1] as u32,
                amount: v[2] as u32,
            },
            Kind::SurfaceBlur => Filter::SurfaceBlur {
                radius: v[0] as u32,
                threshold: v[1] as u8,
            },
            Kind::DustAndScratches => Filter::DustAndScratches {
                radius: v[0] as u32,
                threshold: v[1] as u8,
            },
            Kind::TraceContour => Filter::TraceContour {
                level: v[0] as u8,
                upper: v[1] == 1.0,
            },
            Kind::Wind => Filter::Wind {
                method: [WindMethod::Wind, WindMethod::Blast, WindMethod::Stagger][v[0] as usize],
                from_left: v[1] == 1.0,
            },
            // Every adjustment has its own dialog
            _ => return None,
        };
        Some(Effect::Filter(filter))
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("adjust-dialog"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let size = match self.custom {
                    Some(Custom::BrightnessContrast(_)) => brightness_contrast::SIZE,
                    Some(Custom::ColorBalance(_)) => color_balance::SIZE,
                    Some(Custom::HueSaturation(_)) => hue_saturation::SIZE,
                    Some(Custom::Levels(_)) => levels::SIZE,
                    Some(Custom::Curves(_)) => curves::SIZE,
                    Some(Custom::ChannelMixer(_)) => channel_mixer::SIZE,
                    Some(Custom::SelectiveColor(_)) => selective_color::SIZE,
                    Some(Custom::Vibrance(_)) => vibrance::VIBRANCE_SIZE,
                    Some(Custom::Posterize(_)) => vibrance::POSTERIZE_SIZE,
                    Some(Custom::Exposure(_)) => exposure::SIZE,
                    Some(Custom::PhotoFilter(_)) => photo_filter::SIZE,
                    Some(Custom::BlackWhite(_)) => black_white::SIZE,
                    Some(Custom::Threshold(_)) => threshold::SIZE,
                    Some(Custom::GradientMap(_)) => gradient_map::SIZE,
                    Some(Custom::Kernel(_)) => custom_filter::SIZE,
                    None => self.kind.size(),
                };
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                outcome = if self.custom.is_some() {
                    self.custom_ui(ui, rect)
                } else if let Some(layout) = self.kind.layout() {
                    self.classic_ui(ui, rect, layout)
                } else if let Some(layout) = self.kind.plain() {
                    self.plain_ui(ui, rect, layout)
                } else {
                    let layout = self
                        .kind
                        .distort()
                        .expect("every filter dialog has a layout");
                    self.distort_ui(ui, rect, layout)
                };
            });
        self.first_frame = false;
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    /// The dialogs rebuilt after Photoshop 2026: their own layout, with the
    /// AppKit title bar.
    fn custom_ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        common::frame(ui, frame, self.kind.title(), theme::dialog_bold(pt(13.0)));
        if self.wants_pane() {
            let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
            let pane = filter_layout::PANE;
            self.pane_ui(
                ui,
                Rect::from_min_max(at(pane[0], pane[1]), at(pane[2], pane[3])),
            );
            zoom_controls(ui, at(0.0, filter_layout::ZOOM_Y).y, frame.left());
        }
        let button = match self.custom.as_mut() {
            Some(Custom::BrightnessContrast(d)) => {
                let pressed = d.ui(ui, frame, self.first_frame, &mut self.preview);
                if pressed == Some(uxp::Button::Third) {
                    d.auto(auto_brightness_contrast(&self.histogram));
                }
                pressed
            }
            Some(Custom::ColorBalance(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::HueSaturation(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Levels(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Curves(d)) => d.ui(ui, frame, &mut self.preview),
            Some(Custom::ChannelMixer(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::SelectiveColor(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Vibrance(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Posterize(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Exposure(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::PhotoFilter(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::BlackWhite(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Threshold(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::GradientMap(d)) => d.ui(ui, frame, &mut self.preview),
            Some(Custom::Kernel(d)) => {
                let (button, request) = d.ui(ui, frame, self.first_frame, &mut self.preview);
                match request {
                    Some(custom_filter::Request::Load) => load_kernel(d),
                    Some(custom_filter::Request::Save) => save_kernel(d),
                    None => {}
                }
                button
            }
            None => None,
        };
        match button {
            Some(uxp::Button::Ok) => self.effect().map_or(Outcome::Open, Outcome::Apply),
            Some(uxp::Button::Cancel) => Outcome::Cancel,
            _ => Outcome::Open,
        }
    }

    /// A classic Photoshop filter dialog (see `filter_layout`).
    fn classic_ui(&mut self, ui: &mut Ui, frame: Rect, layout: &filter_layout::Layout) -> Outcome {
        use filter_layout::Row;
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |b: [f32; 4]| Rect::from_min_max(at(b[0], b[1]), at(b[2], b[3]));
        common::frame(ui, frame, self.kind.title(), theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();

        if layout.pane {
            self.pane_ui(ui, r(filter_layout::PANE));
            zoom_controls(ui, at(0.0, filter_layout::ZOOM_Y).y, frame.left());
        }
        let params = self.kind.params();
        for (i, row) in layout.rows.iter().enumerate() {
            let p = &params[i];
            match *row {
                Row::Number {
                    label,
                    label_right,
                    field,
                    unit,
                    unit_x,
                    track,
                    scale,
                    dial,
                } => {
                    let field = r(field);
                    let cy = field.center().y;
                    appkit::text(
                        ui,
                        Pos2::new(at(label_right, 0.0).x, cy),
                        Align2::RIGHT_CENTER,
                        label,
                        appkit::TEXT,
                    );
                    let step = 10f32.powi(-(p.decimals as i32));
                    appkit::field(
                        ui,
                        field,
                        &mut self.values[i],
                        ("filter-field", i),
                        (p.min, p.max),
                        step,
                        p.decimals,
                        self.first_frame && i == 0,
                    );
                    let unit_x = if unit == "°" {
                        field.right() + pt(2.5)
                    } else {
                        at(unit_x, 0.0).x
                    };
                    appkit::text(
                        ui,
                        Pos2::new(unit_x, cy),
                        Align2::LEFT_CENTER,
                        unit,
                        appkit::TEXT,
                    );
                    if let Some((x0, x1, top)) = track {
                        self.classic_track(ui, i, at(x0, top), at(x1, top).x, scale);
                    }
                    if let Some((cx, cy, radius)) = dial {
                        self.dial(ui, i, at(cx, cy), pt(radius), self.kind == Kind::MotionBlur);
                    }
                }
                Row::Popup {
                    label,
                    label_right,
                    rect,
                } => {
                    let rect = r(rect);
                    appkit::text(
                        ui,
                        Pos2::new(at(label_right, 0.0).x, rect.center().y),
                        Align2::RIGHT_CENTER,
                        label,
                        appkit::TEXT,
                    );
                    if let ParamKind::Choice(options) = p.kind {
                        let mut chosen = self.value(i).unwrap_or(0.0) as usize;
                        appkit::popup(
                            ui,
                            rect,
                            "filter-popup",
                            options[chosen.min(options.len() - 1)],
                            |ui| {
                                for (k, o) in options.iter().enumerate() {
                                    ui.selectable_value(&mut chosen, k, *o);
                                }
                            },
                        );
                        self.values[i] = chosen.to_string();
                    }
                }
                Row::Radios { title, group, ys } => {
                    let g = r(group);
                    appkit::group(
                        &painter,
                        g,
                        (
                            at(group[0] + 15.0, 0.0).x,
                            at(group[0] + 26.0, 0.0).x + text_width(ui, title),
                        ),
                    );
                    appkit::label(ui, Pos2::new(at(group[0] + 20.0, 0.0).x, g.top()), title);
                    if let ParamKind::Choice(options) = p.kind {
                        let chosen = self.value(i).unwrap_or(0.0) as usize;
                        for (k, (&y, option)) in ys.iter().zip(options.iter()).enumerate() {
                            if appkit::radio(ui, at(27.0, y), option, chosen == k) {
                                self.values[i] = k.to_string();
                            }
                        }
                    }
                }
                Row::Check { label, min } => {
                    let mut on = self.value(i) == Some(1.0);
                    appkit::checkbox(ui, at(min.0, min.1), label, &mut on);
                    self.values[i] = (on as u8).to_string();
                }
            }
        }

        // OK, Cancel and Preview at the top right
        let x0 = layout.size.0 - 90.5;
        let button = |ui: &mut Ui, y: f32, label: &str, default: bool, enabled: bool| {
            appkit::button(
                ui,
                Rect::from_min_max(at(x0, y), at(x0 + layout.button_width, y + 26.0)),
                label,
                default,
                enabled,
            )
        };
        let valid = self.effect().is_some();
        let ok = button(ui, 38.5, "OK", true, valid);
        let cancel = button(ui, 73.5, "Cancel", false, true);
        appkit::checkbox(
            ui,
            at(x0 - 0.5, layout.preview_y),
            "Preview",
            &mut self.preview,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter)
            && let Some(effect) = self.effect()
        {
            return Outcome::Apply(effect);
        }
        Outcome::Open
    }

    /// A Distort filter's plug-in style dialog (see `distort`).
    fn distort_ui(&mut self, ui: &mut Ui, frame: Rect, layout: &distort::Layout) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, self.kind.title(), theme::dialog_bold(pt(13.0)));
        distort::frame(ui, at);
        if let Some(texture) = &self.pane {
            distort::image(ui, at, texture);
        }
        distort::zoom_bar(ui, at);

        let params = self.kind.params();
        match layout.control {
            distort::Control::Slider {
                label_y,
                field_x,
                track_x1,
            } => {
                let p = &params[0];
                let label = p.label.split(" (").next().unwrap_or(p.label);
                distort::label(ui, at(23.5, label_y), Align2::LEFT_CENTER, label);
                let field = r(
                    field_x,
                    distort::FIELD_Y.0,
                    field_x + distort::FIELD_W,
                    distort::FIELD_Y.1,
                );
                appkit::field(
                    ui,
                    field,
                    &mut self.values[0],
                    "distort-field",
                    (p.min, p.max),
                    1.0,
                    0,
                    self.first_frame,
                );
                // Crystallize's and Pointillize's cell size has no unit
                let unit = match self.kind {
                    Kind::Twirl => "°",
                    Kind::Crystallize | Kind::Pointillize => "",
                    _ => "%",
                };
                appkit::text(
                    ui,
                    Pos2::new(field.right() + pt(distort::UNIT_GAP), field.center().y),
                    Align2::LEFT_CENTER,
                    unit,
                    appkit::TEXT,
                );
                let response = ui.interact(
                    distort::slider_rect(at, track_x1),
                    ui.id().with("distort-slider"),
                    Sense::click_and_drag(),
                );
                if (response.dragged() || response.clicked())
                    && let Some(pointer) = response.interact_pointer_pos()
                {
                    let t = distort::slider_place(at, track_x1, pointer.x);
                    self.set(0, (p.min + (p.max - p.min) * t).round());
                }
                let v = self.value(0).unwrap_or(p.default);
                distort::slider(ui, at, track_x1, (v - p.min) / (p.max - p.min));
            }
            distort::Control::None => {}
            distort::Control::Radios(groups) => {
                for (i, group) in groups.iter().enumerate() {
                    distort::group(ui, at, group);
                    if let ParamKind::Choice(options) = params[i].kind {
                        let chosen = self.value(i).unwrap_or(0.0) as usize;
                        for (k, (&y, option)) in group.ys.iter().zip(options).enumerate() {
                            if distort::radio(ui, at(group.x, y), option, chosen == k) {
                                self.values[i] = k.to_string();
                            }
                        }
                    }
                }
            }
        }
        // The pop-up sets the last setting
        let last = params.len().saturating_sub(1);
        if let (Some((rect, label_x)), ParamKind::Choice(options)) = (
            layout.mode,
            params.get(last).map(|p| p.kind).unwrap_or(ParamKind::Check),
        ) {
            distort::label(
                ui,
                at(label_x, (rect[1] + rect[3]) / 2.0),
                Align2::LEFT_CENTER,
                params[last].label,
            );
            let mut chosen = self.value(last).unwrap_or(0.0) as usize;
            appkit::popup(
                ui,
                r(rect[0], rect[1], rect[2], rect[3]),
                "distort-mode",
                options[chosen.min(options.len() - 1)],
                |ui| {
                    for (k, o) in options.iter().enumerate() {
                        ui.selectable_value(&mut chosen, k, *o);
                    }
                },
            );
            self.values[last] = chosen.to_string();
        }
        if let (Some((x, y)), Some(Effect::Filter(filter))) = (layout.diagram, self.effect()) {
            if self.diagram.as_ref().is_none_or(|(v, _)| *v != self.values) {
                let image = distort::diagram(filter, 256);
                let texture =
                    ui.ctx()
                        .load_texture("distort-diagram", image, egui::TextureOptions::NEAREST);
                self.diagram = Some((self.values.clone(), texture));
            }
            if let Some((_, texture)) = &self.diagram {
                ui.painter().image(
                    texture.id(),
                    r(x, y, x + 128.0, y + 128.0),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }

        let x0 = layout.buttons_x;
        let valid = self.effect().is_some();
        let ok = appkit::button_with(
            ui,
            r(x0, 41.0, x0 + 89.0, 67.0),
            "OK",
            (true, valid),
            13.0,
            0.0,
        );
        let cancel = appkit::button_with(
            ui,
            r(x0, 77.0, x0 + 89.0, 103.0),
            "Cancel",
            (false, true),
            13.0,
            0.0,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter)
            && let Some(effect) = self.effect()
        {
            return Outcome::Apply(effect);
        }
        Outcome::Open
    }

    /// A small dialog without a preview (see `plain_filter`).
    fn plain_ui(&mut self, ui: &mut Ui, frame: Rect, layout: &plain_filter::Layout) -> Outcome {
        use plain_filter::Item;
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |b: [f32; 4]| Rect::from_min_max(at(b[0], b[1]), at(b[2], b[3]));
        common::frame(ui, frame, self.kind.title(), theme::dialog_bold(pt(13.0)));
        let params = self.kind.params();
        let mut i = 0;
        for item in layout.items {
            match *item {
                Item::Text { text, at: (x, y) } => {
                    distort::label(ui, at(x, y), Align2::LEFT_CENTER, text);
                }
                Item::Field { label, rect, unit } => {
                    let field = r(rect);
                    let cy = field.center().y;
                    distort::label(
                        ui,
                        Pos2::new(at(label.1, 0.0).x, cy),
                        Align2::LEFT_CENTER,
                        label.0,
                    );
                    let p = &params[i];
                    appkit::field(
                        ui,
                        field,
                        &mut self.values[i],
                        ("plain-field", i),
                        (p.min, p.max),
                        1.0,
                        p.decimals,
                        self.first_frame && i == 0,
                    );
                    if let Some((unit, x)) = unit {
                        distort::label(ui, Pos2::new(at(x, 0.0).x, cy), Align2::LEFT_CENTER, unit);
                    }
                    i += 1;
                }
                Item::Radios { centers, gap } => {
                    if let ParamKind::Choice(options) = params[i].kind {
                        let chosen = self.value(i).unwrap_or(0.0) as usize;
                        for (k, (&(x, y), option)) in centers.iter().zip(options).enumerate() {
                            let font = distort::label_font();
                            if appkit::radio_with(ui, at(x, y), option, chosen == k, (gap, font)) {
                                self.values[i] = k.to_string();
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
        let x0 = layout.buttons_x;
        let valid = self.effect().is_some();
        let ok = appkit::button_with(
            ui,
            Rect::from_min_max(at(x0, 41.0), at(x0 + 89.0, 67.0)),
            "OK",
            (true, valid),
            13.0,
            0.0,
        );
        let cancel = appkit::button_with(
            ui,
            Rect::from_min_max(at(x0, 77.0), at(x0 + 89.0, 103.0)),
            "Cancel",
            (false, true),
            13.0,
            0.0,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter)
            && let Some(effect) = self.effect()
        {
            return Outcome::Apply(effect);
        }
        Outcome::Open
    }

    /// The preview pane: the document, as previewed, at 100% (one image
    /// pixel per screen pixel), centered.
    fn pane_ui(&mut self, ui: &mut Ui, rect: Rect) {
        let Some(texture) = &self.pane else {
            return;
        };
        let size = texture.size_vec2() * pt(0.5);
        let shown = Rect::from_center_size(rect.center(), size).intersect(rect);
        let uv = Rect::from_min_max(
            Pos2::new(
                (shown.left() - (rect.center().x - size.x / 2.0)) / size.x,
                (shown.top() - (rect.center().y - size.y / 2.0)) / size.y,
            ),
            Pos2::new(
                (shown.right() - (rect.center().x - size.x / 2.0)) / size.x,
                (shown.bottom() - (rect.center().y - size.y / 2.0)) / size.y,
            ),
        );
        ui.painter().image(texture.id(), shown, uv, Color32::WHITE);
    }

    /// A 3 pt track with a white pin under it, travelling from 2.25 pt left
    /// of the track's start to 0.75 pt past its end; dragging along it sets
    /// the value through `scale`.
    fn classic_track(
        &mut self,
        ui: &mut Ui,
        i: usize,
        start: Pos2,
        end_x: f32,
        scale: filter_layout::Scale,
    ) {
        let p = &self.kind.params()[i];
        let (min, max, decimals) = (p.min, p.max, p.decimals);
        let line = Rect::from_min_max(start, Pos2::new(end_x, start.y + pt(3.0)));
        ui.painter().rect_filled(line, 0, Color32::from_gray(0x75));
        let (x0, x1) = (start.x - pt(2.25), end_x + pt(0.75));
        let hit = Rect::from_min_max(
            Pos2::new(x0 - pt(6.0), line.top() - pt(4.0)),
            Pos2::new(x1 + pt(6.0), line.bottom() + pt(12.0)),
        );
        let response = ui.interact(
            hit,
            ui.id().with(("classic-track", i)),
            Sense::click_and_drag(),
        );
        if (response.dragged() || response.clicked())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let t = ((pointer.x - x0) / (x1 - x0)).clamp(0.0, 1.0);
            let round = 10f32.powi(decimals as i32);
            self.set(i, (scale.value(t, min, max) * round).round() / round);
        }
        let v = self.value(i).unwrap_or(p.default);
        let x = x0 + (x1 - x0) * scale.place(v, min, max);
        appkit::pin(
            ui.painter(),
            Pos2::new(x, line.bottom() + pt(0.5)),
            appkit::Pin::White,
        );
    }

    /// An angle dial: a circle with a line from its center at the angle
    /// (through it when `both`, as Motion Blur's); dragging sets the angle.
    fn dial(&mut self, ui: &mut Ui, i: usize, center: Pos2, radius: f32, both: bool) {
        let response = ui.interact(
            Rect::from_center_size(center, egui::Vec2::splat(radius * 2.0)),
            ui.id().with(("dial", i)),
            Sense::click_and_drag(),
        );
        if (response.dragged() || response.clicked())
            && let Some(p) = response.interact_pointer_pos()
        {
            let d = p - center;
            let mut angle = (-d.y).atan2(d.x).to_degrees().round();
            if both && angle.abs() > 90.0 {
                angle -= 180.0 * angle.signum();
            }
            self.set(i, angle);
        }
        let painter = ui.painter();
        let stroke = Stroke::new(pt(1.0), Color32::from_gray(0xc8));
        painter.circle_stroke(center, radius - pt(0.5), stroke);
        let a = self.value(i).unwrap_or(0.0).to_radians();
        let dir = vec2(a.cos(), -a.sin()) * (radius - pt(2.5));
        let from = if both { center - dir } else { center };
        painter.line_segment([from, center + dir], stroke);
        painter.circle_filled(center, pt(1.5), Color32::from_gray(0xc8));
    }
}

/// Brightness/Contrast's Auto: the brightness and contrast whose curve
/// comes closest (over the image's histogram) to Auto Contrast's stretch,
/// the 0.1% darkest and lightest values clipped. Photoshop's own choice
/// is not reproduced exactly.
fn auto_brightness_contrast(histogram: &[u64; 256]) -> (i32, i32) {
    let total: u64 = histogram.iter().sum();
    if total == 0 {
        return (0, 0);
    }
    let clip = total / 1000;
    let mut acc = 0;
    let lo = (0..256).find(|&i| {
        acc += histogram[i];
        acc > clip
    });
    acc = 0;
    let hi = (0..256).rev().find(|&i| {
        acc += histogram[i];
        acc > clip
    });
    let (lo, hi) = (lo.unwrap_or(0) as f32, hi.unwrap_or(255) as f32);
    if hi <= lo {
        return (0, 0);
    }
    let target = |v: usize| ((v as f32 - lo) / (hi - lo) * 255.0).clamp(0.0, 255.0);
    let mut best = (f64::MAX, (0, 0));
    for b in (-150..=150).step_by(2) {
        for c in (-50..=100).step_by(2) {
            let adj = Adjustment::BrightnessContrast {
                brightness: b,
                contrast: c,
                legacy: false,
            };
            let table = adj.tables().expect("a tone curve")[0];
            let err: f64 = (0..256)
                .filter(|&v| histogram[v] > 0)
                .map(|v| {
                    let d = table[v] as f32 - target(v);
                    histogram[v] as f64 * (d * d) as f64
                })
                .sum();
            if err < best.0 {
                best = (err, (b, c));
            }
        }
    }
    best.1
}

#[cfg(test)]
mod tests {
    use super::*;
    use op_core::adjust::{HueSaturation, Levels};

    fn dialog(kind: Kind) -> AdjustDialog {
        let doc = op_core::Document::new_with_background("t", 1, 1, op_core::Color::WHITE);
        AdjustDialog::new(kind, [0; 256], doc.snapshot())
    }

    #[test]
    fn fields_show_values_as_photoshop_does() {
        // Gaussian Blur keeps its decimal, Add Noise and Minimum drop zeros
        assert_eq!(dialog(Kind::GaussianBlur).values[0], "1.0");
        assert_eq!(dialog(Kind::AddNoise).values[0], "12.5");
        assert_eq!(dialog(Kind::Minimum).values[0], "1");
        assert_eq!(dialog(Kind::Maximum).values[0], "1");
        assert_eq!(dialog(Kind::UnsharpMask).values[1], "1.0");
    }

    #[test]
    fn every_filter_dialog_has_an_effect() {
        // OK stays disabled without one, so a missing mapping greys it out
        for kind in [
            Kind::GaussianBlur,
            Kind::BoxBlur,
            Kind::UnsharpMask,
            Kind::AddNoise,
            Kind::Median,
            Kind::Minimum,
            Kind::Maximum,
            Kind::HighPass,
            Kind::Offset,
            Kind::Mosaic,
            Kind::MotionBlur,
            Kind::Emboss,
            Kind::Twirl,
            Kind::Crystallize,
            Kind::Pointillize,
            Kind::Diffuse,
            Kind::Ripple,
            Kind::Mezzotint,
            Kind::Tiles,
            Kind::ColorHalftone,
            Kind::Pinch,
            Kind::Spherize,
            Kind::PolarCoordinates,
            Kind::SurfaceBlur,
            Kind::DustAndScratches,
            Kind::TraceContour,
            Kind::Wind,
        ] {
            assert!(
                matches!(dialog(kind).effect(), Some(Effect::Filter(_))),
                "{kind:?}"
            );
            // Every filter dialog has a classic or plug-in style layout
            assert!(kind.size().x > 0.0, "{kind:?}");
        }
    }

    #[test]
    fn defaults_match_photoshop() {
        assert_eq!(
            dialog(Kind::Levels).effect(),
            Some(Effect::Adjustment(Adjustment::Levels(
                Levels::IDENTITY.composite()
            )))
        );
        assert_eq!(
            dialog(Kind::Exposure).effect(),
            Some(Effect::Adjustment(Adjustment::Exposure {
                exposure: 0.0,
                offset: 0.0,
                gamma: 1.0
            }))
        );
        assert_eq!(
            dialog(Kind::HueSaturation).effect(),
            Some(Effect::Adjustment(Adjustment::HueSaturation(
                HueSaturation::master(0, 0, 0)
            )))
        );
    }

    #[test]
    fn filters_read_choices_and_checkboxes() {
        let mut d = dialog(Kind::AddNoise);
        d.values[1] = "1".into();
        d.values[2] = "1".into();
        assert_eq!(
            d.effect(),
            Some(Effect::Filter(Filter::AddNoise {
                amount: 12.5,
                gaussian: true,
                monochromatic: true
            }))
        );
        let mut d = dialog(Kind::Offset);
        d.values[2] = "2".into();
        assert_eq!(
            d.effect(),
            Some(Effect::Filter(Filter::Offset {
                dx: 0,
                dy: 0,
                fill: OffsetFill::Wrap
            }))
        );
    }
}
