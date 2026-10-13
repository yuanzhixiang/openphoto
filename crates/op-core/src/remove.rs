//! Deterministic offline exemplar filling. This is not Adobe's Remove model.
use crate::{Document, Layer, Selection, TiledImage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Target,
    Dimensions,
    NoContext,
    Limit,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Target => {
                "Remove requires a visible, unlocked pixel layer; mask editing is unsupported."
            }
            Self::Dimensions => "The removal area does not match the document dimensions.",
            Self::NoContext => "There is not enough unmarked image context to fill this area.",
            Self::Limit => "This removal area exceeds the offline processing limit.",
        }
    }
}
const R: i32 = 2;
const MAX_PIXELS: usize = 262_144;
const MAX_WORK: usize = 32_000_000;

/// Fill marked pixels with context-matched unmarked 5x5 source exemplars.
/// All work happens on a clone; errors never return partial output.
pub fn inpaint(source: &TiledImage, area: &Selection) -> Result<TiledImage, Error> {
    let (w, h) = (source.width(), source.height());
    if (area.width(), area.height()) != (w, h) {
        return Err(Error::Dimensions);
    }
    let Some((x0, y0, x1, y1)) = area.bounds() else {
        return Ok(source.clone());
    };
    let (bw, bh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    if bw.saturating_mul(bh) > MAX_PIXELS {
        return Err(Error::Limit);
    }
    let index = |x: u32, y: u32| ((y - y0) as usize) * bw + (x - x0) as usize;
    let mut missing: Vec<bool> = (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| area.get(x, y) > 0))
        .collect();
    let mut count = missing.iter().filter(|&&v| v).count();
    let known = |x: i32, y: i32, missing: &[bool]| {
        x >= 0
            && y >= 0
            && x < w as i32
            && y < h as i32
            && (!(x >= x0 as i32 && y >= y0 as i32 && x < x1 as i32 && y < y1 as i32)
                || !missing[index(x as u32, y as u32)])
    };
    let valid_source = |x: i32, y: i32| {
        (-R..=R).all(|dy| {
            (-R..=R).all(|dx| {
                let (a, b) = (x + dx, y + dy);
                a >= 0
                    && b >= 0
                    && a < w as i32
                    && b < h as i32
                    && area.get(a as u32, b as u32) == 0
                    && source.pixel(a as u32, b as u32)[3] > 0
            })
        })
    };
    // Search a bounded local context grid, including both sides of the hole.
    let (sx0, sy0, sx1, sy1) = (
        x0.saturating_sub(48),
        y0.saturating_sub(48),
        x1.saturating_add(48).min(w),
        y1.saturating_add(48).min(h),
    );
    let context = (sx1 - sx0) as usize * (sy1 - sy0) as usize;
    let stride = ((context as f64 / 512.0).sqrt().ceil() as usize).max(1);
    let mut candidates = Vec::new();
    for y in (sy0..sy1).step_by(stride) {
        for x in (sx0..sx1).step_by(stride) {
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                if stride == 1 && (dx != 0 || dy != 0) {
                    continue;
                }
                let (cx, cy) = (x as i32 + dx, y as i32 + dy);
                if valid_source(cx, cy) {
                    candidates.push((cx, cy));
                }
            }
        }
    }
    if candidates.is_empty() {
        return Err(Error::NoContext);
    }
    let mut out = source.clone();
    let mut work = 0usize;
    while count > 0 {
        let mut boundary = None;
        let mut best_known = 0;
        for y in y0..y1 {
            for x in x0..x1 {
                work += 1;
                if work > MAX_WORK {
                    return Err(Error::Limit);
                }
                if !missing[index(x, y)] {
                    continue;
                }
                let n = (-R..=R)
                    .flat_map(|dy| (-R..=R).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| known(x as i32 + dx, y as i32 + dy, &missing))
                    .count();
                if n > best_known {
                    best_known = n;
                    boundary = Some((x as i32, y as i32));
                }
            }
        }
        let Some((tx, ty)) = boundary else {
            return Err(Error::NoContext);
        };
        let mut best = None;
        for &(sx, sy) in &candidates {
            let mut cost = 0u64;
            let mut n = 0;
            for dy in -R..=R {
                for dx in -R..=R {
                    if !known(tx + dx, ty + dy, &missing) {
                        continue;
                    }
                    work += 1;
                    if work > MAX_WORK {
                        return Err(Error::Limit);
                    }
                    let a = out.pixel((tx + dx) as u32, (ty + dy) as u32);
                    let b = source.pixel((sx + dx) as u32, (sy + dy) as u32);
                    for c in 0..4 {
                        cost += (a[c] as i64 - b[c] as i64).pow(2) as u64;
                    }
                    n += 1;
                }
            }
            let distance = (tx - sx).unsigned_abs() as u64 + (ty - sy).unsigned_abs() as u64;
            let score = (cost / n.max(1), distance);
            if best.is_none_or(|(s, _, _)| score < s) {
                best = Some((score, sx, sy));
            }
        }
        let (_, sx, sy) = best.ok_or(Error::NoContext)?;
        for dy in -R..=R {
            for dx in -R..=R {
                let (x, y) = (tx + dx, ty + dy);
                if x < x0 as i32 || y < y0 as i32 || x >= x1 as i32 || y >= y1 as i32 {
                    continue;
                }
                let i = index(x as u32, y as u32);
                if missing[i] {
                    out.set_pixel(
                        x as u32,
                        y as u32,
                        source.pixel((sx + dx) as u32, (sy + dy) as u32),
                    );
                    missing[i] = false;
                    count -= 1;
                }
            }
        }
    }
    Ok(out)
}

/// Apply a removal atomically, respecting selection and destination permissions.
/// New-layer/sample-all output contains only repaired pixels.
pub fn apply(
    doc: &mut Document,
    area: &Selection,
    all_layers: bool,
    new_layer: bool,
) -> Result<bool, Error> {
    let id = doc.active_layer.ok_or(Error::Target)?;
    let layer = doc.layer(id).ok_or(Error::Target)?;
    if !layer.visible
        || doc.pixels_locked(id)
        || layer.image().is_none()
        || doc.editing_mask()
        || doc.quick_mask.is_some()
    {
        return Err(Error::Target);
    }
    if (area.width(), area.height()) != (doc.width, doc.height) {
        return Err(Error::Dimensions);
    }
    let mut area = area.clone();
    if let Some(selection) = doc.selection() {
        area = Selection::combine(Some(selection), area, crate::SelectionOp::Intersect);
    }
    if area.is_empty() {
        return Ok(false);
    }
    let source = doc
        .sample_source(if all_layers {
            crate::SampleScope::All
        } else {
            crate::SampleScope::Current
        })
        .ok_or(Error::Target)?;
    let repaired = inpaint(&source, &area)?;
    let target = layer.image().ok_or(Error::Target)?;
    let mut result = if new_layer {
        TiledImage::new(doc.width, doc.height)
    } else {
        target.clone()
    };
    let mut changed = false;
    let (x0, y0, x1, y1) = area.bounds().ok_or(Error::NoContext)?;
    for y in y0..y1 {
        for x in x0..x1 {
            let a = area.get(x, y) as f32 / 255.0;
            if a == 0.0 {
                continue;
            }
            let before = source.pixel(x, y);
            let p = repaired.pixel(x, y);
            if p == before {
                continue;
            }
            let out = if new_layer || all_layers {
                [p[0], p[1], p[2], (p[3] as f32 * a).round() as u8]
            } else {
                let b = target.pixel(x, y);
                [0, 1, 2, 3].map(|c| (b[c] as f32 + (p[c] as f32 - b[c] as f32) * a).round() as u8)
            };
            if result.pixel(x, y) != out {
                result.set_pixel(x, y, out);
                changed = true;
            }
        }
    }
    if changed {
        if new_layer {
            doc.insert_above_active(Layer::raster(doc.new_layer_id(), "Remove", result));
        } else {
            *doc.layer_mut(id)
                .and_then(|l| l.image_mut())
                .ok_or(Error::Target)? = result;
            doc.mark_dirty();
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_known_texture_and_fills_a_spot_deterministically() {
        let mut image = TiledImage::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                let v = if (x + y) % 2 == 0 { 40 } else { 200 };
                image.set_pixel(x, y, [v, v, v, 255]);
            }
        }
        let clean = image.clone();
        let area = Selection::rect(40, 40, crate::selection::Rect::new(16., 16., 21., 21.));
        for y in 16..21 {
            for x in 16..21 {
                image.set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        let out = inpaint(&image, &area).unwrap();
        assert_eq!(out.to_rgba8(), clean.to_rgba8());
        assert_eq!(inpaint(&image, &area).unwrap().to_rgba8(), out.to_rgba8());
        assert_eq!(image.pixel(18, 18), [255, 0, 0, 255]);
    }
    #[test]
    fn rejects_missing_context_and_does_not_mutate_locked_targets() {
        let mut doc = Document::new_with_background("remove", 32, 32, crate::Color::WHITE);
        let before = doc.composite_rgba8();
        let revision = doc.revision();
        assert_eq!(
            apply(&mut doc, &Selection::all(32, 32), false, false),
            Err(Error::NoContext)
        );
        doc.layers[0].lock_pixels = true;
        assert_eq!(
            apply(&mut doc, &Selection::all(32, 32), false, true),
            Err(Error::Target)
        );
        assert_eq!(doc.composite_rgba8(), before);
        assert_eq!(doc.revision(), revision);
    }
}
