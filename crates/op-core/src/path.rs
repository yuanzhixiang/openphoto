//! Editable cubic Bezier paths in document coordinates, independent of pixels.
pub type Point = (f32, f32);
#[derive(Clone, Debug, PartialEq)]
pub struct Knot {
    pub incoming: Point,
    pub anchor: Point,
    pub outgoing: Point,
    pub linked: bool,
}
impl Knot {
    pub fn corner(anchor: Point) -> Self {
        Self {
            incoming: anchor,
            anchor,
            outgoing: anchor,
            linked: false,
        }
    }
    pub fn translate(&mut self, delta: Point) {
        for p in [&mut self.incoming, &mut self.anchor, &mut self.outgoing] {
            p.0 += delta.0;
            p.1 += delta.1;
        }
    }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Subpath {
    pub knots: Vec<Knot>,
    pub closed: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub name: String,
    pub subpaths: Vec<Subpath>,
}
fn lerp(a: Point, b: Point, t: f32) -> Point {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}
pub fn cubic(a: &Knot, b: &Knot, t: f32) -> Point {
    let p = lerp(a.anchor, a.outgoing, t);
    let q = lerp(a.outgoing, b.incoming, t);
    let r = lerp(b.incoming, b.anchor, t);
    lerp(lerp(p, q, t), lerp(q, r, t), t)
}
impl Subpath {
    pub fn segments(&self) -> usize {
        if self.closed {
            self.knots.len()
        } else {
            self.knots.len().saturating_sub(1)
        }
    }
    /// Exact de Casteljau split: inserting an anchor preserves both halves.
    pub fn insert(&mut self, segment: usize, t: f32) -> Option<usize> {
        if segment >= self.segments() || !t.is_finite() || !(0.0..1.0).contains(&t) || t == 0.0 {
            return None;
        }
        let next = (segment + 1) % self.knots.len();
        let a = &self.knots[segment];
        let b = &self.knots[next];
        let p = lerp(a.anchor, a.outgoing, t);
        let q = lerp(a.outgoing, b.incoming, t);
        let r = lerp(b.incoming, b.anchor, t);
        let s = lerp(p, q, t);
        let u = lerp(q, r, t);
        let at = lerp(s, u, t);
        self.knots[segment].outgoing = p;
        self.knots[next].incoming = r;
        let index = segment + 1;
        self.knots.insert(
            index,
            Knot {
                incoming: s,
                anchor: at,
                outgoing: u,
                linked: true,
            },
        );
        Some(index)
    }
    pub fn flatten(&self) -> Vec<Point> {
        let Some(first) = self.knots.first() else {
            return vec![];
        };
        let mut out = vec![first.anchor];
        for i in 0..self.segments() {
            let a = &self.knots[i];
            let b = &self.knots[(i + 1) % self.knots.len()];
            flatten_curve([a.anchor, a.outgoing, b.incoming, b.anchor], 0, &mut out);
        }
        out
    }
}
fn flatten_curve(p: [Point; 4], depth: u8, out: &mut Vec<Point>) {
    let ux = 3. * p[1].0 - 2. * p[0].0 - p[3].0;
    let uy = 3. * p[1].1 - 2. * p[0].1 - p[3].1;
    let vx = 3. * p[2].0 - 2. * p[3].0 - p[0].0;
    let vy = 3. * p[2].1 - 2. * p[3].1 - p[0].1;
    if depth >= 12 || ux * ux + uy * uy + vx * vx + vy * vy <= 0.25 {
        out.push(p[3]);
        return;
    }
    let a = lerp(p[0], p[1], 0.5);
    let b = lerp(p[1], p[2], 0.5);
    let c = lerp(p[2], p[3], 0.5);
    let d = lerp(a, b, 0.5);
    let e = lerp(b, c, 0.5);
    let m = lerp(d, e, 0.5);
    flatten_curve([p[0], a, d, m], depth + 1, out);
    flatten_curve([m, e, c, p[3]], depth + 1, out);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn splitting_preserves_cubic_geometry() {
        let a = Knot {
            outgoing: (10., 70.),
            ..Knot::corner((0., 0.))
        };
        let b = Knot {
            incoming: (90., -30.),
            ..Knot::corner((100., 50.))
        };
        let mut path = Subpath {
            knots: vec![a.clone(), b.clone()],
            closed: false,
        };
        assert_eq!(path.insert(0, 0.3), Some(1));
        for i in 0..=100 {
            let t = i as f32 / 100.;
            let old = cubic(&a, &b, t);
            let new = if t < 0.3 {
                cubic(&path.knots[0], &path.knots[1], t / 0.3)
            } else {
                cubic(&path.knots[1], &path.knots[2], (t - 0.3) / 0.7)
            };
            assert!((old.0 - new.0).abs() < 0.0001 && (old.1 - new.1).abs() < 0.0001);
        }
        assert!(path.flatten().len() > 3);
        assert_eq!(path.insert(5, 0.5), None);
    }
}

#[cfg(test)]
mod document_tests {
    use super::*;
    #[test]
    fn paths_follow_canvas_resize_rotate_crop_and_history() {
        let mut d = crate::Document::new_with_background("Paths", 100, 80, crate::Color::WHITE);
        d.paths.push(Path {
            name: "Path 1".into(),
            subpaths: vec![Subpath {
                knots: vec![Knot::corner((20., 30.))],
                closed: false,
            }],
        });
        let snap = d.snapshot();
        d.resize_canvas(120, 100, crate::Anchor::CENTER, crate::Color::WHITE);
        assert_eq!(d.paths[0].subpaths[0].knots[0].anchor, (30., 40.));
        crate::image_ops::reorient(&mut d, crate::image_ops::Orientation::Rotate90Clockwise);
        assert_eq!(d.paths[0].subpaths[0].knots[0].anchor, (60., 30.));
        crate::image_ops::crop(&mut d, 10, 10, 90, 100);
        assert_eq!(d.paths[0].subpaths[0].knots[0].anchor, (50., 20.));
        crate::image_ops::resize(
            &mut d,
            160,
            180,
            crate::image_ops::Resample::NearestNeighbor,
        );
        assert_eq!(d.paths[0].subpaths[0].knots[0].anchor, (100., 40.));
        d.restore(&snap);
        assert_eq!(d.paths[0].subpaths[0].knots[0].anchor, (20., 30.));
    }
}

/// An interpolating cubic spline through sampled anchors. Corner flags keep
/// both controls at the anchor; smooth knots share a centered tangent.
pub fn interpolating_subpath(points: &[(Point, bool)], closed: bool) -> Subpath {
    let mut knots = Vec::with_capacity(points.len());
    for (i, &(p, corner)) in points.iter().enumerate() {
        let mut k = Knot::corner(p);
        if !corner && points.len() > 1 {
            let prev = if i > 0 {
                points[i - 1].0
            } else if closed {
                points[points.len() - 1].0
            } else {
                p
            };
            let next = if i + 1 < points.len() {
                points[i + 1].0
            } else if closed {
                points[0].0
            } else {
                p
            };
            let tangent = ((next.0 - prev.0) / 6., (next.1 - prev.1) / 6.);
            k.incoming = (p.0 - tangent.0, p.1 - tangent.1);
            k.outgoing = (p.0 + tangent.0, p.1 + tangent.1);
            k.linked = true;
        }
        knots.push(k);
    }
    Subpath { knots, closed }
}
#[cfg(test)]
mod interpolation_tests {
    use super::*;
    #[test]
    fn samples_interpolate_with_continuous_tangents_and_explicit_corners() {
        let p = interpolating_subpath(
            &[((0., 0.), false), ((30., 40.), false), ((60., 0.), true)],
            false,
        );
        assert_eq!(p.knots[1].anchor, (30., 40.));
        assert_eq!(p.knots[2].incoming, p.knots[2].anchor);
        assert_eq!(cubic(&p.knots[0], &p.knots[1], 1.), p.knots[1].anchor);
        assert_eq!(cubic(&p.knots[1], &p.knots[2], 0.), p.knots[1].anchor);
        assert_eq!(
            (
                p.knots[1].anchor.0 - p.knots[1].incoming.0,
                p.knots[1].anchor.1 - p.knots[1].incoming.1
            ),
            (
                p.knots[1].outgoing.0 - p.knots[1].anchor.0,
                p.knots[1].outgoing.1 - p.knots[1].anchor.1
            )
        );
    }
}
