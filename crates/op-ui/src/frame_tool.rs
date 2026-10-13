//! Transactional Frame canvas input.
use crate::{
    document_view::{to_doc, to_screen},
    state::DocState,
};
use egui::{Key, Modifiers, Pos2, Rect, Response, Ui};
use op_core::{
    LayerId,
    frame::{self, Frame},
};
pub struct Drag {
    start: Pos2,
    current: Pos2,
    before: op_core::Snapshot,
    target: Option<(LayerId, Frame)>,
    elliptical: bool,
    resize: Option<(i8, i8)>,
}
pub fn cancel(state: &mut DocState) {
    if let Some(d) = state.frame_drag.take() {
        state.doc.restore(&d.before);
    }
}
fn preview(d: &Drag, w: u32, h: u32, mods: Modifiers) -> Frame {
    if let Some((_, old)) = d.target {
        let (x0, y0, x1, y1) = old.bounds;
        if let Some((hx, hy)) = d.resize {
            let dx = (d.current.x - d.start.x).round() as i64;
            let dy = (d.current.y - d.start.y).round() as i64;
            let (mut x0, mut y0, mut x1, mut y1) = (x0, y0, x1, y1);
            if hx < 0 {
                x0 = (x0 as i64 + dx).clamp(0, x1 as i64 - 1) as u32;
            }
            if hx > 0 {
                x1 = (x1 as i64 + dx).clamp(x0 as i64 + 1, w as i64) as u32;
            }
            if hy < 0 {
                y0 = (y0 as i64 + dy).clamp(0, y1 as i64 - 1) as u32;
            }
            if hy > 0 {
                y1 = (y1 as i64 + dy).clamp(y0 as i64 + 1, h as i64) as u32;
            }
            return Frame {
                bounds: (x0, y0, x1, y1),
                ..old
            };
        }
        let dx = (d.current.x - d.start.x).round() as i64;
        let dy = (d.current.y - d.start.y).round() as i64;
        let dx = dx.clamp(-(x0 as i64), w as i64 - x1 as i64);
        let dy = dy.clamp(-(y0 as i64), h as i64 - y1 as i64);
        return Frame {
            bounds: (
                (x0 as i64 + dx) as u32,
                (y0 as i64 + dy) as u32,
                (x1 as i64 + dx) as u32,
                (y1 as i64 + dy) as u32,
            ),
            ..old
        };
    }
    let mut end = d.current;
    if mods.shift {
        let delta = end - d.start;
        let side = delta.x.abs().max(delta.y.abs());
        end = d.start + egui::vec2(side * delta.x.signum(), side * delta.y.signum());
    }
    let r = if mods.alt {
        let delta = (end - d.start).abs();
        Rect::from_center_size(d.start, delta * 2.0)
    } else {
        Rect::from_two_pos(d.start, end)
    };
    Frame {
        bounds: (
            r.left().round().clamp(0.0, w as f32) as u32,
            r.top().round().clamp(0.0, h as f32) as u32,
            r.right().round().clamp(0.0, w as f32) as u32,
            r.bottom().round().clamp(0.0, h as f32) as u32,
        ),
        elliptical: d.elliptical,
    }
}
pub fn input(ui: &mut Ui, response: &Response, state: &mut DocState, elliptical: bool, ppp: f32) {
    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        cancel(state);
        return;
    }
    if response.contains_pointer()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = response.interact_pointer_pos()
    {
        let p = to_doc(state, p, ppp);
        if p.x >= 0.0 && p.y >= 0.0 && p.x < state.doc.width as f32 && p.y < state.doc.height as f32
        {
            let mut resize = None;
            let handle_target = state.doc.active_layer.and_then(|id| {
                state.doc.layer(id).and_then(|l| l.frame).and_then(|f| {
                    let (x0, y0, x1, y1) = f.bounds;
                    let center = egui::pos2((x0 + x1) as f32 / 2.0, (y0 + y1) as f32 / 2.0);
                    for (hx, hy) in [
                        (-1, -1),
                        (0, -1),
                        (1, -1),
                        (1, 0),
                        (1, 1),
                        (0, 1),
                        (-1, 1),
                        (-1, 0),
                    ] {
                        let q = center
                            + egui::vec2(
                                hx as f32 * (x1 - x0) as f32 / 2.0,
                                hy as f32 * (y1 - y0) as f32 / 2.0,
                            );
                        if q.distance(p) * state.view.zoom / ppp <= 8.0 {
                            resize = Some((hx, hy));
                            return Some(id);
                        }
                    }
                    None
                })
            });
            let target = handle_target
                .or_else(|| frame::at(&state.doc, p.x, p.y))
                .and_then(|id| state.doc.layer(id).and_then(|l| l.frame).map(|f| (id, f)));
            if let Some((id, _)) = target {
                state.doc.select_layer(id);
            }
            state.frame_drag = Some(Drag {
                start: p,
                current: p,
                before: state.doc.snapshot(),
                target,
                elliptical,
                resize,
            });
        }
    }
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        if let Some(d) = &mut state.frame_drag {
            d.current = p;
        }
    }
    if let Some(d) = &state.frame_drag
        && let Some((id, _)) = d.target
    {
        let f = preview(
            d,
            state.doc.width,
            state.doc.height,
            ui.input(|i| i.modifiers),
        );
        state.doc.restore(&d.before);
        frame::set(&mut state.doc, id, f, d.resize.is_none());
    }
    if ui.input(|i| i.pointer.primary_released())
        && let Some(d) = state.frame_drag.take()
    {
        let f = preview(
            &d,
            state.doc.width,
            state.doc.height,
            ui.input(|i| i.modifiers),
        );
        if let Some((_, old)) = d.target {
            if old != f {
                state.record(if d.resize.is_some() {
                    "Resize Frame"
                } else {
                    "Move Frame"
                });
            }
        } else if frame::create(&mut state.doc, f).is_some() {
            state.record("New Frame");
        }
    }
}
pub fn draw(ui: &Ui, state: &DocState, ppp: f32) {
    let mut frames: Vec<_> = state.doc.layers.iter().filter_map(|l| l.frame).collect();
    if let Some(d) = &state.frame_drag
        && d.target.is_none()
    {
        frames.push(preview(
            d,
            state.doc.width,
            state.doc.height,
            ui.input(|i| i.modifiers),
        ));
    }
    for f in frames {
        let (x0, y0, x1, y1) = f.bounds;
        let r = Rect::from_two_pos(
            to_screen(state, egui::pos2(x0 as f32, y0 as f32), ppp),
            to_screen(state, egui::pos2(x1 as f32, y1 as f32), ppp),
        );
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 150, 235));
        if f.elliptical {
            ui.painter().add(egui::epaint::EllipseShape::stroke(
                r.center(),
                r.size() / 2.0,
                stroke,
            ));
        } else {
            ui.painter()
                .rect_stroke(r, 0, stroke, egui::StrokeKind::Inside);
        }
        if state
            .doc
            .active_layer
            .and_then(|id| state.doc.layer(id))
            .and_then(|l| l.frame)
            == Some(f)
        {
            for p in [
                r.min,
                egui::pos2(r.center().x, r.top()),
                r.right_top(),
                egui::pos2(r.right(), r.center().y),
                r.max,
                egui::pos2(r.center().x, r.bottom()),
                r.left_bottom(),
                egui::pos2(r.left(), r.center().y),
            ] {
                ui.painter().rect_filled(
                    Rect::from_center_size(p, egui::vec2(5.0, 5.0)),
                    0,
                    egui::Color32::WHITE,
                );
            }
        }
        ui.painter()
            .line_segment([r.left_top(), r.right_bottom()], stroke);
        ui.painter()
            .line_segment([r.right_top(), r.left_bottom()], stroke);
    }
}
