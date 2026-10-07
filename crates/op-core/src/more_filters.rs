//! More of the Filter menu: Distort › Wave, Shear and Displace; Blur ›
//! Radial Blur, Smart Blur, Shape Blur and Lens Blur; Noise › Reduce
//! Noise; Render › Fibers and Lens Flare; Sharpen › Smart Sharpen;
//! Stylize › Extrude and Oil Paint. Each works on the layer's straight
//! RGBA pixels (w × h, row by row). The shapes follow Photoshop's
//! controls; Adobe's exact algorithms are its own (see
//! `more_filters.md`).

use std::f32::consts::{PI, TAU};

type Px = [u8; 4];

/// A repeatable random value in 0..1 for (`x`, `y`, `k`) and a seed.
fn hash(x: i64, y: i64, k: u64, seed: u32) -> f32 {
    let mut v = (x as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
        .wrapping_add(k.wrapping_mul(0x1656_67B1_9E37_79F9))
        .wrapping_add((seed as u64).wrapping_mul(0x27D4_EB2F_1656_67C5));
    v ^= v >> 33;
    v = v.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    v ^= v >> 33;
    (v >> 40) as f32 / (1u64 << 24) as f32
}

/// What a distortion does where it reaches past the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undefined {
    WrapAround,
    RepeatEdge,
}

/// Bilinear sample at (x, y) (pixel centers at integer + 0.5),
/// premultiplied while mixing; outside, wrapped or edge-repeated.
fn sample(px: &[Px], w: usize, h: usize, x: f32, y: f32, edge: Undefined) -> Px {
    let (fx, fy) = (x - 0.5, y - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - x0, fy - y0);
    let fetch = |xi: i64, yi: i64| {
        let (xi, yi) = match edge {
            Undefined::WrapAround => (xi.rem_euclid(w as i64), yi.rem_euclid(h as i64)),
            Undefined::RepeatEdge => (xi.clamp(0, w as i64 - 1), yi.clamp(0, h as i64 - 1)),
        };
        px[yi as usize * w + xi as usize]
    };
    let mut acc = [0f32; 4];
    for (dy, wy) in [(0, 1.0 - ty), (1, ty)] {
        for (dx, wx) in [(0, 1.0 - tx), (1, tx)] {
            let p = fetch(x0 as i64 + dx, y0 as i64 + dy);
            let wgt = wx * wy;
            let a = p[3] as f32 / 255.0;
            for c in 0..3 {
                acc[c] += p[c] as f32 * a * wgt;
            }
            acc[3] += p[3] as f32 * wgt;
        }
    }
    unpremultiply(acc)
}

fn unpremultiply(acc: [f32; 4]) -> Px {
    let a = acc[3].clamp(0.0, 255.0);
    if a < 0.5 {
        return [0; 4];
    }
    let k = 255.0 / a;
    [
        (acc[0] * k).round().clamp(0.0, 255.0) as u8,
        (acc[1] * k).round().clamp(0.0, 255.0) as u8,
        (acc[2] * k).round().clamp(0.0, 255.0) as u8,
        a.round() as u8,
    ]
}

// ---------------------------------------------------------------- Wave

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveType {
    Sine,
    Triangle,
    Square,
}

/// Distort › Wave: `generators` waves (1–999) of wavelengths and
/// amplitudes between the min and max (pixels), scaled horizontally and
/// vertically (percent); their phases and lengths come from `seed`
/// (Randomize picks another).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wave {
    pub generators: u32,
    pub wavelength: (f32, f32),
    pub amplitude: (f32, f32),
    pub scale: (f32, f32),
    pub kind: WaveType,
    pub undefined: Undefined,
    pub seed: u32,
}

impl Default for Wave {
    /// Photoshop's defaults.
    fn default() -> Self {
        Self {
            generators: 5,
            wavelength: (10.0, 120.0),
            amplitude: (5.0, 35.0),
            scale: (100.0, 100.0),
            kind: WaveType::Sine,
            undefined: Undefined::RepeatEdge,
            seed: 1,
        }
    }
}

pub fn wave(px: &[Px], w: usize, h: usize, s: Wave) -> Vec<Px> {
    let n = s.generators.clamp(1, 999) as usize;
    let gens: Vec<(f32, f32, f32, f32)> = (0..n)
        .map(|k| {
            let r = |j: u64| hash(k as i64, 0, j, s.seed);
            let len = s.wavelength.0 + (s.wavelength.1 - s.wavelength.0) * r(1);
            let amp = s.amplitude.0 + (s.amplitude.1 - s.amplitude.0) * r(2);
            (len.max(1.0), amp / n as f32, r(3) * TAU, r(4) * TAU)
        })
        .collect();
    let shape = |t: f32| match s.kind {
        WaveType::Sine => t.sin(),
        WaveType::Triangle => {
            let u = (t / TAU).rem_euclid(1.0);
            if u < 0.5 {
                4.0 * u - 1.0
            } else {
                3.0 - 4.0 * u
            }
        }
        WaveType::Square => {
            if t.sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }
    };
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
            let (mut dx, mut dy) = (0.0, 0.0);
            for &(len, amp, px_phase, py_phase) in &gens {
                dx += amp * shape(TAU * y / len + px_phase);
                dy += amp * shape(TAU * x / len + py_phase);
            }
            sample(
                px,
                w,
                h,
                x + dx * s.scale.0 / 100.0,
                y + dy * s.scale.1 / 100.0,
                s.undefined,
            )
        })
        .collect()
}

// ---------------------------------------------------------------- Shear

/// Distort › Shear: each row moves sideways by the curve's offset at its
/// height (`offsets`, 0–1 of the image's width, top to bottom, as many
/// as Photoshop's grid has rows: −0.5 left edge … 0.5 right edge).
pub fn shear(px: &[Px], w: usize, h: usize, offsets: &[f32], edge: Undefined) -> Vec<Px> {
    let n = offsets.len().max(1);
    let at = |t: f32| -> f32 {
        if offsets.is_empty() {
            return 0.0;
        }
        let f = t * (n - 1) as f32;
        let i = (f.floor() as usize).min(n.saturating_sub(2));
        let u = f - i as f32;
        if n == 1 {
            offsets[0]
        } else {
            offsets[i] + (offsets[i + 1] - offsets[i]) * u
        }
    };
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
            let shift = at(y / h as f32) * w as f32;
            sample(px, w, h, x - shift, y, edge)
        })
        .collect()
}

// ---------------------------------------------------------- Displace

/// Distort › Displace: a gray map (its own size, tiled or stretched)
/// moves each pixel horizontally and vertically by up to `scale` percent
/// of 128 pixels, 128 meaning no move.
pub struct DisplaceMap<'a> {
    pub width: usize,
    pub height: usize,
    /// The map's red channel steers horizontally, its green vertically
    /// (a one-channel map steers both).
    pub pixels: &'a [Px],
    pub stretch: bool,
}

pub fn displace(
    px: &[Px],
    w: usize,
    h: usize,
    map: &DisplaceMap,
    scale: (f32, f32),
    edge: Undefined,
) -> Vec<Px> {
    (0..w * h)
        .map(|i| {
            let (xi, yi) = (i % w, i / w);
            let (mx, my) = if map.stretch {
                (xi * map.width / w.max(1), yi * map.height / h.max(1))
            } else {
                (xi % map.width.max(1), yi % map.height.max(1))
            };
            let m = map.pixels[my * map.width + mx];
            let dx = (m[0] as f32 - 128.0) / 128.0 * 128.0 * scale.0 / 100.0;
            let dy = (m[1] as f32 - 128.0) / 128.0 * 128.0 * scale.1 / 100.0;
            sample(px, w, h, xi as f32 + 0.5 + dx, yi as f32 + 0.5 + dy, edge)
        })
        .collect()
}

// ------------------------------------------------------- Radial Blur

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadialMethod {
    Spin,
    Zoom,
}

/// Blur › Radial Blur: Spin blurs along circles about the center by
/// `amount` (1–100) degrees' worth; Zoom along the lines from it. Quality
/// (Draft, Good, Best) sets how many samples are taken. The center is in
/// 0–1 of the image.
pub fn radial_blur(
    px: &[Px],
    w: usize,
    h: usize,
    amount: f32,
    method: RadialMethod,
    quality: u8,
    center: (f32, f32),
) -> Vec<Px> {
    let samples = [8usize, 16, 32][quality.min(2) as usize];
    let (cx, cy) = (center.0 * w as f32, center.1 * h as f32);
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
            let (dx, dy) = (x - cx, y - cy);
            let mut acc = [0f32; 4];
            for k in 0..samples {
                let t = k as f32 / (samples - 1) as f32 - 0.5;
                let (sx, sy) = match method {
                    RadialMethod::Spin => {
                        let a = t * amount.to_radians();
                        let (s, c) = a.sin_cos();
                        (cx + dx * c - dy * s, cy + dx * s + dy * c)
                    }
                    RadialMethod::Zoom => {
                        let k = 1.0 + t * amount / 100.0;
                        (cx + dx * k, cy + dy * k)
                    }
                };
                let p = sample(px, w, h, sx, sy, Undefined::RepeatEdge);
                let a = p[3] as f32 / 255.0;
                for c in 0..3 {
                    acc[c] += p[c] as f32 * a;
                }
                acc[3] += p[3] as f32;
            }
            unpremultiply(acc.map(|v| v / samples as f32))
        })
        .collect()
}

// -------------------------------------------------------- Smart Blur

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmartBlurMode {
    Normal,
    EdgeOnly,
    OverlayEdge,
}

/// Blur › Smart Blur: each pixel averages the neighbours within `radius`
/// whose colors are within `threshold` levels of it (edges stay sharp).
/// Edge Only draws the edges (where neighbours differ more) white on
/// black; Overlay Edge draws them white over the blurred image. Quality
/// (0 Low, 1 Medium, 2 High) is how densely the neighbourhood is read:
/// every third, every second or every pixel across and down (the nearest
/// four always count, so edges are found the same way).
pub fn smart_blur(
    px: &[Px],
    w: usize,
    h: usize,
    radius: f32,
    threshold: f32,
    quality: u8,
    mode: SmartBlurMode,
) -> Vec<Px> {
    let r = radius.round().max(1.0) as i64;
    let stride = 3 - quality.min(2) as i64;
    let at = |x: i64, y: i64| {
        px[y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize]
    };
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            let p = px[i];
            let mut acc = [0f32; 3];
            let mut n = 0.0;
            let mut edge = false;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy > r * r {
                        continue;
                    }
                    let near = dx.abs() + dy.abs() <= 1;
                    if !near && (dx % stride != 0 || dy % stride != 0) {
                        continue;
                    }
                    let q = at(x + dx, y + dy);
                    let diff = (0..3)
                        .map(|c| (q[c] as f32 - p[c] as f32).abs())
                        .fold(0f32, f32::max);
                    if diff <= threshold {
                        for c in 0..3 {
                            acc[c] += q[c] as f32;
                        }
                        n += 1.0;
                    } else if dx.abs() + dy.abs() == 1 {
                        edge = true;
                    }
                }
            }
            let blurred = [
                (acc[0] / n).round() as u8,
                (acc[1] / n).round() as u8,
                (acc[2] / n).round() as u8,
                p[3],
            ];
            match mode {
                SmartBlurMode::Normal => blurred,
                SmartBlurMode::EdgeOnly => {
                    let v = if edge { 255 } else { 0 };
                    [v, v, v, p[3]]
                }
                SmartBlurMode::OverlayEdge => {
                    if edge {
                        [255, 255, 255, p[3]]
                    } else {
                        blurred
                    }
                }
            }
        })
        .collect()
}

// -------------------------------------------------------- Shape Blur

/// Blur › Shape Blur's kernels: a shape's footprint in a disc of the
/// radius.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlurShape {
    Circle,
    Square,
    Star,
    Heart,
    Diamond,
}

impl BlurShape {
    pub const ALL: [Self; 5] = [
        Self::Circle,
        Self::Square,
        Self::Star,
        Self::Heart,
        Self::Diamond,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Circle => "Circle",
            Self::Square => "Square",
            Self::Star => "Star",
            Self::Heart => "Heart",
            Self::Diamond => "Diamond",
        }
    }

    /// Whether (u, v) in −1..1 is inside the shape.
    fn contains(self, u: f32, v: f32) -> bool {
        match self {
            Self::Circle => u * u + v * v <= 1.0,
            Self::Square => u.abs() <= 0.8 && v.abs() <= 0.8,
            Self::Diamond => u.abs() + v.abs() <= 1.0,
            Self::Star => {
                let r = (u * u + v * v).sqrt();
                let a = v.atan2(u) + PI / 2.0;
                let edge = 0.5 + 0.5 * (0.5 + 0.5 * (5.0 * a).cos());
                r <= edge
            }
            Self::Heart => {
                let (x, y) = (u * 1.2, -v * 1.2 + 0.25);
                (x * x + y * y - 1.0).powi(3) - x * x * y.powi(3) <= 0.0
            }
        }
    }
}

/// The average over `shape`'s footprint of `radius` (premultiplied).
fn kernel_blur(px: &[Px], w: usize, h: usize, taps: &[(i64, i64, f32)]) -> Vec<Px> {
    let total: f32 = taps.iter().map(|t| t.2).sum::<f32>().max(1e-6);
    let at = |x: i64, y: i64| {
        px[y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize]
    };
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            let mut acc = [0f32; 4];
            for &(dx, dy, wgt) in taps {
                let q = at(x + dx, y + dy);
                let a = q[3] as f32 / 255.0;
                for c in 0..3 {
                    acc[c] += q[c] as f32 * a * wgt;
                }
                acc[3] += q[3] as f32 * wgt;
            }
            unpremultiply(acc.map(|v| v / total))
        })
        .collect()
}

pub fn shape_blur(px: &[Px], w: usize, h: usize, radius: f32, shape: BlurShape) -> Vec<Px> {
    let r = radius.round().max(1.0) as i64;
    let taps: Vec<(i64, i64, f32)> = (-r..=r)
        .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| shape.contains(dx as f32 / r as f32, dy as f32 / r as f32))
        .map(|(dx, dy)| (dx, dy, 1.0))
        .collect();
    kernel_blur(px, w, h, &taps)
}

// --------------------------------------------------------- Lens Blur

/// Where Lens Blur's depth comes from: none (the whole image blurs
/// evenly), the layer's transparency, or its layer mask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthSource {
    None,
    Transparency,
    LayerMask,
}

/// Lens Blur's settings, as its dialog has them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LensBlur {
    /// Iris: Radius 0–100, the blades (3–8), Blade Curvature 0–100 (100 is
    /// a circle) and Rotation in degrees.
    pub radius: f32,
    pub blades: u32,
    pub curvature: f32,
    pub rotation: f32,
    /// Specular Highlights: Brightness 0–100 and Threshold 0–255.
    pub brightness: f32,
    pub threshold: u8,
    /// Noise: Amount 0–100 (as Add Noise's), Gaussian or uniform,
    /// Monochromatic.
    pub noise: f32,
    pub gaussian: bool,
    pub monochromatic: bool,
    /// Depth Map: its source, Blur Focal Distance (0–255: the depth that
    /// stays sharp) and Invert.
    pub depth: DepthSource,
    pub focal: u8,
    pub invert: bool,
}

impl Default for LensBlur {
    /// Photoshop's: Hexagon, Radius 15, Threshold 255, no depth map.
    fn default() -> Self {
        Self {
            radius: 15.0,
            blades: 6,
            curvature: 0.0,
            rotation: 0.0,
            brightness: 0.0,
            threshold: 255,
            noise: 0.0,
            gaussian: false,
            monochromatic: false,
            depth: DepthSource::None,
            focal: 0,
            invert: false,
        }
    }
}

impl LensBlur {
    /// Whether (u, v), in units of the radius, is inside the iris: the
    /// polygon of `blades` sides turned by Rotation, rounded toward the
    /// circle by Blade Curvature.
    pub fn inside(&self, u: f32, v: f32) -> bool {
        let r = (u * u + v * v).sqrt();
        if self.blades < 3 {
            return r <= 1.0;
        }
        let n = self.blades as f32;
        let a = v.atan2(u) - self.rotation.to_radians();
        let sector = (a / (TAU / n)).floor();
        let mid = (sector + 0.5) * TAU / n;
        // The polygon's edge at this angle, then toward the circle
        let edge = (PI / n).cos() / (a - mid).cos();
        let k = (self.curvature / 100.0).clamp(0.0, 1.0);
        r <= edge + (1.0 - edge) * k
    }
}

/// Blur › Lens Blur: each pixel averages the iris-shaped neighborhood of
/// its radius, pixels at or above Threshold counting more by Brightness
/// (so they spread as highlights), then noise as Add Noise's. With a depth
/// map (`depth`: 0–255 a pixel, from the transparency or the mask) the
/// radius scales with how far a pixel's depth is from Blur Focal Distance
/// (Invert flips the depths), so the focal depth stays sharp.
pub fn lens_blur(px: &[Px], w: usize, h: usize, o: &LensBlur, depth: Option<&[u8]>) -> Vec<Px> {
    let max_r = o.radius.round().max(0.0) as i64;
    // Each radius's taps, made when first needed
    let mut kernels: Vec<Option<Vec<(i64, i64)>>> = vec![None; max_r as usize + 1];
    let mut kernel = |r: i64| -> Vec<(i64, i64)> {
        kernels[r as usize]
            .get_or_insert_with(|| {
                if r == 0 {
                    return vec![(0, 0)];
                }
                (-r..=r)
                    .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| o.inside(dx as f32 / r as f32, dy as f32 / r as f32))
                    .collect()
            })
            .clone()
    };
    let boost = 1.0 + o.brightness / 100.0 * 8.0;
    let boosted: Vec<[f32; 4]> = px
        .iter()
        .map(|&p| {
            let l = (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3;
            let k = if l >= o.threshold as u32 { boost } else { 1.0 };
            [
                p[0] as f32 * k,
                p[1] as f32 * k,
                p[2] as f32 * k,
                p[3] as f32,
            ]
        })
        .collect();
    let radius_at = |i: usize| -> i64 {
        let Some(d) = depth.and_then(|d| d.get(i)) else {
            return max_r;
        };
        let d = if o.invert { 255 - *d } else { *d };
        (o.radius * (d as f32 - o.focal as f32).abs() / 255.0).round() as i64
    };
    // Add Noise's strength (`filter.md`): uniform spans ±(amount% × 255 +
    // 0.75), Gaussian has that standard deviation
    let spread = o.noise / 100.0 * 255.0;
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            let taps = kernel(radius_at(i).min(max_r));
            let mut acc = [0f32; 4];
            for &(dx, dy) in &taps {
                let xi = (x + dx).clamp(0, w as i64 - 1) as usize;
                let yi = (y + dy).clamp(0, h as i64 - 1) as usize;
                let q = boosted[yi * w + xi];
                for c in 0..4 {
                    acc[c] += q[c];
                }
            }
            let total = taps.len() as f32;
            let mut out = [0u8; 4];
            for c in 0..3 {
                let channel = if o.monochromatic { 0 } else { c as u64 };
                let n = if o.noise <= 0.0 {
                    0.0
                } else if o.gaussian {
                    let s: f32 = (0..4).map(|k| hash(x, y, 20 + channel * 4 + k, 7)).sum();
                    (s - 2.0) * 3f32.sqrt() * spread
                } else {
                    (hash(x, y, 10 + channel, 7) * 2.0 - 1.0) * (spread + 0.75)
                };
                out[c] = (acc[c] / total + n).round().clamp(0.0, 255.0) as u8;
            }
            out[3] = (acc[3] / total).round().clamp(0.0, 255.0) as u8;
            out
        })
        .collect()
}

// ------------------------------------------------------ Reduce Noise

/// Reduce Noise's settings, as its dialog has them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReduceNoise {
    /// Strength 0–10, Preserve Details, Reduce Color Noise and Sharpen
    /// Details 0–100 %.
    pub strength: f32,
    pub preserve: f32,
    pub color: f32,
    pub sharpen: f32,
    /// Remove JPEG Artifact.
    pub jpeg: bool,
    /// Advanced › Per Channel: each of red, green and blue's own Strength
    /// (0–10) and Preserve Details (0–100 %).
    pub channels: [(f32, f32); 3],
}

impl Default for ReduceNoise {
    /// Photoshop's Default setting.
    fn default() -> Self {
        Self {
            strength: 6.0,
            preserve: 60.0,
            color: 45.0,
            sharpen: 25.0,
            jpeg: false,
            channels: [(0.0, 60.0); 3],
        }
    }
}

/// Remove JPEG Artifact: across each edge of the 8 × 8 blocks JPEG
/// compresses in, the two pixels on either side move halfway toward each
/// other where they differ by less than 24 levels (a block's step, not a
/// real edge).
fn deblock(px: &[Px], w: usize, h: usize) -> Vec<Px> {
    let mut out = px.to_vec();
    let mut smooth = |a: usize, b: usize| {
        let (p, q) = (px[a], px[b]);
        if (0..3).all(|c| (p[c] as i32 - q[c] as i32).abs() < 24) {
            for c in 0..3 {
                let m = (p[c] as f32 + q[c] as f32) / 2.0;
                out[a][c] = ((p[c] as f32 + m) / 2.0).round() as u8;
                out[b][c] = ((q[c] as f32 + m) / 2.0).round() as u8;
            }
        }
    };
    for y in 0..h {
        for x in (8..w).step_by(8) {
            smooth(y * w + x - 1, y * w + x);
        }
    }
    for y in (8..h).step_by(8) {
        for x in 0..w {
            smooth((y - 1) * w + x, y * w + x);
        }
    }
    out
}

/// One channel smoothed by its own Strength and Preserve Details (as the
/// overall smoothing, on that channel alone).
fn smooth_channel(px: &mut [Px], w: usize, h: usize, c: usize, strength: f32, preserve: f32) {
    if strength <= 0.0 {
        return;
    }
    let src = px.to_vec();
    let at = |x: i64, y: i64| {
        src[y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize][c]
    };
    let edge = 12.0 + (100.0 - preserve.clamp(0.0, 100.0)) / 100.0 * 40.0;
    let k = strength / 10.0;
    for i in 0..w * h {
        let (x, y) = ((i % w) as i64, (i / w) as i64);
        let p = src[i][c] as f32;
        let (mut acc, mut n) = (0.0, 0.0);
        for dy in -2..=2 {
            for dx in -2..=2 {
                let q = at(x + dx, y + dy) as f32;
                let wgt = (1.0 - (q - p).abs() / edge).max(0.0);
                acc += q * wgt;
                n += wgt;
            }
        }
        let v = p + (acc / f32::max(n, 1e-3) - p) * k;
        px[i][c] = v.round().clamp(0.0, 255.0) as u8;
    }
}

/// Noise › Reduce Noise: Remove JPEG Artifact first (`deblock`), then
/// each channel's own smoothing (Per Channel); then Strength (0–10)
/// smooths each channel within what Preserve Details (0–100 %) lets
/// through as edges; Reduce Color Noise (0–100 %) smooths the chroma
/// more; Sharpen Details (0–100 %) puts edge contrast back.
pub fn reduce_noise(px: &[Px], w: usize, h: usize, o: &ReduceNoise) -> Vec<Px> {
    let mut first = if o.jpeg {
        deblock(px, w, h)
    } else {
        px.to_vec()
    };
    for (c, &(s, p)) in o.channels.iter().enumerate() {
        smooth_channel(&mut first, w, h, c, s, p);
    }
    let px = &first[..];
    let (strength, preserve, color, sharpen) = (o.strength, o.preserve, o.color, o.sharpen);
    let at = |x: i64, y: i64| {
        px[y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize]
    };
    // Differences above this count as detail: the more details are
    // preserved, the smaller
    let edge = 12.0 + (100.0 - preserve.clamp(0.0, 100.0)) / 100.0 * 40.0;
    let k = strength / 10.0;
    let out: Vec<Px> = (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            let p = px[i];
            let mut acc = [0f32; 3];
            let mut chroma = [0f32; 3];
            let mut n = 0.0;
            let mut m = 0.0;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let q = at(x + dx, y + dy);
                    let diff = (0..3)
                        .map(|c| (q[c] as f32 - p[c] as f32).abs())
                        .fold(0f32, f32::max);
                    let wgt = (1.0 - diff / edge).max(0.0);
                    for c in 0..3 {
                        acc[c] += q[c] as f32 * wgt;
                        chroma[c] += q[c] as f32;
                    }
                    n += wgt;
                    m += 1.0;
                }
            }
            let smooth = [0, 1, 2].map(|c| acc[c] / n.max(1e-3));
            let mut v = [0, 1, 2].map(|c| p[c] as f32 + (smooth[c] - p[c] as f32) * k);
            // Color noise: the chroma toward its neighbourhood's
            let lum = (v[0] + v[1] + v[2]) / 3.0;
            let avg = [0, 1, 2].map(|c| chroma[c] / m);
            let avg_l = (avg[0] + avg[1] + avg[2]) / 3.0;
            for c in 0..3 {
                let own = v[c] - lum;
                let near = avg[c] - avg_l;
                v[c] = lum + own + (near - own) * color / 100.0;
            }
            [
                v[0].round().clamp(0.0, 255.0) as u8,
                v[1].round().clamp(0.0, 255.0) as u8,
                v[2].round().clamp(0.0, 255.0) as u8,
                p[3],
            ]
        })
        .collect();
    if sharpen <= 0.0 {
        return out;
    }
    // Sharpen Details: an unsharp mask of radius 1
    let blur = kernel_blur(
        &out,
        w,
        h,
        &[
            (-1, 0, 1.0),
            (1, 0, 1.0),
            (0, -1, 1.0),
            (0, 1, 1.0),
            (0, 0, 4.0),
        ],
    );
    out.iter()
        .zip(&blur)
        .map(|(o, b)| {
            let s = |c: usize| {
                (o[c] as f32 + (o[c] as f32 - b[c] as f32) * sharpen / 50.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            [s(0), s(1), s(2), o[3]]
        })
        .collect()
}

// ------------------------------------------------------- Smart Sharpen

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharpenRemove {
    GaussianBlur,
    LensBlur,
    MotionBlur,
}

/// Smart Sharpen's Shadows or Highlights: how much the sharpening fades
/// there (0–100 %), how far into the tones that reaches (Tonal Width,
/// 0–100 %) and over how wide a neighbourhood a pixel's tone is judged
/// (Radius, 1–100 pixels).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToneFade {
    pub fade: f32,
    pub tonal_width: f32,
    pub radius: f32,
}

impl Default for ToneFade {
    /// Photoshop's: no fade, 50 % tonal width, radius 1.
    fn default() -> Self {
        Self {
            fade: 0.0,
            tonal_width: 50.0,
            radius: 1.0,
        }
    }
}

/// Smart Sharpen's settings, as its dialog has them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmartSharpen {
    /// Amount 1–500 %, Radius 0.1–64 pixels, Reduce Noise 0–100 %.
    pub amount: f32,
    pub radius: f32,
    pub noise: f32,
    /// The blur it undoes, and Motion Blur's angle in degrees.
    pub remove: SharpenRemove,
    pub angle: f32,
    pub shadows: ToneFade,
    pub highlights: ToneFade,
    /// Use Legacy: the older sharpening, without Reduce Noise; More
    /// Accurate (legacy only) sharpens in two finer passes.
    pub legacy: bool,
    pub more_accurate: bool,
}

impl Default for SmartSharpen {
    /// Photoshop's Default preset.
    fn default() -> Self {
        Self {
            amount: 200.0,
            radius: 1.0,
            noise: 10.0,
            remove: SharpenRemove::LensBlur,
            angle: 0.0,
            shadows: ToneFade::default(),
            highlights: ToneFade::default(),
            legacy: false,
            more_accurate: false,
        }
    }
}

/// Sharpen › Smart Sharpen: an unsharp mask of Radius and Amount with the
/// blur it undoes (Gaussian; Lens: a disc, finer edges; Motion: along the
/// angle) and Reduce Noise keeping small differences out (not in Legacy).
/// Each pixel's tone is its neighbourhood's luminance (a box of the
/// shadows' or highlights' Radius); the sharpening fades by Fade Amount
/// over the darkest (shadows) or brightest (highlights) Tonal Width of the
/// tones, fully at black or white and not at all past the width. Legacy's
/// More Accurate runs two passes of half the amount.
pub fn smart_sharpen(px: &[Px], w: usize, h: usize, o: &SmartSharpen) -> Vec<Px> {
    if o.legacy && o.more_accurate {
        let half = SmartSharpen {
            amount: o.amount / 2.0,
            more_accurate: false,
            ..*o
        };
        let once = smart_sharpen(px, w, h, &half);
        return smart_sharpen(&once, w, h, &half);
    }
    let r = o.radius.max(0.1);
    let taps: Vec<(i64, i64, f32)> = match o.remove {
        SharpenRemove::MotionBlur => {
            let (s, c) = o.angle.to_radians().sin_cos();
            let n = r.round().max(1.0) as i64;
            (-n..=n)
                .map(|k| {
                    (
                        (k as f32 * c).round() as i64,
                        (-k as f32 * s).round() as i64,
                        1.0,
                    )
                })
                .collect()
        }
        _ => {
            let n = (r * 2.0).ceil().max(1.0) as i64;
            let sigma = if o.remove == SharpenRemove::LensBlur {
                r * 0.6
            } else {
                r
            };
            (-n..=n)
                .flat_map(|dy| (-n..=n).map(move |dx| (dx, dy)))
                .map(|(dx, dy)| {
                    let d2 = (dx * dx + dy * dy) as f32;
                    (dx, dy, (-d2 / (2.0 * sigma * sigma)).exp())
                })
                .collect()
        }
    };
    let blur = kernel_blur(px, w, h, &taps);
    let floor = if o.legacy {
        0.0
    } else {
        o.noise / 100.0 * 10.0
    };
    // The tones the fades judge by: luminance over each one's radius
    let tones = |fade: &ToneFade| -> Option<Vec<f32>> {
        if fade.fade <= 0.0 {
            return None;
        }
        let n = fade.radius.round().max(1.0) as i64 - 1;
        let lum: Vec<Px> = px
            .iter()
            .map(|p| {
                let l = ((p[0] as u32 + p[1] as u32 + p[2] as u32) / 3) as u8;
                [l, l, l, 255]
            })
            .collect();
        let box_taps: Vec<(i64, i64, f32)> = (-n..=n)
            .flat_map(|dy| (-n..=n).map(move |dx| (dx, dy, 1.0)))
            .collect();
        Some(
            kernel_blur(&lum, w, h, &box_taps)
                .iter()
                .map(|p| p[0] as f32 / 255.0)
                .collect(),
        )
    };
    let (shadow_tone, highlight_tone) = (tones(&o.shadows), tones(&o.highlights));
    let width = |f: &ToneFade| (f.tonal_width / 100.0).max(0.01);
    px.iter()
        .zip(&blur)
        .enumerate()
        .map(|(i, (p, b))| {
            let mut fade_k = 1.0;
            if let Some(t) = &shadow_tone {
                let wd = width(&o.shadows);
                fade_k -= o.shadows.fade / 100.0 * (1.0 - t[i] / wd).clamp(0.0, 1.0);
            }
            if let Some(t) = &highlight_tone {
                let wd = width(&o.highlights);
                fade_k -= o.highlights.fade / 100.0 * ((t[i] - (1.0 - wd)) / wd).clamp(0.0, 1.0);
            }
            let s = |c: usize| {
                let d = p[c] as f32 - b[c] as f32;
                let d = if d.abs() <= floor {
                    0.0
                } else {
                    d - floor * d.signum()
                };
                (p[c] as f32 + d * o.amount / 100.0 * f32::max(fade_k, 0.0))
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            [s(0), s(1), s(2), p[3]]
        })
        .collect()
}

// ------------------------------------------------------------ Fibers

/// Render › Fibers: vertical streaks between the foreground and
/// background colors; Variance (1–64) sets how fast they change along
/// their length, Strength (1–64) how long and stiff they are.
pub fn fibers(
    w: usize,
    h: usize,
    variance: f32,
    strength: f32,
    fg: [u8; 3],
    bg: [u8; 3],
    seed: u32,
) -> Vec<Px> {
    // Each column a random walk, smoothed along the column by Strength
    let step = variance / 64.0;
    let mut out = vec![[0u8; 4]; w * h];
    for x in 0..w {
        let mut v = hash(x as i64, 0, 1, seed);
        let mut col = Vec::with_capacity(h);
        for y in 0..h {
            v += (hash(x as i64, y as i64, 2, seed) - 0.5) * step;
            v = v.clamp(0.0, 1.0);
            col.push(v);
        }
        let reach = (strength / 2.0).round().max(1.0) as i64;
        for y in 0..h {
            let (lo, hi) = (
                (y as i64 - reach).max(0),
                (y as i64 + reach).min(h as i64 - 1),
            );
            let t: f32 = (lo..=hi).map(|k| col[k as usize]).sum::<f32>() / (hi - lo + 1) as f32;
            let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
            out[y * w + x] = [mix(bg[0], fg[0]), mix(bg[1], fg[1]), mix(bg[2], fg[2]), 255];
        }
    }
    out
}

// -------------------------------------------------------- Lens Flare

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LensType {
    Zoom50_300,
    Prime35,
    Prime105,
    MoviePrime,
}

impl LensType {
    pub const ALL: [Self; 4] = [
        Self::Zoom50_300,
        Self::Prime35,
        Self::Prime105,
        Self::MoviePrime,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Zoom50_300 => "50-300mm Zoom",
            Self::Prime35 => "35mm Prime",
            Self::Prime105 => "105mm Prime",
            Self::MoviePrime => "Movie Prime",
        }
    }
}

/// Render › Lens Flare: a bright glow at `center` (0–1 of the image) with
/// rings and ghosts along the line through the image's middle, added
/// (Screen) over the image; Brightness 10–300 %.
pub fn lens_flare(
    px: &[Px],
    w: usize,
    h: usize,
    center: (f32, f32),
    brightness: f32,
    lens: LensType,
) -> Vec<Px> {
    let (cx, cy) = (center.0 * w as f32, center.1 * h as f32);
    let (mx, my) = (w as f32 / 2.0, h as f32 / 2.0);
    let size = (w.max(h) as f32)
        * match lens {
            LensType::Zoom50_300 => 0.12,
            LensType::Prime35 => 0.09,
            LensType::Prime105 => 0.15,
            LensType::MoviePrime => 0.1,
        };
    let k = brightness / 100.0;
    // Ghosts: discs along the axis through the middle, tinted
    let ghosts: [(f32, f32, [f32; 3]); 4] = [
        (0.6, 0.08, [0.3, 0.6, 1.0]),
        (1.2, 0.05, [0.4, 1.0, 0.5]),
        (1.6, 0.12, [1.0, 0.6, 0.3]),
        (2.0, 0.04, [0.8, 0.5, 1.0]),
    ];
    px.iter()
        .enumerate()
        .map(|(i, &p)| {
            let (x, y) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
            let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / size;
            // The glow and a ring
            let glow = (-d * d * 2.0).exp() + 0.25 * (-(d - 1.6).powi(2) * 40.0).exp();
            let mut light = [glow, glow * 0.95, glow * 0.85];
            if lens == LensType::MoviePrime {
                // A horizontal streak
                let streak = (-((y - cy) / size * 20.0).powi(2)).exp()
                    * (-(x - cx).abs() / (size * 6.0)).exp();
                for c in &mut light {
                    *c += streak * 0.6;
                }
            }
            for &(t, r, tint) in &ghosts {
                let (gx, gy) = (cx + (mx - cx) * t, cy + (my - cy) * t);
                let gd = ((x - gx).powi(2) + (y - gy).powi(2)).sqrt() / (size * r * 4.0);
                let g = if gd < 1.0 {
                    0.12 * (1.0 - gd * gd)
                } else {
                    0.0
                };
                for c in 0..3 {
                    light[c] += g * tint[c];
                }
            }
            let screen = |v: u8, l: f32| {
                let a = v as f32 / 255.0;
                let b = (l * k).clamp(0.0, 1.0);
                ((1.0 - (1.0 - a) * (1.0 - b)) * 255.0).round() as u8
            };
            [
                screen(p[0], light[0]),
                screen(p[1], light[1]),
                screen(p[2], light[2]),
                p[3],
            ]
        })
        .collect()
}

// ----------------------------------------------------------- Extrude

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtrudeType {
    Blocks,
    Pyramids,
}

/// Stylize › Extrude: the image cut into `size`-pixel squares raised
/// toward the viewer (Blocks: a front face in the square's average color,
/// or the image with Solid Front Faces off; Pyramids: four shaded
/// faces), `depth` (1–255) setting how strongly the sides darken, random
/// or by the square's brightness (Level-based). With `mask_incomplete`
/// the squares cut off by the image's edge are left as they are.
#[allow(clippy::too_many_arguments)] // one per control in the dialog
pub fn extrude(
    px: &[Px],
    w: usize,
    h: usize,
    kind: ExtrudeType,
    size: usize,
    depth: f32,
    level_based: bool,
    solid: bool,
    mask_incomplete: bool,
    seed: u32,
) -> Vec<Px> {
    let size = size.max(2);
    let mut out = px.to_vec();
    for by in (0..h).step_by(size) {
        for bx in (0..w).step_by(size) {
            let (bw, bh) = ((w - bx).min(size), (h - by).min(size));
            if mask_incomplete && (bw < size || bh < size) {
                continue;
            }
            let mut avg = [0f32; 3];
            for y in by..by + bh {
                for x in bx..bx + bw {
                    for c in 0..3 {
                        avg[c] += px[y * w + x][c] as f32;
                    }
                }
            }
            let n = (bw * bh) as f32;
            avg = avg.map(|v| v / n);
            let height = if level_based {
                (avg[0] + avg[1] + avg[2]) / 3.0 / 255.0
            } else {
                hash(bx as i64, by as i64, 3, seed)
            } * depth
                / 255.0;
            for y in by..by + bh {
                for x in bx..bx + bw {
                    let (u, v) = (
                        (x - bx) as f32 / bw.max(2).saturating_sub(1) as f32,
                        (y - by) as f32 / bh.max(2).saturating_sub(1) as f32,
                    );
                    let base = if kind == ExtrudeType::Blocks && solid {
                        avg
                    } else {
                        let p = px[y * w + x];
                        [p[0] as f32, p[1] as f32, p[2] as f32]
                    };
                    let shade = match kind {
                        ExtrudeType::Blocks => {
                            // A rim on the lower and right edges in shadow
                            let rim = 1.0 - (1.0 - u).min(1.0 - v) * 6.0;
                            if rim > 0.0 {
                                1.0 - height * 0.8 * rim
                            } else {
                                1.0
                            }
                        }
                        ExtrudeType::Pyramids => {
                            // Four faces lit from the top left
                            let (du, dv) = (u - 0.5, v - 0.5);
                            let face = if du.abs() > dv.abs() {
                                if du < 0.0 { 1.15 } else { 0.7 }
                            } else if dv < 0.0 {
                                1.1
                            } else {
                                0.75
                            };
                            1.0 + (face - 1.0) * (0.3 + height)
                        }
                    };
                    let p = &mut out[y * w + x];
                    for c in 0..3 {
                        p[c] = (base[c] * shade).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
    }
    out
}

// --------------------------------------------------------- Oil Paint

/// Oil Paint's settings, as its dialog has them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OilPaint {
    /// Brush: Stylization 0.1–10, Cleanliness 0–10, Scale 0.1–10, Bristle
    /// Detail 0–10.
    pub stylization: f32,
    pub cleanliness: f32,
    pub scale: f32,
    pub bristle: f32,
    /// Lighting on, its angle in degrees and Shine 0–10.
    pub lighting: bool,
    pub angle: f32,
    pub shine: f32,
}

impl Default for OilPaint {
    fn default() -> Self {
        Self {
            stylization: 1.5,
            cleanliness: 5.0,
            scale: 1.0,
            bristle: 0.0,
            lighting: true,
            angle: -60.0,
            shine: 1.0,
        }
    }
}

/// Stylize › Oil Paint: brush strokes from a smoothing that keeps edges
/// (Stylization, Cleanliness and Scale set its reach); Bristle Detail
/// puts the image's fine grain back over the strokes (bristle marks, up
/// to 0.8 of it at 10); with Lighting, the paint's luminosity is lit from
/// the angle by Shine like raised paint.
pub fn oil_paint(px: &[Px], w: usize, h: usize, o: &OilPaint) -> Vec<Px> {
    let (stylization, cleanliness, scale, angle, shine) =
        (o.stylization, o.cleanliness, o.scale, o.angle, o.shine);
    let r = ((stylization + scale) * 0.6).round().clamp(1.0, 12.0) as i64;
    let at = |x: i64, y: i64| {
        px[y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize]
    };
    // Kuwahara: the calmest of four quadrants gives the color
    let painted: Vec<Px> = (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            let mut best = (f32::MAX, [0f32; 3]);
            for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let mut sum = [0f32; 3];
                let mut sq = 0f32;
                let mut n = 0.0;
                for dy in 0..=r {
                    for dx in 0..=r {
                        let q = at(x + dx * sx, y + dy * sy);
                        let l = (q[0] as f32 + q[1] as f32 + q[2] as f32) / 3.0;
                        for c in 0..3 {
                            sum[c] += q[c] as f32;
                        }
                        sq += l * l;
                        n += 1.0;
                    }
                }
                let mean_l = (sum[0] + sum[1] + sum[2]) / 3.0 / n;
                let var = sq / n - mean_l * mean_l;
                if var < best.0 {
                    best = (var, sum.map(|v| v / n));
                }
            }
            let p = at(x, y);
            // Cleanliness: more of the painted color, less of the photo
            let k = 0.6 + cleanliness / 25.0;
            let mix =
                |c: usize| (p[c] as f32 + (best.1[c] - p[c] as f32) * k.min(1.0)).round() as u8;
            [mix(0), mix(1), mix(2), p[3]]
        })
        .collect();
    // Bristle Detail: the image's fine grain (its difference from a 3 × 3
    // blur) over the strokes
    let painted: Vec<Px> = if o.bristle > 0.0 {
        let k = o.bristle / 10.0 * 0.8;
        (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as i64, (i / w) as i64);
                let mut blur = [0f32; 3];
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let q = at(x + dx, y + dy);
                        for c in 0..3 {
                            blur[c] += q[c] as f32 / 9.0;
                        }
                    }
                }
                let (p, orig) = (painted[i], px[i]);
                let v = |c: usize| {
                    (p[c] as f32 + (orig[c] as f32 - blur[c]) * k)
                        .round()
                        .clamp(0.0, 255.0) as u8
                };
                [v(0), v(1), v(2), p[3]]
            })
            .collect()
    } else {
        painted
    };
    if !o.lighting || shine <= 0.0 {
        return painted;
    }
    // Lighting: the paint's luminosity as a height, lit from `angle`
    let (s, c) = angle.to_radians().sin_cos();
    let lum = |p: Px| (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0;
    let pat = |x: i64, y: i64| {
        painted[y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize]
    };
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            let gx = lum(pat(x + 1, y)) - lum(pat(x - 1, y));
            let gy = lum(pat(x, y + 1)) - lum(pat(x, y - 1));
            let light = (gx * c - gy * s) / 255.0 * shine * 0.6;
            let p = painted[i];
            let lit = |v: u8| (v as f32 * (1.0 + light)).round().clamp(0.0, 255.0) as u8;
            [lit(p[0]), lit(p[1]), lit(p[2]), p[3]]
        })
        .collect()
}

/// Displacement maps in use, by number (`Filter::Displace` carries the
/// number so the filter stays a small copyable value).
/// A map: its width, height and pixels.
pub type Map = (usize, usize, Vec<Px>);

static MAPS: std::sync::Mutex<Vec<std::sync::Arc<Map>>> = std::sync::Mutex::new(Vec::new());

/// Keeps a displacement map (`width` × `height` RGBA); returns its number.
pub fn register_map(width: usize, height: usize, pixels: Vec<Px>) -> u32 {
    let mut all = MAPS.lock().expect("map registry");
    all.push(std::sync::Arc::new((width, height, pixels)));
    (all.len() - 1) as u32
}

/// The map registered as `id`.
pub fn map(id: u32) -> Option<std::sync::Arc<Map>> {
    MAPS.lock().ok()?.get(id as usize).cloned()
}

/// Shear's curve: up to eight points (height 0–1, offset −0.5–0.5 of the
/// width) joined by a smooth curve (a natural cubic spline, as Curves'),
/// as offsets every row.
pub fn shear_offsets(points: &[(f32, f32)], rows: usize) -> Vec<f32> {
    let spline = crate::adjust::Spline::new(points);
    (0..rows.max(2))
        .map(|k| spline.at(k as f32 / (rows.max(2) - 1) as f32))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: usize, h: usize) -> Vec<Px> {
        (0..w * h)
            .map(|i| {
                let v = ((i % w) * 255 / (w - 1)) as u8;
                [v, v, v, 255]
            })
            .collect()
    }

    fn checker(w: usize, h: usize) -> Vec<Px> {
        (0..w * h)
            .map(|i| {
                if (i % w / 4 + i / w / 4).is_multiple_of(2) {
                    [20, 20, 20, 255]
                } else {
                    [230, 230, 230, 255]
                }
            })
            .collect()
    }

    #[test]
    fn wave_and_shear_move_pixels_sideways() {
        let px = checker(40, 40);
        let waved = wave(&px, 40, 40, Wave::default());
        assert_ne!(waved, px);
        // No amplitude, no change
        let still = wave(
            &px,
            40,
            40,
            Wave {
                amplitude: (0.0, 0.0),
                ..Default::default()
            },
        );
        assert_eq!(still, px);
        // Shearing a gradient by a quarter of the width (wrapped)
        let g = gradient(40, 4);
        let sheared = shear(&g, 40, 4, &[0.25, 0.25], Undefined::WrapAround);
        assert_eq!(sheared[10], g[0]);
    }

    #[test]
    fn displace_by_a_gray_map() {
        let g = gradient(40, 4);
        // 128 everywhere: no move; 192 in red: 64 px × 10 % right
        let flat = vec![[128, 128, 128, 255]; 4];
        let map = DisplaceMap {
            width: 2,
            height: 2,
            pixels: &flat,
            stretch: false,
        };
        assert_eq!(
            displace(&g, 40, 4, &map, (10.0, 10.0), Undefined::RepeatEdge),
            g
        );
        let right = vec![[192, 128, 128, 255]; 4];
        let map = DisplaceMap {
            width: 2,
            height: 2,
            pixels: &right,
            stretch: true,
        };
        let out = displace(&g, 40, 4, &map, (10.0, 10.0), Undefined::RepeatEdge);
        assert!(out[5][0] > g[5][0]);
    }

    #[test]
    fn reduce_noise_jpeg_and_channels() {
        // 8 × 8 blocks alternating 100 and 110: Remove JPEG Artifact softens
        // the step across the blocks' edges
        let blocks: Vec<Px> = (0..16 * 16)
            .map(|i| {
                let v = if ((i % 16) / 8 + (i / 16) / 8) % 2 == 0 {
                    100
                } else {
                    110
                };
                [v, v, v, 255]
            })
            .collect();
        let off = ReduceNoise {
            strength: 0.0,
            color: 0.0,
            sharpen: 0.0,
            ..Default::default()
        };
        assert_eq!(reduce_noise(&blocks, 16, 16, &off), blocks);
        let jpeg = reduce_noise(&blocks, 16, 16, &ReduceNoise { jpeg: true, ..off });
        let step = |p: &[Px]| (p[8][0] as i32 - p[7][0] as i32).abs();
        assert!(step(&jpeg) < step(&blocks));
        // Per Channel: red's own Strength smooths red alone
        let noisy: Vec<Px> = (0..16 * 16)
            .map(|i| {
                let n = (hash((i % 16) as i64, (i / 16) as i64, 2, 3) * 20.0) as u8;
                [100 + n, 100 + n, 100 + n, 255]
            })
            .collect();
        let red = reduce_noise(
            &noisy,
            16,
            16,
            &ReduceNoise {
                channels: [(10.0, 0.0), (0.0, 60.0), (0.0, 60.0)],
                ..off
            },
        );
        assert!(red.iter().zip(&noisy).any(|(a, b)| a[0] != b[0]));
        assert!(
            red.iter()
                .zip(&noisy)
                .all(|(a, b)| a[1] == b[1] && a[2] == b[2])
        );
    }

    #[test]
    fn lens_blur_iris_and_depth() {
        let hexagon = LensBlur::default();
        // A hexagon's flat side is inside the circle; full curvature rounds it
        let corner_gap = (0.95f32 * (PI / 6.0).cos(), 0.0);
        assert!(hexagon.inside(corner_gap.0, corner_gap.1));
        let flat = (PI / 6.0).cos() + 0.03;
        assert!(!hexagon.inside(flat * (PI / 6.0).cos(), flat * (PI / 6.0).sin()));
        let round = LensBlur {
            curvature: 100.0,
            ..hexagon
        };
        assert!(round.inside(flat * (PI / 6.0).cos(), flat * (PI / 6.0).sin()));
        // Rotation turns the corners: a corner direction becomes a side's
        let corner = (0.97, 0.0);
        assert!(hexagon.inside(corner.0, corner.1));
        let turned = LensBlur {
            rotation: 30.0,
            ..hexagon
        };
        assert!(!turned.inside(corner.0, corner.1));
        // With a depth map, the focal depth stays sharp
        let px = checker(16, 16);
        let near = vec![0u8; 16 * 16];
        let o = LensBlur {
            radius: 4.0,
            depth: DepthSource::Transparency,
            focal: 0,
            ..Default::default()
        };
        assert_eq!(lens_blur(&px, 16, 16, &o, Some(&near)), px);
        let far = vec![255u8; 16 * 16];
        assert_ne!(lens_blur(&px, 16, 16, &o, Some(&far)), px);
        let inverted = LensBlur { invert: true, ..o };
        assert_eq!(lens_blur(&px, 16, 16, &inverted, Some(&far)), px);
    }

    #[test]
    fn blurs_soften_and_keep_flat_areas() {
        let px = checker(32, 32);
        for out in [
            radial_blur(&px, 32, 32, 30.0, RadialMethod::Spin, 1, (0.5, 0.5)),
            radial_blur(&px, 32, 32, 30.0, RadialMethod::Zoom, 1, (0.5, 0.5)),
            shape_blur(&px, 32, 32, 3.0, BlurShape::Star),
            lens_blur(
                &px,
                32,
                32,
                &LensBlur {
                    radius: 3.0,
                    ..Default::default()
                },
                None,
            ),
        ] {
            assert_ne!(out, px);
        }
        let flat = vec![[90, 120, 150, 255]; 16 * 16];
        assert_eq!(shape_blur(&flat, 16, 16, 3.0, BlurShape::Heart), flat);
        assert_eq!(
            smart_blur(&flat, 16, 16, 3.0, 25.0, 2, SmartBlurMode::Normal),
            flat
        );
        // Smart Blur keeps the checker's edges (beyond the threshold)
        assert_eq!(
            smart_blur(&px, 32, 32, 3.0, 25.0, 2, SmartBlurMode::Normal),
            px
        );
        let edges = smart_blur(&px, 32, 32, 3.0, 25.0, 2, SmartBlurMode::EdgeOnly);
        // Lower quality reads fewer neighbours, so it blurs a little
        // differently
        let noisy: Vec<Px> = (0..32 * 32)
            .map(|i| {
                let v = (hash((i % 32) as i64, (i / 32) as i64, 1, 5) * 20.0) as u8 + 100;
                [v, v, v, 255]
            })
            .collect();
        assert_ne!(
            smart_blur(&noisy, 32, 32, 4.0, 25.0, 0, SmartBlurMode::Normal),
            smart_blur(&noisy, 32, 32, 4.0, 25.0, 2, SmartBlurMode::Normal)
        );
        assert_eq!(edges[0][0], 0);
        assert_eq!(edges[3][0], 255);
    }

    #[test]
    fn reduce_noise_and_smart_sharpen() {
        // Speckle on gray loses its spread; a flat field stays
        let noisy: Vec<Px> = (0..32 * 32)
            .map(|i| {
                let v = if hash(i as i64, 0, 0, 1) > 0.5 {
                    134
                } else {
                    122
                };
                [v, v, v, 255]
            })
            .collect();
        let quiet = reduce_noise(
            &noisy,
            32,
            32,
            &ReduceNoise {
                strength: 10.0,
                preserve: 0.0,
                color: 0.0,
                sharpen: 0.0,
                ..Default::default()
            },
        );
        let spread = |p: &[Px]| {
            let n = p.len() as f32;
            let mean = p.iter().map(|q| q[0] as f32).sum::<f32>() / n;
            (p.iter().map(|q| (q[0] as f32 - mean).powi(2)).sum::<f32>() / n).sqrt()
        };
        assert!(
            spread(&quiet) < spread(&noisy) * 0.6,
            "{} {}",
            spread(&quiet),
            spread(&noisy)
        );
        let px = checker(32, 32);
        let plain = SmartSharpen {
            amount: 100.0,
            noise: 0.0,
            remove: SharpenRemove::GaussianBlur,
            ..Default::default()
        };
        let sharp = smart_sharpen(&px, 32, 32, &plain);
        // Edges gain contrast
        let i = 3; // the last dark column before a light one
        assert!(sharp[i][0] < px[i][0]);
        // Shadows faded fully: the dark side of the edge keeps its value
        let faded = smart_sharpen(
            &px,
            32,
            32,
            &SmartSharpen {
                shadows: ToneFade {
                    fade: 100.0,
                    tonal_width: 100.0,
                    radius: 1.0,
                },
                ..plain
            },
        );
        assert!(faded[i][0] > sharp[i][0]);
        // Legacy's More Accurate differs from one pass
        let legacy = SmartSharpen {
            legacy: true,
            ..plain
        };
        assert_ne!(
            smart_sharpen(&px, 32, 32, &legacy),
            smart_sharpen(
                &px,
                32,
                32,
                &SmartSharpen {
                    more_accurate: true,
                    ..legacy
                }
            )
        );
    }

    #[test]
    fn renders() {
        let fib = fibers(16, 16, 16.0, 4.0, [0, 0, 0], [255, 255, 255], 1);
        assert!(fib.iter().any(|p| p[0] > 10 && p[0] < 245));
        let black = vec![[0, 0, 0, 255]; 32 * 32];
        let flare = lens_flare(&black, 32, 32, (0.25, 0.25), 100.0, LensType::Zoom50_300);
        assert!(flare[8 * 32 + 8][0] > 200);
        assert!(flare[31 * 32 + 31][0] < 60);
        let blocks = extrude(
            &checker(32, 32),
            32,
            32,
            ExtrudeType::Pyramids,
            8,
            30.0,
            true,
            false,
            false,
            1,
        );
        assert_ne!(blocks, checker(32, 32));
        // 32 is 3 squares of 10 and 2 pixels: Mask Incomplete Blocks leaves
        // the last 2 rows and columns alone
        let masked = extrude(
            &checker(32, 32),
            32,
            32,
            ExtrudeType::Blocks,
            10,
            30.0,
            false,
            true,
            true,
            1,
        );
        assert_eq!(masked[31 * 32 + 31], checker(32, 32)[31 * 32 + 31]);
        assert_eq!(masked[5 * 32 + 31], checker(32, 32)[5 * 32 + 31]);
        assert_ne!(&masked[..10], &checker(32, 32)[..10]);
        let o = OilPaint {
            stylization: 2.0,
            shine: 2.0,
            ..Default::default()
        };
        let oil = oil_paint(&checker(32, 32), 32, 32, &o);
        assert_eq!(oil.len(), 32 * 32);
        // Bristle Detail and Lighting each change the strokes
        let bristle = oil_paint(&checker(32, 32), 32, 32, &OilPaint { bristle: 10.0, ..o });
        assert_ne!(bristle, oil);
        let unlit = oil_paint(
            &checker(32, 32),
            32,
            32,
            &OilPaint {
                lighting: false,
                ..o
            },
        );
        assert_ne!(unlit, oil);
    }
}
