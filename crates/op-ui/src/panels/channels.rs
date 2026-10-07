//! Channels panel: the composite and the red, green and blue channels,
//! each with its eye and thumbnail. A click targets a channel (edits reach
//! only it) and shows it alone in gray; ⌘-click loads it as a selection.

use egui::{Align2, Color32, Pos2, Rect, Sense, Ui, Vec2};

use crate::ps_icons::Icon;
use crate::state::{AppState, DocState};
use crate::theme::{color, pt};

/// A row, the eye column, the thumbnail.
const ROW: f32 = pt(37.0);
const EYE_W: f32 = pt(29.5);
const THUMB: f32 = pt(28.0);
const NAMES: [&str; 4] = ["RGB", "Red", "Green", "Blue"];
const SHORTCUTS: [&str; 4] = ["⌘2", "⌘3", "⌘4", "⌘5"];

/// Row `k` (0 the composite) targeted alone, shown alone in gray; the
/// composite targets and shows all three.
pub fn choose(state: &mut DocState, k: usize) {
    if k == 0 {
        state.channels_targeted = [true; 3];
        state.channels_shown = [true; 3];
    } else {
        let only = std::array::from_fn(|c| c + 1 == k);
        state.channels_targeted = only;
        state.channels_shown = only;
    }
}

/// Shift-click: channel `k` (1–3) joins the targeted ones or leaves them.
fn toggle(state: &mut DocState, k: usize) {
    let c = k - 1;
    let mut t = state.channels_targeted;
    t[c] = !t[c];
    if t.iter().any(|&on| on) {
        state.channels_targeted = t;
        state.channels_shown = t;
    }
}

/// ⌘-click: the channel (the composite's luminosity for RGB) as the
/// selection, recorded as "Load Selection".
pub fn load_selection(state: &mut DocState, k: usize) {
    let (w, h) = (state.doc.width, state.doc.height);
    let pixels = state.doc.composite_rgba8();
    let mask: Vec<u8> = pixels
        .chunks(4)
        .map(|p| {
            let v = if k == 0 {
                0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
            } else {
                p[k - 1] as f32
            };
            (v * p[3] as f32 / 255.0).round() as u8
        })
        .collect();
    let selection = op_core::selection::Selection::from_mask(w, h, mask, true);
    state
        .doc
        .set_selection((!selection.is_empty()).then_some(selection));
    state.record("Load Selection");
}

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let full = ui.max_rect();
    let Some(state) = app.active() else {
        return;
    };
    let mods = ui.input(|i| i.modifiers);
    let thumbs = state.channel_thumbnails(ui.ctx());
    let mut action = None;
    for k in 0..4 {
        let row = Rect::from_min_size(
            full.min + Vec2::new(0.0, k as f32 * ROW),
            Vec2::new(full.width(), ROW),
        );
        let targeted = if k == 0 {
            state.channels_targeted == [true; 3]
        } else {
            state.channels_targeted[k - 1]
        };
        let shown = if k == 0 {
            state.channels_shown == [true; 3]
        } else {
            state.channels_shown[k - 1]
        };
        let eye_rect = Rect::from_min_size(row.min, Vec2::new(EYE_W, ROW));
        let eye = ui.interact(eye_rect, ui.id().with(("channel-eye", k)), Sense::click());
        let body = Rect::from_min_max(Pos2::new(eye_rect.right() + pt(1.0), row.top()), row.max);
        let response = ui.interact(body, ui.id().with(("channel-row", k)), Sense::click());
        let painter = ui.painter();
        let fill = if targeted {
            color::ROW_SELECTED
        } else if response.hovered() {
            color::HOVER
        } else {
            color::PANEL
        };
        painter.rect_filled(body, 0, fill);
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(row.left(), row.bottom() - pt(1.0)),
                Vec2::new(row.width(), pt(1.0)),
            ),
            0,
            Color32::from_gray(0x45),
        );
        if shown {
            crate::ps_icons::paint(
                painter,
                Pos2::new(row.left() + pt(15.0), row.center().y),
                Icon::Eye,
                color::OPTIONS_ICON,
                color::PANEL,
            );
        }
        let thumb = Rect::from_min_size(
            Pos2::new(body.left() + pt(5.0), row.top() + (ROW - THUMB) / 2.0),
            Vec2::splat(THUMB),
        );
        if let Some(t) = thumbs.as_ref().map(|t| &t[k]) {
            let size = t.size_vec2();
            let scale = (THUMB / size.x).min(THUMB / size.y);
            let shown_rect = Rect::from_center_size(thumb.center(), size * scale);
            painter.image(
                t.id(),
                shown_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        painter.text(
            Pos2::new(thumb.right() + pt(10.0), row.center().y),
            Align2::LEFT_CENTER,
            NAMES[k],
            crate::theme::body(),
            color::TEXT,
        );
        painter.text(
            Pos2::new(row.right() - pt(16.0), row.center().y),
            Align2::RIGHT_CENTER,
            SHORTCUTS[k],
            crate::theme::body(),
            color::TEXT_DIM,
        );
        if eye.clicked() {
            action = Some(Action::Eye(k));
        } else if response.clicked() {
            action = Some(if mods.command {
                Action::Load(k)
            } else if mods.shift && k > 0 {
                Action::Toggle(k)
            } else {
                Action::Choose(k)
            });
        }
    }
    match action {
        Some(Action::Choose(k)) => choose(state, k),
        Some(Action::Toggle(k)) => toggle(state, k),
        Some(Action::Load(k)) => load_selection(state, k),
        Some(Action::Eye(0)) => state.channels_shown = [true; 3],
        Some(Action::Eye(k)) => {
            let mut s = state.channels_shown;
            s[k - 1] = !s[k - 1];
            if s.iter().any(|&on| on) {
                state.channels_shown = s;
            }
        }
        None => {}
    }
}

enum Action {
    Choose(usize),
    Toggle(usize),
    Load(usize),
    Eye(usize),
}
