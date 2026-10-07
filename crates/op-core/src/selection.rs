//! Pixel selections. Like Photoshop, a selection is an 8-bit mask the size of
//! the document: 255 is fully selected, 0 unselected, values in between are
//! partially selected (anti-aliased or feathered edges).

/// How a new selection shape combines with the existing selection (the four
/// buttons in the marquee options bar, or Shift/Alt while dragging).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelectionOp {
    #[default]
    Replace,
    Add,
    Subtract,
    Intersect,
}

/// A rectangle in document pixels; `x1`/`y1` are exclusive and may lie
/// outside the document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Rect {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.x1 - self.x0 < 1.0 || self.y1 - self.y0 < 1.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    width: u32,
    height: u32,
    mask: Vec<u8>,
}

impl Selection {
    fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            mask: vec![0; (width as usize) * (height as usize)],
        }
    }

    /// Select > All.
    pub fn all(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            mask: vec![255; (width as usize) * (height as usize)],
        }
    }

    /// A rectangle snapped to whole pixels (the Rectangular Marquee).
    pub fn rect(width: u32, height: u32, r: Rect) -> Self {
        let mut s = Self::empty(width, height);
        let x0 = r.x0.round().clamp(0.0, width as f32) as usize;
        let x1 = r.x1.round().clamp(0.0, width as f32) as usize;
        let y0 = r.y0.round().clamp(0.0, height as f32) as usize;
        let y1 = r.y1.round().clamp(0.0, height as f32) as usize;
        for y in y0..y1 {
            s.mask[y * width as usize + x0..y * width as usize + x1].fill(255);
        }
        s
    }

    /// The ellipse inscribed in `r` (the Elliptical Marquee). With
    /// `anti_alias`, edge pixels are partially selected by coverage
    /// (4×4 supersampling); otherwise a pixel is in when its center is.
    pub fn ellipse(width: u32, height: u32, r: Rect, anti_alias: bool) -> Self {
        let mut s = Self::empty(width, height);
        let (cx, cy) = ((r.x0 + r.x1) / 2.0, (r.y0 + r.y1) / 2.0);
        let (rx, ry) = ((r.x1 - r.x0) / 2.0, (r.y1 - r.y0) / 2.0);
        if rx <= 0.0 || ry <= 0.0 {
            return s;
        }
        let inside = |x: f32, y: f32| {
            let (dx, dy) = ((x - cx) / rx, (y - cy) / ry);
            dx * dx + dy * dy <= 1.0
        };
        let x0 = r.x0.floor().max(0.0) as u32;
        let x1 = (r.x1.ceil() as u32).min(width);
        let y0 = r.y0.floor().max(0.0) as u32;
        let y1 = (r.y1.ceil() as u32).min(height);
        const N: u32 = 4;
        for y in y0..y1 {
            for x in x0..x1 {
                let v = if anti_alias {
                    let mut hits = 0;
                    for sy in 0..N {
                        for sx in 0..N {
                            let px = x as f32 + (sx as f32 + 0.5) / N as f32;
                            let py = y as f32 + (sy as f32 + 0.5) / N as f32;
                            hits += inside(px, py) as u32;
                        }
                    }
                    (hits * 255 / (N * N)) as u8
                } else if inside(x as f32 + 0.5, y as f32 + 0.5) {
                    255
                } else {
                    0
                };
                s.mask[(y * width + x) as usize] = v;
            }
        }
        s
    }

    /// The inside of a closed polygon (the Lasso tools), with the even-odd
    /// rule for self-intersecting outlines. With `anti_alias`, edge pixels
    /// are partly selected by coverage (4 sample rows per pixel, exact
    /// horizontal overlap); otherwise a pixel is in when its center is.
    pub fn polygon(width: u32, height: u32, points: &[(f32, f32)], anti_alias: bool) -> Self {
        let mut s = Self::empty(width, height);
        if points.len() < 3 {
            return s;
        }
        let edges: Vec<((f32, f32), (f32, f32))> = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .map(|(&a, &b)| (a, b))
            .collect();
        // Spans [x0, x1) inside the polygon along the horizontal line at y
        let spans = |y: f32| -> Vec<(f32, f32)> {
            let mut xs: Vec<f32> = edges
                .iter()
                .filter(|((_, ay), (_, by))| (*ay <= y) != (*by <= y))
                .map(|((ax, ay), (bx, by))| ax + (y - ay) / (by - ay) * (bx - ax))
                .collect();
            xs.sort_by(f32::total_cmp);
            xs.as_chunks::<2>().0.iter().map(|&[a, b]| (a, b)).collect()
        };
        let w = width as usize;
        const N: u32 = 4;
        let mut coverage = vec![0f32; w];
        for y in 0..height {
            coverage.fill(0.0);
            if anti_alias {
                for sub in 0..N {
                    let sy = y as f32 + (sub as f32 + 0.5) / N as f32;
                    for (a, b) in spans(sy) {
                        let (a, b) = (a.clamp(0.0, width as f32), b.clamp(0.0, width as f32));
                        let mut x = a.floor() as usize;
                        while (x as f32) < b && x < w {
                            let overlap = b.min(x as f32 + 1.0) - a.max(x as f32);
                            coverage[x] += overlap.max(0.0) / N as f32;
                            x += 1;
                        }
                    }
                }
            } else {
                for (a, b) in spans(y as f32 + 0.5) {
                    for (x, c) in coverage.iter_mut().enumerate() {
                        let cx = x as f32 + 0.5;
                        if cx >= a && cx < b {
                            *c = 1.0;
                        }
                    }
                }
            }
            let row = &mut s.mask[y as usize * w..(y as usize + 1) * w];
            for (m, c) in row.iter_mut().zip(&coverage) {
                *m = (c.min(1.0) * 255.0).round() as u8;
            }
        }
        s
    }

    /// A selection from a 0/255 mask; with `anti_alias`, pixels on the edge
    /// of the region become half selected (smoother fills).
    pub fn from_mask(width: u32, height: u32, mask: Vec<u8>, anti_alias: bool) -> Self {
        assert_eq!(mask.len(), (width as usize) * (height as usize));
        let mut s = Self {
            width,
            height,
            mask,
        };
        if anti_alias {
            let src = s.mask.clone();
            let (w, h) = (width as i64, height as i64);
            let at = |x: i64, y: i64| {
                x >= 0 && y >= 0 && x < w && y < h && src[(y * w + x) as usize] == 255
            };
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) as usize;
                    if src[i] == 0 && (at(x - 1, y) || at(x + 1, y) || at(x, y - 1) || at(x, y + 1))
                    {
                        s.mask[i] = 128;
                    }
                }
            }
        }
        s
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Selection amount at a pixel, 0..=255 (0 outside the document).
    /// The same selection moved by (`dx`, `dy`) pixels (what moves out of
    /// the canvas is lost).
    pub fn translated(&self, dx: i64, dy: i64) -> Self {
        let mut s = Self::empty(self.width, self.height);
        for y in 0..self.height as i64 {
            for x in 0..self.width as i64 {
                let (sx, sy) = (x - dx, y - dy);
                if sx >= 0 && sy >= 0 && sx < self.width as i64 && sy < self.height as i64 {
                    s.mask[(y * self.width as i64 + x) as usize] =
                        self.mask[(sy * self.width as i64 + sx) as usize];
                }
            }
        }
        s
    }

    pub fn get(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.mask[(y * self.width + x) as usize]
    }

    pub fn is_empty(&self) -> bool {
        self.mask.iter().all(|&v| v == 0)
    }

    /// Bounding box of the selected pixels, if any (x0, y0, x1, y1; exclusive).
    pub fn bounds(&self) -> Option<(u32, u32, u32, u32)> {
        let w = self.width as usize;
        let mut b: Option<(u32, u32, u32, u32)> = None;
        for (y, row) in self.mask.chunks(w.max(1)).enumerate() {
            let Some(first) = row.iter().position(|&v| v > 0) else {
                continue;
            };
            let last = row.iter().rposition(|&v| v > 0).unwrap();
            let (x0, x1, y) = (first as u32, last as u32 + 1, y as u32);
            b = Some(match b {
                None => (x0, y, x1, y + 1),
                Some((bx0, by0, bx1, _)) => (bx0.min(x0), by0, bx1.max(x1), y + 1),
            });
        }
        b
    }

    /// Like [`crate::TiledImage::remapped`]: a `width`×`height` selection
    /// whose pixel (x, y) is this selection's pixel `source(x, y)`.
    pub fn remapped(
        &self,
        width: u32,
        height: u32,
        source: impl Fn(u32, u32) -> (u32, u32),
    ) -> Self {
        let mut mask = vec![0; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let (sx, sy) = source(x, y);
                mask[(y * width + x) as usize] = self.get(sx, sy);
            }
        }
        Self {
            width,
            height,
            mask,
        }
    }

    /// Select > Inverse.
    pub fn inverse(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            mask: self.mask.iter().map(|v| 255 - v).collect(),
        }
    }

    /// Combines `shape` into `current` (which may be no selection).
    pub fn combine(current: Option<&Selection>, shape: Selection, op: SelectionOp) -> Selection {
        let Some(cur) = current else {
            return match op {
                SelectionOp::Replace | SelectionOp::Add => shape,
                SelectionOp::Subtract | SelectionOp::Intersect => {
                    Self::empty(shape.width, shape.height)
                }
            };
        };
        let f: fn(u8, u8) -> u8 = match op {
            SelectionOp::Replace => return shape,
            SelectionOp::Add => |a, b| a.max(b),
            SelectionOp::Subtract => |a, b| ((a as u32 * (255 - b as u32) + 127) / 255) as u8,
            SelectionOp::Intersect => |a, b| a.min(b),
        };
        Self {
            width: cur.width,
            height: cur.height,
            mask: cur
                .mask
                .iter()
                .zip(&shape.mask)
                .map(|(&a, &b)| f(a, b))
                .collect(),
        }
    }

    /// Softens the edge with a blur of `radius` pixels (Feather). Uses three
    /// box-blur passes, which approximate a Gaussian.
    pub fn feather(&self, radius: f32) -> Self {
        if radius <= 0.0 {
            return self.clone();
        }
        let (w, h) = (self.width as usize, self.height as usize);
        let mut buf: Vec<f32> = self.mask.iter().map(|&v| v as f32).collect();
        // Box size for 3 passes approximating a Gaussian with sigma = radius / 2
        let sigma = radius / 2.0;
        let box_r = (((12.0 * sigma * sigma / 3.0) + 1.0).sqrt() / 2.0)
            .round()
            .max(1.0) as usize;
        let mut tmp = vec![0.0; buf.len()];
        for _ in 0..3 {
            box_blur_h(&buf, &mut tmp, w, h, box_r);
            box_blur_v(&tmp, &mut buf, w, h, box_r);
        }
        Self {
            width: self.width,
            height: self.height,
            mask: buf
                .iter()
                .map(|v| v.round().clamp(0.0, 255.0) as u8)
                .collect(),
        }
    }

    /// Euclidean distance from each pixel to the nearest pixel where `target`
    /// is true (infinite if there is none). With `edge_counts`, the area
    /// just outside the canvas counts as a target too.
    fn distance(&self, target: impl Fn(u8) -> bool, edge_counts: bool) -> Vec<f32> {
        // Pad by one pixel so the canvas edge can count as a target
        let pad = usize::from(edge_counts);
        let (w, h) = (
            self.width as usize + 2 * pad,
            self.height as usize + 2 * pad,
        );
        let mut f: Vec<f32> = vec![f32::INFINITY; w * h];
        for y in 0..h {
            for x in 0..w {
                let inside = x >= pad && y >= pad && x < w - pad && y < h - pad;
                let hit = if inside {
                    target(self.mask[(y - pad) * self.width as usize + (x - pad)])
                } else {
                    true
                };
                if hit {
                    f[y * w + x] = 0.0;
                }
            }
        }
        // Felzenszwalb-Huttenlocher squared distance transform, columns then rows
        let transform = |line: &mut [f32]| {
            let n = line.len();
            let src = line.to_vec();
            let mut v = vec![0usize; n];
            let mut z = vec![0f32; n + 1];
            let mut k = 0;
            let first = src.iter().position(|d| d.is_finite());
            let Some(first) = first else {
                return;
            };
            v[0] = first;
            z[0] = f32::NEG_INFINITY;
            z[1] = f32::INFINITY;
            for q in first + 1..n {
                if !src[q].is_finite() {
                    continue;
                }
                loop {
                    let p = v[k];
                    let s = ((src[q] + (q * q) as f32) - (src[p] + (p * p) as f32))
                        / (2.0 * (q as f32 - p as f32));
                    if s <= z[k] && k > 0 {
                        k -= 1;
                        continue;
                    }
                    if s <= z[k] {
                        v[0] = q;
                        z[1] = f32::INFINITY;
                        break;
                    }
                    k += 1;
                    v[k] = q;
                    z[k] = s;
                    z[k + 1] = f32::INFINITY;
                    break;
                }
            }
            let mut k = 0;
            for (q, out) in line.iter_mut().enumerate() {
                while z[k + 1] < q as f32 {
                    k += 1;
                }
                let p = v[k];
                let d = q as f32 - p as f32;
                *out = d * d + src[p];
            }
        };
        let mut col = vec![0f32; h];
        for x in 0..w {
            for y in 0..h {
                col[y] = f[y * w + x];
            }
            transform(&mut col);
            for y in 0..h {
                f[y * w + x] = col[y];
            }
        }
        for y in 0..h {
            transform(&mut f[y * w..(y + 1) * w]);
        }
        let (sw, sh) = (self.width as usize, self.height as usize);
        let mut out = Vec::with_capacity(sw * sh);
        for y in 0..sh {
            for x in 0..sw {
                out.push(f[(y + pad) * w + x + pad].sqrt());
            }
        }
        out
    }

    fn with_coverage(&self, coverage: impl Fn(usize) -> f32) -> Self {
        Self {
            width: self.width,
            height: self.height,
            mask: (0..self.mask.len())
                .map(|i| (coverage(i).clamp(0.0, 1.0) * 255.0).round() as u8)
                .collect(),
        }
    }

    /// Select > Modify > Expand: grows the selection by `radius` pixels
    /// (round corners), with a one-pixel soft edge.
    pub fn expand(&self, radius: f32) -> Self {
        let d = self.distance(|v| v >= 128, false);
        // Pixels up to `radius` away are fully in; the next ring partly
        self.with_coverage(|i| radius + 1.0 - d[i])
    }

    /// Select > Modify > Contract: shrinks the selection by `radius`
    /// pixels. With `at_bounds` ("Apply effect at canvas bounds") the
    /// canvas edge counts as unselected and the selection pulls away from
    /// it; otherwise the edge doesn't shrink it.
    pub fn contract(&self, radius: f32, at_bounds: bool) -> Self {
        let d = self.distance(|v| v < 128, at_bounds);
        self.with_coverage(|i| d[i] - radius)
    }

    /// Select > Modify > Border: a band `width` pixels wide centered on the
    /// selection's edge.
    pub fn border(&self, width: f32) -> Self {
        let half = width / 2.0;
        let to_out = self.distance(|v| v < 128, false);
        let to_in = self.distance(|v| v >= 128, false);
        self.with_coverage(|i| {
            let d = if self.mask[i] >= 128 {
                to_out[i]
            } else {
                to_in[i]
            };
            half + 1.0 - d
        })
    }

    /// Select > Modify > Smooth: each pixel is selected when most pixels
    /// within `radius` are, rounding off corners and removing specks.
    pub fn smooth(&self, radius: u32) -> Self {
        let (w, h) = (self.width as usize, self.height as usize);
        let buf: Vec<f32> = self
            .mask
            .iter()
            .map(|&v| if v >= 128 { 255.0 } else { 0.0 })
            .collect();
        let mut tmp = vec![0.0; buf.len()];
        let mut out = vec![0.0; buf.len()];
        box_blur_h(&buf, &mut tmp, w, h, radius as usize);
        box_blur_v(&tmp, &mut out, w, h, radius as usize);
        self.with_coverage(|i| if out[i] >= 127.5 { 1.0 } else { 0.0 })
    }

    /// The selection moved to a resized canvas (Canvas Size), placed at
    /// (`dx`, `dy`). New areas are unselected.
    pub fn with_canvas(&self, width: u32, height: u32, dx: i64, dy: i64) -> Self {
        let mut out = Self::empty(width, height);
        for y in 0..height as i64 {
            let sy = y - dy;
            if sy < 0 || sy >= self.height as i64 {
                continue;
            }
            for x in 0..width as i64 {
                let sx = x - dx;
                if sx >= 0 && sx < self.width as i64 {
                    out.mask[(y as u32 * width + x as u32) as usize] =
                        self.mask[(sy as u32 * self.width + sx as u32) as usize];
                }
            }
        }
        out
    }

    /// Outline for marching ants: horizontal and vertical unit-pixel edges
    /// between pixels at least half selected and the rest, merged into runs.
    /// Each segment is `[x0, y0, x1, y1]` in document pixels.
    pub fn outline(&self) -> Vec<[u32; 4]> {
        let (w, h) = (self.width, self.height);
        let inside = |x: i64, y: i64| {
            x >= 0
                && y >= 0
                && x < w as i64
                && y < h as i64
                && self.mask[(y as u32 * w + x as u32) as usize] >= 128
        };
        let mut segs = Vec::new();
        // Horizontal edges at y between rows y-1 and y
        for y in 0..=h as i64 {
            let mut start: Option<i64> = None;
            for x in 0..=w as i64 {
                let edge = x < w as i64 && inside(x, y - 1) != inside(x, y);
                match (edge, start) {
                    (true, None) => start = Some(x),
                    (false, Some(s)) => {
                        segs.push([s as u32, y as u32, x as u32, y as u32]);
                        start = None;
                    }
                    _ => {}
                }
            }
        }
        // Vertical edges at x between columns x-1 and x
        for x in 0..=w as i64 {
            let mut start: Option<i64> = None;
            for y in 0..=h as i64 {
                let edge = y < h as i64 && inside(x - 1, y) != inside(x, y);
                match (edge, start) {
                    (true, None) => start = Some(y),
                    (false, Some(s)) => {
                        segs.push([x as u32, s as u32, x as u32, y as u32]);
                        start = None;
                    }
                    _ => {}
                }
            }
        }
        segs
    }
}

fn box_blur_h(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    let norm = 1.0 / (2 * r + 1) as f32;
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        let at = |x: isize| row[x.clamp(0, w as isize - 1) as usize];
        let mut acc: f32 = (-(r as isize)..=r as isize).map(at).sum();
        for x in 0..w {
            dst[y * w + x] = acc * norm;
            acc += at(x as isize + r as isize + 1) - at(x as isize - r as isize);
        }
    }
}

fn box_blur_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    let norm = 1.0 / (2 * r + 1) as f32;
    for x in 0..w {
        let at = |y: isize| src[y.clamp(0, h as isize - 1) as usize * w + x];
        let mut acc: f32 = (-(r as isize)..=r as isize).map(at).sum();
        for y in 0..h {
            dst[y * w + x] = acc * norm;
            acc += at(y as isize + r as isize + 1) - at(y as isize - r as isize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_and_bounds() {
        let s = Selection::rect(10, 10, Rect::new(2.2, 3.0, 5.0, 8.6));
        assert_eq!(s.bounds(), Some((2, 3, 5, 9)));
        assert_eq!(s.get(2, 3), 255);
        assert_eq!(s.get(5, 3), 0);
        // Partly outside the document is clipped
        let s = Selection::rect(10, 10, Rect::new(-5.0, -5.0, 3.0, 3.0));
        assert_eq!(s.bounds(), Some((0, 0, 3, 3)));
    }

    #[test]
    fn combine_ops() {
        let a = Selection::rect(10, 1, Rect::new(0.0, 0.0, 6.0, 1.0));
        let b = Selection::rect(10, 1, Rect::new(4.0, 0.0, 10.0, 1.0));
        let add = Selection::combine(Some(&a), b.clone(), SelectionOp::Add);
        assert_eq!(add.bounds(), Some((0, 0, 10, 1)));
        let sub = Selection::combine(Some(&a), b.clone(), SelectionOp::Subtract);
        assert_eq!(sub.bounds(), Some((0, 0, 4, 1)));
        let int = Selection::combine(Some(&a), b.clone(), SelectionOp::Intersect);
        assert_eq!(int.bounds(), Some((4, 0, 6, 1)));
        let rep = Selection::combine(Some(&a), b.clone(), SelectionOp::Replace);
        assert_eq!(rep, b);
        assert!(Selection::combine(None, b, SelectionOp::Subtract).is_empty());
    }

    #[test]
    fn inverse_and_all() {
        let a = Selection::rect(4, 4, Rect::new(0.0, 0.0, 2.0, 4.0));
        let inv = a.inverse();
        assert_eq!(inv.bounds(), Some((2, 0, 4, 4)));
        assert_eq!(Selection::all(3, 2).bounds(), Some((0, 0, 3, 2)));
        assert!(Selection::all(3, 2).inverse().is_empty());
    }

    #[test]
    fn ellipse_anti_aliasing() {
        let r = Rect::new(0.0, 0.0, 20.0, 10.0);
        let hard = Selection::ellipse(20, 10, r, false);
        let soft = Selection::ellipse(20, 10, r, true);
        assert_eq!(hard.get(10, 5), 255);
        assert_eq!(hard.get(0, 0), 0);
        assert!(hard.mask.iter().all(|&v| v == 0 || v == 255));
        assert!(soft.mask.iter().any(|&v| v > 0 && v < 255));
    }

    #[test]
    fn feather_softens_edges() {
        let s = Selection::rect(40, 1, Rect::new(10.0, 0.0, 30.0, 1.0)).feather(4.0);
        assert!(s.get(20, 0) > 240);
        let edge = s.get(10, 0);
        assert!(edge > 60 && edge < 200, "{edge}");
        assert!(s.get(2, 0) < 10);
    }

    #[test]
    fn outline_of_rect() {
        let s = Selection::rect(10, 10, Rect::new(2.0, 3.0, 5.0, 8.0));
        let mut segs = s.outline();
        segs.sort();
        assert_eq!(
            segs,
            vec![[2, 3, 2, 8], [2, 3, 5, 3], [2, 8, 5, 8], [5, 3, 5, 8]]
        );
    }

    #[test]
    fn modify_expand_contract_border_smooth() {
        // A 4×4 square in the middle of 12×12
        let s = Selection::rect(12, 12, Rect::new(4.0, 4.0, 8.0, 8.0));
        let e = s.expand(2.0);
        assert_eq!(e.bounds(), Some((2, 2, 10, 10)));
        assert_eq!(e.get(2, 6), 255);
        // Round corners: the far corner pixel is not fully selected
        assert!(e.get(2, 2) < 255);
        let c = s.contract(1.0, false);
        assert_eq!(c.bounds(), Some((5, 5, 7, 7)));
        // At the canvas edge, contract only pulls away with at_bounds
        let all = Selection::all(6, 6);
        assert_eq!(all.contract(2.0, false).get(0, 0), 255);
        assert_eq!(all.contract(2.0, true).get(0, 0), 0);
        assert_eq!(all.contract(2.0, true).get(2, 2), 255);
        // A 2-pixel border straddles the edge
        let b = s.border(2.0);
        assert_eq!((b.get(3, 6), b.get(4, 6), b.get(6, 6)), (255, 255, 0));
        // Smoothing removes a lone pixel
        let mut speck = Selection::rect(12, 12, Rect::new(4.0, 4.0, 8.0, 8.0));
        speck.mask[0] = 255;
        let sm = speck.smooth(1);
        assert_eq!(sm.get(0, 0), 0);
        assert_eq!(sm.get(6, 6), 255);
    }

    #[test]
    fn polygons_fill_their_inside() {
        // A right triangle with legs of 4: 8 pixels' worth of area
        let tri = [(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)];
        let s = Selection::polygon(4, 4, &tri, true);
        let total: u32 = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .map(|(x, y)| s.get(x, y) as u32)
            .sum();
        assert!((total as f32 / 255.0 - 8.0).abs() < 0.1, "{total}");
        assert_eq!(s.get(0, 0), 255);
        assert_eq!(s.get(3, 3), 0);
        // Without anti-aliasing every pixel is all in or all out
        let hard = Selection::polygon(4, 4, &tri, false);
        assert!((0..4).all(|y| (0..4).all(|x| matches!(hard.get(x, y), 0 | 255))));
        assert_eq!(hard.get(1, 1), 255);
        assert_eq!(hard.get(2, 2), 0);
        // Fewer than three points select nothing
        assert!(Selection::polygon(4, 4, &[(0.0, 0.0), (3.0, 3.0)], true).is_empty());
    }

    #[test]
    fn with_canvas_shifts() {
        let s = Selection::rect(4, 4, Rect::new(0.0, 0.0, 2.0, 2.0));
        let moved = s.with_canvas(6, 6, 1, 1);
        assert_eq!(moved.bounds(), Some((1, 1, 3, 3)));
    }
}
