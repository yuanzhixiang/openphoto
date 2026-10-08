//! Tone adjustments that look at a pixel's surroundings: Image >
//! Adjustments > Shadows/Highlights and HDR Toning. Both work on the
//! luminosity: a blurred copy of it tells shadows from highlights (and
//! large shapes from detail), and each pixel's color is scaled to its new
//! luminosity. The shapes follow Photoshop's controls and defaults; the
//! exact curves are Adobe's own (see `tone.md`).

/// Shadows/Highlights' settings (Photoshop's, with Show More Options).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowsHighlights {
    /// Shadows: amount (0–100 %), tone (0–100 %), radius (0–2500 px).
    pub shadows: [f32; 3],
    /// Highlights: amount, tone, radius.
    pub highlights: [f32; 3],
    /// Color (−100–100): saturation in the changed areas.
    pub color: f32,
    /// Midtone (−100–100): midtone contrast.
    pub midtone: f32,
    /// Black and White Clip (0–50 %).
    pub clip: [f32; 2],
}

impl Default for ShadowsHighlights {
    /// Photoshop 2026's defaults.
    fn default() -> Self {
        Self {
            shadows: [35.0, 50.0, 30.0],
            highlights: [0.0, 50.0, 30.0],
            color: 20.0,
            midtone: 0.0,
            clip: [0.01, 0.01],
        }
    }
}

/// HDR Toning's methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HdrMethod {
    ExposureGamma,
    HighlightCompression,
    EqualizeHistogram,
    LocalAdaptation,
}

impl HdrMethod {
    pub const ALL: [Self; 4] = [
        Self::ExposureGamma,
        Self::HighlightCompression,
        Self::EqualizeHistogram,
        Self::LocalAdaptation,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ExposureGamma => "Exposure and Gamma",
            Self::HighlightCompression => "Highlight Compression",
            Self::EqualizeHistogram => "Equalize Histogram",
            Self::LocalAdaptation => "Local Adaptation",
        }
    }
}

/// HDR Toning's settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HdrToning {
    pub method: HdrMethod,
    /// Edge Glow: radius (1–500 px) and strength (0.1–4).
    pub radius: f32,
    pub strength: f32,
    /// Gamma (0.1–2), exposure (−5–5), detail (−100–300 %).
    pub gamma: f32,
    pub exposure: f32,
    pub detail: f32,
    /// Advanced: shadow, highlight, vibrance, saturation (−100–100).
    pub shadow: f32,
    pub highlight: f32,
    pub vibrance: f32,
    pub saturation: f32,
    /// Edge Glow's Smooth Edges: the glow keeps to edges (the base
    /// follows the image across strong steps).
    pub smooth_edges: bool,
    /// The Toning Curve: its points (input, output 0–255, by input), the
    /// first `curve_len` used, and which are corners (bit k for point k).
    pub curve: [(u8, u8); 16],
    pub curve_len: u8,
    pub corners: u16,
}

impl HdrToning {
    /// The Toning Curve as a table: smooth through its points, and bent
    /// sharply at corner points (each run between corners its own curve).
    pub fn curve_table(&self) -> [u8; 256] {
        let pts = &self.curve[..(self.curve_len as usize).clamp(0, 16)];
        if pts.len() < 2 {
            return std::array::from_fn(|i| i as u8);
        }
        let mut table = [0u8; 256];
        let mut start = 0;
        for k in 1..pts.len() {
            let corner = self.corners & (1 << k) != 0;
            if corner || k == pts.len() - 1 {
                let part = crate::adjust::curve_table(&pts[start..=k]);
                let (a, b) = (pts[start].0 as usize, pts[k].0 as usize);
                table[a..=b].copy_from_slice(&part[a..=b]);
                start = k;
            }
        }
        // Outside the end points the curve stays level
        let (first, last) = (pts[0], pts[pts.len() - 1]);
        table[..first.0 as usize].fill(first.1);
        table[last.0 as usize + 1..].fill(last.1);
        table
    }
}

impl Default for HdrToning {
    /// Photoshop 2026's defaults (Local Adaptation).
    fn default() -> Self {
        Self {
            method: HdrMethod::LocalAdaptation,
            radius: 15.0,
            strength: 0.52,
            gamma: 1.0,
            exposure: 0.0,
            detail: 30.0,
            shadow: 0.0,
            highlight: 0.0,
            vibrance: 0.0,
            saturation: 20.0,
            smooth_edges: false,
            curve: {
                let mut c = [(0, 0); 16];
                c[1] = (255, 255);
                c
            },
            curve_len: 2,
            corners: 0,
        }
    }
}

/// Rec. 601 luminosity, 0–1.
fn luma(p: [u8; 4]) -> f32 {
    (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0
}

/// A Gaussian blur of `v` (w × h) of standard deviation `sigma`, as three
/// box blurs with the edges repeated.
pub(crate) fn blur(v: &[f32], w: usize, h: usize, sigma: f32) -> Vec<f32> {
    if sigma < 0.5 || w == 0 || h == 0 {
        return v.to_vec();
    }
    // Box width for three passes of the same variance
    let r = (((12.0 * sigma * sigma / 3.0) + 1.0).sqrt() / 2.0)
        .floor()
        .max(1.0) as usize;
    let mut a = v.to_vec();
    let mut b = vec![0f32; v.len()];
    for _ in 0..3 {
        box_pass(&a, &mut b, w, h, r, true);
        box_pass(&b, &mut a, w, h, r, false);
    }
    a
}

fn box_pass(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize, horizontal: bool) {
    let (n, lines) = if horizontal { (w, h) } else { (h, w) };
    let at = |line: usize, i: usize| {
        if horizontal {
            line * w + i
        } else {
            i * w + line
        }
    };
    let span = (2 * r + 1) as f32;
    for line in 0..lines {
        let get = |i: isize| src[at(line, i.clamp(0, n as isize - 1) as usize)];
        let mut acc: f32 = (-(r as isize)..=r as isize).map(get).sum();
        for i in 0..n {
            dst[at(line, i)] = acc / span;
            acc += get(i as isize + r as isize + 1) - get(i as isize - r as isize);
        }
    }
}

/// Scales a pixel's color to luminosity `to` (from `from`), lifting toward
/// white where scaling alone would clip, and changes its saturation by
/// `sat` (1 keeps it).
fn relight(p: [u8; 4], from: f32, to: f32, sat: f32) -> [u8; 4] {
    let rgb = [p[0], p[1], p[2]].map(|v| v as f32 / 255.0);
    let gray = from;
    // Saturation about the pixel's own gray
    let rgb = rgb.map(|c| gray + (c - gray) * sat);
    let out = if from > 1e-4 {
        let k = to / from;
        rgb.map(|c| c * k)
    } else {
        rgb.map(|c| c + to)
    };
    // Whatever rose past white spills into the other channels
    let over = out.iter().fold(0f32, |m, &c| m.max(c - 1.0));
    let out = if over > 0.0 {
        let t = (over / (1.0 - to).max(1e-4)).min(1.0);
        out.map(|c| c.min(1.0) + (1.0 - c.min(1.0)) * t * 0.5)
    } else {
        out
    };
    let to8 = |c: f32| (c * 255.0).round().clamp(0.0, 255.0) as u8;
    [to8(out[0]), to8(out[1]), to8(out[2]), p[3]]
}

/// The luminosity below which (above which) the darkest (brightest) `clip`
/// percent lie.
fn ends(lum: &[f32], clip: [f32; 2]) -> (f32, f32) {
    let mut hist = [0u32; 256];
    for &l in lum {
        hist[(l * 255.0).round().clamp(0.0, 255.0) as usize] += 1;
    }
    let total = lum.len() as f32;
    let find = |pct: f32, from_top: bool| {
        let limit = total * pct / 100.0;
        let mut acc = 0.0;
        for k in 0..256 {
            let i = if from_top { 255 - k } else { k };
            acc += hist[i] as f32;
            if acc > limit {
                return i as f32 / 255.0;
            }
        }
        if from_top { 0.0 } else { 1.0 }
    };
    (find(clip[0], false), find(clip[1], true))
}

/// Black and White Clip: the result's darkest and brightest `clip`
/// percent go to black and white (the rest is left as it is).
fn clip_ends(out: &mut [f32], clip: [f32; 2]) {
    if clip[0] <= 0.0 && clip[1] <= 0.0 {
        return;
    }
    let (lo, hi) = ends(out, clip);
    for l in out.iter_mut() {
        if clip[0] > 0.0 && *l < lo {
            *l = 0.0;
        } else if clip[1] > 0.0 && *l > hi {
            *l = 1.0;
        }
    }
}

/// Shadows/Highlights on `px` (w × h, straight RGBA). Dark areas (by the
/// blurred luminosity, within the tone width) are lifted, bright ones
/// lowered, each by its amount; Midtone stretches the middle tones; Color
/// changes the saturation where the tones changed; then the clips.
pub fn shadows_highlights(px: &mut [[u8; 4]], w: usize, h: usize, s: ShadowsHighlights) {
    let lum: Vec<f32> = px.iter().map(|&p| luma(p)).collect();
    let base_s = blur(&lum, w, h, s.shadows[2] / 2.0);
    let base_h = if s.highlights[2] == s.shadows[2] {
        base_s.clone()
    } else {
        blur(&lum, w, h, s.highlights[2] / 2.0)
    };
    let (amt_s, tone_s) = (s.shadows[0] / 100.0, (s.shadows[1] / 100.0).max(0.01));
    let (amt_h, tone_h) = (s.highlights[0] / 100.0, (s.highlights[1] / 100.0).max(0.01));
    let mid = s.midtone / 100.0;
    let mut out: Vec<f32> = lum
        .iter()
        .enumerate()
        .map(|(i, &l)| {
            let ws = (1.0 - base_s[i] / tone_s).clamp(0.0, 1.0).powi(2);
            let wh = ((base_h[i] - (1.0 - tone_h)) / tone_h)
                .clamp(0.0, 1.0)
                .powi(2);
            let mut v = l + amt_s * ws * (1.0 - l) * 1.25 - amt_h * wh * l * 1.25;
            // Midtone contrast, strongest at the middle
            let weight = 1.0 - (2.0 * v - 1.0).abs();
            v = 0.5 + (v - 0.5) * (1.0 + mid * weight);
            v.clamp(0.0, 1.0)
        })
        .collect();
    clip_ends(&mut out, s.clip);
    for (i, p) in px.iter_mut().enumerate() {
        let change = (out[i] - lum[i]).abs();
        let sat = (1.0 + s.color / 100.0 * change * 2.0).max(0.0);
        *p = relight(*p, lum[i], out[i], sat);
    }
}

/// HDR Toning on `px` (w × h). Local Adaptation splits the luminosity into
/// a blurred base (Edge Glow radius) and detail, compresses the base by
/// Strength, adds the detail back by Detail, then Exposure, Gamma, Shadow
/// and Highlight, Vibrance and Saturation. The other methods are global:
/// Exposure and Gamma; Highlight Compression; Equalize Histogram.
pub fn hdr_toning(px: &mut [[u8; 4]], w: usize, h: usize, t: HdrToning) {
    let lum: Vec<f32> = px.iter().map(|&p| luma(p)).collect();
    let mut out: Vec<f32> = match t.method {
        HdrMethod::ExposureGamma => lum
            .iter()
            .map(|&l| {
                (l * 2f32.powf(t.exposure))
                    .clamp(0.0, 1.0)
                    .powf(1.0 / t.gamma.max(0.1))
            })
            .collect(),
        HdrMethod::HighlightCompression => lum
            .iter()
            .map(|&l| {
                let k = 2.0;
                l * (1.0 + k) / (1.0 + k * l)
            })
            .collect(),
        HdrMethod::EqualizeHistogram => {
            let mut hist = [0u32; 256];
            for &l in &lum {
                hist[(l * 255.0).round() as usize] += 1;
            }
            let mut cdf = [0f32; 256];
            let mut acc = 0u32;
            for (i, &n) in hist.iter().enumerate() {
                acc += n;
                cdf[i] = acc as f32 / lum.len().max(1) as f32;
            }
            lum.iter()
                .map(|&l| cdf[(l * 255.0).round() as usize])
                .collect()
        }
        HdrMethod::LocalAdaptation => {
            let mut base = blur(&lum, w, h, t.radius / 2.0);
            // Smooth Edges: across a strong step the base keeps to the
            // pixel's own level, so the glow doesn't spill over edges
            if t.smooth_edges {
                for (b, &l) in base.iter_mut().zip(&lum) {
                    let d = *b - l;
                    *b = l + d * (-(d * d) / 0.02).exp();
                }
            }
            let table = t.curve_table();
            let detail = 1.0 + t.detail / 100.0;
            let squeeze = 1.0 / (1.0 + t.strength);
            lum.iter()
                .zip(&base)
                .map(|(&l, &b)| {
                    let b2 = 0.5 + (b - 0.5) * squeeze;
                    let v = b2 + (l - b) * detail;
                    let v = (v * 2f32.powf(t.exposure)).clamp(0.0, 1.0);
                    let v = v.powf(1.0 / t.gamma.max(0.1));
                    // Shadow and Highlight lift or lower those ends
                    let v = v
                        + t.shadow / 100.0 * (1.0 - v).powi(3) * 0.5
                        + t.highlight / 100.0 * v.powi(3) * 0.5;
                    // The Toning Curve, between table entries
                    let x = v.clamp(0.0, 1.0) * 255.0;
                    let (i, f) = ((x.floor() as usize).min(254), x - x.floor());
                    let c = table[i] as f32 + (table[i + 1] as f32 - table[i] as f32) * f;
                    (c / 255.0).clamp(0.0, 1.0)
                })
                .collect()
        }
    };
    if t.method != HdrMethod::LocalAdaptation {
        for v in &mut out {
            *v = v.clamp(0.0, 1.0);
        }
    }
    let adaptive = t.method == HdrMethod::LocalAdaptation;
    for (i, p) in px.iter_mut().enumerate() {
        let mut q = relight(*p, lum[i], out[i], 1.0);
        if adaptive {
            // Saturation for all colors, Vibrance more for muted ones
            let rgb = [q[0], q[1], q[2]].map(|v| v as f32 / 255.0);
            let gray = (rgb[0] + rgb[1] + rgb[2]) / 3.0;
            let spread = rgb.iter().fold(0f32, |m, &c| m.max((c - gray).abs()));
            let k = 1.0 + t.saturation / 100.0 + t.vibrance / 100.0 * (1.0 - spread * 2.0).max(0.0);
            let to8 = |c: f32| (c * 255.0).round().clamp(0.0, 255.0) as u8;
            q = [
                to8(gray + (rgb[0] - gray) * k),
                to8(gray + (rgb[1] - gray) * k),
                to8(gray + (rgb[2] - gray) * k),
                q[3],
            ];
        }
        *p = q;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dark left half and a bright right half.
    fn halves() -> (Vec<[u8; 4]>, usize, usize) {
        let (w, h) = (200, 10);
        let px = (0..w * h)
            .map(|i| {
                if i % w < w / 2 {
                    [40, 30, 30, 255]
                } else {
                    [230, 225, 225, 255]
                }
            })
            .collect();
        (px, w, h)
    }

    #[test]
    fn shadows_lift_and_highlights_fall() {
        let (mut px, w, h) = halves();
        let s = ShadowsHighlights {
            clip: [0.0, 0.0],
            ..Default::default()
        };
        shadows_highlights(&mut px, w, h, s);
        // The dark side is lighter, the bright side unchanged (no amount)
        assert!(px[2][0] > 55, "{:?}", px[2]);
        assert!((px[w - 3][0] as i32 - 230).abs() <= 2, "{:?}", px[w - 3]);
        // Highlights at 50% lower the bright side
        let (mut px, w, h) = halves();
        let s = ShadowsHighlights {
            shadows: [0.0, 50.0, 30.0],
            highlights: [50.0, 50.0, 30.0],
            clip: [0.0, 0.0],
            ..Default::default()
        };
        shadows_highlights(&mut px, w, h, s);
        assert!(px[w - 3][0] < 215, "{:?}", px[w - 3]);
        assert_eq!(px[2], [40, 30, 30, 255]);
    }

    #[test]
    fn hdr_toning_compresses_and_keeps_detail() {
        let (mut px, w, h) = halves();
        hdr_toning(&mut px, w, h, HdrToning::default());
        // The two halves move toward each other
        assert!(
            px[2][0] > 40 && px[w - 3][0] < 230,
            "{:?} {:?}",
            px[2],
            px[w - 3]
        );
        // Exposure and Gamma: +1 stop doubles a dark gray
        let mut px = vec![[50, 50, 50, 255]; 4];
        let t = HdrToning {
            method: HdrMethod::ExposureGamma,
            exposure: 1.0,
            ..Default::default()
        };
        hdr_toning(&mut px, 2, 2, t);
        assert!((px[0][0] as i32 - 100).abs() <= 1, "{:?}", px[0]);
    }

    #[test]
    fn toning_curve_table_and_corners() {
        let mut t = HdrToning::default();
        assert_eq!(t.curve_table()[100], 100);
        // An inverting curve
        t.curve[0] = (0, 255);
        t.curve[1] = (255, 0);
        assert_eq!(t.curve_table()[0], 255);
        assert_eq!(t.curve_table()[255], 0);
        // A corner bends sharply: straight lines either side of it
        t.curve = [(0, 0); 16];
        t.curve[1] = (128, 200);
        t.curve[2] = (255, 255);
        t.curve_len = 3;
        t.corners = 0b010;
        let table = t.curve_table();
        assert!((table[64] as i32 - 100).abs() <= 1, "{}", table[64]);
        assert_eq!(table[128], 200);
        // Toning with the inverting curve turns a light gray dark
        let mut px = vec![[200, 200, 200, 255]; 4];
        let mut inv = HdrToning::default();
        inv.curve[0] = (0, 255);
        inv.curve[1] = (255, 0);
        hdr_toning(&mut px, 2, 2, inv);
        assert!(px[0][0] < 100, "{:?}", px[0]);
    }

    #[test]
    fn blur_keeps_a_flat_field() {
        let v = vec![0.5f32; 30 * 20];
        assert!(
            blur(&v, 30, 20, 5.0)
                .iter()
                .all(|&x| (x - 0.5).abs() < 1e-5)
        );
    }
}
