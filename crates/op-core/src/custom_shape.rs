//! Custom geometry comes from named Bezier paths, preserving their contours.
use crate::{
    Document, Layer, TiledImage,
    path::{Path, Point},
    selection::Selection,
};
fn extrema(a: f32, b: f32, c: f32, d: f32) -> Vec<f32> {
    let aa = -a + 3. * b - 3. * c + d;
    let bb = 2. * (a - 2. * b + c);
    let cc = b - a;
    if aa.abs() < 1e-8 {
        if bb.abs() < 1e-8 {
            vec![]
        } else {
            vec![-cc / bb]
        }
    } else {
        let disc = bb * bb - 4. * aa * cc;
        if disc < 0. {
            vec![]
        } else {
            vec![
                (-bb + disc.sqrt()) / (2. * aa),
                (-bb - disc.sqrt()) / (2. * aa),
            ]
        }
    }
}
pub fn bounds(path: &Path) -> Option<(Point, Point)> {
    let mut points = vec![];
    for sub in &path.subpaths {
        points.extend(sub.knots.iter().map(|k| k.anchor));
        for i in 0..sub.segments() {
            let a = &sub.knots[i];
            let b = &sub.knots[(i + 1) % sub.knots.len()];
            let ts = extrema(a.anchor.0, a.outgoing.0, b.incoming.0, b.anchor.0)
                .into_iter()
                .chain(extrema(a.anchor.1, a.outgoing.1, b.incoming.1, b.anchor.1));
            points.extend(
                ts.filter(|t| *t > 0. && *t < 1.)
                    .map(|t| crate::path::cubic(a, b, t)),
            );
        }
    }
    let first = *points.first()?;
    let mut min = first;
    let mut max = first;
    for p in points {
        if !p.0.is_finite() || !p.1.is_finite() {
            return None;
        }
        min = (min.0.min(p.0), min.1.min(p.1));
        max = (max.0.max(p.0), max.1.max(p.1));
    }
    Some((min, max))
}
pub fn placed(path: &Path, a: Point, b: Point) -> Option<Path> {
    let (min, max) = bounds(path)?;
    let (w, h) = (max.0 - min.0, max.1 - min.1);
    if w <= 0. || h <= 0. || a.0 == b.0 || a.1 == b.1 {
        return None;
    }
    let lo = (a.0.min(b.0), a.1.min(b.1));
    let hi = (a.0.max(b.0), a.1.max(b.1));
    let map = |p: Point| {
        (
            lo.0 + (p.0 - min.0) * (hi.0 - lo.0) / w,
            lo.1 + (p.1 - min.1) * (hi.1 - lo.1) / h,
        )
    };
    let mut result = path.clone();
    for sub in &mut result.subpaths {
        for k in &mut sub.knots {
            k.incoming = map(k.incoming);
            k.anchor = map(k.anchor);
            k.outgoing = map(k.outgoing);
        }
    }
    Some(result)
}
/// Even/odd filling across all contours; open contours close for filling.
pub fn coverage(path: &Path, w: u32, h: u32) -> Option<Selection> {
    if (w as u64) * (h as u64) > 16_777_216 {
        return None;
    }
    let (lo, hi) = bounds(path)?;
    let contours = path
        .subpaths
        .iter()
        .map(|s| s.flatten())
        .filter(|p| p.len() > 2)
        .collect::<Vec<_>>();
    let mut values = vec![0; (w as usize) * (h as usize)];
    for y in (lo.1.floor().max(0.) as u32).min(h)..(hi.1.ceil().max(0.) as u32).min(h) {
        for x in (lo.0.floor().max(0.) as u32).min(w)..(hi.0.ceil().max(0.) as u32).min(w) {
            let mut covered = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let (px, py) = (
                        x as f32 + (sx as f32 + 0.5) / 4.,
                        y as f32 + (sy as f32 + 0.5) / 4.,
                    );
                    let mut inside = false;
                    for points in &contours {
                        for i in 0..points.len() {
                            let a = points[i];
                            let b = points[(i + 1) % points.len()];
                            if (a.1 > py) != (b.1 > py)
                                && px < (b.0 - a.0) * (py - a.1) / (b.1 - a.1) + a.0
                            {
                                inside = !inside;
                            }
                        }
                    }
                    covered += u32::from(inside);
                }
            }
            values[(y * w + x) as usize] = ((covered * 255 + 8) / 16) as u8;
        }
    }
    Some(Selection::from_mask(w, h, values, false))
}
pub fn add_layer(
    doc: &mut Document,
    path: &Path,
    a: Point,
    b: Point,
    color: [u8; 3],
) -> Option<crate::LayerId> {
    let shape = placed(path, a, b)?;
    let coverage = coverage(&shape, doc.width, doc.height)?;
    if coverage.is_empty() {
        return None;
    }
    let mut image = TiledImage::new(doc.width, doc.height);
    let (x0, y0, x1, y1) = coverage.bounds()?;
    for y in y0..y1 {
        for x in x0..x1 {
            let alpha = coverage.get(x, y);
            if alpha > 0 {
                image.set_pixel(x, y, [color[0], color[1], color[2], alpha]);
            }
        }
    }
    let id = doc.new_layer_id();
    doc.insert_above_active(Layer::raster(id, format!("{} shape", path.name), image));
    Some(id)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::{Knot, Subpath};
    #[test]
    fn custom_shapes_preserve_even_odd_holes_and_exact_curve_extents() {
        let rect = |lo: f32, hi: f32| Subpath {
            closed: true,
            knots: vec![
                Knot::corner((lo, lo)),
                Knot::corner((hi, lo)),
                Knot::corner((hi, hi)),
                Knot::corner((lo, hi)),
            ],
        };
        let shape = Path {
            name: "Donut".into(),
            subpaths: vec![rect(0., 10.), rect(3., 7.)],
        };
        let mapped = placed(&shape, (10., 20.), (110., 70.)).unwrap();
        let mask = coverage(&mapped, 128, 96).unwrap();
        assert_eq!(mask.get(20, 25), 255);
        assert_eq!(mask.get(60, 45), 0);
        assert_eq!(mask.get(5, 5), 0);
        let curve = Path {
            name: "Curve".into(),
            subpaths: vec![Subpath {
                closed: false,
                knots: vec![
                    Knot {
                        outgoing: (0., 100.),
                        ..Knot::corner((0., 0.))
                    },
                    Knot {
                        incoming: (100., 100.),
                        ..Knot::corner((100., 0.))
                    },
                ],
            }],
        };
        assert_eq!(bounds(&curve), Some(((0., 0.), (100., 75.))));
    }
}
