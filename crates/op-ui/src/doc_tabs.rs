//! Document tab bar above the canvas, measured from Photoshop 2026 at 1:1.

use egui::{Color32, Rect, Sense, Stroke, Ui, pos2, vec2};

use crate::document_view;
use crate::state::{AppState, DocState};
use crate::theme::{self, color, pt};

/// Height of the bar, including its 1 pt bottom line.
pub const HEIGHT: f32 = pt(30.0);
const CLOSE_X: f32 = pt(11.0);
const TEXT_X: f32 = pt(24.0);
const RIGHT_PAD: f32 = pt(12.0);
const FONT: f32 = pt(12.0);
const BOTTOM_LINE: Color32 = Color32::from_gray(0x36);
const TAB_EDGE: Color32 = Color32::from_gray(0x40);
const CLOSE: Color32 = Color32::from_gray(0x8c);

/// Photoshop's tab title: `name @ zoom (mode/bits)`. A `#` after the bit
/// depth marks a document without an embedded color profile, and a
/// trailing ` *` unsaved changes.
pub fn title(state: &DocState) -> String {
    let d = &state.doc;
    format!(
        "{} @ {} ({}{}/{}{}){}",
        d.title,
        document_view::zoom_label(state.view.zoom),
        // Photoshop 2026 names its temporary layer while cropping
        if crate::crop_tool::previewing(state) {
            "Crop Preview, "
        } else {
            ""
        },
        // Photoshop shows the channel being edited in Quick Mask
        if d.quick_mask.is_some() {
            "Quick Mask"
        } else {
            d.color_mode.short()
        },
        d.bit_depth.bits(),
        if state.untagged { "#" } else { "" },
        if state.is_dirty() { " *" } else { "" },
    )
}

/// Draws the tabs into `rect`. Clicking a tab activates it; its "×" closes
/// the document.
pub fn show(ui: &mut Ui, app: &mut AppState, rect: Rect) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0, color::TAB_BAR);

    let font = theme::semibold(FONT);
    let mut x = rect.left();
    let mut close = None;
    for id in app.doc_order.clone() {
        let Some(state) = app.docs.get(&id) else {
            continue;
        };
        let galley = painter.layout_no_wrap(title(state), font.clone(), color::TEXT);
        let width = TEXT_X + galley.size().x + RIGHT_PAD;
        let tab = Rect::from_min_size(pos2(x, rect.top()), vec2(width, rect.height() - 1.0));
        let active = app.active_doc == Some(id);

        let response = ui.interact(tab, ui.id().with(("doc-tab", id.0)), Sense::click());
        let close_rect = Rect::from_center_size(
            pos2(tab.left() + CLOSE_X, tab.center().y),
            egui::Vec2::splat(pt(12.0)),
        );
        let close_response = ui.interact(
            close_rect,
            ui.id().with(("doc-tab-close", id.0)),
            Sense::click(),
        );

        if active {
            painter.rect_filled(tab, 0, color::TAB_ACTIVE);
        } else if response.hovered() {
            painter.rect_filled(tab, 0, color::TAB_INACTIVE);
        }
        painter.line_segment(
            [
                tab.right_top() + vec2(-0.5, 0.0),
                tab.right_bottom() + vec2(-0.5, 0.0),
            ],
            Stroke::new(1.0, TAB_EDGE),
        );

        let text = if active { color::TEXT } else { color::TEXT_DIM };
        painter.galley(
            pos2(tab.left() + TEXT_X, tab.center().y - galley.size().y / 2.0),
            galley,
            text,
        );

        // The close cross is a thin 6 pt "×"
        let c = close_rect.center();
        let r = pt(3.0);
        let tint = if close_response.hovered() {
            color::TEXT
        } else {
            CLOSE
        };
        let stroke = Stroke::new(pt(1.0), tint);
        painter.line_segment([c + vec2(-r, -r), c + vec2(r, r)], stroke);
        painter.line_segment([c + vec2(-r, r), c + vec2(r, -r)], stroke);

        if close_response.clicked() {
            close = Some(id);
        } else if response.clicked() {
            app.active_doc = Some(id);
        }
        x = tab.right();
    }
    painter.line_segment(
        [
            rect.left_bottom() - vec2(0.0, 0.5),
            rect.right_bottom() - vec2(0.0, 0.5),
        ],
        Stroke::new(1.0, BOTTOM_LINE),
    );
    if let Some(id) = close {
        crate::actions::request_close(app, vec![id]);
    }
}

/// Keeps the active document valid after documents were added or closed.
pub fn ensure_active(app: &mut AppState) {
    if app.active_doc.is_none_or(|id| !app.docs.contains_key(&id)) {
        app.active_doc = app.doc_order.last().copied();
    }
}
