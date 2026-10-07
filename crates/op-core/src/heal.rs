//! Healing: Photoshop's healing tools keep the texture of a source and take
//! the shading of the place they repair. The difference between target and
//! source along the repaired area's edge is spread smoothly inside it (a
//! membrane: Laplace's equation with those edge values), and the result
//! is the source plus that membrane, so the repair matches its surroundings
//! at the edge without a seam.

use crate::document::Document;
use crate::selection::Selection;
use crate::tile::TiledImage;

/// Solves Laplace's equation on a `w` × `h` window: values of the pixels
/// outside `inside` stay as in `values`; those inside become the smooth
/// interpolation of their neighbors (successive over-relaxation until
/// the largest change is below a hundredth of a level).
pub fn membrane(w: usize, h: usize, inside: &[bool], values: &mut [[f32; 3]]) {
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
                let target = acc[c] / k;
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
    (x0, y0, w, h): (u32, u32, usize, usize),
    inside: &[bool],
    (dx, dy): (i64, i64),
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
    membrane(w, h, inside, &mut corr);
    Some(
        (0..w * h)
            .map(|k| {
                let (x, y) = (x0 + (k % w) as u32, y0 + (k / w) as u32);
                if inside[k] {
                    [0, 1, 2].map(|c| (sources[k][c] + corr[k][c]).clamp(0.0, 255.0))
                } else {
                    rgb(image.pixel(x, y))
                }
            })
            .collect(),
    )
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
pub fn patch(doc: &mut Document, selection: &Selection, (dx, dy): (i64, i64)) -> bool {
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
    // The source is where the selection went: target pixel (x, y) takes
    // the texture at (x + dx, y + dy)
    let source = image.clone();
    let Some(healed) = heal_window(image, &source, (x0, y0, w, h), &inside, (-dx, -dy)) else {
        return false;
    };
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
}
