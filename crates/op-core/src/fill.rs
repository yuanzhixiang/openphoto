//! Edit > Fill, Edit > Clear and the Paint Bucket.

use crate::blend;
use crate::document::Document;
use crate::layer::BlendMode;
use crate::selection::Selection;
use crate::tile::TiledImage;

/// Why a fill can't be done; the messages match Photoshop's alerts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillError {
    NoLayer,
    Locked,
    Hidden,
    /// The active layer is a group, which has no pixels.
    Group,
}

impl FillError {
    pub fn message(self, command: &str) -> String {
        match self {
            Self::NoLayer => {
                format!("Could not complete the {command} command because there is no layer.")
            }
            Self::Locked => {
                format!("Could not complete the {command} command because the layer is locked.")
            }
            Self::Hidden => format!(
                "Could not complete the {command} command because the target layer is hidden."
            ),
            Self::Group => format!(
                "Could not complete the {command} command because the target layer is a group."
            ),
        }
    }
}

/// Fill settings (the Blending section of the Fill dialog).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FillOptions {
    pub mode: BlendMode,
    pub opacity: f32,
    /// Keep each pixel's alpha (only recolor existing pixels).
    pub preserve_transparency: bool,
}

impl Default for FillOptions {
    fn default() -> Self {
        Self {
            mode: BlendMode::Normal,
            opacity: 1.0,
            preserve_transparency: false,
        }
    }
}

fn target(doc: &Document) -> Result<crate::layer::LayerId, FillError> {
    let id = doc.active_layer.ok_or(FillError::NoLayer)?;
    // Quick Mask can be painted whatever the layer's state
    if doc.quick_mask.is_some() {
        return Ok(id);
    }
    let layer = doc.layer(id).ok_or(FillError::NoLayer)?;
    if !layer.visible {
        return Err(FillError::Hidden);
    }
    if doc.pixels_locked(layer.id) {
        return Err(FillError::Locked);
    }
    if layer.is_group() {
        return Err(FillError::Group);
    }
    Ok(id)
}

/// Fills the selection (or the whole layer without one) on the active layer
/// with `color`, using `mask` as an extra coverage limit (the Paint Bucket's
/// region). Returns whether anything changed.
fn fill_masked(
    doc: &mut Document,
    color: [u8; 3],
    options: FillOptions,
    mask: Option<&Selection>,
) -> Result<bool, FillError> {
    target(doc)?;
    let selection = doc.selection().cloned();
    let (w, h) = (doc.width, doc.height);
    let target = doc.edit_target().expect("target layer exists");
    // The background layer and masks are always opaque; locked transparency
    // keeps alpha
    let keep_alpha = target.keep_alpha || options.preserve_transparency;
    let color = if target.mask {
        crate::adjust::mask_gray(color)
    } else {
        color
    };
    let image = target.image;
    let src = color.map(|v| v as f32 / 255.0);
    let mut changed = false;
    for y in 0..h {
        for x in 0..w {
            let mut amount = options.opacity;
            if let Some(s) = &selection {
                amount *= s.get(x, y) as f32 / 255.0;
            }
            if let Some(m) = mask {
                amount *= m.get(x, y) as f32 / 255.0;
            }
            if amount <= 0.0 {
                continue;
            }
            let base = image.pixel(x, y);
            if keep_alpha && base[3] == 0 {
                continue;
            }
            let dst = base.map(|v| v as f32 / 255.0);
            let out = if keep_alpha {
                // Recolor in place: blend as if the pixel were opaque, keep alpha
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
                changed = true;
            }
        }
    }
    doc.mark_dirty();
    Ok(changed)
}

/// Edit > Fill with a solid color.
pub fn fill(doc: &mut Document, color: [u8; 3], options: FillOptions) -> Result<(), FillError> {
    fill_masked(doc, color, options, None).map(|_| ())
}

/// Edit > Clear (Delete): erases the selection on a regular layer. On the
/// background layer (or with transparent pixels locked) Photoshop fills with
/// the background color instead.
pub fn clear(doc: &mut Document, background: [u8; 3]) -> Result<(), FillError> {
    let id = target(doc)?;
    let layer = doc.layer(id).expect("target layer exists");
    // A mask is cleared to the background color's gray
    if layer.is_background
        || doc.transparency_locked(id)
        || doc.editing_mask()
        || doc.quick_mask.is_some()
    {
        return fill(doc, background, FillOptions::default());
    }
    let selection = doc.selection().cloned();
    let (w, h) = (doc.width, doc.height);
    let layer = doc.layer_mut(id).expect("target layer exists");
    let image = layer.image_mut().expect("checked: not a group");
    for y in 0..h {
        for x in 0..w {
            let amount = selection
                .as_ref()
                .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
            if amount <= 0.0 {
                continue;
            }
            let [r, g, b, a] = image.pixel(x, y);
            if a > 0 {
                image.set_pixel(x, y, [r, g, b, (a as f32 * (1.0 - amount)).round() as u8]);
            }
        }
    }
    doc.mark_dirty();
    Ok(())
}

/// Paint Bucket settings (its options bar).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BucketOptions {
    pub fill: FillOptions,
    /// 0–255: how far a channel may differ from the clicked color.
    pub tolerance: u8,
    pub anti_alias: bool,
    pub contiguous: bool,
    /// Sample the merged image instead of the active layer.
    pub all_layers: bool,
}

impl Default for BucketOptions {
    /// Photoshop's defaults: tolerance 32, anti-alias and contiguous on.
    fn default() -> Self {
        Self {
            fill: FillOptions::default(),
            tolerance: 32,
            anti_alias: true,
            contiguous: true,
            all_layers: false,
        }
    }
}

/// The Magic Wand: the region a click at (`x`, `y`) selects, by the same
/// rule as the Paint Bucket (tolerance, contiguous, anti-alias), sampling
/// the active layer or, with `all_layers`, the merged image. `None` outside
/// the canvas or without an active layer.
pub fn magic_wand(doc: &Document, x: u32, y: u32, options: &BucketOptions) -> Option<Selection> {
    if x >= doc.width || y >= doc.height {
        return None;
    }
    let source = if options.all_layers {
        TiledImage::from_rgba8(doc.width, doc.height, &doc.composite_rgba8())
    } else {
        let layer = doc.active_layer.and_then(|id| doc.layer(id))?;
        layer.image()?.clone()
    };
    Some(bucket_region(&source, x, y, options))
}

/// Select > Grow (`contiguous`) and Select > Similar: adds the pixels
/// whose colors lie within the Magic Wand's tolerance of the colors already
/// selected; Grow only those connected to the selection. The result is the
/// union with the current selection; `None` without a selection.
pub fn grow(doc: &Document, options: &BucketOptions, contiguous: bool) -> Option<Selection> {
    let selection = doc.selection()?;
    let (w, h) = (doc.width, doc.height);
    let source = if options.all_layers {
        TiledImage::from_rgba8(w, h, &doc.composite_rgba8())
    } else {
        let layer = doc.active_layer.and_then(|id| doc.layer(id))?;
        layer.image()?.clone()
    };
    // The range of each channel among the (mostly) selected pixels
    let mut lo = [255i32; 4];
    let mut hi = [0i32; 4];
    let mut seeds = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if selection.get(x, y) >= 128 {
                let px = source.pixel(x, y);
                for c in 0..4 {
                    lo[c] = lo[c].min(px[c] as i32);
                    hi[c] = hi[c].max(px[c] as i32);
                }
                seeds.push((x, y));
            }
        }
    }
    if seeds.is_empty() {
        return Some(selection.clone());
    }
    let tol = options.tolerance as i32;
    let similar = |px: [u8; 4]| {
        (0..4).all(|c| (px[c] as i32) >= lo[c] - tol && (px[c] as i32) <= hi[c] + tol)
    };
    let mut mask = vec![0u8; (w * h) as usize];
    if contiguous {
        let mut stack = seeds;
        while let Some((px, py)) = stack.pop() {
            let i = (py * w + px) as usize;
            if mask[i] != 0 || !similar(source.pixel(px, py)) {
                continue;
            }
            mask[i] = 255;
            if px > 0 {
                stack.push((px - 1, py));
            }
            if py > 0 {
                stack.push((px, py - 1));
            }
            if px + 1 < w {
                stack.push((px + 1, py));
            }
            if py + 1 < h {
                stack.push((px, py + 1));
            }
        }
    } else {
        for y in 0..h {
            for x in 0..w {
                if similar(source.pixel(x, y)) {
                    mask[(y * w + x) as usize] = 255;
                }
            }
        }
    }
    let region = Selection::from_mask(w, h, mask, options.anti_alias);
    Some(Selection::combine(
        Some(selection),
        region,
        crate::selection::SelectionOp::Add,
    ))
}

/// The region a Paint Bucket click at (`x`, `y`) fills: pixels whose
/// channels all lie within `tolerance` of the clicked pixel, either
/// connected to it (4-neighborhood) or anywhere.
fn bucket_region(source: &TiledImage, x: u32, y: u32, options: &BucketOptions) -> Selection {
    let (w, h) = (source.width(), source.height());
    let seed = source.pixel(x, y);
    let tol = options.tolerance as i32;
    let similar = |px: [u8; 4]| (0..4).all(|c| (px[c] as i32 - seed[c] as i32).abs() <= tol);
    let mut mask = vec![0u8; (w * h) as usize];
    if options.contiguous {
        let mut stack = vec![(x, y)];
        while let Some((px, py)) = stack.pop() {
            let i = (py * w + px) as usize;
            if mask[i] != 0 || !similar(source.pixel(px, py)) {
                continue;
            }
            mask[i] = 255;
            if px > 0 {
                stack.push((px - 1, py));
            }
            if px + 1 < w {
                stack.push((px + 1, py));
            }
            if py > 0 {
                stack.push((px, py - 1));
            }
            if py + 1 < h {
                stack.push((px, py + 1));
            }
        }
    } else {
        for py in 0..h {
            for px in 0..w {
                if similar(source.pixel(px, py)) {
                    mask[(py * w + px) as usize] = 255;
                }
            }
        }
    }
    Selection::from_mask(w, h, mask, options.anti_alias)
}

/// Paint Bucket click at (`x`, `y`) with `color`. Returns `Ok(false)` when
/// the click is outside the document.
pub fn bucket(
    doc: &mut Document,
    x: u32,
    y: u32,
    color: [u8; 3],
    options: BucketOptions,
) -> Result<bool, FillError> {
    let id = target(doc)?;
    if x >= doc.width || y >= doc.height {
        return Ok(false);
    }
    let source = if options.all_layers {
        TiledImage::from_rgba8(doc.width, doc.height, &doc.composite_rgba8())
    } else {
        let layer = doc.layer(id).expect("target layer exists");
        layer.image().expect("checked: not a group").clone()
    };
    let region = bucket_region(&source, x, y, &options);
    fill_masked(doc, color, options.fill, Some(&region))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::Rect;
    use crate::{Color, Layer};

    fn doc() -> Document {
        Document::new_with_background("t", 10, 10, Color::WHITE)
    }

    fn px(doc: &Document, x: u32, y: u32) -> [u8; 4] {
        let img = doc
            .layer(doc.active_layer.unwrap())
            .unwrap()
            .image()
            .unwrap();
        img.pixel(x, y)
    }

    #[test]
    fn grow_and_similar() {
        // White canvas, black squares at 2..4 and 7..9 on one row
        let mut d = doc();
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        for x in (2..4).chain(7..9) {
            image.set_pixel(x, 5, [0, 0, 0, 255]);
        }
        d.set_selection(Some(Selection::rect(10, 10, Rect::new(2.0, 5.0, 3.0, 6.0))));
        let options = BucketOptions {
            anti_alias: false,
            ..Default::default()
        };
        let g = grow(&d, &options, true).unwrap();
        assert_eq!(g.bounds(), Some((2, 5, 4, 6)));
        let s = grow(&d, &options, false).unwrap();
        assert_eq!(s.bounds(), Some((2, 5, 9, 6)));
        assert_eq!(s.get(5, 5), 0);
        d.set_selection(None);
        assert!(grow(&d, &options, true).is_none());
    }

    #[test]
    fn magic_wand_selects_the_clicked_area() {
        let mut d = doc();
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        for y in 2..5 {
            for x in 2..5 {
                image.set_pixel(x, y, [0, 0, 0, 255]);
            }
        }
        let options = BucketOptions {
            anti_alias: false,
            ..Default::default()
        };
        let s = magic_wand(&d, 3, 3, &options).unwrap();
        assert_eq!(s.bounds(), Some((2, 2, 5, 5)));
        // The white around it, everywhere else
        let s = magic_wand(&d, 0, 0, &options).unwrap();
        assert_eq!(s.get(3, 3), 0);
        assert_eq!(s.get(9, 9), 255);
        assert!(magic_wand(&d, 10, 0, &options).is_none());
    }

    #[test]
    fn fill_respects_selection_opacity_and_mode() {
        let mut d = doc();
        d.set_selection(Some(Selection::rect(
            10,
            10,
            Rect::new(0.0, 0.0, 5.0, 10.0),
        )));
        fill(
            &mut d,
            [0, 0, 0],
            FillOptions {
                opacity: 0.5,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(px(&d, 2, 2), [128, 128, 128, 255]);
        assert_eq!(px(&d, 7, 2), [255, 255, 255, 255]);
        d.set_selection(None);
        fill(
            &mut d,
            [128, 128, 128],
            FillOptions {
                mode: BlendMode::Multiply,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(px(&d, 7, 2), [128, 128, 128, 255]);
    }

    #[test]
    fn clear_erases_or_fills_background() {
        let mut d = doc();
        d.set_selection(Some(Selection::rect(10, 10, Rect::new(0.0, 0.0, 5.0, 5.0))));
        clear(&mut d, [9, 9, 9]).unwrap();
        assert_eq!(px(&d, 1, 1), [9, 9, 9, 255]);
        let id = d.new_layer_id();
        d.layers.push(Layer::raster(
            id,
            "L",
            TiledImage::filled(10, 10, [1, 2, 3, 255]),
        ));
        d.active_layer = Some(id);
        clear(&mut d, [9, 9, 9]).unwrap();
        assert_eq!(px(&d, 1, 1)[3], 0);
        assert_eq!(px(&d, 8, 8), [1, 2, 3, 255]);
    }

    #[test]
    fn preserve_transparency_keeps_alpha() {
        let mut d = doc();
        let id = d.new_layer_id();
        let mut img = TiledImage::new(10, 10);
        img.set_pixel(3, 3, [0, 0, 0, 100]);
        d.layers.push(Layer::raster(id, "L", img));
        d.active_layer = Some(id);
        fill(
            &mut d,
            [255, 0, 0],
            FillOptions {
                preserve_transparency: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(px(&d, 3, 3), [255, 0, 0, 100]);
        assert_eq!(px(&d, 4, 4)[3], 0);
    }

    #[test]
    fn bucket_fills_contiguous_region() {
        let mut d = doc();
        // A black wall at x = 5 splits the white image
        for y in 0..10 {
            set(&mut d, 5, y, [0, 0, 0, 255]);
        }
        let opts = BucketOptions {
            anti_alias: false,
            ..Default::default()
        };
        bucket(&mut d, 1, 1, [255, 0, 0], opts).unwrap();
        assert_eq!(px(&d, 4, 9), [255, 0, 0, 255]);
        assert_eq!(px(&d, 5, 5), [0, 0, 0, 255]);
        assert_eq!(px(&d, 8, 8), [255, 255, 255, 255]);
        // Not contiguous: all similar pixels, on both sides of the wall
        let opts = BucketOptions {
            contiguous: false,
            anti_alias: false,
            ..Default::default()
        };
        bucket(&mut d, 8, 8, [0, 255, 0], opts).unwrap();
        assert_eq!(px(&d, 8, 8), [0, 255, 0, 255]);
        assert_eq!(px(&d, 1, 1), [255, 0, 0, 255]);
    }

    pub(super) fn set(d: &mut Document, x: u32, y: u32, rgba: [u8; 4]) {
        let id = d.active_layer.unwrap();
        let img = d.layer_mut(id).unwrap().image_mut().unwrap();
        img.set_pixel(x, y, rgba);
    }

    #[test]
    fn locked_layer_refuses() {
        let mut d = doc();
        let id = d.active_layer.unwrap();
        d.layer_mut(id).unwrap().lock_pixels = true;
        assert_eq!(
            fill(&mut d, [0; 3], FillOptions::default()),
            Err(FillError::Locked)
        );
        assert_eq!(
            FillError::Locked.message("Fill"),
            "Could not complete the Fill command because the layer is locked."
        );
    }
}

/// The Magic Eraser: erases, on the active layer, the region a Magic Wand
/// click at (`x`, `y`) would select (tolerance, contiguous, anti-alias,
/// all layers), by `opacity`, within the selection if any. On the
/// background it first becomes a regular layer, as in Photoshop (with
/// locked transparency the region is filled with `background` instead).
/// Returns whether anything changed.
pub fn magic_erase(
    doc: &mut Document,
    (x, y): (u32, u32),
    options: &BucketOptions,
    opacity: f32,
    background: [u8; 3],
) -> Result<bool, FillError> {
    target(doc)?;
    let Some(region) = magic_wand(doc, x, y, options) else {
        return Ok(false);
    };
    let id = doc.active_layer.ok_or(FillError::NoLayer)?;
    if doc.layer(id).is_some_and(|l| l.is_background) {
        crate::layer_ops::layer_from_background(doc);
    }
    let keep_alpha = doc.transparency_locked(id);
    let selection = doc.selection().cloned();
    let Some(image) = doc.layer_mut(id).and_then(|l| l.image_mut()) else {
        return Err(FillError::Group);
    };
    let mut changed = false;
    for py in 0..image.height() {
        for px in 0..image.width() {
            let mut a = region.get(px, py) as f32 / 255.0 * opacity;
            if let Some(s) = &selection {
                a *= s.get(px, py) as f32 / 255.0;
            }
            if a <= 0.0 {
                continue;
            }
            let p = image.pixel(px, py);
            let out = if keep_alpha {
                let m = |c: u8, b: u8| (c as f32 + (b as f32 - c as f32) * a).round() as u8;
                [
                    m(p[0], background[0]),
                    m(p[1], background[1]),
                    m(p[2], background[2]),
                    p[3],
                ]
            } else {
                [p[0], p[1], p[2], (p[3] as f32 * (1.0 - a)).round() as u8]
            };
            if out != p {
                image.set_pixel(px, py, out);
                changed = true;
            }
        }
    }
    if changed {
        doc.mark_dirty();
    }
    Ok(changed)
}

/// The Red Eye tool: around a click at (`x`, `y`), the red pixels
/// connected to the reddest one nearby lose their red (it becomes the
/// average of green and blue) and darken by `darken` (0–1, at most 60%).
/// A larger `pupil` (0–1) takes in less red pixels. Returns whether
/// anything changed (nothing when no red is found near the click).
pub fn red_eye(
    doc: &mut Document,
    (x, y): (u32, u32),
    pupil: f32,
    darken: f32,
) -> Result<bool, FillError> {
    let id = target(doc)?;
    let selection = doc.selection().cloned();
    let Some(image) = doc.layer_mut(id).and_then(|l| l.image_mut()) else {
        return Err(FillError::Group);
    };
    let (w, h) = (image.width(), image.height());
    if x >= w || y >= h {
        return Ok(false);
    }
    let redness = |p: [u8; 4]| p[0] as f32 - p[1].max(p[2]) as f32;
    let threshold = 40.0 + (1.0 - pupil.clamp(0.0, 1.0)) * 60.0;
    // The seed: the reddest pixel within a few percent of the image
    let reach = ((w.min(h) as f32 * 0.03).max(8.0)) as i64;
    let mut seed = None;
    let mut best = threshold;
    for sy in (y as i64 - reach).max(0)..=(y as i64 + reach).min(h as i64 - 1) {
        for sx in (x as i64 - reach).max(0)..=(x as i64 + reach).min(w as i64 - 1) {
            let r = redness(image.pixel(sx as u32, sy as u32));
            if r > best {
                best = r;
                seed = Some((sx as u32, sy as u32));
            }
        }
    }
    let Some(seed) = seed else {
        return Ok(false);
    };
    // Flood through red pixels, no farther than four times the reach
    let limit = (reach * 4) as u32;
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![seed];
    seen.insert(seed);
    let mut region = Vec::new();
    while let Some((px, py)) = stack.pop() {
        region.push((px, py));
        for (nx, ny) in [
            (px.wrapping_sub(1), py),
            (px + 1, py),
            (px, py.wrapping_sub(1)),
            (px, py + 1),
        ] {
            if nx >= w || ny >= h || nx.abs_diff(seed.0) > limit || ny.abs_diff(seed.1) > limit {
                continue;
            }
            if !seen.contains(&(nx, ny)) && redness(image.pixel(nx, ny)) > threshold * 0.6 {
                seen.insert((nx, ny));
                stack.push((nx, ny));
            }
        }
    }
    let dark = 1.0 - 0.6 * darken.clamp(0.0, 1.0);
    for (px, py) in region {
        let amount = selection
            .as_ref()
            .map_or(1.0, |s| s.get(px, py) as f32 / 255.0);
        if amount <= 0.0 {
            continue;
        }
        let p = image.pixel(px, py);
        let m = (p[1] as f32 + p[2] as f32) / 2.0;
        let fixed = [m * dark, p[1] as f32 * dark, p[2] as f32 * dark];
        let out =
            [0, 1, 2].map(|c| (p[c] as f32 + (fixed[c] - p[c] as f32) * amount).round() as u8);
        image.set_pixel(px, py, [out[0], out[1], out[2], p[3]]);
    }
    doc.mark_dirty();
    Ok(true)
}

#[cfg(test)]
mod eraser_and_red_eye_tests {
    use super::*;
    use crate::Color;

    #[test]
    fn magic_eraser_and_red_eye() {
        // A white background with a red disc
        let mut doc = Document::new_with_background("t", 60, 60, Color::WHITE);
        let id = doc.layers[0].id;
        {
            let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
            for y in 0..60u32 {
                for x in 0..60u32 {
                    let (dx, dy) = (x as f32 - 30.0, y as f32 - 30.0);
                    if dx * dx + dy * dy < 64.0 {
                        image.set_pixel(x, y, [220, 30, 40, 255]);
                    }
                }
            }
        }
        // Red Eye near (but not on) the disc: its red is gone
        assert!(red_eye(&mut doc, (33, 33), 0.5, 0.5).unwrap());
        let p = doc.layer(id).unwrap().image().unwrap().pixel(30, 30);
        assert!(p[0] < 60 && p[0] <= p[1] + 20, "{p:?}");
        assert_eq!(
            doc.layer(id).unwrap().image().unwrap().pixel(5, 5),
            [255, 255, 255, 255]
        );
        // Nothing red near a corner
        assert!(!red_eye(&mut doc, (2, 2), 0.5, 0.5).unwrap());

        // The Magic Eraser on the white: the background becomes a layer and
        // the white goes, the disc stays
        let options = BucketOptions::default();
        assert!(magic_erase(&mut doc, (2, 2), &options, 1.0, [0, 0, 0]).unwrap());
        let layer = doc.layer(id).unwrap();
        assert!(!layer.is_background);
        assert_eq!(layer.image().unwrap().pixel(2, 2)[3], 0);
        assert_eq!(layer.image().unwrap().pixel(30, 30)[3], 255);
    }
}
