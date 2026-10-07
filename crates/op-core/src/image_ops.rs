//! Image menu operations that reshape the canvas: Image Rotation, the
//! canvas flips, Crop and Trim.

use crate::document::{Document, Guide};

/// Image > Image Rotation (fixed angles and flips).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Rotate180,
    Rotate90Clockwise,
    Rotate90CounterClockwise,
    FlipHorizontal,
    FlipVertical,
}

impl Orientation {
    /// The history name Photoshop uses.
    pub fn history_name(self) -> &'static str {
        match self {
            Self::Rotate180 | Self::Rotate90Clockwise | Self::Rotate90CounterClockwise => {
                "Rotate Canvas"
            }
            Self::FlipHorizontal => "Flip Canvas Horizontal",
            Self::FlipVertical => "Flip Canvas Vertical",
        }
    }
}

/// Rotates or flips the whole document: every layer and the selection.
pub fn reorient(doc: &mut Document, orientation: Orientation) {
    let (w, h) = (doc.width, doc.height);
    let (nw, nh) = match orientation {
        Orientation::Rotate90Clockwise | Orientation::Rotate90CounterClockwise => (h, w),
        _ => (w, h),
    };
    // Source pixel of each destination pixel
    let source = move |x: u32, y: u32| match orientation {
        Orientation::Rotate180 => (w - 1 - x, h - 1 - y),
        Orientation::Rotate90Clockwise => (y, h - 1 - x),
        Orientation::Rotate90CounterClockwise => (w - 1 - y, x),
        Orientation::FlipHorizontal => (w - 1 - x, y),
        Orientation::FlipVertical => (x, h - 1 - y),
    };
    // Where each source pixel lands, for pixels outside the canvas, which
    // turn with it (as in Photoshop)
    let (sw, sh) = (w as i64, h as i64);
    let dest = move |x: i64, y: i64| match orientation {
        Orientation::Rotate180 => (sw - 1 - x, sh - 1 - y),
        Orientation::Rotate90Clockwise => (sh - 1 - y, x),
        Orientation::Rotate90CounterClockwise => (y, sw - 1 - x),
        Orientation::FlipHorizontal => (sw - 1 - x, y),
        Orientation::FlipVertical => (x, sh - 1 - y),
    };
    doc.transform_canvas(
        nw,
        nh,
        |image| {
            if !image.has_pixels_outside() {
                return image.remapped(nw, nh, source);
            }
            let mut out = crate::tile::TiledImage::new(nw, nh);
            if let Some((x0, y0, x1, y1)) = image.content_bounds() {
                for y in y0..y1 {
                    for x in x0..x1 {
                        let px = image.pixel_at(x, y);
                        if px[3] > 0 {
                            let (dx, dy) = dest(x, y);
                            out.set_pixel_at(dx, dy, px);
                        }
                    }
                }
            }
            out
        },
        |selection| selection.remapped(nw, nh, source),
    );
    let (w, h) = (w as f32, h as f32);
    doc.map_guides(|g| {
        let p = g.position;
        match (orientation, g.vertical) {
            (Orientation::Rotate180, v) => Guide {
                vertical: v,
                position: if v { w - p } else { h - p },
            },
            // Clockwise: a vertical guide at x becomes horizontal at x;
            // a horizontal guide at y becomes vertical at h - y
            (Orientation::Rotate90Clockwise, true) => Guide {
                vertical: false,
                position: p,
            },
            (Orientation::Rotate90Clockwise, false) => Guide {
                vertical: true,
                position: h - p,
            },
            (Orientation::Rotate90CounterClockwise, true) => Guide {
                vertical: false,
                position: w - p,
            },
            (Orientation::Rotate90CounterClockwise, false) => Guide {
                vertical: true,
                position: p,
            },
            (Orientation::FlipHorizontal, true) => Guide {
                vertical: true,
                position: w - p,
            },
            (Orientation::FlipVertical, false) => Guide {
                vertical: false,
                position: h - p,
            },
            _ => g,
        }
    });
}

/// Image > Image Rotation > Arbitrary... ("Rotate Canvas"): turns the
/// whole document by `degrees` (clockwise when positive) about its center.
/// The canvas grows to the turned image's bounding box, rounded up
/// (200 × 100 by 30° gives 224 × 187, as in Photoshop 2026); the
/// background's new corners take `background`, other layers stay
/// transparent there. Pixels are resampled bilinearly; layers keep pixels
/// that turn past the canvas. Returns false for a multiple of 360°.
pub fn rotate_arbitrary(doc: &mut Document, degrees: f32, background: crate::Color) -> bool {
    let turns = degrees / 360.0;
    if (turns - turns.round()).abs() < 1e-6 {
        return false;
    }
    let (w, h) = (doc.width as f64, doc.height as f64);
    let t = (degrees as f64).to_radians();
    let (c, s) = (t.cos(), t.sin());
    let nw = (w * c.abs() + h * s.abs() - 1e-6).ceil().max(1.0);
    let nh = (w * s.abs() + h * c.abs() - 1e-6).ceil().max(1.0);
    // New canvas point -> old canvas point (pixel corners)
    let back = move |x: f64, y: f64| {
        let (dx, dy) = (x - nw / 2.0, y - nh / 2.0);
        (dx * c + dy * s + w / 2.0, -dx * s + dy * c + h / 2.0)
    };
    // Old -> new, for where a layer's pixels land
    let forth = move |x: f64, y: f64| {
        let (dx, dy) = (x - w / 2.0, y - h / 2.0);
        (dx * c - dy * s + nw / 2.0, dx * s + dy * c + nh / 2.0)
    };
    let (cw, ch) = (nw as u32, nh as u32);
    let fill = background.to_rgba8();
    let turn = |image: &crate::tile::TiledImage, outside: [u8; 4], keep_outside: bool| {
        let (sx0, sy0, sx1, sy1) = image.content_bounds().map_or((0, 0, 1, 1), |b| {
            (b.0.min(0), b.1.min(0), b.2.max(w as i64), b.3.max(h as i64))
        });
        let (sw, sh) = ((sx1 - sx0) as usize, (sy1 - sy0) as usize);
        let raw = image.region_rgba8(sx0, sy0, sw as u32, sh as u32);
        // Premultiplied, so edges blend without dark fringes
        let src: Vec<[f32; 4]> = raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&[r, g, b, a]| {
                let k = a as f32 / 255.0;
                [r as f32 * k, g as f32 * k, b as f32 * k, a as f32]
            })
            .collect();
        // Where the content lands, with the new canvas
        let (mut dx0, mut dy0, mut dx1, mut dy1) = (0i64, 0i64, cw as i64, ch as i64);
        if keep_outside {
            for (x, y) in [(sx0, sy0), (sx1, sy0), (sx1, sy1), (sx0, sy1)] {
                let (fx, fy) = forth(x as f64, y as f64);
                dx0 = dx0.min(fx.floor() as i64);
                dy0 = dy0.min(fy.floor() as i64);
                dx1 = dx1.max(fx.ceil() as i64);
                dy1 = dy1.max(fy.ceil() as i64);
            }
        }
        let (dw, dh) = ((dx1 - dx0) as usize, (dy1 - dy0) as usize);
        let at = |x: i64, y: i64| -> [f32; 4] {
            if x < 0 || y < 0 || x >= sw as i64 || y >= sh as i64 {
                [0.0; 4]
            } else {
                src[y as usize * sw + x as usize]
            }
        };
        let mut out = vec![0u8; dw * dh * 4];
        for y in 0..dh {
            for x in 0..dw {
                let (ox, oy) = back((dx0 + x as i64) as f64 + 0.5, (dy0 + y as i64) as f64 + 0.5);
                let (fx, fy) = (
                    (ox - sx0 as f64 - 0.5) as f32,
                    (oy - sy0 as f64 - 0.5) as f32,
                );
                let (ix, iy) = (fx.floor(), fy.floor());
                let (tx, ty) = (fx - ix, fy - iy);
                let (ix, iy) = (ix as i64, iy as i64);
                let mut p = [0f32; 4];
                for (dxi, dyi, wgt) in [
                    (0, 0, (1.0 - tx) * (1.0 - ty)),
                    (1, 0, tx * (1.0 - ty)),
                    (0, 1, (1.0 - tx) * ty),
                    (1, 1, tx * ty),
                ] {
                    let q = at(ix + dxi, iy + dyi);
                    for k in 0..4 {
                        p[k] += q[k] * wgt;
                    }
                }
                // What isn't covered shows `outside` (the background color)
                let a = p[3] / 255.0;
                let oa = outside[3] as f32 / 255.0;
                let ra = a + oa * (1.0 - a);
                let o = (y * dw + x) * 4;
                if ra > 0.0 {
                    for k in 0..3 {
                        let v = (p[k] + outside[k] as f32 * oa * (1.0 - a)) / ra;
                        out[o + k] = v.round().clamp(0.0, 255.0) as u8;
                    }
                    out[o + 3] = (ra * 255.0).round() as u8;
                }
            }
        }
        crate::tile::TiledImage::from_region(cw, ch, dx0, dy0, dw as u32, dh as u32, &out)
    };
    for layer in &mut doc.layers {
        let is_background = layer.is_background;
        if let Some(image) = layer.image_mut() {
            *image = if is_background {
                turn(image, fill, false).clipped()
            } else {
                turn(image, [0; 4], true)
            };
        }
        // New corners of a mask reveal, as Canvas Size does
        if let Some(mask) = &mut layer.mask {
            mask.image = turn(&mask.image, [255; 4], false).clipped();
        }
    }
    // The selection turns with the image
    for sel in [doc.selection().cloned()].into_iter().flatten() {
        let mut mask = vec![0u8; (cw * ch) as usize];
        for y in 0..ch {
            for x in 0..cw {
                let (ox, oy) = back(x as f64 + 0.5, y as f64 + 0.5);
                if ox >= 0.0 && oy >= 0.0 && ox < w && oy < h {
                    mask[(y * cw + x) as usize] = sel.get(ox as u32, oy as u32);
                }
            }
        }
        doc.set_selection(Some(crate::selection::Selection::from_mask(
            cw, ch, mask, false,
        )));
    }
    doc.width = cw;
    doc.height = ch;
    doc.mark_dirty();
    true
}

/// Image > Reveal All: grows the canvas so every layer's pixels, including
/// the ones outside it, are on it. The background layer is extended with
/// `background`. Returns false (and changes nothing) when nothing lies
/// outside the canvas.
pub fn reveal_all(doc: &mut Document, background: crate::Color) -> bool {
    let (x0, y0, x1, y1) = doc.content_bounds();
    if (x0, y0, x1, y1) == (0, 0, doc.width as i64, doc.height as i64) {
        return false;
    }
    doc.place_canvas((x1 - x0) as u32, (y1 - y0) as u32, -x0, -y0, background);
    true
}

/// Cuts the canvas down to x0..x1 × y0..y1 (exclusive ends, inside the
/// canvas). The selection keeps its place on the image. Pixels outside the
/// new canvas are deleted (Photoshop's Delete Cropped Pixels).
pub fn crop(doc: &mut Document, x0: u32, y0: u32, x1: u32, y1: u32) {
    assert!(x0 < x1 && y0 < y1 && x1 <= doc.width && y1 <= doc.height);
    let (w, h) = (x1 - x0, y1 - y0);
    let (dx, dy) = (-(x0 as i64), -(y0 as i64));
    doc.transform_canvas(
        w,
        h,
        |image| image.with_canvas(w, h, dx, dy, [0; 4]).clipped(),
        |selection| selection.with_canvas(w, h, dx, dy),
    );
    doc.map_guides(|g| Guide {
        position: g.position - if g.vertical { x0 } else { y0 } as f32,
        ..g
    });
}

/// The Crop tool's crop to x0..x1 × y0..y1 (exclusive ends), which may
/// reach past the canvas: the canvas grows there, as in Photoshop, the
/// background with `background` (Fill: Background) and other layers
/// transparent. With `delete_cropped` (Delete Cropped Pixels) the pixels
/// outside the new canvas are deleted; without it they stay on the
/// layers, the background becoming a regular "Layer 0" so it can keep
/// them. Returns false for an empty box.
pub fn crop_extended(
    doc: &mut Document,
    (x0, y0, x1, y1): (i64, i64, i64, i64),
    delete_cropped: bool,
    background: crate::Color,
) -> bool {
    if x1 <= x0 || y1 <= y0 {
        return false;
    }
    let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
    let keeps_background_pixels =
        !delete_cropped && (x0 > 0 || y0 > 0 || x1 < doc.width as i64 || y1 < doc.height as i64);
    if keeps_background_pixels && let Some(bg) = doc.layers.iter_mut().find(|l| l.is_background) {
        bg.is_background = false;
        bg.name = "Layer 0".into();
        bg.set_locks(crate::Locks::default());
    }
    doc.place_canvas(w, h, -x0, -y0, background);
    if delete_cropped {
        for layer in &mut doc.layers {
            if let Some(image) = layer.image_mut() {
                *image = image.clipped();
            }
        }
    }
    true
}

/// Image > Crop: crops to the selection's bounding box. Returns false
/// without a selection.
pub fn crop_to_selection(doc: &mut Document) -> bool {
    let Some((x0, y0, x1, y1)) = doc.selection().and_then(|s| s.bounds()) else {
        return false;
    };
    crop(doc, x0, y0, x1, y1);
    true
}

/// Image > Image Size's resampling methods, in Photoshop 2026's menu order
/// (with Alt+1 ... Alt+8 as their shortcuts in the dialog).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Resample {
    /// Photoshop's default: Bicubic Sharper when reducing, Bicubic Smoother
    /// when enlarging.
    #[default]
    Automatic,
    PreserveDetails,
    PreserveDetails2,
    BicubicSmoother,
    BicubicSharper,
    Bicubic,
    NearestNeighbor,
    Bilinear,
}

impl Resample {
    pub const ALL: [Self; 8] = [
        Self::Automatic,
        Self::PreserveDetails,
        Self::PreserveDetails2,
        Self::BicubicSmoother,
        Self::BicubicSharper,
        Self::Bicubic,
        Self::NearestNeighbor,
        Self::Bilinear,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Automatic => "Automatic",
            Self::PreserveDetails => "Preserve Details (enlargement)",
            Self::PreserveDetails2 => "Preserve Details 2.0",
            Self::BicubicSmoother => "Bicubic Smoother (enlargement)",
            Self::BicubicSharper => "Bicubic Sharper (reduction)",
            Self::Bicubic => "Bicubic (smooth gradients)",
            Self::NearestNeighbor => "Nearest Neighbor (hard edges)",
            Self::Bilinear => "Bilinear",
        }
    }

    /// Whether the menu has a separator after this item (Photoshop 2026).
    pub fn separator_after(self) -> bool {
        matches!(
            self,
            Self::Automatic | Self::BicubicSmoother | Self::BicubicSharper
        )
    }

    /// The method actually used: Automatic depends on the direction.
    fn resolve(self, enlarging: bool) -> Self {
        match self {
            Self::Automatic if enlarging => Self::BicubicSmoother,
            Self::Automatic => Self::BicubicSharper,
            m => m,
        }
    }

    /// Filter weight at distance `x` (in source pixels).
    fn weight(self, x: f32) -> f32 {
        let x = x.abs();
        // Keys' cubic convolution with parameter `a`
        let keys = |a: f32| {
            if x < 1.0 {
                (a + 2.0) * x * x * x - (a + 3.0) * x * x + 1.0
            } else if x < 2.0 {
                a * x * x * x - 5.0 * a * x * x + 8.0 * a * x - 4.0 * a
            } else {
                0.0
            }
        };
        match self {
            Self::Bicubic => keys(-0.5),
            // Stronger negative lobes: crisper reductions
            Self::BicubicSharper => keys(-0.75),
            // Mitchell–Netravali (B = C = 1/3): softer enlargements
            Self::BicubicSmoother => {
                let (b, c) = (1.0 / 3.0, 1.0 / 3.0);
                let v = if x < 1.0 {
                    (12.0 - 9.0 * b - 6.0 * c) * x * x * x
                        + (-18.0 + 12.0 * b + 6.0 * c) * x * x
                        + (6.0 - 2.0 * b)
                } else if x < 2.0 {
                    (-b - 6.0 * c) * x * x * x
                        + (6.0 * b + 30.0 * c) * x * x
                        + (-12.0 * b - 48.0 * c) * x
                        + (8.0 * b + 24.0 * c)
                } else {
                    0.0
                };
                v / 6.0
            }
            // Lanczos 3: keeps edges and texture when enlarging
            Self::PreserveDetails | Self::PreserveDetails2 => {
                if x < 1e-6 {
                    1.0
                } else if x < 3.0 {
                    let px = std::f32::consts::PI * x;
                    3.0 * px.sin() * (px / 3.0).sin() / (px * px)
                } else {
                    0.0
                }
            }
            Self::Bilinear => (1.0 - x).max(0.0),
            Self::NearestNeighbor | Self::Automatic => unreachable!("resolved or sampled directly"),
        }
    }

    fn radius(self) -> f32 {
        match self {
            Self::PreserveDetails | Self::PreserveDetails2 => 3.0,
            Self::Bilinear => 1.0,
            Self::NearestNeighbor => 0.5,
            _ => 2.0,
        }
    }
}

/// Resamples `channels`-wide rows of `src` (sw × sh) to dw × dh, one axis
/// at a time. When shrinking, the filter widens to cover every source
/// pixel (area-correct downsampling).
fn resample_buffer(
    src: &[f32],
    sw: usize,
    sh: usize,
    dw: usize,
    dh: usize,
    ch: usize,
    method: Resample,
) -> Vec<f32> {
    let method = method.resolve(dw * dh > sw * sh);
    let pass = |src: &[f32], sw: usize, sh: usize, dw: usize, horizontal: bool| -> Vec<f32> {
        // `n` = length along the resampled axis, `m` = the other axis
        let (n_src, n_dst, m) = if horizontal {
            (sw, dw, sh)
        } else {
            (sh, dw, sw)
        };
        let scale = n_dst as f32 / n_src as f32;
        let support = if scale < 1.0 { 1.0 / scale } else { 1.0 };
        let mut out = vec![0f32; ch * if horizontal { dw * sh } else { sw * dw }];
        for i in 0..n_dst {
            let center = (i as f32 + 0.5) / scale - 0.5;
            let taps: Vec<(usize, f32)> = if method == Resample::NearestNeighbor {
                vec![((center.round().max(0.0) as usize).min(n_src - 1), 1.0)]
            } else {
                let r = method.radius() * support;
                let lo = (center - r).floor() as isize;
                let hi = (center + r).ceil() as isize;
                let mut taps: Vec<(usize, f32)> = (lo..=hi)
                    .map(|j| {
                        let w = method.weight((j as f32 - center) / support);
                        (j.clamp(0, n_src as isize - 1) as usize, w)
                    })
                    .filter(|(_, w)| *w != 0.0)
                    .collect();
                let sum: f32 = taps.iter().map(|(_, w)| w).sum();
                taps.iter_mut().for_each(|(_, w)| *w /= sum);
                taps
            };
            for k in 0..m {
                let mut acc = vec![0f32; ch];
                for &(j, w) in &taps {
                    let idx = if horizontal { k * sw + j } else { j * sw + k };
                    for c in 0..ch {
                        acc[c] += src[idx * ch + c] * w;
                    }
                }
                let o = if horizontal { k * dw + i } else { i * sw + k };
                out[o * ch..o * ch + ch].copy_from_slice(&acc);
            }
        }
        out
    };
    let horizontal = pass(src, sw, sh, dw, true);
    pass(&horizontal, dw, sh, dh, false)
}

/// Image > Image Size with resampling: every layer and the selection are
/// scaled to `width` × `height`. Colors are resampled premultiplied by
/// alpha; results are clamped (bicubic can overshoot).
pub fn resize(doc: &mut Document, width: u32, height: u32, method: Resample) {
    resize_reducing_noise(doc, width, height, method, 0.0);
}

/// [`resize`] with Preserve Details' Reduce Noise (`amount` 0–1): after
/// resampling, each color is pulled toward the mean of its 3 × 3
/// neighbours that are close to it (edges are kept), by `amount`. Other
/// methods ignore it.
pub fn resize_reducing_noise(
    doc: &mut Document,
    width: u32,
    height: u32,
    method: Resample,
    amount: f32,
) {
    assert!(width > 0 && height > 0);
    let amount = match method {
        Resample::PreserveDetails | Resample::PreserveDetails2 => amount.clamp(0.0, 1.0),
        _ => 0.0,
    };
    let (sw, sh) = (doc.width as usize, doc.height as usize);
    let (kx, ky) = (width as f32 / sw as f32, height as f32 / sh as f32);
    doc.map_guides(|g| Guide {
        position: g.position * if g.vertical { kx } else { ky },
        ..g
    });
    let (dw, dh) = (width as usize, height as usize);
    doc.transform_canvas(
        width,
        height,
        |image| {
            if image.has_pixels_outside() {
                return resize_with_outside(image, (kx, ky), width, height, method, amount);
            }
            let raw = image.to_rgba8();
            let src: Vec<f32> = raw
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|&[r, g, b, a]| {
                    let k = a as f32 / 255.0;
                    [r as f32 * k, g as f32 * k, b as f32 * k, a as f32]
                })
                .collect();
            let mut out = resample_buffer(&src, sw, sh, dw, dh, 4, method);
            reduce_noise(&mut out, dw, dh, amount);
            let pixels: Vec<u8> = out
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|&[r, g, b, a]| {
                    let a = a.clamp(0.0, 255.0);
                    if a < 0.5 {
                        return [0; 4];
                    }
                    let k = 255.0 / a;
                    [
                        (r * k).round().clamp(0.0, 255.0) as u8,
                        (g * k).round().clamp(0.0, 255.0) as u8,
                        (b * k).round().clamp(0.0, 255.0) as u8,
                        a.round() as u8,
                    ]
                })
                .collect();
            crate::tile::TiledImage::from_rgba8(width, height, &pixels)
        },
        |selection| {
            let src: Vec<f32> = (0..sw * sh)
                .map(|i| selection.get((i % sw) as u32, (i / sw) as u32) as f32)
                .collect();
            let out = resample_buffer(&src, sw, sh, dw, dh, 1, method);
            let mask = out
                .iter()
                .map(|v| v.round().clamp(0.0, 255.0) as u8)
                .collect();
            crate::selection::Selection::from_mask(width, height, mask, false)
        },
    );
}

/// Image Size for a layer with pixels outside the canvas: the canvas and
/// everything around it are scaled together, so those pixels stay in
/// place relative to the image (as in Photoshop).
fn resize_with_outside(
    image: &crate::tile::TiledImage,
    (kx, ky): (f32, f32),
    width: u32,
    height: u32,
    method: Resample,
    amount: f32,
) -> crate::tile::TiledImage {
    let (cw, ch) = (image.width() as i64, image.height() as i64);
    let (x0, y0, x1, y1) = image.content_bounds().map_or((0, 0, cw, ch), |b| {
        (b.0.min(0), b.1.min(0), b.2.max(cw), b.3.max(ch))
    });
    let (rw, rh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let raw = image.region_rgba8(x0, y0, rw as u32, rh as u32);
    let src: Vec<f32> = raw
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| {
            let k = a as f32 / 255.0;
            [r as f32 * k, g as f32 * k, b as f32 * k, a as f32]
        })
        .collect();
    let dw = ((rw as f32 * kx).round() as usize).max(1);
    let dh = ((rh as f32 * ky).round() as usize).max(1);
    let mut out = resample_buffer(&src, rw, rh, dw, dh, 4, method);
    reduce_noise(&mut out, dw, dh, amount);
    let pixels: Vec<u8> = out
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| {
            let a = a.clamp(0.0, 255.0);
            if a < 0.5 {
                return [0; 4];
            }
            let k = 255.0 / a;
            [
                (r * k).round().clamp(0.0, 255.0) as u8,
                (g * k).round().clamp(0.0, 255.0) as u8,
                (b * k).round().clamp(0.0, 255.0) as u8,
                a.round() as u8,
            ]
        })
        .collect();
    let (dx0, dy0) = (
        (x0 as f32 * kx).round() as i64,
        (y0 as f32 * ky).round() as i64,
    );
    crate::tile::TiledImage::from_region(width, height, dx0, dy0, dw as u32, dh as u32, &pixels)
}

/// Pulls each premultiplied RGBA pixel of `buf` (w × h) toward the mean of
/// the 3 × 3 neighbours whose color is within 24 levels of it, by `amount`
/// (Reduce Noise: smooths speckle, keeps edges).
fn reduce_noise(buf: &mut [f32], w: usize, h: usize, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    const CLOSE: f32 = 24.0;
    let src = buf.to_vec();
    let at = |x: usize, y: usize| &src[(y * w + x) * 4..(y * w + x) * 4 + 4];
    for y in 0..h {
        for x in 0..w {
            let p = at(x, y);
            let mut sum = [0.0f32; 4];
            let mut n = 0.0;
            for ny in y.saturating_sub(1)..(y + 2).min(h) {
                for nx in x.saturating_sub(1)..(x + 2).min(w) {
                    let q = at(nx, ny);
                    if (0..4).all(|c| (q[c] - p[c]).abs() <= CLOSE) {
                        for c in 0..4 {
                            sum[c] += q[c];
                        }
                        n += 1.0;
                    }
                }
            }
            let out = &mut buf[(y * w + x) * 4..(y * w + x) * 4 + 4];
            for c in 0..4 {
                out[c] = p[c] + (sum[c] / n - p[c]) * amount;
            }
        }
    }
}

/// The Perspective Crop tool's output size for `quad` (top-left, top-right,
/// bottom-right, bottom-left) when none is typed: the mean lengths of its
/// opposite sides.
pub fn perspective_size(quad: [(f32, f32); 4]) -> (u32, u32) {
    let len = |a: (f32, f32), b: (f32, f32)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    let [tl, tr, br, bl] = quad;
    let w = (len(tl, tr) + len(bl, br)) / 2.0;
    let h = (len(tl, bl) + len(tr, br)) / 2.0;
    (w.round().max(1.0) as u32, h.round().max(1.0) as u32)
}

/// The Perspective Crop tool: the area inside `quad` (top-left, top-right,
/// bottom-right, bottom-left, document pixels) becomes a `width` × `height`
/// canvas, every layer, mask and the selection mapped through the
/// projective map that takes the canvas's corners to the quad's (bilinear
/// samples, premultiplied). Pixels around the image read as transparent.
pub fn perspective_crop(doc: &mut Document, quad: [(f32, f32); 4], width: u32, height: u32) {
    use crate::transform::Projective;
    assert!(width > 0 && height > 0);
    let m = Projective::rect_to_quad((0.0, 0.0, width as f32, height as f32), quad);
    let x0 = quad.iter().map(|p| p.0).fold(f32::MAX, f32::min).floor() as i64 - 1;
    let y0 = quad.iter().map(|p| p.1).fold(f32::MAX, f32::min).floor() as i64 - 1;
    let x1 = quad.iter().map(|p| p.0).fold(f32::MIN, f32::max).ceil() as i64 + 1;
    let y1 = quad.iter().map(|p| p.1).fold(f32::MIN, f32::max).ceil() as i64 + 1;
    let (rw, rh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let (dw, dh) = (width as usize, height as usize);
    // Where each output pixel's center comes from, relative to the region
    let sources: Vec<(f32, f32)> = (0..dw * dh)
        .map(|i| {
            let (x, y) = ((i % dw) as f32 + 0.5, (i / dw) as f32 + 0.5);
            let (sx, sy) = m.apply((x, y));
            (sx - 0.5 - x0 as f32, sy - 0.5 - y0 as f32)
        })
        .collect();
    let bilinear = |src: &[f32], ch: usize, (sx, sy): (f32, f32), out: &mut [f32]| {
        let (fx, fy) = (sx.floor(), sy.floor());
        let (tx, ty) = (sx - fx, sy - fy);
        out.iter_mut().for_each(|v| *v = 0.0);
        for (dy, wy) in [(0, 1.0 - ty), (1, ty)] {
            for (dx, wx) in [(0, 1.0 - tx), (1, tx)] {
                let (px, py) = (fx as i64 + dx, fy as i64 + dy);
                if px < 0 || py < 0 || px >= rw as i64 || py >= rh as i64 {
                    continue;
                }
                let k = (py as usize * rw + px as usize) * ch;
                for c in 0..ch {
                    out[c] += src[k + c] * wx * wy;
                }
            }
        }
    };
    doc.transform_canvas(
        width,
        height,
        |image| {
            let raw = image.region_rgba8(x0, y0, rw as u32, rh as u32);
            let src: Vec<f32> = raw
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|&[r, g, b, a]| {
                    let k = a as f32 / 255.0;
                    [r as f32 * k, g as f32 * k, b as f32 * k, a as f32]
                })
                .collect();
            let mut px = [0.0f32; 4];
            let pixels: Vec<u8> = sources
                .iter()
                .flat_map(|&s| {
                    bilinear(&src, 4, s, &mut px);
                    let a = px[3].clamp(0.0, 255.0);
                    if a < 0.5 {
                        return [0; 4];
                    }
                    let k = 255.0 / a;
                    [
                        (px[0] * k).round().clamp(0.0, 255.0) as u8,
                        (px[1] * k).round().clamp(0.0, 255.0) as u8,
                        (px[2] * k).round().clamp(0.0, 255.0) as u8,
                        a.round() as u8,
                    ]
                })
                .collect();
            crate::tile::TiledImage::from_rgba8(width, height, &pixels)
        },
        |selection| {
            let (sw, sh) = (selection.width() as i64, selection.height() as i64);
            let src: Vec<f32> = (0..rw * rh)
                .map(|i| {
                    let (x, y) = (x0 + (i % rw) as i64, y0 + (i / rw) as i64);
                    if x < 0 || y < 0 || x >= sw || y >= sh {
                        0.0
                    } else {
                        selection.get(x as u32, y as u32) as f32
                    }
                })
                .collect();
            let mut v = [0.0f32];
            let mask = sources
                .iter()
                .map(|&s| {
                    bilinear(&src, 1, s, &mut v);
                    v[0].round().clamp(0.0, 255.0) as u8
                })
                .collect();
            crate::selection::Selection::from_mask(width, height, mask, false)
        },
    );
}

/// What Image > Trim removes ("Based On").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimBasis {
    /// Fully transparent pixels.
    Transparent,
    /// Pixels the color of the top-left pixel.
    TopLeftColor,
    /// Pixels the color of the bottom-right pixel.
    BottomRightColor,
}

/// Which sides Image > Trim cuts ("Trim Away").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrimSides {
    pub top: bool,
    pub left: bool,
    pub bottom: bool,
    pub right: bool,
}

impl Default for TrimSides {
    fn default() -> Self {
        Self {
            top: true,
            left: true,
            bottom: true,
            right: true,
        }
    }
}

/// The area Image > Trim keeps (x0, y0, x1, y1; exclusive ends), judged on
/// the merged image. `None` when every pixel would be trimmed away.
pub fn trim_bounds(
    doc: &Document,
    basis: TrimBasis,
    sides: TrimSides,
) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = (doc.width, doc.height);
    if w == 0 || h == 0 {
        return None;
    }
    let pixels = doc.composite_rgba8();
    let at = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        pixels[i..i + 4].try_into().unwrap()
    };
    let reference = match basis {
        TrimBasis::Transparent => None,
        TrimBasis::TopLeftColor => Some(at(0, 0)),
        TrimBasis::BottomRightColor => Some(at(w - 1, h - 1)),
    };
    let trimmed = |p: [u8; 4]| match reference {
        None => p[3] == 0,
        Some(r) => p == r,
    };
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if !trimmed(at(x, y)) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    if x0 >= x1 {
        return None;
    }
    Some((
        if sides.left { x0 } else { 0 },
        if sides.top { y0 } else { 0 },
        if sides.right { x1 } else { w },
        if sides.bottom { y1 } else { h },
    ))
}

/// Image > Trim. Returns whether the canvas changed.
pub fn trim(doc: &mut Document, basis: TrimBasis, sides: TrimSides) -> bool {
    match trim_bounds(doc, basis, sides) {
        Some(b) if b != (0, 0, doc.width, doc.height) => {
            crop(doc, b.0, b.1, b.2, b.3);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::selection::{Rect, Selection};

    /// Photoshop 2026 on a 100×100 document: A's square half past the left
    /// edge, B merged into it, then rotated, halved and flipped; the layer's
    /// bounds after each step, pixels outside the canvas included.
    #[test]
    fn pixels_outside_the_canvas_follow_merges_rotation_and_image_size() {
        let mut doc = Document::new_with_background("t", 100, 100, Color::WHITE);
        let square = |x0: i64, y0: i64, x1: i64, y1: i64| {
            let mut image = crate::tile::TiledImage::new(100, 100);
            for y in y0..y1 {
                for x in x0..x1 {
                    image.set_pixel_at(x, y, [255, 0, 0, 255]);
                }
            }
            image
        };
        let a = doc.new_layer_id();
        doc.layers
            .push(crate::Layer::raster(a, "A", square(-20, 10, 10, 40)));
        let b = doc.new_layer_id();
        doc.layers
            .push(crate::Layer::raster(b, "B", square(50, 50, 60, 60)));
        doc.select_layer(b);
        let bounds = |doc: &Document| doc.layer(a).unwrap().image().unwrap().content_bounds();
        assert!(crate::layer_ops::merge_down(&mut doc));
        assert_eq!(bounds(&doc), Some((-20, 10, 60, 60)));
        reorient(&mut doc, Orientation::Rotate90Clockwise);
        assert_eq!(bounds(&doc), Some((40, -20, 90, 60)));
        resize(&mut doc, 50, 50, Resample::NearestNeighbor);
        assert_eq!(bounds(&doc), Some((20, -10, 45, 30)));
        reorient(&mut doc, Orientation::FlipHorizontal);
        assert_eq!(bounds(&doc), Some((5, -10, 30, 30)));
        // The background never keeps anything outside
        assert!(!doc.layers[0].image().unwrap().has_pixels_outside());
    }

    /// Every method keeps a flat color flat; Automatic sharpens when
    /// reducing and smooths when enlarging; the sharper kernel overshoots
    /// more at an edge than the smoother one.
    #[test]
    fn resample_methods() {
        assert_eq!(Resample::Automatic.resolve(false), Resample::BicubicSharper);
        assert_eq!(Resample::Automatic.resolve(true), Resample::BicubicSmoother);
        let flat = vec![100.0f32; 8 * 8];
        for m in Resample::ALL {
            for (dw, dh) in [(3, 3), (20, 20)] {
                let out = resample_buffer(&flat, 8, 8, dw, dh, 1, m);
                assert!(out.iter().all(|v| (v - 100.0).abs() < 0.01), "{m:?}");
            }
        }
        // A step from 0 to 200, enlarged 4×: the peak right of the edge
        let step: Vec<f32> = (0..8).map(|x| if x < 4 { 0.0 } else { 200.0 }).collect();
        let peak = |m| {
            resample_buffer(&step, 8, 1, 32, 1, 1, m)
                .into_iter()
                .fold(f32::MIN, f32::max)
        };
        assert!(peak(Resample::BicubicSharper) > peak(Resample::Bicubic));
        assert!(peak(Resample::Bicubic) > peak(Resample::BicubicSmoother));
        assert!(peak(Resample::Bilinear) <= 200.0 + 1e-3);
    }

    /// Photoshop 2026: 200 × 100 by 30° is 224 × 187, then by −45° is
    /// 291 × 291; the background's new corners take the background color,
    /// a layer's stay transparent, and a dot turns about the center.
    #[test]
    fn rotate_arbitrary_like_photoshop() {
        let mut doc = Document::new_with_background("t", 200, 100, Color::WHITE);
        let mut image = crate::tile::TiledImage::new(200, 100);
        // A dot right of the center
        for y in 48..52 {
            for x in 148..152 {
                image.set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        let id = doc.new_layer_id();
        doc.layers.push(crate::Layer::raster(id, "dot", image));
        assert!(!rotate_arbitrary(&mut doc, 360.0, Color::BLACK));
        assert!(rotate_arbitrary(&mut doc, 90.0, Color::BLACK));
        assert_eq!((doc.width, doc.height), (100, 200));
        // Clockwise: right of the center goes below it
        let px = |doc: &Document, x, y| doc.layer(id).unwrap().image().unwrap().pixel(x, y);
        assert_eq!(px(&doc, 50, 150)[0], 255);
        let mut doc = Document::new_with_background("t", 200, 100, Color::WHITE);
        assert!(rotate_arbitrary(&mut doc, 30.0, Color::BLACK));
        assert_eq!((doc.width, doc.height), (224, 187));
        // A corner is outside the turned image: background color
        assert_eq!(doc.layers[0].image().unwrap().pixel(0, 0), [0, 0, 0, 255]);
        assert_eq!(
            doc.layers[0].image().unwrap().pixel(112, 93),
            [255, 255, 255, 255]
        );
        assert!(rotate_arbitrary(&mut doc, -45.0, Color::BLACK));
        assert_eq!((doc.width, doc.height), (291, 291));
    }

    #[test]
    fn crop_tool_crops_past_the_canvas_and_can_keep_pixels() {
        let mut doc = Document::new_with_background("t", 10, 10, Color::WHITE);
        let mut image = crate::tile::TiledImage::new(10, 10);
        image.set_pixel(1, 1, [255, 0, 0, 255]);
        let id = doc.new_layer_id();
        doc.layers.push(crate::Layer::raster(id, "L", image));
        // Past the right and bottom edges: the background grows with the
        // background color, the layer stays transparent there
        assert!(crop_extended(&mut doc, (5, 5, 15, 12), true, Color::BLACK));
        assert_eq!((doc.width, doc.height), (10, 7));
        let bg = doc.layers[0].image().unwrap();
        assert_eq!(bg.pixel(0, 0), [255, 255, 255, 255]);
        assert_eq!(bg.pixel(9, 6), [0, 0, 0, 255]);
        assert!(!doc.layer(id).unwrap().image().unwrap().has_pixels_outside());
        // Without deleting: the dot stays outside, the background becomes
        // a layer
        let mut doc = Document::new_with_background("t", 10, 10, Color::WHITE);
        let mut image = crate::tile::TiledImage::new(10, 10);
        image.set_pixel(1, 1, [255, 0, 0, 255]);
        doc.layers.push(crate::Layer::raster(id, "L", image));
        assert!(crop_extended(&mut doc, (4, 4, 8, 8), false, Color::BLACK));
        assert!(!doc.layers[0].is_background);
        assert_eq!(doc.layers[0].name, "Layer 0");
        assert_eq!(
            doc.layer(id).unwrap().image().unwrap().pixel_at(-3, -3),
            [255, 0, 0, 255]
        );
        assert!(!crop_extended(&mut doc, (4, 4, 4, 8), false, Color::BLACK));
    }

    /// 3×2 white background with red at (0, 0) and blue at (2, 1).
    fn doc() -> Document {
        let mut doc = Document::new_with_background("t", 3, 2, Color::WHITE);
        let id = doc.active_layer.unwrap();
        let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(0, 0, [255, 0, 0, 255]);
        image.set_pixel(2, 1, [0, 0, 255, 255]);
        doc
    }

    fn pixel(doc: &Document, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * doc.width + x) * 4) as usize;
        doc.composite_rgba8()[i..i + 4].try_into().unwrap()
    }

    const RED: [u8; 4] = [255, 0, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];

    #[test]
    fn rotations_and_flips_move_the_corners() {
        let mut d = doc();
        reorient(&mut d, Orientation::Rotate90Clockwise);
        assert_eq!((d.width, d.height), (2, 3));
        assert_eq!(pixel(&d, 1, 0), RED);
        assert_eq!(pixel(&d, 0, 2), BLUE);

        let mut d = doc();
        reorient(&mut d, Orientation::Rotate90CounterClockwise);
        assert_eq!(pixel(&d, 0, 2), RED);
        assert_eq!(pixel(&d, 1, 0), BLUE);

        let mut d = doc();
        reorient(&mut d, Orientation::Rotate180);
        assert_eq!(pixel(&d, 2, 1), RED);
        assert_eq!(pixel(&d, 0, 0), BLUE);

        let mut d = doc();
        reorient(&mut d, Orientation::FlipHorizontal);
        assert_eq!(pixel(&d, 2, 0), RED);
        reorient(&mut d, Orientation::FlipVertical);
        assert_eq!(pixel(&d, 2, 1), RED);
    }

    #[test]
    fn guides_follow_the_canvas() {
        let mut d = doc();
        d.guides = vec![
            Guide {
                vertical: true,
                position: 1.0,
            },
            Guide {
                vertical: false,
                position: 0.5,
            },
        ];
        // 3×2 turned clockwise: x = 1 stays 1 as a row; y = 0.5 becomes column 1.5
        reorient(&mut d, Orientation::Rotate90Clockwise);
        assert_eq!(
            d.guides,
            [
                Guide {
                    vertical: false,
                    position: 1.0
                },
                Guide {
                    vertical: true,
                    position: 1.5
                }
            ]
        );
        crop(&mut d, 1, 0, 2, 3);
        assert_eq!(d.guides[1].position, 0.5);
        resize(&mut d, 2, 6, Resample::NearestNeighbor);
        assert_eq!((d.guides[0].position, d.guides[1].position), (2.0, 1.0));
    }

    #[test]
    fn the_selection_turns_with_the_canvas() {
        let mut d = doc();
        let s = Selection::rect(3, 2, Rect::new(0.0, 0.0, 1.0, 1.0));
        d.set_selection(Some(s));
        reorient(&mut d, Orientation::Rotate90Clockwise);
        assert_eq!(d.selection().unwrap().bounds(), Some((1, 0, 2, 1)));
    }

    #[test]
    fn crop_keeps_the_selected_area() {
        let mut d = doc();
        let s = Selection::rect(3, 2, Rect::new(1.0, 1.0, 3.0, 2.0));
        d.set_selection(Some(s));
        assert!(crop_to_selection(&mut d));
        assert_eq!((d.width, d.height), (2, 1));
        assert_eq!(pixel(&d, 1, 0), BLUE);
        assert_eq!(d.selection().unwrap().bounds(), Some((0, 0, 2, 1)));
        d.set_selection(None);
        assert!(!crop_to_selection(&mut d));
    }

    #[test]
    fn resize_scales_layers_and_selection() {
        let mut d = Document::new_with_background("t", 4, 2, Color::WHITE);
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(0, 0, RED);
        image.set_pixel(1, 0, RED);
        image.set_pixel(0, 1, RED);
        image.set_pixel(1, 1, RED);
        d.set_selection(Some(Selection::rect(4, 2, Rect::new(0.0, 0.0, 2.0, 2.0))));
        // Halving: the red half becomes one (mostly) red pixel; bicubic
        // takes in a little of the white next to it
        resize(&mut d, 2, 1, Resample::Bicubic);
        assert_eq!((d.width, d.height), (2, 1));
        let red = pixel(&d, 0, 0);
        assert!(red[0] == 255 && red[1] < 40, "{red:?}");
        assert!(pixel(&d, 1, 0)[1] > 215);
        assert!(d.selection().unwrap().get(0, 0) > 215);
        // Nearest neighbor doubling keeps hard edges
        let mut d = Document::new_with_background("t", 2, 1, Color::WHITE);
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(0, 0, RED);
        resize(&mut d, 4, 2, Resample::NearestNeighbor);
        assert_eq!(pixel(&d, 1, 1), RED);
        assert_eq!(pixel(&d, 2, 1), [255, 255, 255, 255]);
        // Bilinear doubling blends at the edge
        let mut e = doc();
        resize(&mut e, 6, 4, Resample::Bilinear);
        let edge = pixel(&e, 1, 0);
        assert!(edge[1] > 0 && edge[1] < 255, "{edge:?}");
    }

    #[test]
    fn reduce_noise_smooths_speckle_and_keeps_edges() {
        // Gray speckle (±10) on the left, black on the right
        let make = || {
            let mut d = Document::new_with_background("t", 16, 8, Color::WHITE);
            let id = d.active_layer.unwrap();
            let image = d.layer_mut(id).unwrap().image_mut().unwrap();
            for y in 0..8 {
                for x in 0..16 {
                    let v = if x >= 8 {
                        0
                    } else if (x + y) % 2 == 0 {
                        138
                    } else {
                        118
                    };
                    image.set_pixel(x, y, [v, v, v, 255]);
                }
            }
            d
        };
        let spread = |d: &Document| {
            let row: Vec<i32> = (2..12).map(|x| pixel(d, x, 8)[0] as i32).collect();
            row.iter().max().unwrap() - row.iter().min().unwrap()
        };
        let mut plain = make();
        resize(&mut plain, 32, 16, Resample::PreserveDetails);
        let mut quiet = make();
        resize_reducing_noise(&mut quiet, 32, 16, Resample::PreserveDetails, 1.0);
        assert!(
            spread(&quiet) < spread(&plain) / 2,
            "{} {}",
            spread(&quiet),
            spread(&plain)
        );
        // The edge stays hard
        assert!(pixel(&quiet, 18, 8)[0] < 5);
        // Other methods ignore it
        let mut a = make();
        let mut b = make();
        resize(&mut a, 32, 16, Resample::Bicubic);
        resize_reducing_noise(&mut b, 32, 16, Resample::Bicubic, 1.0);
        assert_eq!(pixel(&a, 4, 4), pixel(&b, 4, 4));
    }

    #[test]
    fn perspective_crop_maps_the_quad_onto_the_canvas() {
        // Four colored quarters
        let mut d = Document::new_with_background("t", 100, 100, Color::WHITE);
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        let colors = [
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [0, 0, 255, 255],
            [255, 255, 0, 255],
        ];
        for y in 0..100 {
            for x in 0..100 {
                let k = (x >= 50) as usize + 2 * (y >= 50) as usize;
                image.set_pixel(x, y, colors[k]);
            }
        }
        let quad = [(10.0, 10.0), (90.0, 20.0), (80.0, 90.0), (20.0, 80.0)];
        assert_eq!(perspective_size(quad), (71, 71));
        perspective_crop(&mut d, quad, 60, 40);
        assert_eq!((d.width, d.height), (60, 40));
        // Each corner of the result shows the quarter its quad corner is in
        assert_eq!(pixel(&d, 1, 1), colors[0]);
        assert_eq!(pixel(&d, 58, 1), colors[1]);
        assert_eq!(pixel(&d, 1, 38), colors[2]);
        assert_eq!(pixel(&d, 58, 38), colors[3]);
        // A quad that is the canvas keeps the image
        let mut e = doc();
        let before = pixel(&e, 1, 0);
        let (ew, eh) = (e.width, e.height);
        let (w, h) = (ew as f32, eh as f32);
        perspective_crop(&mut e, [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)], ew, eh);
        assert_eq!(pixel(&e, 1, 0), before);
    }

    #[test]
    fn trim_removes_borders_of_the_corner_color() {
        let mut d = Document::new_with_background("t", 5, 5, Color::WHITE);
        let id = d.active_layer.unwrap();
        let image = d.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(2, 1, RED);
        image.set_pixel(3, 3, RED);
        assert_eq!(
            trim_bounds(&d, TrimBasis::TopLeftColor, TrimSides::default()),
            Some((2, 1, 4, 4))
        );
        let only_top = TrimSides {
            top: true,
            left: false,
            bottom: false,
            right: false,
        };
        assert_eq!(
            trim_bounds(&d, TrimBasis::BottomRightColor, only_top),
            Some((0, 1, 5, 5))
        );
        // Nothing transparent on an opaque background
        assert!(!trim(&mut d, TrimBasis::Transparent, TrimSides::default()));
        assert!(trim(&mut d, TrimBasis::TopLeftColor, TrimSides::default()));
        assert_eq!((d.width, d.height), (2, 3));
    }

    #[test]
    fn reveal_all_grows_the_canvas_to_the_hidden_pixels() {
        let mut doc = Document::new_with_background("t", 4, 4, Color::WHITE);
        let id = doc.new_layer_id();
        let mut image = crate::TiledImage::new(4, 4);
        image.set_pixel_at(-2, 1, [255, 0, 0, 255]);
        image.set_pixel_at(5, 6, [0, 0, 255, 255]);
        doc.layers.push(crate::Layer::raster(id, "L", image));
        assert!(reveal_all(&mut doc, Color::BLACK));
        assert_eq!((doc.width, doc.height), (8, 7));
        let image = doc.layer(id).unwrap().image().unwrap();
        assert_eq!(image.pixel(0, 1), [255, 0, 0, 255]);
        assert_eq!(image.pixel(7, 6), [0, 0, 255, 255]);
        // The background is extended with the background color
        let bg = doc.layers[0].image().unwrap();
        assert_eq!(bg.pixel(0, 0), [0, 0, 0, 255]);
        assert_eq!(bg.pixel(2, 0), [255, 255, 255, 255]);
        assert!(!reveal_all(&mut doc, Color::BLACK));
    }
}
