//! Color adjustments that work in Lab or through a lookup cube: Image >
//! Adjustments > Replace Color, Match Color and Color Lookup.

use std::sync::{Arc, Mutex};

/// sRGB (0–255) to CIE Lab (D50, Photoshop's connection space).
pub fn to_lab(rgb: [u8; 3]) -> [f32; 3] {
    let lin = rgb.map(|v| {
        let c = v as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    const M: [[f32; 3]; 3] = [
        [0.4361, 0.3851, 0.1431],
        [0.2225, 0.7169, 0.0606],
        [0.0139, 0.0971, 0.7141],
    ];
    const WHITE: [f32; 3] = [0.9643, 1.0, 0.8251];
    let xyz = M.map(|row| row[0] * lin[0] + row[1] * lin[1] + row[2] * lin[2]);
    let f = |t: f32| {
        if t > 216.0 / 24389.0 {
            t.cbrt()
        } else {
            (24389.0 / 27.0 * t + 16.0) / 116.0
        }
    };
    let [fx, fy, fz] = [0, 1, 2].map(|i| f(xyz[i] / WHITE[i]));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIE Lab (D50) back to sRGB 0–255, clamped.
pub fn from_lab([l, a, b]: [f32; 3]) -> [u8; 3] {
    const WHITE: [f32; 3] = [0.9643, 1.0, 0.8251];
    let fy = (l + 16.0) / 116.0;
    let (fx, fz) = (fy + a / 500.0, fy - b / 200.0);
    let inv = |t: f32| {
        if t.powi(3) > 216.0 / 24389.0 {
            t.powi(3)
        } else {
            (116.0 * t - 16.0) * 27.0 / 24389.0
        }
    };
    let xyz = [inv(fx) * WHITE[0], inv(fy) * WHITE[1], inv(fz) * WHITE[2]];
    const M: [[f32; 3]; 3] = [
        [3.1336, -1.6168, -0.4907],
        [-0.9787, 1.9161, 0.0335],
        [0.0721, -0.2291, 1.4054],
    ];
    let lin = M.map(|row| row[0] * xyz[0] + row[1] * xyz[1] + row[2] * xyz[2]);
    lin.map(|c| {
        let c = c.clamp(0.0, 1.0);
        let v = if c <= 0.003_130_8 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (v * 255.0).round().clamp(0.0, 255.0) as u8
    })
}

/// The mean and standard deviation of L, a and b over some pixels (Match
/// Color's statistics): `[mean L, mean a, mean b, sd L, sd a, sd b]`.
pub fn lab_stats(pixels: impl Iterator<Item = [u8; 3]>) -> [f32; 6] {
    let (mut n, mut sum, mut sq) = (0f64, [0f64; 3], [0f64; 3]);
    for p in pixels {
        let lab = to_lab(p);
        for c in 0..3 {
            sum[c] += lab[c] as f64;
            sq[c] += (lab[c] as f64).powi(2);
        }
        n += 1.0;
    }
    if n == 0.0 {
        return [50.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    }
    let mean = sum.map(|s| s / n);
    let sd = [0, 1, 2].map(|c| (sq[c] / n - mean[c].powi(2)).max(0.0).sqrt().max(1e-3));
    [
        mean[0] as f32,
        mean[1] as f32,
        mean[2] as f32,
        sd[0] as f32,
        sd[1] as f32,
        sd[2] as f32,
    ]
}

/// Match Color: moves a pixel from the target's statistics to the
/// source's (each Lab channel shifted and scaled, Reinhard's transfer),
/// then Luminance (percent) scales the lightness, Color Intensity
/// (percent) the chroma, and Fade (percent) mixes the original back.
pub fn match_color(
    rgb: [u8; 3],
    target: [f32; 6],
    source: [f32; 6],
    luminance: f32,
    intensity: f32,
    fade: f32,
) -> [u8; 3] {
    let lab = to_lab(rgb);
    let mut out = [0f32; 3];
    for c in 0..3 {
        out[c] = (lab[c] - target[c]) * (source[c + 3] / target[c + 3]) + source[c];
    }
    out[0] *= luminance / 100.0;
    out[1] *= intensity / 100.0;
    out[2] *= intensity / 100.0;
    let f = (fade / 100.0).clamp(0.0, 1.0);
    let mixed = [0, 1, 2].map(|c| out[c] + (lab[c] - out[c]) * f);
    from_lab([mixed[0].clamp(0.0, 100.0), mixed[1], mixed[2]])
}

/// Replace Color's selection: how much a pixel belongs to the color
/// sampled (0–1), by its Lab distance and Fuzziness (0–200).
pub fn replace_weight(rgb: [u8; 3], sample: [u8; 3], fuzziness: f32) -> f32 {
    let (a, b) = (to_lab(rgb), to_lab(sample));
    let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    // Fully in within half the fuzziness, fading out to all of it
    let full = fuzziness * 0.5;
    if d <= full {
        1.0
    } else if d >= fuzziness.max(1e-3) {
        0.0
    } else {
        1.0 - (d - full) / (fuzziness - full).max(1e-3)
    }
}

/// Replace Color's sampled colors: the colors picked with the eyedropper
/// and added with the plus eyedropper (up to eight), the colors taken out
/// with the minus eyedropper, and where they were picked (Localized Color
/// Clusters grows the selection from there).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReplaceSamples {
    pub colors: [[u8; 3]; 8],
    pub count: u8,
    pub minus: [[u8; 3]; 8],
    pub minus_count: u8,
    pub seeds: [(u32, u32); 8],
    pub seed_count: u8,
    /// Localized Color Clusters: only the parts of the selection connected
    /// to the picked points.
    pub localized: bool,
}

impl Default for ReplaceSamples {
    fn default() -> Self {
        Self::single([0; 3])
    }
}

impl ReplaceSamples {
    /// One color, picked nowhere in particular.
    pub fn single(color: [u8; 3]) -> Self {
        Self {
            colors: [color; 8],
            count: 1,
            minus: [[0; 3]; 8],
            minus_count: 0,
            seeds: [(0, 0); 8],
            seed_count: 0,
            localized: false,
        }
    }

    /// The eyedropper: `color` (picked at `at`) alone.
    pub fn pick(&mut self, color: [u8; 3], at: Option<(u32, u32)>) {
        let localized = self.localized;
        *self = Self::single(color);
        self.localized = localized;
        if let Some(p) = at {
            self.seeds[0] = p;
            self.seed_count = 1;
        }
    }

    /// The plus eyedropper: `color` joins (the oldest added one gives way
    /// once there are eight).
    pub fn add(&mut self, color: [u8; 3], at: Option<(u32, u32)>) {
        let n = self.count as usize;
        if n < 8 {
            self.colors[n] = color;
            self.count += 1;
        } else {
            self.colors.copy_within(2.., 1);
            self.colors[7] = color;
        }
        if let Some(p) = at {
            let k = self.seed_count as usize;
            if k < 8 {
                self.seeds[k] = p;
                self.seed_count += 1;
            }
        }
    }

    /// The minus eyedropper: pixels like `color` leave the selection.
    pub fn subtract(&mut self, color: [u8; 3]) {
        let n = self.minus_count as usize;
        if n < 8 {
            self.minus[n] = color;
            self.minus_count += 1;
        } else {
            self.minus.copy_within(1.., 0);
            self.minus[7] = color;
        }
    }

    /// The color shown in the dialog's Color swatch: the last one picked.
    pub fn shown(&self) -> [u8; 3] {
        self.colors[(self.count.max(1) - 1) as usize]
    }

    /// How much `rgb` belongs to the selection (0–1): its weight for the
    /// nearest added color, less its weight for the nearest taken-out one.
    pub fn weight(&self, rgb: [u8; 3], fuzziness: f32) -> f32 {
        let plus = self.colors[..self.count.max(1) as usize]
            .iter()
            .map(|&c| replace_weight(rgb, c, fuzziness))
            .fold(0.0, f32::max);
        let minus = self.minus[..self.minus_count as usize]
            .iter()
            .map(|&c| replace_weight(rgb, c, fuzziness))
            .fold(0.0, f32::max);
        plus * (1.0 - minus)
    }
}

/// Localized Color Clusters: the selection's weights over `image`, kept
/// only where they connect (through pixels of some weight, 4-neighbors)
/// to a picked point. Without picked points, the plain weights.
pub fn localized_weights(
    image: &crate::tile::TiledImage,
    samples: &ReplaceSamples,
    fuzziness: f32,
) -> Vec<f32> {
    let (w, h) = (image.width() as usize, image.height() as usize);
    let weights: Vec<f32> = (0..w * h)
        .map(|k| {
            let [r, g, b, _] = image.pixel((k % w) as u32, (k / w) as u32);
            samples.weight([r, g, b], fuzziness)
        })
        .collect();
    if samples.seed_count == 0 {
        return weights;
    }
    let mut keep = vec![false; w * h];
    let mut stack: Vec<usize> = samples.seeds[..samples.seed_count as usize]
        .iter()
        .filter(|&&(x, y)| (x as usize) < w && (y as usize) < h)
        .map(|&(x, y)| y as usize * w + x as usize)
        .filter(|&k| weights[k] > 0.0)
        .collect();
    for &k in &stack {
        keep[k] = true;
    }
    while let Some(k) = stack.pop() {
        let (x, y) = (k % w, k / w);
        let mut visit = |n: usize| {
            if !keep[n] && weights[n] > 0.0 {
                keep[n] = true;
                stack.push(n);
            }
        };
        if x > 0 {
            visit(k - 1);
        }
        if x + 1 < w {
            visit(k + 1);
        }
        if y > 0 {
            visit(k - w);
        }
        if y + 1 < h {
            visit(k + w);
        }
    }
    weights
        .into_iter()
        .zip(keep)
        .map(|(v, k)| if k { v } else { 0.0 })
        .collect()
}

/// A 3D lookup table (Color Lookup): `size`³ RGB entries, red varying
/// fastest (as `.cube` files list them), values 0–1.
#[derive(Clone, Debug, PartialEq)]
pub struct Lut {
    pub size: usize,
    pub data: Vec<[f32; 3]>,
}

impl Lut {
    /// A `.cube` file's table (`LUT_3D_SIZE` and the rows; a
    /// `DOMAIN_MIN`/`DOMAIN_MAX` other than 0–1 is scaled).
    pub fn parse_cube(text: &str) -> Option<Self> {
        let mut size = 0;
        let mut min = [0f32; 3];
        let mut max = [1f32; 3];
        let mut data = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut words = line.split_whitespace();
            let first = words.next()?;
            let nums = |w: std::str::SplitWhitespace| -> Option<[f32; 3]> {
                let v: Vec<f32> = w.filter_map(|t| t.parse().ok()).collect();
                (v.len() == 3).then(|| [v[0], v[1], v[2]])
            };
            match first {
                "LUT_3D_SIZE" => size = words.next()?.parse().ok()?,
                "DOMAIN_MIN" => min = nums(words)?,
                "DOMAIN_MAX" => max = nums(words)?,
                "TITLE" | "LUT_1D_SIZE" | "LUT_1D_INPUT_RANGE" | "LUT_3D_INPUT_RANGE" => {}
                _ => {
                    if let Ok(r) = first.parse::<f32>() {
                        let rest: Vec<f32> = words.filter_map(|t| t.parse().ok()).collect();
                        if rest.len() == 2 {
                            let v = [r, rest[0], rest[1]];
                            data.push(
                                [0, 1, 2].map(|c| (v[c] - min[c]) / (max[c] - min[c]).max(1e-6)),
                            );
                        }
                    }
                }
            }
        }
        (size >= 2 && data.len() == size * size * size).then_some(Self { size, data })
    }

    /// A `.3dl` file's table: a first line of input levels, then `n`³ rows
    /// of integer outputs (blue varying fastest) in the file's bit depth.
    pub fn parse_3dl(text: &str) -> Option<Self> {
        let rows: Vec<Vec<f32>> = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| {
                l.split_whitespace()
                    .filter_map(|t| t.parse().ok())
                    .collect()
            })
            .collect();
        let size = rows.first()?.len();
        let body: Vec<&Vec<f32>> = rows.iter().skip(1).filter(|r| r.len() == 3).collect();
        if size < 2 || body.len() != size * size * size {
            return None;
        }
        let top = body
            .iter()
            .flat_map(|r| r.iter().copied())
            .fold(0f32, f32::max);
        let scale = [1023.0f32, 4095.0, 65535.0]
            .into_iter()
            .find(|&m| top <= m)
            .unwrap_or(top.max(1.0));
        // Reorder from blue-fastest to red-fastest
        let mut data = vec![[0f32; 3]; body.len()];
        for (k, r) in body.iter().enumerate() {
            let (ri, gi, bi) = (k / (size * size), (k / size) % size, k % size);
            data[ri + gi * size + bi * size * size] = [r[0] / scale, r[1] / scale, r[2] / scale];
        }
        Some(Self { size, data })
    }

    /// The color for `rgb`, interpolated trilinearly.
    pub fn apply(&self, rgb: [u8; 3]) -> [u8; 3] {
        self.apply_exact(rgb)
            .map(|c| c.round().clamp(0.0, 255.0) as u8)
    }

    /// [`apply`](Self::apply) before rounding (0–255).
    pub fn apply_exact(&self, rgb: [u8; 3]) -> [f32; 3] {
        let n = self.size;
        let pos = rgb.map(|v| v as f32 / 255.0 * (n - 1) as f32);
        let i0 = pos.map(|p| (p.floor() as usize).min(n - 2));
        let t = [0, 1, 2].map(|c| pos[c] - i0[c] as f32);
        let at = |r: usize, g: usize, b: usize| self.data[r + g * n + b * n * n];
        let mut out = [0f32; 3];
        for (dr, wr) in [(0, 1.0 - t[0]), (1, t[0])] {
            for (dg, wg) in [(0, 1.0 - t[1]), (1, t[1])] {
                for (db, wb) in [(0, 1.0 - t[2]), (1, t[2])] {
                    let v = at(i0[0] + dr, i0[1] + dg, i0[2] + db);
                    let w = wr * wg * wb;
                    for c in 0..3 {
                        out[c] += v[c] * w;
                    }
                }
            }
        }
        out.map(|c| c * 255.0)
    }
}

/// Lookup cubes in use, by number (`Adjustment::ColorLookup` carries the
/// number so the adjustment stays a small copyable value).
static LUTS: Mutex<Vec<Arc<Lut>>> = Mutex::new(Vec::new());

/// Keeps `lut` for adjustments to use; returns its number.
pub fn register(lut: Lut) -> u32 {
    let mut all = LUTS.lock().expect("lut registry");
    if let Some(i) = all.iter().position(|l| **l == lut) {
        return i as u32;
    }
    all.push(Arc::new(lut));
    (all.len() - 1) as u32
}

/// The cube registered as `id`.
pub fn lut(id: u32) -> Option<Arc<Lut>> {
    LUTS.lock().ok()?.get(id as usize).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_round_trips() {
        for rgb in [[0, 0, 0], [255, 255, 255], [200, 100, 50], [12, 200, 240]] {
            assert_eq!(from_lab(to_lab(rgb)), rgb);
        }
        let white = to_lab([255, 255, 255]);
        assert!((white[0] - 100.0).abs() < 0.1 && white[1].abs() < 0.2 && white[2].abs() < 0.2);
    }

    #[test]
    fn match_color_moves_toward_the_source() {
        // A bluish image matched to a reddish one turns warmer
        let target = lab_stats([[60, 80, 200], [80, 100, 220]].into_iter());
        let source = lab_stats([[200, 80, 60], [220, 100, 80]].into_iter());
        let out = match_color([70, 90, 210], target, source, 100.0, 100.0, 0.0);
        assert!(out[0] > out[2], "{out:?}");
        // Fade 100 keeps the original
        let same = match_color([70, 90, 210], target, source, 100.0, 100.0, 100.0);
        assert_eq!(same, [70, 90, 210]);
    }

    #[test]
    fn replace_weight_falls_off_with_fuzziness() {
        assert_eq!(replace_weight([200, 0, 0], [200, 0, 0], 40.0), 1.0);
        assert_eq!(replace_weight([0, 0, 200], [200, 0, 0], 40.0), 0.0);
        let near = replace_weight([190, 20, 10], [200, 0, 0], 40.0);
        assert!(near > 0.0 && near <= 1.0);
    }

    #[test]
    fn cube_and_3dl_luts() {
        // The identity cube of size 2, and one that inverts
        let identity = "LUT_3D_SIZE 2\n0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n";
        let lut = Lut::parse_cube(identity).unwrap();
        assert_eq!(lut.apply([12, 130, 250]), [12, 130, 250]);
        let invert = "LUT_3D_SIZE 2\n1 1 1\n0 1 1\n1 0 1\n0 0 1\n1 1 0\n0 1 0\n1 0 0\n0 0 0\n";
        assert_eq!(
            Lut::parse_cube(invert).unwrap().apply([0, 255, 100]),
            [255, 0, 155]
        );
        // A 3dl identity (blue fastest, 10-bit)
        let mut text = String::from("0 1023\n");
        for r in [0, 1023] {
            for g in [0, 1023] {
                for b in [0, 1023] {
                    text.push_str(&format!("{r} {g} {b}\n"));
                }
            }
        }
        assert_eq!(
            Lut::parse_3dl(&text).unwrap().apply([40, 80, 160]),
            [40, 80, 160]
        );
        assert!(Lut::parse_cube("LUT_3D_SIZE 3\n0 0 0\n").is_none());
        // Registering twice gives the same number
        let a = register(lut.clone());
        assert_eq!(register(lut), a);
    }

    #[test]
    fn replace_samples_add_subtract_and_localize() {
        let red = [220, 30, 30];
        let mut s = ReplaceSamples::single(red);
        assert!(s.weight([220, 30, 30], 40.0) > 0.99);
        assert_eq!(s.weight([30, 30, 220], 40.0), 0.0);
        // Adding blue selects it too; taking blue out again removes it
        s.add([30, 30, 220], None);
        assert!(s.weight([30, 30, 220], 40.0) > 0.99);
        s.subtract([30, 30, 220]);
        assert_eq!(s.weight([30, 30, 220], 40.0), 0.0);
        assert_eq!(s.shown(), [30, 30, 220]);
        // Two red squares apart: localized from a point in the first keeps
        // only that one
        let mut image = crate::tile::TiledImage::new(10, 1);
        for x in 0..10 {
            let c = if x < 3 || x > 6 { red } else { [255, 255, 255] };
            image.set_pixel(x, 0, [c[0], c[1], c[2], 255]);
        }
        let mut s = ReplaceSamples::single(red);
        s.pick(red, Some((1, 0)));
        s.localized = true;
        let w = localized_weights(&image, &s, 40.0);
        assert!(w[0] > 0.99 && w[2] > 0.99);
        assert_eq!(w[8], 0.0);
    }
}
