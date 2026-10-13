//! Windowless-testable Artboard canvas gestures and option actions.
use crate::{
    document_view::{to_doc, to_screen},
    state::{AppState, DocState, View},
    theme::{color, pt},
};
use egui::{Key, PointerButton, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};
use op_core::{
    LayerId, Snapshot,
    artboard::{self, Artboard},
};

#[derive(Clone, Copy)]
pub struct Options {
    pub width: u32,
    pub height: u32,
    pub background: Option<[u8; 3]>,
    pub auto_nest: bool,
}
pub fn options(app: &AppState) -> Options {
    let dim = |key, default| {
        app.tool_settings
            .get(key)
            .and_then(|v| crate::options_bar::typed_number(v))
            .unwrap_or(default)
            .round()
            .clamp(1.0, 30000.0) as u32
    };
    Options {
        width: dim("artboard.w", 750.0),
        height: dim("artboard.h", 1334.0),
        background: match app
            .tool_settings
            .get("artboard.bg")
            .map(String::as_str)
            .unwrap_or("0")
        {
            "1" => Some([0; 3]),
            "2" => None,
            "3" => Some(custom_color(app)),
            _ => Some([255; 3]),
        },
        auto_nest: app
            .tool_settings
            .get("artboard.auto_nest")
            .is_none_or(|v| v != "0"),
    }
}
pub fn custom_color(app: &AppState) -> [u8; 3] {
    let value = app
        .tool_settings
        .get("artboard.color")
        .map(String::as_str)
        .unwrap_or("ffffff");
    let n = u32::from_str_radix(value, 16).unwrap_or(0xffffff);
    [(n >> 16) as u8, (n >> 8) as u8, n as u8]
}
pub fn set_custom_color(app: &mut AppState, [r, g, b]: [u8; 3]) {
    app.tool_settings.insert("artboard.bg", "3".into());
    app.tool_settings
        .insert("artboard.color", format!("{r:02x}{g:02x}{b:02x}"));
    apply_options(app);
}

pub fn active_board(app: &AppState) -> Option<Artboard> {
    let state = app.active_doc.and_then(|id| app.docs.get(&id))?;
    state
        .doc
        .active_layer
        .and_then(|id| state.doc.layer(id))?
        .artboard
}
pub fn apply_options(app: &mut AppState) {
    let opts = options(app);
    if let Some(state) = app.active()
        && let Some(id) = state.doc.active_layer
        && let Some(b) = state.doc.layer(id).and_then(|l| l.artboard)
        && artboard::set(
            &mut state.doc,
            id,
            Artboard {
                width: opts.width,
                height: opts.height,
                background: opts.background,
                ..b
            },
            false,
        )
    {
        state.record("Artboard Options");
    }
}
pub fn preset(app: &mut AppState, index: usize) {
    let sizes = [
        (750, 1334),
        (750, 1334),
        (1080, 1920),
        (1125, 2436),
        (2048, 2732),
        (1280, 800),
        (1920, 1080),
    ];
    if index == 0 {
        return;
    }
    let (w, h) = sizes[index.min(sizes.len() - 1)];
    app.tool_settings.insert("artboard.w", format!("{w} px"));
    app.tool_settings.insert("artboard.h", format!("{h} px"));
    apply_options(app);
}
pub fn add(app: &mut AppState) {
    let opts = options(app);
    if let Some(state) = app.active() {
        let b = state
            .doc
            .active_layer
            .and_then(|id| state.doc.layer(id))
            .and_then(|l| l.artboard);
        let (x, y) = b.map_or((0, 0), |b| (b.x + b.width as i64 + 20, b.y));
        if artboard::create(
            &mut state.doc,
            Artboard {
                x,
                y,
                width: opts.width,
                height: opts.height,
                background: opts.background,
            },
            false,
        )
        .is_some()
        {
            state.record("New Artboard");
        }
    }
}
pub fn orient(app: &mut AppState, landscape: bool) {
    let opts = options(app);
    let (w, h) = if landscape {
        (opts.width.max(opts.height), opts.width.min(opts.height))
    } else {
        (opts.width.min(opts.height), opts.width.max(opts.height))
    };
    app.tool_settings.insert("artboard.w", format!("{w} px"));
    app.tool_settings.insert("artboard.h", format!("{h} px"));
    app.tool_settings.insert("artboard.size", "0".into());
    apply_options(app);
}

#[derive(Clone, Copy)]
enum Kind {
    Create,
    Move(LayerId, Artboard),
    Copy(LayerId, Artboard),
    Resize(LayerId, Artboard, i8, i8),
}
pub struct Drag {
    before: Snapshot,
    selected: Vec<LayerId>,
    view: View,
    start: Pos2,
    kind: Kind,
    changed: bool,
}

pub fn input(ui: &Ui, response: &egui::Response, state: &mut DocState, opts: Options, ppp: f32) {
    if state.artboard_drag.is_some()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape))
    {
        let drag = state.artboard_drag.take().expect("drag");
        state.doc.restore(&drag.before);
        state.doc.set_selected_layers(drag.selected);
        state.view = drag.view;
        return;
    }
    let pressed = ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
    if pressed
        && state.artboard_drag.is_none()
        && let Some(board) = state
            .doc
            .active_layer
            .and_then(|id| state.doc.layer(id))
            .and_then(|l| l.artboard)
        && ui.input(|i| {
            i.pointer.press_origin().is_some_and(|p| {
                let rect = Rect::from_two_pos(
                    to_screen(state, Pos2::new(board.x as f32, board.y as f32), ppp),
                    to_screen(
                        state,
                        Pos2::new(
                            (board.x + board.width as i64) as f32,
                            (board.y + board.height as i64) as f32,
                        ),
                        ppp,
                    ),
                );
                [
                    rect.left_center() - Vec2::new(pt(22.0), 0.0),
                    rect.right_center() + Vec2::new(pt(22.0), 0.0),
                    rect.center_top() - Vec2::new(0.0, pt(30.0)),
                    rect.center_bottom() + Vec2::new(0.0, pt(22.0)),
                ]
                .into_iter()
                .any(|center| Rect::from_center_size(center, Vec2::splat(pt(18.0))).contains(p))
            })
        })
    {
        return;
    }
    if response.contains_pointer()
        && pressed
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        let d = to_doc(state, p, ppp);
        let tolerance = pt(7.0) * ppp / state.view.zoom;
        let selected = state.doc.active_layer.and_then(|id| {
            state
                .doc
                .layer(id)
                .and_then(|l| l.artboard)
                .map(|b| (id, b))
        });
        let handle = selected.and_then(|(id, b)| {
            let rect = Rect::from_min_size(
                Pos2::new(b.x as f32, b.y as f32),
                Vec2::new(b.width as f32, b.height as f32),
            );
            if !rect.expand(tolerance).contains(d) {
                return None;
            }
            let x = if (d.x - rect.left()).abs() < tolerance {
                -1
            } else if (d.x - rect.right()).abs() < tolerance {
                1
            } else {
                0
            };
            let y = if (d.y - rect.top()).abs() < tolerance {
                -1
            } else if (d.y - rect.bottom()).abs() < tolerance {
                1
            } else {
                0
            };
            (x != 0 || y != 0).then_some(Kind::Resize(id, b, x, y))
        });
        let kind = handle
            .or_else(|| {
                artboard::at(&state.doc, d.x.round() as i64, d.y.round() as i64)
                    .and_then(|id| state.doc.layer(id)?.artboard.map(|b| Kind::Move(id, b)))
            })
            .unwrap_or(Kind::Create);
        if let Kind::Move(id, _) | Kind::Resize(id, _, _, _) = kind {
            state.doc.select_layer(id);
        }
        let kind = if ui.input(|i| i.modifiers.alt) {
            if let Kind::Move(id, b) = kind {
                Kind::Copy(id, b)
            } else {
                kind
            }
        } else {
            kind
        };
        state.artboard_drag = Some(Drag {
            before: state.doc.snapshot(),
            selected: state.doc.selected_layers(),
            view: state.view,
            start: d,
            kind,
            changed: false,
        });
    }
    let Some(mut drag) = state.artboard_drag.take() else {
        return;
    };
    let pointer = ui.input(|i| i.pointer.interact_pos());
    // Recompute against the original document and view; canvas expansion cannot accumulate drift.
    state.doc.restore(&drag.before);
    state.doc.set_selected_layers(drag.selected.clone());
    state.view = drag.view;
    if let Some(pointer) = pointer {
        let point = to_doc(state, pointer, ppp);
        let mut delta = point - drag.start;
        if matches!(drag.kind, Kind::Move(..) | Kind::Copy(..)) && ui.input(|i| i.modifiers.shift) {
            let (x, y) = op_core::move_tool::constrain_offset(delta.x, delta.y);
            delta = Vec2::new(x, y);
        }
        let (dx, dy) = (delta.x.round() as i64, delta.y.round() as i64);
        let old_size = Vec2::new(state.doc.width as f32, state.doc.height as f32);
        drag.changed = match drag.kind {
            Kind::Create if delta.length() > 3.0 => {
                let mut width = dx.unsigned_abs().clamp(1, 30000) as u32;
                let mut height = dy.unsigned_abs().clamp(1, 30000) as u32;
                if ui.input(|i| i.modifiers.shift) {
                    let side = width.max(height);
                    width = side;
                    height = side;
                }
                artboard::create(
                    &mut state.doc,
                    Artboard {
                        x: drag.start.x.round() as i64 + dx.min(0),
                        y: drag.start.y.round() as i64 + dy.min(0),
                        width,
                        height,
                        background: opts.background,
                    },
                    opts.auto_nest,
                )
                .is_some()
            }
            Kind::Create => false,
            Kind::Copy(id, b) if dx != 0 || dy != 0 => {
                if state.doc.position_locked(id) {
                    false
                } else {
                    state.doc.select_layer(id);
                    let name =
                        op_core::layer_ops::duplicate_name(&state.doc).expect("artboard selected");
                    op_core::layer_ops::duplicate_named(&mut state.doc, &name).is_some_and(|copy| {
                        artboard::set(
                            &mut state.doc,
                            copy,
                            Artboard {
                                x: b.x + dx,
                                y: b.y + dy,
                                ..b
                            },
                            true,
                        )
                    })
                }
            }
            Kind::Copy(..) => false,
            Kind::Move(id, b) => artboard::set(
                &mut state.doc,
                id,
                Artboard {
                    x: b.x + dx,
                    y: b.y + dy,
                    ..b
                },
                true,
            ),
            Kind::Resize(id, b, x, y) => {
                let x0 = b.x + if x < 0 { dx } else { 0 };
                let y0 = b.y + if y < 0 { dy } else { 0 };
                let x1 = b.x + b.width as i64 + if x > 0 { dx } else { 0 };
                let y1 = b.y + b.height as i64 + if y > 0 { dy } else { 0 };
                if x1 > x0 && y1 > y0 {
                    artboard::set(
                        &mut state.doc,
                        id,
                        Artboard {
                            x: x0,
                            y: y0,
                            width: (x1 - x0) as u32,
                            height: (y1 - y0) as u32,
                            ..b
                        },
                        false,
                    )
                } else {
                    false
                }
            }
        };
        if !drag.changed {
            state.doc.restore(&drag.before);
            state.doc.set_selected_layers(drag.selected.clone());
        }
        let mut shift = Vec2::ZERO;
        if drag.changed
            && let Some(actual) = state
                .doc
                .active_layer
                .and_then(|id| state.doc.layer(id))
                .and_then(|l| l.artboard)
        {
            let (x, y) = match drag.kind {
                Kind::Create => (
                    drag.start.x.round() as i64 + dx.min(0),
                    drag.start.y.round() as i64 + dy.min(0),
                ),
                Kind::Move(_, b) | Kind::Copy(_, b) => (b.x + dx, b.y + dy),
                Kind::Resize(_, b, x, y) => (
                    b.x + if x < 0 { dx } else { 0 },
                    b.y + if y < 0 { dy } else { 0 },
                ),
            };
            shift = Vec2::new((x - actual.x) as f32, (y - actual.y) as f32);
        }
        state.view.offset +=
            ((Vec2::new(state.doc.width as f32, state.doc.height as f32) - old_size) * 0.5 + shift)
                * state.view.zoom
                / ppp;
    }
    if !ui.input(|i| i.pointer.primary_down()) {
        if drag.changed {
            state.record(match drag.kind {
                Kind::Create => "New Artboard",
                Kind::Move(..) => "Move Artboard",
                Kind::Copy(..) => "Duplicate Artboard",
                Kind::Resize(..) => "Resize Artboard",
            });
        }
    } else {
        state.artboard_drag = Some(drag);
    }
}

pub fn paint(ui: &Ui, state: &mut DocState, _opts: Options, ppp: f32) {
    let painter = ui.painter_at(state.view.viewport);
    let mut adjacent = None;
    for layer in state.doc.layers.iter().filter(|l| state.doc.is_shown(l.id)) {
        let Some(b) = layer.artboard else {
            continue;
        };
        let p = |x, y| to_screen(state, Pos2::new(x as f32, y as f32), ppp);
        let rect = Rect::from_two_pos(p(b.x, b.y), p(b.x + b.width as i64, b.y + b.height as i64));
        let selected = state.doc.active_layer == Some(layer.id);
        painter.rect_stroke(
            rect,
            0,
            Stroke::new(
                pt(1.0),
                if selected {
                    color::TEXT_BRIGHT
                } else {
                    color::TEXT_DIM
                },
            ),
            StrokeKind::Outside,
        );
        painter.text(
            rect.left_top() - Vec2::new(0.0, pt(6.0)),
            egui::Align2::LEFT_BOTTOM,
            &layer.name,
            crate::theme::body(),
            color::TEXT_DIM,
        );
        if selected && state.artboard_drag.is_none() {
            for (label, position, dx, dy) in [
                (
                    "Add artboard left",
                    rect.left_center() - Vec2::new(pt(22.0), 0.0),
                    -(b.width as i64) - 20,
                    0,
                ),
                (
                    "Add artboard right",
                    rect.right_center() + Vec2::new(pt(22.0), 0.0),
                    b.width as i64 + 20,
                    0,
                ),
                (
                    "Add artboard above",
                    rect.center_top() - Vec2::new(0.0, pt(30.0)),
                    0,
                    -(b.height as i64) - 20,
                ),
                (
                    "Add artboard below",
                    rect.center_bottom() + Vec2::new(0.0, pt(22.0)),
                    0,
                    b.height as i64 + 20,
                ),
            ] {
                let hit = Rect::from_center_size(position, Vec2::splat(pt(18.0)));
                let response =
                    ui.interact(hit, ui.id().with((label, layer.id.0)), egui::Sense::click());
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label)
                });
                painter.rect_filled(hit, pt(9.0), color::TAB_ACTIVE);
                painter.text(
                    position,
                    egui::Align2::CENTER_CENTER,
                    "+",
                    crate::theme::body(),
                    color::TEXT_BRIGHT,
                );
                if response.clicked() {
                    adjacent = Some(Artboard {
                        x: b.x + dx,
                        y: b.y + dy,
                        ..b
                    });
                }
            }
        }
        if selected {
            for point in [
                rect.left_top(),
                rect.center_top(),
                rect.right_top(),
                rect.left_center(),
                rect.right_center(),
                rect.left_bottom(),
                rect.center_bottom(),
                rect.right_bottom(),
            ] {
                painter.rect_filled(
                    Rect::from_center_size(point, Vec2::splat(pt(5.0))),
                    0,
                    color::TEXT_BRIGHT,
                );
            }
        }
    }
    if let Some(board) = adjacent
        && artboard::create(&mut state.doc, board, false).is_some()
    {
        state.record("New Artboard");
    }
}

pub fn sync_options(app: &mut AppState) {
    if let Some(b) = active_board(app) {
        let presets = [
            (750, 1334),
            (1080, 1920),
            (1125, 2436),
            (2048, 2732),
            (1280, 800),
            (1920, 1080),
        ];
        let size = presets
            .iter()
            .position(|&(w, h)| w == b.width && h == b.height)
            .map_or(0, |i| i + 1);
        app.tool_settings.insert("artboard.size", size.to_string());
        app.tool_settings
            .insert("artboard.w", format!("{} px", b.width));
        app.tool_settings
            .insert("artboard.h", format!("{} px", b.height));
        if let Some([r, g, blue]) = b.background {
            app.tool_settings
                .insert("artboard.color", format!("{r:02x}{g:02x}{blue:02x}"));
        }
        app.tool_settings.insert(
            "artboard.bg",
            match b.background {
                None => "2",
                Some([0, 0, 0]) => "1",
                Some([255, 255, 255]) => "0",
                Some(_) => "3",
            }
            .into(),
        );
    }
}

pub fn align(app: &mut AppState, horizontal: bool) {
    let Some(state) = app.active() else {
        return;
    };
    let boards: Vec<_> = state
        .doc
        .selected_layers()
        .into_iter()
        .filter_map(|id| state.doc.layer(id)?.artboard.map(|b| (id, b)))
        .collect();
    if boards.len() < 2 {
        return;
    }
    let edge = boards
        .iter()
        .map(|(_, b)| if horizontal { b.x } else { b.y })
        .min()
        .expect("selected boards");
    let mut changed = false;
    for (id, b) in boards {
        changed |= artboard::set(
            &mut state.doc,
            id,
            if horizontal {
                Artboard { x: edge, ..b }
            } else {
                Artboard { y: edge, ..b }
            },
            true,
        );
    }
    if changed {
        state.record("Align Artboards");
    }
}
