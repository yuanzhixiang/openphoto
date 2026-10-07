//! Image > Adjustments applied to the active layer's pixels, limited to the
//! selection.

use crate::document::Document;
use crate::fill::FillError;

/// A pixel adjustment with its settings.
#[derive(Clone, Copy, Debug, PartialEq)]
// Copied around freely (dialogs, previews, history names): the pencil's
// tables stay inline rather than boxed
#[allow(clippy::large_enum_variant)]
pub enum Adjustment {
    Invert,
    /// Gray at each pixel's HSL lightness, (max + min) / 2, like Photoshop.
    Desaturate,
    /// Pixels at least this bright (luminosity 0–255) become white, the
    /// rest black. Photoshop's range is 1–255.
    Threshold(u8),
    /// The number of tonal levels per channel, 2–255.
    Posterize(u8),
    /// Spreads the brightness values evenly (histogram equalization).
    Equalize,
    /// Equalize with the selected area's histogram applied to the whole
    /// layer (Photoshop's "Equalize entire image based on selected area").
    EqualizeEntireImage,
    /// Levels for the composite RGB channel and the red, green and blue
    /// channels (in that order); each channel's own levels apply first.
    Levels([Levels; 4]),
    /// Hue/Saturation: the master settings, the six color ranges and
    /// Colorize.
    HueSaturation(HueSaturation),
    /// Exposure in stops (−20–20), offset (−0.5–0.5) and gamma correction
    /// (0.01–9.99), computed in linear light.
    Exposure {
        exposure: f32,
        offset: f32,
        gamma: f32,
    },
    /// Brightness −150–150 and contrast −50–100. The default behavior uses
    /// Photoshop's own tone curves (measured from Photoshop 2026, see
    /// `data/brightness_contrast.bin`); `legacy` is Use Legacy, a straight
    /// shift and stretch.
    BrightnessContrast {
        brightness: i32,
        contrast: i32,
        legacy: bool,
    },
    /// Color Balance: cyan–red, magenta–green and yellow–blue shifts
    /// (−100–100) for the shadows, midtones and highlights.
    ColorBalance {
        shadows: [i32; 3],
        midtones: [i32; 3],
        highlights: [i32; 3],
        preserve_luminosity: bool,
    },
    /// Black & White: how bright reds, yellows, greens, cyans, blues and
    /// magentas turn (percent, −200–300).
    BlackWhite {
        weights: [i32; 6],
        /// Tint: the gray takes this color's hue and saturation (Photoshop's
        /// Color blend).
        tint: Option<[u8; 3]>,
    },
    /// Vibrance (boosts muted colors most) and Saturation, −100–100 each.
    Vibrance {
        vibrance: i32,
        saturation: i32,
    },
    /// Photo Filter: a color multiplied in at `density` percent.
    PhotoFilter {
        color: [u8; 3],
        density: u8,
        preserve_luminosity: bool,
    },
    /// Gradient Map: luminosity mapped from `from` (shadows) to `to`.
    GradientMap {
        from: [u8; 3],
        to: [u8; 3],
        method: crate::gradient::Method,
    },
    /// Image > Auto Tone: each channel stretched to the full range,
    /// ignoring the darkest and lightest 0.1%.
    AutoTone,
    /// Image > Auto Contrast: all channels stretched together, so colors
    /// keep their balance.
    AutoContrast,
    /// Curves for the composite RGB channel and the red, green and blue
    /// channels: up to 16 (input, output) points each, the first
    /// `counts[c]` of `points[c]` used. Each channel's own curve applies
    /// first.
    Curves {
        points: [[(u8, u8); 16]; 4],
        counts: [u8; 4],
    },
    /// Curves drawn with the pencil: a table for the composite RGB channel
    /// and the red, green and blue channels (each channel's own first).
    CurveTables([[u8; 256]; 4]),
    /// Channel Mixer: each output channel (or, with `monochrome`, the gray)
    /// is red, green and blue in percent (−200–200) plus a constant
    /// (−200–200 percent of white).
    ChannelMixer {
        rows: [[i32; 4]; 3],
        monochrome: bool,
    },
    /// Selective Color: cyan, magenta, yellow and black (−100–100 percent)
    /// for Reds, Yellows, Greens, Cyans, Blues, Magentas, Whites, Neutrals
    /// and Blacks; `absolute` is the Absolute method (otherwise Relative).
    SelectiveColor {
        colors: [[i32; 4]; 9],
        absolute: bool,
    },
    /// Image > Auto Color: here each channel stretched like Auto Tone
    /// (Photoshop also neutralizes the midtones).
    AutoColor,
}

impl Adjustment {
    /// The menu and history name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Invert => "Invert",
            Self::Desaturate => "Desaturate",
            Self::Threshold(_) => "Threshold",
            Self::Posterize(_) => "Posterize",
            Self::Equalize | Self::EqualizeEntireImage => "Equalize",
            Self::Levels(_) => "Levels",
            Self::HueSaturation(_) => "Hue/Saturation",
            Self::Exposure { .. } => "Exposure",
            Self::BrightnessContrast { .. } => "Brightness/Contrast",
            Self::ColorBalance { .. } => "Color Balance",
            Self::BlackWhite { .. } => "Black & White",
            Self::Vibrance { .. } => "Vibrance",
            Self::PhotoFilter { .. } => "Photo Filter",
            Self::GradientMap { .. } => "Gradient Map",
            Self::AutoTone => "Auto Tone",
            Self::AutoContrast => "Auto Contrast",
            Self::AutoColor => "Auto Color",
            Self::Curves { .. } | Self::CurveTables(_) => "Curves",
            Self::ChannelMixer { .. } => "Channel Mixer",
            Self::SelectiveColor { .. } => "Selective Color",
        }
    }

    /// Curves on the composite channel from a list of (input, output)
    /// points.
    pub fn curves(points: &[(u8, u8)]) -> Self {
        Self::curves_per_channel([points, &[], &[], &[]])
    }

    /// Curves with points for the composite, red, green and blue channels;
    /// an empty list leaves that channel alone.
    pub fn curves_per_channel(channels: [&[(u8, u8)]; 4]) -> Self {
        let mut points = [[(0u8, 0u8); 16]; 4];
        let mut counts = [0u8; 4];
        for (c, list) in channels.iter().enumerate() {
            let n = list.len().min(16);
            points[c][..n].copy_from_slice(&list[..n]);
            counts[c] = n as u8;
        }
        Self::Curves { points, counts }
    }

    /// Lookup tables for the red, green and blue channels, for
    /// adjustments that treat each channel on its own.
    pub fn tables(self) -> Option<[[u8; 256]; 3]> {
        let same = |t: [u8; 256]| Some([t; 3]);
        match self {
            Self::Curves { points, counts } => {
                let table = |c: usize| match counts[c] {
                    0 => IDENTITY,
                    n => curve_table(&points[c][..n as usize]),
                };
                Some(per_channel(table(0), [table(1), table(2), table(3)]))
            }
            Self::CurveTables(t) => Some(per_channel(t[0], [t[1], t[2], t[3]])),
            Self::Levels(levels) => Some(per_channel(
                levels[0].table(),
                [levels[1].table(), levels[2].table(), levels[3].table()],
            )),
            Self::BrightnessContrast {
                brightness,
                contrast,
                legacy,
            } => same(brightness_contrast_table(brightness, contrast, legacy)),
            Self::ColorBalance {
                shadows,
                midtones,
                highlights,
                preserve_luminosity,
            } => Some(color_balance_tables(
                shadows,
                midtones,
                highlights,
                preserve_luminosity,
            )),
            Self::Exposure {
                exposure,
                offset,
                gamma,
            } => {
                let mut table = [0u8; 256];
                for (i, t) in table.iter_mut().enumerate() {
                    // Linear light by a 2.2 gamma (not the sRGB curve), as
                    // Photoshop's Exposure works
                    let linear = (i as f32 / 255.0).powf(2.2) * 2f32.powf(exposure) + offset;
                    let v = linear.max(0.0).powf(1.0 / gamma).powf(1.0 / 2.2) * 255.0;
                    *t = v.round().clamp(0.0, 255.0) as u8;
                }
                same(table)
            }
            _ => None,
        }
    }
}

const IDENTITY: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        t[i] = i as u8;
        i += 1;
    }
    t
};

/// Each channel's own table followed by the composite one.
fn per_channel(composite: [u8; 256], channels: [[u8; 256]; 3]) -> [[u8; 256]; 3] {
    channels.map(|t| t.map(|v| composite[v as usize]))
}

/// Rounds halves up, as Photoshop's tone tables do.
fn round_half_up(v: f32) -> u8 {
    (v + 0.5).floor().clamp(0.0, 255.0) as u8
}

/// Photoshop's gamma curve on 0–1: `t^(1/gamma)`, except that for gammas
/// above 1 the slope at black is limited to `2^gamma`, with a quadratic
/// toe that meets the power curve smoothly (measured from Photoshop 2026's
/// Levels; at gamma 8 and above the power curve is never steeper).
pub fn levels_gamma(t: f32, gamma: f32) -> f32 {
    let power = t.powf(1.0 / gamma);
    if gamma <= 1.0 {
        return power;
    }
    // The toe y = a·t + b·t² has slope a at 0 and meets the power curve
    // with the same slope at t = x1
    let a = 2f32.powf(gamma);
    let x1 = (a / (2.0 - 1.0 / gamma)).powf(1.0 / (1.0 / gamma - 1.0));
    if !(0.0..1.0).contains(&x1) || t >= x1 {
        return power;
    }
    let y1 = x1.powf(1.0 / gamma);
    let b = (y1 - a * x1) / (x1 * x1);
    a * t + b * t * t
}

/// One channel's Levels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Levels {
    pub input_black: u8,
    pub input_white: u8,
    /// Midtone gamma, 0.01–9.99; above 1 brightens.
    pub gamma: f32,
    pub output_black: u8,
    pub output_white: u8,
}

impl Levels {
    pub const IDENTITY: Self = Self {
        input_black: 0,
        input_white: 255,
        gamma: 1.0,
        output_black: 0,
        output_white: 255,
    };

    /// Levels on the composite channel only.
    pub fn composite(self) -> [Self; 4] {
        [self, Self::IDENTITY, Self::IDENTITY, Self::IDENTITY]
    }

    pub fn table(&self) -> [u8; 256] {
        if *self == Self::IDENTITY {
            return IDENTITY;
        }
        let ib = self.input_black as f32;
        let iw = (self.input_white.max(self.input_black.saturating_add(1))) as f32;
        let (ob, ow) = (self.output_black as f32, self.output_white as f32);
        let mut table = [0u8; 256];
        for (i, t) in table.iter_mut().enumerate() {
            let u = ((i as f32 - ib) / (iw - ib)).clamp(0.0, 1.0);
            *t = round_half_up(ob + (ow - ob) * levels_gamma(u, self.gamma));
        }
        table
    }
}

/// Photoshop's brightness (−150–150) and contrast (−50–100) tone tables,
/// 256 entries each, measured from Photoshop 2026: brightness first, then
/// contrast.
static BRIGHTNESS_CONTRAST: &[u8; 452 * 256] = include_bytes!("../data/brightness_contrast.bin");

fn brightness_contrast_table(brightness: i32, contrast: i32, legacy: bool) -> [u8; 256] {
    let mut table = [0u8; 256];
    if legacy {
        // Use Legacy: brightness shifts, contrast stretches around 128; a
        // reduced contrast applies before the shift, a raised one after
        let shift = brightness as f32;
        let first = if contrast < 0 { 0.0 } else { shift };
        for (i, t) in table.iter_mut().enumerate() {
            let v = i as f32 + first;
            let out = if contrast >= 100 {
                if v >= 128.0 { 255.0 } else { 0.0 }
            } else {
                let k = if contrast > 0 {
                    100.0 / (100.0 - contrast as f32)
                } else {
                    (100.0 + contrast as f32) / 100.0
                };
                // Rounded half away from the middle
                let d = (v - 128.0) * k;
                128.0 + (d.abs() + 0.5).floor().copysign(d)
            };
            let out = if contrast < 0 { out + shift } else { out };
            *t = out.clamp(0.0, 255.0) as u8;
        }
        return table;
    }
    let b = (brightness.clamp(-150, 150) + 150) as usize * 256;
    let c = (301 + (contrast.clamp(-50, 100) + 50) as usize) * 256;
    for (i, t) in table.iter_mut().enumerate() {
        let v = BRIGHTNESS_CONTRAST[b + i];
        *t = BRIGHTNESS_CONTRAST[c + v as usize];
    }
    table
}

/// Color Balance as one Levels-like curve per channel (Photoshop 2026,
/// measured): shadows below 0 raise the input black point, highlights
/// above 0 lower the input white point, and the midtones bend a gamma of
/// 2^(G/100). Without Preserve Luminosity, G = midtones + (shadows +
/// highlights) / 2. With it, the shadows shift so the largest is 0, the
/// highlights so the smallest is 0, the midtones so the largest and
/// smallest are centered on 0, and only the midtones bend the gamma.
fn color_balance_tables(
    shadows: [i32; 3],
    midtones: [i32; 3],
    highlights: [i32; 3],
    preserve: bool,
) -> [[u8; 256]; 3] {
    let f = |v: [i32; 3]| v.map(|x| x as f32);
    let (mut s, mut m, mut h) = (f(shadows), f(midtones), f(highlights));
    if preserve {
        let max = s[0].max(s[1]).max(s[2]);
        s = s.map(|v| v - max);
        let min = h[0].min(h[1]).min(h[2]);
        h = h.map(|v| v - min);
        let mid = (m[0].max(m[1]).max(m[2]) + m[0].min(m[1]).min(m[2])) / 2.0;
        m = m.map(|v| v - mid);
    }
    [0, 1, 2].map(|c| {
        let black = (-s[c]).max(0.0);
        let white = 255.0 - h[c].max(0.0);
        let g = if preserve {
            m[c]
        } else {
            m[c] + (s[c] + h[c]) / 2.0
        };
        let gamma = 2f32.powf(g / 100.0);
        let mut table = [0u8; 256];
        for (i, t) in table.iter_mut().enumerate() {
            let u = ((i as f32 - black) / (white - black)).clamp(0.0, 1.0);
            *t = round_half_up(255.0 * levels_gamma(u, gamma));
        }
        table
    })
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// RGB (0–1) to hue (0–360), saturation and lightness (0–1).
pub(crate) fn rgb_to_hsl([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return [0.0, 0.0, l];
    }
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    [h * 60.0, s, l]
}

pub(crate) fn hsl_to_rgb([h, s, l]: [f32; 3]) -> [f32; 3] {
    if s == 0.0 {
        return [l; 3];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let channel = |t: f32| {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    let h = h / 360.0;
    [channel(h + 1.0 / 3.0), channel(h), channel(h - 1.0 / 3.0)]
}

/// The curve through `points` (sorted by input) as a 256-entry table: a
/// natural cubic spline, flat beyond the first and last points, clamped to
/// 0–255.
pub fn curve_table(points: &[(u8, u8)]) -> [u8; 256] {
    let mut pts: Vec<(f32, f32)> = points.iter().map(|&(x, y)| (x as f32, y as f32)).collect();
    pts.sort_by(|a, b| a.0.total_cmp(&b.0));
    pts.dedup_by(|a, b| a.0 == b.0);
    let mut table = [0u8; 256];
    match pts.len() {
        0 => {
            for (i, t) in table.iter_mut().enumerate() {
                *t = i as u8;
            }
            return table;
        }
        1 => {
            table.fill(pts[0].1.round() as u8);
            return table;
        }
        _ => {}
    }
    // Second derivatives of the natural spline (tridiagonal solve)
    let n = pts.len();
    let mut m = vec![0f32; n];
    if n > 2 {
        let h: Vec<f32> = (0..n - 1).map(|i| pts[i + 1].0 - pts[i].0).collect();
        let mut a = vec![0f32; n];
        let mut b = vec![0f32; n];
        let mut c = vec![0f32; n];
        let mut d = vec![0f32; n];
        for i in 1..n - 1 {
            a[i] = h[i - 1];
            b[i] = 2.0 * (h[i - 1] + h[i]);
            c[i] = h[i];
            d[i] = 6.0 * ((pts[i + 1].1 - pts[i].1) / h[i] - (pts[i].1 - pts[i - 1].1) / h[i - 1]);
        }
        // Thomas algorithm on rows 1..n-1 (m[0] = m[n-1] = 0)
        for i in 2..n - 1 {
            let w = a[i] / b[i - 1];
            b[i] -= w * c[i - 1];
            d[i] -= w * d[i - 1];
        }
        for i in (1..n - 1).rev() {
            let next = if i + 1 < n - 1 { m[i + 1] } else { 0.0 };
            m[i] = (d[i] - c[i] * next) / b[i];
        }
    }
    for (x, t) in table.iter_mut().enumerate() {
        let x = x as f32;
        let y = if x <= pts[0].0 {
            pts[0].1
        } else if x >= pts[n - 1].0 {
            pts[n - 1].1
        } else {
            let i = (0..n - 1).find(|&i| x <= pts[i + 1].0).unwrap_or(n - 2);
            let (x0, y0) = pts[i];
            let (x1, y1) = pts[i + 1];
            let h = x1 - x0;
            let (s, u) = ((x1 - x) / h, (x - x0) / h);
            s * y0 + u * y1 + ((s * s * s - s) * m[i] + (u * u * u - u) * m[i + 1]) * h * h / 6.0
        };
        *t = y.round().clamp(0.0, 255.0) as u8;
    }
    table
}

/// Linear sRGB to XYZ adapted to D50 (Photoshop's profile connection
/// space).
const SRGB_TO_XYZ_D50: [[f32; 3]; 3] = [
    [0.4361, 0.3851, 0.1431],
    [0.2225, 0.7169, 0.0606],
    [0.0139, 0.0971, 0.7141],
];

fn to_xyz(rgb: [f32; 3]) -> [f32; 3] {
    SRGB_TO_XYZ_D50.map(|row| row[0] * rgb[0] + row[1] * rgb[1] + row[2] * rgb[2])
}

fn from_xyz(xyz: [f32; 3]) -> [f32; 3] {
    // The inverse of SRGB_TO_XYZ_D50
    const M: [[f32; 3]; 3] = [
        [3.1336, -1.6168, -0.4907],
        [-0.9787, 1.9161, 0.0335],
        [0.0721, -0.2291, 1.4054],
    ];
    M.map(|row| row[0] * xyz[0] + row[1] * xyz[1] + row[2] * xyz[2])
}

fn lum601([r, g, b]: [f32; 3]) -> f32 {
    0.3 * r + 0.59 * g + 0.11 * b
}

/// The blend modes' SetLum (0–1 values): moves every channel by the same
/// amount so the luminosity (0.3 R + 0.59 G + 0.11 B) is `target`, then
/// pulls the channels toward it until they fit 0–1 (ClipColor).
fn set_lum(rgb: [f32; 3], target: f32) -> [f32; 3] {
    let d = target - lum601(rgb);
    let c = rgb.map(|v| v + d);
    let l = lum601(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    let mut c = c;
    if n < 0.0 {
        c = c.map(|v| l + (v - l) * l / (l - n));
    }
    if x > 1.0 {
        c = c.map(|v| l + (v - l) * (1.0 - l) / (x - l));
    }
    c
}

fn to_u8(rgb: [f32; 3], a: u8) -> [u8; 4] {
    let [r, g, b] = rgb.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
    [r, g, b, a]
}

/// The per-pixel adjustments that mix channels.
fn color_adjust(adjustment: Adjustment, px: [u8; 4]) -> [u8; 4] {
    let rgb = [px[0], px[1], px[2]].map(|v| v as f32 / 255.0);
    let lum = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2];
    match adjustment {
        Adjustment::ChannelMixer { rows, monochrome } => {
            let mix = |r: &[i32; 4]| {
                let v = px[0] as f32 * r[0] as f32
                    + px[1] as f32 * r[1] as f32
                    + px[2] as f32 * r[2] as f32;
                (v / 100.0 + r[3] as f32 * 2.55) / 255.0
            };
            let out = if monochrome {
                [mix(&rows[0]); 3]
            } else {
                [mix(&rows[0]), mix(&rows[1]), mix(&rows[2])]
            };
            let [r, g, b] = out.map(|v| round_half_up(v * 255.0));
            [r, g, b, px[3]]
        }
        Adjustment::SelectiveColor { colors, absolute } => selective_color(px, &colors, absolute),
        Adjustment::BlackWhite { weights, tint } => {
            // The gray is the darkest channel plus the primary and the
            // secondary hue's shares, each weighted
            let (r, g, b) = (rgb[0], rgb[1], rgb[2]);
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let mid = r + g + b - max - min;
            let w = |i: usize| weights[i] as f32 / 100.0;
            let primary = if max == r {
                w(0)
            } else if max == g {
                w(2)
            } else {
                w(4)
            };
            // Yellow (red+green), cyan (green+blue) or magenta (red+blue)
            let secondary = if min == b {
                w(1)
            } else if min == r {
                w(3)
            } else {
                w(5)
            };
            let gray = min + (mid - min) * secondary + (max - mid) * primary;
            let out = match tint {
                Some(t) => set_lum(t.map(|v| v as f32 / 255.0), gray.clamp(0.0, 1.0)),
                None => [gray; 3],
            };
            to_u8(out, px[3])
        }
        Adjustment::Vibrance {
            vibrance,
            saturation,
        } => {
            let [h, s, l] = rgb_to_hsl(rgb);
            let v = vibrance as f32 / 100.0;
            // Vibrance acts most on muted colors, and when raising
            // saturation spares skin tones (oranges about 25°), as
            // Photoshop's does (an approximation)
            let skin = (-((h - 25.0) / 20.0).powi(2)).exp();
            let v = if v > 0.0 { v * (1.0 - 0.7 * skin) } else { v };
            let s = (s * (1.0 + v * (1.0 - s))).clamp(0.0, 1.0);
            let rgb = hsl_to_rgb([h, s, l]);
            // Saturation scales the channels away from (or toward) a
            // linear-light gray of 0.2878 R + 0.7122 G, as Photoshop's
            let lin = rgb.map(srgb_to_linear);
            let gray = 0.2878 * lin[0] + 0.7122 * lin[1];
            let k = 1.0 + saturation as f32 / 100.0;
            to_u8(
                lin.map(|c| linear_to_srgb((gray + (c - gray) * k).clamp(0.0, 1.0))),
                px[3],
            )
        }
        Adjustment::PhotoFilter {
            color,
            density,
            preserve_luminosity,
        } => {
            // The filter multiplies X, Y and Z (D50) of the linear color,
            // each by 1 − density + density × the filter's own (relative to
            // white), as Photoshop does
            let d = density as f32 / 100.0;
            let filter = to_xyz(color.map(|v| srgb_to_linear(v as f32 / 255.0)));
            let white = to_xyz([1.0; 3]);
            let xyz = to_xyz(rgb.map(srgb_to_linear));
            let k = [0, 1, 2].map(|i| 1.0 - d + d * filter[i] / white[i]);
            let mut out = from_xyz([0, 1, 2].map(|i| xyz[i] * k[i]))
                .map(|v| linear_to_srgb(v.clamp(0.0, 1.0)));
            if preserve_luminosity {
                out = set_lum(out, lum601(rgb));
            }
            to_u8(out, px[3])
        }
        Adjustment::GradientMap { from, to, method } => {
            // Each pixel's luminosity picks its place along the gradient
            let [r, g, b] = crate::gradient::blend_colors(from, to, lum, method);
            [r, g, b, px[3]]
        }
        _ => px,
    }
}

/// Selective Color on one pixel (Photoshop 2026, fitted within a level):
/// each color range has a weight for the pixel — Reds, Greens, Blues: how
/// far the largest channel (it alone) is above the middle one; Cyans,
/// Magentas, Yellows: how far the middle is above the smallest (it alone);
/// Whites: twice how far the smallest is above 50%; Blacks: twice how far
/// the largest is below it; Neutrals: 1 − (|max − 50%| + |min − 50%|).
/// A channel's ink (1 − value) changes by weight × d with
/// d = amount + black × (1 + amount), times the ink itself in Relative
/// mode, each range's change kept within what the ink can take (−ink to
/// 1 − ink); the ranges' changes add up.
fn selective_color(px: [u8; 4], colors: &[[i32; 4]; 9], absolute: bool) -> [u8; 4] {
    let [r, g, b] = [px[0], px[1], px[2]].map(|v| v as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mid = r + g + b - max - min;
    let top = |v: f32, o1: f32, o2: f32| if v > o1 && v > o2 { max - mid } else { 0.0 };
    let bottom = |v: f32, o1: f32, o2: f32| if v < o1 && v < o2 { mid - min } else { 0.0 };
    let weights = [
        top(r, g, b),
        bottom(b, r, g),
        top(g, r, b),
        bottom(r, g, b),
        top(b, r, g),
        bottom(g, r, b),
        ((min - 0.5) * 2.0).max(0.0),
        1.0 - ((max - 0.5).abs() + (min - 0.5).abs()),
        ((0.5 - max) * 2.0).max(0.0),
    ];
    let mut out = [px[0], px[1], px[2], px[3]];
    for (c, v) in [r, g, b].into_iter().enumerate() {
        let ink = 1.0 - v;
        let mut change = 0.0;
        for (w, settings) in weights.iter().zip(colors) {
            if *w <= 0.0 {
                continue;
            }
            let a = settings[c] as f32 / 100.0;
            let k = settings[3] as f32 / 100.0;
            let mut d = a + k * (1.0 + a);
            if !absolute {
                d *= ink;
            }
            change += w * d.clamp(-ink, 1.0 - ink);
        }
        out[c] = round_half_up((1.0 - (ink + change).clamp(0.0, 1.0)) * 255.0);
    }
    out
}

/// Auto Tone / Auto Contrast lookup tables: a channel's range without its
/// darkest and lightest 0.1% stretched to 0–255.
fn auto_tables(doc: &Document, per_channel: bool) -> [[u8; 256]; 3] {
    let mut hists = [[0u64; 256]; 3];
    if let Some(image) = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .and_then(|l| l.image())
    {
        let selection = doc.selection();
        for y in 0..doc.height {
            for x in 0..doc.width {
                if selection.is_some_and(|s| s.get(x, y) == 0) {
                    continue;
                }
                let px = image.pixel(x, y);
                if px[3] == 0 {
                    continue;
                }
                for c in 0..3 {
                    hists[c][px[c] as usize] += 1;
                }
            }
        }
    }
    let range = |hist: &[u64; 256]| -> (usize, usize) {
        let total: u64 = hist.iter().sum();
        let clip = total / 1000;
        let mut acc = 0;
        let lo = (0..256).find(|&i| {
            acc += hist[i];
            acc > clip
        });
        acc = 0;
        let hi = (0..256).rev().find(|&i| {
            acc += hist[i];
            acc > clip
        });
        (lo.unwrap_or(0), hi.unwrap_or(255))
    };
    let table = |(lo, hi): (usize, usize)| {
        let mut t = [0u8; 256];
        for (i, v) in t.iter_mut().enumerate() {
            *v = if hi <= lo {
                i as u8
            } else {
                ((i as f32 - lo as f32) / (hi - lo) as f32 * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
        }
        t
    };
    if per_channel {
        hists.map(|h| table(range(&h)))
    } else {
        let mut all = [0u64; 256];
        for h in &hists {
            for (a, v) in all.iter_mut().zip(h) {
                *a += v;
            }
        }
        [table(range(&all)); 3]
    }
}

/// One of Hue/Saturation's six color ranges: where it starts fading in,
/// where it is at full strength, where full strength ends and where it has
/// faded out (degrees, going around the hue circle), and its settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HueRange {
    pub bounds: [i32; 4],
    pub hue: i32,
    pub saturation: i32,
    pub lightness: i32,
}

/// Photoshop's ranges for Reds, Yellows, Greens, Cyans, Blues and Magentas.
pub const HUE_RANGES: [[i32; 4]; 6] = [
    [315, 345, 15, 45],
    [15, 45, 75, 105],
    [75, 105, 135, 165],
    [135, 165, 195, 225],
    [195, 225, 255, 285],
    [255, 285, 315, 345],
];

impl HueRange {
    /// How much of the range's adjustment a hue gets, 0–1. The fades are
    /// linear, half a degree later than the bounds say (as Photoshop's
    /// results come out).
    pub fn weight(&self, hue: f32) -> f32 {
        let [a, b, c, d] = self.bounds.map(|v| v as f32);
        let around = |v: f32| (v - a).rem_euclid(360.0);
        let (t, b, c, d) = (around(hue), around(b), around(c), around(d));
        if t < b {
            ((t - 0.5) / b).max(0.0)
        } else if t <= c {
            1.0
        } else if t < d {
            ((d - t + 0.5) / (d - c)).min(1.0)
        } else {
            0.0
        }
    }

    fn is_neutral(&self) -> bool {
        self.hue == 0 && self.saturation == 0 && self.lightness == 0
    }
}

/// Hue/Saturation's settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HueSaturation {
    /// Master hue (−180–180°, or 0–360° with Colorize), saturation
    /// (−100–100, or 0–100 with Colorize) and lightness (−100–100).
    pub master: [i32; 3],
    pub ranges: [HueRange; 6],
    pub colorize: bool,
}

impl HueSaturation {
    /// Master settings only.
    pub fn master(hue: i32, saturation: i32, lightness: i32) -> Self {
        Self {
            master: [hue, saturation, lightness],
            ranges: HUE_RANGES.map(|bounds| HueRange {
                bounds,
                hue: 0,
                saturation: 0,
                lightness: 0,
            }),
            colorize: false,
        }
    }

    /// Photoshop's Hue/Saturation on one pixel (fitted to Photoshop 2026
    /// within a few levels). Each range's settings count by the range's
    /// weight at the pixel's original hue. The hue turns by the master's
    /// plus the ranges' turn, keeping the largest and smallest channel;
    /// master lightness mixes toward white or black; the ranges' lightness
    /// instead moves the channels toward the largest (up) or the smallest
    /// (down) one; then the ranges' saturation and after it the master's
    /// push the channels away from (or pull them toward) their HSL
    /// lightness. Colorize paints the pixel's HSL lightness with the master
    /// hue and saturation.
    pub fn apply(&self, px: [u8; 4]) -> [u8; 4] {
        let rgb = [px[0], px[1], px[2]].map(|v| v as f32);
        let [hue, sat, light] = self.master.map(|v| v as f32);
        let lighten = |v: f32| {
            if light >= 0.0 {
                v + (255.0 - v) * light / 100.0
            } else {
                v * (1.0 + light / 100.0)
            }
        };
        let out = if self.colorize {
            let max = rgb[0].max(rgb[1]).max(rgb[2]);
            let min = rgb[0].min(rgb[1]).min(rgb[2]);
            let l = lighten((max + min) / 2.0) / 255.0;
            hsl_to_rgb([hue, sat / 100.0, l]).map(|v| v * 255.0)
        } else {
            let max = rgb[0].max(rgb[1]).max(rgb[2]);
            let min = rgb[0].min(rgb[1]).min(rgb[2]);
            let (mut dh, mut ds, mut dl) = (hue, 0.0, 0.0);
            if max > min {
                let h = rgb_to_hsl(rgb.map(|v| v / 255.0))[0];
                for r in self.ranges.iter().filter(|r| !r.is_neutral()) {
                    let w = r.weight(h);
                    dh += w * r.hue as f32;
                    ds += w * r.saturation as f32;
                    dl += w * r.lightness as f32;
                }
            }
            let mut c = turn_hue(rgb, dh).map(lighten);
            if dl != 0.0 {
                let max = c[0].max(c[1]).max(c[2]);
                let min = c[0].min(c[1]).min(c[2]);
                let k = dl / 100.0;
                c = c.map(|v| {
                    if k > 0.0 {
                        v + (max - v) * k
                    } else {
                        min + (v - min) * (1.0 + k)
                    }
                });
            }
            let c = saturate(c, ds.clamp(-100.0, 100.0) / 100.0);
            saturate(c, sat / 100.0)
        };
        let [r, g, b] = out.map(round_half_up);
        [r, g, b, px[3]]
    }
}

/// An HSL color (hue in degrees, saturation and lightness 0–1) as RGB.
pub fn hsl_color(hue: f32, saturation: f32, lightness: f32) -> [u8; 3] {
    hsl_to_rgb([hue, saturation, lightness]).map(|v| round_half_up(v * 255.0))
}

/// The HSL hue (degrees) of an RGB color; 0 for grays.
pub fn hue_of([r, g, b]: [u8; 3]) -> f32 {
    rgb_to_hsl([r, g, b].map(|v| v as f32 / 255.0))[0]
}

/// Turns the hue by `degrees`, keeping the largest and smallest channel.
fn turn_hue(rgb: [f32; 3], degrees: f32) -> [f32; 3] {
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    let min = rgb[0].min(rgb[1]).min(rgb[2]);
    if degrees == 0.0 || max == min {
        return rgb;
    }
    let h = rgb_to_hsl(rgb.map(|v| v / 255.0))[0];
    let h = (h + degrees).rem_euclid(360.0) / 60.0;
    let sector = (h as usize).min(5);
    let f = h - sector as f32;
    let (up, down) = (min + (max - min) * f, max - (max - min) * f);
    match sector {
        0 => [max, up, min],
        1 => [down, max, min],
        2 => [min, max, up],
        3 => [min, down, max],
        4 => [up, min, max],
        _ => [max, min, down],
    }
}

/// Photoshop's saturation step (−1–1): above 0 the channels move away from
/// their HSL lightness by 1/a − 1, where a is 1 − amount, or the color's
/// own saturation once amount + saturation reaches 1 (full saturation);
/// below 0 they move toward it.
fn saturate(rgb: [f32; 3], amount: f32) -> [f32; 3] {
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    let min = rgb[0].min(rgb[1]).min(rgb[2]);
    if amount == 0.0 || max == min {
        return rgb;
    }
    let l = (max + min) / 2.0;
    if amount < 0.0 {
        return rgb.map(|v| l + (v - l) * (1.0 + amount));
    }
    let (d, lf) = ((max - min) / 255.0, l / 255.0);
    let s = if lf < 0.5 {
        d / (2.0 * lf)
    } else {
        d / (2.0 - 2.0 * lf)
    };
    let a = if amount + s >= 1.0 { s } else { 1.0 - amount };
    let k = 1.0 / a - 1.0;
    rgb.map(|v| v + (v - l) * k)
}

/// The gray a color paints on a layer mask (its luminosity).
pub fn mask_gray([r, g, b]: [u8; 3]) -> [u8; 3] {
    let l = luminosity([r, g, b, 255]);
    [l, l, l]
}

/// Luminosity on Photoshop's 0–255 scale (Rec. 601 weights).
pub fn luminosity([r, g, b, _]: [u8; 4]) -> u8 {
    ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114 + 500) / 1000) as u8
}

fn posterize(v: u8, levels: u8) -> u8 {
    let steps = (levels.max(2) - 1) as f32;
    let level = (v as f32 * steps / 255.0).round();
    (level * 255.0 / steps).round() as u8
}

/// Histogram of the brightness values (all three channels together) of the
/// active layer's selected, non-transparent pixels.
pub fn channel_histogram(doc: &Document) -> [u64; 256] {
    let mut hist = [0u64; 256];
    let Some(layer) = doc.active_layer.and_then(|id| doc.layer(id)) else {
        return hist;
    };
    let Some(image) = layer.image() else {
        return hist;
    };
    let selection = doc.selection();
    for y in 0..doc.height {
        for x in 0..doc.width {
            if selection.is_some_and(|s| s.get(x, y) == 0) {
                continue;
            }
            let px = image.pixel(x, y);
            if px[3] == 0 {
                continue;
            }
            for &c in &px[..3] {
                hist[c as usize] += 1;
            }
        }
    }
    hist
}

/// Histograms of the red, green and blue channels of the active layer's
/// selected, non-transparent pixels (Levels and Curves show them).
pub fn rgb_histograms(doc: &Document) -> [[u64; 256]; 3] {
    let mut hists = [[0u64; 256]; 3];
    let Some(image) = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .and_then(|l| l.image())
    else {
        return hists;
    };
    let selection = doc.selection();
    for y in 0..doc.height {
        for x in 0..doc.width {
            if selection.is_some_and(|s| s.get(x, y) == 0) {
                continue;
            }
            let px = image.pixel(x, y);
            if px[3] == 0 {
                continue;
            }
            for c in 0..3 {
                hists[c][px[c] as usize] += 1;
            }
        }
    }
    hists
}

/// Equalize's lookup table: each value maps to its place in the cumulative
/// histogram.
fn equalize_table(hist: &[u64; 256]) -> [u8; 256] {
    let total: u64 = hist.iter().sum();
    let mut table = [0u8; 256];
    if total == 0 {
        for (i, t) in table.iter_mut().enumerate() {
            *t = i as u8;
        }
        return table;
    }
    let mut cumulative = 0u64;
    for (i, &count) in hist.iter().enumerate() {
        cumulative += count;
        table[i] = ((cumulative * 255 + total / 2) / total) as u8;
    }
    table
}

/// Histogram of the active layer's luminosity inside the selection, as the
/// Threshold dialog shows it.
pub fn luminosity_histogram(doc: &Document) -> [u64; 256] {
    let mut hist = [0u64; 256];
    let Some(layer) = doc.active_layer.and_then(|id| doc.layer(id)) else {
        return hist;
    };
    let Some(image) = layer.image() else {
        return hist;
    };
    let selection = doc.selection();
    for y in 0..doc.height {
        for x in 0..doc.width {
            if selection.is_some_and(|s| s.get(x, y) == 0) {
                continue;
            }
            let px = image.pixel(x, y);
            if px[3] > 0 {
                hist[luminosity(px) as usize] += 1;
            }
        }
    }
    hist
}

/// The checks every adjustment makes before changing pixels; the messages
/// match Photoshop's ("Could not complete the Invert command because the
/// target layer is hidden.").
pub fn check(doc: &Document) -> Result<(), FillError> {
    let layer = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .ok_or(FillError::NoLayer)?;
    if !layer.visible {
        return Err(FillError::Hidden);
    }
    if doc.pixels_locked(layer.id) {
        return Err(FillError::Locked);
    }
    if layer.is_group() {
        return Err(FillError::Group);
    }
    Ok(())
}

/// Applies `adjustment` to the active layer. Partly selected pixels get a
/// proportional mix of the old and new color; alpha is never changed.
pub fn apply(doc: &mut Document, adjustment: Adjustment) -> Result<(), FillError> {
    check(doc)?;
    let tables = match adjustment {
        Adjustment::Equalize | Adjustment::EqualizeEntireImage => {
            Some([equalize_table(&channel_histogram(doc)); 3])
        }
        Adjustment::AutoTone | Adjustment::AutoColor => Some(auto_tables(doc, true)),
        Adjustment::AutoContrast => Some(auto_tables(doc, false)),
        other => other.tables(),
    };
    let map = |px: [u8; 4]| -> [u8; 4] {
        let [r, g, b, a] = px;
        match adjustment {
            Adjustment::Invert => [255 - r, 255 - g, 255 - b, a],
            Adjustment::Desaturate => {
                let l = (r.max(g).max(b) as u16 + r.min(g).min(b) as u16).div_ceil(2) as u8;
                [l, l, l, a]
            }
            Adjustment::Threshold(level) => {
                let v = if luminosity(px) >= level { 255 } else { 0 };
                [v, v, v, a]
            }
            Adjustment::Posterize(levels) => [
                posterize(r, levels),
                posterize(g, levels),
                posterize(b, levels),
                a,
            ],
            Adjustment::Equalize
            | Adjustment::EqualizeEntireImage
            | Adjustment::Levels(_)
            | Adjustment::Exposure { .. }
            | Adjustment::BrightnessContrast { .. }
            | Adjustment::ColorBalance { .. }
            | Adjustment::Curves { .. }
            | Adjustment::CurveTables(_)
            | Adjustment::AutoTone
            | Adjustment::AutoContrast
            | Adjustment::AutoColor => {
                let t = tables.as_ref().expect("computed above");
                [t[0][r as usize], t[1][g as usize], t[2][b as usize], a]
            }
            Adjustment::BlackWhite { .. }
            | Adjustment::ChannelMixer { .. }
            | Adjustment::SelectiveColor { .. }
            | Adjustment::Vibrance { .. }
            | Adjustment::PhotoFilter { .. }
            | Adjustment::GradientMap { .. } => color_adjust(adjustment, px),
            Adjustment::HueSaturation(hs) => hs.apply(px),
        }
    };
    let selection = match adjustment {
        Adjustment::EqualizeEntireImage => None,
        _ => doc.selection().cloned(),
    };
    let (w, h) = (doc.width, doc.height);
    let id = doc.active_layer.expect("checked");
    let image = doc
        .layer_mut(id)
        .and_then(|l| l.image_mut())
        .expect("checked: not a group");
    for y in 0..h {
        for x in 0..w {
            let amount = selection.as_ref().map_or(255, |s| s.get(x, y));
            if amount == 0 {
                continue;
            }
            let old = image.pixel(x, y);
            if old[3] == 0 {
                continue;
            }
            let mut new = map(old);
            if amount < 255 {
                for c in 0..3 {
                    let (o, n) = (old[c] as u32, new[c] as u32);
                    new[c] = ((o * (255 - amount as u32) + n * amount as u32 + 127) / 255) as u8;
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
    use super::*;
    use crate::color::Color;
    use crate::selection::{Rect, Selection};

    fn doc(px: [u8; 4]) -> Document {
        let mut doc = Document::new_with_background("t", 2, 1, Color::WHITE);
        let id = doc.active_layer.unwrap();
        let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(0, 0, px);
        doc
    }

    fn first(doc: &Document) -> [u8; 4] {
        doc.composite_rgba8()[..4].try_into().unwrap()
    }

    #[test]
    fn invert_desaturate_threshold_posterize() {
        let mut d = doc([200, 100, 0, 255]);
        apply(&mut d, Adjustment::Invert).unwrap();
        assert_eq!(first(&d), [55, 155, 255, 255]);

        let mut d = doc([200, 100, 0, 255]);
        apply(&mut d, Adjustment::Desaturate).unwrap();
        assert_eq!(first(&d), [100, 100, 100, 255]);

        // Luminosity of (200, 100, 0) is 118.5 -> 119
        let mut d = doc([200, 100, 0, 255]);
        apply(&mut d, Adjustment::Threshold(119)).unwrap();
        assert_eq!(first(&d), [255, 255, 255, 255]);
        let mut d = doc([200, 100, 0, 255]);
        apply(&mut d, Adjustment::Threshold(120)).unwrap();
        assert_eq!(first(&d), [0, 0, 0, 255]);

        let mut d = doc([200, 100, 30, 255]);
        apply(&mut d, Adjustment::Posterize(2)).unwrap();
        assert_eq!(first(&d), [255, 0, 0, 255]);
        let mut d = doc([200, 100, 30, 255]);
        apply(&mut d, Adjustment::Posterize(4)).unwrap();
        assert_eq!(first(&d), [170, 85, 0, 255]);
    }

    #[test]
    fn equalize_stretches_the_range() {
        // Values 100 and 200 only: equalized to the middle and the top
        let mut d = doc([100, 100, 100, 255]);
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(1, 0, [200, 200, 200, 255]);
        apply(&mut d, Adjustment::Equalize).unwrap();
        assert_eq!(first(&d), [128, 128, 128, 255]);
        assert_eq!(&d.composite_rgba8()[4..8], [255, 255, 255, 255]);
    }

    #[test]
    fn levels_hue_saturation_and_exposure() {
        let levels = Adjustment::Levels(
            Levels {
                input_black: 50,
                input_white: 200,
                ..Levels::IDENTITY
            }
            .composite(),
        );
        let mut d = doc([125, 50, 220, 255]);
        apply(&mut d, levels).unwrap();
        assert_eq!(first(&d), [128, 0, 255, 255]);
        // Gamma 2 brightens the midtones: (128/255)^(1/2) * 255 = 180.7
        // (the toe near black does not reach 128)
        let mut d = doc([128, 128, 128, 255]);
        let gamma = Adjustment::Levels(
            Levels {
                gamma: 2.0,
                ..Levels::IDENTITY
            }
            .composite(),
        );
        apply(&mut d, gamma).unwrap();
        assert_eq!(first(&d), [181, 181, 181, 255]);

        // Red turned by 120° is green; saturation −100 is gray
        let mut d = doc([255, 0, 0, 255]);
        let hue = Adjustment::HueSaturation(HueSaturation::master(120, 0, 0));
        apply(&mut d, hue).unwrap();
        assert_eq!(first(&d), [0, 255, 0, 255]);
        let mut d = doc([255, 0, 0, 255]);
        let gray = Adjustment::HueSaturation(HueSaturation::master(0, -100, 50));
        apply(&mut d, gray).unwrap();
        assert_eq!(first(&d), [191, 191, 191, 255]);

        // One stop up doubles linear light (gamma 2.2): 128 -> 175.4
        let mut d = doc([128, 128, 128, 255]);
        let exposure = Adjustment::Exposure {
            exposure: 1.0,
            offset: 0.0,
            gamma: 1.0,
        };
        apply(&mut d, exposure).unwrap();
        assert_eq!(first(&d), [175, 175, 175, 255]);
    }

    #[test]
    fn color_adjustments() {
        // Black & White with the default weights: pure red at 40% is 102
        let mut d = doc([255, 0, 0, 255]);
        let bw = Adjustment::BlackWhite {
            weights: [40, 60, 40, 60, 20, 80],
            tint: None,
        };
        apply(&mut d, bw).unwrap();
        assert_eq!(first(&d), [102, 102, 102, 255]);
        // Yellow uses the yellows weight
        let mut d = doc([255, 255, 0, 255]);
        apply(&mut d, bw).unwrap();
        assert_eq!(first(&d)[0], 153);

        // Gradient map: black to red
        let mut d = doc([255, 255, 255, 255]);
        let gm = Adjustment::GradientMap {
            from: [0, 0, 0],
            to: [255, 0, 0],
            method: crate::gradient::Method::Classic,
        };
        apply(&mut d, gm).unwrap();
        assert_eq!(first(&d), [255, 0, 0, 255]);

        // ...and spares skin tones: a muted orange gains less than an
        // equally muted blue
        let gain = |rgb: [u8; 3]| {
            let vib = Adjustment::Vibrance {
                vibrance: 100,
                saturation: 0,
            };
            let mut d = doc([rgb[0], rgb[1], rgb[2], 255]);
            apply(&mut d, vib).unwrap();
            let out = first(&d);
            let s0 = rgb_to_hsl(rgb.map(|v| v as f32 / 255.0))[1];
            let s1 = rgb_to_hsl([out[0], out[1], out[2]].map(|v| v as f32 / 255.0))[1];
            s1 - s0
        };
        assert!(gain([200, 160, 140]) < gain([140, 160, 200]) * 0.6);
        // Vibrance saturates a muted color more than a vivid one
        let mut muted = doc([140, 120, 120, 255]);
        let vib = Adjustment::Vibrance {
            vibrance: 100,
            saturation: 0,
        };
        apply(&mut muted, vib).unwrap();
        assert!(first(&muted)[0] - first(&muted)[1] > 20);

        // Photo filter: a blue filter takes red out, multiplying in linear
        // light (200 is 0.578; half of it is 146)
        let mut d = doc([200, 200, 200, 255]);
        let pf = Adjustment::PhotoFilter {
            color: [0, 0, 255],
            density: 50,
            preserve_luminosity: false,
        };
        apply(&mut d, pf).unwrap();
        assert_eq!(first(&d), [146, 146, 200, 255]);
    }

    #[test]
    fn curves_pass_through_their_points() {
        let identity = curve_table(&[(0, 0), (255, 255)]);
        assert!(identity.iter().enumerate().all(|(i, &v)| v == i as u8));
        // An S-curve through (64, 40) and (192, 215) keeps its points and
        // stays smooth and increasing
        let s = curve_table(&[(0, 0), (64, 40), (192, 215), (255, 255)]);
        assert_eq!((s[64], s[192]), (40, 215));
        assert!(s.windows(2).all(|w| w[1] >= w[0]));
        // Beyond the end points the curve is flat
        let clipped = curve_table(&[(30, 0), (220, 255)]);
        assert_eq!((clipped[10], clipped[240]), (0, 255));
        let mut d = doc([64, 64, 64, 255]);
        apply(&mut d, Adjustment::curves(&[(0, 0), (64, 128), (255, 255)])).unwrap();
        assert_eq!(first(&d), [128, 128, 128, 255]);
    }

    #[test]
    fn auto_tone_stretches_each_channel() {
        let mut d = doc([50, 100, 150, 255]);
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(1, 0, [100, 200, 250, 255]);
        apply(&mut d, Adjustment::AutoTone).unwrap();
        assert_eq!(first(&d), [0, 0, 0, 255]);
        assert_eq!(&d.composite_rgba8()[4..8], [255, 255, 255, 255]);
    }

    #[test]
    fn channel_histograms() {
        let d = doc([10, 20, 30, 255]);
        let [r, g, b] = rgb_histograms(&d);
        // The second pixel is the white background
        assert_eq!((r[10], g[20], b[30], r[255]), (1, 1, 1, 1));
        let merged = channel_histogram(&d);
        assert_eq!((merged[10], merged[255]), (1, 3));
    }

    #[test]
    fn equalize_the_entire_image_from_the_selection() {
        // Selected: the 100 pixel; the 200 one outside is equalized too,
        // by the selection's table (everything at or above 100 is white)
        let mut d = doc([100, 100, 100, 255]);
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(1, 0, [200, 200, 200, 255]);
        d.set_selection(Some(Selection::rect(2, 1, Rect::new(0.0, 0.0, 1.0, 1.0))));
        let mut selected_only = doc([100, 100, 100, 255]);
        let other = selected_only.active_layer.unwrap();
        let image = selected_only.layer_mut(other).unwrap().image_mut().unwrap();
        image.set_pixel(1, 0, [200, 200, 200, 255]);
        selected_only.set_selection(d.selection().cloned());
        apply(&mut d, Adjustment::EqualizeEntireImage).unwrap();
        assert_eq!(first(&d), [255, 255, 255, 255]);
        assert_eq!(&d.composite_rgba8()[4..8], [255, 255, 255, 255]);
        // Selected area only: the outside pixel keeps its 200
        apply(&mut selected_only, Adjustment::Equalize).unwrap();
        assert_eq!(&selected_only.composite_rgba8()[4..8], [200, 200, 200, 255]);
    }

    #[test]
    fn only_the_selection_changes() {
        let mut d = doc([0, 0, 0, 255]);
        d.set_selection(Some(Selection::rect(2, 1, Rect::new(1.0, 0.0, 2.0, 1.0))));
        apply(&mut d, Adjustment::Invert).unwrap();
        assert_eq!(first(&d), [0, 0, 0, 255]);
        assert_eq!(&d.composite_rgba8()[4..8], [0, 0, 0, 255]);
    }

    #[test]
    fn hidden_layers_are_refused() {
        let mut d = doc([0, 0, 0, 255]);
        d.layers[0].visible = false;
        let e = apply(&mut d, Adjustment::Invert).unwrap_err();
        assert_eq!(
            e.message("Invert"),
            "Could not complete the Invert command because the target layer is hidden."
        );
    }

    /// Comparisons with Photoshop 2026's own results (fixtures/adjust):
    /// probe.rgb is a 64 × 72 image (a 16-level RGB cube, a gray ramp and
    /// random colors), the other .rgb files are Photoshop's output.
    mod photoshop {
        use super::*;

        const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/adjust/");

        fn read(name: &str) -> Vec<u8> {
            std::fs::read(format!("{FIXTURES}{name}")).unwrap()
        }

        /// Applies `adjustment` to the probe image and returns the largest
        /// channel difference from Photoshop's `expected` output and how
        /// many channels differ by more than 1.
        fn compare(adjustment: Adjustment, expected: &str) -> (u8, usize) {
            let (w, h) = (64, 72);
            let probe = read("probe.rgb");
            let want = read(expected);
            let mut doc = Document::new_with_background("t", w, h, Color::WHITE);
            let id = doc.active_layer.unwrap();
            let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
            for (i, px) in probe.chunks(3).enumerate() {
                image.set_pixel(i as u32 % w, i as u32 / w, [px[0], px[1], px[2], 255]);
            }
            apply(&mut doc, adjustment).unwrap();
            let got = doc.composite_rgba8();
            let mut worst = 0u8;
            let mut off = 0;
            for (i, want) in want.chunks(3).enumerate() {
                for c in 0..3 {
                    let d = got[i * 4 + c].abs_diff(want[c]);
                    worst = worst.max(d);
                    off += (d > 1) as usize;
                }
            }
            (worst, off)
        }

        fn ranges(edits: &[(usize, [i32; 3])]) -> [HueRange; 6] {
            let mut ranges = HueSaturation::master(0, 0, 0).ranges;
            for &(i, [hue, saturation, lightness]) in edits {
                ranges[i] = HueRange {
                    hue,
                    saturation,
                    lightness,
                    ..ranges[i]
                };
            }
            ranges
        }

        #[test]
        fn hue_saturation_master_matches_photoshop() {
            for (name, h, s, l, worst_allowed) in [
                ("hs_s50.rgb", 0, 50, 0, 2),
                ("hs_sm50.rgb", 0, -50, 0, 1),
                ("hs_h60.rgb", 60, 0, 0, 1),
                ("hs_hm30.rgb", -30, 0, 0, 2),
                ("hs_l50.rgb", 0, 0, 50, 2),
                ("hs_lm50.rgb", 0, 0, -50, 1),
                ("hs_mix.rgb", 40, 30, -20, 4),
            ] {
                let adj = Adjustment::HueSaturation(HueSaturation::master(h, s, l));
                let (worst, _) = compare(adj, name);
                assert!(worst <= worst_allowed, "{name}: off by {worst}");
            }
        }

        #[test]
        fn hue_saturation_colorize_and_ranges_match_photoshop() {
            let colorize = HueSaturation {
                colorize: true,
                ..HueSaturation::master(200, 40, 10)
            };
            let (worst, _) = compare(Adjustment::HueSaturation(colorize), "hs_c.rgb");
            assert!(worst <= 3, "colorize: off by {worst}");
            for (name, master, edits) in [
                ("hr_red_s.rgb", [0, 0, 0], vec![(0, [0, -100, 0])]),
                ("hr_red_hm.rgb", [30, 0, 0], vec![(0, [60, 0, 0])]),
                ("hr_red_l.rgb", [0, 0, 0], vec![(0, [0, 0, 50])]),
                ("hr_red_sm.rgb", [0, 50, 0], vec![(0, [0, -50, 0])]),
                ("hr_blue_l.rgb", [0, 0, -30], vec![(4, [0, 0, -50])]),
            ] {
                let hs = HueSaturation {
                    ranges: ranges(&edits),
                    ..HueSaturation::master(master[0], master[1], master[2])
                };
                // The range edges land within a few levels of Photoshop's
                let (worst, off) = compare(Adjustment::HueSaturation(hs), name);
                assert!(
                    worst <= 5 && off < 400,
                    "{name}: off by {worst} ({off} channels)"
                );
            }
        }

        /// A 256 × 1 gray ramp through `table`'s red channel.
        fn ramp(adjustment: Adjustment) -> [[u8; 256]; 3] {
            adjustment.tables().unwrap()
        }

        #[test]
        fn levels_match_photoshop() {
            let data = read("levels.bin");
            for (row, &(ib, iw, gamma, ob, ow)) in LEVELS_ROWS.iter().enumerate() {
                let levels = Levels {
                    input_black: ib,
                    input_white: iw,
                    gamma,
                    output_black: ob,
                    output_white: ow,
                };
                let table = levels.table();
                let want = &data[row * 256..row * 256 + 256];
                let worst = (0..256).map(|i| table[i].abs_diff(want[i])).max().unwrap();
                // Photoshop's toe for gammas 5–7 is a few levels off ours
                // in the darkest values only
                let allowed = if (5.5..7.5).contains(&gamma) { 6 } else { 2 };
                assert!(worst <= allowed, "levels {row}: off by {worst}");
            }
        }

        #[test]
        fn channel_levels_apply_before_the_composite() {
            // Photoshop: composite output white 128 after red input black
            // 128 turns 160 into 32
            let mut levels = Levels {
                output_white: 128,
                ..Levels::IDENTITY
            }
            .composite();
            levels[1] = Levels {
                input_black: 128,
                ..Levels::IDENTITY
            };
            let t = ramp(Adjustment::Levels(levels));
            assert_eq!((t[0][160], t[1][160], t[0][255]), (32, 80, 128));
        }

        #[test]
        fn brightness_contrast_matches_photoshop() {
            let data = read("brightness_contrast.bin");
            for (row, &(legacy, brightness, contrast)) in
                BRIGHTNESS_CONTRAST_ROWS.iter().enumerate()
            {
                let table = brightness_contrast_table(brightness, contrast, legacy);
                let want = &data[row * 256..row * 256 + 256];
                let worst = (0..256).map(|i| table[i].abs_diff(want[i])).max().unwrap();
                // Legacy contrast at 25, 99 and with a fractional stretch
                // rounds a level apart from Photoshop's at most
                let allowed = if legacy && contrast == 99 { 15 } else { 1 };
                assert!(
                    worst <= allowed,
                    "{legacy} {brightness} {contrast}: off by {worst}"
                );
            }
            // The tables themselves: brightness 0 and contrast 0 change nothing
            assert_eq!(brightness_contrast_table(0, 0, false), IDENTITY);
            assert_eq!(brightness_contrast_table(50, 0, false)[128], 171);
        }

        #[test]
        fn color_balance_matches_photoshop() {
            let data = read("color_balance.rgb");
            for (row, &(tones, preserve)) in COLOR_BALANCE_ROWS.iter().enumerate() {
                let t = ramp(Adjustment::ColorBalance {
                    shadows: tones[0],
                    midtones: tones[1],
                    highlights: tones[2],
                    preserve_luminosity: preserve,
                });
                for i in 0..256 {
                    for c in 0..3 {
                        let want = data[(row * 256 + i) * 3 + c];
                        let d = t[c][i].abs_diff(want);
                        assert!(
                            d <= 3,
                            "row {row} channel {c} at {i}: {} vs {want}",
                            t[c][i]
                        );
                    }
                }
            }
        }

        #[test]
        fn channel_mixer_matches_photoshop() {
            for (name, rows, monochrome) in [
                (
                    "cm2.rgb",
                    [[50, 30, 20, 0], [-40, 120, 10, 10], [0, 0, 100, -20]],
                    false,
                ),
                ("cm3.rgb", [[40, 40, 20, 0], [0; 4], [0; 4]], true),
                (
                    "cm4.rgb",
                    [[200, -50, 0, 5], [0, 100, 0, 0], [30, 30, 30, 0]],
                    false,
                ),
            ] {
                let (worst, _) = compare(Adjustment::ChannelMixer { rows, monochrome }, name);
                assert!(worst <= 1, "{name}: off by {worst}");
            }
        }

        #[test]
        fn selective_color_matches_photoshop() {
            let mut colors = [[0; 4]; 9];
            let set = |colors: &mut [[i32; 4]; 9], i: usize, v: [i32; 4]| colors[i] = v;
            set(&mut colors, 0, [60, -40, 30, 20]);
            set(&mut colors, 7, [30, 20, -50, -10]);
            set(&mut colors, 6, [-60, 0, 40, 0]);
            set(&mut colors, 1, [0, 80, -30, 10]);
            let (worst, _) = compare(
                Adjustment::SelectiveColor {
                    colors,
                    absolute: false,
                },
                "sc_multi.rgb",
            );
            assert!(worst <= 1, "relative: off by {worst}");
            let mut colors = [[0; 4]; 9];
            set(&mut colors, 0, [60, -40, 30, 20]);
            set(&mut colors, 7, [30, 20, -50, -10]);
            set(&mut colors, 8, [0, 30, 0, 50]);
            set(&mut colors, 3, [-70, 0, 40, 0]);
            let (worst, _) = compare(
                Adjustment::SelectiveColor {
                    colors,
                    absolute: true,
                },
                "sc_multi_abs.rgb",
            );
            assert!(worst <= 1, "absolute: off by {worst}");
            for (name, i, v, absolute) in [
                ("sc_n_abs.rgb", 7, [20, -20, 0, 30], true),
                ("sc_k.rgb", 8, [0, 0, 0, 40], true),
                ("sc_y.rgb", 1, [100, 0, 0, 0], false),
            ] {
                let mut colors = [[0; 4]; 9];
                colors[i] = v;
                let (worst, _) = compare(Adjustment::SelectiveColor { colors, absolute }, name);
                assert!(worst <= 1, "{name}: off by {worst}");
            }
        }

        #[test]
        fn black_white_photo_filter_and_exposure_match_photoshop() {
            let defaults = [40, 60, 40, 60, 20, 80];
            for (name, adjustment, allowed) in [
                (
                    "bw_x.rgb",
                    Adjustment::BlackWhite {
                        weights: [-50, 150, 200, 10, 300, -100],
                        tint: None,
                    },
                    1,
                ),
                (
                    "bw_t.rgb",
                    Adjustment::BlackWhite {
                        weights: defaults,
                        tint: Some([225, 211, 179]),
                    },
                    2,
                ),
                (
                    "bw_t2.rgb",
                    Adjustment::BlackWhite {
                        weights: defaults,
                        tint: Some([40, 90, 200]),
                    },
                    2,
                ),
                (
                    "pf_w.rgb",
                    Adjustment::PhotoFilter {
                        color: [236, 138, 0],
                        density: 25,
                        preserve_luminosity: true,
                    },
                    6,
                ),
                (
                    "pf_w2.rgb",
                    Adjustment::PhotoFilter {
                        color: [236, 138, 0],
                        density: 60,
                        preserve_luminosity: false,
                    },
                    1,
                ),
                (
                    "pf_b.rgb",
                    Adjustment::PhotoFilter {
                        color: [0, 0, 255],
                        density: 50,
                        preserve_luminosity: false,
                    },
                    1,
                ),
                (
                    "pf_b2.rgb",
                    Adjustment::PhotoFilter {
                        color: [0, 0, 255],
                        density: 80,
                        preserve_luminosity: true,
                    },
                    6,
                ),
                (
                    "ex_1.rgb",
                    Adjustment::Exposure {
                        exposure: 1.0,
                        offset: 0.0,
                        gamma: 1.0,
                    },
                    2,
                ),
                (
                    "ex_m.rgb",
                    Adjustment::Exposure {
                        exposure: -1.5,
                        offset: 0.05,
                        gamma: 1.0,
                    },
                    2,
                ),
                (
                    "ex_g.rgb",
                    Adjustment::Exposure {
                        exposure: 0.0,
                        offset: 0.0,
                        gamma: 2.0,
                    },
                    2,
                ),
                (
                    "ex_o.rgb",
                    Adjustment::Exposure {
                        exposure: 0.5,
                        offset: -0.1,
                        gamma: 0.7,
                    },
                    2,
                ),
                (
                    "vib_sm100.rgb",
                    Adjustment::Vibrance {
                        vibrance: 0,
                        saturation: -100,
                    },
                    2,
                ),
                (
                    "vib_s50.rgb",
                    Adjustment::Vibrance {
                        vibrance: 0,
                        saturation: 50,
                    },
                    2,
                ),
                (
                    "vib_sm50.rgb",
                    Adjustment::Vibrance {
                        vibrance: 0,
                        saturation: -50,
                    },
                    2,
                ),
            ] {
                let (worst, _) = compare(adjustment, name);
                assert!(worst <= allowed, "{name}: off by {worst}");
            }
        }

        #[test]
        fn gradient_map_matches_photoshop() {
            use crate::gradient::Method;
            for (name, method, allowed) in [
                ("gm_classic.rgb", Method::Classic, 1),
                ("gm_perc.rgb", Method::Perceptual, 2),
                ("gm_lin.rgb", Method::Linear, 2),
                // Smooth's easing is measured only roughly
                ("gm_smooth.rgb", Method::Smooth, 6),
            ] {
                let adjustment = Adjustment::GradientMap {
                    from: [255, 0, 0],
                    to: [0, 0, 255],
                    method,
                };
                let (worst, _) = compare(adjustment, name);
                assert!(worst <= allowed, "{name}: off by {worst}");
            }
        }

        #[test]
        fn curves_per_channel() {
            // Red's own curve first, then the composite (Photoshop's order)
            let t = ramp(Adjustment::curves_per_channel([
                &[(0, 0), (255, 128)],
                &[(128, 0), (255, 255)],
                &[],
                &[],
            ]));
            assert_eq!((t[0][160], t[1][160], t[2][255]), (32, 80, 128));
        }

        /// Photoshop 2026 Color Balance settings for the rows of color_balance.rgb:
        /// shadows, midtones, highlights, Preserve Luminosity.
        const COLOR_BALANCE_ROWS: [([[i32; 3]; 3], bool); 80] = [
            ([[0, -6, 0], [0, 20, 0], [-51, 38, 0]], false),
            ([[0, 33, 0], [0, 0, 0], [-32, 52, 82]], false),
            ([[1, -66, 0], [-66, -45, 72], [99, 7, 46]], false),
            ([[36, 0, 0], [0, 78, 0], [0, 62, 0]], false),
            ([[-84, 63, -78], [-83, 0, -25], [96, 0, 0]], false),
            ([[-4, 41, 0], [-91, 0, 0], [-50, -26, 0]], false),
            ([[-14, 0, -4], [17, 0, 58], [0, -23, -34]], false),
            ([[40, -98, 48], [-95, 0, -85], [19, 73, 55]], false),
            ([[88, 0, 0], [-6, 60, -24], [-55, -53, 94]], false),
            ([[52, -24, 0], [0, 0, 67], [-39, -53, 0]], false),
            ([[-74, 0, 0], [-80, 0, 45], [0, 0, 0]], false),
            ([[0, -29, 0], [58, 0, 7], [32, 18, 62]], false),
            ([[-26, 45, -91], [0, -99, 59], [0, -92, 92]], false),
            ([[39, 0, 50], [0, 0, 0], [0, -97, 0]], false),
            ([[-57, 0, 34], [0, 0, -36], [0, 0, 0]], false),
            ([[0, 0, 0], [-99, 60, -88], [-37, 58, -87]], false),
            ([[0, 0, 0], [0, 0, 0], [31, 0, 0]], false),
            ([[-11, 65, 50], [-8, -52, 0], [0, 83, 0]], false),
            ([[-90, 17, 0], [59, -87, 60], [94, 7, 0]], false),
            ([[0, 37, 0], [0, 0, -93], [-5, -69, 0]], false),
            ([[87, 0, 87], [0, 0, 0], [-40, 0, 0]], false),
            ([[68, 0, 0], [0, 72, 78], [48, 0, 0]], false),
            ([[0, 0, 40], [0, 0, 0], [16, -36, 53]], false),
            ([[-11, -79, 0], [0, 6, -61], [0, -76, 91]], false),
            ([[78, 0, -66], [0, -63, -41], [71, 52, -45]], false),
            ([[-95, 22, 0], [45, -39, 0], [7, 0, 0]], false),
            ([[0, 0, 0], [0, -45, 0], [98, 0, 0]], false),
            ([[0, -89, -54], [0, 0, -33], [-91, 15, 0]], false),
            ([[-15, 11, 0], [-47, 0, 39], [-70, -81, -72]], false),
            ([[35, -76, 73], [93, -25, -73], [0, 70, 30]], false),
            ([[-85, 0, 0], [-55, 67, 0], [0, -16, 42]], false),
            ([[0, 17, 0], [0, 0, 93], [0, -32, 0]], false),
            ([[0, 22, -37], [-15, 0, -10], [0, 0, 0]], false),
            ([[0, 0, 45], [71, 0, 0], [0, 0, 0]], false),
            ([[-49, -6, -93], [4, -20, 62], [35, 70, 0]], false),
            ([[0, 0, 0], [34, 0, 0], [-91, -72, 0]], false),
            ([[-66, 0, -94], [0, 0, 0], [-14, -16, 0]], false),
            ([[0, 0, 89], [0, -49, 0], [24, 0, 0]], false),
            ([[0, 0, 0], [37, -47, 0], [18, 93, 0]], false),
            ([[0, 84, -7], [14, 52, 0], [-47, 100, -64]], false),
            ([[0, 0, 0], [49, 0, 0], [0, 0, 13]], true),
            ([[0, -95, 14], [0, 0, -39], [-90, 0, 0]], true),
            ([[-76, 0, -58], [0, 0, 0], [86, -30, -24]], true),
            ([[-37, -93, 61], [0, -86, 0], [0, 0, 0]], true),
            ([[0, 0, -52], [-49, -13, 0], [-3, 92, -78]], true),
            ([[-53, -71, 40], [37, -56, 41], [-53, 6, -42]], true),
            ([[99, -12, -57], [0, 0, 0], [0, 0, 0]], true),
            ([[0, 0, 15], [87, 0, 0], [0, -14, -72]], true),
            ([[0, 0, 0], [0, 0, 0], [-44, 29, 0]], true),
            ([[0, 0, 0], [0, 0, 0], [-73, 0, 0]], true),
            ([[64, 94, 3], [0, -26, 0], [0, 18, 0]], true),
            ([[-13, -51, 0], [41, 0, 0], [-37, -85, -40]], true),
            ([[0, 0, 0], [0, 0, 88], [82, 0, 59]], true),
            ([[23, 0, 0], [0, 0, 0], [63, 0, 0]], true),
            ([[-30, 0, -12], [0, 91, 0], [13, 5, 0]], true),
            ([[-61, 95, -4], [-59, -21, 0], [-8, 0, 18]], true),
            ([[-9, 0, 0], [0, 0, 0], [0, 0, 97]], true),
            ([[-10, 0, 0], [0, -30, 0], [0, 15, 0]], true),
            ([[72, 7, 0], [-93, 0, 0], [-66, 30, 0]], true),
            ([[0, 0, -96], [0, 17, 33], [0, 0, 0]], true),
            ([[0, 4, 13], [-29, 15, 0], [47, 0, 5]], true),
            ([[-41, 49, -93], [0, 0, -29], [-74, 0, 0]], true),
            ([[31, 92, -51], [0, 67, 1], [-88, 0, -19]], true),
            ([[56, 42, 0], [6, 0, 0], [0, 0, 0]], true),
            ([[0, 0, 10], [23, -32, 0], [0, 0, 0]], true),
            ([[0, 76, 72], [0, -64, 0], [0, 0, 0]], true),
            ([[0, 0, 0], [0, 10, 0], [0, 0, -63]], true),
            ([[-68, 0, 0], [0, 0, -84], [39, -74, 0]], true),
            ([[89, 35, 45], [18, 0, 36], [0, 93, 0]], true),
            ([[0, -19, 0], [0, 0, 0], [0, 3, 34]], true),
            ([[-77, 38, 0], [0, -18, 29], [63, 51, 79]], true),
            ([[76, -27, 0], [0, 87, 0], [-28, -32, -67]], true),
            ([[0, 58, 0], [-83, 8, 0], [-3, 0, 0]], true),
            ([[-86, -70, 87], [82, -97, -5], [2, 0, 49]], true),
            ([[-63, 0, 0], [0, 0, -47], [11, 0, -93]], true),
            ([[36, 88, 0], [12, 0, 30], [-43, -65, 0]], true),
            ([[0, 0, 0], [-11, -99, -53], [0, 0, 30]], true),
            ([[50, 11, 0], [-84, 92, 0], [-17, 0, 68]], true),
            ([[64, 0, 0], [65, 0, 0], [0, 0, 0]], true),
            ([[20, -28, 0], [-49, 0, 48], [58, 0, 0]], true),
        ];
        /// Levels settings (input black, input white, gamma, output black,
        /// output white) for the rows of levels.bin.
        #[allow(clippy::approx_constant)]
        const LEVELS_ROWS: [(u8, u8, f32, u8, u8); 28] = [
            (0, 255, 0.1, 0, 255),
            (0, 255, 0.2, 0, 255),
            (0, 255, 0.3, 0, 255),
            (0, 255, 0.4, 0, 255),
            (0, 255, 0.5, 0, 255),
            (0, 255, 0.6, 0, 255),
            (0, 255, 0.7, 0, 255),
            (0, 255, 0.8, 0, 255),
            (0, 255, 0.9, 0, 255),
            (0, 255, 1.1, 0, 255),
            (0, 255, 1.2, 0, 255),
            (0, 255, 1.4, 0, 255),
            (0, 255, 1.5, 0, 255),
            (0, 255, 1.7, 0, 255),
            (0, 255, 2.0, 0, 255),
            (0, 255, 2.5, 0, 255),
            (0, 255, 3.0, 0, 255),
            (0, 255, 4.0, 0, 255),
            (0, 255, 5.0, 0, 255),
            (0, 255, 6.0, 0, 255),
            (0, 255, 7.0, 0, 255),
            (0, 255, 8.0, 0, 255),
            (0, 255, 9.99, 0, 255),
            (30, 200, 1.0, 0, 255),
            (0, 255, 1.0, 20, 230),
            (50, 180, 1.5, 10, 240),
            (10, 240, 0.6, 30, 200),
            (100, 255, 0.7071, 0, 255),
        ];
        /// Use Legacy, brightness and contrast for the rows of
        /// brightness_contrast.bin.
        const BRIGHTNESS_CONTRAST_ROWS: [(bool, i32, i32); 18] = [
            (false, 30, 40),
            (false, -60, 70),
            (false, 100, -30),
            (false, -120, -50),
            (false, 150, 100),
            (false, 50, 50),
            (false, -40, 20),
            (true, 50, 0),
            (true, -50, 0),
            (true, 0, 50),
            (true, 0, -50),
            (true, 0, 100),
            (true, 30, 40),
            (true, 0, 25),
            (true, 0, 75),
            (true, 0, 99),
            (true, -100, -30),
            (true, 0, -25),
        ];
    }
}
