//! Filter menu filters applied to the active layer, limited to the
//! selection.
//!
//! Filters read the whole layer (a blur near the selection edge sees the
//! pixels outside it) and write only the selected pixels. Colors are
//! filtered premultiplied by alpha, so transparent pixels don't bleed their
//! (meaningless) color into their neighbors.

use crate::document::Document;
use crate::fill::FillError;
use crate::tile::TiledImage;

/// What Filter > Other > Offset puts in the area uncovered by the shift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffsetFill {
    /// Transparent, or the background color on the background layer.
    Background,
    /// The nearest edge pixels.
    RepeatEdges,
    /// The pixels pushed off the other side.
    Wrap,
}

/// Spherize's Mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpherizeMode {
    Normal,
    HorizontalOnly,
    VerticalOnly,
}

/// The channel models HSB/HSL converts between.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorModel {
    Rgb,
    Hsb,
    Hsl,
}

/// ZigZag's Style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZigZagStyle {
    /// Pixels turn back and forth around the center.
    AroundCenter,
    /// Pixels move toward and away from the center.
    OutFromCenter,
    /// Both, like rings on water.
    PondRipples,
}

/// What Tiles fills the gaps between tiles with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilesFill {
    Background,
    Foreground,
    Inverse,
    Unaltered,
}

/// Ripple's Size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RippleSize {
    Small,
    Medium,
    Large,
}

impl RippleSize {
    /// The waves' length in pixels.
    fn wavelength(self) -> f32 {
        match self {
            Self::Small => 6.0,
            Self::Medium => 12.0,
            Self::Large => 24.0,
        }
    }
}

/// Mezzotint's Type, in Photoshop's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MezzotintType {
    FineDots,
    MediumDots,
    GrainyDots,
    CoarseDots,
    ShortLines,
    MediumLines,
    LongLines,
    ShortStrokes,
    MediumStrokes,
    LongStrokes,
}

impl MezzotintType {
    pub const ALL: [Self; 10] = [
        Self::FineDots,
        Self::MediumDots,
        Self::GrainyDots,
        Self::CoarseDots,
        Self::ShortLines,
        Self::MediumLines,
        Self::LongLines,
        Self::ShortStrokes,
        Self::MediumStrokes,
        Self::LongStrokes,
    ];
}

/// Diffuse's Mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffuseMode {
    /// Any neighbor.
    Normal,
    /// Only a darker neighbor replaces a pixel.
    DarkenOnly,
    /// Only a lighter one.
    LightenOnly,
    /// The neighbor closest in color, so edges stay soft-edged.
    Anisotropic,
}

/// Wind's Method.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindMethod {
    Wind,
    Blast,
    Stagger,
}

/// Pinch's radial shift at 100% for distances 0, 0.05, … 1 of the
/// radius, measured from Photoshop 2026 (it scales with the amount).
const PINCH_SHIFT: [f32; 21] = [
    0.0, 0.0422, 0.0796, 0.1133, 0.1447, 0.1727, 0.1943, 0.2111, 0.2211, 0.2248, 0.2204, 0.2101,
    0.1936, 0.1707, 0.1436, 0.1135, 0.0828, 0.0529, 0.0269, 0.0077, 0.0,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Filter {
    /// Blur > Gaussian Blur; the radius is the standard deviation in pixels.
    GaussianBlur { radius: f32 },
    /// Blur > Box Blur: the average of a (2r + 1)² square.
    BoxBlur { radius: u32 },
    /// Blur > Average: the selection filled with its average color.
    Average,
    /// Sharpen > Unsharp Mask: amount in percent, Gaussian radius, and the
    /// threshold (0–255) a difference must exceed to be sharpened.
    UnsharpMask {
        amount: f32,
        radius: f32,
        threshold: u8,
    },
    /// Noise > Add Noise: amount in percent; uniform or Gaussian; the same
    /// noise on all channels when monochromatic.
    AddNoise {
        amount: f32,
        gaussian: bool,
        monochromatic: bool,
    },
    /// Noise > Median over a (2r + 1)² square, per channel.
    Median { radius: u32 },
    /// Other > Minimum: each channel's smallest value within `radius`
    /// (Preserve: Squareness, or Roundness when `round`).
    Minimum { radius: f32, round: bool },
    /// Other > Maximum: the largest value.
    Maximum { radius: f32, round: bool },
    /// Blur > Blur: a light 3 × 3 blur.
    Blur,
    /// Blur > Blur More: a stronger 3 × 3 blur.
    BlurMore,
    /// Sharpen > Sharpen.
    Sharpen,
    /// Sharpen > Sharpen More.
    SharpenMore,
    /// Stylize > Find Edges: white where flat, darker the steeper the edge.
    FindEdges,
    /// Blur > Motion Blur: `distance` pixels along `angle` degrees.
    MotionBlur { angle: i32, distance: u32 },
    /// Stylize > Emboss: angle in degrees, height in pixels (1–100), amount
    /// in percent (1–500).
    Emboss {
        angle: i32,
        height: u32,
        amount: u32,
    },
    /// Pixelate > Fragment: four copies offset 4 pixels diagonally,
    /// averaged.
    Fragment,
    /// Other > Custom: a 5 × 5 kernel (rows top to bottom, −999–999),
    /// the sum divided by `scale` plus `offset`.
    Custom {
        kernel: [i16; 25],
        scale: i16,
        offset: i16,
    },
    /// Blur > Surface Blur: radius 1–100, threshold 2–255.
    SurfaceBlur { radius: u32, threshold: u8 },
    /// Noise > Dust & Scratches: radius 1–500, threshold 0–255.
    DustAndScratches { radius: u32, threshold: u8 },
    /// Noise > Despeckle: Blur More, held back where the image has edges.
    Despeckle,
    /// Sharpen > Sharpen Edges: Sharpen, only where the image has edges.
    SharpenEdges,
    /// Stylize > Trace Contour: per channel, white with black lines along
    /// where values cross `level`, on its lower side (`upper`) or its
    /// upper side.
    TraceContour { level: u8, upper: bool },
    /// Stylize > Wind: streaks blown from the left (`from_left`) or the
    /// right. Random, like Photoshop's, but repeatable.
    Wind { method: WindMethod, from_left: bool },
    /// Distort > Twirl: degrees (−999–999) at the center, fading out.
    Twirl { angle: i32 },
    /// Distort > Pinch: −100–100 percent (positive pinches in).
    Pinch { amount: i32 },
    /// Distort > Spherize: −100–100 percent, in `mode`.
    Spherize { amount: i32, mode: SpherizeMode },
    /// Distort > Polar Coordinates: Rectangular to Polar (`to_polar`) or
    /// Polar to Rectangular.
    PolarCoordinates { to_polar: bool },
    /// Other > High Pass: the image minus its Gaussian blur, around gray.
    HighPass { radius: f32 },
    /// Other > Offset: shifts the layer by (dx, dy) pixels.
    Offset { dx: i32, dy: i32, fill: OffsetFill },
    /// Pixelate > Mosaic: squares of `cell` pixels filled with their average.
    Mosaic { cell: u32 },
    /// Stylize > Solarize: values above 127 are inverted.
    Solarize,
    /// Pixelate > Crystallize: Voronoi cells around one randomly placed
    /// point per `cell` × `cell` block, each filled with its average color.
    Crystallize { cell: u32 },
    /// Pixelate > Pointillize: dots about `cell` pixels across in the
    /// colors under them (a little varied), on the background color.
    Pointillize { cell: u32 },
    /// Distort > Ripple: waves along both axes, `amount` −999–999 percent,
    /// their length set by `size`.
    Ripple { amount: i32, size: RippleSize },
    /// Pixelate > Mezzotint: each channel turned to full or nothing at
    /// random, in dots, lines or strokes.
    Mezzotint { kind: MezzotintType },
    /// Stylize > Tiles: the layer cut into square tiles, at least `count`
    /// across its shorter side, each shifted at random up to `offset`
    /// percent of a tile, the gaps filled as `fill` says.
    Tiles {
        count: u32,
        offset: u32,
        fill: TilesFill,
        foreground: [u8; 3],
    },
    /// Pixelate > Color Halftone: each of the first three channels as dots
    /// on a screen at its angle (degrees), up to `radius` pixels.
    ColorHalftone { radius: u32, angles: [i32; 4] },
    /// Distort > ZigZag: ripples out from the center, `amount` −100–100,
    /// `ridges` 0–20 across the radius, in `style`.
    ZigZag {
        amount: i32,
        ridges: u32,
        style: ZigZagStyle,
    },
    /// Other > HSB/HSL: the channels read as `input` and written as
    /// `output` (hue as 0–255 for 0–360°).
    HsbHsl {
        input: ColorModel,
        output: ColorModel,
    },
    /// Pixelate > Facet: similar neighboring colors clumped into flat
    /// patches.
    Facet,
    /// Stylize > Diffuse: pixels swapped with random neighbors, in `mode`.
    Diffuse { mode: DiffuseMode },
    /// Render > Clouds: soft fractal noise between the foreground and
    /// background colors, filling the layer opaquely; a new pattern each
    /// time (`seed`).
    Clouds {
        foreground: [u8; 3],
        background: [u8; 3],
        seed: u32,
    },
    /// Render > Difference Clouds: the same clouds in Difference mode over
    /// the layer.
    DifferenceClouds {
        foreground: [u8; 3],
        background: [u8; 3],
        seed: u32,
    },
    /// Image > Adjustments > Shadows/Highlights (it looks at each pixel's
    /// surroundings, so it runs as a filter; see `tone.rs`).
    ShadowsHighlights(crate::tone::ShadowsHighlights),
    /// Image > Adjustments > HDR Toning (see `tone.rs`).
    HdrToning(crate::tone::HdrToning),
    /// Distort > Wave (see `more_filters.rs` for these and the others
    /// below).
    Wave(crate::more_filters::Wave),
    /// Distort > Shear: the curve's points (height, offset), the first
    /// `count` used.
    Shear {
        points: [(f32, f32); 8],
        count: u8,
        undefined: crate::more_filters::Undefined,
    },
    /// Distort > Displace with the map registered as `map`.
    Displace {
        map: u32,
        scale: (f32, f32),
        stretch: bool,
        undefined: crate::more_filters::Undefined,
    },
    RadialBlur {
        amount: f32,
        method: crate::more_filters::RadialMethod,
        quality: u8,
        center: (f32, f32),
    },
    SmartBlur {
        radius: f32,
        threshold: f32,
        mode: crate::more_filters::SmartBlurMode,
    },
    ShapeBlur {
        radius: f32,
        shape: crate::more_filters::BlurShape,
    },
    LensBlur(crate::more_filters::LensBlur),
    ReduceNoise {
        strength: f32,
        preserve: f32,
        color: f32,
        sharpen: f32,
    },
    SmartSharpen {
        amount: f32,
        radius: f32,
        noise: f32,
        remove: crate::more_filters::SharpenRemove,
        angle: f32,
        fade: (f32, f32),
    },
    Fibers {
        variance: f32,
        strength: f32,
        foreground: [u8; 3],
        background: [u8; 3],
        seed: u32,
    },
    LensFlare {
        center: (f32, f32),
        brightness: f32,
        lens: crate::more_filters::LensType,
    },
    Extrude {
        kind: crate::more_filters::ExtrudeType,
        size: u32,
        depth: f32,
        level_based: bool,
        solid: bool,
        mask_incomplete: bool,
        seed: u32,
    },
    OilPaint {
        stylization: f32,
        cleanliness: f32,
        scale: f32,
        angle: f32,
        shine: f32,
    },
}

impl Filter {
    /// The menu and history name.
    pub fn name(self) -> &'static str {
        match self {
            Self::ShadowsHighlights(_) => "Shadows/Highlights",
            Self::Wave(_) => "Wave",
            Self::Shear { .. } => "Shear",
            Self::Displace { .. } => "Displace",
            Self::RadialBlur { .. } => "Radial Blur",
            Self::SmartBlur { .. } => "Smart Blur",
            Self::ShapeBlur { .. } => "Shape Blur",
            Self::LensBlur(_) => "Lens Blur",
            Self::ReduceNoise { .. } => "Reduce Noise",
            Self::SmartSharpen { .. } => "Smart Sharpen",
            Self::Fibers { .. } => "Fibers",
            Self::LensFlare { .. } => "Lens Flare",
            Self::Extrude { .. } => "Extrude",
            Self::OilPaint { .. } => "Oil Paint",
            Self::HdrToning(_) => "HDR Toning",
            Self::GaussianBlur { .. } => "Gaussian Blur",
            Self::BoxBlur { .. } => "Box Blur",
            Self::Average => "Average",
            Self::UnsharpMask { .. } => "Unsharp Mask",
            Self::AddNoise { .. } => "Add Noise",
            Self::Median { .. } => "Median",
            Self::Minimum { .. } => "Minimum",
            Self::Maximum { .. } => "Maximum",
            Self::HighPass { .. } => "High Pass",
            Self::Offset { .. } => "Offset",
            Self::Mosaic { .. } => "Mosaic",
            Self::Solarize => "Solarize",
            Self::Blur => "Blur",
            Self::BlurMore => "Blur More",
            Self::Sharpen => "Sharpen",
            Self::SharpenMore => "Sharpen More",
            Self::FindEdges => "Find Edges",
            Self::MotionBlur { .. } => "Motion Blur",
            Self::Emboss { .. } => "Emboss",
            Self::Twirl { .. } => "Twirl",
            Self::Fragment => "Fragment",
            Self::Custom { .. } => "Custom",
            Self::SurfaceBlur { .. } => "Surface Blur",
            Self::DustAndScratches { .. } => "Dust & Scratches",
            Self::Despeckle => "Despeckle",
            Self::SharpenEdges => "Sharpen Edges",
            Self::TraceContour { .. } => "Trace Contour",
            Self::Wind { .. } => "Wind",
            Self::Pinch { .. } => "Pinch",
            Self::Spherize { .. } => "Spherize",
            Self::PolarCoordinates { .. } => "Polar Coordinates",
            Self::Crystallize { .. } => "Crystallize",
            Self::Ripple { .. } => "Ripple",
            Self::Tiles { .. } => "Tiles",
            Self::Facet => "Facet",
            Self::ZigZag { .. } => "ZigZag",
            Self::HsbHsl { .. } => "HSB/HSL",
            Self::ColorHalftone { .. } => "Color Halftone",
            Self::Mezzotint { .. } => "Mezzotint",
            Self::Pointillize { .. } => "Pointillize",
            Self::Diffuse { .. } => "Diffuse",
            Self::Clouds { .. } => "Clouds",
            Self::DifferenceClouds { .. } => "Difference Clouds",
        }
    }
}

/// `a` mixed toward `b` by `t` (0–1).
fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [0, 1, 2].map(|c| (a[c] as f32 + (b[c] as f32 - a[c] as f32) * t).round() as u8)
}

/// Render > Clouds' pattern: fractal value noise, 0–1 per pixel (row by
/// row). Octaves from cells a quarter of the image's longer side (at least
/// 32 pixels) down to 2 pixels, each half as strong as the one before,
/// stretched to use the whole range.
pub fn clouds(w: usize, h: usize, seed: u32) -> Vec<f32> {
    let largest = (w.max(h) / 4).max(32).next_power_of_two() / 2;
    let cells: Vec<usize> =
        std::iter::successors(Some(largest.max(32)), |&c| (c > 2).then_some(c / 2)).collect();
    let hash = |octave: u32, x: usize, y: usize| {
        let mut v = seed
            ^ octave.wrapping_mul(0x9e37_79b9)
            ^ (x as u32).wrapping_mul(0x85eb_ca6b)
            ^ (y as u32).wrapping_mul(0xc2b2_ae35);
        v ^= v >> 16;
        v = v.wrapping_mul(0x7feb_352d);
        v ^= v >> 15;
        v = v.wrapping_mul(0x846c_a68b);
        v ^= v >> 16;
        v as f32 / u32::MAX as f32
    };
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let mut out = vec![0f32; w * h];
    for (octave, &cell) in cells.iter().enumerate() {
        let amp = 0.5f32.powi(octave as i32);
        for y in 0..h {
            let (gy, fy) = (y / cell, smooth((y % cell) as f32 / cell as f32));
            for x in 0..w {
                let (gx, fx) = (x / cell, smooth((x % cell) as f32 / cell as f32));
                let o = octave as u32;
                let v00 = hash(o, gx, gy);
                let v10 = hash(o, gx + 1, gy);
                let v01 = hash(o, gx, gy + 1);
                let v11 = hash(o, gx + 1, gy + 1);
                let top = v00 + (v10 - v00) * fx;
                let bottom = v01 + (v11 - v01) * fx;
                out[y * w + x] += amp * (top + (bottom - top) * fy);
            }
        }
    }
    let (lo, hi) = out
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let span = (hi - lo).max(1e-6);
    for v in &mut out {
        *v = (*v - lo) / span;
    }
    out
}

/// Photoshop's Gaussian Blur kernels for radii 0.1–2.0 pixels (the center
/// weight, then each side's, out of 256), measured from Photoshop 2026.
const SMALL_KERNELS: [&[u16]; 20] = [
    &[256],
    &[236, 10],
    &[182, 37],
    &[158, 49],
    &[144, 55, 1],
    &[134, 58, 3],
    &[122, 60, 7],
    &[112, 61, 11],
    &[102, 61, 15, 1],
    &[96, 60, 18, 2],
    &[88, 59, 21, 4],
    &[80, 58, 24, 5, 1],
    &[76, 56, 26, 7, 1],
    &[70, 54, 28, 9, 2],
    &[66, 52, 29, 11, 3],
    &[60, 51, 30, 12, 4, 1],
    &[60, 49, 30, 14, 4, 1],
    &[54, 47, 31, 15, 6, 2],
    &[54, 45, 31, 16, 7, 2],
    &[48, 44, 31, 17, 8, 3, 1],
];

/// Photoshop's kernels for radii 2.1–2.9 (center weight, then each side's),
/// measured from Photoshop 2026 at 16 bits; 2.2–2.4 and 2.5–2.6 share one.
const MID_KERNELS: [&[f32]; 9] = [
    &[
        0.175297, 0.160498, 0.123455, 0.076526, 0.037043, 0.012358, 0.002472,
    ],
    &[
        0.165436, 0.153079, 0.120156, 0.078996, 0.041984, 0.017300, 0.004943, 0.000824,
    ],
    &[
        0.165436, 0.153079, 0.120156, 0.078996, 0.041984, 0.017300, 0.004943, 0.000824,
    ],
    &[
        0.165436, 0.153079, 0.120156, 0.078996, 0.041984, 0.017300, 0.004943, 0.000824,
    ],
    &[
        0.148638, 0.139269, 0.114553, 0.081963, 0.050380, 0.025693, 0.010375, 0.002960, 0.000488,
    ],
    &[
        0.148638, 0.139269, 0.114553, 0.081963, 0.050380, 0.025693, 0.010375, 0.002960, 0.000488,
    ],
    &[
        0.141314, 0.133319, 0.111989, 0.082665, 0.053340, 0.029355, 0.013335, 0.004455, 0.000885,
    ],
    &[
        0.135978, 0.128868, 0.109338, 0.082667, 0.055111, 0.032011, 0.015716, 0.006225, 0.001770,
        0.000305,
    ],
    &[
        0.130676, 0.124268, 0.106689, 0.082672, 0.057068, 0.034668, 0.018127, 0.007996, 0.002655,
        0.000519,
    ],
];

/// A layer as premultiplied RGBA floats (0–255).
struct Buffer {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

impl Buffer {
    fn from_image(image: &TiledImage) -> Self {
        let raw = image.to_rgba8();
        let px = raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&[r, g, b, a]| {
                let k = a as f32 / 255.0;
                [r as f32 * k, g as f32 * k, b as f32 * k, a as f32]
            })
            .collect();
        Self {
            w: image.width() as usize,
            h: image.height() as usize,
            px,
        }
    }

    fn at(&self, x: isize, y: isize) -> [f32; 4] {
        let x = x.clamp(0, self.w as isize - 1) as usize;
        let y = y.clamp(0, self.h as isize - 1) as usize;
        self.px[y * self.w + x]
    }

    fn straight(p: [f32; 4]) -> [u8; 4] {
        let a = p[3].clamp(0.0, 255.0);
        if a <= 0.0 {
            return [0; 4];
        }
        let k = 255.0 / a;
        [
            (p[0] * k).round().clamp(0.0, 255.0) as u8,
            (p[1] * k).round().clamp(0.0, 255.0) as u8,
            (p[2] * k).round().clamp(0.0, 255.0) as u8,
            a.round() as u8,
        ]
    }

    /// Convolves rows, then columns, with a symmetric 1-D kernel.
    fn separable(&self, kernel: &[f32]) -> Self {
        let r = (kernel.len() / 2) as isize;
        let pass = |src: &Buffer, horizontal: bool| -> Buffer {
            let mut out = vec![[0f32; 4]; src.w * src.h];
            for y in 0..src.h as isize {
                for x in 0..src.w as isize {
                    let mut acc = [0f32; 4];
                    for (k, &wgt) in kernel.iter().enumerate() {
                        let d = k as isize - r;
                        let p = if horizontal {
                            src.at(x + d, y)
                        } else {
                            src.at(x, y + d)
                        };
                        for c in 0..4 {
                            acc[c] += p[c] * wgt;
                        }
                    }
                    out[y as usize * src.w + x as usize] = acc;
                }
            }
            Buffer {
                w: src.w,
                h: src.h,
                px: out,
            }
        };
        pass(&pass(self, true), false)
    }

    /// Photoshop's Gaussian Blur (measured from Photoshop 2026): radii up
    /// to 2 pixels (in Photoshop's 0.1 steps) use its own 8-bit kernels,
    /// 2.1–2.9 its measured kernels, larger ones five extended boxes whose
    /// variance is exactly radius² (within a hundredth of a level of
    /// Photoshop's at radius 8).
    fn gaussian(&self, sigma: f32) -> Self {
        let tenths = (sigma * 10.0).round() as usize;
        if tenths == 0 {
            return Self {
                w: self.w,
                h: self.h,
                px: self.px.clone(),
            };
        }
        if tenths <= SMALL_KERNELS.len() {
            let half = SMALL_KERNELS[tenths - 1];
            let kernel: Vec<f32> = half[1..]
                .iter()
                .rev()
                .chain(half)
                .map(|&w| w as f32 / 256.0)
                .collect();
            return self.separable(&kernel);
        }
        if tenths < 30 {
            let half = MID_KERNELS[tenths - 21];
            let kernel: Vec<f32> = half[1..].iter().rev().chain(half).copied().collect();
            return self.separable(&kernel);
        }
        // Five extended boxes (Gwosdek et al.), each of variance sigma² / 5:
        // radius l plus fractional end taps alpha, convolved into one
        // kernel that is applied once with the edges repeated (as
        // Photoshop does)
        let v = (sigma * sigma / 5.0) as f64;
        let l = ((0.5 * (12.0 * v + 1.0).sqrt() - 0.5).floor()).max(0.0);
        let alpha =
            (2.0 * l + 1.0) * (l * (l + 1.0) - 3.0 * v) / (6.0 * (v - (l + 1.0) * (l + 1.0)));
        let mut boxed = vec![1.0f64; 2 * l as usize + 3];
        let last = boxed.len() - 1;
        boxed[0] = alpha;
        boxed[last] = alpha;
        let total: f64 = boxed.iter().sum();
        boxed.iter_mut().for_each(|w| *w /= total);
        let mut kernel = vec![1.0f64];
        for _ in 0..5 {
            let mut next = vec![0.0; kernel.len() + boxed.len() - 1];
            for (i, a) in kernel.iter().enumerate() {
                for (j, b) in boxed.iter().enumerate() {
                    next[i + j] += a * b;
                }
            }
            kernel = next;
        }
        let kernel: Vec<f32> = kernel.into_iter().map(|w| w as f32).collect();
        self.separable(&kernel)
    }

    /// The sum of the pixels at `taps` (offset, weight) around each pixel,
    /// edges repeated.
    fn taps(&self, taps: &[((isize, isize), f32)]) -> Self {
        let mut out = vec![[0f32; 4]; self.w * self.h];
        for y in 0..self.h as isize {
            for x in 0..self.w as isize {
                let mut acc = [0f32; 4];
                for &((dx, dy), wgt) in taps {
                    let p = self.at(x + dx, y + dy);
                    for c in 0..4 {
                        acc[c] += p[c] * wgt;
                    }
                }
                out[y as usize * self.w + x as usize] = acc;
            }
        }
        Self {
            w: self.w,
            h: self.h,
            px: out,
        }
    }

    fn transposed(&self) -> Self {
        let mut px = vec![[0f32; 4]; self.w * self.h];
        for y in 0..self.h {
            for x in 0..self.w {
                px[x * self.h + y] = self.px[y * self.w + x];
            }
        }
        Self {
            w: self.h,
            h: self.w,
            px,
        }
    }

    /// Motion Blur as Photoshop computes it (fitted to its impulse
    /// responses, within a level at every angle and distance measured): the
    /// line is `m = ⌊d·cos a + 0.5⌋` pixels along and `n = ⌊d·sin a + 0.5⌋`
    /// across (truncated toward zero; the steeper axis is "along"). The
    /// image is sheared so the line lies along a row, each column moved by
    /// −(x + 0.5)·n/m with a = −0.75 bicubic interpolation; the rows are
    /// averaged over m + 1 pixels from −⌈m/2⌉; the result is sheared back.
    /// Each intermediate is clamped to 0–255.
    fn motion_blur(&self, angle: i32, distance: u32) -> Self {
        let a = (angle.clamp(-90, 90) as f64).to_radians();
        let d = distance as f64;
        let m = (d * a.cos() + 0.5) as i64;
        let n = (d * a.sin() + 0.5) as i64;
        if n.abs() > m.abs() {
            return self.transposed().motion_rows(n, m).transposed();
        }
        self.motion_rows(m, n)
    }

    /// Motion Blur along rows (`m` along, `n` across; see `motion_blur`).
    fn motion_rows(&self, m: i64, n: i64) -> Self {
        let (w, h) = (self.w, self.h);
        let taps = m.unsigned_abs() as usize + 1;
        let start = -((m.abs() + 1) / 2);
        // Averages a row of `w` pixels over the taps, edges repeated
        let average = |row: &[[f32; 4]]| -> Vec<[f32; 4]> {
            let lo = start;
            let hi = start + taps as i64 - 1;
            // prefix sums over x − o for o in lo..=hi
            let first = -hi;
            let len = w as i64 + hi - lo;
            let mut prefix = vec![[0f32; 4]; len as usize + 1];
            for i in 0..len {
                let p = row[(first + i).clamp(0, w as i64 - 1) as usize];
                for c in 0..4 {
                    prefix[i as usize + 1][c] = prefix[i as usize][c] + p[c];
                }
            }
            (0..w as i64)
                .map(|x| {
                    let (a, b) = ((x - hi - first) as usize, (x - lo - first) as usize + 1);
                    let mut out = [0f32; 4];
                    for c in 0..4 {
                        out[c] = ((prefix[b][c] - prefix[a][c]) / taps as f32).clamp(0.0, 255.0);
                    }
                    out
                })
                .collect()
        };
        if n == 0 {
            let mut px = Vec::with_capacity(w * h);
            for y in 0..h {
                px.extend(average(&self.px[y * w..(y + 1) * w]));
            }
            return Self { w, h, px };
        }
        let cubic = |t: f64| -> f32 {
            const A: f64 = -0.75;
            let t = t.abs();
            (if t < 1.0 {
                (A + 2.0) * t * t * t - (A + 3.0) * t * t + 1.0
            } else if t < 2.0 {
                A * t * t * t - 5.0 * A * t * t + 8.0 * A * t - 4.0 * A
            } else {
                0.0
            }) as f32
        };
        // Each column's shift: its integer part and four weights
        let slope = n as f64 / m as f64;
        let split = |s: f64| {
            let i0 = s.floor();
            let f = s - i0;
            (i0 as i64, [-1.0, 0.0, 1.0, 2.0].map(|j: f64| cubic(f - j)))
        };
        let forward: Vec<_> = (0..w).map(|x| split(-(x as f64 + 0.5) * slope)).collect();
        let back: Vec<_> = (0..w).map(|x| split((x as f64 + 0.5) * slope)).collect();
        // Sheared rows reach this far beyond the image
        let reach = (w as f64 * slope.abs()).ceil() as i64 + 4;
        let mut acc = vec![[0f32; 4]; w * h];
        let mut sheared = vec![[0f32; 4]; w];
        for r in -reach..h as i64 + reach {
            // Row r of the sheared image
            for (x, (i0, wts)) in forward.iter().enumerate() {
                let mut p = [0f32; 4];
                for (j, wt) in wts.iter().enumerate() {
                    let y = (r + i0 + j as i64 - 1).clamp(0, h as i64 - 1) as usize;
                    let q = self.px[y * w + x];
                    for c in 0..4 {
                        p[c] += wt * q[c];
                    }
                }
                sheared[x] = p.map(|v| v.clamp(0.0, 255.0));
            }
            let blurred = average(&sheared);
            // Sheared back: output (x, y) reads rows y + i0 + j − 1
            for (x, (i0, wts)) in back.iter().enumerate() {
                for (j, wt) in wts.iter().enumerate() {
                    let y = r - i0 - j as i64 + 1;
                    if (0..h as i64).contains(&y) {
                        let o = &mut acc[y as usize * w + x];
                        for c in 0..4 {
                            o[c] += wt * blurred[x][c];
                        }
                    }
                }
            }
        }
        for p in &mut acc {
            *p = p.map(|v| v.clamp(0.0, 255.0));
        }
        Self { w, h, px: acc }
    }

    /// A 3 × 3 kernel (rows top to bottom) as taps.
    fn kernel3(&self, k: [[f32; 3]; 3]) -> Self {
        let mut taps = Vec::new();
        for (dy, row) in k.iter().enumerate() {
            for (dx, &w) in row.iter().enumerate() {
                if w != 0.0 {
                    taps.push(((dx as isize - 1, dy as isize - 1), w));
                }
            }
        }
        self.taps(&taps)
    }

    /// The bilinearly interpolated pixel at (`x`, `y`), edges repeated.
    fn sample(&self, x: f32, y: f32) -> [f32; 4] {
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (x0, y0) = (x0 as isize, y0 as isize);
        let mut acc = [0f32; 4];
        for (dx, dy, w) in [
            (0, 0, (1.0 - fx) * (1.0 - fy)),
            (1, 0, fx * (1.0 - fy)),
            (0, 1, (1.0 - fx) * fy),
            (1, 1, fx * fy),
        ] {
            let p = self.at(x0 + dx, y0 + dy);
            for c in 0..4 {
                acc[c] += p[c] * w;
            }
        }
        acc
    }

    /// Minimum or Maximum (`max`) within `radius`: a square of the radius
    /// rounded half up, or a disc whose edge pixels count by how much of
    /// them it covers (1 − (distance − radius), measured from Photoshop).
    fn extreme(&self, radius: f32, round: bool, max: bool) -> Self {
        let reach = radius.ceil() as isize + 1;
        let mut window = Vec::new();
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let cover = if round {
                    (1.0 - (((dx * dx + dy * dy) as f32).sqrt() - radius)).clamp(0.0, 1.0)
                } else {
                    let r = (radius + 0.5).floor() as isize;
                    if dx.abs() <= r && dy.abs() <= r {
                        1.0
                    } else {
                        0.0
                    }
                };
                if cover > 0.0 {
                    window.push(((dx, dy), cover));
                }
            }
        }
        let mut out = vec![[0f32; 4]; self.w * self.h];
        for y in 0..self.h as isize {
            for x in 0..self.w as isize {
                let here = self.at(x, y);
                let mut best = here;
                for &((dx, dy), cover) in &window {
                    let p = self.at(x + dx, y + dy);
                    for c in 0..4 {
                        let v = here[c] + (p[c] - here[c]) * cover;
                        best[c] = if max { best[c].max(v) } else { best[c].min(v) };
                    }
                }
                out[y as usize * self.w + x as usize] = best;
            }
        }
        Self {
            w: self.w,
            h: self.h,
            px: out,
        }
    }

    /// Applies `f` to each channel's values in the (2r + 1)² square.
    fn rank(&self, radius: u32, f: impl Fn(&mut Vec<f32>) -> f32) -> Self {
        let r = radius as isize;
        let mut out = vec![[0f32; 4]; self.w * self.h];
        let mut values = Vec::with_capacity(((2 * r + 1) * (2 * r + 1)) as usize);
        for y in 0..self.h as isize {
            for x in 0..self.w as isize {
                let mut p = [0f32; 4];
                for (c, v) in p.iter_mut().enumerate() {
                    values.clear();
                    for dy in -r..=r {
                        for dx in -r..=r {
                            values.push(self.at(x + dx, y + dy)[c]);
                        }
                    }
                    *v = f(&mut values);
                }
                out[y as usize * self.w + x as usize] = p;
            }
        }
        Self {
            w: self.w,
            h: self.h,
            px: out,
        }
    }
}

/// Where Twirl, Pinch, Spherize or Polar Coordinates takes the color of
/// pixel (`x`, `y`) of a `w` × `h` image from (other filters: the pixel
/// itself). The filter dialogs draw their diagrams with it.
pub fn distortion_source(filter: Filter, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    distort_source(filter, x, y, w / 2.0, h / 2.0, w, h)
}

/// Where a distortion filter takes pixel (`x`, `y`)'s color from. Twirl,
/// Pinch and Spherize act inside the ellipse touching the image's edges
/// (distances measured as fractions of it); Polar Coordinates maps angle
/// around the center (from straight up, anticlockwise) to x and distance
/// to y.
fn distort_source(filter: Filter, x: f32, y: f32, cx: f32, cy: f32, w: f32, h: f32) -> (f32, f32) {
    let (dx, dy) = ((x - cx) / cx, (y - cy) / cy);
    let t = (dx * dx + dy * dy).sqrt();
    let radial = |s: f32| {
        if t <= 0.0 || t >= 1.0 {
            (x, y)
        } else {
            (cx + dx * s / t * cx, cy + dy * s / t * cy)
        }
    };
    // Spherize's map for one distance: arcsine out (positive) or sine in
    let sphere = |t: f32, amount: i32| -> f32 {
        let a = amount as f32 / 100.0;
        let curve = if a >= 0.0 {
            t.clamp(0.0, 1.0).asin() * 2.0 / std::f32::consts::PI
        } else {
            (t.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2).sin()
        };
        t + a.abs() * (curve - t)
    };
    match filter {
        // Waves: horizontal shifts that vary down the image and vertical
        // ones that vary across it
        Filter::Ripple { amount, size } => {
            let len = size.wavelength();
            let a = amount as f32 / 100.0 * len * 0.25;
            let k = std::f32::consts::TAU / len;
            (x + a * (y * k).sin(), y + a * (x * k + 1.0).sin())
        }
        // Waves along the radius, fading toward the edge of the ellipse
        Filter::ZigZag {
            amount,
            ridges,
            style,
        } => {
            if t >= 1.0 || t <= 0.0 {
                return (x, y);
            }
            // Waves fading out toward the edge; displacements also shrink
            // toward the center (compared with Photoshop's diagram)
            let a = amount as f32 / 100.0;
            let wave = (t * ridges as f32 * std::f32::consts::TAU).sin() * (1.0 - t) * 2.0 * a;
            let (turn, push) = match style {
                ZigZagStyle::AroundCenter => (wave, 0.0),
                ZigZagStyle::OutFromCenter => (0.0, wave * t),
                ZigZagStyle::PondRipples => (wave * 0.5, wave * t * 0.5),
            };
            let s = (t + push) / t;
            let (sin, cos) = turn.sin_cos();
            (
                cx + (dx * cos - dy * sin) * s * cx,
                cy + (dx * sin + dy * cos) * s * cy,
            )
        }
        Filter::Twirl { angle } => {
            if t >= 1.0 {
                return (x, y);
            }
            // The turn fades with the square of the distance to the edge
            let turn = -(angle as f32).to_radians() * (1.0 - t) * (1.0 - t);
            let (sin, cos) = turn.sin_cos();
            (
                cx + (dx * cos - dy * sin) * cx,
                cy + (dx * sin + dy * cos) * cy,
            )
        }
        Filter::Pinch { amount } => {
            let f = t * 20.0;
            let i = (f as usize).min(19);
            let shift = PINCH_SHIFT[i] + (PINCH_SHIFT[i + 1] - PINCH_SHIFT[i]) * (f - i as f32);
            radial(t + amount as f32 / 100.0 * shift)
        }
        Filter::Spherize { amount, mode } => match mode {
            SpherizeMode::Normal => radial(sphere(t, amount)),
            SpherizeMode::HorizontalOnly => {
                let s = if dx.abs() < 1.0 {
                    sphere(dx.abs(), amount)
                } else {
                    dx.abs()
                };
                (cx + dx.signum() * s * cx, y)
            }
            SpherizeMode::VerticalOnly => {
                let s = if dy.abs() < 1.0 {
                    sphere(dy.abs(), amount)
                } else {
                    dy.abs()
                };
                (x, cy + dy.signum() * s * cy)
            }
        },
        Filter::PolarCoordinates { to_polar: true } => {
            let (ox, oy) = (x - cx, y - cy);
            let angle = (-ox).atan2(-oy).rem_euclid(std::f32::consts::TAU);
            let r = ((ox / cx).powi(2) + (oy / cy).powi(2)).sqrt();
            (angle / std::f32::consts::TAU * w, r * h)
        }
        Filter::PolarCoordinates { to_polar: false } => {
            let angle = (x + 1.0) / w * std::f32::consts::TAU;
            let r = y / h;
            (cx - r * angle.sin() * cx, cy - r * angle.cos() * cy)
        }
        _ => (x, y),
    }
}

/// Crystallize: each `cell` × `cell` block holds one seed point at a
/// random spot; every pixel joins its nearest seed (searching the blocks
/// around it), and each cell is filled with its pixels' average color.
fn crystallize(src: &Buffer, cell: usize) -> Vec<[u8; 4]> {
    let (w, h) = (src.w, src.h);
    let (bw, bh) = (w.div_ceil(cell), h.div_ceil(cell));
    let seed = |bx: usize, by: usize| {
        (
            (bx * cell) as f32 + noise(bx, by, 21) * cell as f32,
            (by * cell) as f32 + noise(bx, by, 22) * cell as f32,
        )
    };
    let mut owner = vec![0usize; w * h];
    for y in 0..h {
        for x in 0..w {
            let (bx, by) = (x / cell, y / cell);
            let mut best = (f32::MAX, 0usize);
            for ny in by.saturating_sub(1)..(by + 2).min(bh) {
                for nx in bx.saturating_sub(1)..(bx + 2).min(bw) {
                    let (sx, sy) = seed(nx, ny);
                    let (dx, dy) = (x as f32 + 0.5 - sx, y as f32 + 0.5 - sy);
                    let d = dx * dx + dy * dy;
                    if d < best.0 {
                        best = (d, ny * bw + nx);
                    }
                }
            }
            owner[y * w + x] = best.1;
        }
    }
    let mut sum = vec![[0f32; 5]; bw * bh];
    for (i, &o) in owner.iter().enumerate() {
        let p = src.px[i];
        for c in 0..4 {
            sum[o][c] += p[c];
        }
        sum[o][4] += 1.0;
    }
    owner
        .iter()
        .map(|&o| {
            let s = sum[o];
            Buffer::straight([s[0] / s[4], s[1] / s[4], s[2] / s[4], s[3] / s[4]])
        })
        .collect()
}

/// Pointillize: the background color, with dots `cell` pixels across
/// placed at a random spot in each block of a grid `cell` × 0.75 apart,
/// each colored by the image under its center, varied a little per
/// channel; later dots cover earlier ones.
fn pointillize(src: &Buffer, cell: usize, paper: [u8; 3]) -> Vec<[u8; 4]> {
    let (w, h) = (src.w, src.h);
    let mut out = vec![[paper[0], paper[1], paper[2], 255]; w * h];
    let step = ((cell as f32 * 0.75).round() as usize).max(2);
    let r = cell as f32 / 2.0;
    for by in 0..h.div_ceil(step) {
        for bx in 0..w.div_ceil(step) {
            let cx = (bx * step) as f32 + noise(bx, by, 31) * step as f32;
            let cy = (by * step) as f32 + noise(bx, by, 32) * step as f32;
            let base = Buffer::straight(src.at(cx as isize, cy as isize));
            let color = [0, 1, 2].map(|c| {
                let jitter = (noise(bx, by, 33 + c) - 0.5) * 48.0;
                (base[c] as f32 + jitter).round().clamp(0.0, 255.0) as u8
            });
            let (x0, x1) = (
                (cx - r).floor().max(0.0) as usize,
                ((cx + r).ceil() as usize).min(w),
            );
            let (y0, y1) = (
                (cy - r).floor().max(0.0) as usize,
                ((cy + r).ceil() as usize).min(h),
            );
            for y in y0..y1 {
                for x in x0..x1 {
                    let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                    if dx * dx + dy * dy <= r * r {
                        out[y * w + x] = [color[0], color[1], color[2], 255];
                    }
                }
            }
        }
    }
    out
}

/// Tiles: tiles `min(w, h) / count` pixels square, each moved by a
/// repeatable random offset up to `offset` percent of its size, drawn over
/// the gaps' fill (the background or foreground color, the inverted
/// image, or the image as it was); tiles are drawn in order, later ones
/// over earlier.
fn tiles(
    src: &Buffer,
    (count, offset): (u32, u32),
    fill: TilesFill,
    (foreground, background): ([u8; 3], [u8; 3]),
) -> Vec<[u8; 4]> {
    let (w, h) = (src.w, src.h);
    let size = (w.min(h) / count as usize).max(1);
    let reach = (size as f32 * offset as f32 / 100.0).round() as i64;
    let mut out: Vec<[u8; 4]> = (0..w * h)
        .map(|i| {
            let p = Buffer::straight(src.px[i]);
            match fill {
                TilesFill::Background => [background[0], background[1], background[2], 255],
                TilesFill::Foreground => [foreground[0], foreground[1], foreground[2], 255],
                TilesFill::Inverse => [255 - p[0], 255 - p[1], 255 - p[2], p[3]],
                TilesFill::Unaltered => p,
            }
        })
        .collect();
    for ty in 0..h.div_ceil(size) {
        for tx in 0..w.div_ceil(size) {
            let shift = |k: usize| {
                if reach == 0 {
                    0
                } else {
                    (noise(tx, ty, k) * (2 * reach + 1) as f32) as i64 - reach
                }
            };
            let (dx, dy) = (shift(70), shift(71));
            for y in ty * size..((ty + 1) * size).min(h) {
                for x in tx * size..((tx + 1) * size).min(w) {
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                        out[ny as usize * w + nx as usize] = Buffer::straight(src.px[y * w + x]);
                    }
                }
            }
        }
    }
    out
}

/// Color Halftone: for each of the first three channels, a screen of
/// cells `2 × radius` pixels turned by the channel's angle; each cell
/// holds a dot whose area follows the channel's value at the cell's
/// center (full on at 255, none at 0), anti-aliased over a pixel. The
/// fourth angle is for CMYK images and doesn't apply to RGB.
fn color_halftone(src: &Buffer, radius: u32, angles: [i32; 4]) -> Vec<[u8; 4]> {
    let (w, h) = (src.w, src.h);
    let cell = 2.0 * radius as f32;
    let mut out: Vec<[u8; 4]> = src.px.iter().map(|&p| Buffer::straight(p)).collect();
    for (c, &angle) in angles.iter().take(3).enumerate() {
        let (sin, cos) = (angle as f32).to_radians().sin_cos();
        for y in 0..h {
            for x in 0..w {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                // Into the screen's turned frame, to the cell's center
                let (u, v) = (px * cos + py * sin, -px * sin + py * cos);
                let (cu, cv) = (
                    (u / cell).floor() * cell + cell / 2.0,
                    (v / cell).floor() * cell + cell / 2.0,
                );
                let (sx, sy) = (cu * cos - cv * sin, cu * sin + cv * cos);
                let sample = Buffer::straight(src.at(sx as isize, sy as isize));
                let value = sample[c] as f32 / 255.0;
                // A dot of this area within the cell (√2 reaches its corners)
                let r = cell / 2.0 * std::f32::consts::SQRT_2 * value.sqrt();
                let d = ((u - cu).powi(2) + (v - cv).powi(2)).sqrt();
                let coverage = (r - d + 0.5).clamp(0.0, 1.0);
                out[y * w + x][c] = (coverage * 255.0).round() as u8;
            }
        }
    }
    out
}

/// Facet (a Kuwahara filter): each pixel takes the average of whichever
/// of the four 3 × 3 squares it is a corner of has the least spread of
/// brightness, so flat areas clump into patches and edges stay sharp.
fn facet(src: &Buffer) -> Vec<[u8; 4]> {
    let (w, h) = (src.w, src.h);
    let luma = |p: [f32; 4]| p[0] * 0.299 + p[1] * 0.587 + p[2] * 0.114;
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as isize, (i / w) as isize);
            let mut best = (f32::MAX, [0f32; 4]);
            for (sx, sy) in [(-2, -2), (0, -2), (-2, 0), (0, 0)] {
                let mut sum = [0f32; 4];
                let (mut l, mut l2) = (0f32, 0f32);
                for dy in 0..3 {
                    for dx in 0..3 {
                        let p = src.at(x + sx + dx, y + sy + dy);
                        for c in 0..4 {
                            sum[c] += p[c] / 9.0;
                        }
                        let v = luma(p);
                        l += v / 9.0;
                        l2 += v * v / 9.0;
                    }
                }
                let spread = l2 - l * l;
                if spread < best.0 {
                    best = (spread, sum);
                }
            }
            Buffer::straight(best.1)
        })
        .collect()
}

/// Three channels read in `model`, as RGB (hue 0–255 standing for
/// 0–360°).
fn to_rgb(model: ColorModel, c: [u8; 3]) -> [u8; 3] {
    let (h, s, v) = (
        c[0] as f32 / 255.0 * 6.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
    );
    let hue = |h: f32, chroma: f32, m: f32| {
        let x = chroma * (1.0 - ((h % 2.0) - 1.0).abs());
        let (r, g, b) = match h as u32 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        [r + m, g + m, b + m].map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8)
    };
    let h = h.min(5.9999);
    match model {
        ColorModel::Rgb => c,
        ColorModel::Hsb => {
            let chroma = v * s;
            hue(h, chroma, v - chroma)
        }
        ColorModel::Hsl => {
            let chroma = (1.0 - (2.0 * v - 1.0).abs()) * s;
            hue(h, chroma, v - chroma / 2.0)
        }
    }
}

/// RGB written as three channels in `model`.
fn from_rgb(model: ColorModel, c: [u8; 3]) -> [u8; 3] {
    let [r, g, b] = c.map(|v| v as f32 / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    let q = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    match model {
        ColorModel::Rgb => c,
        ColorModel::Hsb => [q(h), q(if max == 0.0 { 0.0 } else { d / max }), q(max)],
        ColorModel::Hsl => {
            let l = (max + min) / 2.0;
            let s = if d == 0.0 {
                0.0
            } else {
                d / (1.0 - (2.0 * l - 1.0).abs())
            };
            [q(h), q(s), q(l)]
        }
    }
}

/// Mezzotint: each channel becomes 255 or 0, at random with its value as
/// the chance. Dots draw a fresh chance per pixel (Medium and Coarse per 2
/// and 3 pixel blocks, Grainy with half the chances shared by a 2 pixel
/// grain); Lines share a chance along horizontal runs of 4, 8 or 16 pixels,
/// Strokes along diagonal runs of 4, 8 or 16.
fn mezzotint(src: &Buffer, kind: MezzotintType) -> Vec<[u8; 4]> {
    use MezzotintType::*;
    let (w, h) = (src.w, src.h);
    let chance = |x: usize, y: usize, c: usize| -> f32 {
        match kind {
            FineDots => noise(x, y, 50 + c),
            MediumDots => noise(x / 2, y / 2, 50 + c),
            CoarseDots => noise(x / 3, y / 3, 50 + c),
            GrainyDots => (noise(x, y, 50 + c) + noise(x / 2, y / 2, 60 + c)) / 2.0,
            ShortLines | MediumLines | LongLines => {
                let run = match kind {
                    ShortLines => 4,
                    MediumLines => 8,
                    _ => 16,
                };
                noise(x / run, y, 50 + c)
            }
            ShortStrokes | MediumStrokes | LongStrokes => {
                let run = match kind {
                    ShortStrokes => 4,
                    MediumStrokes => 8,
                    _ => 16,
                };
                // Along the diagonal: x − y is constant
                let d = x + h - y;
                noise(d, (x + y) / run, 50 + c)
            }
        }
    };
    (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let p = Buffer::straight(src.px[i]);
            let mut out = p;
            for c in 0..3 {
                out[c] = if chance(x, y, c) < p[c] as f32 / 255.0 {
                    255
                } else {
                    0
                };
            }
            out
        })
        .collect()
}

/// Diffuse: each pixel takes a randomly chosen pixel of its 3 × 3
/// neighborhood (Normal), only if darker or lighter (Darken / Lighten
/// Only), or the neighbor closest in color among a random few
/// (Anisotropic).
fn diffuse(src: &Buffer, mode: DiffuseMode) -> Vec<[u8; 4]> {
    let (w, h) = (src.w, src.h);
    let luma = |p: [f32; 4]| p[0] * 0.299 + p[1] * 0.587 + p[2] * 0.114;
    let pick = |x: usize, y: usize, k: usize| {
        let n = (noise(x, y, 41 + k) * 9.0) as isize;
        src.at(x as isize + n % 3 - 1, y as isize + n / 3 - 1)
    };
    (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let here = src.px[i];
            let p = match mode {
                DiffuseMode::Normal => pick(x, y, 0),
                DiffuseMode::DarkenOnly => {
                    let n = pick(x, y, 0);
                    if luma(n) < luma(here) { n } else { here }
                }
                DiffuseMode::LightenOnly => {
                    let n = pick(x, y, 0);
                    if luma(n) > luma(here) { n } else { here }
                }
                DiffuseMode::Anisotropic => (0..3)
                    .map(|k| pick(x, y, k))
                    .min_by(|a, b| {
                        let d = |p: [f32; 4]| (0..4).map(|c| (p[c] - here[c]).abs()).sum::<f32>();
                        d(*a).total_cmp(&d(*b))
                    })
                    .unwrap_or(here),
            };
            Buffer::straight(p)
        })
        .collect()
}

/// A repeatable per-pixel random value in 0..1.
fn noise(x: usize, y: usize, channel: usize) -> f32 {
    let mut v = (x as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
        .wrapping_add((channel as u64).wrapping_mul(0x1656_67B1_9E37_79F9));
    v ^= v >> 33;
    v = v.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    v ^= v >> 33;
    (v >> 40) as f32 / (1u64 << 24) as f32
}

/// The filtered layer as straight RGBA8, computed from the whole layer.
fn filtered(
    image: &TiledImage,
    filter: Filter,
    selection: Option<&crate::selection::Selection>,
    background: Option<[u8; 3]>,
    paper: [u8; 3],
    mask: Option<&[u8]>,
) -> Vec<[u8; 4]> {
    let src = Buffer::from_image(image);
    let (w, h) = (src.w, src.h);
    let per_pixel = |f: &dyn Fn(usize, usize, [u8; 4]) -> [u8; 4]| -> Vec<[u8; 4]> {
        (0..w * h)
            .map(|i| f(i % w, i / w, Buffer::straight(src.px[i])))
            .collect()
    };
    match filter {
        Filter::GaussianBlur { radius } => src
            .gaussian(radius)
            .px
            .into_iter()
            .map(Buffer::straight)
            .collect(),
        Filter::BoxBlur { radius } => {
            let n = 2 * radius as usize + 1;
            src.separable(&vec![1.0 / n as f32; n])
                .px
                .into_iter()
                .map(Buffer::straight)
                .collect()
        }
        Filter::Average => {
            // Premultiplied mean of the selected (or all) pixels
            let mut sum = [0f64; 4];
            let mut weight = 0f64;
            for (i, p) in src.px.iter().enumerate() {
                let m = selection.map_or(1.0, |s| {
                    s.get((i % w) as u32, (i / w) as u32) as f64 / 255.0
                });
                for c in 0..4 {
                    sum[c] += p[c] as f64 * m;
                }
                weight += m;
            }
            let mean = sum.map(|v| (v / weight.max(1e-9)) as f32);
            vec![Buffer::straight(mean); w * h]
        }
        Filter::UnsharpMask {
            amount,
            radius,
            threshold,
        } => {
            let blurred = src.gaussian(radius);
            (0..w * h)
                .map(|i| {
                    let orig = Buffer::straight(src.px[i]);
                    let blur = Buffer::straight(blurred.px[i]);
                    let mut out = orig;
                    // The sharpening, less the threshold (Photoshop's
                    // threshold eases in rather than cutting off)
                    for c in 0..3 {
                        let delta = (orig[c] as f32 - blur[c] as f32) * amount / 100.0;
                        let eased = delta.signum() * (delta.abs() - threshold as f32).max(0.0);
                        out[c] = (orig[c] as f32 + eased).round().clamp(0.0, 255.0) as u8;
                    }
                    out
                })
                .collect()
        }
        Filter::AddNoise {
            amount,
            gaussian,
            monochromatic,
        } => per_pixel(&|x, y, px| {
            let mut out = px;
            for c in 0..3 {
                let channel = if monochromatic { 0 } else { c };
                let u = noise(x, y, channel);
                // Measured from Photoshop: Gaussian noise (a sum of four
                // uniforms, bounded like Photoshop's at about 3.5σ) has a
                // standard deviation of amount% × 255; uniform noise spans
                // ±(amount% × 255 + 0.75)
                let spread = amount / 100.0 * 255.0;
                let n = if gaussian {
                    let s: f32 = (0..4).map(|k| noise(x, y, channel * 4 + k + 8)).sum();
                    (s - 2.0) * 3f32.sqrt() * spread
                } else {
                    (u * 2.0 - 1.0) * (spread + 0.75)
                };
                let v = px[c] as f32 + n;
                out[c] = v.round().clamp(0.0, 255.0) as u8;
            }
            out
        }),
        Filter::Median { radius } => src
            .rank(radius, |v| {
                let mid = v.len() / 2;
                *v.select_nth_unstable_by(mid, f32::total_cmp).1
            })
            .px
            .into_iter()
            .map(Buffer::straight)
            .collect(),
        Filter::Minimum { radius, round } => src
            .extreme(radius, round, false)
            .px
            .into_iter()
            .map(Buffer::straight)
            .collect(),
        Filter::Maximum { radius, round } => src
            .extreme(radius, round, true)
            .px
            .into_iter()
            .map(Buffer::straight)
            .collect(),
        // The 3 × 3 blurs and sharpens measured from Photoshop 2026
        Filter::Blur | Filter::BlurMore | Filter::Sharpen | Filter::SharpenMore => {
            let k =
                match filter {
                    Filter::Blur => [[0.0, 1.0, 0.0], [1.0, 4.0, 1.0], [0.0, 1.0, 0.0]]
                        .map(|r| r.map(|v| v / 8.0)),
                    Filter::BlurMore => [[1.0, 2.0, 1.0], [2.0, 2.0, 2.0], [1.0, 2.0, 1.0]]
                        .map(|r| r.map(|v| v / 14.0)),
                    Filter::Sharpen => [[0.0, -0.25, 0.0], [-0.25, 2.0, -0.25], [0.0, -0.25, 0.0]],
                    _ => [
                        [-0.25, -0.25, -0.25],
                        [-0.25, 3.0, -0.25],
                        [-0.25, -0.25, -0.25],
                    ],
                };
            src.kernel3(k)
                .px
                .into_iter()
                .map(Buffer::straight)
                .collect()
        }
        Filter::Despeckle | Filter::SharpenEdges => {
            // Blur More (Despeckle) or Sharpen (Sharpen Edges), clipped,
            // mixed in by how strong the Sobel gradient is in that channel:
            // Sharpen Edges takes (|g| − 64) / 192 of it, Despeckle the
            // rest (measured from Photoshop 2026)
            let despeckle = filter == Filter::Despeckle;
            let target = src.kernel3(if despeckle {
                [[1.0, 2.0, 1.0], [2.0, 2.0, 2.0], [1.0, 2.0, 1.0]].map(|r| r.map(|v| v / 14.0))
            } else {
                [[0.0, -0.25, 0.0], [-0.25, 2.0, -0.25], [0.0, -0.25, 0.0]]
            });
            let gx = src.kernel3([[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]]);
            let gy = src.kernel3([[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]]);
            (0..w * h)
                .map(|i| {
                    let orig = Buffer::straight(src.px[i]);
                    let a = src.px[i][3];
                    let mut out = orig;
                    if a <= 0.0 {
                        return out;
                    }
                    for (c, v) in out.iter_mut().take(3).enumerate() {
                        let k = 255.0 / a;
                        let (dx, dy) = (gx.px[i][c] * k, gy.px[i][c] * k);
                        let edge = (((dx * dx + dy * dy).sqrt() - 64.0) / 192.0).clamp(0.0, 1.0);
                        let mix = if despeckle { 1.0 - edge } else { edge };
                        let t = (target.px[i][c] * k).clamp(0.0, 255.0);
                        let o = *v as f32;
                        *v = (o + (t - o) * mix).round().clamp(0.0, 255.0) as u8;
                    }
                    out
                })
                .collect()
        }
        Filter::TraceContour { level, upper } => {
            // A pixel is on the line where it is at or below the level
            // (Upper; at or above for Lower) and a side neighbour is past it
            let level = level as i32;
            let past = |v: u8| {
                if upper {
                    v as i32 > level
                } else {
                    (v as i32) < level
                }
            };
            let straight: Vec<[u8; 4]> = src.px.iter().map(|&p| Buffer::straight(p)).collect();
            let at = |x: isize, y: isize| {
                straight
                    [y.clamp(0, h as isize - 1) as usize * w + x.clamp(0, w as isize - 1) as usize]
            };
            (0..w * h)
                .map(|i| {
                    let (x, y) = ((i % w) as isize, (i / w) as isize);
                    let mut out = straight[i];
                    for (c, v) in out.iter_mut().take(3).enumerate() {
                        let line = !past(*v)
                            && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                                .iter()
                                .any(|&(dx, dy)| past(at(x + dx, y + dy)[c]));
                        *v = if line { 0 } else { 255 };
                    }
                    out
                })
                .collect()
        }
        Filter::Wind { method, from_left } => {
            // Measured from Photoshop 2026 (`photoshop_filters.md`). Every
            // edge where the row gets darker downwind takes a random number
            // of passes (Poisson, mean 1.1: none a third of the time), most
            // from the edge, a few one to three pixels on. A Wind pass
            // averages each pixel with the one before it (keeping the
            // lighter) until nothing changes, so a streak halves with each
            // pixel and passes stack (½, ¾, ⅞ of the step); a Blast pass
            // carries the lighter pixel ten pixels on. Stagger moves each
            // row round by 0.636 of its longest run of equal pixels, give or
            // take 0.36·√run.
            let mut out: Vec<[u8; 4]> = src.px.iter().map(|&p| Buffer::straight(p)).collect();
            for y in 0..h {
                // The row in the wind's direction
                let index = |k: usize| y * w + if from_left { k } else { w - 1 - k };
                let mut row: Vec<[u8; 4]> = (0..w).map(|k| out[index(k)]).collect();
                if method == WindMethod::Stagger {
                    let mut longest = 0;
                    let mut run = 0;
                    for k in 0..w {
                        run = if k > 0 && row[k] == row[k - 1] {
                            run + 1
                        } else {
                            1
                        };
                        longest = longest.max(run);
                    }
                    let g = ((0..4).map(|c| noise(c, y, 31)).sum::<f32>() - 2.0) * 3f32.sqrt();
                    let shift = (0.636 * longest as f32 + 0.36 * (longest as f32).sqrt() * g)
                        .round()
                        .clamp(0.0, w as f32) as usize
                        % w;
                    row.rotate_right(shift);
                } else {
                    let lum = |p: [u8; 4]| p[0] as i32 * 3 + p[1] as i32 * 6 + p[2] as i32;
                    // How many passes start at a pixel: Poisson, mean 1.1 at
                    // the image's own edges and 0.4 elsewhere
                    let edge: Vec<bool> = (0..w)
                        .map(|k| k > 0 && lum(row[k - 1]) > lum(row[k]))
                        .collect();
                    let passes_at = |k: usize| {
                        let u = noise(k, y, 7);
                        let cdf: &[f32] = if edge[k] {
                            &[0.333, 0.699, 0.900, 0.974, 0.995]
                        } else {
                            &[0.670, 0.938, 0.992, 0.999]
                        };
                        cdf.iter().take_while(|&&c| u >= c).count()
                    };
                    if method == WindMethod::Blast {
                        for k in (1..w).filter(|&k| edge[k]) {
                            // Each pass carries on where the last one stopped
                            for pass in 0..passes_at(k) {
                                let from = (k + pass * 10).min(w);
                                for x in from..(from + 10).min(w) {
                                    let before = row[x - 1];
                                    for (v, b) in row[x].iter_mut().zip(before).take(3) {
                                        *v = (*v).max(b);
                                    }
                                }
                            }
                        }
                    } else {
                        // Passes start anywhere; one that changes nothing
                        // stops at once, so only edges and streaks take them
                        for k in 1..w {
                            for _ in 0..passes_at(k) {
                                for x in k..w {
                                    let mut changed = false;
                                    let before = row[x - 1];
                                    for (v, b) in row[x].iter_mut().zip(before).take(3) {
                                        let avg = ((b as u16 + *v as u16) / 2) as u8;
                                        if avg > *v {
                                            *v = avg;
                                            changed = true;
                                        }
                                    }
                                    if !changed {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                for (k, p) in row.into_iter().enumerate() {
                    out[index(k)] = p;
                }
            }
            out
        }
        Filter::FindEdges => {
            // 255 less the Sobel gradient's length, per channel
            let gx = src.kernel3([[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]]);
            let gy = src.kernel3([[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]]);
            (0..w * h)
                .map(|i| {
                    let orig = Buffer::straight(src.px[i]);
                    let mut out = orig;
                    for (c, v) in out.iter_mut().take(3).enumerate() {
                        let (a, b) = (gx.px[i][c], gy.px[i][c]);
                        *v = (255.0 - (a * a + b * b).sqrt()).round().clamp(0.0, 255.0) as u8;
                    }
                    out
                })
                .collect()
        }
        Filter::MotionBlur { angle, distance } => src
            .motion_blur(angle, distance)
            .px
            .into_iter()
            .map(Buffer::straight)
            .collect(),
        Filter::Fragment => {
            let taps: Vec<_> = [(4, 4), (-4, 4), (4, -4), (-4, -4)]
                .map(|o| (o, 0.25))
                .to_vec();
            src.taps(&taps)
                .px
                .into_iter()
                .map(Buffer::straight)
                .collect()
        }
        Filter::Custom {
            kernel,
            scale,
            offset,
        } => {
            let scale = if scale == 0 { 1.0 } else { scale as f32 };
            let mut taps = Vec::new();
            for (i, &k) in kernel.iter().enumerate() {
                if k != 0 {
                    taps.push((
                        ((i % 5) as isize - 2, (i / 5) as isize - 2),
                        k as f32 / scale,
                    ));
                }
            }
            let sum = src.taps(&taps);
            (0..w * h)
                .map(|i| {
                    let mut out = Buffer::straight(src.px[i]);
                    let a = src.px[i][3];
                    if a > 0.0 {
                        for (o, s) in out.iter_mut().zip(&sum.px[i][..3]) {
                            let v = s * 255.0 / a + offset as f32;
                            *o = v.round().clamp(0.0, 255.0) as u8;
                        }
                    }
                    out
                })
                .collect()
        }
        Filter::SurfaceBlur { radius, threshold } => {
            // Each channel averages the square around it, a neighbour
            // counting 1 − |difference| / (2.5 × threshold) (no less than 0)
            let r = radius as isize;
            let reach = 2.5 * threshold.max(1) as f32;
            (0..w * h)
                .map(|i| {
                    let (x, y) = ((i % w) as isize, (i / w) as isize);
                    let here = Buffer::straight(src.px[i]);
                    let mut out = here;
                    for (c, v) in out.iter_mut().take(3).enumerate() {
                        let (mut num, mut den) = (0f32, 0f32);
                        for dy in -r..=r {
                            for dx in -r..=r {
                                let p = Buffer::straight(src.at(x + dx, y + dy))[c] as f32;
                                let wgt = (1.0 - (p - here[c] as f32).abs() / reach).max(0.0);
                                num += wgt * p;
                                den += wgt;
                            }
                        }
                        *v = (num / den).round().clamp(0.0, 255.0) as u8;
                    }
                    out
                })
                .collect()
        }
        Filter::DustAndScratches { radius, threshold } => {
            // The median, where any channel differs from it by more than
            // the threshold
            let median = src.rank(radius, |v| {
                let mid = v.len() / 2;
                *v.select_nth_unstable_by(mid, f32::total_cmp).1
            });
            (0..w * h)
                .map(|i| {
                    let here = Buffer::straight(src.px[i]);
                    let m = Buffer::straight(median.px[i]);
                    let far = (0..3).any(|c| here[c].abs_diff(m[c]) > threshold);
                    if far { m } else { here }
                })
                .collect()
        }
        Filter::Twirl { .. }
        | Filter::Pinch { .. }
        | Filter::Spherize { .. }
        | Filter::Ripple { .. }
        | Filter::ZigZag { .. }
        | Filter::PolarCoordinates { .. } => {
            // Each pixel takes the color at the place the distortion maps
            // it from (measured from Photoshop 2026 on a coordinate image)
            let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
            (0..w * h)
                .map(|i| {
                    let (x, y) = ((i % w) as f32, (i / w) as f32);
                    let (sx, sy) = distort_source(filter, x, y, cx, cy, w as f32, h as f32);
                    Buffer::straight(src.sample(sx, sy))
                })
                .collect()
        }
        Filter::Emboss {
            angle,
            height,
            amount,
        } => {
            // Gray plus the difference between the points half the height
            // away on either side along the angle. Each point reads the
            // pixels within one pixel of it, weighted 1 − d − 0.012·d³ by
            // their distance d and not normalized: fitted to Photoshop's
            // impulse responses (within a level at every angle measured,
            // exact on the axes).
            let (sin, cos) = (angle as f32).to_radians().sin_cos();
            let reach = height as f32 / 2.0;
            let (qx, qy) = (reach * cos, -reach * sin);
            let mut taps = Vec::new();
            for dy in qy.floor() as isize - 1..=qy.ceil() as isize + 1 {
                for dx in qx.floor() as isize - 1..=qx.ceil() as isize + 1 {
                    let d = ((dx as f32 - qx).powi(2) + (dy as f32 - qy).powi(2)).sqrt();
                    let wgt = 1.0 - d - 0.012 * d * d * d;
                    if wgt > 0.0 {
                        taps.push((dx, dy, wgt));
                    }
                }
            }
            let k = amount as f32 / 100.0;
            let color = |x: isize, y: isize, c: usize| {
                let p = src.at(x, y);
                if p[3] > 0.0 { p[c] * 255.0 / p[3] } else { 0.0 }
            };
            (0..w * h)
                .map(|i| {
                    let (x, y) = ((i % w) as isize, (i / w) as isize);
                    let mut out = Buffer::straight(src.px[i]);
                    for (c, v) in out.iter_mut().take(3).enumerate() {
                        let diff: f32 = taps
                            .iter()
                            .map(|&(dx, dy, wgt)| {
                                (color(x + dx, y + dy, c) - color(x - dx, y - dy, c)) * wgt
                            })
                            .sum();
                        *v = (128.0 + diff * k).round().clamp(0.0, 255.0) as u8;
                    }
                    out
                })
                .collect()
        }
        Filter::HighPass { radius } => {
            let blurred = src.gaussian(radius);
            (0..w * h)
                .map(|i| {
                    let orig = Buffer::straight(src.px[i]);
                    let blur = Buffer::straight(blurred.px[i]);
                    let mut out = orig;
                    for c in 0..3 {
                        out[c] = (orig[c] as f32 - blur[c] as f32 + 128.0)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                    }
                    out
                })
                .collect()
        }
        Filter::Offset { dx, dy, fill } => {
            let empty = background.map_or([0; 4], |[r, g, b]| [r, g, b, 255]);
            per_pixel(&|x, y, _| {
                let sx = x as i64 - dx as i64;
                let sy = y as i64 - dy as i64;
                let (wi, hi) = (w as i64, h as i64);
                let inside = (0..wi).contains(&sx) && (0..hi).contains(&sy);
                let (sx, sy) = match fill {
                    _ if inside => (sx, sy),
                    OffsetFill::Background => return empty,
                    OffsetFill::RepeatEdges => (sx.clamp(0, wi - 1), sy.clamp(0, hi - 1)),
                    OffsetFill::Wrap => (sx.rem_euclid(wi), sy.rem_euclid(hi)),
                };
                Buffer::straight(src.px[(sy * wi + sx) as usize])
            })
        }
        Filter::Mosaic { cell } => {
            let cell = cell.max(1) as usize;
            let mut out = vec![[0u8; 4]; w * h];
            for cy in (0..h).step_by(cell) {
                for cx in (0..w).step_by(cell) {
                    let (x1, y1) = ((cx + cell).min(w), (cy + cell).min(h));
                    let mut sum = [0f32; 4];
                    for y in cy..y1 {
                        for x in cx..x1 {
                            let p = src.px[y * w + x];
                            for c in 0..4 {
                                sum[c] += p[c];
                            }
                        }
                    }
                    let n = ((x1 - cx) * (y1 - cy)) as f32;
                    let mean = Buffer::straight(sum.map(|v| v / n));
                    for y in cy..y1 {
                        out[y * w + cx..y * w + x1].fill(mean);
                    }
                }
            }
            out
        }
        Filter::Crystallize { cell } => crystallize(&src, cell.max(3) as usize),
        Filter::Pointillize { cell } => pointillize(&src, cell.max(3) as usize, paper),
        Filter::Diffuse { mode } => diffuse(&src, mode),
        Filter::Mezzotint { kind } => mezzotint(&src, kind),
        Filter::Facet => facet(&src),
        Filter::ShadowsHighlights(settings) => {
            let mut px: Vec<[u8; 4]> = src.px.iter().map(|&p| Buffer::straight(p)).collect();
            crate::tone::shadows_highlights(&mut px, w, h, settings);
            px
        }
        Filter::HdrToning(settings) => {
            let mut px: Vec<[u8; 4]> = src.px.iter().map(|&p| Buffer::straight(p)).collect();
            crate::tone::hdr_toning(&mut px, w, h, settings);
            px
        }
        Filter::Wave(_)
        | Filter::Shear { .. }
        | Filter::Displace { .. }
        | Filter::RadialBlur { .. }
        | Filter::SmartBlur { .. }
        | Filter::ShapeBlur { .. }
        | Filter::LensBlur(_)
        | Filter::ReduceNoise { .. }
        | Filter::SmartSharpen { .. }
        | Filter::Fibers { .. }
        | Filter::LensFlare { .. }
        | Filter::Extrude { .. }
        | Filter::OilPaint { .. } => {
            let px: Vec<[u8; 4]> = src.px.iter().map(|&p| Buffer::straight(p)).collect();
            // Lens Blur's depth map from the transparency
            let alpha: Option<Vec<u8>> = match filter {
                Filter::LensBlur(o) if o.depth == crate::more_filters::DepthSource::Transparency => {
                    Some(px.iter().map(|p| p[3]).collect())
                }
                _ => None,
            };
            more(&px, w, h, filter, alpha.as_deref().or(mask))
        }
        Filter::HsbHsl { input, output } => per_pixel(&|_, _, p| {
            let rgb = to_rgb(input, [p[0], p[1], p[2]]);
            let out = from_rgb(output, rgb);
            [out[0], out[1], out[2], p[3]]
        }),
        Filter::Tiles {
            count,
            offset,
            fill,
            foreground,
        } => tiles(&src, (count.max(1), offset), fill, (foreground, paper)),
        Filter::ColorHalftone { radius, angles } => color_halftone(&src, radius.max(4), angles),
        Filter::Clouds {
            foreground,
            background,
            seed,
        } => {
            let t = clouds(w, h, seed);
            (0..w * h)
                .map(|i| {
                    let c = mix_rgb(foreground, background, t[i]);
                    [c[0], c[1], c[2], 255]
                })
                .collect()
        }
        Filter::DifferenceClouds {
            foreground,
            background,
            seed,
        } => {
            let t = clouds(w, h, seed);
            (0..w * h)
                .map(|i| {
                    let px = Buffer::straight(src.px[i]);
                    let c = mix_rgb(foreground, background, t[i]);
                    [
                        px[0].abs_diff(c[0]),
                        px[1].abs_diff(c[1]),
                        px[2].abs_diff(c[2]),
                        px[3],
                    ]
                })
                .collect()
        }
        Filter::Solarize => per_pixel(&|_, _, px| {
            let s = |v: u8| if v > 127 { 255 - v } else { v };
            [s(px[0]), s(px[1]), s(px[2]), px[3]]
        }),
    }
}

/// The `more_filters` ones.
/// The filters in `more_filters` (`depth`: Lens Blur's depth map from the
/// layer mask, one value a pixel).
fn more(
    px: &[[u8; 4]],
    w: usize,
    h: usize,
    filter: Filter,
    depth: Option<&[u8]>,
) -> Vec<[u8; 4]> {
    use crate::more_filters as m;
    match filter {
        Filter::Wave(s) => m::wave(px, w, h, s),
        Filter::Shear {
            points,
            count,
            undefined,
        } => {
            let offsets = m::shear_offsets(&points[..count.min(8) as usize], h);
            m::shear(px, w, h, &offsets, undefined)
        }
        Filter::Displace {
            map,
            scale,
            stretch,
            undefined,
        } => match m::map(map) {
            Some(data) => {
                let map = m::DisplaceMap {
                    width: data.0,
                    height: data.1,
                    pixels: &data.2,
                    stretch,
                };
                m::displace(px, w, h, &map, scale, undefined)
            }
            None => px.to_vec(),
        },
        Filter::RadialBlur {
            amount,
            method,
            quality,
            center,
        } => m::radial_blur(px, w, h, amount, method, quality, center),
        Filter::SmartBlur {
            radius,
            threshold,
            mode,
        } => m::smart_blur(px, w, h, radius, threshold, mode),
        Filter::ShapeBlur { radius, shape } => m::shape_blur(px, w, h, radius, shape),
        Filter::LensBlur(o) => m::lens_blur(px, w, h, &o, depth),
        Filter::ReduceNoise {
            strength,
            preserve,
            color,
            sharpen,
        } => m::reduce_noise(px, w, h, strength, preserve, color, sharpen),
        Filter::SmartSharpen {
            amount,
            radius,
            noise,
            remove,
            angle,
            fade,
        } => m::smart_sharpen(px, w, h, amount, radius, noise, remove, angle, fade),
        Filter::Fibers {
            variance,
            strength,
            foreground,
            background,
            seed,
        } => m::fibers(w, h, variance, strength, foreground, background, seed),
        Filter::LensFlare {
            center,
            brightness,
            lens,
        } => m::lens_flare(px, w, h, center, brightness, lens),
        Filter::Extrude {
            kind,
            size,
            depth,
            level_based,
            solid,
            mask_incomplete,
            seed,
        } => m::extrude(
            px,
            w,
            h,
            kind,
            size as usize,
            depth,
            level_based,
            solid,
            mask_incomplete,
            seed,
        ),
        Filter::OilPaint {
            stylization,
            cleanliness,
            scale,
            angle,
            shine,
        } => m::oil_paint(px, w, h, stylization, cleanliness, scale, angle, shine),
        _ => px.to_vec(),
    }
}

/// Applies `filter` to the active layer. The background layer and layers
/// with locked transparency keep their alpha; partly selected pixels mix
/// the old and new values.
pub fn apply(doc: &mut Document, filter: Filter, background: [u8; 3]) -> Result<(), FillError> {
    crate::adjust::check(doc)?;
    let selection = doc.selection().cloned();
    let (w, h) = (doc.width, doc.height);
    let id = doc.active_layer.expect("checked");
    let transparency_locked = doc.transparency_locked(id);
    let layer = doc.layer_mut(id).expect("checked");
    let keep_alpha = layer.is_background || transparency_locked;
    let is_background = layer.is_background;
    // Lens Blur's depth map from the layer mask
    let mask: Option<Vec<u8>> = match filter {
        Filter::LensBlur(o) if o.depth == crate::more_filters::DepthSource::LayerMask => {
            layer.mask.as_ref().map(|m| {
                (0..h)
                    .flat_map(|y| (0..w).map(move |x| (x, y)))
                    .map(|(x, y)| m.value(x, y))
                    .collect()
            })
        }
        _ => None,
    };
    let image = layer.image_mut().expect("checked: not a group");
    let out = filtered(
        image,
        filter,
        selection.as_ref(),
        is_background.then_some(background),
        background,
        mask.as_deref(),
    );
    for y in 0..h {
        for x in 0..w {
            let m = selection.as_ref().map_or(255, |s| s.get(x, y)) as u32;
            if m == 0 {
                continue;
            }
            let old = image.pixel(x, y);
            let mut new = out[(y * w + x) as usize];
            if keep_alpha {
                new[3] = old[3];
            }
            if m < 255 {
                for c in 0..4 {
                    new[c] = ((old[c] as u32 * (255 - m) + new[c] as u32 * m + 127) / 255) as u8;
                }
            }
            if new != old {
                image.set_pixel(x, y, new);
            }
        }
    }
    doc.mark_dirty();
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn motion_blur_speed() {
        let (w, h) = (3000, 2000);
        let src = Buffer {
            w,
            h,
            px: (0..w * h)
                .map(|i| [(i % 251) as f32, 50.0, 90.0, 255.0])
                .collect(),
        };
        let t = std::time::Instant::now();
        let out = src.motion_blur(30, 50);
        eprintln!("3000 x 2000, 30°, 50 px: {:?}", t.elapsed());
        assert_eq!(out.px.len(), w * h);
    }

    use super::*;
    use crate::color::Color;
    use crate::layer::Layer;

    /// A 5×1 white background with a black pixel in the middle.
    fn doc() -> Document {
        let mut doc = Document::new_with_background("t", 5, 1, Color::WHITE);
        let id = doc.active_layer.unwrap();
        let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(2, 0, [0, 0, 0, 255]);
        doc
    }

    fn row(doc: &Document) -> Vec<u8> {
        doc.composite_rgba8().chunks(4).map(|p| p[0]).collect()
    }

    fn run(filter: Filter) -> Vec<u8> {
        let mut d = doc();
        apply(&mut d, filter, [255, 255, 255]).unwrap();
        row(&d)
    }

    #[test]
    fn blurs_spread_the_dark_pixel() {
        assert_eq!(
            run(Filter::BoxBlur { radius: 1 }),
            [255, 170, 170, 170, 255]
        );
        let g = run(Filter::GaussianBlur { radius: 1.0 });
        assert!(g[2] > 0 && g[2] < g[1] && g[1] < g[0], "{g:?}");
        assert_eq!(g[1], g[3]);
        assert_eq!(run(Filter::Average), [204; 5]);
    }

    #[test]
    fn rank_filters() {
        assert_eq!(run(Filter::Median { radius: 1 }), [255; 5]);
        assert_eq!(
            run(Filter::Minimum {
                radius: 1.0,
                round: false
            }),
            [255, 0, 0, 0, 255]
        );
        assert_eq!(
            run(Filter::Maximum {
                radius: 1.0,
                round: false
            }),
            [255; 5]
        );
    }

    #[test]
    fn sharpen_high_pass_and_solarize() {
        // Unsharp Mask darkens the dark pixel's neighbors' contrast edge
        let u = run(Filter::UnsharpMask {
            amount: 100.0,
            radius: 1.0,
            threshold: 0,
        });
        assert_eq!(u[2], 0);
        assert_eq!(u[0], 255);
        let hp = run(Filter::HighPass { radius: 1.0 });
        assert!(hp[2] < 128 && hp[0] > 128, "{hp:?}");
        assert_eq!(run(Filter::Solarize), [0, 0, 0, 0, 0]);
    }

    #[test]
    fn offset_wraps_or_fills_with_the_background() {
        let wrap = Filter::Offset {
            dx: 3,
            dy: 0,
            fill: OffsetFill::Wrap,
        };
        assert_eq!(run(wrap), [0, 255, 255, 255, 255]);
        let mut d = doc();
        let back = Filter::Offset {
            dx: -2,
            dy: 0,
            fill: OffsetFill::Background,
        };
        apply(&mut d, back, [9, 9, 9]).unwrap();
        assert_eq!(row(&d), [0, 255, 255, 9, 9]);
    }

    #[test]
    fn mosaic_and_noise() {
        assert_eq!(run(Filter::Mosaic { cell: 5 }), [204; 5]);
        let noisy = Filter::AddNoise {
            amount: 50.0,
            gaussian: false,
            monochromatic: true,
        };
        let mut a = doc();
        apply(&mut a, noisy, [255; 3]).unwrap();
        let mut b = doc();
        apply(&mut b, noisy, [255; 3]).unwrap();
        // Repeatable, so previews and the final result agree
        assert_eq!(a.composite_rgba8(), b.composite_rgba8());
        // Monochromatic noise keeps pixels gray
        let px = &a.composite_rgba8()[4..8];
        assert_eq!((px[0], px[0]), (px[1], px[2]));
    }

    #[test]
    fn wind_streaks_downwind() {
        // Bright dots on a dark ground, one per row
        let make = || {
            let mut d =
                Document::new_with_background("t", 64, 32, Color::from_rgba8([40, 40, 40, 255]));
            let id = d.active_layer.unwrap();
            let image = d.layer_mut(id).unwrap().image_mut().unwrap();
            for y in 0..32 {
                image.set_pixel(32, y, [220, 220, 220, 255]);
            }
            d
        };
        let row = |d: &Document, y: usize| -> Vec<u8> {
            d.composite_rgba8()[y * 64 * 4..(y + 1) * 64 * 4]
                .chunks(4)
                .map(|p| p[0])
                .collect()
        };
        for method in [WindMethod::Wind, WindMethod::Blast] {
            for from_left in [true, false] {
                let filter = Filter::Wind { method, from_left };
                let mut a = make();
                apply(&mut a, filter, [255; 3]).unwrap();
                let mut b = make();
                apply(&mut b, filter, [255; 3]).unwrap();
                assert_eq!(a.composite_rgba8(), b.composite_rgba8(), "repeatable");
                let (mut streaks, mut upwind) = (0, 0);
                for y in 0..32 {
                    let r = row(&a, y);
                    assert_eq!(r[32], 220, "the dot stays");
                    // Streaks only brighten, and only downwind
                    assert!(r.iter().all(|&v| v >= 40));
                    let (down, up) = if from_left { (33, 31) } else { (31, 33) };
                    streaks += (r[down] > 40) as usize;
                    upwind += (r[up] > 40) as usize;
                }
                assert!(streaks > 4 && streaks < 28, "{method:?}: {streaks} streaks");
                assert_eq!(upwind, 0);
            }
        }
        // A flat image has nothing to blow
        let mut flat = Document::new_with_background("t", 16, 4, Color::WHITE);
        let before = flat.composite_rgba8();
        for method in [WindMethod::Wind, WindMethod::Blast, WindMethod::Stagger] {
            apply(
                &mut flat,
                Filter::Wind {
                    method,
                    from_left: true,
                },
                [255; 3],
            )
            .unwrap();
        }
        assert_eq!(flat.composite_rgba8(), before);
    }

    #[test]
    fn transparent_pixels_do_not_bleed_color() {
        // Red pixel on a transparent layer: the blur stays red, fading out
        let mut d = Document::new_with_background("t", 3, 1, Color::WHITE);
        let mut image = TiledImage::new(3, 1);
        image.set_pixel(1, 0, [255, 0, 0, 255]);
        d.insert_above_active(Layer::raster(d.new_layer_id(), "Layer 1", image));
        apply(&mut d, Filter::BoxBlur { radius: 1 }, [255; 3]).unwrap();
        let image = d.layers[1].image().unwrap();
        assert_eq!(image.pixel(0, 0), [255, 0, 0, 85]);
    }

    /// The distortions against Photoshop: the mean difference allowed (a
    /// mapping a fraction of a pixel off moves isolated dots a lot, so the
    /// worst pixel says little).
    #[test]
    fn distortions_match_photoshop() {
        for (name, filter, allowed) in [
            ("f_tw.rgb", Filter::Twirl { angle: 120 }, 0.6),
            ("f_pinch.rgb", Filter::Pinch { amount: 60 }, 0.25),
            ("f_pinchm.rgb", Filter::Pinch { amount: -40 }, 0.25),
            (
                "f_sph.rgb",
                Filter::Spherize {
                    amount: 70,
                    mode: SpherizeMode::Normal,
                },
                1.5,
            ),
            (
                "f_sphm.rgb",
                Filter::Spherize {
                    amount: -80,
                    mode: SpherizeMode::Normal,
                },
                1.4,
            ),
            (
                "f_sphv.rgb",
                Filter::Spherize {
                    amount: 100,
                    mode: SpherizeMode::VerticalOnly,
                },
                1.8,
            ),
            (
                "f_polar.rgb",
                Filter::PolarCoordinates { to_polar: true },
                0.7,
            ),
            (
                "f_rect.rgb",
                Filter::PolarCoordinates { to_polar: false },
                0.1,
            ),
        ] {
            let (_, _, mean) = photoshop::compare_mean(filter, name);
            assert!(mean <= allowed, "{name}: off by {mean} on average");
        }
    }

    /// Comparisons with Photoshop 2026's own results (fixtures/filter):
    /// probe.rgb is a 64 × 64 image (a red–green ramp, a white square, a
    /// black line, scattered dots); the other files are Photoshop's output.
    mod photoshop {
        use super::*;
        use crate::color::Color;

        const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/filter/");

        /// Applies `filter` to the probe and returns the largest channel
        /// difference from Photoshop's `expected` output and how many
        /// channels differ by more than 2.
        pub fn compare(filter: Filter, expected: &str) -> (u8, usize) {
            let (worst, off, _) = compare_mean(filter, expected);
            (worst, off)
        }

        /// [`compare`] with the mean difference too.
        pub fn compare_mean(filter: Filter, expected: &str) -> (u8, usize, f32) {
            let probe = std::fs::read(format!("{FIXTURES}probe.rgb")).unwrap();
            let want = std::fs::read(format!("{FIXTURES}{expected}")).unwrap();
            let mut doc = Document::new_with_background("t", 64, 64, Color::WHITE);
            let id = doc.active_layer.unwrap();
            let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
            for (i, px) in probe.chunks(3).enumerate() {
                image.set_pixel(i as u32 % 64, i as u32 / 64, [px[0], px[1], px[2], 255]);
            }
            apply(&mut doc, filter, [255; 3]).unwrap();
            let got = doc.composite_rgba8();
            let (mut worst, mut off, mut sum) = (0u8, 0, 0u32);
            for (i, w) in want.chunks(3).enumerate() {
                for c in 0..3 {
                    let d = got[i * 4 + c].abs_diff(w[c]);
                    worst = worst.max(d);
                    off += (d > 2) as usize;
                    sum += d as u32;
                }
            }
            (worst, off, sum as f32 / want.len() as f32)
        }

        /// Each filter against Photoshop: the largest difference allowed
        /// (1 level where the algorithm is Photoshop's; Minimum/Maximum's
        /// roundness, angled Motion Blur and Emboss are approximations).
        #[test]
        fn filters_match_photoshop() {
            for (name, filter, allowed) in [
                ("f_g03.rgb", Filter::GaussianBlur { radius: 0.3 }, 1),
                ("f_g1.rgb", Filter::GaussianBlur { radius: 1.0 }, 1),
                ("f_g25.rgb", Filter::GaussianBlur { radius: 2.5 }, 1),
                ("f_g10.rgb", Filter::GaussianBlur { radius: 10.0 }, 1),
                ("f_box1.rgb", Filter::BoxBlur { radius: 1 }, 1),
                ("f_box3.rgb", Filter::BoxBlur { radius: 3 }, 1),
                ("f_med1.rgb", Filter::Median { radius: 1 }, 1),
                ("f_med3.rgb", Filter::Median { radius: 3 }, 1),
                ("f_blur.rgb", Filter::Blur, 1),
                ("f_blurm.rgb", Filter::BlurMore, 1),
                ("f_shrp.rgb", Filter::Sharpen, 1),
                ("f_shrpm.rgb", Filter::SharpenMore, 1),
                ("f_fnde.rgb", Filter::FindEdges, 1),
                ("f_hp10.rgb", Filter::HighPass { radius: 10.0 }, 1),
                ("f_hp2.rgb", Filter::HighPass { radius: 2.0 }, 1),
                ("f_mos5.rgb", Filter::Mosaic { cell: 5 }, 1),
                ("f_sol.rgb", Filter::Solarize, 1),
                ("f_frag.rgb", Filter::Fragment, 1),
                ("f_desp.rgb", Filter::Despeckle, 1),
                ("f_shrpe.rgb", Filter::SharpenEdges, 1),
                (
                    "f_tracel.rgb",
                    Filter::TraceContour {
                        level: 100,
                        upper: false,
                    },
                    0,
                ),
                (
                    "f_cust.rgb",
                    Filter::Custom {
                        kernel: [
                            0, 0, 0, 0, 0, 0, -1, -1, -1, 0, 0, -1, 12, -1, 0, 0, -1, -1, -1, 0, 0,
                            0, 0, 0, 0,
                        ],
                        scale: 4,
                        offset: 10,
                    },
                    1,
                ),
                (
                    "f_cust2.rgb",
                    Filter::Custom {
                        kernel: [1; 25],
                        scale: 25,
                        offset: 0,
                    },
                    1,
                ),
                (
                    "f_surf.rgb",
                    Filter::SurfaceBlur {
                        radius: 5,
                        threshold: 15,
                    },
                    1,
                ),
                (
                    "f_surf2.rgb",
                    Filter::SurfaceBlur {
                        radius: 2,
                        threshold: 40,
                    },
                    1,
                ),
                (
                    "f_dust.rgb",
                    Filter::DustAndScratches {
                        radius: 2,
                        threshold: 0,
                    },
                    1,
                ),
                (
                    "f_dust2.rgb",
                    Filter::DustAndScratches {
                        radius: 3,
                        threshold: 20,
                    },
                    1,
                ),
            ] {
                let (worst, _) = compare(filter, name);
                assert!(worst <= allowed, "{name}: off by {worst}");
            }
        }

        #[test]
        fn trace_contour_upper_matches_photoshop() {
            // One channel value of 12 288 differs: a lone 255 among
            // neighbours exactly at the level, which Photoshop marks
            let (_, off) = compare(
                Filter::TraceContour {
                    level: 128,
                    upper: true,
                },
                "f_trace.rgb",
            );
            assert!(off <= 1, "{off} values differ");
        }
    }

    #[test]
    fn clouds_span_the_colors_without_repeating() {
        let t = clouds(512, 300, 7);
        let (lo, hi) = t
            .iter()
            .fold((1f32, 0f32), |(a, b), &v| (a.min(v), b.max(v)));
        assert_eq!((lo, hi), (0.0, 1.0));
        let shifted = (0..300)
            .filter(|y| t[y * 512 + 10] == t[y * 512 + 266])
            .count();
        assert!(shifted < 10, "{shifted}");
        // The same seed makes the same clouds, another seed others
        assert_eq!(t, clouds(512, 300, 7));
        assert_ne!(t, clouds(512, 300, 8));
        // Soft: neighbors differ little
        let step = t
            .chunks(512)
            .flat_map(|row| row.windows(2).map(|w| (w[0] - w[1]).abs()))
            .fold(0f32, f32::max);
        assert!(step < 0.3, "{step}");
    }

    /// A 40 × 40 horizontal gradient as a filter source.
    fn ramp() -> Buffer {
        let mut img = TiledImage::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                let v = (x * 6) as u8;
                img.set_pixel(x, y, [v, 255 - v, (y * 6) as u8, 255]);
            }
        }
        Buffer::from_image(&img)
    }

    #[test]
    fn crystallize_fills_cells_with_one_color() {
        let out = crystallize(&ramp(), 10);
        let mut colors: Vec<[u8; 4]> = out.clone();
        colors.sort();
        colors.dedup();
        // About one color per 10 × 10 block, far fewer than pixels
        assert!((10..=25).contains(&colors.len()), "{}", colors.len());
        // Neighbors mostly share their cell's color
        let same = (0..out.len() - 1).filter(|&i| out[i] == out[i + 1]).count();
        assert!(same > out.len() * 8 / 10);
    }

    #[test]
    fn pointillize_paints_dots_on_the_background() {
        let out = pointillize(&ramp(), 6, [255, 255, 255]);
        let paper = out.iter().filter(|p| **p == [255, 255, 255, 255]).count();
        assert!(paper > 0 && paper < out.len() / 2, "{paper}");
        assert!(out.iter().all(|p| p[3] == 255));
    }

    #[test]
    fn diffuse_moves_pixels_between_neighbors() {
        let src = ramp();
        let normal = diffuse(&src, DiffuseMode::Normal);
        let moved = (0..normal.len())
            .filter(|&i| normal[i] != Buffer::straight(src.px[i]))
            .count();
        assert!(moved > normal.len() / 2, "{moved}");
        // Darken Only never lightens; Lighten Only never darkens
        let luma = |p: [u8; 4]| p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114;
        let dark = diffuse(&src, DiffuseMode::DarkenOnly);
        let light = diffuse(&src, DiffuseMode::LightenOnly);
        for i in 0..dark.len() {
            let here = luma(Buffer::straight(src.px[i]));
            assert!(luma(dark[i]) <= here + 0.01 && luma(light[i]) >= here - 0.01);
        }
    }

    #[test]
    fn mezzotint_leaves_only_full_or_empty_channels() {
        for kind in MezzotintType::ALL {
            let out = mezzotint(&ramp(), kind);
            assert!(
                out.iter()
                    .all(|p| p[..3].iter().all(|&c| c == 0 || c == 255))
            );
            // Brighter columns get more full red
            let red = |x: usize| (0..40).filter(|&y| out[y * 40 + x][0] == 255).count();
            assert!(red(38) > red(2), "{kind:?}");
        }
    }

    #[test]
    fn ripple_shifts_by_its_amount() {
        let at = |amount| {
            distortion_source(
                Filter::Ripple {
                    amount,
                    size: RippleSize::Medium,
                },
                20.0,
                3.0,
                40.0,
                40.0,
            )
        };
        assert_eq!(at(0), (20.0, 3.0));
        let (x, _) = at(100);
        let (x2, _) = at(200);
        assert!((x - 20.0).abs() > 1.0 && ((x2 - 20.0) / (x - 20.0) - 2.0).abs() < 1e-3);
    }

    #[test]
    fn tiles_cut_and_shift_the_image() {
        let src = ramp();
        // No offset: the image as it was
        let same = tiles(&src, (4, 0), TilesFill::Background, ([0; 3], [255; 3]));
        assert!((0..same.len()).all(|i| same[i] == Buffer::straight(src.px[i])));
        // With an offset some background shows between the tiles
        let moved = tiles(
            &src,
            (4, 30),
            TilesFill::Background,
            ([0; 3], [255, 0, 255]),
        );
        assert!(moved.contains(&[255, 0, 255, 255]));
        let fg = tiles(
            &src,
            (4, 30),
            TilesFill::Foreground,
            ([0, 255, 0], [255; 3]),
        );
        assert!(fg.contains(&[0, 255, 0, 255]));
    }

    #[test]
    fn halftone_dots_follow_the_values() {
        let src = ramp();
        let out = color_halftone(&src, 4, [108, 162, 90, 45]);
        // Red rises across the ramp: more red ink on the right
        let red = |x0: usize| {
            (0..40)
                .flat_map(|y| (x0..x0 + 8).map(move |x| (x, y)))
                .map(|(x, y)| out[y * 40 + x][0] as u32)
                .sum::<u32>()
        };
        assert!(red(32) > red(0) * 2, "{} {}", red(32), red(0));
    }

    #[test]
    fn facet_flattens_noise_and_keeps_edges() {
        // Left half dark, right half light, with a little noise
        let mut img = TiledImage::new(20, 20);
        for y in 0..20 {
            for x in 0..20 {
                let base: u8 = if x < 10 { 40 } else { 200 };
                let v = base + ((x * 7 + y * 13) % 5) as u8;
                img.set_pixel(x, y, [v, v, v, 255]);
            }
        }
        let out = facet(&Buffer::from_image(&img));
        // The edge stays where it was
        assert!(out[5 * 20 + 9][0] < 60 && out[5 * 20 + 10][0] > 180);
        // The noise is smoothed: fewer distinct values on a row
        let mut row: Vec<u8> = (0..10).map(|x| out[5 * 20 + x][0]).collect();
        row.sort();
        row.dedup();
        assert!(row.len() <= 3, "{row:?}");
    }

    #[test]
    fn zigzag_leaves_the_center_and_edge_and_moves_between() {
        let f = |style| Filter::ZigZag {
            amount: 50,
            ridges: 4,
            style,
        };
        for style in [
            ZigZagStyle::AroundCenter,
            ZigZagStyle::OutFromCenter,
            ZigZagStyle::PondRipples,
        ] {
            assert_eq!(
                distortion_source(f(style), 50.0, 50.0, 100.0, 100.0),
                (50.0, 50.0)
            );
            assert_eq!(
                distortion_source(f(style), 0.0, 50.0, 100.0, 100.0),
                (0.0, 50.0)
            );
            let (x, y) = distortion_source(f(style), 60.0, 50.0, 100.0, 100.0);
            assert!((x - 60.0).abs() + (y - 50.0).abs() > 0.1, "{style:?}");
        }
        // Around Center keeps the distance; Out From Center keeps the line
        let (x, y) = distortion_source(f(ZigZagStyle::AroundCenter), 60.0, 50.0, 100.0, 100.0);
        assert!((((x - 50.0).powi(2) + (y - 50.0).powi(2)).sqrt() - 10.0).abs() < 0.01);
        let (_, y) = distortion_source(f(ZigZagStyle::OutFromCenter), 60.0, 50.0, 100.0, 100.0);
        assert!((y - 50.0).abs() < 1e-4);
    }

    #[test]
    fn hsb_and_hsl_round_trip() {
        // Pure red: hue 0, full saturation, full brightness / half lightness
        assert_eq!(from_rgb(ColorModel::Hsb, [255, 0, 0]), [0, 255, 255]);
        assert_eq!(from_rgb(ColorModel::Hsl, [255, 0, 0]), [0, 255, 128]);
        // Blue's hue is 240° → 170
        assert_eq!(from_rgb(ColorModel::Hsb, [0, 0, 255])[0], 170);
        for c in [[200, 100, 50], [10, 220, 140], [128, 128, 128], [0, 0, 0]] {
            for m in [ColorModel::Hsb, ColorModel::Hsl] {
                let back = to_rgb(m, from_rgb(m, c));
                assert!(
                    (0..3).all(|k| back[k].abs_diff(c[k]) <= 3),
                    "{m:?} {c:?} {back:?}"
                );
            }
        }
    }
}
