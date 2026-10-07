//! The Perspective Crop tool (C, with the Crop tool): a four-cornered box
//! drawn on the image, each corner moved on its own, then cropped so that
//! what was inside becomes the whole (rectangular) canvas, as Photoshop
//! 2026's tool does.
//!
//! Dragging draws a box (or four clicks place its corners); corner handles
//! move one corner, side handles move a side, dragging inside moves the
//! box, and dragging outside draws a new one. Enter, a double click inside
//! or switching tools crops; Escape removes the box.

use egui::{Color32, CursorIcon, Key, Mesh, Modifiers, Pos2, Rect, Shape, Stroke, Ui, Vec2};

use crate::document_view::{to_doc, to_screen};
use crate::state::DocState;
use crate::theme::{color, pt};

const GRAB: f32 = pt(10.0);
const HANDLE: f32 = pt(7.0);
const LIGHT: Color32 = Color32::from_gray(0xf4);
const DARK: Color32 = Color32::from_gray(0x32);
/// The grid's lines are about this far apart on screen.
const GRID_STEP: f32 = pt(40.0);

/// The box on the image (document pixels): top-left, top-right,
/// bottom-right, bottom-left as drawn; points clicked before it exists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PerspectiveBox {
    pub quad: Option<[Pos2; 4]>,
    pub clicks: Vec<Pos2>,
    pub drag: Option<Drag>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drag {
    pub kind: DragKind,
    /// Where the drag started (document pixels) and the box then.
    pub start: Pos2,
    pub quad: [Pos2; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragKind {
    /// Drawing a new box.
    New,
    Corner(usize),
    /// The side from corner `i` to corner `i + 1`.
    Side(usize),
    Move,
}

/// The options bar's settings: the size and resolution to crop to (empty:
/// the box's own size) and Show Grid.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PerspectiveOptions {
    pub width: String,
    pub height: String,
    pub resolution: String,
    pub per_cm: bool,
    pub grid: bool,
}

impl PerspectiveOptions {
    /// From the options bar's settings (`pcrop.*`).
    pub fn from_app(app: &mut crate::state::AppState) -> Self {
        Self {
            width: app.setting("pcrop.w", "").clone(),
            height: app.setting("pcrop.h", "").clone(),
            resolution: app.setting("pcrop.res", "").clone(),
            per_cm: app.setting("pcrop.unit", "0") == "1",
            grid: app.flag("pcrop.grid", true),
        }
    }

    fn ppi(&self) -> Option<f32> {
        let v: f32 = self.resolution.trim().parse().ok()?;
        (v > 0.0).then_some(if self.per_cm { v * 2.54 } else { v })
    }

    /// The size typed, in pixels (units as the Crop tool's fields read
    /// them; inches and the like need a resolution).
    pub fn size(&self) -> Option<(u32, u32)> {
        let px = |text: &str| {
            let (v, unit) = crate::crop_tool::parse_length(text)?;
            let v = match unit.and_then(|u| u.per_inch()) {
                None => v,
                Some(k) => v / k * self.ppi()?,
            };
            Some(v.round().max(1.0) as u32)
        };
        Some((px(&self.width)?, px(&self.height)?))
    }
}

fn tuple(p: Pos2) -> (f32, f32) {
    (p.x, p.y)
}

/// Whether `p` is inside the (convex) quad.
fn inside(q: &[Pos2; 4], p: Pos2) -> bool {
    let mut sign = 0.0f32;
    for i in 0..4 {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        let cross = (b - a).x * (p - a).y - (b - a).y * (p - a).x;
        if cross.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}

/// The handle under screen point `p`: a corner, or a side's middle.
fn handle_at(state: &DocState, q: &[Pos2; 4], p: Pos2, ppp: f32) -> Option<DragKind> {
    let s = q.map(|c| to_screen(state, c, ppp));
    (0..4)
        .find(|&i| s[i].distance(p) <= GRAB)
        .map(DragKind::Corner)
        .or_else(|| {
            (0..4)
                .find(|&i| s[i].lerp(s[(i + 1) % 4], 0.5).distance(p) <= GRAB)
                .map(DragKind::Side)
        })
}

/// Crops to the box: the size typed, or the box's mean side lengths, and
/// the resolution typed. Returns true when the image changed.
pub fn commit(state: &mut DocState, options: &PerspectiveOptions) -> bool {
    let Some(quad) = state.perspective_crop.take().and_then(|b| b.quad) else {
        return false;
    };
    let quad = quad.map(tuple);
    let (w, h) = options
        .size()
        .unwrap_or_else(|| op_core::image_ops::perspective_size(quad));
    op_core::image_ops::perspective_crop(&mut state.doc, quad, w, h);
    if let Some(ppi) = options.ppi() {
        state.doc.resolution = ppi;
    }
    state.record("Perspective Crop");
    true
}

/// Handles the tool's input on the canvas.
pub fn input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    options: &PerspectiveOptions,
    ppp: f32,
) {
    let typing = ui.ctx().egui_wants_keyboard_input();
    let (enter, escape) = ui.input_mut(|i| {
        (
            !typing && i.consume_key(Modifiers::NONE, Key::Enter),
            !typing && i.consume_key(Modifiers::NONE, Key::Escape),
        )
    });
    if escape {
        state.perspective_crop = None;
        return;
    }
    let pb = state.perspective_crop.get_or_insert_with(Default::default);
    let quad = pb.quad;
    let double_inside = response.double_clicked()
        && quad.is_some_and(|q| {
            response
                .interact_pointer_pos()
                .is_some_and(|p| inside(&q, to_doc(state, p, ppp)))
        });
    if (enter || double_inside) && quad.is_some() {
        commit(state, options);
        return;
    }
    // Four clicks place the corners
    if response.clicked()
        && let Some(p) = response.interact_pointer_pos()
        && quad.is_none()
    {
        let d = to_doc(state, p, ppp);
        let pb = state.perspective_crop.get_or_insert_with(Default::default);
        pb.clicks.push(d);
        if pb.clicks.len() == 4 {
            let c = std::mem::take(&mut pb.clicks);
            pb.quad = Some([c[0], c[1], c[2], c[3]]);
        }
        return;
    }
    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        let d = to_doc(state, p, ppp);
        let kind = match quad {
            Some(q) => match handle_at(state, &q, p, ppp) {
                Some(k) => k,
                None if inside(&q, d) => DragKind::Move,
                None => DragKind::New,
            },
            None => DragKind::New,
        };
        let pb = state.perspective_crop.get_or_insert_with(Default::default);
        pb.clicks.clear();
        pb.drag = Some(Drag {
            kind,
            start: d,
            quad: quad.unwrap_or([d; 4]),
        });
    }
    let Some(drag) = state.perspective_crop.as_ref().and_then(|b| b.drag) else {
        return;
    };
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let at = to_doc(state, p, ppp);
        let d = at - drag.start;
        let mut q = drag.quad;
        match drag.kind {
            DragKind::New => {
                let r = Rect::from_two_pos(drag.start, at);
                q = [
                    r.left_top(),
                    r.right_top(),
                    r.right_bottom(),
                    r.left_bottom(),
                ];
            }
            DragKind::Corner(i) => q[i] += d,
            DragKind::Side(i) => {
                q[i] += d;
                q[(i + 1) % 4] += d;
            }
            DragKind::Move => q.iter_mut().for_each(|c| *c += d),
        }
        if let Some(pb) = &mut state.perspective_crop {
            pb.quad = Some(q);
        }
    }
    if (response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()))
        && let Some(pb) = &mut state.perspective_crop
    {
        pb.drag = None;
        // A box too small to see is dropped
        if pb.quad.is_some_and(|q| {
            let r = Rect::from_points(&q);
            r.width() < 1.0 || r.height() < 1.0
        }) {
            pb.quad = if drag.kind == DragKind::New {
                None
            } else {
                Some(drag.quad)
            };
        }
    }
    ui.ctx().request_repaint();
}

pub fn cursor(state: &DocState, p: Pos2, ppp: f32) -> CursorIcon {
    let Some(q) = state.perspective_crop.as_ref().and_then(|b| b.quad) else {
        return CursorIcon::Crosshair;
    };
    match handle_at(state, &q, p, ppp) {
        Some(DragKind::Corner(_)) | Some(DragKind::Side(_)) => CursorIcon::Crosshair,
        _ if inside(&q, to_doc(state, p, ppp)) => CursorIcon::Move,
        _ => CursorIcon::Crosshair,
    }
}

/// The box on the canvas: the image outside it shielded as the Crop
/// tool's is, the grid inside (Show Grid), its lines and handles; before
/// the box exists, the corners clicked so far.
pub fn draw(ui: &Ui, state: &DocState, options: &PerspectiveOptions, canvas: Rect, ppp: f32) {
    let Some(pb) = &state.perspective_crop else {
        return;
    };
    let painter = ui.painter_at(canvas);
    let one = pt(1.0);
    if pb.quad.is_none() && !pb.clicks.is_empty() {
        let points: Vec<Pos2> = pb
            .clicks
            .iter()
            .map(|&c| to_screen(state, c, ppp))
            .collect();
        painter.add(Shape::line(points.clone(), Stroke::new(one, LIGHT)));
        for p in points {
            handle(&painter, p);
        }
        return;
    }
    let Some(q) = pb.quad else {
        return;
    };
    let s = q.map(|c| to_screen(state, c, ppp));
    // The shield: the image (and the box, if it reaches past it) around
    // the box, as a ring of four quads between an outer frame and the box
    let image = Rect::from_two_pos(
        to_screen(state, Pos2::ZERO, ppp),
        to_screen(
            state,
            Pos2::new(state.doc.width as f32, state.doc.height as f32),
            ppp,
        ),
    );
    let outer = image
        .union(Rect::from_points(&s))
        .intersect(canvas.expand(one));
    let o = [
        outer.left_top(),
        outer.right_top(),
        outer.right_bottom(),
        outer.left_bottom(),
    ];
    let shield = Color32::from_rgba_unmultiplied(
        color::PASTEBOARD.r(),
        color::PASTEBOARD.g(),
        color::PASTEBOARD.b(),
        (0.75 * 255.0) as u8,
    );
    let mut mesh = Mesh::default();
    for p in o.iter().chain(s.iter()) {
        mesh.colored_vertex(*p, shield);
    }
    for i in 0..4u32 {
        let j = (i + 1) % 4;
        mesh.add_triangle(i, j, 4 + i);
        mesh.add_triangle(j, 4 + j, 4 + i);
    }
    painter.add(Shape::mesh(mesh));
    if options.grid {
        grid(&painter, &q, state, ppp);
    }
    painter.add(Shape::closed_line(s.to_vec(), Stroke::new(one, DARK)));
    painter.add(Shape::closed_line(
        s.to_vec(),
        Stroke::new(one * 0.5, LIGHT),
    ));
    for i in 0..4 {
        handle(&painter, s[i]);
        handle(&painter, s[i].lerp(s[(i + 1) % 4], 0.5));
    }
}

/// Lines across the box through the projective map, about
/// [`GRID_STEP`] apart on screen.
fn grid(painter: &egui::Painter, q: &[Pos2; 4], state: &DocState, ppp: f32) {
    let m = op_core::transform::Projective::rect_to_quad((0.0, 0.0, 1.0, 1.0), q.map(tuple));
    let s = q.map(|c| to_screen(state, c, ppp));
    let across = |a: usize, b: usize| (s[a].distance(s[b]) / GRID_STEP).round().max(2.0) as usize;
    let (nu, nv) = (
        across(0, 1).max(across(3, 2)),
        across(0, 3).max(across(1, 2)),
    );
    let at = |u: f32, v: f32| {
        let (x, y) = m.apply((u, v));
        to_screen(state, Pos2::new(x, y), ppp)
    };
    let line = Stroke::new(pt(1.0), Color32::from_white_alpha(110));
    for k in 1..nu {
        let u = k as f32 / nu as f32;
        painter.line_segment([at(u, 0.0), at(u, 1.0)], line);
    }
    for k in 1..nv {
        let v = k as f32 / nv as f32;
        painter.line_segment([at(0.0, v), at(1.0, v)], line);
    }
}

fn handle(painter: &egui::Painter, p: Pos2) {
    let r = Rect::from_center_size(p, Vec2::splat(HANDLE));
    painter.rect(
        r,
        0,
        LIGHT,
        Stroke::new(pt(1.0), DARK),
        egui::StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_sizes_and_the_inside_test() {
        let mut o = PerspectiveOptions {
            width: "4 in".into(),
            height: "300".into(),
            resolution: "100".into(),
            ..Default::default()
        };
        assert_eq!(o.size(), Some((400, 300)));
        o.resolution.clear();
        assert_eq!(o.size(), None);
        let q = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 2.0),
            Pos2::new(9.0, 9.0),
            Pos2::new(1.0, 10.0),
        ];
        assert!(inside(&q, Pos2::new(5.0, 5.0)));
        assert!(!inside(&q, Pos2::new(11.0, 5.0)));
    }
}
