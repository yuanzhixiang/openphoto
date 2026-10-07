//! The toolbar's tool icons, drawn as vector shapes in Photoshop 2026's
//! style: 2 px (1 pt) strokes and solid silhouettes, sized and placed as
//! Photoshop's (measured on its toolbar at 2x). Our own drawings, not
//! Adobe's icon assets.
//!
//! Coordinates are device pixels at 2x in a 56 × 52 cell whose center
//! (28, 26) is the tool button's center; `x` runs right and `y` down.

use egui::{Color32, Painter, Pos2, Shape, Stroke, vec2};
use op_tools::Tool;

use crate::theme::pt;

/// Draws in one icon cell.
struct Pen<'a> {
    painter: &'a Painter,
    /// The cell's top-left corner on screen.
    origin: Pos2,
    color: Color32,
    /// The button's background, for holes cut into shapes.
    bg: Color32,
    /// Drawing scale (1 in the toolbar).
    scale: f32,
}

impl Pen<'_> {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(pt(x / 2.0), pt(y / 2.0)) * self.scale
    }

    /// A filled rectangle from (x0, y0) to (x1, y1).
    fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.painter.rect_filled(
            egui::Rect::from_min_max(self.p(x0, y0), self.p(x1, y1)),
            0,
            self.color,
        );
    }

    /// Several filled rectangles.
    fn rects(&self, rs: &[[f32; 4]]) {
        for r in rs {
            self.rect(r[0], r[1], r[2], r[3]);
        }
    }

    /// A filled convex polygon.
    fn poly(&self, pts: &[(f32, f32)]) {
        let pts = pts.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.painter
            .add(Shape::convex_polygon(pts, self.color, Stroke::NONE));
    }

    /// A polygon's outline, `w` px wide.
    fn outline(&self, pts: &[(f32, f32)], w: f32) {
        let pts = pts.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.painter.add(Shape::closed_line(pts, self.stroke(w)));
    }

    /// A line through `pts`, `w` px wide.
    fn line(&self, pts: &[(f32, f32)], w: f32) {
        let pts = pts.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.painter.add(Shape::line(pts, self.stroke(w)));
    }

    /// The same pen in another color.
    fn with(&self, color: Color32) -> Pen<'_> {
        Pen {
            painter: self.painter,
            origin: self.origin,
            color,
            bg: self.bg,
            scale: self.scale,
        }
    }

    /// The pen that cuts holes (paints the background).
    fn hole(&self) -> Pen<'_> {
        self.with(self.bg)
    }

    /// A filled triangle fan around `center` through `pts` (a star-shaped
    /// polygon, which `poly` can't fill when concave).
    fn fan(&self, center: (f32, f32), pts: &[(f32, f32)]) {
        // One mesh, so the triangles' shared edges don't show; a thin line
        // around it smooths the outer edge
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(self.p(center.0, center.1), self.color);
        for &(x, y) in pts {
            mesh.colored_vertex(self.p(x, y), self.color);
        }
        let n = pts.len() as u32;
        for i in 0..n {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        self.painter.add(Shape::mesh(mesh));
        self.outline(pts, 0.8);
    }

    /// A rectangle filled left to right from `left` to `right`.
    fn gradient(&self, x0: f32, y0: f32, x1: f32, y1: f32, left: Color32, right: Color32) {
        let mut mesh = egui::Mesh::default();
        let (a, b, c, d) = (
            self.p(x0, y0),
            self.p(x1, y0),
            self.p(x1, y1),
            self.p(x0, y1),
        );
        mesh.colored_vertex(a, left);
        mesh.colored_vertex(b, right);
        mesh.colored_vertex(c, right);
        mesh.colored_vertex(d, left);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        self.painter.add(Shape::mesh(mesh));
    }

    /// A 4-point sparkle centered at (cx, cy), `h` px from the center to
    /// each tip.
    fn sparkle(&self, cx: f32, cy: f32, h: f32) {
        self.poly(&[(cx, cy - h), (cx + 1.4, cy), (cx, cy + h), (cx - 1.4, cy)]);
        self.poly(&[(cx - h, cy), (cx, cy - 1.4), (cx + h, cy), (cx, cy + 1.4)]);
    }

    /// A round-ended bar from `a` to `b`, `w` px wide.
    fn bar(&self, a: (f32, f32), b: (f32, f32), w: f32) {
        self.line(&[a, b], w);
        self.disc(a.0, a.1, w / 2.0, w / 2.0);
        self.disc(b.0, b.1, w / 2.0, w / 2.0);
    }

    fn stroke(&self, w: f32) -> Stroke {
        Stroke::new(pt(w / 2.0) * self.scale, self.color)
    }

    /// A filled ellipse.
    fn disc(&self, cx: f32, cy: f32, rx: f32, ry: f32) {
        self.poly(&ellipse(cx, cy, rx, ry, 0.0, std::f32::consts::TAU));
    }

    /// An ellipse's outline, `w` px wide, centered on the radii.
    fn ring(&self, cx: f32, cy: f32, rx: f32, ry: f32, w: f32) {
        let mut pts = ellipse(cx, cy, rx, ry, 0.0, std::f32::consts::TAU);
        pts.pop();
        self.outline(&pts, w);
    }

    /// An arc of an ellipse from angle `a0` to `a1` (radians, clockwise
    /// from 3 o'clock on screen), `w` px wide.
    fn arc(&self, (cx, cy): (f32, f32), (rx, ry): (f32, f32), (a0, a1): (f32, f32), w: f32) {
        self.line(&ellipse(cx, cy, rx, ry, a0, a1), w);
    }

    /// A dashed ellipse: `n` dashes centered at angles `k · 2π / n` from
    /// 3 o'clock, each `span` radians long.
    fn dashed_ring(&self, c: (f32, f32), r: (f32, f32), n: usize, span: f32, w: f32) {
        for k in 0..n {
            let a = k as f32 * std::f32::consts::TAU / n as f32;
            self.arc(c, r, (a - span / 2.0, a + span / 2.0), w);
        }
    }
}

/// Points along an ellipse from `a0` to `a1`.
fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
    let n = ((a1 - a0).abs() * (rx.max(ry)) / 1.5).ceil().max(4.0) as usize;
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            (cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect()
}

/// Photoshop's icon gray (`#dddddd`) and the gray inside shape icons.
pub const COLOR: Color32 = Color32::from_gray(0xdd);
const FILL: Color32 = Color32::from_gray(0x6f);

/// Paints `tool`'s icon in the cell centered on `center` over the button's
/// background `bg`; false when it has no drawing (the caller falls back to
/// the icon font).
pub fn paint(painter: &Painter, center: Pos2, tool: Tool, color: Color32, bg: Color32) -> bool {
    // Photoshop's cell, measured: its top 25 device pixels above the
    // button's center rounded down, one lower on the rows whose center
    // falls on a whole pixel (every fifth: the 25.9 pt pitch repeats every
    // 5 rows), and its left 29 pixels left of the center
    let ppp = painter.ctx().pixels_per_point();
    let (cx, cy) = (center.x * ppp, center.y * ppp);
    let whole = (cy - cy.round()).abs() < 0.05;
    let top = cy.floor() - 25.0 + if whole { 1.0 } else { 0.0 };
    let (nx, ny) = nudge(tool);
    let o = Pos2::new(((cx - 29.0).round() + nx) / ppp, (top + ny) / ppp);
    let pen = Pen {
        painter,
        origin: o,
        color,
        bg,
        scale: 1.0,
    };
    draw(&pen, tool)
}

/// Paints `tool`'s icon at `scale`, its cell centered on `center` (the
/// tool flyout's smaller icons); false when it has no drawing.
pub fn paint_scaled(
    painter: &Painter,
    center: Pos2,
    tool: Tool,
    color: Color32,
    bg: Color32,
    scale: f32,
) -> bool {
    let (nx, ny) = nudge(tool);
    let pen = Pen {
        painter,
        origin: center - vec2(pt((29.0 - nx) / 2.0), pt((25.0 - ny) / 2.0)) * scale,
        color,
        bg,
        scale,
    };
    draw(&pen, tool)
}

/// Device pixels to move a drawing by, where its coordinates came out off
/// Photoshop's (found by overlapping each icon with Photoshop's).
fn nudge(tool: Tool) -> (f32, f32) {
    use Tool::*;
    match tool {
        PolygonalLasso | MagicWand | Eyedropper | SpotHealingBrush | HealingBrush | Pencil
        | MagicEraser | PaintBucket | Blur | Sharpen | Smudge | Pen | FreeformPen
        | CurvaturePen | AddAnchorPoint | DeleteAnchorPoint | Polygon => (1.0, 0.0),
        Lasso => (1.0, 1.0),
        Remove => (2.0, 2.0),
        SelectionBrush | ArtHistoryBrush | Burn | Sponge => (0.0, 1.0),
        Slice => (1.0, 0.0),
        BackgroundEraser => (2.0, 0.0),
        Dodge => (-1.0, 0.0),
        DirectSelection => (0.0, -1.0),
        _ => (0.0, 0.0),
    }
}

/// A paint brush: the handle from `top` toward the head, and the head's
/// outline (its points relative to the brush's offset `(dx, dy)`).
fn brush(d: &Pen, dx: f32, dy: f32) {
    let m = |x: f32, y: f32| (x + dx, y + dy);
    d.poly(&[
        m(40.0, 10.0),
        m(42.5, 11.5),
        m(31.5, 26.5),
        m(28.5, 28.5),
        m(25.5, 26.5),
        m(26.5, 24.0),
    ]);
    d.poly(&[m(28.5, 28.0), m(30.0, 27.0), m(30.0, 29.0), m(29.0, 30.0)]);
    d.poly(&[
        m(21.0, 28.5),
        m(25.0, 29.5),
        m(27.0, 32.0),
        m(27.0, 34.5),
        m(26.0, 37.0),
        m(23.0, 39.0),
        m(18.5, 40.5),
        m(12.0, 41.0),
        m(13.5, 39.0),
        m(14.0, 35.5),
        m(15.0, 32.0),
        m(17.0, 29.5),
    ]);
}

/// A pen nib, offset by (dx, dy) from the Pen tool's.
fn nib(d: &Pen, dx: f32, dy: f32) {
    let m = |x: f32, y: f32| (x + dx, y + dy);
    d.bar(m(35.5, 16.0), m(40.5, 21.5), 7.0);
    d.outline(
        &[
            m(32.0, 17.5),
            m(19.0, 23.5),
            m(16.0, 33.0),
            m(14.5, 41.5),
            m(23.0, 39.0),
            m(33.0, 36.0),
            m(38.5, 25.5),
        ],
        2.6,
    );
    d.line(&[m(15.5, 40.5), m(25.0, 31.0)], 2.4);
    d.disc(m(26.6, 29.2).0, m(26.6, 29.2).1, 2.7, 2.7);
}

/// An eraser: its block (the upper part solid, the lower part outlined),
/// offset by (dx, dy), cut off at `bottom`.
fn eraser(d: &Pen, dx: f32, dy: f32, bottom: f32) {
    let m = |x: f32, y: f32| (x + dx, y + dy);
    let (a, b) = (m(31.5, 12.5), m(42.5, 23.0));
    let (p, q) = (m(20.5, 24.5), m(31.5, 35.0));
    d.poly(&[a, b, q, p]);
    let cut = |x0: f32, y0: f32, ddx: f32, ddy: f32| {
        // where the side from (x0, y0) along (ddx, ddy) meets the bottom
        let t = (bottom - y0) / ddy;
        (x0 + ddx * t, bottom)
    };
    let side = (-19.5, 20.5);
    let low_right = cut(b.0, b.1, side.0, side.1);
    let low_left = cut(m(12.0, 33.0).0, m(12.0, 33.0).1, 10.5, 10.5);
    d.line(&[p, m(12.0, 33.0), low_left], 2.6);
    d.line(&[q, low_right], 2.6);
}

/// The healing brushes' bandage.
fn bandage(d: &Pen) {
    d.bar((18.0, 34.0), (37.0, 15.0), 13.0);
    let h = d.hole();
    for (x, y) in [(27.5, 20.5), (31.5, 24.5), (23.5, 24.5), (27.5, 28.5)] {
        h.disc(x, y, 2.1, 2.1);
    }
}

/// A rubber stamp, its base's top at y 30.
fn stamp(d: &Pen) {
    d.disc(29.0, 16.5, 6.2, 5.8);
    d.rect(27.0, 21.0, 31.0, 29.0);
    d.poly(&[(25.0, 28.5), (33.0, 28.5), (39.0, 30.0), (19.0, 30.0)]);
    d.rect(16.0, 30.0, 42.0, 37.0);
    d.rect(18.0, 39.0, 40.0, 41.0);
}

fn draw(d: &Pen, tool: Tool) -> bool {
    use Tool::*;
    let fill = d.with(FILL);
    match tool {
        Move => {
            d.rect(26.0, 14.0, 28.0, 34.0);
            d.rect(13.0, 23.0, 41.0, 25.0);
            d.poly(&[(27.0, 8.8), (32.2, 15.0), (21.8, 15.0)]);
            d.poly(&[(21.8, 33.0), (32.2, 33.0), (27.0, 39.2)]);
            d.poly(&[(11.8, 24.0), (18.0, 18.8), (18.0, 29.2)]);
            d.poly(&[(42.2, 24.0), (36.0, 29.2), (36.0, 18.8)]);
        }
        Artboard => {
            d.rects(&[
                [16.0, 5.0, 18.0, 11.0],
                [8.0, 13.0, 14.0, 15.0],
                [16.0, 13.0, 18.0, 37.0],
                [16.0, 13.0, 35.0, 15.0],
                [40.0, 21.0, 42.0, 37.0],
                [16.0, 35.0, 42.0, 37.0],
            ]);
            d.poly(&[
                (34.0, 13.0),
                (37.0, 13.0),
                (42.0, 18.0),
                (42.0, 21.0),
                (34.0, 21.0),
            ]);
        }
        RectangularMarquee => d.rects(&[
            [12.0, 10.0, 20.0, 12.0],
            [24.0, 10.0, 32.0, 12.0],
            [36.0, 10.0, 44.0, 12.0],
            [12.0, 34.0, 20.0, 36.0],
            [24.0, 34.0, 32.0, 36.0],
            [36.0, 34.0, 44.0, 36.0],
            [12.0, 12.0, 14.0, 16.0],
            [12.0, 20.0, 14.0, 26.0],
            [12.0, 30.0, 14.0, 34.0],
            [42.0, 12.0, 44.0, 16.0],
            [42.0, 20.0, 44.0, 26.0],
            [42.0, 30.0, 44.0, 34.0],
        ]),
        EllipticalMarquee => {
            d.dashed_ring((28.0, 24.0), (15.0, 13.0), 8, 0.6, 2.0);
        }
        SingleRowMarquee => d.rects(&[
            [12.0, 22.0, 20.0, 24.0],
            [24.0, 22.0, 34.0, 24.0],
            [38.0, 22.0, 44.0, 24.0],
            [12.0, 24.0, 14.0, 26.0],
            [42.0, 24.0, 44.0, 26.0],
            [12.0, 26.0, 18.0, 28.0],
            [22.0, 26.0, 32.0, 28.0],
            [36.0, 26.0, 44.0, 28.0],
        ]),
        SingleColumnMarquee => d.rects(&[
            [24.0, 8.0, 30.0, 10.0],
            [24.0, 10.0, 26.0, 14.0],
            [28.0, 10.0, 30.0, 16.0],
            [24.0, 18.0, 26.0, 26.0],
            [28.0, 20.0, 30.0, 28.0],
            [24.0, 30.0, 26.0, 36.0],
            [28.0, 32.0, 30.0, 36.0],
            [24.0, 36.0, 30.0, 38.0],
        ]),
        Crop => d.rects(&[
            [16.0, 6.5, 20.0, 34.0],
            [16.0, 30.0, 32.0, 34.0],
            [8.0, 12.0, 14.0, 16.0],
            [22.0, 12.0, 38.0, 16.0],
            [34.0, 12.0, 38.0, 39.5],
            [40.0, 30.0, 46.0, 34.0],
        ]),
        Frame => {
            d.rects(&[
                [12.0, 10.0, 16.0, 12.0],
                [42.0, 10.0, 46.0, 12.0],
                [12.0, 12.0, 46.0, 14.0],
                [14.0, 14.0, 16.0, 34.0],
                [42.0, 14.0, 44.0, 34.0],
                [12.0, 34.0, 46.0, 36.0],
                [12.0, 36.0, 16.0, 38.0],
                [42.0, 36.0, 46.0, 38.0],
            ]);
            d.line(&[(16.7, 14.0), (41.3, 34.0)], 2.6);
            d.line(&[(41.3, 14.0), (16.7, 34.0)], 2.6);
        }
        Lasso => {
            d.ring(30.0, 19.5, 14.8, 9.2, 2.5);
            d.ring(19.0, 31.0, 4.6, 2.6, 2.2);
            d.line(&[(22.5, 33.5), (23.5, 37.5), (21.0, 41.5)], 2.0);
        }
        PolygonalLasso => {
            d.line(
                &[
                    (21.0, 27.5),
                    (13.0, 13.0),
                    (33.5, 18.5),
                    (42.5, 9.5),
                    (42.5, 26.5),
                    (29.0, 34.5),
                ],
                2.6,
            );
            d.ring(24.5, 31.0, 3.6, 3.0, 2.4);
            d.line(&[(22.0, 33.5), (13.5, 36.5)], 2.6);
            d.line(&[(26.5, 34.5), (22.0, 41.5)], 2.6);
        }
        MagneticLasso => {
            d.arc((37.0, 15.5), (6.0, 6.5), (-1.7, 1.7), 4.0);
            d.rect(28.0, 9.0, 33.0, 12.0);
            d.rect(28.0, 19.0, 33.0, 22.0);
            d.line(&[(13.5, 13.0), (26.5, 17.0)], 2.6);
            d.line(
                &[(15.5, 14.5), (23.0, 27.5), (29.5, 29.5), (43.0, 24.5)],
                2.6,
            );
            d.ring(26.0, 31.0, 3.6, 3.0, 2.4);
            d.line(&[(23.5, 33.5), (15.0, 36.5)], 2.6);
            d.line(&[(27.5, 34.5), (22.5, 41.5)], 2.6);
        }
        SelectionBrush => {
            let gray = d.with(Color32::from_gray(0x75));
            let blob: Vec<(f32, f32)> = (0..48)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 48.0;
                    let (x, y) = (17.5 * a.cos(), 11.0 * a.sin());
                    let r = std::f32::consts::FRAC_PI_4;
                    (
                        28.0 + x * r.cos() - y * r.sin(),
                        23.5 + x * r.sin() + y * r.cos(),
                    )
                })
                .collect();
            gray.fan((28.0, 23.5), &blob);
            for (i, w) in blob.iter().enumerate().filter(|(i, _)| i % 4 == 0) {
                let next = blob[(i + 2) % blob.len()];
                d.line(&[*w, next], 2.4);
            }
            brush(d, 0.5, -4.5);
        }
        ObjectSelection => {
            for k in 0..7 {
                d.rect(14.0 + 4.0 * k as f32, 8.0, 16.0 + 4.0 * k as f32, 10.0);
            }
            for k in 1..7 {
                d.rect(14.0, 8.0 + 4.0 * k as f32, 16.0, 10.0 + 4.0 * k as f32);
            }
            for k in 1..5 {
                d.rect(38.0, 8.0 + 4.0 * k as f32, 40.0, 10.0 + 4.0 * k as f32);
            }
            for k in 1..4 {
                d.rect(14.0 + 4.0 * k as f32, 32.0, 16.0 + 4.0 * k as f32, 34.0);
            }
            d.rects(&[[20.0, 14.0, 34.0, 19.0], [20.0, 19.0, 28.0, 28.0]]);
            d.poly(&[(32.0, 23.0), (45.0, 35.5), (32.0, 35.5)]);
            d.poly(&[(32.0, 35.5), (37.0, 35.5), (33.0, 42.0), (32.0, 42.0)]);
        }
        QuickSelection => {
            for (x, y) in [
                (16.0, 12.0),
                (20.0, 12.0),
                (12.0, 14.0),
                (24.0, 14.0),
                (10.0, 18.0),
                (26.0, 18.0),
                (10.0, 22.0),
                (10.0, 26.0),
                (10.0, 30.0),
                (12.0, 34.0),
                (16.0, 36.0),
                (20.0, 36.0),
            ] {
                d.rect(x, y, x + 2.0, y + 2.0);
            }
            d.poly(&[
                (44.5, 10.5),
                (45.5, 13.0),
                (39.5, 22.5),
                (36.0, 24.0),
                (35.0, 21.5),
            ]);
            d.poly(&[
                (31.0, 23.5),
                (35.5, 24.0),
                (38.5, 27.0),
                (38.5, 30.0),
                (36.5, 33.5),
                (33.0, 35.5),
                (28.0, 35.5),
                (24.0, 36.0),
                (25.5, 33.0),
                (27.0, 29.0),
                (29.0, 25.5),
            ]);
        }
        MagicWand => {
            d.bar((16.0, 37.0), (29.5, 23.5), 8.0);
            d.outline(
                &[(29.0, 21.0), (34.0, 16.0), (39.0, 21.0), (34.0, 26.0)],
                2.4,
            );
            d.sparkle(27.0, 10.5, 4.6);
            d.sparkle(42.5, 10.5, 4.0);
            d.sparkle(44.5, 22.5, 3.6);
        }
        PerspectiveCrop => {
            fill.poly(&[(20.0, 11.5), (38.0, 14.5), (38.0, 32.0), (20.0, 34.5)]);
            for (x, y) in [(16.0, 6.0), (36.0, 10.0), (36.0, 30.0), (16.0, 34.0)] {
                d.rects(&[
                    [x, y, x + 6.0, y + 2.0],
                    [x, y + 4.0, x + 6.0, y + 6.0],
                    [x, y + 2.0, x + 2.0, y + 4.0],
                    [x + 4.0, y + 2.0, x + 6.0, y + 4.0],
                ]);
            }
            d.rects(&[
                [18.0, 12.0, 20.0, 34.0],
                [26.0, 11.0, 28.0, 35.0],
                [32.0, 13.0, 34.0, 33.0],
                [38.0, 16.0, 40.0, 30.0],
                [18.0, 22.0, 40.0, 24.0],
            ]);
            d.line(&[(21.0, 10.5), (36.5, 13.0)], 2.6);
            d.line(&[(21.0, 35.5), (36.5, 33.0)], 2.6);
        }
        Slice | SliceSelect => {
            d.poly(&[(30.0, 14.0), (46.0, 14.0), (38.5, 24.0), (36.0, 24.0)]);
            d.poly(&[
                (31.5, 22.0),
                (34.5, 25.0),
                (16.0, 37.5),
                (12.0, 38.0),
                (13.5, 36.0),
            ]);
            if tool == SliceSelect {
                d.poly(&[(10.0, 10.0), (20.5, 20.5), (10.0, 24.0)]);
            }
        }
        Eyedropper | ColorSampler => {
            let dx = if tool == ColorSampler { 2.0 } else { 0.0 };
            d.bar((39.0 + dx, 14.0), (36.0 + dx, 17.0), 6.5);
            d.bar((29.0 + dx, 15.5), (38.0 + dx, 25.5), 4.0);
            d.outline(
                &[
                    (25.0 + dx, 21.5),
                    (31.5 + dx, 27.5),
                    (17.0 + dx, 39.5),
                    (13.5 + dx, 40.5),
                    (12.0 + dx, 37.0),
                    (12.5 + dx, 34.5),
                ],
                2.5,
            );
            if tool == ColorSampler {
                d.ring(15.0, 15.5, 4.0, 4.0, 2.0);
                d.rects(&[
                    [14.0, 7.0, 16.0, 11.5],
                    [14.0, 19.5, 16.0, 25.0],
                    [6.0, 15.0, 10.5, 17.0],
                    [19.5, 15.0, 24.0, 17.0],
                ]);
            }
        }
        Ruler => d.rects(&[
            [12.0, 17.0, 46.0, 25.0],
            [12.0, 25.0, 16.0, 31.0],
            [18.0, 25.0, 26.0, 29.0],
            [18.0, 29.0, 20.0, 31.0],
            [22.0, 29.0, 26.0, 31.0],
            [28.0, 25.0, 36.0, 29.0],
            [28.0, 29.0, 30.0, 31.0],
            [32.0, 29.0, 36.0, 31.0],
            [38.0, 25.0, 46.0, 29.0],
            [38.0, 29.0, 40.0, 31.0],
            [42.0, 29.0, 46.0, 31.0],
        ]),
        Note => {
            d.rects(&[
                [16.0, 11.0, 42.0, 19.0],
                [16.0, 19.0, 20.0, 21.0],
                [37.0, 19.0, 42.0, 21.0],
                [16.0, 21.0, 42.0, 23.0],
                [16.0, 23.0, 20.0, 25.0],
                [37.0, 23.0, 42.0, 25.0],
                [16.0, 25.0, 42.0, 27.0],
                [16.0, 27.0, 20.0, 29.0],
                [26.0, 27.0, 30.0, 29.0],
                [16.0, 29.0, 30.0, 39.0],
            ]);
            d.poly(&[(32.0, 29.0), (42.0, 29.0), (32.0, 39.0)]);
        }
        Count => {
            d.rects(&[[14.0, 13.0, 17.0, 28.0], [10.0, 27.0, 19.0, 29.0]]);
            d.line(&[(11.5, 16.5), (14.5, 13.5)], 2.2);
            d.arc(
                (25.0, 24.0),
                (4.2, 3.8),
                (std::f32::consts::PI * 1.05, std::f32::consts::TAU * 1.02),
                2.4,
            );
            d.line(&[(29.0, 25.0), (21.0, 37.0)], 2.6);
            d.rect(20.0, 37.0, 33.0, 39.0);
            d.arc(
                (40.5, 13.5),
                (4.2, 3.6),
                (std::f32::consts::PI * 1.1, std::f32::consts::PI * 2.45),
                2.4,
            );
            d.arc(
                (40.5, 21.5),
                (4.6, 4.2),
                (
                    -std::f32::consts::FRAC_PI_2 * 1.1,
                    std::f32::consts::PI * 0.85,
                ),
                2.4,
            );
        }
        SpotHealingBrush => {
            bandage(d);
            // Dashes around the top left, clear of the bandage
            for k in 0..6 {
                let a = 2.05 + k as f32 * 0.5;
                d.arc((27.5, 24.0), (17.0, 16.5), (a, a + 0.24), 2.4);
            }
        }
        HealingBrush => bandage(d),
        Patch => {
            d.with(Color32::from_gray(144)).rect(18.0, 16.0, 38.0, 34.0);
            d.rects(&[
                [16.0, 14.0, 40.0, 16.0],
                [16.0, 34.0, 40.0, 36.0],
                [16.0, 14.0, 18.0, 36.0],
                [38.0, 14.0, 40.0, 36.0],
                [20.0, 10.0, 22.0, 17.0],
                [26.0, 10.0, 28.0, 17.0],
                [32.0, 10.0, 34.0, 17.0],
                [22.0, 33.0, 24.0, 40.0],
                [28.0, 33.0, 30.0, 40.0],
                [34.0, 33.0, 36.0, 40.0],
            ]);
            for y in [18.0, 24.0, 30.0] {
                d.rects(&[[13.0, y, 19.0, y + 2.0], [37.0, y, 43.0, y + 2.0]]);
            }
        }
        ContentAwareMove => {
            d.line(&[(13.0, 15.0), (21.0, 16.0), (36.0, 31.0)], 4.2);
            d.poly(&[(36.0, 24.5), (44.5, 31.5), (36.0, 38.5)]);
            d.line(&[(43.0, 15.0), (35.0, 16.0), (20.0, 31.0)], 4.2);
            d.poly(&[(20.0, 24.5), (20.0, 38.5), (11.5, 31.5)]);
        }
        RedEye => {
            d.rects(&[[16.0, 7.0, 18.0, 20.0], [10.0, 13.0, 24.0, 15.0]]);
            d.ring(30.5, 29.5, 13.0, 9.5, 3.4);
            d.disc(30.5, 29.0, 5.2, 5.2);
            d.hole().disc(33.0, 26.5, 1.8, 1.8);
        }
        Brush => brush(d, 0.0, 0.0),
        Pencil => {
            let body = [(36.0, 8.5), (44.5, 17.0), (23.5, 37.5), (15.0, 29.0)];
            fill.poly(&body);
            d.outline(
                &[
                    (36.0, 8.5),
                    (44.5, 17.0),
                    (23.5, 37.5),
                    (12.0, 41.0),
                    (15.0, 29.0),
                ],
                2.6,
            );
            d.poly(&[(13.5, 35.5), (17.5, 39.5), (12.0, 41.0)]);
        }
        ColorReplacement => {
            d.rects(&[
                [6.0, 17.0, 16.0, 19.0],
                [6.0, 25.0, 16.0, 27.0],
                [6.0, 19.0, 8.0, 25.0],
                [14.0, 19.0, 16.0, 25.0],
            ]);
            fill.rect(8.0, 19.0, 14.0, 25.0);
            d.rects(&[[14.0, 7.0, 16.0, 13.0], [12.0, 9.0, 21.0, 11.0]]);
            d.line(&[(20.0, 10.0), (23.0, 13.0), (23.0, 17.0)], 2.2);
            d.poly(&[(19.5, 17.0), (26.5, 17.0), (23.0, 20.5)]);
            brush(d, 1.5, 0.0);
        }
        MixerBrush => {
            d.disc(11.5, 23.5, 5.6, 5.6);
            d.poly(&[(10.5, 14.0), (16.6, 22.0), (6.4, 22.0)]);
            brush(d, 3.5, 0.0);
        }
        CloneStamp => stamp(d),
        PatternStamp => {
            stamp(d);
            d.rects(&[
                [6.0, 11.0, 10.0, 15.0],
                [14.0, 11.0, 18.0, 15.0],
                [10.0, 15.0, 14.0, 19.0],
                [6.0, 19.0, 10.0, 23.0],
                [14.0, 19.0, 18.0, 23.0],
            ]);
        }
        HistoryBrush => {
            brush(d, 1.5, 0.0);
            d.arc((26.5, 23.0), (9.5, 7.5), (3.6, 4.85), 2.6);
            d.poly(&[(10.0, 22.5), (21.5, 22.5), (15.5, 15.5)]);
            for (x, y) in [(40.0, 23.0), (40.0, 27.0), (38.0, 31.0), (34.0, 35.0)] {
                d.rect(x, y, x + 2.0, y + 2.0);
            }
        }
        ArtHistoryBrush => {
            d.arc((21.5, 15.0), (9.0, 7.5), (2.4, 6.6), 2.5);
            d.arc((20.5, 17.5), (3.5, 3.0), (0.0, 4.2), 2.3);
            brush(d, 2.5, -1.0);
            d.arc((30.0, 24.0), (15.0, 15.0), (-0.45, 1.25), 2.5);
        }
        Eraser => {
            eraser(d, 0.0, 0.0, 38.0);
            d.rect(17.0, 38.0, 42.0, 40.0);
        }
        BackgroundEraser => {
            eraser(d, 2.0, 1.5, 40.0);
            d.ring(23.5, 11.0, 2.8, 2.8, 2.2);
            d.ring(23.5, 20.5, 2.8, 2.8, 2.2);
            d.line(&[(21.0, 13.0), (10.5, 20.5)], 2.4);
            d.line(&[(21.0, 18.5), (10.5, 10.0)], 2.4);
        }
        MagicEraser => {
            eraser(d, 2.0, 1.5, 40.0);
            d.sparkle(26.5, 12.5, 4.6);
            d.sparkle(14.5, 16.5, 4.6);
            d.sparkle(10.5, 28.5, 4.6);
        }
        Gradient => {
            d.gradient(
                14.0,
                14.0,
                42.0,
                36.0,
                Color32::from_gray(47),
                Color32::from_gray(202),
            );
            d.rects(&[
                [12.0, 12.0, 44.0, 14.0],
                [12.0, 36.0, 44.0, 38.0],
                [12.0, 14.0, 14.0, 36.0],
                [42.0, 14.0, 44.0, 36.0],
            ]);
        }
        PaintBucket => {
            d.outline(
                &[(11.5, 28.5), (25.5, 14.5), (40.0, 26.5), (24.5, 41.0)],
                2.6,
            );
            d.line(
                &[
                    (19.5, 21.0),
                    (19.5, 11.0),
                    (22.5, 9.0),
                    (25.0, 11.0),
                    (28.5, 21.0),
                ],
                2.4,
            );
            d.disc(28.5, 23.0, 2.6, 2.6);
            d.disc(42.0, 33.0, 3.8, 3.8);
            d.poly(&[(43.0, 26.0), (45.8, 32.5), (38.2, 32.5)]);
        }
        Blur => {
            d.disc(27.5, 30.0, 10.0, 10.0);
            d.poly(&[(26.8, 10.0), (36.8, 28.0), (17.2, 28.0)]);
        }
        Sharpen => d.poly(&[(28.5, 10.0), (42.0, 40.0), (15.0, 40.0)]),
        Smudge => {
            d.fan(
                (32.0, 21.0),
                &[
                    (25.0, 12.0),
                    (41.0, 12.0),
                    (42.0, 18.0),
                    (42.0, 27.0),
                    (37.0, 33.0),
                    (31.0, 34.0),
                    (25.0, 29.0),
                    (18.0, 27.0),
                    (18.0, 16.0),
                ],
            );
            d.bar((31.0, 25.0), (18.0, 39.5), 7.0);
            let h = d.hole();
            h.line(&[(35.5, 25.0), (29.0, 33.0)], 1.4);
            h.line(&[(27.0, 23.0), (21.0, 30.0)], 1.4);
        }
        AdjustmentBrush => {
            brush(d, -1.5, -0.5);
            d.ring(38.0, 33.5, 7.5, 7.5, 2.5);
            let half: Vec<(f32, f32)> = ellipse(38.0, 33.5, 7.5, 7.5, -0.8, 2.35);
            d.poly(&half);
        }
        Dodge => {
            d.disc(32.0, 21.5, 10.0, 10.0);
            d.line(&[(31.0, 30.0), (13.5, 41.5)], 4.5);
        }
        Burn => {
            d.fan(
                (30.0, 27.0),
                &[
                    (19.0, 14.0),
                    (29.0, 14.5),
                    (41.0, 21.0),
                    (44.0, 26.0),
                    (43.0, 33.0),
                    (38.0, 35.5),
                    (18.0, 36.0),
                    (14.0, 33.0),
                    (13.0, 29.5),
                    (19.0, 28.0),
                    (12.0, 26.0),
                    (12.0, 18.0),
                    (15.0, 16.0),
                ],
            );
            d.hole().poly(&[
                (19.5, 23.0),
                (23.5, 22.5),
                (26.0, 25.5),
                (26.0, 27.5),
                (15.0, 28.0),
            ]);
        }
        Sponge => {
            d.disc(27.5, 25.5, 16.0, 13.5);
            let h = d.hole();
            for (x, y, r) in [
                (20.0, 16.5, 1.4),
                (28.0, 15.5, 1.4),
                (36.5, 17.5, 1.5),
                (16.0, 22.0, 1.3),
                (24.0, 22.5, 1.5),
                (32.5, 22.0, 1.3),
                (40.0, 23.5, 1.4),
                (19.5, 28.0, 1.4),
                (28.0, 28.5, 1.5),
                (36.0, 29.0, 1.4),
                (23.5, 33.5, 1.3),
                (31.5, 34.0, 1.4),
            ] {
                h.disc(x, y, r, r);
            }
        }
        Pen => nib(d, 0.0, 0.0),
        FreeformPen => {
            nib(d, 2.0, -2.0);
            for (x, y) in [
                (8.0, 11.0),
                (12.0, 13.0),
                (12.0, 17.0),
                (10.0, 21.0),
                (8.0, 25.0),
                (6.0, 29.0),
                (6.0, 33.0),
                (8.0, 37.0),
                (12.0, 39.0),
            ] {
                d.rect(x, y, x + 2.0, y + 2.0);
            }
        }
        CurvaturePen => {
            nib(d, 6.0, -2.0);
            d.ring(16.5, 21.0, 2.8, 2.8, 2.0);
            d.ring(12.5, 32.5, 2.6, 2.6, 2.0);
            d.line(&[(14.5, 23.5), (12.0, 27.0), (11.5, 30.0)], 2.2);
            d.line(&[(13.0, 35.0), (16.5, 37.5), (19.0, 39.0)], 2.2);
        }
        AddAnchorPoint => {
            nib(d, 0.0, 0.0);
            d.rects(&[[14.0, 11.0, 16.0, 21.0], [10.0, 15.0, 20.0, 17.0]]);
        }
        DeleteAnchorPoint => {
            nib(d, 0.0, 0.0);
            d.rect(10.0, 15.0, 20.0, 17.0);
        }
        ConvertPoint => {
            d.rect(22.0, 13.0, 24.0, 40.0);
            d.line(&[(23.0, 14.0), (40.5, 32.5)], 2.6);
        }
        HorizontalType | VerticalType => {
            d.rects(&[
                [18.5, 13.0, 41.5, 17.0],
                [18.0, 17.0, 22.0, 20.0],
                [38.0, 17.0, 42.0, 20.0],
                [28.0, 17.0, 32.0, 37.0],
                [23.0, 37.0, 37.0, 40.5],
            ]);
            if tool == VerticalType {
                d.rect(10.0, 13.0, 12.0, 35.0);
                d.poly(&[(6.0, 35.0), (16.0, 35.0), (11.0, 41.0)]);
            }
        }
        HorizontalTypeMask | VerticalTypeMask => {
            d.rects(&[
                [18.0, 13.0, 22.0, 15.0],
                [24.0, 13.0, 30.0, 15.0],
                [32.0, 13.0, 36.0, 15.0],
                [38.0, 13.0, 42.0, 15.0],
                [16.0, 17.0, 18.0, 19.0],
                [22.0, 17.0, 26.0, 19.0],
                [34.0, 17.0, 38.0, 19.0],
                [42.0, 17.0, 44.0, 19.0],
                [16.0, 19.0, 20.0, 21.0],
                [26.0, 19.0, 28.0, 21.0],
                [32.0, 19.0, 34.0, 21.0],
                [40.0, 19.0, 44.0, 21.0],
                [26.0, 21.0, 28.0, 23.0],
                [32.0, 21.0, 34.0, 23.0],
                [26.0, 25.0, 28.0, 29.0],
                [32.0, 25.0, 34.0, 29.0],
                [26.0, 31.0, 28.0, 35.0],
                [32.0, 31.0, 34.0, 35.0],
                [24.0, 37.0, 26.0, 41.0],
                [34.0, 37.0, 36.0, 41.0],
                [28.0, 39.0, 32.0, 41.0],
            ]);
            if tool == VerticalTypeMask {
                d.rect(10.0, 13.0, 12.0, 35.0);
                d.poly(&[(6.0, 35.0), (16.0, 35.0), (11.0, 41.0)]);
            }
        }
        PathSelection => {
            let black = d.with(Color32::BLACK);
            let body = [(21.0, 12.5), (39.5, 29.5), (30.5, 30.0), (21.5, 39.5)];
            black.poly(&[body[0], body[1], body[2]]);
            black.poly(&[body[0], body[2], body[3]]);
            black.poly(&[(28.0, 31.0), (31.0, 31.0), (35.0, 42.5), (32.5, 43.5)]);
            d.outline(
                &[
                    (21.0, 12.5),
                    (39.5, 29.5),
                    (30.5, 30.0),
                    (35.0, 42.5),
                    (32.5, 43.5),
                    (27.5, 33.0),
                    (21.5, 39.5),
                ],
                2.4,
            );
        }
        DirectSelection => {
            d.poly(&[(18.0, 11.0), (38.5, 30.5), (30.0, 31.0)]);
            d.poly(&[(18.0, 11.0), (30.0, 31.0), (18.0, 40.5)]);
            d.poly(&[(27.0, 31.0), (31.0, 31.0), (35.5, 43.5), (31.5, 44.0)]);
        }
        Rectangle => {
            fill.rect(14.0, 15.0, 42.0, 37.0);
            d.rects(&[
                [12.0, 13.0, 44.0, 15.0],
                [12.0, 37.0, 44.0, 39.0],
                [12.0, 15.0, 14.0, 37.0],
                [42.0, 15.0, 44.0, 37.0],
            ]);
        }
        Ellipse => {
            fill.disc(27.0, 26.0, 14.5, 12.5);
            d.ring(27.0, 26.0, 14.8, 12.8, 2.6);
        }
        Triangle => {
            fill.poly(&[(28.0, 14.0), (42.0, 37.5), (14.0, 37.5)]);
            d.outline(&[(28.0, 12.8), (43.0, 37.8), (13.0, 37.8)], 2.8);
        }
        Polygon => {
            let hex = [
                (21.0, 14.0),
                (36.0, 14.0),
                (43.0, 26.0),
                (36.0, 38.0),
                (21.0, 38.0),
                (14.5, 26.0),
            ];
            fill.poly(&hex);
            d.outline(&hex, 2.6);
        }
        Line => d.line(&[(15.0, 40.0), (41.5, 13.5)], 2.8),
        CustomShape => {
            let shape = [
                (31.0, 11.5),
                (35.0, 13.5),
                (36.0, 17.5),
                (42.0, 17.5),
                (45.0, 19.5),
                (45.0, 22.5),
                (41.0, 26.0),
                (36.0, 29.0),
                (36.0, 32.0),
                (38.5, 35.5),
                (36.5, 39.0),
                (33.0, 40.5),
                (30.0, 39.0),
                (28.0, 35.0),
                (23.0, 34.5),
                (18.0, 38.0),
                (13.0, 39.5),
                (10.5, 37.0),
                (10.5, 33.0),
                (14.0, 29.0),
                (18.0, 26.0),
                (18.5, 21.0),
                (17.0, 17.0),
                (19.5, 15.5),
                (25.5, 15.0),
                (27.5, 13.0),
            ];
            fill.fan((28.0, 26.0), &shape);
            d.outline(&shape, 2.5);
        }
        Hand => {
            for (x0, top, x1) in [
                (21.0, 15.0, 25.0),
                (27.5, 12.0, 31.0),
                (34.0, 13.5, 37.5),
                (40.0, 20.5, 44.0),
            ] {
                d.rect(x0, top, x1, 28.0);
                d.disc((x0 + x1) / 2.0, top, (x1 - x0) / 2.0, 1.5);
            }
            d.fan(
                (31.0, 34.0),
                &[
                    (21.0, 26.5),
                    (44.0, 26.5),
                    (43.5, 35.0),
                    (39.0, 40.0),
                    (32.0, 43.5),
                    (25.0, 43.0),
                    (20.0, 38.0),
                    (17.0, 32.0),
                ],
            );
            d.bar((13.5, 26.5), (20.0, 32.5), 5.0);
        }
        RotateView => {
            d.arc((30.0, 23.0), (12.5, 11.5), (3.6, 5.0), 2.4);
            d.poly(&[(38.0, 9.0), (46.0, 16.0), (35.5, 16.5)]);
            d.fan(
                (36.5, 27.0),
                &[(30.0, 15.5), (45.5, 27.0), (35.0, 38.5), (28.5, 30.0)],
            );
            for (x, top) in [(18.0, 23.0), (22.0, 21.0), (26.0, 22.0)] {
                d.rect(x, top, x + 2.0, 33.0);
            }
            d.fan(
                (22.5, 36.5),
                &[
                    (13.0, 31.0),
                    (18.0, 32.0),
                    (32.0, 30.0),
                    (32.0, 38.0),
                    (28.0, 41.5),
                    (20.0, 42.0),
                    (16.0, 38.0),
                ],
            );
        }
        Zoom => {
            d.ring(24.5, 23.5, 10.4, 10.4, 2.6);
            d.line(&[(32.5, 32.0), (43.0, 42.0)], 4.6);
        }
        Remove => {
            bandage(d);
            // Two four-point stars at the top left, each cut clear of the
            // bandage
            let star = |pen: &self::Pen, cx: f32, cy: f32, h: f32| {
                let w = h * 0.32;
                pen.poly(&[(cx, cy - h), (cx + w, cy), (cx, cy + h), (cx - w, cy)]);
                pen.poly(&[(cx - h, cy), (cx, cy - w), (cx + h, cy), (cx, cy + w)]);
            };
            for (cx, cy, h) in [(21.4, 13.0, 7.8), (14.5, 22.5, 5.0)] {
                let hole = d.hole();
                star(&hole, cx, cy, h + 2.5);
                hole.disc(cx, cy, h * 0.55, h * 0.55);
                star(d, cx, cy, h);
            }
        }
    }
    true
}
