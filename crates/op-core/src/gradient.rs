//! The Gradient tool (classic gradient): a two-color gradient painted on
//! the active layer, limited to the selection.

use crate::blend;
use crate::document::Document;
use crate::fill::FillError;
use crate::layer::BlendMode;

/// The gradient shape (the five buttons in the options bar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GradientKind {
    #[default]
    Linear,
    Radial,
    Angle,
    Reflected,
    Diamond,
}

impl GradientKind {
    pub const ALL: [Self; 5] = [
        Self::Linear,
        Self::Radial,
        Self::Angle,
        Self::Reflected,
        Self::Diamond,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear Gradient",
            Self::Radial => "Radial Gradient",
            Self::Angle => "Angle Gradient",
            Self::Reflected => "Reflected Gradient",
            Self::Diamond => "Diamond Gradient",
        }
    }

    /// Position along the gradient (0 at the start color, 1 at the end) of
    /// the point (`x`, `y`) for a drag from `a` to `b`.
    pub fn position(self, a: (f32, f32), b: (f32, f32), x: f32, y: f32) -> f32 {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len2 = dx * dx + dy * dy;
        if len2 <= 0.0 {
            return 0.0;
        }
        let len = len2.sqrt();
        let (px, py) = (x - a.0, y - a.1);
        let along = (px * dx + py * dy) / len2;
        match self {
            Self::Linear => along.clamp(0.0, 1.0),
            Self::Radial => ((px * px + py * py).sqrt() / len).min(1.0),
            Self::Reflected => along.abs().min(1.0),
            Self::Diamond => {
                // Distance in the frame turned to the drag direction
                let (ux, uy) = (dx / len, dy / len);
                let u = (px * ux + py * uy).abs();
                let v = (-px * uy + py * ux).abs();
                ((u + v) / len).min(1.0)
            }
            Self::Angle => {
                // Sweeps once around the start point, from the drag's direction
                let start = dy.atan2(dx);
                let angle = py.atan2(px);
                let turn = (start - angle).rem_euclid(std::f32::consts::TAU);
                turn / std::f32::consts::TAU
            }
        }
    }
}

/// How a gradient blends between its colors (Photoshop 2026's Method).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Method {
    /// In OKLab, with the classic easing.
    Perceptual,
    /// In linear light, with the classic easing.
    Linear,
    /// In sRGB values, with the classic easing (Photoshop before 2023).
    Classic,
    /// In OKLab, with a lighter easing (the dialogs' default).
    #[default]
    Smooth,
}

impl Method {
    pub const ALL: [Self; 4] = [Self::Perceptual, Self::Linear, Self::Classic, Self::Smooth];

    pub fn label(self) -> &'static str {
        match self {
            Self::Perceptual => "Perceptual",
            Self::Linear => "Linear",
            Self::Classic => "Classic",
            Self::Smooth => "Smooth",
        }
    }
}

/// The color `t` (0–1) of the way from `a` to `b` (measured against
/// Photoshop 2026's Gradient Map): Photoshop's gradients ease between two
/// stops — the classic easing is the mean of `t` and smoothstep — and
/// Smooth eases 40% as much.
pub fn blend_colors(a: [u8; 3], b: [u8; 3], t: f32, method: Method) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    let smoothstep = t * t * (3.0 - 2.0 * t);
    let classic = (t + smoothstep) / 2.0;
    let round = |v: f32| (v * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8;
    match method {
        Method::Classic => {
            [0, 1, 2].map(|c| round((a[c] as f32 + (b[c] as f32 - a[c] as f32) * classic) / 255.0))
        }
        Method::Linear => {
            let (la, lb) = (a.map(srgb_to_linear), b.map(srgb_to_linear));
            [0, 1, 2].map(|c| round(linear_to_srgb(la[c] + (lb[c] - la[c]) * classic)))
        }
        Method::Perceptual | Method::Smooth => {
            let k = if method == Method::Perceptual {
                classic
            } else {
                t + (classic - t) * 0.4
            };
            let (oa, ob) = (oklab(a), oklab(b));
            let mixed = [0, 1, 2].map(|i| oa[i] + (ob[i] - oa[i]) * k);
            from_oklab(mixed).map(round)
        }
    }
}

fn srgb_to_linear(v: u8) -> f32 {
    let v = v as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB to OKLab (Björn Ottosson's).
fn oklab(c: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = c.map(srgb_to_linear);
    let l = 0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let [l, m, s] = [l, m, s].map(f32::cbrt);
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// OKLab back to sRGB (0–1, clipped).
fn from_oklab([ll, a, b]: [f32; 3]) -> [f32; 3] {
    let l = ll + 0.396_337_78 * a + 0.215_803_76 * b;
    let m = ll - 0.105_561_346 * a - 0.063_854_17 * b;
    let s = ll - 0.089_484_18 * a - 1.291_485_5 * b;
    let [l, m, s] = [l, m, s].map(|v| v * v * v);
    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_93 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
    .map(linear_to_srgb)
}

/// A color stop: where (0–1), the color, and where between it and the
/// next stop the halfway color falls (the midpoint, 0.05–0.95 of the way).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorStop {
    pub location: f32,
    pub color: [u8; 3],
    pub midpoint: f32,
}

/// An opacity stop (0–1), with its midpoint like a color stop's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpacityStop {
    pub location: f32,
    pub opacity: f32,
    pub midpoint: f32,
}

/// A gradient as the Gradient Editor makes it: color and opacity stops
/// (sorted by location), how it blends (Method) and Smoothness (0–1:
/// how much of the stops' easing is used).
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub name: String,
    pub colors: Vec<ColorStop>,
    pub opacities: Vec<OpacityStop>,
    pub method: Method,
    pub smoothness: f32,
}

impl Gradient {
    /// From `a` to `b`, opaque (Foreground to Background).
    pub fn two(name: &str, a: [u8; 3], b: [u8; 3]) -> Self {
        Self {
            name: name.to_owned(),
            colors: vec![
                ColorStop {
                    location: 0.0,
                    color: a,
                    midpoint: 0.5,
                },
                ColorStop {
                    location: 1.0,
                    color: b,
                    midpoint: 0.5,
                },
            ],
            opacities: vec![
                OpacityStop {
                    location: 0.0,
                    opacity: 1.0,
                    midpoint: 0.5,
                },
                OpacityStop {
                    location: 1.0,
                    opacity: 1.0,
                    midpoint: 0.5,
                },
            ],
            method: Method::default(),
            smoothness: 1.0,
        }
    }

    /// The gradient the other way round (Reverse).
    pub fn reversed(&self) -> Self {
        let mut colors: Vec<ColorStop> = self
            .colors
            .iter()
            .map(|s| ColorStop {
                location: 1.0 - s.location,
                ..*s
            })
            .collect();
        let mut opacities: Vec<OpacityStop> = self
            .opacities
            .iter()
            .map(|s| OpacityStop {
                location: 1.0 - s.location,
                ..*s
            })
            .collect();
        colors.reverse();
        opacities.reverse();
        // Each midpoint belongs to the stop before it, so they shift
        let shift = |m: Vec<f32>| -> Vec<f32> {
            let n = m.len();
            (0..n)
                .map(|i| if i + 1 < n { 1.0 - m[n - 2 - i] } else { 0.5 })
                .collect()
        };
        let cm = shift(colors.iter().map(|s| s.midpoint).collect());
        let om = shift(opacities.iter().map(|s| s.midpoint).collect());
        for (s, m) in colors.iter_mut().zip(cm) {
            s.midpoint = m;
        }
        for (s, m) in opacities.iter_mut().zip(om) {
            s.midpoint = m;
        }
        Self {
            colors,
            opacities,
            ..self.clone()
        }
    }

    /// The span `t` falls in among stops at `locations`, and how far along
    /// it (0–1) after its midpoint's remapping.
    fn locate(locations: &[(f32, f32)], t: f32) -> (usize, f32) {
        let n = locations.len();
        if n < 2 || t <= locations[0].0 {
            return (0, 0.0);
        }
        if t >= locations[n - 1].0 {
            return (n - 2, 1.0);
        }
        let i = (0..n - 1)
            .find(|&i| t <= locations[i + 1].0)
            .unwrap_or(n - 2);
        let (a, mid) = locations[i];
        let b = locations[i + 1].0;
        let u = ((t - a) / (b - a).max(1e-6)).clamp(0.0, 1.0);
        let mid = mid.clamp(0.05, 0.95);
        let v = if u < mid {
            u / mid * 0.5
        } else {
            0.5 + (u - mid) / (1.0 - mid) * 0.5
        };
        (i, v)
    }

    /// The color and opacity at `t` (0–1).
    pub fn sample(&self, t: f32) -> ([u8; 3], f32) {
        let t = t.clamp(0.0, 1.0);
        let color = match self.colors.len() {
            0 => [0; 3],
            1 => self.colors[0].color,
            _ => {
                let at: Vec<(f32, f32)> = self
                    .colors
                    .iter()
                    .map(|s| (s.location, s.midpoint))
                    .collect();
                let (i, v) = Self::locate(&at, t);
                // Smoothness 0 is a straight blend; 1 the method's easing
                let (a, b) = (self.colors[i].color, self.colors[i + 1].color);
                let eased = blend_colors(a, b, v, self.method);
                let straight = [0, 1, 2]
                    .map(|c| (a[c] as f32 + (b[c] as f32 - a[c] as f32) * v).round() as u8);
                let k = self.smoothness.clamp(0.0, 1.0);
                [0, 1, 2].map(|c| {
                    (straight[c] as f32 + (eased[c] as f32 - straight[c] as f32) * k).round() as u8
                })
            }
        };
        let opacity = match self.opacities.len() {
            0 => 1.0,
            1 => self.opacities[0].opacity,
            _ => {
                let at: Vec<(f32, f32)> = self
                    .opacities
                    .iter()
                    .map(|s| (s.location, s.midpoint))
                    .collect();
                let (i, v) = Self::locate(&at, t);
                let (a, b) = (self.opacities[i].opacity, self.opacities[i + 1].opacity);
                a + (b - a) * v
            }
        };
        (color, opacity)
    }

    /// 256 colors along the gradient (Gradient Map's table: luminosity to
    /// color).
    pub fn table(&self) -> [[u8; 3]; 256] {
        std::array::from_fn(|i| self.sample(i as f32 / 255.0).0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientOptions {
    pub kind: GradientKind,
    pub mode: BlendMode,
    pub opacity: f32,
    /// Swaps the start and end colors.
    pub reverse: bool,
}

impl Default for GradientOptions {
    fn default() -> Self {
        Self {
            kind: GradientKind::Linear,
            mode: BlendMode::Normal,
            opacity: 1.0,
            reverse: false,
        }
    }
}

/// Paints a gradient from `from` (at `a`) to `to` (at `b`) on the active
/// layer. Colors are interpolated in RGB; partly selected pixels get a
/// proportional amount; the background layer and layers with locked
/// transparency keep their alpha.
pub fn gradient(
    doc: &mut Document,
    a: (f32, f32),
    b: (f32, f32),
    colors: ([u8; 3], [u8; 3]),
    options: GradientOptions,
) -> Result<(), FillError> {
    let mut g = Gradient::two("", colors.0, colors.1);
    g.method = Method::Classic;
    g.smoothness = 0.0;
    gradient_with(doc, a, b, &g, options, false)
}

/// [`gradient`] with a gradient of any stops (and opacities): each pixel
/// takes the gradient's color at its place, at its opacity; Dither adds
/// up to half a level of noise so smooth ramps don't band.
pub fn gradient_with(
    doc: &mut Document,
    a: (f32, f32),
    b: (f32, f32),
    gradient: &Gradient,
    options: GradientOptions,
    dither: bool,
) -> Result<(), FillError> {
    // Quick Mask can be painted whatever the layer's state
    if doc.quick_mask.is_none() {
        crate::adjust::check(doc)?;
    }
    let g = if options.reverse {
        gradient.reversed()
    } else {
        gradient.clone()
    };
    // 1024 places along it are plenty
    let table: Vec<([f32; 3], f32)> = (0..1024)
        .map(|i| {
            let (c, o) = g.sample(i as f32 / 1023.0);
            (c.map(|v| v as f32 / 255.0), o)
        })
        .collect();
    let selection = doc.selection().cloned();
    let (w, h) = (doc.width, doc.height);
    let target = doc.edit_target().expect("checked");
    let keep_alpha = target.keep_alpha;
    // On a mask the gradient runs between the colors' grays
    let mask = target.mask;
    let image = target.image;
    for y in 0..h {
        for x in 0..w {
            let amount = options.opacity
                * selection
                    .as_ref()
                    .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
            if amount <= 0.0 {
                continue;
            }
            let base = image.pixel(x, y);
            if keep_alpha && base[3] == 0 {
                continue;
            }
            let t = options.kind.position(a, b, x as f32 + 0.5, y as f32 + 0.5);
            // Between two of the table's places
            let f = t.clamp(0.0, 1.0) * 1023.0;
            let (i, k) = ((f.floor() as usize).min(1022), f - f.floor());
            let ((c0, o0), (c1, o1)) = (table[i], table[i + 1]);
            let mut src = [0, 1, 2].map(|c| c0[c] + (c1[c] - c0[c]) * k);
            let opacity = o0 + (o1 - o0) * k;
            if mask {
                let [r, g, b] = src.map(|v| (v * 255.0).round() as u8);
                src = [crate::adjust::mask_gray([r, g, b])[0] as f32 / 255.0; 3];
            }
            if dither {
                let n = (crate::adjust::dither_noise(x, y) - 0.5) / 255.0;
                src = src.map(|v| v + n);
            }
            let amount = amount * opacity;
            if amount <= 0.0 {
                continue;
            }
            let dst = base.map(|v| v as f32 / 255.0);
            let out = if keep_alpha {
                let mixed = blend::composite(
                    options.mode,
                    [dst[0], dst[1], dst[2], 1.0],
                    src,
                    amount,
                    x,
                    y,
                );
                [mixed[0], mixed[1], mixed[2], dst[3]]
            } else {
                blend::composite(options.mode, dst, src, amount, x, y)
            };
            let px = out.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
            if px != base {
                image.set_pixel(x, y, px);
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

    fn row(doc: &Document) -> Vec<u8> {
        doc.composite_rgba8().chunks(4).map(|p| p[0]).collect()
    }

    #[test]
    fn positions_of_each_kind() {
        let (a, b) = ((0.0, 0.0), (10.0, 0.0));
        assert_eq!(GradientKind::Linear.position(a, b, 5.0, 3.0), 0.5);
        assert_eq!(GradientKind::Linear.position(a, b, -4.0, 0.0), 0.0);
        assert_eq!(GradientKind::Radial.position(a, b, 0.0, 5.0), 0.5);
        assert_eq!(GradientKind::Reflected.position(a, b, -5.0, 0.0), 0.5);
        assert_eq!(GradientKind::Diamond.position(a, b, 2.5, 2.5), 0.5);
        assert_eq!(GradientKind::Angle.position(a, b, 5.0, 0.0), 0.0);
        let quarter = GradientKind::Angle.position(a, b, 0.0, -5.0);
        assert!((quarter - 0.25).abs() < 1e-6, "{quarter}");
    }

    #[test]
    fn stops_midpoints_opacity_and_reverse() {
        let mut g = Gradient::two("t", [0, 0, 0], [255, 255, 255]);
        g.method = Method::Classic;
        g.smoothness = 0.0;
        assert_eq!(g.sample(0.0).0, [0; 3]);
        assert_eq!(g.sample(1.0).0, [255; 3]);
        assert_eq!(g.sample(0.5).0, [128; 3]);
        // A midpoint at 25%: the halfway gray comes a quarter of the way
        g.colors[0].midpoint = 0.25;
        assert_eq!(g.sample(0.25).0, [128; 3]);
        // A third stop in the middle (red)
        g.colors.insert(
            1,
            ColorStop {
                location: 0.5,
                color: [255, 0, 0],
                midpoint: 0.5,
            },
        );
        g.colors[0].midpoint = 0.5;
        assert_eq!(g.sample(0.5).0, [255, 0, 0]);
        assert_eq!(g.sample(0.75).0, [255, 128, 128]);
        // Opacity from opaque to clear
        g.opacities[1].opacity = 0.0;
        assert!((g.sample(0.5).1 - 0.5).abs() < 1e-6);
        // Reversed: red stays in the middle, the ends swap
        let r = g.reversed();
        assert_eq!(r.sample(0.0).0, [255; 3]);
        assert_eq!(r.sample(0.5).0, [255, 0, 0]);
        assert!((r.sample(0.0).1).abs() < 1e-6);
        // Gradient Map's table
        assert_eq!(g.table()[255], [255; 3]);
    }

    #[test]
    fn painting_stops_with_opacity_and_dither() {
        let mut d = Document::new_with_background("t", 5, 1, Color::WHITE);
        let mut g = Gradient::two("t", [0, 0, 0], [0, 0, 0]);
        g.opacities[1].opacity = 0.0;
        gradient_with(
            &mut d,
            (0.5, 0.5),
            (4.5, 0.5),
            &g,
            GradientOptions::default(),
            false,
        )
        .unwrap();
        // Black fading out over white
        assert_eq!(row(&d), [0, 64, 128, 191, 255]);
        // Dither: a long flat ramp varies by at most a level
        let mut d = Document::new_with_background("t", 64, 1, Color::WHITE);
        let g = Gradient::two("t", [100, 100, 100], [101, 101, 101]);
        gradient_with(
            &mut d,
            (0.5, 0.5),
            (63.5, 0.5),
            &g,
            GradientOptions::default(),
            true,
        )
        .unwrap();
        let r = row(&d);
        assert!(r.iter().all(|&v| (99..=102).contains(&v)));
    }

    #[test]
    fn paints_black_to_white() {
        let mut d = Document::new_with_background("t", 5, 1, Color::WHITE);
        let g = GradientOptions::default();
        gradient(&mut d, (0.5, 0.5), (4.5, 0.5), ([0; 3], [255; 3]), g).unwrap();
        assert_eq!(row(&d), [0, 64, 128, 191, 255]);
        let reversed = GradientOptions { reverse: true, ..g };
        gradient(&mut d, (0.5, 0.5), (4.5, 0.5), ([0; 3], [255; 3]), reversed).unwrap();
        assert_eq!(row(&d), [255, 191, 128, 64, 0]);
    }

    #[test]
    fn half_opacity_and_selection() {
        let mut d = Document::new_with_background("t", 2, 1, Color::WHITE);
        d.set_selection(Some(crate::selection::Selection::rect(
            2,
            1,
            crate::selection::Rect::new(0.0, 0.0, 1.0, 1.0),
        )));
        let g = GradientOptions {
            opacity: 0.5,
            ..Default::default()
        };
        gradient(&mut d, (0.0, 0.0), (1.0, 0.0), ([0; 3], [0; 3]), g).unwrap();
        assert_eq!(row(&d), [128, 255]);
    }
}
