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
    ColorModel, DiffuseMode, Filter, MezzotintType, OffsetFill, RippleSize, SpherizeMode,
    TilesFill, WindMethod, ZigZagStyle,
};

use super::{
    appkit, black_white, brightness_contrast, channel_mixer, color_balance, common, curves,
    custom_filter, distort, exposure, filter_layout, gradient_map, hue_saturation, levels,
    photo_filter, plain_filter, selective_color, threshold, uxp, vibrance,
};
use crate::theme::{self, pt};

/// What a dialog applies: an adjustment or a filter.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)] // as `Adjustment`
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
const HSB_HSL: &[Param] = &[
    choice("Input mode", &["RGB", "HSB", "HSL"], 0),
    choice("Row order", &["RGB", "HSB", "HSL"], 1),
];
const ZIGZAG: &[Param] = &[
    param("Amount", -100.0, 100.0, 30.0, 0),
    param("Ridges", 0.0, 20.0, 4.0, 0),
    choice(
        "Style",
        &["Around Center", "Out From Center", "Pond Ripples"],
        0,
    ),
];
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
const RADIAL_BLUR: &[Param] = &[
    param("Amount:", 1.0, 100.0, 10.0, 0),
    choice("Blur Method", &["Spin", "Zoom"], 0),
    choice("Quality", &["Draft", "Good", "Best"], 1),
];

const SMART_BLUR: &[Param] = &[
    param("Radius:", 0.1, 100.0, 3.0, 1),
    param("Threshold:", 0.1, 100.0, 25.0, 1),
    choice("Quality:", &["Low", "Medium", "High"], 2),
    choice("Mode:", &["Normal", "Edge Only", "Overlay Edge"], 0),
];

const SHAPE_BLUR: &[Param] = &[
    param("Radius:", 5.0, 1000.0, 10.0, 0),
    choice(
        "Shape:",
        &["Circle", "Square", "Star", "Heart", "Diamond"],
        0,
    ),
];

/// In the dialog's order (`lens_blur`'s indexes).
const LENS_BLUR: &[Param] = &[
    choice("Preview", &["Faster", "More Accurate"], 0),
    choice("Source:", &["None", "Transparency", "Layer Mask"], 0),
    param("Blur Focal Distance", 0.0, 255.0, 0.0, 0),
    check("Invert", false),
    choice(
        "Shape:",
        &[
            "Triangle (3)",
            "Square (4)",
            "Pentagon (5)",
            "Hexagon (6)",
            "Heptagon (7)",
            "Octagon (8)",
        ],
        3,
    ),
    param("Radius", 0.0, 100.0, 15.0, 0),
    param("Blade Curvature", 0.0, 100.0, 0.0, 0),
    param("Rotation", 0.0, 360.0, 0.0, 0),
    param("Brightness", 0.0, 100.0, 0.0, 0),
    param("Threshold", 0.0, 255.0, 255.0, 0),
    param("Amount", 0.0, 100.0, 0.0, 0),
    choice("Distribution", &["Uniform", "Gaussian"], 0),
    check("Monochromatic", false),
];

/// In the dialog's order (`reduce_noise`'s indexes).
const REDUCE_NOISE: &[Param] = &[
    choice("Mode", &["Basic", "Advanced"], 0),
    param("Strength:", 0.0, 10.0, 6.0, 0),
    param("Preserve Details:", 0.0, 100.0, 60.0, 0),
    param("Reduce Color Noise:", 0.0, 100.0, 45.0, 0),
    param("Sharpen Details:", 0.0, 100.0, 25.0, 0),
    check("Remove JPEG Artifact", false),
    choice("Channel:", &["Red", "Green", "Blue"], 0),
    param("Red Strength:", 0.0, 10.0, 0.0, 0),
    param("Red Preserve Details:", 0.0, 100.0, 60.0, 0),
    param("Green Strength:", 0.0, 10.0, 0.0, 0),
    param("Green Preserve Details:", 0.0, 100.0, 60.0, 0),
    param("Blue Strength:", 0.0, 10.0, 0.0, 0),
    param("Blue Preserve Details:", 0.0, 100.0, 60.0, 0),
];

/// In the dialog's order (`smart_sharpen`'s indexes).
const SMART_SHARPEN: &[Param] = &[
    param("Amount:", 1.0, 500.0, 200.0, 0),
    param("Radius:", 0.1, 64.0, 1.0, 1),
    param("Reduce Noise:", 0.0, 100.0, 10.0, 0),
    choice("Remove:", &["Gaussian Blur", "Lens Blur", "Motion Blur"], 1),
    param("Angle:", -180.0, 180.0, 0.0, 0),
    param("Shadows Fade Amount:", 0.0, 100.0, 0.0, 0),
    param("Shadows Tonal Width:", 0.0, 100.0, 50.0, 0),
    param("Shadows Radius:", 1.0, 100.0, 1.0, 0),
    param("Highlights Fade Amount:", 0.0, 100.0, 0.0, 0),
    param("Highlights Tonal Width:", 0.0, 100.0, 50.0, 0),
    param("Highlights Radius:", 1.0, 100.0, 1.0, 0),
    check("Use Legacy", false),
    check("More Accurate", false),
];

const FIBERS: &[Param] = &[
    param("Variance:", 1.0, 64.0, 16.0, 0),
    param("Strength:", 1.0, 64.0, 4.0, 0),
];

const LENS_FLARE: &[Param] = &[
    param("Brightness:", 10.0, 300.0, 100.0, 0),
    choice(
        "Lens Type",
        &["50-300mm Zoom", "35mm Prime", "105mm Prime", "Movie Prime"],
        0,
    ),
];

const EXTRUDE: &[Param] = &[
    choice("Type", &["Blocks", "Pyramids"], 0),
    param("Size:", 2.0, 255.0, 30.0, 0),
    param("Depth:", 1.0, 255.0, 30.0, 0),
    choice("Depth Based On", &["Random", "Level-based"], 0),
    check("Solid Front Faces", false),
    check("Mask Incomplete Blocks", false),
];

/// In the dialog's order (`oil_paint`'s indexes).
const OIL_PAINT: &[Param] = &[
    param("Stylization:", 0.1, 10.0, 1.5, 1),
    param("Cleanliness:", 0.0, 10.0, 5.0, 1),
    param("Scale:", 0.1, 10.0, 1.0, 1),
    param("Bristle Detail:", 0.0, 10.0, 0.0, 1),
    check("Lighting", true),
    param("Angle:", -180.0, 180.0, -60.0, 0),
    param("Shine:", 0.0, 10.0, 1.0, 1),
];

const WAVE: &[Param] = &[
    param("Number of Generators:", 1.0, 999.0, 5.0, 0),
    param("Wavelength Min.:", 1.0, 998.0, 10.0, 0),
    param("Wavelength Max.:", 2.0, 999.0, 120.0, 0),
    param("Amplitude Min.:", 1.0, 998.0, 5.0, 0),
    param("Amplitude Max.:", 1.0, 999.0, 35.0, 0),
    param("Scale Horiz.:", 1.0, 100.0, 100.0, 0),
    param("Scale Vert.:", 1.0, 100.0, 100.0, 0),
    choice("Type", &["Sine", "Triangle", "Square"], 0),
    choice("Undefined Areas", &["Wrap Around", "Repeat Edge Pixels"], 1),
];

/// Shear's curve is the dialog's own (`Extra::shear_points`).
const SHEAR: &[Param] = &[choice(
    "Undefined Areas",
    &["Wrap Around", "Repeat Edge Pixels"],
    0,
)];

const DISPLACE: &[Param] = &[
    param("Horizontal Scale:", -999.0, 999.0, 10.0, 0),
    param("Vertical Scale:", -999.0, 999.0, 10.0, 0),
    choice("Displacement Map", &["Stretch To Fit", "Tile"], 0),
    choice("Undefined Areas", &["Wrap Around", "Repeat Edge Pixels"], 1),
];

const SHADOWS_HIGHLIGHTS: &[Param] = &[
    param("Shadows Amount:", 0.0, 100.0, 35.0, 0),
    param("Shadows Tone:", 0.0, 100.0, 50.0, 0),
    param("Shadows Radius:", 0.0, 2500.0, 30.0, 0),
    param("Highlights Amount:", 0.0, 100.0, 0.0, 0),
    param("Highlights Tone:", 0.0, 100.0, 50.0, 0),
    param("Highlights Radius:", 0.0, 2500.0, 30.0, 0),
    param("Color:", -100.0, 100.0, 20.0, 0),
    param("Midtone:", -100.0, 100.0, 0.0, 0),
    param("Black Clip:", 0.0, 50.0, 0.01, 2),
    param("White Clip:", 0.0, 50.0, 0.01, 2),
    check("Show More Options", false),
];

const HDR_TONING: &[Param] = &[
    choice(
        "Method:",
        &[
            "Exposure and Gamma",
            "Highlight Compression",
            "Equalize Histogram",
            "Local Adaptation",
        ],
        3,
    ),
    param("Radius:", 1.0, 500.0, 15.0, 0),
    param("Strength:", 0.1, 4.0, 0.52, 2),
    param("Gamma:", 0.1, 2.0, 1.0, 2),
    param("Exposure:", -5.0, 5.0, 0.0, 2),
    param("Detail:", -100.0, 300.0, 30.0, 0),
    param("Shadow:", -100.0, 100.0, 0.0, 0),
    param("Highlight:", -100.0, 100.0, 0.0, 0),
    param("Vibrance:", -100.0, 100.0, 0.0, 0),
    param("Saturation:", -100.0, 100.0, 20.0, 0),
    check("Smooth Edges", false),
];

const REPLACE_COLOR: &[Param] = &[
    param("Fuzziness:", 0.0, 200.0, 40.0, 0),
    param("Hue:", -180.0, 180.0, 0.0, 0),
    param("Saturation:", -100.0, 100.0, 0.0, 0),
    param("Lightness:", -100.0, 100.0, 0.0, 0),
    check("Localized Color Clusters", false),
];

const MATCH_COLOR: &[Param] = &[
    param("Luminance:", 1.0, 200.0, 100.0, 0),
    param("Color Intensity:", 1.0, 200.0, 100.0, 0),
    param("Fade:", 0.0, 100.0, 0.0, 0),
    check("Neutralize", false),
    choice("Source:", &["None"], 0),
    choice("Layer:", &["Merged"], 0),
    check("Ignore Selection when Applying Adjustment", false),
    check("Use Selection in Source to Calculate Colors", false),
    check("Use Selection in Target to Calculate Adjustment", false),
];

const COLOR_LOOKUP: &[Param] = &[
    choice("3DLUT File:", &["Load 3D LUT..."], 0),
    choice("Source:", &["3DLUT File", "Abstract", "Device Link"], 0),
    check("Dither", true),
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
    ZigZag,
    HsbHsl,
    Pinch,
    Spherize,
    PolarCoordinates,
    SurfaceBlur,
    DustAndScratches,
    /// Filter > Other > Custom...
    Custom,
    TraceContour,
    Wind,
    RadialBlur,
    SmartBlur,
    ShapeBlur,
    LensBlur,
    ReduceNoise,
    SmartSharpen,
    Fibers,
    LensFlare,
    Extrude,
    OilPaint,
    Wave,
    Shear,
    Displace,
    ShadowsHighlights,
    HdrToning,
    ReplaceColor,
    MatchColor,
    ColorLookup,
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
            Self::ZigZag => "ZigZag",
            Self::HsbHsl => "HSB/HSL Parameters",
            Self::Pinch => "Pinch",
            Self::Spherize => "Spherize",
            Self::PolarCoordinates => "Polar Coordinates",
            Self::SurfaceBlur => "Surface Blur",
            Self::DustAndScratches => "Dust & Scratches",
            Self::Custom => "Custom",
            Self::TraceContour => "Trace Contour",
            Self::Wind => "Wind",
            Self::RadialBlur => "Radial Blur",
            Self::SmartBlur => "Smart Blur",
            Self::ShapeBlur => "Shape Blur",
            Self::LensBlur => "Lens Blur",
            Self::ReduceNoise => "Reduce Noise",
            Self::SmartSharpen => "Smart Sharpen",
            Self::Fibers => "Fibers",
            Self::LensFlare => "Lens Flare",
            Self::Extrude => "Extrude",
            Self::OilPaint => "Oil Paint",
            Self::Wave => "Wave",
            Self::Shear => "Shear",
            Self::Displace => "Displace",
            Self::ShadowsHighlights => "Shadows/Highlights",
            Self::HdrToning => "HDR Toning",
            Self::ReplaceColor => "Replace Color",
            Self::MatchColor => "Match Color",
            Self::ColorLookup => "Color Lookup",
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
            Self::ZigZag => ZIGZAG,
            Self::HsbHsl => HSB_HSL,
            Self::Pinch => PINCH,
            Self::Spherize => SPHERIZE,
            Self::PolarCoordinates => POLAR,
            Self::SurfaceBlur => SURFACE_BLUR,
            Self::DustAndScratches => DUST_AND_SCRATCHES,
            Self::TraceContour => TRACE_CONTOUR,
            Self::Wind => WIND,
            Self::RadialBlur => RADIAL_BLUR,
            Self::SmartBlur => SMART_BLUR,
            Self::ShapeBlur => SHAPE_BLUR,
            Self::LensBlur => LENS_BLUR,
            Self::ReduceNoise => REDUCE_NOISE,
            Self::SmartSharpen => SMART_SHARPEN,
            Self::Fibers => FIBERS,
            Self::LensFlare => LENS_FLARE,
            Self::Extrude => EXTRUDE,
            Self::OilPaint => OIL_PAINT,
            Self::Wave => WAVE,
            Self::Shear => SHEAR,
            Self::Displace => DISPLACE,
            Self::ShadowsHighlights => SHADOWS_HIGHLIGHTS,
            Self::HdrToning => HDR_TONING,
            Self::ReplaceColor => REPLACE_COLOR,
            Self::MatchColor => MATCH_COLOR,
            Self::ColorLookup => COLOR_LOOKUP,
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
            Self::ShadowsHighlights => l::SHADOWS_HIGHLIGHTS,
            _ => return None,
        })
    }

    /// The small dialogs without a preview.
    fn plain(self) -> Option<&'static plain_filter::Layout> {
        Some(match self {
            Self::Tiles => &plain_filter::TILES,
            Self::ColorHalftone => &plain_filter::COLOR_HALFTONE,
            Self::HsbHsl => &plain_filter::HSB_HSL,
            Self::RadialBlur => &plain_filter::RADIAL_BLUR,
            Self::Shear => &plain_filter::SHEAR,
            Self::Displace => &plain_filter::DISPLACE,
            Self::Extrude => &plain_filter::EXTRUDE,
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
            Self::ZigZag => &distort::ZIGZAG,
            Self::Pinch => &distort::PINCH,
            Self::Spherize => &distort::SPHERIZE,
            Self::PolarCoordinates => &distort::POLAR,
            Self::Wind => &distort::WIND,
            Self::Fibers => &distort::FIBERS,
            Self::SmartBlur => &distort::SMART_BLUR,
            Self::LensFlare => &distort::LENS_FLARE,
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
/// The preview's zoom steps (Photoshop's), as fractions.
const PANE_ZOOMS: [f32; 18] = [
    0.0625, 0.0833, 0.125, 0.1667, 0.25, 0.3333, 0.5, 0.6667, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0,
    8.0, 12.0, 16.0,
];

/// A zoom as the preview shows it ("33%", "6.25%").
fn zoom_label(zoom: f32) -> String {
    let percent = zoom * 100.0;
    if (percent - percent.round()).abs() < 0.01 || percent >= 10.0 {
        format!("{}%", percent.round())
    } else {
        format!("{percent:.2}%")
    }
}

/// The zoom controls under the classic preview pane: zoom out and zoom
/// in (each dimmed at its end), the zoom between them. Returns whether
/// zoom out or zoom in was clicked.
fn zoom_controls(ui: &Ui, y: f32, left: f32, zoom: f32) -> Option<bool> {
    let painter = ui.painter();
    let x = |v: f32| left + pt(v);
    appkit::text(
        ui,
        Pos2::new(x(113.75), y),
        Align2::CENTER_CENTER,
        &zoom_label(zoom),
        appkit::TEXT,
    );
    let mut clicked = None;
    for (cx, plus) in [(57.0, false), (171.75, true)] {
        let enabled = if plus {
            zoom < PANE_ZOOMS[PANE_ZOOMS.len() - 1]
        } else {
            zoom > PANE_ZOOMS[0]
        };
        let tint = if enabled {
            appkit::TEXT
        } else {
            Color32::from_gray(0x80)
        };
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
        let hit = Rect::from_center_size(c + vec2(pt(1.5), pt(1.5)), vec2(pt(16.0), pt(16.0)));
        if enabled
            && ui
                .interact(hit, ui.id().with(("pane-zoom", plus)), Sense::click())
                .clicked()
        {
            clicked = Some(plus);
        }
    }
    clicked
}

#[allow(clippy::large_enum_variant)] // as `Adjustment`
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
    /// The preview's zoom (1 is 100%) and the document point at its center
    /// (None: the document's middle); changing them clears `pane`.
    pub pane_zoom: f32,
    pub pane_center: Option<(f32, f32)>,
    /// Where a drag in the preview started: the pointer and the center.
    pane_drag: Option<(Pos2, (f32, f32))>,
    /// The Distort dialogs' diagram and the settings it was drawn for.
    diagram: Option<(Vec<String>, egui::TextureHandle)>,
    /// Where the dialog was last drawn (clicks elsewhere may sample).
    pub rect: Rect,
    /// Another dialog (the Color Picker) is over this one: it is drawn but
    /// takes no keys or clicks.
    pub blocked: bool,
    /// Where a targeted adjustment drag started (screen position).
    pub target_from: Option<egui::Pos2>,
    /// What some dialogs need besides their fields.
    pub extra: Extra,
}

/// What Replace Color's preview picture was made from: the sampled color,
/// Fuzziness (as bits) and whether it shows the image.
type PreviewKey = (op_core::color_match::ReplaceSamples, u32, bool);

/// A Match Color source: an open document (or loaded statistics) and the
/// Lab statistics of its merged image and of each layer (name, all of it,
/// inside the document's selection when it has one).
#[derive(Clone, Default, Debug)]
pub struct MatchSource {
    pub name: String,
    pub layers: Vec<(String, [f32; 6], Option<[f32; 6]>)>,
    /// Its merged image made small, for the dialog's thumbnail.
    pub thumb: Option<(usize, usize, Vec<[u8; 3]>)>,
}

/// What some filter and adjustment dialogs need besides their fields.
#[derive(Clone, Default)]
pub struct Extra {
    /// The random pattern (Fibers, Wave, Extrude): a new one each opening.
    pub seed: u32,
    /// The foreground and background colors (Fibers).
    pub colors: ([u8; 3], [u8; 3]),
    /// Replace Color's sampled colors (the foreground at first; the
    /// eyedroppers pick, add and take out colors from the image) and the
    /// chosen eyedropper (0 pick, 1 add, 2 subtract).
    pub replace: op_core::color_match::ReplaceSamples,
    pub rc_tool: usize,
    /// HDR Toning's Toning Curve and sections (`hdr_toning.rs`).
    pub hdr: hdr_toning::HdrExtra,
    /// Match Color: the layer's Lab statistics (all of it, and inside the
    /// selection when there is one), its document's title and layer name,
    /// and each source's.
    pub target: Option<[f32; 6]>,
    pub target_selected: Option<[f32; 6]>,
    pub target_name: String,
    pub target_layer: String,
    pub sources: Vec<MatchSource>,
    /// Color Lookup's cube files (name, path) and the ones loaded.
    pub luts: Vec<(String, std::path::PathBuf)>,
    pub lut_ids: std::collections::HashMap<usize, u32>,
    /// Displace's map, once chosen (after OK).
    pub displace_map: Option<u32>,
    /// Levels' and Curves' Auto: the layer's pixels (`op_core::auto::samples`)
    /// and the Auto Color Correction Options in use.
    pub auto_samples: Vec<[u8; 3]>,
    pub auto_options: op_core::auto::Options,
    /// Options... was clicked: the app opens Auto Color Correction Options.
    pub wants_auto_options: bool,
    /// Replace Color's preview: the layer made small (width, height,
    /// pixels), whether it shows the image instead of the selection, and
    /// the texture with what it was made from.
    pub thumb: Option<(usize, usize, Vec<[u8; 3]>)>,
    pub show_image: bool,
    /// The active layer is the Background layer (Offset's first choice
    /// reads "Set to Background").
    pub on_background: bool,
    pub preview_texture: Option<(PreviewKey, egui::TextureHandle)>,
    /// Radial Blur's center, 0–1 across and down the image (the middle at
    /// first; Blur Center sets it).
    pub radial_center: (f32, f32),
    /// Shear's curve: (height 0–1, offset −0.5–0.5 of the width), the top
    /// and bottom ends first (a straight line at first), and the point
    /// being dragged.
    pub shear_points: Vec<(f32, f32)>,
    pub shear_drag: Option<usize>,
    /// Lens Blur's Set Focal Point is on, and the document point clicked
    /// in the preview (the app reads its depth, `set_focal_distance`).
    pub lens_pick: bool,
    pub lens_focal_at: Option<(f32, f32)>,
    /// Lens Flare's center, 0–1 across and down the image (the middle at
    /// first; a click in the preview moves it).
    pub flare_center: (f32, f32),
    /// Reduce Noise: the Per Channel tab is showing, the saved setting
    /// chosen, and the layer's red, green and blue as gray thumbnails.
    pub rn_per_channel: bool,
    pub rn_setting: Option<String>,
    pub rn_thumbs: [Option<egui::TextureHandle>; 3],
    /// Smart Sharpen: Shadows / Highlights open, the saved preset chosen.
    pub ss_open: bool,
    pub ss_preset: Option<String>,
    /// Shape Blur's shapes, white on black.
    pub shape_thumbs: Vec<egui::TextureHandle>,
    /// The document's size in pixels (Lens Flare's center needs it).
    pub doc_size: (f32, f32),
}

/// The active layer made small for a preview box `max` pixels at most
/// (each pixel the nearest one; transparent ones white).
pub fn thumbnail(
    doc: &op_core::Document,
    max: (usize, usize),
) -> Option<(usize, usize, Vec<[u8; 3]>)> {
    let image = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .and_then(|l| l.image())?;
    let (w, h) = (doc.width as usize, doc.height as usize);
    if w == 0 || h == 0 {
        return None;
    }
    let k = (max.0 as f32 / w as f32)
        .min(max.1 as f32 / h as f32)
        .min(1.0);
    let (tw, th) = (
        ((w as f32 * k) as usize).max(1),
        ((h as f32 * k) as usize).max(1),
    );
    let mut px = Vec::with_capacity(tw * th);
    for y in 0..th {
        for x in 0..tw {
            let sx = ((x as f32 + 0.5) / k) as u32;
            let sy = ((y as f32 + 0.5) / k) as u32;
            let p = image.pixel(sx.min(doc.width - 1), sy.min(doc.height - 1));
            px.push(if p[3] == 0 {
                [255; 3]
            } else {
                [p[0], p[1], p[2]]
            });
        }
    }
    Some((tw, th, px))
}

impl Extra {
    /// The popup labels a dialog fills in itself (Match Color's sources,
    /// Color Lookup's files), for the field at `i`.
    /// `source_pick` is Match Color's Source choice (its Layer pop-up
    /// lists that source's layers).
    fn labels(&self, kind: Kind, i: usize, source_pick: Option<usize>) -> Option<Vec<String>> {
        match (kind, i) {
            (Kind::MatchColor, 4) => Some(
                std::iter::once("None".to_owned())
                    .chain(self.sources.iter().map(|s| s.name.clone()))
                    .collect(),
            ),
            (Kind::MatchColor, 5) => Some(match source_pick {
                Some(k) if k > 0 => self
                    .sources
                    .get(k - 1)
                    .map(|s| s.layers.iter().map(|l| l.0.clone()).collect())
                    .unwrap_or_default(),
                _ => vec![self.target_layer.clone()],
            }),
            (Kind::ColorLookup, 0) => Some(
                std::iter::once("Load 3D LUT...".to_owned())
                    .chain(self.luts.iter().map(|(n, _)| n.clone()))
                    .collect(),
            ),
            _ => None,
        }
    }
}

/// The cube files Color Lookup lists: Photoshop's own (when installed) and
/// any in OpenPhoto's folder, `.cube` and `.3dl`.
pub fn lut_files() -> Vec<(String, std::path::PathBuf)> {
    let mut dirs = vec![std::path::PathBuf::from(
        "/Applications/Adobe Photoshop 2026/Presets/3DLUTs",
    )];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(
            std::path::PathBuf::from(home).join("Library/Application Support/OpenPhoto/3DLUTs"),
        );
    }
    let mut out: Vec<(String, std::path::PathBuf)> = dirs
        .iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("cube") || e.eq_ignore_ascii_case("3dl"))
        })
        .map(|p| {
            (
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                p,
            )
        })
        .collect();
    out.sort_by_key(|a| a.0.to_lowercase());
    out
}

/// Reads a `.cube` or `.3dl` file into the lookup registry.
pub fn load_lut(path: &std::path::Path) -> Option<u32> {
    let text = std::fs::read_to_string(path).ok()?;
    let is_3dl = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("3dl"));
    let lut = if is_3dl {
        op_core::color_match::Lut::parse_3dl(&text)?
    } else {
        op_core::color_match::Lut::parse_cube(&text)?
    };
    Some(op_core::color_match::register(lut))
}

/// An adjustment dialog's settings kept from its last OK.
#[derive(Clone)]
pub struct Remembered(Custom);

/// A dialog with its own layout and settings.
#[derive(Clone)]
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
            // (Shear previews in its own box)
            preview: kind.plain().is_none() || kind == Kind::Shear,
            histogram,
            first_frame: true,
            previewing: None,
            before,
            pane: None,
            pane_zoom: 1.0,
            pane_center: None,
            pane_drag: None,
            diagram: None,
            rect: Rect::NOTHING,
            blocked: false,
            target_from: None,
            extra: Extra {
                seed: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(1, |d| d.subsec_nanos()),
                luts: if kind == Kind::ColorLookup {
                    lut_files()
                } else {
                    Vec::new()
                },
                radial_center: (0.5, 0.5),
                flare_center: (0.5, 0.5),
                shear_points: vec![(0.0, 0.0), (1.0, 0.0)],
                ..Default::default()
            },
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

    /// Options...: the Auto Color Correction Options to open with.
    pub fn take_auto_options_request(&mut self) -> Option<op_core::auto::Options> {
        std::mem::take(&mut self.extra.wants_auto_options).then_some(self.extra.auto_options)
    }

    /// Auto Color Correction Options' OK: the options take effect at once
    /// (Auto runs with them).
    pub fn set_auto_options(&mut self, options: op_core::auto::Options) {
        self.extra.auto_options = options;
        let channels = op_core::auto::compute(&self.extra.auto_samples, &options);
        match &mut self.custom {
            Some(Custom::Levels(d)) => d.auto(channels),
            Some(Custom::Curves(d)) => d.auto(channels),
            _ => {}
        }
    }

    /// Levels' (and Curves') per-channel histograms.
    pub fn set_channel_histograms(&mut self, histograms: [[u64; 256]; 3]) {
        match &mut self.custom {
            Some(Custom::Levels(d)) => d.set_histograms(histograms),
            Some(Custom::Curves(d)) => d.set_histograms(histograms),
            _ => {}
        }
    }

    /// Photo Filter's swatch asks for the Color Picker; returns its color
    /// once.
    pub fn take_color_request(&mut self) -> Option<[u8; 3]> {
        match &mut self.custom {
            Some(Custom::PhotoFilter(d)) => std::mem::take(&mut d.wants_picker).then_some(d.color),
            _ => None,
        }
    }

    /// The Color Picker's OK for Photo Filter: its color, chosen.
    pub fn set_filter_color(&mut self, rgb: [u8; 3]) {
        if let Some(Custom::PhotoFilter(d)) = &mut self.custom {
            d.color = rgb;
            d.use_color = true;
        }
    }

    /// A popup's choice `k` for the field at `i`. Color Lookup's "Load 3D
    /// LUT..." opens a file and adds it to the list.
    fn pick(&mut self, i: usize, k: usize) {
        if self.kind == Kind::ColorLookup && k == 0 {
            let path = rfd::FileDialog::new()
                .add_filter("3D LUT", &["cube", "3dl", "CUBE", "3DL"])
                .pick_file();
            if let Some(path) = path
                && let Some(id) = load_lut(&path)
            {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                self.extra.luts.push((name, path));
                let index = self.extra.luts.len();
                self.extra.lut_ids.insert(index, id);
                self.values[i] = index.to_string();
            }
            return;
        }
        if self.kind == Kind::ColorLookup
            && !self.extra.lut_ids.contains_key(&k)
            && let Some(id) = self
                .extra
                .luts
                .get(k - 1)
                .and_then(|(_, path)| load_lut(path))
        {
            self.extra.lut_ids.insert(k, id);
        }
        self.values[i] = k.to_string();
    }

    /// Fibers' colors, and Replace Color's first color (the foreground).
    pub fn set_colors(&mut self, colors: ([u8; 3], [u8; 3])) {
        self.extra.colors = colors;
        self.extra.replace = op_core::color_match::ReplaceSamples::single(colors.0);
    }

    /// Match Color's statistics: the layer's and the other documents'.
    /// Match Color's statistics: the target layer's (all of it, and inside
    /// the selection), its document's title and layer's name, and the
    /// other documents'.
    pub fn set_match_sources(
        &mut self,
        (target, selected): ([f32; 6], Option<[f32; 6]>),
        (name, layer): (String, String),
        sources: Vec<MatchSource>,
    ) {
        self.extra.target = Some(target);
        self.extra.target_selected = selected;
        self.extra.target_name = name;
        self.extra.target_layer = layer;
        self.extra.sources = sources;
    }

    /// Whether an eyedropper is chosen (Levels, Curves): a click on the
    /// image then samples.
    pub fn sampling(&self) -> bool {
        // Replace Color samples its color from the image
        if self.kind == Kind::ReplaceColor {
            return true;
        }
        match &self.custom {
            Some(Custom::Levels(d)) => d.eyedropper.is_some(),
            Some(Custom::Curves(d)) => d.eyedropper.is_some(),
            Some(Custom::HueSaturation(d)) => d.eyedropper.is_some(),
            _ => false,
        }
    }

    /// Whether Hue/Saturation's targeted adjustment hand is on: drags on
    /// the image change the range under the pointer.
    pub fn targeting(&self) -> bool {
        match &self.custom {
            Some(Custom::HueSaturation(d)) => d.targeting,
            Some(Custom::Curves(d)) => d.targeting,
            _ => false,
        }
    }

    /// The hand pressed on a pixel of color `rgb` (Command: hue).
    pub fn target_press(&mut self, rgb: [u8; 3], hue: bool) {
        match &mut self.custom {
            Some(Custom::HueSaturation(d)) => d.target_press(rgb, hue),
            Some(Custom::Curves(d)) => d.target_press(rgb),
            _ => {}
        }
    }

    /// The hand dragged by `delta` points from where it was pressed:
    /// Hue/Saturation follows it sideways, Curves up and down.
    pub fn target_drag(&mut self, delta: egui::Vec2, hue: bool) {
        match &mut self.custom {
            Some(Custom::HueSaturation(d)) => d.target_drag(delta.x, hue),
            Some(Custom::Curves(d)) => d.target_drag(delta.y),
            _ => {}
        }
    }

    pub fn target_release(&mut self) {
        match &mut self.custom {
            Some(Custom::HueSaturation(d)) => d.target_release(),
            Some(Custom::Curves(d)) => d.target_release(),
            _ => {}
        }
    }

    /// Curves: the image pixel under the pointer, marked on the curve.
    pub fn curves_probe(&mut self, rgb: Option<[u8; 3]>) {
        if let Some(Custom::Curves(d)) = &mut self.custom {
            d.set_probe(rgb);
        }
    }

    /// Curves: ⌘-click on the image (⇧ for every channel) adds points.
    /// Returns whether this is the Curves dialog.
    pub fn curves_add_points(&mut self, rgb: [u8; 3], each_channel: bool) -> bool {
        if let Some(Custom::Curves(d)) = &mut self.custom {
            d.add_points_at(rgb, each_channel);
            return true;
        }
        false
    }

    /// Replace Color's eyedropper click on a pixel of color `rgb` at `at`
    /// (document pixels): the chosen tool picks, adds or takes out the
    /// color; Shift adds and Option subtracts whatever tool is chosen, as
    /// in Photoshop.
    pub fn sample_at(&mut self, rgb: [u8; 3], at: Option<(u32, u32)>, (shift, alt): (bool, bool)) {
        let tool = if alt {
            2
        } else if shift {
            1
        } else {
            self.extra.rc_tool
        };
        let r = &mut self.extra.replace;
        match tool {
            1 => r.add(rgb, at),
            2 => r.subtract(rgb),
            _ => r.pick(rgb, at),
        }
    }

    /// The chosen eyedropper's click on a pixel of color `rgb`.
    pub fn sample(&mut self, rgb: [u8; 3]) {
        if self.kind == Kind::ReplaceColor {
            self.sample_at(rgb, None, (false, false));
            return;
        }
        match &mut self.custom {
            Some(Custom::Levels(d)) => d.sample(rgb),
            Some(Custom::Curves(d)) => d.sample(rgb),
            Some(Custom::HueSaturation(d)) => d.sample(rgb),
            _ => {}
        }
    }

    /// The adjustment dialogs' settings at OK, to open with when the
    /// command is chosen with Option held (Photoshop's "last settings").
    pub fn remembered(&self) -> Option<Remembered> {
        self.custom.clone().map(Remembered)
    }

    /// Opens with settings from [`remembered`](Self::remembered) (of the
    /// same kind of dialog).
    pub fn recall(&mut self, last: &Remembered) {
        if let Some(c) = &self.custom
            && std::mem::discriminant(c) == std::mem::discriminant(&last.0)
        {
            self.custom = Some(last.0.clone());
        }
    }

    /// Exposure's exposure field (tests).
    #[cfg(test)]
    pub fn test_exposure(&self) -> String {
        match &self.custom {
            Some(Custom::Exposure(d)) => d.values[0].clone(),
            _ => String::new(),
        }
    }

    /// Types `text` into the field at `i` (tests).
    #[cfg(test)]
    pub fn test_set_value(&mut self, i: usize, text: &str) {
        self.values[i] = text.into();
    }

    /// Photo Filter's Color choice and color (tests).
    #[cfg(test)]
    pub fn test_filter_color(&self) -> Option<(bool, [u8; 3])> {
        match &self.custom {
            Some(Custom::PhotoFilter(d)) => Some((d.use_color, d.color)),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn test_set_exposure(&mut self, text: &str) {
        if let Some(Custom::Exposure(d)) = &mut self.custom {
            d.values[0] = text.into();
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
            || self.kind == Kind::Shear
            || self.kind == Kind::LensBlur
            || self.kind == Kind::Wave
            || legacy::preview_rect(self.kind).is_some()
            || self.kind.distort().is_some()
            || self.layout().is_some_and(|l| l.pane)
    }

    /// The classic layout as shown now: Shadows/Highlights has a short one
    /// until Show More Options is checked.
    fn layout(&self) -> Option<&'static filter_layout::Layout> {
        if self.kind == Kind::ShadowsHighlights && self.shadows_highlights_short() {
            return None;
        }
        self.kind.layout()
    }

    /// Gradient Map's two colors (the foreground and background colors).
    pub fn set_gradient_colors(&mut self, colors: ([u8; 3], [u8; 3])) {
        if let Some(Custom::GradientMap(d)) = &mut self.custom {
            d.gradient =
                op_core::gradient::Gradient::two("Foreground to Background", colors.0, colors.1);
        }
    }

    /// Gradient Map's gradient was clicked: the editor opens with it.
    pub fn take_editor_request(&mut self) -> Option<op_core::gradient::Gradient> {
        match &mut self.custom {
            Some(Custom::GradientMap(d)) => {
                std::mem::take(&mut d.wants_editor).then(|| d.gradient.clone())
            }
            _ => None,
        }
    }

    /// The Gradient Editor's OK for Gradient Map.
    pub fn set_map_gradient(&mut self, g: op_core::gradient::Gradient) {
        if let Some(Custom::GradientMap(d)) = &mut self.custom {
            d.gradient = g;
        }
    }

    /// Hue/Saturation's Colorize starts from this hue.
    pub fn set_colorize_hue(&mut self, hue: i32) {
        if let Some(Custom::HueSaturation(d)) = &mut self.custom {
            d.colorize_values[0] = hue.to_string();
        }
    }

    /// Match Color's Source choice, as typed (no range check).
    fn source_pick(&self) -> Option<usize> {
        if self.kind != Kind::MatchColor {
            return None;
        }
        self.values
            .get(4)?
            .trim()
            .parse::<f32>()
            .ok()
            .map(|v| v as usize)
    }

    fn value(&self, i: usize) -> Option<f32> {
        let p = &self.kind.params()[i];
        // (a field may show its unit, as Shadows/Highlights' "35%")
        let v: f32 = self.values[i]
            .trim()
            .trim_end_matches('%')
            .trim()
            .parse()
            .ok()?;
        // Popups whose choices the dialog fills in itself
        if let Some(labels) = self.extra.labels(self.kind, i, self.source_pick()) {
            return (v >= 0.0 && (v as usize) < labels.len()).then_some(v);
        }
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
        use op_core::more_filters as mf;
        let pick = |k: f32| k as usize;
        let undefined =
            |k: f32| [mf::Undefined::WrapAround, mf::Undefined::RepeatEdge][pick(k).min(1)];
        let e = &self.extra;
        let filter = match self.kind {
            Kind::RadialBlur => Filter::RadialBlur {
                amount: v[0],
                method: [mf::RadialMethod::Spin, mf::RadialMethod::Zoom][pick(v[1])],
                quality: v[2] as u8,
                center: e.radial_center,
            },
            Kind::SmartBlur => Filter::SmartBlur {
                radius: v[0],
                threshold: v[1],
                quality: v[2] as u8,
                mode: [
                    mf::SmartBlurMode::Normal,
                    mf::SmartBlurMode::EdgeOnly,
                    mf::SmartBlurMode::OverlayEdge,
                ][pick(v[3])],
            },
            Kind::ShapeBlur => Filter::ShapeBlur {
                radius: v[0],
                shape: mf::BlurShape::ALL[pick(v[1])],
            },
            Kind::LensBlur => Filter::LensBlur(mf::LensBlur {
                radius: v[lens_blur::RADIUS],
                blades: v[lens_blur::SHAPE] as u32 + 3,
                curvature: v[lens_blur::CURVATURE],
                rotation: v[lens_blur::ROTATION],
                brightness: v[lens_blur::BRIGHTNESS],
                threshold: v[lens_blur::THRESHOLD] as u8,
                noise: v[lens_blur::AMOUNT],
                gaussian: v[lens_blur::DISTRIBUTION] == 1.0,
                monochromatic: v[lens_blur::MONOCHROMATIC] == 1.0,
                depth: [
                    mf::DepthSource::None,
                    mf::DepthSource::Transparency,
                    mf::DepthSource::LayerMask,
                ][pick(v[lens_blur::SOURCE]).min(2)],
                focal: v[lens_blur::FOCAL] as u8,
                invert: v[lens_blur::INVERT] == 1.0,
            }),
            Kind::ReduceNoise => {
                // Per Channel counts only in Advanced
                let advanced = v[reduce_noise::MODE] == 1.0;
                let c = reduce_noise::CHANNELS;
                let channel = |k: usize| {
                    if advanced {
                        (v[c + k * 2], v[c + k * 2 + 1])
                    } else {
                        (0.0, 60.0)
                    }
                };
                Filter::ReduceNoise(mf::ReduceNoise {
                    strength: v[1],
                    preserve: v[2],
                    color: v[3],
                    sharpen: v[4],
                    jpeg: v[reduce_noise::JPEG] == 1.0,
                    channels: [channel(0), channel(1), channel(2)],
                })
            }
            Kind::SmartSharpen => {
                let tone = |k: usize| mf::ToneFade {
                    fade: v[k],
                    tonal_width: v[k + 1],
                    radius: v[k + 2],
                };
                Filter::SmartSharpen(mf::SmartSharpen {
                    amount: v[0],
                    radius: v[1],
                    noise: v[2],
                    remove: [
                        mf::SharpenRemove::GaussianBlur,
                        mf::SharpenRemove::LensBlur,
                        mf::SharpenRemove::MotionBlur,
                    ][pick(v[3]).min(2)],
                    angle: v[4],
                    shadows: tone(smart_sharpen::SHADOWS),
                    highlights: tone(smart_sharpen::HIGHLIGHTS),
                    legacy: v[smart_sharpen::LEGACY] == 1.0,
                    more_accurate: v[smart_sharpen::MORE_ACCURATE] == 1.0,
                })
            }
            Kind::Fibers => Filter::Fibers {
                variance: v[0],
                strength: v[1],
                foreground: e.colors.0,
                background: e.colors.1,
                seed: e.seed,
            },
            Kind::LensFlare => Filter::LensFlare {
                center: e.flare_center,
                brightness: v[0],
                lens: mf::LensType::ALL[pick(v[1])],
            },
            Kind::Extrude => Filter::Extrude {
                kind: [mf::ExtrudeType::Blocks, mf::ExtrudeType::Pyramids][pick(v[0])],
                size: v[1] as u32,
                depth: v[2],
                level_based: v[3] == 1.0,
                solid: v[4] == 1.0,
                mask_incomplete: v[5] == 1.0,
                seed: e.seed,
            },
            Kind::OilPaint => Filter::OilPaint(mf::OilPaint {
                stylization: v[0],
                cleanliness: v[1],
                scale: v[2],
                bristle: v[oil_paint::BRISTLE],
                lighting: v[oil_paint::LIGHTING] == 1.0,
                angle: v[oil_paint::ANGLE],
                shine: v[oil_paint::SHINE],
            }),
            Kind::Wave => Filter::Wave(mf::Wave {
                generators: v[0] as u32,
                wavelength: (v[1], v[2].max(v[1] + 1.0)),
                amplitude: (v[3], v[4].max(v[3])),
                scale: (v[5], v[6]),
                kind: [
                    mf::WaveType::Sine,
                    mf::WaveType::Triangle,
                    mf::WaveType::Square,
                ][pick(v[7])],
                undefined: undefined(v[8]),
                seed: e.seed,
            }),
            Kind::Shear => {
                let mut points = [(0.0, 0.0); 8];
                let count = e.shear_points.len().min(8);
                points[..count].copy_from_slice(&e.shear_points[..count]);
                Filter::Shear {
                    points,
                    count: count as u8,
                    undefined: undefined(v[0]),
                }
            }
            Kind::Displace => Filter::Displace {
                map: e.displace_map.unwrap_or(u32::MAX),
                scale: (v[0], v[1]),
                stretch: v[2] == 0.0,
                undefined: undefined(v[3]),
            },
            Kind::ShadowsHighlights => {
                Filter::ShadowsHighlights(op_core::tone::ShadowsHighlights {
                    shadows: [v[0], v[1], v[2]],
                    highlights: [v[3], v[4], v[5]],
                    color: v[6],
                    midtone: v[7],
                    clip: [v[8], v[9]],
                })
            }
            Kind::HdrToning => Filter::HdrToning(op_core::tone::HdrToning {
                method: op_core::tone::HdrMethod::ALL[pick(v[0])],
                radius: v[1],
                strength: v[2],
                gamma: v[3],
                exposure: v[4],
                detail: v[5],
                shadow: v[6],
                highlight: v[7],
                vibrance: v[8],
                saturation: v[9],
                smooth_edges: v[10] != 0.0,
                curve: {
                    let mut c = [(0, 0); 16];
                    for (d, &p) in c.iter_mut().zip(&e.hdr.curve) {
                        *d = p;
                    }
                    c
                },
                curve_len: e.hdr.curve.len().min(16) as u8,
                corners: e
                    .hdr
                    .corners
                    .iter()
                    .enumerate()
                    .fold(0, |m, (k, &c)| if c { m | 1 << k } else { m }),
            }),
            Kind::ReplaceColor => {
                let mut samples = e.replace;
                samples.localized = v[4] != 0.0;
                return Some(Effect::Adjustment(Adjustment::ReplaceColor {
                    samples,
                    fuzziness: v[0] as u8,
                    shift: [v[1] as i32, v[2] as i32, v[3] as i32],
                }));
            }
            Kind::MatchColor => {
                // Use Selection in Target / Source: the statistics inside
                // the selection, where there is one
                let target = match (v[8] != 0.0, e.target_selected) {
                    (true, Some(t)) => t,
                    _ => e.target?,
                };
                let mut source = match pick(v[4]) {
                    0 => target,
                    k => {
                        let layer = e.sources.get(k - 1)?.layers.get(pick(v[5]))?;
                        match (v[7] != 0.0, layer.2) {
                            (true, Some(s)) => s,
                            _ => layer.1,
                        }
                    }
                };
                if v[3] == 1.0 {
                    // Neutralize: the color cast's mean taken out
                    source[1] = 0.0;
                    source[2] = 0.0;
                }
                return Some(Effect::Adjustment(Adjustment::MatchColor {
                    target,
                    source,
                    luminance: v[0],
                    intensity: v[1],
                    fade: v[2],
                    ignore_selection: v[6] != 0.0,
                }));
            }
            Kind::ColorLookup => {
                // Only 3DLUT files apply (Abstract and Device Link are off)
                if v[1] != 0.0 {
                    return None;
                }
                let id = *e.lut_ids.get(&pick(v[0]))?;
                return Some(Effect::Adjustment(Adjustment::ColorLookup(id, v[2] != 0.0)));
            }
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
            Kind::HsbHsl => {
                let model =
                    |i: f32| [ColorModel::Rgb, ColorModel::Hsb, ColorModel::Hsl][i as usize];
                Filter::HsbHsl {
                    input: model(v[0]),
                    output: model(v[1]),
                }
            }
            Kind::ZigZag => Filter::ZigZag {
                amount: v[0] as i32,
                ridges: v[1] as u32,
                style: [
                    ZigZagStyle::AroundCenter,
                    ZigZagStyle::OutFromCenter,
                    ZigZagStyle::PondRipples,
                ][v[2] as usize],
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
                    None if self.kind == Kind::LensBlur => lens_blur::size(ctx.content_rect()),
                    None if self.kind == Kind::Wave => wave::SIZE,
                    None if self.kind == Kind::ReduceNoise => reduce_noise::SIZE,
                    None if self.kind == Kind::SmartSharpen => self.smart_sharpen_size(),
                    None if self.kind == Kind::OilPaint => oil_paint::SIZE,
                    None if self.kind == Kind::ShapeBlur => shape_blur::SIZE,
                    None if self.kind == Kind::ColorLookup => color_lookup::SIZE,
                    None if self.kind == Kind::ReplaceColor => replace_color::SIZE,
                    None if self.kind == Kind::MatchColor => match_color::SIZE,
                    None if self.kind == Kind::HdrToning => self.hdr_size(),
                    None if self.kind == Kind::ShadowsHighlights
                        && self.shadows_highlights_short() =>
                    {
                        shadows_highlights::SIZE
                    }
                    None => self
                        .layout()
                        .map_or_else(|| self.kind.size(), |l| vec2(pt(l.size.0), pt(l.size.1))),
                };
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                self.rect = rect;
                outcome = if self.custom.is_some() {
                    self.custom_ui(ui, rect)
                } else if self.kind == Kind::LensBlur {
                    self.lens_ui(ui, rect)
                } else if self.kind == Kind::Wave {
                    self.wave_ui(ui, rect)
                } else if self.kind == Kind::ReduceNoise {
                    self.reduce_noise_ui(ui, rect)
                } else if self.kind == Kind::SmartSharpen {
                    self.smart_sharpen_ui(ui, rect)
                } else if self.kind == Kind::OilPaint {
                    self.oil_paint_ui(ui, rect)
                } else if self.kind == Kind::ShapeBlur {
                    self.shape_blur_ui(ui, rect)
                } else if self.kind == Kind::ColorLookup {
                    self.color_lookup_ui(ui, rect)
                } else if self.kind == Kind::ReplaceColor {
                    self.replace_color_ui(ui, rect)
                } else if self.kind == Kind::MatchColor {
                    self.match_color_ui(ui, rect)
                } else if self.kind == Kind::HdrToning {
                    self.hdr_toning_ui(ui, rect)
                } else if self.kind == Kind::ShadowsHighlights && self.shadows_highlights_short() {
                    self.shadows_highlights_ui(ui, rect)
                } else if let Some(layout) = self.layout() {
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
        if self.blocked {
            return Outcome::Open;
        }
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
            let zoom = zoom_controls(
                ui,
                at(0.0, filter_layout::ZOOM_Y).y,
                frame.left(),
                self.pane_zoom,
            );
            self.zoom_pane(zoom);
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
            Some(Custom::Levels(d)) => {
                let pressed = d.ui(ui, frame, self.first_frame, &mut self.preview);
                if std::mem::take(&mut d.wants_auto) {
                    d.auto(op_core::auto::compute(
                        &self.extra.auto_samples,
                        &self.extra.auto_options,
                    ));
                }
                self.extra.wants_auto_options |= std::mem::take(&mut d.wants_options);
                pressed
            }
            Some(Custom::Curves(d)) => {
                let pressed = d.ui(ui, frame, &mut self.preview);
                if std::mem::take(&mut d.wants_auto) {
                    d.auto(op_core::auto::compute(
                        &self.extra.auto_samples,
                        &self.extra.auto_options,
                    ));
                }
                self.extra.wants_auto_options |= std::mem::take(&mut d.wants_options);
                pressed
            }
            Some(Custom::ChannelMixer(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::SelectiveColor(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Vibrance(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Posterize(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::Exposure(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::PhotoFilter(d)) => d.ui(ui, frame, self.first_frame, &mut self.preview),
            Some(Custom::BlackWhite(d)) => {
                let pressed = d.ui(ui, frame, self.first_frame, &mut self.preview);
                if pressed == Some(uxp::Button::Third) {
                    d.auto(op_core::auto::black_white(&self.extra.auto_samples));
                }
                pressed
            }
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
            let zoom = zoom_controls(
                ui,
                at(0.0, filter_layout::ZOOM_Y).y,
                frame.left(),
                self.pane_zoom,
            );
            self.zoom_pane(zoom);
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
                        let labels: Vec<String> = self
                            .extra
                            .labels(self.kind, i, self.source_pick())
                            .unwrap_or_else(|| options.iter().map(|o| o.to_string()).collect());
                        let chosen = (self.value(i).unwrap_or(0.0) as usize).min(labels.len() - 1);
                        let id = format!("filter-popup-{i}");
                        if let Some(k) = appkit::popup(
                            ui,
                            rect,
                            &id,
                            &labels[chosen],
                            &appkit::choices(labels.iter().map(String::as_str), chosen),
                        ) {
                            self.pick(i, k);
                        }
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
                            // Offset on the Background layer fills with the
                            // background color, and says so
                            let option = if self.kind == Kind::Offset
                                && k == 0
                                && self.extra.on_background
                            {
                                "Set to Background"
                            } else {
                                option
                            };
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

    /// The plug-in style dialogs' field-over-slider rows: one per setting
    /// from the first, the first row's field at `distort::FIELD_Y` and its
    /// track at `distort::TRACK_Y`, later rows lower by their labels'
    /// distance from the first's.
    fn distort_sliders(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        label_ys: &[f32],
        (field_x, track_x1): (f32, f32),
        (rows_dy, field_w): (f32, f32),
    ) {
        let params = self.kind.params();
        for (k, &label_y) in label_ys.iter().enumerate() {
            let dy = label_y - label_ys[0];
            let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y + dy));
            // The field and track, moved by the layout's `rows_dy`
            let at_row = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y + dy + rows_dy));
            let p = &params[k];
            let label = p.label.split(" (").next().unwrap_or(p.label);
            // Lens Flare's keeps its colon
            let label = if self.kind == Kind::LensFlare {
                label
            } else {
                label.trim_end_matches(':')
            };
            distort::label(ui, at(23.5, label_ys[0]), Align2::LEFT_CENTER, label);
            let field = Rect::from_min_max(
                at_row(field_x, distort::FIELD_Y.0),
                at_row(field_x + field_w, distort::FIELD_Y.1),
            );
            appkit::field(
                ui,
                field,
                &mut self.values[k],
                ("distort-field", k),
                (p.min, p.max),
                10f32.powi(-(p.decimals as i32)),
                p.decimals,
                self.first_frame && k == 0,
            );
            // Crystallize's, Pointillize's and ZigZag's settings have no unit
            let unit = match self.kind {
                Kind::Twirl => "°",
                Kind::Crystallize
                | Kind::Pointillize
                | Kind::ZigZag
                | Kind::Fibers
                | Kind::SmartBlur => "",
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
                distort::slider_rect(at_row, track_x1),
                ui.id().with(("distort-slider", k)),
                Sense::click_and_drag(),
            );
            if (response.dragged() || response.clicked())
                && let Some(pointer) = response.interact_pointer_pos()
            {
                let t = distort::slider_place(at_row, track_x1, pointer.x);
                let round = 10f32.powi(p.decimals as i32);
                self.set(k, ((p.min + (p.max - p.min) * t) * round).round() / round);
            }
            let v = self.value(k).unwrap_or(p.default);
            distort::slider(ui, at_row, track_x1, (v - p.min) / (p.max - p.min));
        }
    }

    /// Lens Flare's center in its preview: a crosshair where it is; a click
    /// or drag puts it there (as a fraction of the image).
    fn flare_center_ui(&mut self, ui: &mut Ui, view: Rect) {
        let (Some((cx, cy)), (w, h)) = (self.pane_center, self.extra.doc_size) else {
            return;
        };
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        // The preview shows the image at its zoom around `pane_center`
        let k = pt(0.5) * self.pane_zoom;
        let to_view = |(fx, fy): (f32, f32)| {
            Pos2::new(
                view.center().x + (fx * w - cx) * k,
                view.center().y + (fy * h - cy) * k,
            )
        };
        let response = ui.interact(view, ui.id().with("flare-center"), Sense::click_and_drag());
        if (response.clicked() || response.dragged())
            && let Some(p) = response.interact_pointer_pos()
        {
            self.extra.flare_center = (
                ((cx + (p.x - view.center().x) / k) / w).clamp(0.0, 1.0),
                ((cy + (p.y - view.center().y) / k) / h).clamp(0.0, 1.0),
            );
        }
        let c = to_view(self.extra.flare_center);
        let painter = ui.painter_at(view);
        let s = Stroke::new(pt(1.0), Color32::BLACK);
        painter.line_segment([c - vec2(pt(4.0), 0.0), c + vec2(pt(4.0), 0.0)], s);
        painter.line_segment([c - vec2(0.0, pt(4.0)), c + vec2(0.0, pt(4.0))], s);
    }

    /// A Distort filter's plug-in style dialog (see `distort`).
    fn distort_ui(&mut self, ui: &mut Ui, frame: Rect, layout: &distort::Layout) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, self.kind.title(), theme::dialog_bold(pt(13.0)));
        if let Some(v) = layout.view {
            // A plain box; Lens Flare's center is placed in it
            let b = r(v[0], v[1], v[2], v[3]);
            ui.painter().rect(
                b,
                0,
                Color32::from_gray(0x4d),
                Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
                egui::StrokeKind::Inside,
            );
            let inner = b.shrink(pt(1.0));
            self.pane_ui(ui, inner);
            if self.kind == Kind::LensFlare {
                self.flare_center_ui(ui, inner);
            }
        } else {
            distort::frame(ui, at);
            if let Some(texture) = &self.pane {
                distort::image(ui, at, texture);
            }
            let view = distort::VIEW;
            self.pane_drag(ui, r(view[0], view[1], view[2], view[3]));
            let zoom = distort::zoom_bar(ui, at, &zoom_label(self.pane_zoom));
            self.zoom_pane(zoom);
        }

        let params = self.kind.params();
        match layout.control {
            distort::Control::Slider {
                label_y,
                field_x,
                track_x1,
            } => self.distort_sliders(
                ui,
                frame,
                &[label_y],
                (field_x, track_x1),
                (layout.rows_dy, layout.field_w),
            ),
            distort::Control::Sliders {
                label_ys,
                field_x,
                track_x1,
            } => self.distort_sliders(
                ui,
                frame,
                label_ys,
                (field_x, track_x1),
                (layout.rows_dy, layout.field_w),
            ),
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
        // Radio buttons for the setting after the sliders
        if let Some(group) = &layout.after {
            let i = match layout.control {
                distort::Control::Slider { .. } => 1,
                distort::Control::Sliders { label_ys, .. } => label_ys.len(),
                _ => 0,
            };
            distort::group(ui, at, group);
            if let Some(ParamKind::Choice(options)) = params.get(i).map(|p| p.kind) {
                let chosen = self.value(i).unwrap_or(0.0) as usize;
                for (k, (&y, option)) in group.ys.iter().zip(options).enumerate() {
                    if distort::radio(ui, at(group.x, y), option, chosen == k) {
                        self.values[i] = k.to_string();
                    }
                }
            }
        }
        // Pop-ups for the settings after the sliders
        let rows = match layout.control {
            distort::Control::Slider { .. } => 1,
            distort::Control::Sliders { label_ys, .. } => label_ys.len(),
            _ => 0,
        };
        for (k, &(rect, label_right)) in layout.popups.iter().enumerate() {
            let i = rows + k;
            let Some(ParamKind::Choice(options)) = params.get(i).map(|p| p.kind) else {
                continue;
            };
            distort::label(
                ui,
                at(label_right, (rect[1] + rect[3]) / 2.0),
                Align2::RIGHT_CENTER,
                params[i].label,
            );
            let chosen = (self.value(i).unwrap_or(0.0) as usize).min(options.len() - 1);
            if let Some(k) = appkit::popup(
                ui,
                r(rect[0], rect[1], rect[2], rect[3]),
                &format!("distort-popup-{i}"),
                options[chosen],
                &appkit::choices(options.iter().copied(), chosen),
            ) {
                self.values[i] = k.to_string();
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
            let chosen = (self.value(last).unwrap_or(0.0) as usize).min(options.len() - 1);
            if let Some(k) = appkit::popup(
                ui,
                r(rect[0], rect[1], rect[2], rect[3]),
                "distort-mode",
                options[chosen],
                &appkit::choices(options.iter().copied(), chosen),
            ) {
                self.values[last] = k.to_string();
            }
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

        if let Some(b) = layout.randomize {
            let rect = r(b[0], b[1], b[2], b[3]);
            if flat_button(ui, rect, "Randomize", "distort-randomize") {
                self.extra.seed = self
                    .extra
                    .seed
                    .wrapping_mul(747_796_405)
                    .wrapping_add(2_891_336_453);
            }
        }

        let x0 = layout.buttons_x;
        let valid = self.effect().is_some();
        let ok = appkit::button_with(
            ui,
            r(x0, 41.0, x0 + layout.buttons_w, 67.0),
            "OK",
            (true, valid),
            13.0,
            0.0,
        );
        let cancel = appkit::button_with(
            ui,
            r(x0, 77.0, x0 + layout.buttons_w, 103.0),
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
        let font = theme::dialog(pt(layout.text_size));
        let label_at = |ui: &Ui, pos: Pos2, text: &str| {
            let galley = theme::tracked_galley(ui.painter(), text, font.clone(), appkit::TEXT);
            let rect = Align2::LEFT_CENTER.anchor_size(pos, galley.size());
            ui.painter().galley(rect.min, galley, appkit::TEXT);
        };
        let label = |ui: &Ui, pos: Pos2, text: &str| label_at(ui, pos, text);
        let mut i = 0;
        // The first field takes the focus
        let mut focused = false;
        for item in layout.items {
            match *item {
                Item::Text { text, at: (x, y) } => {
                    label(ui, at(x, y), text);
                }
                Item::Field { label, rect, unit } => {
                    let field = r(rect);
                    let cy = field.center().y;
                    label_at(ui, Pos2::new(at(label.1, 0.0).x, cy), label.0);
                    let p = &params[i];
                    appkit::field(
                        ui,
                        field,
                        &mut self.values[i],
                        ("plain-field", i),
                        (p.min, p.max),
                        1.0,
                        p.decimals,
                        self.first_frame && !focused,
                    );
                    focused = true;
                    if let Some((unit, x)) = unit {
                        label_at(ui, Pos2::new(at(x, 0.0).x, cy), unit);
                    }
                    i += 1;
                }
                Item::Track { x0, x1, top, pins } => self.plain_track(
                    ui,
                    i - 1,
                    (at(x0, top), at(x1, top).x),
                    (at(pins.0, 0.0).x, at(pins.1, 0.0).x),
                ),
                Item::Check {
                    label,
                    min,
                    size,
                    gap,
                } => {
                    let mut on = self.value(i) == Some(1.0);
                    appkit::checkbox_with(ui, at(min.0, min.1), (size, gap), label, &mut on, true);
                    self.values[i] = (on as u8).to_string();
                    i += 1;
                }
                Item::Group {
                    title,
                    rect,
                    title_x,
                } => {
                    let g = r(rect);
                    let title_x = at(title_x, 0.0).x;
                    let galley =
                        theme::tracked_galley(ui.painter(), title, font.clone(), appkit::TEXT);
                    appkit::group(
                        ui.painter(),
                        g,
                        (title_x - pt(4.0), title_x + galley.size().x + pt(4.0)),
                    );
                    label(ui, Pos2::new(title_x, g.top() - pt(0.25)), title);
                }
                Item::CenterBox { rect } => self.center_box(ui, r(rect)),
                Item::CurveGrid { rect } => self.curve_grid(ui, r(rect)),
                Item::Pane { rect } => {
                    let b = r(rect);
                    ui.painter().rect(
                        b,
                        0,
                        Color32::from_gray(0x4d),
                        Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
                        egui::StrokeKind::Inside,
                    );
                    self.pane_ui(ui, b.shrink(pt(1.0)));
                }
                Item::Radios { centers, gap } => {
                    if let ParamKind::Choice(options) = params[i].kind {
                        let chosen = self.value(i).unwrap_or(0.0) as usize;
                        for (k, (&(x, y), option)) in centers.iter().zip(options).enumerate() {
                            let font = font.clone();
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
        let (bw, ok_y, cancel_y, bh, size) = layout.buttons;
        let valid = self.effect().is_some();
        let ok = appkit::button_with(
            ui,
            Rect::from_min_max(at(x0, ok_y), at(x0 + bw, ok_y + bh)),
            "OK",
            (true, valid),
            size,
            0.0,
        );
        let cancel = appkit::button_with(
            ui,
            Rect::from_min_max(at(x0, cancel_y), at(x0 + bw, cancel_y + bh)),
            "Cancel",
            (false, true),
            size,
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
    /// The preview's size in pixels (the classic pane's 196 pt square,
    /// the plug-in style dialogs' 256 pt one, at two pixels a point).
    pub fn pane_px(&self) -> (usize, usize) {
        if let Some([x0, y0, x1, y1]) = self.legacy_area() {
            (((x1 - x0) * 2.0) as usize, ((y1 - y0) * 2.0) as usize)
        } else if self.kind == Kind::Shear {
            (600, 300)
        } else if self.kind == Kind::Wave {
            let [x0, y0, x1, y1] = wave::PREVIEW;
            (
                ((x1 - x0 - 2.0) * 2.0) as usize,
                ((y1 - y0 - 2.0) * 2.0) as usize,
            )
        } else if self.kind == Kind::LensBlur {
            // The preview's image area, at two pixels a point
            let image = lens_blur::image_rect(self.rect);
            if !image.is_positive() || !image.is_finite() {
                return (800, 600);
            }
            (
                (image.width() / pt(1.0) * 2.0) as usize,
                (image.height() / pt(1.0) * 2.0) as usize,
            )
        } else if self.kind.distort().is_some() {
            (512, 512)
        } else {
            (392, 392)
        }
    }

    /// Zoom out (false) or in (true) a step.
    fn zoom_pane(&mut self, step: Option<bool>) {
        let Some(up) = step else {
            return;
        };
        let i = PANE_ZOOMS
            .iter()
            .position(|&z| z >= self.pane_zoom - 1e-4)
            .unwrap_or(PANE_ZOOMS.len() - 1);
        let j = if up {
            (i + 1).min(PANE_ZOOMS.len() - 1)
        } else {
            i.saturating_sub(1)
        };
        if PANE_ZOOMS[j] != self.pane_zoom {
            self.pane_zoom = PANE_ZOOMS[j];
            self.pane = None;
        }
    }

    /// A click on the document while the preview shows: it centers there.
    pub fn center_pane(&mut self, x: f32, y: f32) {
        self.pane_center = Some((x, y));
        self.pane = None;
    }

    /// Dragging in the preview moves the picture with the pointer.
    fn pane_drag(&mut self, ui: &mut Ui, rect: Rect) {
        let response = ui.interact(rect, ui.id().with("pane-drag"), Sense::drag());
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(if response.dragged() {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::Grab
            });
        }
        if response.drag_started()
            && let Some(p) = response.interact_pointer_pos()
        {
            self.pane_drag = Some((p, self.pane_center.unwrap_or((f32::NAN, f32::NAN))));
        }
        if let Some((from, start)) = self.pane_drag
            && let Some(p) = response.interact_pointer_pos()
            && response.dragged()
            && !start.0.is_nan()
        {
            // Two preview pixels a point
            let d = (p - from) / pt(1.0) * 2.0 / self.pane_zoom;
            let center = (start.0 - d.x, start.1 - d.y);
            if Some(center) != self.pane_center {
                self.pane_center = Some(center);
                self.pane = None;
            }
        }
        if response.drag_stopped() {
            self.pane_drag = None;
        }
    }

    fn pane_ui(&mut self, ui: &mut Ui, rect: Rect) {
        self.pane_drag(ui, rect);
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

    /// A plain dialog's slider (`plain_filter::Item::Track`): the track
    /// from `start` to `end_x`, the pin standing on it (tip 3 pt above),
    /// its middle from `pins.0` (minimum) to `pins.1` (maximum), evenly.
    fn plain_track(
        &mut self,
        ui: &mut Ui,
        i: usize,
        (start, end_x): (Pos2, f32),
        pins: (f32, f32),
    ) {
        let p = &self.kind.params()[i];
        let (min, max, decimals) = (p.min, p.max, p.decimals);
        let line = Rect::from_min_max(start, Pos2::new(end_x, start.y + pt(3.0)));
        ui.painter().rect_filled(line, 0, Color32::from_gray(0x75));
        let hit = Rect::from_min_max(
            Pos2::new(line.left() - pt(4.0), line.top() - pt(5.0)),
            Pos2::new(line.right() + pt(4.0), line.bottom() + pt(7.0)),
        );
        let response = ui.interact(
            hit,
            ui.id().with(("plain-track", i)),
            Sense::click_and_drag(),
        );
        if (response.dragged() || response.clicked())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let t = ((pointer.x - pins.0) / (pins.1 - pins.0)).clamp(0.0, 1.0);
            let round = 10f32.powi(decimals as i32);
            self.set(i, ((min + (max - min) * t) * round).round() / round);
        }
        let v = self.value(i).unwrap_or(p.default);
        let x = pins.0 + (pins.1 - pins.0) * ((v - min) / (max - min)).clamp(0.0, 1.0);
        appkit::pin(
            ui.painter(),
            Pos2::new(x, line.top() - pt(3.0)),
            appkit::Pin::White,
        );
    }

    /// Radial Blur's Blur Center: white, with the blur's pattern drawn
    /// from points along a grid around the center (Spin: arcs of the amount's
    /// angle round the center; Zoom: strokes away from it, amount% of their
    /// distance long). A click or drag puts the center there.
    fn center_box(&mut self, ui: &mut Ui, rect: Rect) {
        let response = ui.interact(rect, ui.id().with("blur-center"), Sense::click_and_drag());
        if (response.clicked() || response.dragged())
            && let Some(p) = response.interact_pointer_pos()
        {
            self.extra.radial_center = (
                ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
                ((p.y - rect.top()) / rect.height()).clamp(0.0, 1.0),
            );
        }
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0, Color32::WHITE);
        let c = Pos2::new(
            rect.left() + rect.width() * self.extra.radial_center.0,
            rect.top() + rect.height() * self.extra.radial_center.1,
        );
        let amount = self.value(0).unwrap_or(10.0).clamp(1.0, 100.0);
        let zoom = self.value(1) == Some(1.0);
        let stroke = Stroke::new(pt(0.5), Color32::BLACK);
        // Points 8 pt apart along grid lines a quarter of the box apart,
        // through the center
        let (step, spacing) = (pt(8.0), rect.width() / 4.0);
        let lines = (rect.width().max(rect.height()) / spacing).ceil() as i32 + 1;
        let dots = (rect.width().max(rect.height()) / step).ceil() as i32 + 1;
        let mut points = Vec::new();
        for k in -lines..=lines {
            for j in -dots..=dots {
                points.push(vec2(k as f32 * spacing, j as f32 * step));
                points.push(vec2(j as f32 * step, k as f32 * spacing));
            }
        }
        for d in points {
            let p = c + d;
            if !rect.expand(pt(4.0)).contains(p) {
                continue;
            }
            let r = d.length();
            if zoom {
                let half = d * (amount / 200.0);
                painter.line_segment([p - half, p + half], stroke);
            } else if r < 0.5 {
                painter.circle_filled(p, pt(0.5), Color32::BLACK);
            } else {
                let a0 = d.y.atan2(d.x);
                let span = amount.to_radians();
                let n = 6;
                let arc: Vec<Pos2> = (0..=n)
                    .map(|k| {
                        let a = a0 + span * (k as f32 / n as f32 - 0.5);
                        c + vec2(a.cos(), a.sin()) * r
                    })
                    .collect();
                painter.add(egui::Shape::line(arc, stroke));
            }
        }
    }

    /// Shear's curve grid: dotted lines at the quarters, the curve (a
    /// natural spline through its points, as the filter uses) and its
    /// points as 4 pt black squares. Dragging a point moves it (the ends
    /// only sideways); pressing on the curve elsewhere adds a point there,
    /// up to eight; an inner point dragged off the box goes away.
    fn curve_grid(&mut self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter_at(rect.expand(pt(4.0)));
        painter.rect(
            rect,
            0,
            Color32::WHITE,
            Stroke::new(pt(0.5), Color32::BLACK),
            egui::StrokeKind::Inside,
        );
        let inner = rect.shrink(pt(0.5));
        // Dotted: a physical pixel on, one off, as a mesh (no feathering,
        // which would fade pixel-sized dots away)
        let px = 1.0 / ui.ctx().pixels_per_point();
        let snap = |v: f32| (v / px).round() * px;
        let mut mesh = egui::Mesh::default();
        let mut dot = |p: Pos2| {
            mesh.add_colored_rect(
                Rect::from_min_size(Pos2::new(snap(p.x), snap(p.y)), egui::Vec2::splat(px)),
                Color32::BLACK,
            );
        };
        for k in 1..4 {
            let f = k as f32 / 4.0;
            let x = inner.left() + inner.width() * f;
            let y = inner.top() + inner.height() * f;
            let mut t = 0.0;
            while t < inner.height() {
                dot(Pos2::new(x, inner.top() + t));
                dot(Pos2::new(inner.left() + t, y));
                t += 2.0 * px;
            }
        }
        painter.add(mesh);
        let to_screen = |(h, o): (f32, f32)| {
            Pos2::new(
                inner.center().x + o * inner.width(),
                inner.top() + h * inner.height(),
            )
        };
        let from_screen = |p: Pos2| {
            (
                ((p.y - inner.top()) / inner.height()).clamp(0.0, 1.0),
                ((p.x - inner.center().x) / inner.width()).clamp(-0.5, 0.5),
            )
        };
        let response = ui.interact(rect, ui.id().with("shear-grid"), Sense::click_and_drag());
        let points = &mut self.extra.shear_points;
        if response.drag_started() || (response.clicked() && self.extra.shear_drag.is_none()) {
            // Where the press was, not where the pointer is now
            let press = ui.input(|i| i.pointer.press_origin());
            if let Some(p) = press.or(response.interact_pointer_pos()) {
                let near = points
                    .iter()
                    .position(|&q| to_screen(q).distance(p) <= pt(4.0));
                self.extra.shear_drag = near.or_else(|| {
                    let spline = op_core::adjust::Spline::new(points);
                    let (h, _) = from_screen(p);
                    let on_curve = to_screen((h, spline.at(h))).distance(p) <= pt(4.0);
                    (on_curve && points.len() < 8 && h > 0.0 && h < 1.0).then(|| {
                        points.push(from_screen(p));
                        points.len() - 1
                    })
                });
            }
        }
        if let (Some(i), Some(p)) = (self.extra.shear_drag, response.interact_pointer_pos())
            && response.dragged()
            && i < points.len()
        {
            let (h, o) = from_screen(p);
            if i < 2 {
                // The ends stay at the top and bottom
                points[i].1 = o;
            } else if !rect.expand(pt(8.0)).contains(p) {
                points.remove(i);
                self.extra.shear_drag = None;
            } else {
                points[i] = (h.clamp(0.001, 0.999), o);
            }
        }
        if response.drag_stopped() || response.clicked() {
            self.extra.shear_drag = None;
        }
        let spline = op_core::adjust::Spline::new(points);
        let n = (inner.height() / pt(1.0)).ceil() as usize;
        let line: Vec<Pos2> = (0..=n)
            .map(|k| {
                let h = k as f32 / n as f32;
                to_screen((h, spline.at(h)))
            })
            .collect();
        painter.add(egui::Shape::line(
            line,
            Stroke::new(pt(0.5), Color32::BLACK),
        ));
        for (k, &q) in points.iter().enumerate() {
            let c = to_screen(q);
            // The ends' squares sit inside the box
            let c = match k {
                0 => c + vec2(0.0, pt(2.0)),
                1 => c - vec2(0.0, pt(2.0)),
                _ => c,
            };
            painter.rect_filled(
                Rect::from_center_size(c, egui::Vec2::splat(pt(4.0))),
                0,
                Color32::BLACK,
            );
        }
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

mod color_lookup;
mod hdr_toning;
mod legacy;
mod lens_blur;
mod match_color;
mod oil_paint;
mod reduce_noise;
mod replace_color;
mod shadows_highlights;
mod shape_blur;
mod smart_sharpen;

/// A flat 13 pt button as the plug-in dialogs' Randomize (`#454545`,
/// `#666666` edge, `#5a5a5a` while pressed). Returns whether it was
/// clicked.
fn flat_button(ui: &mut Ui, rect: Rect, label: &str, id: &str) -> bool {
    let response = ui.interact(rect, ui.id().with(id), Sense::click());
    ui.painter().rect(
        rect,
        pt(2.0),
        if response.is_pointer_button_down_on() {
            Color32::from_gray(0x5a)
        } else {
            Color32::from_gray(0x45)
        },
        Stroke::new(pt(1.0), Color32::from_gray(0x66)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        theme::dialog(pt(13.0)),
        appkit::TEXT,
    );
    response.clicked()
}
mod wave;
#[cfg(test)]
pub use lens_blur::image_rect as lens_image_rect;
pub use lens_blur::source as lens_blur_source;

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
            Kind::ZigZag,
            Kind::HsbHsl,
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
