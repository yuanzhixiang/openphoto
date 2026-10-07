//! Healing: Photoshop's healing tools keep the texture of a source and take
//! the shading of the place they repair. The difference between target and
//! source along the repaired area's edge is spread smoothly inside it (a
//! membrane: Laplace's equation with those edge values), and the result
//! is the source plus that membrane, so the repair matches its surroundings
//! at the edge without a seam.

use crate::document::Document;
use crate::layer::BlendMode;
use crate::selection::Selection;
use crate::tile::TiledImage;

/// The healing tools' Mode menu: how the healed pixels combine with the
/// ones they repair.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HealMode {
    #[default]
    Normal,
    /// Keeps the grain at a soft brush's edge: a pixel there takes the
    /// healed color or keeps its own (dithered by coverage) instead of
    /// a mix.
    Replace,
    Multiply,
    Screen,
    Darken,
    Lighten,
    Color,
    Luminosity,
}

impl HealMode {
    pub const ALL: [Self; 8] = [
        Self::Normal,
        Self::Replace,
        Self::Multiply,
        Self::Screen,
        Self::Darken,
        Self::Lighten,
        Self::Color,
        Self::Luminosity,
    ];

    /// The healed color `healed` over `under` by this mode.
    pub fn apply(self, under: [f32; 3], healed: [f32; 3]) -> [f32; 3] {
        let mode = match self {
            Self::Normal | Self::Replace => return healed,
            Self::Multiply => BlendMode::Multiply,
            Self::Screen => BlendMode::Screen,
            Self::Darken => BlendMode::Darken,
            Self::Lighten => BlendMode::Lighten,
            Self::Color => BlendMode::Color,
            Self::Luminosity => BlendMode::Luminosity,
        };
        let unit = |c: [f32; 3]| c.map(|v| v / 255.0);
        crate::blend::blend(mode, unit(under), unit(healed)).map(|v| (v * 255.0).clamp(0.0, 255.0))
    }
}

/// How a repair is made: the Mode menu, and Diffusion (1–7: how far the
/// shading at the area's edge reaches in; None is the Legacy healing,
/// which spreads it all the way).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealStyle {
    pub mode: HealMode,
    pub diffusion: Option<u8>,
}

impl Default for HealStyle {
    fn default() -> Self {
        Self {
            mode: HealMode::Normal,
            diffusion: None,
        }
    }
}

impl HealStyle {
    /// The membrane's damping for this Diffusion: the edge's shading fades
    /// over about 3 × 2^(d−1) pixels (3 at 1, 48 at the default 5, 192 at
    /// 7); none for Legacy.
    fn damping(self) -> f32 {
        match self.diffusion {
            None => 0.0,
            Some(d) => {
                let reach = 3.0 * 2f32.powi(d.clamp(1, 7) as i32 - 1);
                1.0 / (reach * reach)
            }
        }
    }
}

/// Whether a Replace-mode pixel at (x, y) with `coverage` takes the healed
/// color: a fixed dither, so repeated dabs agree.
pub fn replace_takes(x: u32, y: u32, coverage: f32) -> bool {
    let mut v = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    v ^= v >> 31;
    v = v.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    v ^= v >> 33;
    ((v >> 40) as f32 / (1u64 << 24) as f32) < coverage
}

/// Solves Laplace's equation on a `w` × `h` window: values of the pixels
/// outside `inside` stay as in `values`; those inside become the smooth
/// interpolation of their neighbors (successive over-relaxation until
/// the largest change is below a hundredth of a level).
pub fn membrane(w: usize, h: usize, inside: &[bool], values: &mut [[f32; 3]]) {
    membrane_damped(w, h, inside, values, 0.0);
}

/// [`membrane`] with `damping` (≥ 0): the screened equation
/// ∇²u = damping·u, so the edge values fade toward 0 inside over about
/// 1/√damping pixels.
pub fn membrane_damped(w: usize, h: usize, inside: &[bool], values: &mut [[f32; 3]], damping: f32) {
    let cells: Vec<usize> = (0..w * h).filter(|&i| inside[i]).collect();
    if cells.is_empty() {
        return;
    }
    // Start inside from the mean of the edge for faster convergence
    let mut sum = [0f32; 3];
    let mut n = 0f32;
    for i in 0..w * h {
        if !inside[i] {
            let (x, y) = (i % w, i / w);
            let touches = [
                (x > 0, i.wrapping_sub(1)),
                (x + 1 < w, i + 1),
                (y > 0, i.wrapping_sub(w)),
                (y + 1 < h, i + w),
            ]
            .iter()
            .any(|&(ok, j)| ok && inside[j]);
            if touches {
                for c in 0..3 {
                    sum[c] += values[i][c];
                }
                n += 1.0;
            }
        }
    }
    if n > 0.0 {
        let mean = sum.map(|s| s / n);
        for &i in &cells {
            values[i] = mean;
        }
    }
    let omega = 1.9f32;
    let max_iter = (2 * w.max(h)).clamp(30, 4000);
    for _ in 0..max_iter {
        let mut change = 0f32;
        for &i in &cells {
            let (x, y) = (i % w, i / w);
            let mut acc = [0f32; 3];
            let mut k = 0f32;
            for (ok, j) in [
                (x > 0, i.wrapping_sub(1)),
                (x + 1 < w, i + 1),
                (y > 0, i.wrapping_sub(w)),
                (y + 1 < h, i + w),
            ] {
                if ok {
                    for c in 0..3 {
                        acc[c] += values[j][c];
                    }
                    k += 1.0;
                }
            }
            if k == 0.0 {
                continue;
            }
            for c in 0..3 {
                let target = acc[c] / (k + damping);
                let d = (target - values[i][c]) * omega;
                values[i][c] += d;
                change = change.max(d.abs());
            }
        }
        if change < 0.01 {
            break;
        }
    }
}

/// Heals the pixels of `image` where `weight` (row by row over the window
/// `x0`, `y0`, `w` × `h`) is above zero with the texture of `source` moved
/// by (`dx`, `dy`) (a pixel takes the source's pixel at (x − dx, y − dy)),
/// matched to `image` along the area's edge. Returns the healed colors
/// (straight RGB, 0–255) for the window; pixels outside the area keep
/// `image`'s. `None` when the source falls outside its image.
pub fn heal_window(
    image: &TiledImage,
    source: &TiledImage,
    window: (u32, u32, usize, usize),
    inside: &[bool],
    offset: (i64, i64),
) -> Option<Vec<[f32; 3]>> {
    heal_window_with(image, source, window, inside, offset, HealStyle::default())
}

/// [`heal_window`] with a style: the membrane damped by its Diffusion and
/// the healed colors combined with `image`'s by its mode.
pub fn heal_window_with(
    image: &TiledImage,
    source: &TiledImage,
    (x0, y0, w, h): (u32, u32, usize, usize),
    inside: &[bool],
    (dx, dy): (i64, i64),
    style: HealStyle,
) -> Option<Vec<[f32; 3]>> {
    let rgb = |p: [u8; 4]| [p[0] as f32, p[1] as f32, p[2] as f32];
    let src = |x: u32, y: u32| -> Option<[f32; 3]> {
        let (sx, sy) = (x as i64 - dx, y as i64 - dy);
        if sx < 0 || sy < 0 || sx >= source.width() as i64 || sy >= source.height() as i64 {
            return None;
        }
        Some(rgb(source.pixel(sx as u32, sy as u32)))
    };
    // The correction: target − source on the edge, spread inside
    let mut corr = vec![[0f32; 3]; w * h];
    let mut sources = vec![[0f32; 3]; w * h];
    for j in 0..h {
        for i in 0..w {
            let (x, y) = (x0 + i as u32, y0 + j as u32);
            let s = src(x, y)?;
            let t = rgb(image.pixel(x, y));
            sources[j * w + i] = s;
            corr[j * w + i] = [t[0] - s[0], t[1] - s[1], t[2] - s[2]];
        }
    }
    membrane_damped(w, h, inside, &mut corr, style.damping());
    Some(
        (0..w * h)
            .map(|k| {
                let (x, y) = (x0 + (k % w) as u32, y0 + (k / w) as u32);
                let under = rgb(image.pixel(x, y));
                if inside[k] {
                    let healed = [0, 1, 2].map(|c| (sources[k][c] + corr[k][c]).clamp(0.0, 255.0));
                    style.mode.apply(under, healed)
                } else {
                    under
                }
            })
            .collect(),
    )
}

/// The Spot Healing Brush's Create Texture: a `w` × `h` image for the
/// window at (`x0`, `y0`) whose pixels inside a dab of `radius` at
/// (`cx`, `cy`) come from `image` mirrored through the dab's edge (a pixel
/// at distance ρ from the center takes the one at 2·radius + 2 − ρ, in the
/// same direction), so the texture around the dab fills it. Pixels outside
/// keep their own. Heal with it at offset (x0, y0).
pub fn mirrored_texture(
    image: &TiledImage,
    (x0, y0, w, h): (u32, u32, usize, usize),
    (cx, cy): (f32, f32),
    radius: f32,
) -> TiledImage {
    let (iw, ih) = (image.width() as f32, image.height() as f32);
    let mut out = TiledImage::new(w as u32, h as u32);
    for j in 0..h {
        for i in 0..w {
            let (x, y) = (x0 + i as u32, y0 + j as u32);
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let rho = (dx * dx + dy * dy).sqrt();
            let (sx, sy) = if rho <= radius + 1.0 {
                let (ux, uy) = if rho > 1e-3 {
                    (dx / rho, dy / rho)
                } else {
                    (1.0, 0.0)
                };
                let far = 2.0 * radius + 2.0 - rho;
                (cx + ux * far, cy + uy * far)
            } else {
                (x as f32 + 0.5, y as f32 + 0.5)
            };
            // Reflect at the image's edges
            let fold = |v: f32, n: f32| {
                let v = v.abs();
                let v = if v >= n {
                    (2.0 * n - v - 1.0).max(0.0)
                } else {
                    v
                };
                v.min(n - 1.0) as u32
            };
            out.set_pixel(i as u32, j as u32, image.pixel(fold(sx, iw), fold(sy, ih)));
        }
    }
    out
}

/// Where the Spot Healing Brush takes its source for a dab of `radius` at
/// (`cx`, `cy`): of the spots one and a half diameters away in eight
/// directions, the one whose surroundings (a ring just outside the dab)
/// look most like the dab's own. Returns the offset (target − source).
pub fn proximity_match(
    image: &TiledImage,
    (cx, cy): (f32, f32),
    radius: f32,
) -> Option<(i64, i64)> {
    let (w, h) = (image.width() as i64, image.height() as i64);
    let ring: Vec<(i64, i64)> = {
        let (r0, r1) = (radius + 1.0, radius + 4.0);
        let n = (r1.ceil() as i64) + 1;
        let mut v = Vec::new();
        for y in -n..=n {
            for x in -n..=n {
                let d = ((x * x + y * y) as f32).sqrt();
                if d >= r0 && d <= r1 {
                    v.push((x, y));
                }
            }
        }
        v
    };
    let center = (cx.round() as i64, cy.round() as i64);
    let dist = (radius * 3.0).max(4.0);
    let mut best: Option<((i64, i64), f32)> = None;
    for k in 0..8 {
        let a = std::f32::consts::FRAC_PI_4 * k as f32;
        let off = (
            (a.cos() * dist).round() as i64,
            (a.sin() * dist).round() as i64,
        );
        let mut cost = 0f32;
        let mut count = 0f32;
        let mut ok = true;
        for &(rx, ry) in &ring {
            let (tx, ty) = (center.0 + rx, center.1 + ry);
            let (sx, sy) = (tx - off.0, ty - off.1);
            if sx < 0 || sy < 0 || sx >= w || sy >= h {
                ok = false;
                break;
            }
            if tx < 0 || ty < 0 || tx >= w || ty >= h {
                continue;
            }
            let (t, s) = (
                image.pixel(tx as u32, ty as u32),
                image.pixel(sx as u32, sy as u32),
            );
            cost += (0..3)
                .map(|c| (t[c] as f32 - s[c] as f32).abs())
                .sum::<f32>();
            count += 1.0;
        }
        // The source's own area must lie in the image too
        let r = radius.ceil() as i64;
        let (sx, sy) = (center.0 - off.0, center.1 - off.1);
        if !ok || sx - r < 0 || sy - r < 0 || sx + r >= w || sy + r >= h || count == 0.0 {
            continue;
        }
        let cost = cost / count;
        if best.is_none_or(|(_, c)| cost < c) {
            best = Some((off, cost));
        }
    }
    best.map(|(o, _)| o)
}

/// The Patch tool (Source mode): the selected area of the active layer is
/// healed with the texture found where the selection was dragged to
/// (offset `dx`, `dy` from the selection), blended by the selection's
/// value. Returns whether anything changed.
pub fn patch(doc: &mut Document, selection: &Selection, offset: (i64, i64)) -> bool {
    let Some(source) = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .and_then(|l| l.image())
        .cloned()
    else {
        return false;
    };
    patch_with(
        doc,
        selection,
        &source,
        (-offset.0, -offset.1),
        HealStyle::default(),
        false,
    )
}

/// The Patch tool healing the selection from `source` (target (x, y)
/// takes the source at (x − dx, y − dy); Use Pattern passes the pattern
/// tiled over the layer with no offset), in `style`. With `transparent`
/// only the source's texture (its difference from its 5 × 5 average) goes
/// over the layer's own colors. Blended by the selection's value; alpha is
/// kept. Returns whether anything changed.
pub fn patch_with(
    doc: &mut Document,
    selection: &Selection,
    source: &TiledImage,
    offset: (i64, i64),
    style: HealStyle,
    transparent: bool,
) -> bool {
    let Some(id) = doc.active_layer else {
        return false;
    };
    let Some((bx0, by0, bx1, by1)) = selection.bounds() else {
        return false;
    };
    let Some(image) = doc.layer(id).and_then(|l| l.image()) else {
        return false;
    };
    let (iw, ih) = (image.width(), image.height());
    // A one-pixel edge around the area for its boundary values
    let x0 = bx0.saturating_sub(1);
    let y0 = by0.saturating_sub(1);
    let x1 = (bx1 + 1).min(iw);
    let y1 = (by1 + 1).min(ih);
    let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let inside: Vec<bool> = (0..w * h)
        .map(|k| selection.get(x0 + (k % w) as u32, y0 + (k / w) as u32) > 0)
        .collect();
    let Some(mut healed) = heal_window_with(image, source, (x0, y0, w, h), &inside, offset, style)
    else {
        return false;
    };
    if transparent {
        healed = texture_over(image, &healed, (x0, y0, w, h));
    }
    let Some(image) = doc.layer_mut(id).and_then(|l| l.image_mut()) else {
        return false;
    };
    for k in 0..w * h {
        if !inside[k] {
            continue;
        }
        let (x, y) = (x0 + (k % w) as u32, y0 + (k / w) as u32);
        let a = selection.get(x, y) as f32 / 255.0;
        let p = image.pixel(x, y);
        let out = [0, 1, 2].map(|c| (p[c] as f32 + (healed[k][c] - p[c] as f32) * a).round() as u8);
        image.set_pixel(x, y, [out[0], out[1], out[2], p[3]]);
    }
    doc.mark_dirty();
    true
}

/// Transparent: `healed`'s texture (its difference from its own 5 × 5
/// average) over `image`'s colors, for the window.
fn texture_over(
    image: &TiledImage,
    healed: &[[f32; 3]],
    (x0, y0, w, h): (u32, u32, usize, usize),
) -> Vec<[f32; 3]> {
    (0..w * h)
        .map(|k| {
            let (i, j) = ((k % w) as i64, (k / w) as i64);
            let mut avg = [0f32; 3];
            let mut n = 0.0;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let (a, b) = (i + dx, j + dy);
                    if a >= 0 && b >= 0 && (a as usize) < w && (b as usize) < h {
                        let q = healed[b as usize * w + a as usize];
                        for c in 0..3 {
                            avg[c] += q[c];
                        }
                        n += 1.0;
                    }
                }
            }
            let p = image.pixel(x0 + i as u32, y0 + j as u32);
            [0, 1, 2].map(|c| (p[c] as f32 + healed[k][c] - avg[c] / n).clamp(0.0, 255.0))
        })
        .collect()
}

/// `pattern` tiled over a `w` × `h` image from its top-left corner: Use
/// Pattern's and the Healing Brush's Pattern source.
pub fn tiled_pattern(pattern: &TiledImage, w: u32, h: u32) -> TiledImage {
    let mut out = TiledImage::new(w, h);
    let (pw, ph) = (pattern.width(), pattern.height());
    if pw == 0 || ph == 0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let [r, g, b, _] = pattern.pixel(x % pw, y % ph);
            out.set_pixel(x, y, [r, g, b, 255]);
        }
    }
    out
}

/// The Patch tool's Destination mode and Content-Aware Move's Extend: the
/// selected pixels are copied by (`dx`, `dy`), healed into their new
/// place; the original stays. Returns whether anything changed.
pub fn patch_to(doc: &mut Document, selection: &Selection, (dx, dy): (i64, i64)) -> bool {
    let Some(id) = doc.active_layer else {
        return false;
    };
    let Some(original) = doc.layer(id).and_then(|l| l.image()).cloned() else {
        return false;
    };
    let Some(result) = place_healed(&original, &original, selection, (dx, dy)) else {
        return false;
    };
    let Some(image) = doc.layer_mut(id).and_then(|l| l.image_mut()) else {
        return false;
    };
    *image = result;
    doc.mark_dirty();
    true
}

/// `onto` with the selected pixels of `source` moved by (`dx`, `dy`) and
/// healed into it.
fn place_healed(
    onto: &TiledImage,
    source: &TiledImage,
    selection: &Selection,
    (dx, dy): (i64, i64),
) -> Option<TiledImage> {
    let (bx0, by0, bx1, by1) = selection.bounds()?;
    let (iw, ih) = (onto.width() as i64, onto.height() as i64);
    let x0 = (bx0 as i64 + dx - 1).clamp(0, iw) as u32;
    let y0 = (by0 as i64 + dy - 1).clamp(0, ih) as u32;
    let x1 = (bx1 as i64 + dx + 1).clamp(0, iw) as u32;
    let y1 = (by1 as i64 + dy + 1).clamp(0, ih) as u32;
    let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let value = |k: usize| {
        let (x, y) = (
            x0 as i64 + (k % w) as i64 - dx,
            y0 as i64 + (k / w) as i64 - dy,
        );
        if x < 0 || y < 0 {
            0
        } else {
            selection.get(x as u32, y as u32)
        }
    };
    let inside: Vec<bool> = (0..w * h).map(|k| value(k) > 0).collect();
    let healed = heal_window(onto, source, (x0, y0, w, h), &inside, (dx, dy))?;
    let mut result = onto.clone();
    for k in 0..w * h {
        if !inside[k] {
            continue;
        }
        let (x, y) = (x0 + (k % w) as u32, y0 + (k / w) as u32);
        let a = value(k) as f32 / 255.0;
        let p = result.pixel(x, y);
        let out = [0, 1, 2].map(|c| (p[c] as f32 + (healed[k][c] - p[c] as f32) * a).round() as u8);
        result.set_pixel(x, y, [out[0], out[1], out[2], p[3]]);
    }
    Some(result)
}

/// Content-Aware Move: the selected pixels move by (`dx`, `dy`), healed
/// into their new place, and the area they leave is filled smoothly from
/// its surroundings. Returns whether anything changed.
pub fn content_aware_move(doc: &mut Document, selection: &Selection, (dx, dy): (i64, i64)) -> bool {
    let Some(id) = doc.active_layer else {
        return false;
    };
    let Some((bx0, by0, bx1, by1)) = selection.bounds() else {
        return false;
    };
    let Some(original) = doc.layer(id).and_then(|l| l.image()).cloned() else {
        return false;
    };
    let (iw, ih) = (original.width(), original.height());
    let window = |x0: i64, y0: i64, x1: i64, y1: i64| {
        let x0 = x0.clamp(0, iw as i64) as u32;
        let y0 = y0.clamp(0, ih as i64) as u32;
        let x1 = x1.clamp(0, iw as i64) as u32;
        let y1 = y1.clamp(0, ih as i64) as u32;
        (x0, y0, (x1 - x0) as usize, (y1 - y0) as usize)
    };
    // Fill the hole: the membrane of its surroundings
    let (x0, y0, w, h) = window(
        bx0 as i64 - 1,
        by0 as i64 - 1,
        bx1 as i64 + 1,
        by1 as i64 + 1,
    );
    let inside: Vec<bool> = (0..w * h)
        .map(|k| selection.get(x0 + (k % w) as u32, y0 + (k / w) as u32) > 0)
        .collect();
    let mut fill: Vec<[f32; 3]> = (0..w * h)
        .map(|k| {
            let p = original.pixel(x0 + (k % w) as u32, y0 + (k / w) as u32);
            [p[0] as f32, p[1] as f32, p[2] as f32]
        })
        .collect();
    membrane(w, h, &inside, &mut fill);
    let mut result = original.clone();
    for k in 0..w * h {
        if inside[k] {
            let (x, y) = (x0 + (k % w) as u32, y0 + (k / w) as u32);
            let a = selection.get(x, y) as f32 / 255.0;
            let p = original.pixel(x, y);
            let out =
                [0, 1, 2].map(|c| (p[c] as f32 + (fill[k][c] - p[c] as f32) * a).round() as u8);
            result.set_pixel(x, y, [out[0], out[1], out[2], p[3]]);
        }
    }
    // Place the moved pixels, healed into what's there now
    let result = place_healed(&result, &original, selection, (dx, dy)).unwrap_or(result);
    let Some(image) = doc.layer_mut(id).and_then(|l| l.image_mut()) else {
        return false;
    };
    *image = result;
    doc.mark_dirty();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn membrane_interpolates_the_edge() {
        // A row whose ends are 0 and 100: the inside becomes a ramp
        let w = 5;
        let inside = [false, true, true, true, false];
        let mut v = vec![[0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [100.0; 3]];
        membrane(w, 1, &inside, &mut v);
        assert!((v[2][0] - 50.0).abs() < 1.0, "{v:?}");
    }

    #[test]
    fn healing_takes_texture_and_matches_shading() {
        // A brightness ramp with a dark spot; healing the spot from the
        // ramp beside it leaves no spot and keeps the ramp
        let mut img = TiledImage::new(40, 20);
        for y in 0..20 {
            for x in 0..40 {
                let v = (x * 5) as u8;
                img.set_pixel(x, y, [v, v, v, 255]);
            }
        }
        for y in 8..12 {
            for x in 18..22 {
                img.set_pixel(x, y, [0, 0, 0, 255]);
            }
        }
        let (x0, y0, w, h) = (16u32, 6u32, 8usize, 8usize);
        let inside: Vec<bool> = (0..w * h)
            .map(|k| {
                let (x, y) = (x0 + (k % w) as u32, y0 + (k / w) as u32);
                (17..23).contains(&x) && (7..13).contains(&y)
            })
            .collect();
        let healed = heal_window(&img, &img, (x0, y0, w, h), &inside, (0, 6)).unwrap();
        // The spot's center now follows the ramp (x = 20 → 100)
        let k = (10 - 6) * w + (20 - 16);
        assert!((healed[k][0] - 100.0).abs() < 6.0, "{:?}", healed[k]);
    }

    #[test]
    fn patch_and_content_aware_move() {
        let mut doc = Document::new_with_background("t", 40, 40, Color::WHITE);
        let id = doc.layers[0].id;
        for y in 10..14 {
            for x in 10..14 {
                doc.layer_mut(id)
                    .unwrap()
                    .image_mut()
                    .unwrap()
                    .set_pixel(x, y, [0, 0, 0, 255]);
            }
        }
        // Patch the black square from the white beside it
        let sel = Selection::rect(40, 40, crate::selection::Rect::new(8.0, 8.0, 16.0, 16.0));
        assert!(patch(&mut doc, &sel, (16, 0)));
        let p = doc.layer(id).unwrap().image().unwrap().pixel(12, 12);
        assert!(p[0] > 240, "{p:?}");

        // Move a black square right: the old place fills with white
        for y in 10..14 {
            for x in 10..14 {
                doc.layer_mut(id)
                    .unwrap()
                    .image_mut()
                    .unwrap()
                    .set_pixel(x, y, [0, 0, 0, 255]);
            }
        }
        let sel = Selection::rect(40, 40, crate::selection::Rect::new(10.0, 10.0, 14.0, 14.0));
        assert!(content_aware_move(&mut doc, &sel, (15, 0)));
        let image = doc.layer(id).unwrap().image().unwrap();
        assert!(image.pixel(12, 12)[0] > 240, "{:?}", image.pixel(12, 12));
        assert!(image.pixel(27, 12)[0] < 60, "{:?}", image.pixel(27, 12));
    }

    /// A gray field with a vertical step at the window's left edge (0
    /// left of x 2, 100 right), healed from a flat 50 source.
    fn step_heal(style: HealStyle) -> Vec<[f32; 3]> {
        let (w, h) = (60usize, 9usize);
        let mut img = TiledImage::new(w as u32, h as u32);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let v = if x < 2 { 0 } else { 100 };
                img.set_pixel(x, y, [v, v, v, 255]);
            }
        }
        let mut flat = TiledImage::new(w as u32, h as u32);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                flat.set_pixel(x, y, [50, 50, 50, 255]);
            }
        }
        let inside: Vec<bool> = (0..w * h)
            .map(|k| (k % w) >= 2 && (k % w) < w - 1 && (1..h - 1).contains(&(k / w)))
            .collect();
        heal_window_with(&img, &flat, (0, 0, w, h), &inside, (0, 0), style).unwrap()
    }

    #[test]
    fn diffusion_limits_how_far_the_edge_reaches() {
        // Legacy spreads the edges' shading all the way across; a low
        // Diffusion lets it fade within a few pixels
        let row = 4 * 60;
        let legacy = step_heal(HealStyle::default());
        let short = step_heal(HealStyle {
            mode: HealMode::Normal,
            diffusion: Some(1),
        });
        // Far from the left edge Legacy matches the bright surroundings;
        // a low Diffusion keeps more of the source's own 50 there
        assert!(
            (legacy[row + 40][0] - 100.0).abs() < 3.0,
            "{:?}",
            legacy[row + 40]
        );
        assert!(
            short[row + 40][0] < legacy[row + 40][0] - 10.0,
            "{:?}",
            short[row + 40]
        );
        // Next to the dark edge both still match it
        assert!(short[row + 2][0] < 40.0 && legacy[row + 2][0] < 40.0);
    }

    #[test]
    fn modes_combine_with_the_pixels_under() {
        let under = [200.0, 100.0, 50.0];
        let healed = [100.0, 100.0, 100.0];
        assert_eq!(HealMode::Normal.apply(under, healed), healed);
        assert_eq!(HealMode::Darken.apply(under, healed), [100.0, 100.0, 50.0]);
        assert_eq!(
            HealMode::Lighten.apply(under, healed),
            [200.0, 100.0, 100.0]
        );
        let m = HealMode::Multiply.apply(under, healed);
        assert!((m[0] - 200.0 * 100.0 / 255.0).abs() < 0.5, "{m:?}");
        // Replace dithers: about the coverage's share of pixels take it
        let taken = (0..100)
            .flat_map(|y| (0..100).map(move |x| (x, y)))
            .filter(|&(x, y)| replace_takes(x, y, 0.3))
            .count();
        assert!((2700..3300).contains(&taken), "{taken}");
        assert!((0..50).all(|x| replace_takes(x, 7, 1.0) && !replace_takes(x, 7, 0.0)));
    }

    #[test]
    fn create_texture_mirrors_the_surroundings() {
        // Stripes around a dab: the texture inside is the stripes from
        // just outside, mirrored through the edge
        let mut img = TiledImage::new(30, 30);
        for y in 0..30 {
            for x in 0..30 {
                let v = if x % 2 == 0 { 0 } else { 200 };
                img.set_pixel(x, y, [v, v, v, 255]);
            }
        }
        let t = mirrored_texture(&img, (5, 5, 20, 20), (15.0, 15.0), 4.0);
        // Outside the dab: its own pixels
        assert_eq!(t.pixel(0, 0), img.pixel(5, 5));
        // At the center: a pixel from 2r + 2 = 10 away to the right
        assert_eq!(t.pixel(10, 10), img.pixel(25, 15));
        assert_eq!(t.width(), 20);
    }

    #[test]
    fn patch_transparent_and_pattern() {
        let mut doc = Document::new_with_background("t", 20, 20, Color::WHITE);
        let id = doc.layers[0].id;
        // A red layer patched with a black/white checker pattern: opaque,
        // the checker's texture shows; Transparent, it rides on red
        for y in 0..20 {
            for x in 0..20 {
                doc.layer_mut(id)
                    .unwrap()
                    .image_mut()
                    .unwrap()
                    .set_pixel(x, y, [200, 40, 40, 255]);
            }
        }
        let mut checker = TiledImage::new(2, 2);
        for (x, y) in [(0, 0), (1, 1)] {
            checker.set_pixel(x, y, [0, 0, 0, 255]);
        }
        for (x, y) in [(1, 0), (0, 1)] {
            checker.set_pixel(x, y, [255, 255, 255, 255]);
        }
        let tiled = tiled_pattern(&checker, 20, 20);
        assert_eq!(tiled.pixel(3, 5), [0, 0, 0, 255]);
        assert_eq!(tiled.pixel(4, 5), [255, 255, 255, 255]);
        let sel = Selection::rect(20, 20, crate::selection::Rect::new(4.0, 4.0, 16.0, 16.0));
        let style = HealStyle::default();
        let mut clear = doc;
        assert!(patch_with(&mut clear, &sel, &tiled, (0, 0), style, true));
        let p = clear.layer(id).unwrap().image().unwrap().pixel(10, 10);
        let q = clear.layer(id).unwrap().image().unwrap().pixel(11, 10);
        // Still reddish, with the checker's alternation
        assert!(p[0] > p[1] && q[0] > q[1], "{p:?} {q:?}");
        assert!(p[0].abs_diff(q[0]) > 60, "{p:?} {q:?}");
    }
}
