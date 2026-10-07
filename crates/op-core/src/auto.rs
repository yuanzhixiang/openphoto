//! Automatic tonal and color correction: the four algorithms of
//! Photoshop's Auto Color Correction Options, the target colors and
//! clipping, and Snap Neutral Midtones. The result is a set of Levels
//! values per channel (input black, gamma, input white, output black and
//! white), which Levels and Curves show and Image › Auto Tone, Auto
//! Contrast and Auto Color apply.

use crate::Document;

/// The Algorithms group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Algorithm {
    /// Enhance Monochromatic Contrast: all channels clipped identically
    /// (Auto Contrast).
    Monochromatic,
    /// Enhance Per Channel Contrast: each channel stretched on its own
    /// (Auto Tone).
    PerChannel,
    /// Find Dark & Light Colors: the average darkest and lightest pixels
    /// become the shadows and highlights (Auto Color).
    DarkLight,
    /// Enhance Brightness and Contrast: Levels' and Curves' default.
    #[default]
    BrightnessContrast,
}

impl Algorithm {
    pub const ALL: [Algorithm; 4] = [
        Algorithm::Monochromatic,
        Algorithm::PerChannel,
        Algorithm::DarkLight,
        Algorithm::BrightnessContrast,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Algorithm::Monochromatic => "Enhance Monochromatic Contrast",
            Algorithm::PerChannel => "Enhance Per Channel Contrast",
            Algorithm::DarkLight => "Find Dark & Light Colors",
            Algorithm::BrightnessContrast => "Enhance Brightness and Contrast",
        }
    }
}

/// Target Colors & Clipping: what the shadows, midtones and highlights
/// become and how much of each end is clipped (percent).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Targets {
    pub shadows: [u8; 3],
    pub midtones: [u8; 3],
    pub highlights: [u8; 3],
    pub shadow_clip: f32,
    pub highlight_clip: f32,
}

impl Default for Targets {
    fn default() -> Self {
        Self {
            shadows: [0; 3],
            midtones: [128; 3],
            highlights: [255; 3],
            shadow_clip: 0.1,
            highlight_clip: 0.1,
        }
    }
}

/// The whole Auto Color Correction Options.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Options {
    pub algorithm: Algorithm,
    pub snap_neutral: bool,
    pub targets: Targets,
}

impl Options {
    /// Image › Auto Tone.
    pub fn auto_tone(targets: Targets) -> Self {
        Self {
            algorithm: Algorithm::PerChannel,
            snap_neutral: false,
            targets,
        }
    }

    /// Image › Auto Contrast.
    pub fn auto_contrast(targets: Targets) -> Self {
        Self {
            algorithm: Algorithm::Monochromatic,
            snap_neutral: false,
            targets,
        }
    }

    /// Image › Auto Color.
    pub fn auto_color(targets: Targets) -> Self {
        Self {
            algorithm: Algorithm::DarkLight,
            snap_neutral: true,
            targets,
        }
    }
}

/// One channel's Levels values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Channel {
    pub black: f32,
    pub gamma: f32,
    pub white: f32,
    pub out_black: u8,
    pub out_white: u8,
}

impl Channel {
    pub fn identity() -> Self {
        Self {
            black: 0.0,
            gamma: 1.0,
            white: 255.0,
            out_black: 0,
            out_white: 255,
        }
    }

    /// The value `v` maps to.
    pub fn map(&self, v: f32) -> f32 {
        let x = ((v - self.black) / (self.white - self.black).max(1.0)).clamp(0.0, 1.0);
        let y = x.powf(1.0 / self.gamma);
        self.out_black as f32 + y * (self.out_white as f32 - self.out_black as f32)
    }

    pub fn table(&self) -> [u8; 256] {
        std::array::from_fn(|i| self.map(i as f32).round().clamp(0.0, 255.0) as u8)
    }
}

/// The pixels the correction looks at: the active layer's (inside the
/// selection, skipping transparent ones), every `step`-th in each
/// direction so at most about 250,000 are kept.
pub fn samples(doc: &Document) -> Vec<[u8; 3]> {
    let Some(image) = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .and_then(|l| l.image())
    else {
        return Vec::new();
    };
    let selection = doc.selection();
    let area = doc.width as u64 * doc.height as u64;
    let step = ((area as f64 / 250_000.0).sqrt().ceil() as u32).max(1);
    let mut out = Vec::new();
    for y in (0..doc.height).step_by(step as usize) {
        for x in (0..doc.width).step_by(step as usize) {
            if selection.is_some_and(|s| s.get(x, y) == 0) {
                continue;
            }
            let px = image.pixel(x, y);
            if px[3] == 0 {
                continue;
            }
            out.push([px[0], px[1], px[2]]);
        }
    }
    out
}

fn luminosity(p: [u8; 3]) -> f32 {
    0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
}

/// The darkest and lightest values of a histogram once `lo_clip` and
/// `hi_clip` percent of it are set aside at each end.
fn clipped_range(hist: &[u64; 256], lo_clip: f32, hi_clip: f32) -> (f32, f32) {
    let total: u64 = hist.iter().sum();
    if total == 0 {
        return (0.0, 255.0);
    }
    let cut = |pct: f32| (total as f64 * pct as f64 / 100.0) as u64;
    let (lo_cut, hi_cut) = (cut(lo_clip), cut(hi_clip));
    let mut acc = 0;
    let lo = (0..256).find(|&i| {
        acc += hist[i];
        acc > lo_cut
    });
    acc = 0;
    let hi = (0..256).rev().find(|&i| {
        acc += hist[i];
        acc > hi_cut
    });
    (lo.unwrap_or(0) as f32, hi.unwrap_or(255) as f32)
}

/// The Levels values per channel (red, green, blue) for these pixels.
pub fn compute(pixels: &[[u8; 3]], options: &Options) -> [Channel; 3] {
    let t = &options.targets;
    let mut out = [Channel::identity(); 3];
    if pixels.is_empty() {
        return out;
    }
    let mut hists = [[0u64; 256]; 3];
    for p in pixels {
        for c in 0..3 {
            hists[c][p[c] as usize] += 1;
        }
    }
    let ranges: [(f32, f32); 3] = match options.algorithm {
        Algorithm::PerChannel => hists.map(|h| clipped_range(&h, t.shadow_clip, t.highlight_clip)),
        Algorithm::Monochromatic | Algorithm::BrightnessContrast => {
            let mut all = [0u64; 256];
            for h in &hists {
                for (a, v) in all.iter_mut().zip(h) {
                    *a += v;
                }
            }
            [clipped_range(&all, t.shadow_clip, t.highlight_clip); 3]
        }
        Algorithm::DarkLight => {
            // The average color of the darkest and of the lightest pixels
            let mut order: Vec<(f32, usize)> = pixels
                .iter()
                .enumerate()
                .map(|(i, &p)| (luminosity(p), i))
                .collect();
            order.sort_by(|a, b| a.0.total_cmp(&b.0));
            let n = pixels.len();
            let count = |pct: f32| ((n as f64 * pct as f64 / 100.0).round() as usize).clamp(1, n);
            let average = |idx: &mut dyn Iterator<Item = &(f32, usize)>, k: usize| {
                let mut sum = [0.0f64; 3];
                for &(_, i) in idx.take(k) {
                    for c in 0..3 {
                        sum[c] += pixels[i][c] as f64;
                    }
                }
                sum.map(|s| (s / k as f64) as f32)
            };
            let ks = count(t.shadow_clip);
            let kh = count(t.highlight_clip);
            let dark = average(&mut order.iter(), ks);
            let light = average(&mut order.iter().rev(), kh);
            [0, 1, 2].map(|c| (dark[c], light[c]))
        }
    };
    for c in 0..3 {
        let (lo, hi) = ranges[c];
        out[c] = if hi < lo + 2.0 {
            Channel::identity()
        } else {
            Channel {
                black: lo,
                gamma: 1.0,
                white: hi,
                out_black: t.shadows[c],
                out_white: t.highlights[c],
            }
        };
    }
    if options.algorithm == Algorithm::BrightnessContrast {
        // Halfway from the stretched mean luminosity to the middle,
        // through one gamma for all channels
        let mean = pixels
            .iter()
            .map(|p| {
                let m = [0, 1, 2].map(|c| out[c].map(p[c] as f32) / 255.0);
                0.299 * m[0] + 0.587 * m[1] + 0.114 * m[2]
            })
            .sum::<f32>()
            / pixels.len() as f32;
        let mean = mean.clamp(0.02, 0.98);
        let goal = mean + (0.5 - mean) * 0.5;
        let gamma = (mean.ln() / goal.ln()).clamp(0.8, 1.25);
        for ch in &mut out {
            ch.gamma = gamma;
        }
    }
    if options.snap_neutral {
        snap_neutral(pixels, &mut out, t.midtones);
    }
    out
}

/// Snap Neutral Midtones: the average of the nearly neutral midtones
/// (after the stretch) is made the midtone target through each channel's
/// gamma.
fn snap_neutral(pixels: &[[u8; 3]], out: &mut [Channel; 3], midtones: [u8; 3]) {
    let mut sum = [0.0f64; 3];
    let mut count = 0usize;
    for p in pixels {
        let m = [0, 1, 2].map(|c| out[c].map(p[c] as f32));
        let max = m.iter().cloned().fold(0.0f32, f32::max);
        let min = m.iter().cloned().fold(255.0f32, f32::min);
        let l = 0.299 * m[0] + 0.587 * m[1] + 0.114 * m[2];
        if max - min < 40.0 && (40.0..=215.0).contains(&l) {
            for c in 0..3 {
                sum[c] += p[c] as f64;
            }
            count += 1;
        }
    }
    // Too few to tell (under 0.1% of the pixels): left as it is
    if count == 0 || count * 1000 < pixels.len() {
        return;
    }
    let neutral = sum.map(|s| (s / count as f64) as f32);
    for c in 0..3 {
        let ch = &mut out[c];
        let x = ((neutral[c] - ch.black) / (ch.white - ch.black).max(1.0)).clamp(0.01, 0.99);
        let span = ch.out_white as f32 - ch.out_black as f32;
        if span.abs() < 1.0 {
            continue;
        }
        let goal = ((midtones[c] as f32 - ch.out_black as f32) / span).clamp(0.01, 0.99);
        ch.gamma = (x.ln() / goal.ln()).clamp(0.1, 9.99);
    }
}

/// The lookup tables for a document (Image › Auto Tone and the rest).
pub fn tables(doc: &Document, options: &Options) -> [[u8; 256]; 3] {
    compute(&samples(doc), options).map(|c| c.table())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(f: impl Fn(u8) -> [u8; 3]) -> Vec<[u8; 3]> {
        (0..=255u8)
            .flat_map(|v| std::iter::repeat_n(f(v), 10))
            .collect()
    }

    #[test]
    fn per_channel_and_monochromatic() {
        // Red spans 50–200, green 20–220, blue 100–150
        let px = ramp(|v| {
            let s = v as f32 / 255.0;
            [
                (50.0 + s * 150.0) as u8,
                (20.0 + s * 200.0) as u8,
                (100.0 + s * 50.0) as u8,
            ]
        });
        let per = compute(&px, &Options::auto_tone(Targets::default()));
        assert!((per[0].black - 50.0).abs() <= 1.0 && (per[0].white - 200.0).abs() <= 1.0);
        assert!((per[2].black - 100.0).abs() <= 1.0 && (per[2].white - 150.0).abs() <= 1.0);
        let mono = compute(&px, &Options::auto_contrast(Targets::default()));
        assert_eq!(mono[0], mono[2]);
        assert!(mono[0].black <= 21.0 && mono[0].white >= 219.0);
    }

    #[test]
    fn dark_and_light_colors_and_targets() {
        // A bluish shadow and a yellowish highlight
        let mut px = vec![[100, 100, 100]; 1000];
        px.push([10, 10, 30]);
        px.push([240, 240, 200]);
        let targets = Targets {
            shadows: [20, 20, 20],
            ..Targets::default()
        };
        let o = Options {
            snap_neutral: false,
            ..Options::auto_color(targets)
        };
        let ch = compute(&px, &o);
        assert_eq!((ch[2].black, ch[2].white), (30.0, 200.0));
        assert_eq!(ch[0].out_black, 20);
        assert_eq!(ch[0].table()[10], 20);
    }

    #[test]
    fn snapping_neutral_midtones() {
        // Grays with a red cast: the midtones come back neutral
        let px = ramp(|v| [v.saturating_add(20), v, v]);
        let o = Options {
            algorithm: Algorithm::Monochromatic,
            snap_neutral: true,
            targets: Targets::default(),
        };
        let ch = compute(&px, &o);
        let mid = [148u8, 128, 128];
        let out = [0, 1, 2].map(|c| ch[c].map(mid[c] as f32).round());
        assert!((out[0] - out[1]).abs() <= 2.0, "{out:?}");
    }

    #[test]
    fn brightness_and_contrast_lifts_a_dark_image() {
        // Mostly shadows: the stretch alone leaves the mean low
        let px = ramp(|v| [((v as u32 * v as u32) / 255) as u8; 3]);
        let ch = compute(&px, &Options::default());
        assert!(ch[0].gamma > 1.0, "{ch:?}");
        assert_eq!(ch[0], ch[2]);
    }
}
