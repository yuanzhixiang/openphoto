//! Count-group options and transactional count-marker input.
use crate::{
    document_view::{to_doc, to_screen},
    state::{AppState, DocState},
};
use egui::{Key, Modifiers, Pos2, Response, Ui};
#[derive(Clone, Copy, Debug)]
pub struct Drag {
    group: Option<u32>,
    point: Option<usize>,
    original: Pos2,
    current: Pos2,
}
#[derive(Clone, Copy)]
pub struct Options {
    pub marker: u32,
    pub label: u32,
    pub color: [u8; 3],
}
pub fn selected(state: &DocState) -> Option<u32> {
    state
        .count_group
        .filter(|id| state.doc.count_groups.iter().any(|g| g.id == *id))
        .or_else(|| state.doc.count_groups.first().map(|g| g.id))
}
pub fn options(app: &AppState) -> Options {
    let group = app
        .active_doc
        .and_then(|id| app.docs.get(&id))
        .and_then(|s| selected(s).and_then(|id| s.doc.count_groups.iter().find(|g| g.id == id)));
    let size = |key, default, max| {
        app.tool_settings
            .get(key)
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map_or(default, |v| v.round().clamp(1.0, max as f32) as u32)
    };
    let default = group.map_or([0x9c, 0xf2, 0xf4], |g| g.color);
    let color = app
        .tool_settings
        .get("count.color")
        .and_then(|v| {
            v.split(',')
                .map(|v| v.parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .ok()
        })
        .and_then(|v| v.try_into().ok())
        .unwrap_or(default);
    Options {
        marker: size("count.marker", group.map_or(2, |g| g.marker), 100),
        label: size("count.label", group.map_or(8, |g| g.label), 100),
        color,
    }
}
pub fn sync(app: &mut AppState) {
    let current = app
        .active_doc
        .and_then(|id| app.docs.get(&id))
        .and_then(|s| {
            selected(s).and_then(|id| {
                s.doc
                    .count_groups
                    .iter()
                    .find(|g| g.id == id)
                    .map(|g| (s.doc.id, g.clone()))
            })
        });
    let Some((doc, g)) = current else {
        return;
    };
    let key = format!("{}:{}:{}:{}:{:?}", doc.0, g.id, g.marker, g.label, g.color);
    if app.tool_settings.get("count.synced") != Some(&key) {
        app.tool_settings.insert("count.synced", key);
        app.tool_settings
            .insert("count.marker", g.marker.to_string());
        app.tool_settings.insert("count.label", g.label.to_string());
        app.tool_settings.insert(
            "count.color",
            format!("{},{},{}", g.color[0], g.color[1], g.color[2]),
        );
    }
}
pub fn new_group(app: &mut AppState) {
    if let Some(s) = app.active()
        && let Some(id) = op_core::annotation::create_count_group(&mut s.doc)
    {
        s.count_group = Some(id);
        s.record("New Count Group");
    }
    sync(app);
}
pub fn group_action(app: &mut AppState, action: &str) {
    let Some(s) = app.active() else {
        return;
    };
    let Some(id) = selected(s) else {
        return;
    };
    match action {
        "visibility" => {
            if let Some(g) = s.doc.count_groups.iter_mut().find(|g| g.id == id) {
                g.visible = !g.visible;
                s.doc.mark_dirty();
                s.record("Count Visibility");
            }
        }
        "delete" => {
            s.doc.count_groups.retain(|g| g.id != id);
            s.count_group = s.doc.count_groups.first().map(|g| g.id);
            s.count_drag = None;
            s.doc.mark_dirty();
            s.record("Delete Count Group");
        }
        "clear" => {
            if let Some(g) = s.doc.count_groups.iter_mut().find(|g| g.id == id)
                && !g.points.is_empty()
            {
                g.points.clear();
                s.doc.mark_dirty();
                s.record("Clear Counts");
            }
        }
        _ => {}
    }
    sync(app);
}
pub fn apply(state: &mut DocState, options: Options) {
    if let Some(id) = selected(state)
        && let Some(g) = state.doc.count_groups.iter_mut().find(|g| g.id == id)
        && (g.marker != options.marker || g.label != options.label || g.color != options.color)
    {
        g.marker = options.marker;
        g.label = options.label;
        g.color = options.color;
        state.doc.mark_dirty();
        state.record("Count Options");
    }
}
pub fn input(ui: &mut Ui, response: &Response, state: &mut DocState, options: Options, ppp: f32) {
    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        state.count_drag = None;
        state.count_name_edit = None;
        return;
    }
    if !ui.ctx().egui_wants_keyboard_input() {
        apply(state, options);
    }
    if response.contains_pointer()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = response.interact_pointer_pos()
    {
        let p = to_doc(state, p, ppp);
        if p.x >= 0.0 && p.y >= 0.0 && p.x < state.doc.width as f32 && p.y < state.doc.height as f32
        {
            let hit = state
                .doc
                .count_groups
                .iter()
                .rev()
                .filter(|g| g.visible)
                .find_map(|g| {
                    g.points
                        .iter()
                        .enumerate()
                        .find(|(_, q)| {
                            egui::pos2(q.0 as f32, q.1 as f32).distance(p) * state.view.zoom / ppp
                                < 8.0
                        })
                        .map(|(i, q)| (g.id, i, *q))
                });
            if let Some((id, i, q)) = hit {
                state.count_group = Some(id);
                if ui.input(|i| i.modifiers.alt) {
                    if let Some(g) = state.doc.count_groups.iter_mut().find(|g| g.id == id) {
                        g.points.remove(i);
                        state.doc.mark_dirty();
                        state.record("Delete Count");
                    }
                } else {
                    state.count_drag = Some(Drag {
                        group: Some(id),
                        point: Some(i),
                        original: egui::pos2(q.0 as f32, q.1 as f32),
                        current: p,
                    });
                }
            } else if !ui.input(|i| i.modifiers.alt) {
                state.count_drag = Some(Drag {
                    group: selected(state),
                    point: None,
                    original: p,
                    current: p,
                });
            }
        }
    }
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        if let Some(d) = &mut state.count_drag {
            d.current = egui::pos2(
                p.x.round()
                    .clamp(0.0, state.doc.width.saturating_sub(1) as f32),
                p.y.round()
                    .clamp(0.0, state.doc.height.saturating_sub(1) as f32),
            );
        }
    }
    if ui.input(|i| i.pointer.primary_released())
        && let Some(d) = state.count_drag.take()
    {
        let id = d
            .group
            .or_else(|| op_core::annotation::create_count_group(&mut state.doc));
        let Some(id) = id else {
            return;
        };
        state.count_group = Some(id);
        if let Some(g) = state.doc.count_groups.iter_mut().find(|g| g.id == id) {
            if let Some(i) = d.point {
                if i < g.points.len() && d.current.distance(d.original) > 0.5 {
                    g.points[i] = (d.current.x as u32, d.current.y as u32);
                    state.doc.mark_dirty();
                    state.record("Move Count");
                }
            } else {
                g.marker = options.marker;
                g.label = options.label;
                g.color = options.color;
                g.points.push((d.current.x as u32, d.current.y as u32));
                state.doc.mark_dirty();
                state.record("New Count");
            }
        }
    }
}
pub fn draw(ui: &Ui, state: &DocState, ppp: f32) {
    for g in state.doc.count_groups.iter().filter(|g| g.visible) {
        let color = egui::Color32::from_rgb(g.color[0], g.color[1], g.color[2]);
        for (i, q) in g.points.iter().enumerate() {
            let point = state
                .count_drag
                .filter(|d| d.group == Some(g.id) && d.point == Some(i))
                .map_or(egui::pos2(q.0 as f32, q.1 as f32), |d| d.current);
            let p = to_screen(state, point, ppp);
            ui.painter().circle_filled(p, g.marker as f32 / 2.0, color);
            ui.painter().text(
                p + egui::vec2(4.0, -4.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{}", i + 1),
                egui::FontId::proportional(g.label as f32),
                color,
            );
        }
    }
}

pub fn commit_name(state: &mut DocState) {
    let Some((id, name)) = state.count_name_edit.take() else {
        return;
    };
    let name = name.trim();
    if !name.is_empty()
        && let Some(g) = state.doc.count_groups.iter_mut().find(|g| g.id == id)
        && g.name != name
    {
        g.name = name.into();
        state.doc.mark_dirty();
        state.record("Rename Count Group");
    }
}
pub fn editor(ui: &mut Ui, state: &mut DocState) {
    let Some((_, name)) = &mut state.count_name_edit else {
        return;
    };
    let mut open = true;
    let mut done = false;
    let mut cancel = false;
    egui::Window::new("Rename Count Group")
        .id(ui.id().with("rename-count-group"))
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.text_edit_singleline(name);
            ui.horizontal(|ui| {
                done = ui
                    .add_enabled(!name.trim().is_empty(), egui::Button::new("OK"))
                    .clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
    if cancel || !open {
        state.count_name_edit = None;
    } else if done {
        commit_name(state);
    }
}
