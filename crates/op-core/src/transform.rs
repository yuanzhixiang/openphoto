//! Edit > Free Transform and Edit > Transform: moving, scaling, rotating
//! and flipping the selected and linked layers (or the selected pixels) by an
//! affine transform, resampled bilinearly.

use crate::document::Document;
use crate::layer::LayerId;
use crate::selection::Selection;
use crate::tile::TiledImage;

/// A 2-D affine map: `x' = a·x + b·y + c`, `y' = d·x + e·y + f`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Affine {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 1.0,
        f: 0.0,
    };

    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            c: x,
            f: y,
            ..Self::IDENTITY
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            e: sy,
            ..Self::IDENTITY
        }
    }

    /// Rotation by `angle` radians (clockwise on screen, y pointing down).
    pub fn rotate(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            a: c,
            b: -s,
            d: s,
            e: c,
            ..Self::IDENTITY
        }
    }

    /// Skew by `h` radians horizontally and `v` vertically (Free
    /// Transform's H and V).
    pub fn skew(h: f32, v: f32) -> Self {
        Self {
            b: h.tan(),
            d: v.tan(),
            ..Self::IDENTITY
        }
    }

    /// `self` applied after `first`.
    pub fn after(self, first: Self) -> Self {
        Self {
            a: self.a * first.a + self.b * first.d,
            b: self.a * first.b + self.b * first.e,
            c: self.a * first.c + self.b * first.f + self.c,
            d: self.d * first.a + self.e * first.d,
            e: self.d * first.b + self.e * first.e,
            f: self.d * first.c + self.e * first.f + self.f,
        }
    }

    pub fn apply(self, (x, y): (f32, f32)) -> (f32, f32) {
        (
            self.a * x + self.b * y + self.c,
            self.d * x + self.e * y + self.f,
        )
    }

    pub fn inverse(self) -> Option<Self> {
        let det = self.a * self.e - self.b * self.d;
        if det.abs() < 1e-9 {
            return None;
        }
        let (a, b, d, e) = (self.e / det, -self.b / det, -self.d / det, self.a / det);
        Some(Self {
            a,
            b,
            c: -(a * self.c + b * self.f),
            d,
            e,
            f: -(d * self.c + e * self.f),
        })
    }

    /// Scale `sx`, `sy` and rotate `angle` around `center`, then move by
    /// `offset`: what a Free Transform box describes.
    pub fn around(center: (f32, f32), sx: f32, sy: f32, angle: f32, offset: (f32, f32)) -> Self {
        Self::translate(center.0 + offset.0, center.1 + offset.1)
            .after(Self::rotate(angle))
            .after(Self::scale(sx, sy))
            .after(Self::translate(-center.0, -center.1))
    }
}

/// A 2-D projective map (a homography), for Distort and Perspective:
/// `(x, y) -> ((m0 x + m1 y + m2) / w, (m3 x + m4 y + m5) / w)` with
/// `w = m6 x + m7 y + m8`. Computed in doubles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projective {
    pub m: [f64; 9],
}

impl Projective {
    pub const IDENTITY: Self = Self {
        m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    pub fn from_affine(a: Affine) -> Self {
        let f = |v: f32| v as f64;
        Self {
            m: [
                f(a.a),
                f(a.b),
                f(a.c),
                f(a.d),
                f(a.e),
                f(a.f),
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    /// `self` applied after `first`.
    pub fn after(self, first: Self) -> Self {
        let (a, b) = (self.m, first.m);
        let mut m = [0.0; 9];
        for r in 0..3 {
            for c in 0..3 {
                m[r * 3 + c] = (0..3).map(|k| a[r * 3 + k] * b[k * 3 + c]).sum();
            }
        }
        Self { m }
    }

    pub fn apply(self, (x, y): (f32, f32)) -> (f32, f32) {
        let m = self.m;
        let (x, y) = (x as f64, y as f64);
        let w = m[6] * x + m[7] * y + m[8];
        (
            ((m[0] * x + m[1] * y + m[2]) / w) as f32,
            ((m[3] * x + m[4] * y + m[5]) / w) as f32,
        )
    }

    pub fn inverse(self) -> Option<Self> {
        let m = self.m;
        let det = m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
            + m[2] * (m[3] * m[7] - m[4] * m[6]);
        if det.abs() < 1e-12 {
            return None;
        }
        let adj = [
            m[4] * m[8] - m[5] * m[7],
            m[2] * m[7] - m[1] * m[8],
            m[1] * m[5] - m[2] * m[4],
            m[5] * m[6] - m[3] * m[8],
            m[0] * m[8] - m[2] * m[6],
            m[2] * m[3] - m[0] * m[5],
            m[3] * m[7] - m[4] * m[6],
            m[1] * m[6] - m[0] * m[7],
            m[0] * m[4] - m[1] * m[3],
        ];
        Some(Self {
            m: adj.map(|v| v / det),
        })
    }

    /// The unit square's corners (0,0), (1,0), (1,1), (0,1) to `quad`
    /// (Heckbert's construction).
    fn square_to_quad(q: [(f64, f64); 4]) -> Self {
        let [(x0, y0), (x1, y1), (x2, y2), (x3, y3)] = q;
        let (dx3, dy3) = (x0 - x1 + x2 - x3, y0 - y1 + y2 - y3);
        if dx3.abs() < 1e-12 && dy3.abs() < 1e-12 {
            return Self {
                m: [x1 - x0, x3 - x0, x0, y1 - y0, y3 - y0, y0, 0.0, 0.0, 1.0],
            };
        }
        let (dx1, dx2, dy1, dy2) = (x1 - x2, x3 - x2, y1 - y2, y3 - y2);
        let det = dx1 * dy2 - dx2 * dy1;
        let g = (dx3 * dy2 - dx2 * dy3) / det;
        let h = (dx1 * dy3 - dx3 * dy1) / det;
        Self {
            m: [
                x1 - x0 + g * x1,
                x3 - x0 + h * x3,
                x0,
                y1 - y0 + g * y1,
                y3 - y0 + h * y3,
                y0,
                g,
                h,
                1.0,
            ],
        }
    }

    /// The box (x0, y0, x1, y1) to `quad` (top-left, top-right,
    /// bottom-right, bottom-left): what a distorted Free Transform box
    /// describes.
    pub fn rect_to_quad(rect: (f32, f32, f32, f32), quad: [(f32, f32); 4]) -> Self {
        let (x0, y0, x1, y1) = rect;
        let (w, h) = (((x1 - x0) as f64).max(1e-9), ((y1 - y0) as f64).max(1e-9));
        let to_unit = Self {
            m: [
                1.0 / w,
                0.0,
                -x0 as f64 / w,
                0.0,
                1.0 / h,
                -y0 as f64 / h,
                0.0,
                0.0,
                1.0,
            ],
        };
        Self::square_to_quad(quad.map(|(x, y)| (x as f64, y as f64))).after(to_unit)
    }

    /// Whether it keeps lines parallel (an affine map).
    pub fn is_affine(self) -> bool {
        self.m[6].abs() < 1e-12 && self.m[7].abs() < 1e-12
    }
}

/// A map `transform` can apply.
pub trait Mapping: Copy {
    fn map(self, p: (f32, f32)) -> (f32, f32);
    fn inverted(self) -> Option<Self>;
}

impl Mapping for Affine {
    fn map(self, p: (f32, f32)) -> (f32, f32) {
        self.apply(p)
    }
    fn inverted(self) -> Option<Self> {
        self.inverse()
    }
}

impl Mapping for Projective {
    fn map(self, p: (f32, f32)) -> (f32, f32) {
        self.apply(p)
    }
    fn inverted(self) -> Option<Self> {
        self.inverse()
    }
}

/// Why a transform can't be done; the messages match Photoshop's alerts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformError {
    NoLayer,
    Hidden,
    Locked,
    /// Nothing to transform: the layer (or the selection on it) is empty.
    Empty,
}

impl TransformError {
    pub fn message(self, command: &str) -> String {
        match self {
            Self::NoLayer => {
                format!("Could not complete the {command} command because there is no layer.")
            }
            Self::Hidden => format!(
                "Could not complete the {command} command because the target layer is hidden."
            ),
            Self::Locked => {
                format!("Could not complete the {command} command because the layer is locked.")
            }
            Self::Empty => format!(
                "Could not complete the {command} command because the selected area is empty."
            ),
        }
    }
}

/// The pixel layers a transform changes: with a selection, the active
/// layer (or the layers in the active group); without one, also the other
/// selected layers and the layers linked to them, as in Photoshop, leaving
/// out those that are hidden, locked or the background.
fn targets(doc: &Document) -> Vec<LayerId> {
    let Some(active) = doc.active_layer else {
        return Vec::new();
    };
    if doc.selection().is_some() {
        return doc.pixel_layers(&[active]);
    }
    doc.pixel_layers(&crate::link::with_linked(doc))
        .into_iter()
        .filter(|&id| {
            id == active
                || doc.layer(id).is_some_and(|l| {
                    doc.is_shown(id)
                        && !l.is_background
                        && !doc.pixels_locked(id)
                        && !doc.position_locked(id)
                })
        })
        .collect()
}

/// What a transform acts on, as (x0, y0, x1, y1): the selection's bounds,
/// or the box around the target layers' non-transparent pixels. Also checks
/// that the active layer can be transformed: the background only with a
/// selection.
pub fn bounds(doc: &Document) -> Result<(f32, f32, f32, f32), TransformError> {
    let layer = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .ok_or(TransformError::NoLayer)?;
    if !doc.is_shown(layer.id) {
        return Err(TransformError::Hidden);
    }
    let selection = doc.selection();
    if doc.pixels_locked(layer.id)
        || doc.position_locked(layer.id)
        || (layer.is_background && selection.is_none())
    {
        return Err(TransformError::Locked);
    }
    // Without a selection the box is around all the target layers' pixels
    // (a group's layers, the other selected and the linked layers)
    if selection.is_none() {
        let (x0, y0, x1, y1) = targets(doc)
            .into_iter()
            .filter_map(|p| doc.layer(p)?.image()?.content_bounds())
            .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)))
            .ok_or(TransformError::Empty)?;
        return Ok((x0 as f32, y0 as f32, x1 as f32, y1 as f32));
    }
    let Some(image) = layer.image() else {
        return Err(TransformError::Empty);
    };
    let (w, h) = (doc.width, doc.height);
    let b = match selection {
        // With a selection, the box is the selection's (as in Photoshop),
        // once it covers some of the layer's pixels
        Some(s) => {
            let covered =
                (0..h).any(|y| (0..w).any(|x| image.pixel(x, y)[3] > 0 && s.get(x, y) > 0));
            covered
                .then(|| s.bounds())
                .flatten()
                .map(|(x0, y0, x1, y1)| (x0 as i64, y0 as i64, x1 as i64, y1 as i64))
        }
        // Otherwise all of the layer's pixels, including those outside
        // the canvas
        None => image.content_bounds(),
    };
    let (x0, y0, x1, y1) = b.ok_or(TransformError::Empty)?;
    Ok((x0 as f32, y0 as f32, x1 as f32, y1 as f32))
}

/// Free Transform's Interpolation, in Photoshop 2026's menu order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum Interpolation {
    NearestNeighbor,
    Bilinear,
    /// Photoshop's default.
    #[default]
    Bicubic,
    BicubicSmoother,
    BicubicSharper,
    BicubicAutomatic,
}

impl Interpolation {
    pub const ALL: [Self; 6] = [
        Self::NearestNeighbor,
        Self::Bilinear,
        Self::Bicubic,
        Self::BicubicSmoother,
        Self::BicubicSharper,
        Self::BicubicAutomatic,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::NearestNeighbor => "Nearest Neighbor",
            Self::Bilinear => "Bilinear",
            Self::Bicubic => "Bicubic",
            Self::BicubicSmoother => "Bicubic Smoother",
            Self::BicubicSharper => "Bicubic Sharper",
            Self::BicubicAutomatic => "Bicubic Automatic",
        }
    }

    /// The cubic kernel's weight at distance `x`, for the bicubic kinds.
    fn cubic(self, x: f32) -> f32 {
        let x = x.abs();
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
            Self::BicubicSharper => keys(-0.75),
            Self::BicubicSmoother => {
                // Mitchell–Netravali, B = C = 1/3
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
            _ => keys(-0.5),
        }
    }
}

/// A sample of premultiplied RGBA at (x, y) in pixel-center coordinates;
/// transparent outside the image.
fn sample(px: &[[f32; 4]], w: usize, h: usize, x: f32, y: f32, how: Interpolation) -> [f32; 4] {
    let at = |ix: i64, iy: i64| -> [f32; 4] {
        if ix < 0 || iy < 0 || ix >= w as i64 || iy >= h as i64 {
            [0.0; 4]
        } else {
            px[iy as usize * w + ix as usize]
        }
    };
    let (fx, fy) = (x - 0.5, y - 0.5);
    match how {
        Interpolation::NearestNeighbor => at(fx.round() as i64, fy.round() as i64),
        Interpolation::Bilinear => {
            let (x0, y0) = (fx.floor(), fy.floor());
            let (tx, ty) = (fx - x0, fy - y0);
            let (x0, y0) = (x0 as i64, y0 as i64);
            let (p00, p10, p01, p11) = (
                at(x0, y0),
                at(x0 + 1, y0),
                at(x0, y0 + 1),
                at(x0 + 1, y0 + 1),
            );
            let mut out = [0.0; 4];
            for c in 0..4 {
                let top = p00[c] + (p10[c] - p00[c]) * tx;
                let bottom = p01[c] + (p11[c] - p01[c]) * tx;
                out[c] = top + (bottom - top) * ty;
            }
            out
        }
        _ => {
            let (x0, y0) = (fx.floor() as i64, fy.floor() as i64);
            let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
            let wx: [f32; 4] = std::array::from_fn(|k| how.cubic(tx - (k as f32 - 1.0)));
            let wy: [f32; 4] = std::array::from_fn(|k| how.cubic(ty - (k as f32 - 1.0)));
            let mut out = [0.0f32; 4];
            for (j, wyj) in wy.iter().enumerate() {
                for (i, wxi) in wx.iter().enumerate() {
                    let p = at(x0 + i as i64 - 1, y0 + j as i64 - 1);
                    for c in 0..4 {
                        out[c] += p[c] * wxi * wyj;
                    }
                }
            }
            // Cubics overshoot: keep a valid premultiplied color
            out[3] = out[3].clamp(0.0, 255.0);
            for c in 0..3 {
                out[c] = out[c].clamp(0.0, out[3]);
            }
            out
        }
    }
}

/// Applies `m` (source → destination, in document pixels) to the active
/// layer. With a selection only the selected pixels move: the place they
/// leave becomes transparent (the background color on the background
/// layer), and the selection moves with them. Pixels outside the canvas
/// move too and stay on the layer, as in Photoshop; the background layer
/// ends at the canvas.
pub fn transform<M: Mapping>(
    doc: &mut Document,
    m: M,
    background: [u8; 3],
) -> Result<(), TransformError> {
    transform_with(doc, m, background, Interpolation::Bicubic)
}

/// [`transform`] with Free Transform's interpolation.
pub fn transform_with<M: Mapping>(
    doc: &mut Document,
    m: M,
    background: [u8; 3],
    how: Interpolation,
) -> Result<(), TransformError> {
    let src_box = bounds(doc)?;
    let inverse = m.inverted().ok_or(TransformError::Empty)?;
    let (bx0, by0, bx1, by1) = src_box;
    let landing = [(bx0, by0), (bx1, by0), (bx1, by1), (bx0, by1)].map(|p| m.map(p));
    resample_targets(doc, &landing, &|p| Some(inverse.map(p)), background, how)
}

/// Moves the target layers' pixels (or the selected ones): every pixel in
/// the area worked on takes the sample at `back(pixel)` (none: nothing
/// lands there). `landing` are points spanning where the pixels go.
fn resample_targets(
    doc: &mut Document,
    landing: &[(f32, f32)],
    back: &dyn Fn((f32, f32)) -> Option<(f32, f32)>,
    background: [u8; 3],
    how: Interpolation,
) -> Result<(), TransformError> {
    let selection = doc.selection().cloned();
    let (cw, ch) = (doc.width as i64, doc.height as i64);
    // Every target layer, with the same map
    for id in targets(doc) {
        let layer = doc.layer_mut(id).expect("checked");
        let is_background = layer.is_background;
        let Some(image) = layer.image_mut() else {
            continue;
        };

        // The region worked on: the canvas, the layer's pixels and where the
        // box lands
        let corners = landing;
        let fx0 = corners.iter().map(|c| c.0).fold(f32::MAX, f32::min).floor() as i64;
        let fy0 = corners.iter().map(|c| c.1).fold(f32::MAX, f32::min).floor() as i64;
        let fx1 = corners.iter().map(|c| c.0).fold(f32::MIN, f32::max).ceil() as i64;
        let fy1 = corners.iter().map(|c| c.1).fold(f32::MIN, f32::max).ceil() as i64;
        let (cx0, cy0, cx1, cy1) = image.content_bounds().unwrap_or((0, 0, cw, ch));
        let (ux0, uy0) = (0.min(cx0).min(fx0), 0.min(cy0).min(fy0));
        let (ux1, uy1) = (cw.max(cx1).max(fx1), ch.max(cy1).max(fy1));
        let (w, h) = ((ux1 - ux0) as usize, (uy1 - uy0) as usize);

        // The moving pixels (premultiplied) and what stays behind (straight)
        let raw = image.region_rgba8(ux0, uy0, w as u32, h as u32);
        let mut moving = vec![[0f32; 4]; w * h];
        let mut staying = raw.clone();
        for i in 0..w * h {
            let (x, y) = (ux0 + (i % w) as i64, uy0 + (i / w) as i64);
            let m = match &selection {
                None => 1.0,
                Some(s) if x >= 0 && y >= 0 && x < cw && y < ch => {
                    s.get(x as u32, y as u32) as f32 / 255.0
                }
                Some(_) => 0.0,
            };
            if m <= 0.0 {
                continue;
            }
            let p = &raw[i * 4..i * 4 + 4];
            let a = p[3] as f32 * m / 255.0;
            moving[i] = [p[0] as f32 * a, p[1] as f32 * a, p[2] as f32 * a, a * 255.0];
            let left = &mut staying[i * 4..i * 4 + 4];
            if is_background {
                for c in 0..3 {
                    left[c] = (left[c] as f32 * (1.0 - m) + background[c] as f32 * m).round() as u8;
                }
            } else {
                left[3] = (left[3] as f32 * (1.0 - m)).round() as u8;
            }
        }

        // Composite the moved pixels over what stayed
        let mut out = staying;
        for y in 0..h {
            for x in 0..w {
                let (dx, dy) = ((ux0 + x as i64) as f32 + 0.5, (uy0 + y as i64) as f32 + 0.5);
                let Some((sx, sy)) = back((dx, dy)) else {
                    continue;
                };
                let s = sample(&moving, w, h, sx - ux0 as f32, sy - uy0 as f32, how);
                let sa = s[3] / 255.0;
                if sa <= 0.0 {
                    continue;
                }
                let i = (y * w + x) * 4;
                let d = &mut out[i..i + 4];
                let da = d[3] as f32 / 255.0;
                let oa = sa + da * (1.0 - sa);
                for c in 0..3 {
                    let v = (s[c] + d[c] as f32 * da * (1.0 - sa)) / oa;
                    d[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                d[3] = (oa * 255.0).round() as u8;
            }
        }
        *image = TiledImage::from_region(cw as u32, ch as u32, ux0, uy0, w as u32, h as u32, &out);
        if is_background {
            *image = image.clipped();
        }
    }

    if let Some(s) = selection {
        doc.set_selection(Some(turned_selection(&s, back)));
    }
    doc.mark_dirty();
    Ok(())
}

/// `s` moved by the map whose inverse is `back`.
fn turned_selection(s: &Selection, back: &dyn Fn((f32, f32)) -> Option<(f32, f32)>) -> Selection {
    let (w, h) = (s.width() as usize, s.height() as usize);
    let src: Vec<[f32; 4]> = (0..w * h)
        .map(|i| {
            let v = s.get((i % w) as u32, (i / w) as u32) as f32;
            [v, 0.0, 0.0, 0.0]
        })
        .collect();
    let mut mask = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let Some((sx, sy)) = back((x as f32 + 0.5, y as f32 + 0.5)) else {
                continue;
            };
            mask[y * w + x] = sample(&src, w, h, sx, sy, Interpolation::Bilinear)[0]
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    Selection::from_mask(w as u32, h as u32, mask, false)
}

/// The box Select > Transform Selection starts with: the selection's.
pub fn selection_bounds(doc: &Document) -> Option<(f32, f32, f32, f32)> {
    let (x0, y0, x1, y1) = doc.selection()?.bounds()?;
    Some((x0 as f32, y0 as f32, x1 as f32, y1 as f32))
}

/// Select > Transform Selection: moves the selection's outline (not the
/// pixels) by `m`. Returns false without a selection.
pub fn transform_selection<M: Mapping>(doc: &mut Document, m: M) -> bool {
    let Some(inverse) = m.inverted() else {
        return false;
    };
    let Some(s) = doc.selection().cloned() else {
        return false;
    };
    doc.set_selection(Some(turned_selection(&s, &|p| Some(inverse.map(p)))));
    true
}

/// Edit > Transform > Warp's surface: `cols` × `rows` bicubic Bézier
/// patches over the box, sharing their edges. The control points form a
/// (3·cols + 1) × (3·rows + 1) grid, row by row from the top left (in
/// document pixels); undistorted they sit at thirds of each patch. With a
/// `style` (Warp's Arc, Flag...) the surface is that shape instead.
#[derive(Clone, Debug, PartialEq)]
pub struct WarpMesh {
    pub cols: usize,
    pub rows: usize,
    pub points: Vec<(f32, f32)>,
    /// Where each column and row of patches starts and ends on the box
    /// (0 to 1): splitting adds one, keeping the surface's parameters.
    pub us: Vec<f32>,
    pub vs: Vec<f32>,
    pub style: Option<StyleWarp>,
}

fn bernstein(t: f32) -> [f32; 4] {
    let s = 1.0 - t;
    [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t]
}

/// A cubic's four points.
type Cubic = [(f32, f32); 4];

/// Splits the cubic `p` at `t` (de Casteljau): the two halves' points.
fn split_cubic(p: Cubic, t: f32) -> (Cubic, Cubic) {
    let lerp = |a: (f32, f32), b: (f32, f32)| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    let (a, b, c) = (lerp(p[0], p[1]), lerp(p[1], p[2]), lerp(p[2], p[3]));
    let (d, e) = (lerp(a, b), lerp(b, c));
    let f = lerp(d, e);
    ([p[0], a, d, f], [f, e, c, p[3]])
}

impl WarpMesh {
    /// The flat single patch over (x0, y0, x1, y1) (Grid: Default).
    pub fn flat(rect: (f32, f32, f32, f32)) -> Self {
        Self::grid(rect, 1, 1)
    }

    /// The flat mesh of `cols` × `rows` patches over the box (Grid: 3 x 3...).
    pub fn grid((x0, y0, x1, y1): (f32, f32, f32, f32), cols: usize, rows: usize) -> Self {
        let (cols, rows) = (cols.max(1), rows.max(1));
        let (nx, ny) = (3 * cols + 1, 3 * rows + 1);
        let points = (0..nx * ny)
            .map(|k| {
                let (i, j) = ((k % nx) as f32, (k / nx) as f32);
                (
                    x0 + (x1 - x0) * i / (nx - 1) as f32,
                    y0 + (y1 - y0) * j / (ny - 1) as f32,
                )
            })
            .collect();
        Self {
            cols,
            rows,
            points,
            us: (0..=cols).map(|i| i as f32 / cols as f32).collect(),
            vs: (0..=rows).map(|j| j as f32 / rows as f32).collect(),
            style: None,
        }
    }

    /// Control points per row.
    pub fn stride(&self) -> usize {
        3 * self.cols + 1
    }

    /// The patch holding (u, v) and the coordinates in it.
    fn patch(&self, u: f32, v: f32) -> (usize, usize, f32, f32) {
        let find = |knots: &[f32], t: f32| {
            let t = t.clamp(0.0, 1.0);
            let k = knots[1..knots.len() - 1]
                .iter()
                .filter(|&&b| b <= t)
                .count();
            let (a, b) = (knots[k], knots[k + 1]);
            (k, ((t - a) / (b - a).max(1e-9)).clamp(0.0, 1.0))
        };
        let (pi, s) = find(&self.us, u);
        let (pj, t) = find(&self.vs, v);
        (pi, pj, s, t)
    }

    /// The surface point at (u, v) in 0..1.
    pub fn at(&self, u: f32, v: f32) -> (f32, f32) {
        if let Some(style) = &self.style {
            return style.at(u, v);
        }
        let (pi, pj, s, t) = self.patch(u, v);
        let (bu, bv) = (bernstein(s), bernstein(t));
        let stride = self.stride();
        let mut p = (0.0, 0.0);
        for (b, wv) in bv.iter().enumerate() {
            for (a, wu) in bu.iter().enumerate() {
                let c = self.points[(3 * pj + b) * stride + 3 * pi + a];
                p.0 += c.0 * wu * wv;
                p.1 += c.1 * wu * wv;
            }
        }
        p
    }

    /// Drags the surface point at (u, v) by `d`: its patch's control
    /// points move by their share of it, so that point follows exactly
    /// (Photoshop's way of warping by dragging inside the mesh).
    pub fn pull(&mut self, (u, v): (f32, f32), d: (f32, f32)) {
        let (pi, pj, s, t) = self.patch(u, v);
        let (bu, bv) = (bernstein(s), bernstein(t));
        let norm: f32 = (0..16).map(|k| (bu[k % 4] * bv[k / 4]).powi(2)).sum();
        if norm <= 0.0 {
            return;
        }
        let stride = self.stride();
        for (b, wv) in bv.iter().enumerate() {
            for (a, wu) in bu.iter().enumerate() {
                let w = wu * wv;
                let p = &mut self.points[(3 * pj + b) * stride + 3 * pi + a];
                p.0 += d.0 * w / norm;
                p.1 += d.1 * w / norm;
            }
        }
    }

    /// Whether control point `k` is drawn and can be grabbed: those on the
    /// patches' edges (anchors and the handles along the edges).
    pub fn shows_point(&self, k: usize) -> bool {
        let stride = self.stride();
        (k % stride).is_multiple_of(3) || (k / stride).is_multiple_of(3)
    }

    /// Split Vertically at `u`: a new column of patches starts there, the
    /// surface unchanged (each row of the patch's points is cut in two).
    pub fn split_u(&mut self, u: f32) {
        let (pi, _, s, _) = self.patch(u, 0.0);
        if !(0.01..=0.99).contains(&s) {
            return;
        }
        let stride = self.stride();
        let ny = 3 * self.rows + 1;
        let mut points = Vec::with_capacity((stride + 3) * ny);
        for j in 0..ny {
            let row = &self.points[j * stride..(j + 1) * stride];
            let seg = [
                row[3 * pi],
                row[3 * pi + 1],
                row[3 * pi + 2],
                row[3 * pi + 3],
            ];
            let (left, right) = split_cubic(seg, s);
            points.extend_from_slice(&row[..3 * pi]);
            points.extend_from_slice(&left);
            points.extend_from_slice(&right[1..]);
            points.extend_from_slice(&row[3 * pi + 4..]);
        }
        self.points = points;
        self.cols += 1;
        self.us.insert(pi + 1, u);
    }

    /// Split Horizontally at `v`: a new row of patches starts there.
    pub fn split_v(&mut self, v: f32) {
        let (_, pj, _, t) = self.patch(0.0, v);
        if !(0.01..=0.99).contains(&t) {
            return;
        }
        let stride = self.stride();
        let ny = 3 * self.rows + 1;
        let mut columns: Vec<Vec<(f32, f32)>> = (0..stride)
            .map(|i| (0..ny).map(|j| self.points[j * stride + i]).collect())
            .collect();
        for col in &mut columns {
            let seg = [
                col[3 * pj],
                col[3 * pj + 1],
                col[3 * pj + 2],
                col[3 * pj + 3],
            ];
            let (top, bottom) = split_cubic(seg, t);
            let mut out = col[..3 * pj].to_vec();
            out.extend_from_slice(&top);
            out.extend_from_slice(&bottom[1..]);
            out.extend_from_slice(&col[3 * pj + 4..]);
            *col = out;
        }
        self.rows += 1;
        self.vs.insert(pj + 1, v);
        let ny = 3 * self.rows + 1;
        self.points = (0..ny * stride)
            .map(|k| columns[k % stride][k / stride])
            .collect();
    }

    /// The style's shape as free points (choosing Custom after a style):
    /// each control point takes the surface point at its place.
    pub fn freeze_style(&mut self) {
        let Some(style) = self.style.take() else {
            return;
        };
        self.refit(&|u, v| style.at(u, v));
    }

    /// Puts every control point on `surface` at its place in the box
    /// (the anchors exactly, the rest approximately).
    fn refit(&mut self, surface: &dyn Fn(f32, f32) -> (f32, f32)) {
        let (nx, ny) = (self.stride(), 3 * self.rows + 1);
        let place = |knots: &[f32], i: usize| {
            let (k, f) = (i / 3, (i % 3) as f32 / 3.0);
            if k + 1 >= knots.len() {
                return 1.0;
            }
            knots[k] + (knots[k + 1] - knots[k]) * f
        };
        for k in 0..nx * ny {
            let (i, j) = (k % nx, k / nx);
            self.points[k] = surface(place(&self.us, i), place(&self.vs, j));
        }
    }

    /// The same surface on a `cols` × `rows` grid (Warp's Grid menu): the
    /// new control points are put on the current surface.
    pub fn regrid(&self, cols: usize, rows: usize) -> Self {
        let (x0, y0) = self.points[0];
        let mut out = Self::grid((x0, y0, x0 + 1.0, y0 + 1.0), cols, rows);
        out.refit(&|u, v| self.at(u, v));
        out
    }
}

/// Warp's preset shapes (Photoshop's Warp menu, in its order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarpStyle {
    Arc,
    ArcLower,
    ArcUpper,
    Arch,
    Bulge,
    ShellLower,
    ShellUpper,
    Flag,
    Wave,
    Fish,
    Rise,
    Fisheye,
    Inflate,
    Squeeze,
    Twist,
}

impl WarpStyle {
    pub const ALL: [Self; 15] = [
        Self::Arc,
        Self::ArcLower,
        Self::ArcUpper,
        Self::Arch,
        Self::Bulge,
        Self::ShellLower,
        Self::ShellUpper,
        Self::Flag,
        Self::Wave,
        Self::Fish,
        Self::Rise,
        Self::Fisheye,
        Self::Inflate,
        Self::Squeeze,
        Self::Twist,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Arc => "Arc",
            Self::ArcLower => "Arc Lower",
            Self::ArcUpper => "Arc Upper",
            Self::Arch => "Arch",
            Self::Bulge => "Bulge",
            Self::ShellLower => "Shell Lower",
            Self::ShellUpper => "Shell Upper",
            Self::Flag => "Flag",
            Self::Wave => "Wave",
            Self::Fish => "Fish",
            Self::Rise => "Rise",
            Self::Fisheye => "Fisheye",
            Self::Inflate => "Inflate",
            Self::Squeeze => "Squeeze",
            Self::Twist => "Twist",
        }
    }
}

/// A preset warp over a box: the style, Bend and the horizontal and
/// vertical distortion (−1..1, the options bar's percentages), and the
/// orientation. The shapes follow Photoshop's pictures of them; the exact
/// curves are Adobe's own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleWarp {
    pub style: WarpStyle,
    pub rect: (f32, f32, f32, f32),
    pub bend: f32,
    pub horizontal: f32,
    pub vertical: f32,
    /// Vertical orientation: the shape turned a quarter (bending sideways).
    pub vertical_orientation: bool,
    /// The box's transform before the warp (scaling, rotating...), applied
    /// after the shape.
    pub then: Projective,
}

impl StyleWarp {
    /// The surface point at (u, v).
    pub fn at(&self, u: f32, v: f32) -> (f32, f32) {
        let (x0, y0, x1, y1) = self.rect;
        let (w, h) = (x1 - x0, y1 - y0);
        // Worked out for the horizontal orientation in a unit box: (a, b)
        // across and down, the bend making the shape
        let (a, b) = if self.vertical_orientation {
            (v, u)
        } else {
            (u, v)
        };
        let (s, du, dv) = (self.bend, 2.0 * a - 1.0, 2.0 * b - 1.0);
        let (mut x, mut y) = (a, b);
        let pi = std::f32::consts::PI;
        match self.style {
            WarpStyle::Arc => {
                // Bent along circles about a center below (above for a
                // negative bend): the middle row keeps its length
                // (worked out in widths, so the rows keep their spacing)
                if s.abs() > 1e-3 {
                    let k = if self.vertical_orientation {
                        w / h
                    } else {
                        h / w
                    }
                    .max(1e-3);
                    let span = s.abs() * pi / 2.0;
                    let r = 1.0 / span;
                    let row = if s > 0.0 { b } else { 1.0 - b };
                    let rr = r + (0.5 - row) * k;
                    let angle = du * span / 2.0;
                    x = 0.5 + rr * angle.sin();
                    let yy = 0.5 + (r - rr * angle.cos()) / k;
                    y = if s > 0.0 { yy } else { 1.0 - yy };
                }
            }
            _ => {
                let bow = 1.0 - du * du;
                match self.style {
                    WarpStyle::ArcLower => y += s * 0.5 * bow * b,
                    WarpStyle::ArcUpper => y -= s * 0.5 * bow * (1.0 - b),
                    WarpStyle::Arch => y -= s * 0.5 * bow,
                    WarpStyle::Bulge => y += s * 0.5 * bow * dv,
                    WarpStyle::ShellLower => y -= s * 0.5 * du * du * b,
                    WarpStyle::ShellUpper => y += s * 0.5 * du * du * (1.0 - b),
                    WarpStyle::Flag => y -= s * 0.25 * (2.0 * pi * a).sin(),
                    WarpStyle::Wave => {
                        y -= s * 0.25 * (2.0 * pi * a).sin() * (0.5 + 0.5 * dv.abs())
                    }
                    WarpStyle::Fish => y += s * 0.5 * (pi * a).sin() * dv * (1.0 - a * 0.6),
                    WarpStyle::Rise => y -= s * 0.5 * (pi * (a - 0.5)).sin(),
                    WarpStyle::Fisheye | WarpStyle::Inflate => {
                        let r2 = (du * du + dv * dv).min(2.0);
                        let k = if self.style == WarpStyle::Fisheye {
                            s * 0.5 * (1.0 - r2 / 2.0).max(0.0)
                        } else {
                            s * 0.25 * bow * (1.0 - dv * dv)
                        };
                        x += du * 0.5 * k;
                        y += dv * 0.5 * k;
                    }
                    WarpStyle::Squeeze => {
                        x -= s * 0.25 * du * (1.0 - dv * dv);
                        y += s * 0.25 * dv * bow;
                    }
                    WarpStyle::Twist => {
                        let r = ((du * du + dv * dv).sqrt() / 2f32.sqrt()).min(1.0);
                        let turn = s * pi / 2.0 * (1.0 - r);
                        let (sn, cs) = turn.sin_cos();
                        x = 0.5 + 0.5 * (du * cs - dv * sn);
                        y = 0.5 + 0.5 * (du * sn + dv * cs);
                    }
                    WarpStyle::Arc => unreachable!("above"),
                }
            }
        }
        // Horizontal and vertical distortion: perspective-like taper
        let (cx, cy) = (x - 0.5, y - 0.5);
        let x = 0.5 + cx * (1.0 + self.vertical * (2.0 * b - 1.0));
        let y = 0.5 + cy * (1.0 + self.horizontal * (2.0 * a - 1.0));
        let (x, y) = if self.vertical_orientation {
            (y, x)
        } else {
            (x, y)
        };
        self.then.apply((x0 + w * x, y0 + h * y))
    }
}

/// Edit > Transform > Warp: the box (x0, y0, x1, y1) of the target
/// layers (or the selected pixels) bent onto `mesh`. The surface is cut
/// into small cells drawn as triangle pairs, each sampled back from the
/// box with `how`.
pub fn warp(
    doc: &mut Document,
    rect: (f32, f32, f32, f32),
    mesh: &WarpMesh,
    background: [u8; 3],
    how: Interpolation,
) -> Result<(), TransformError> {
    bounds(doc)?;
    const CELLS: usize = 24;
    let (x0, y0, x1, y1) = rect;
    // Every cell's source and destination corners
    let grid: Vec<Vec<Corner>> = (0..=CELLS)
        .map(|j| {
            (0..=CELLS)
                .map(|i| {
                    let (u, v) = (i as f32 / CELLS as f32, j as f32 / CELLS as f32);
                    ((x0 + (x1 - x0) * u, y0 + (y1 - y0) * v), mesh.at(u, v))
                })
                .collect()
        })
        .collect();
    let mut triangles: Vec<Triangle> = Vec::new();
    for j in 0..CELLS {
        for i in 0..CELLS {
            let (a, b, c, d) = (
                grid[j][i],
                grid[j][i + 1],
                grid[j + 1][i + 1],
                grid[j + 1][i],
            );
            triangles.push([a, b, c]);
            triangles.push([a, c, d]);
        }
    }
    // Each target pixel takes its color from the triangle it falls in
    let map = WarpMap::new(triangles);
    let landing: Vec<(f32, f32)> = grid.iter().flatten().map(|(_, d)| *d).collect();
    resample_targets(doc, &landing, &|p| map.back(p), background, how)
}

/// A point's place on the box and on the surface.
type Corner = ((f32, f32), (f32, f32));
type Triangle = [Corner; 3];

/// A warp's triangles (source and destination corners each), bucketed on
/// a grid over where they land so a pixel only tries its cell's.
struct WarpMap {
    triangles: Vec<Triangle>,
    origin: (f32, f32),
    cell: f32,
    cols: usize,
    rows: usize,
    buckets: Vec<Vec<u32>>,
}

impl WarpMap {
    fn new(triangles: Vec<Triangle>) -> Self {
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for t in &triangles {
            for (_, d) in t {
                x0 = x0.min(d.0);
                y0 = y0.min(d.1);
                x1 = x1.max(d.0);
                y1 = y1.max(d.1);
            }
        }
        let cell = ((x1 - x0).max(y1 - y0) / 64.0).max(1.0);
        let cols = ((x1 - x0) / cell).ceil() as usize + 1;
        let rows = ((y1 - y0) / cell).ceil() as usize + 1;
        let mut buckets = vec![Vec::new(); cols * rows];
        for (k, t) in triangles.iter().enumerate() {
            let tx0 = t.iter().map(|(_, d)| d.0).fold(f32::MAX, f32::min);
            let tx1 = t.iter().map(|(_, d)| d.0).fold(f32::MIN, f32::max);
            let ty0 = t.iter().map(|(_, d)| d.1).fold(f32::MAX, f32::min);
            let ty1 = t.iter().map(|(_, d)| d.1).fold(f32::MIN, f32::max);
            let (c0, c1) = (((tx0 - x0) / cell) as usize, ((tx1 - x0) / cell) as usize);
            let (r0, r1) = (((ty0 - y0) / cell) as usize, ((ty1 - y0) / cell) as usize);
            for r in r0..=r1.min(rows - 1) {
                for c in c0..=c1.min(cols - 1) {
                    buckets[r * cols + c].push(k as u32);
                }
            }
        }
        Self {
            triangles,
            origin: (x0, y0),
            cell,
            cols,
            rows,
            buckets,
        }
    }

    /// The source point for destination `p`, if a triangle covers it.
    fn back(&self, p: (f32, f32)) -> Option<(f32, f32)> {
        let (fx, fy) = (
            (p.0 - self.origin.0) / self.cell,
            (p.1 - self.origin.1) / self.cell,
        );
        if fx < 0.0 || fy < 0.0 {
            return None;
        }
        let (c, r) = (fx as usize, fy as usize);
        if c >= self.cols || r >= self.rows {
            return None;
        }
        for &k in &self.buckets[r * self.cols + c] {
            let t = &self.triangles[k as usize];
            let [(s0, d0), (s1, d1), (s2, d2)] = *t;
            let det = (d1.1 - d2.1) * (d0.0 - d2.0) + (d2.0 - d1.0) * (d0.1 - d2.1);
            if det.abs() < 1e-9 {
                continue;
            }
            let l0 = ((d1.1 - d2.1) * (p.0 - d2.0) + (d2.0 - d1.0) * (p.1 - d2.1)) / det;
            let l1 = ((d2.1 - d0.1) * (p.0 - d2.0) + (d0.0 - d2.0) * (p.1 - d2.1)) / det;
            let l2 = 1.0 - l0 - l1;
            let eps = -1e-4;
            if l0 >= eps && l1 >= eps && l2 >= eps {
                return Some((
                    s0.0 * l0 + s1.0 * l1 + s2.0 * l2,
                    s0.1 * l0 + s1.1 * l1 + s2.1 * l2,
                ));
            }
        }
        None
    }
}

/// The fixed transforms of Edit > Transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixedTransform {
    Rotate180,
    Rotate90Clockwise,
    Rotate90CounterClockwise,
    FlipHorizontal,
    FlipVertical,
}

impl FixedTransform {
    /// The map around the center of `bounds`.
    pub fn affine(self, (x0, y0, x1, y1): (f32, f32, f32, f32)) -> Affine {
        let center = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let (sx, sy, angle) = match self {
            Self::Rotate180 => (1.0, 1.0, std::f32::consts::PI),
            Self::Rotate90Clockwise => (1.0, 1.0, std::f32::consts::FRAC_PI_2),
            Self::Rotate90CounterClockwise => (1.0, 1.0, -std::f32::consts::FRAC_PI_2),
            Self::FlipHorizontal => (-1.0, 1.0, 0.0),
            Self::FlipVertical => (1.0, -1.0, 0.0),
        };
        Affine::around(center, sx, sy, angle, (0.0, 0.0))
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Rotate180 => "Rotate 180°",
            Self::Rotate90Clockwise => "Rotate 90° Clockwise",
            Self::Rotate90CounterClockwise => "Rotate 90° Counter Clockwise",
            Self::FlipHorizontal => "Flip Horizontal",
            Self::FlipVertical => "Flip Vertical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::layer::Layer;
    use crate::selection::Rect;

    /// 6×6 white background plus a layer with a 2×2 red square at (1, 1).
    fn doc() -> Document {
        let mut doc = Document::new_with_background("t", 6, 6, Color::WHITE);
        let mut image = TiledImage::new(6, 6);
        for y in 1..3 {
            for x in 1..3 {
                image.set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        doc.insert_above_active(Layer::raster(doc.new_layer_id(), "Layer 1", image));
        doc
    }

    fn layer_px(doc: &Document, x: u32, y: u32) -> [u8; 4] {
        let image = doc.layers[1].image().unwrap();
        image.pixel(x, y)
    }

    #[test]
    fn projective_maps_the_box_onto_any_quad() {
        let close =
            |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3;
        let rect = (10.0, 20.0, 110.0, 70.0);
        // A trapezoid: the top pulled in (Perspective)
        let quad = [(30.0, 20.0), (90.0, 20.0), (110.0, 70.0), (10.0, 70.0)];
        let m = Projective::rect_to_quad(rect, quad);
        for (src, dst) in [(10.0, 20.0), (110.0, 20.0), (110.0, 70.0), (10.0, 70.0)]
            .into_iter()
            .zip(quad)
        {
            assert!(close(m.apply(src), dst), "{src:?}");
        }
        assert!(!m.is_affine());
        let back = m.inverse().unwrap();
        assert!(close(back.apply((90.0, 20.0)), (110.0, 20.0)));
        // A parallelogram is affine, and matches the affine map
        let skew = [(20.0, 20.0), (120.0, 20.0), (110.0, 70.0), (10.0, 70.0)];
        let m = Projective::rect_to_quad(rect, skew);
        assert!(m.is_affine());
        assert!(close(m.apply((60.0, 45.0)), (65.0, 45.0)));
        let a = Affine::around((5.0, 5.0), 2.0, 1.0, 0.3, (1.0, 2.0));
        assert!(close(
            Projective::from_affine(a).apply((3.0, 4.0)),
            a.apply((3.0, 4.0))
        ));
    }

    #[test]
    fn distorting_the_layer() {
        // The 2×2 square at (1, 1) on 6×6: its box's top corners squeezed
        // together, so the top row narrows and the bottom stays
        let mut d = doc();
        let b = bounds(&d).unwrap();
        let quad = [(1.5, 1.0), (2.5, 1.0), (3.0, 3.0), (1.0, 3.0)];
        transform_with(
            &mut d,
            Projective::rect_to_quad(b, quad),
            [255; 3],
            Interpolation::Bilinear,
        )
        .unwrap();
        let row = |y| (0..6).map(|x| layer_px(&d, x, y)[3] as u32).sum::<u32>();
        assert!(row(1) < row(2), "{} {}", row(1), row(2));
        assert!(row(2) > 255);
    }

    #[test]
    fn interpolation_methods() {
        // The 2×2 red square doubled about its corner: Nearest Neighbor
        // keeps hard edges, the others blend them
        let doubled = |how| {
            let mut d = doc();
            let m = Affine::translate(1.0, 1.0)
                .after(Affine::scale(2.0, 2.0))
                .after(Affine::translate(-1.0, -1.0));
            transform_with(&mut d, m, [255; 3], how).unwrap();
            (0..6).map(|x| layer_px(&d, x, 2)[3]).collect::<Vec<_>>()
        };
        assert_eq!(
            doubled(Interpolation::NearestNeighbor),
            [0, 255, 255, 255, 255, 0]
        );
        let soft = doubled(Interpolation::Bilinear);
        assert!(soft[4] > 0 && soft[4] < 255);
        for how in Interpolation::ALL {
            let row = doubled(how);
            // The middle stays solid
            assert_eq!(row[3], 255, "{how:?}");
        }
        assert_eq!(Interpolation::default(), Interpolation::Bicubic);
        assert_eq!(Interpolation::ALL[5].label(), "Bicubic Automatic");
    }

    #[test]
    fn transforming_the_selection_only() {
        let mut d = doc();
        d.set_selection(Some(Selection::rect(6, 6, Rect::new(1.0, 1.0, 3.0, 3.0))));
        assert_eq!(selection_bounds(&d), Some((1.0, 1.0, 3.0, 3.0)));
        assert!(transform_selection(&mut d, Affine::translate(2.0, 1.0)));
        assert_eq!(d.selection().unwrap().bounds(), Some((3, 2, 5, 4)));
        // The pixels stay
        assert_eq!(layer_px(&d, 1, 1), [255, 0, 0, 255]);
        d.set_selection(None);
        assert!(!transform_selection(&mut d, Affine::IDENTITY));
    }

    #[test]
    fn warp_mesh_and_pull() {
        let mut m = WarpMesh::flat((0.0, 0.0, 30.0, 60.0));
        assert_eq!(m.points[5], (10.0, 20.0));
        // Flat: the surface is the box
        let (x, y) = m.at(0.5, 0.25);
        assert!((x - 15.0).abs() < 1e-4 && (y - 15.0).abs() < 1e-4);
        // Pulling a point drags it exactly where it's pulled
        m.pull((0.5, 0.5), (4.0, -2.0));
        let (x, y) = m.at(0.5, 0.5);
        assert!(
            (x - 19.0).abs() < 1e-3 && (y - 28.0).abs() < 1e-3,
            "{x} {y}"
        );
        // The corners move much less
        assert!((m.at(0.0, 0.0).0).abs() < 1.0);
    }

    #[test]
    fn warp_grids_splits_and_styles() {
        let close =
            |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3;
        // A 3 × 3 grid is flat too, with 10 × 10 points
        let g = WarpMesh::grid((0.0, 0.0, 90.0, 60.0), 3, 3);
        assert_eq!(g.points.len(), 100);
        assert!(close(g.at(0.4, 0.7), (36.0, 42.0)));
        // Pulling a point moves only its own patch
        let mut p = g.clone();
        p.pull((0.5, 0.5), (3.0, 0.0));
        assert!(close(p.at(0.5, 0.5), (48.0, 30.0)));
        assert!(close(p.at(0.1, 0.1), g.at(0.1, 0.1)));
        // Splitting keeps the surface and adds a column or row of patches
        let mut m = WarpMesh::flat((0.0, 0.0, 30.0, 60.0));
        m.pull((0.3, 0.6), (5.0, -4.0));
        let mut split = m.clone();
        split.split_u(0.4);
        split.split_v(0.25);
        assert_eq!((split.cols, split.rows, split.points.len()), (2, 2, 49));
        // The same surface
        assert!(close(split.at(0.0, 0.0), m.at(0.0, 0.0)));
        assert!(close(split.at(1.0, 1.0), m.at(1.0, 1.0)));
        for (u, v) in [(0.2, 0.1), (0.4, 0.25), (0.7, 0.6), (0.95, 0.9)] {
            assert!(close(split.at(u, v), m.at(u, v)), "{u} {v}");
        }
        assert_eq!(split.us, vec![0.0, 0.4, 1.0]);
        // A new grid keeps the shape at its anchors
        let r = split.regrid(3, 3);
        assert!(close(
            r.at(1.0 / 3.0, 2.0 / 3.0),
            m.at(1.0 / 3.0, 2.0 / 3.0)
        ));
        // Only anchors and edge handles show
        assert!(split.shows_point(0) && split.shows_point(3) && !split.shows_point(8));
        // A style with no bend is flat; a bend changes the shape
        for style in WarpStyle::ALL {
            let mut w = StyleWarp {
                style,
                rect: (0.0, 0.0, 100.0, 50.0),
                bend: 0.0,
                horizontal: 0.0,
                vertical: 0.0,
                vertical_orientation: false,
                then: Projective::IDENTITY,
            };
            assert!(close(w.at(0.25, 0.75), (25.0, 37.5)), "{style:?}");
            w.bend = 0.5;
            let moved = (0..=4)
                .flat_map(|j| (0..=4).map(move |i| (i as f32 / 4.0, j as f32 / 4.0)))
                .any(|(u, v)| !close(w.at(u, v), (100.0 * u, 50.0 * v)));
            assert!(moved, "{style:?} bends");
        }
        // Arc keeps the middle of the top edge raised for a positive bend
        let arc = StyleWarp {
            style: WarpStyle::Arc,
            rect: (0.0, 0.0, 100.0, 50.0),
            bend: 0.5,
            horizontal: 0.0,
            vertical: 0.0,
            vertical_orientation: false,
            then: Projective::IDENTITY,
        };
        assert!(arc.at(0.5, 0.0).1 < arc.at(0.0, 0.0).1);
        // Freezing a style turns it into points on the surface
        let mut f = WarpMesh::flat((0.0, 0.0, 100.0, 50.0));
        f.style = Some(arc);
        f.freeze_style();
        assert!(f.style.is_none() && close(f.points[0], arc.at(0.0, 0.0)));
    }

    #[test]
    fn warping_the_layer() {
        // A flat mesh changes nothing; a pulled one moves the middle
        let mut d = doc();
        let b = bounds(&d).unwrap();
        let mut mesh = WarpMesh::flat(b);
        warp(&mut d, b, &mesh, [255; 3], Interpolation::Bilinear).unwrap();
        assert_eq!(layer_px(&d, 1, 1), [255, 0, 0, 255]);
        assert_eq!(layer_px(&d, 3, 3)[3], 0);
        let mut d = doc();
        mesh.pull((0.5, 0.5), (2.0, 2.0));
        warp(&mut d, b, &mesh, [255; 3], Interpolation::Bilinear).unwrap();
        assert!(layer_px(&d, 3, 3)[3] > 0, "the middle went down-right");
    }

    #[test]
    fn affine_math() {
        let m = Affine::around((1.0, 1.0), 2.0, 2.0, 0.0, (3.0, 0.0));
        assert_eq!(m.apply((2.0, 1.0)), (6.0, 1.0));
        let back = m.inverse().unwrap().apply((6.0, 1.0));
        assert!((back.0 - 2.0).abs() < 1e-5 && (back.1 - 1.0).abs() < 1e-5);
        let r = Affine::rotate(std::f32::consts::FRAC_PI_2).apply((1.0, 0.0));
        assert!(r.0.abs() < 1e-6 && (r.1 - 1.0).abs() < 1e-6);
        assert!(Affine::scale(0.0, 1.0).inverse().is_none());
    }

    #[test]
    fn bounds_and_locks() {
        let mut d = doc();
        assert_eq!(bounds(&d), Ok((1.0, 1.0, 3.0, 3.0)));
        d.active_layer = Some(d.layers[0].id);
        assert_eq!(bounds(&d), Err(TransformError::Locked));
        d.set_selection(Some(Selection::rect(6, 6, Rect::new(0.0, 0.0, 4.0, 4.0))));
        assert_eq!(bounds(&d), Ok((0.0, 0.0, 4.0, 4.0)));
    }

    /// Layer 1 and a second layer selected, a third linked to the second
    /// and a fourth locked but linked: the box spans the three movable
    /// ones and they move together; the locked one stays.
    #[test]
    fn selected_and_linked_layers_transform_together() {
        let mut d = doc();
        let one = d.layers[1].id;
        let dot = |d: &mut Document, x: u32, y: u32| {
            let mut image = TiledImage::new(6, 6);
            image.set_pixel(x, y, [0, 0, 255, 255]);
            let id = d.new_layer_id();
            d.layers.push(Layer::raster(id, "dot", image));
            id
        };
        let two = dot(&mut d, 4, 4);
        let three = dot(&mut d, 5, 0);
        let locked = dot(&mut d, 0, 5);
        d.layer_mut(locked).unwrap().lock_position = true;
        d.set_selected_layers(vec![two, three, locked]);
        crate::link::link_selected(&mut d);
        d.set_selected_layers(vec![two, one]);
        assert_eq!(bounds(&d).unwrap(), (1.0, 0.0, 6.0, 5.0));
        transform(&mut d, Affine::translate(0.0, 1.0), [255; 3]).unwrap();
        let px = |d: &Document, id, x, y| d.layer(id).unwrap().image().unwrap().pixel(x, y);
        assert_eq!(px(&d, one, 1, 2), [255, 0, 0, 255]);
        assert_eq!(px(&d, two, 4, 5), [0, 0, 255, 255]);
        assert_eq!(px(&d, three, 5, 1), [0, 0, 255, 255]);
        assert_eq!(px(&d, locked, 0, 5), [0, 0, 255, 255]);
        // With a selection only the active layer's selected pixels move
        d.set_selection(Some(Selection::rect(6, 6, Rect::new(0.0, 0.0, 6.0, 6.0))));
        transform(&mut d, Affine::translate(0.0, -1.0), [255; 3]).unwrap();
        assert_eq!(px(&d, one, 1, 1), [255, 0, 0, 255]);
        assert_eq!(px(&d, two, 4, 5), [0, 0, 255, 255]);
    }

    #[test]
    fn move_scale_and_flip_the_layer() {
        let mut d = doc();
        transform(&mut d, Affine::translate(3.0, 2.0), [255; 3]).unwrap();
        assert_eq!(layer_px(&d, 4, 3), [255, 0, 0, 255]);
        assert_eq!(layer_px(&d, 1, 1)[3], 0);

        let mut d = doc();
        let double = Affine::around((2.0, 2.0), 2.0, 2.0, 0.0, (0.0, 0.0));
        transform(&mut d, double, [255; 3]).unwrap();
        // 0..4 after scaling; the outer ring is softened by interpolation
        assert_eq!(layer_px(&d, 1, 1), [255, 0, 0, 255]);
        assert_eq!(layer_px(&d, 2, 2), [255, 0, 0, 255]);
        assert!(layer_px(&d, 0, 0)[3] > 0 && layer_px(&d, 4, 4)[3] < 255);

        let mut d = doc();
        let b = bounds(&d).unwrap();
        let flip = FixedTransform::FlipHorizontal.affine((b.0, b.1, b.2 + 2.0, b.3));
        transform(&mut d, flip, [255; 3]).unwrap();
        assert_eq!(layer_px(&d, 3, 1), [255, 0, 0, 255]);
        assert_eq!(layer_px(&d, 1, 1)[3], 0);
    }

    #[test]
    fn selected_pixels_move_and_leave_the_background_color() {
        let mut d = doc();
        d.active_layer = Some(d.layers[0].id);
        let image = d.layers[0].image_mut().unwrap();
        image.set_pixel(0, 0, [0, 0, 255, 255]);
        d.set_selection(Some(Selection::rect(6, 6, Rect::new(0.0, 0.0, 1.0, 1.0))));
        transform(&mut d, Affine::translate(5.0, 0.0), [0, 255, 0]).unwrap();
        let image = d.layers[0].image().unwrap();
        assert_eq!(image.pixel(5, 0), [0, 0, 255, 255]);
        assert_eq!(image.pixel(0, 0), [0, 255, 0, 255]);
        assert_eq!(d.selection().unwrap().bounds(), Some((5, 0, 6, 1)));
    }

    #[test]
    fn transforming_takes_pixels_outside_the_canvas_along() {
        let mut doc = Document::new_with_background("t", 10, 10, crate::Color::WHITE);
        let id = doc.new_layer_id();
        let mut image = TiledImage::new(10, 10);
        // A 2 × 1 bar half outside the left edge
        image.set_pixel_at(-1, 4, [255, 0, 0, 255]);
        image.set_pixel_at(0, 4, [255, 0, 0, 255]);
        doc.layers.push(crate::Layer::raster(id, "L", image));
        doc.active_layer = Some(id);
        assert_eq!(bounds(&doc), Ok((-1.0, 4.0, 1.0, 5.0)));
        // Moving it right by 3 brings the outside pixel in
        transform(&mut doc, Affine::translate(3.0, 0.0), [0; 3]).unwrap();
        let image = doc.layer(id).unwrap().image().unwrap();
        assert_eq!(image.pixel(2, 4), [255, 0, 0, 255]);
        assert_eq!(image.pixel(3, 4), [255, 0, 0, 255]);
        assert!(!image.has_pixels_outside());
        // Moving it up by 8 puts it outside, where it is kept
        transform(&mut doc, Affine::translate(0.0, -8.0), [0; 3]).unwrap();
        let image = doc.layer(id).unwrap().image().unwrap();
        assert_eq!(image.content_bounds(), Some((2, -4, 4, -3)));
    }
}
