//! Reusable widgets: icon buttons, read-only fields, separators, etc.

use egui::{Align2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::{self, color};

/// Square icon button. Draws a dark background when `selected` (current tool).
pub fn icon_button(ui: &mut Ui, icon: &str, size: f32, selected: bool) -> Response {
    icon_button_sized(ui, icon, Vec2::splat(size), theme::font::ICON, selected)
}

pub fn icon_button_sized(
    ui: &mut Ui,
    icon: &str,
    size: Vec2,
    icon_size: f32,
    selected: bool,
) -> Response {
    icon_button_font(ui, icon, size, theme::icon(icon_size), selected)
}

/// An icon button drawing its icon in `font`.
pub fn icon_button_font(
    ui: &mut Ui,
    icon: &str,
    size: Vec2,
    font: egui::FontId,
    selected: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if selected || response.is_pointer_button_down_on() {
            color::TOOL_ACTIVE
        } else if response.hovered() {
            color::HOVER
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, CornerRadius::same(4), fill);
        let tint = if ui.is_enabled() {
            color::ICON
        } else {
            color::TEXT_DISABLED
        };
        ui.painter()
            .text(rect.center(), Align2::CENTER_CENTER, icon, font, tint);
    }
    response
}

/// Photoshop's checkbox: a 10 pt light-gray rounded square with a dark
/// check mark when on, an outlined square when off, the label 7.5 pt to
/// its right. Clicking the box or the label toggles it.
pub fn checkbox(ui: &mut Ui, checked: &mut bool, label: &str) -> Response {
    use theme::pt;
    let enabled = ui.is_enabled();
    let font = theme::body();
    // Disabled, Photoshop dims the label to #878787
    let text_color = if enabled {
        color::TEXT_BRIGHT
    } else {
        Color32::from_gray(0x87)
    };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font, text_color);
    let gap = if label.is_empty() { 0.0 } else { pt(8.0) };
    let size = Vec2::new(
        pt(10.0) + gap + galley.size().x,
        theme::size::FIELD_HEIGHT.max(galley.size().y),
    );
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let b = Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - pt(5.0)),
            Vec2::splat(pt(10.0)),
        );
        if !enabled {
            // Measured: a #4d4d4d box in a 1 pt #5e5e5e frame
            painter.rect(
                b,
                CornerRadius::same(pt(2.5) as u8),
                Color32::from_gray(0x4d),
                Stroke::new(pt(1.0), Color32::from_gray(0x5e)),
                StrokeKind::Inside,
            );
        }
        let fill = if !enabled {
            Color32::from_gray(0x5e)
        } else if response.hovered() {
            Color32::from_gray(0xe6)
        } else {
            color::CHECKBOX
        };
        if *checked {
            painter.rect_filled(b, CornerRadius::same(pt(2.5) as u8), fill);
            let p = |x: f32, y: f32| b.min + Vec2::new(pt(x), pt(y));
            painter.add(egui::Shape::line(
                vec![p(2.75, 4.75), p(4.5, 6.75), p(8.0, 2.75)],
                Stroke::new(pt(1.7), color::CHECK_MARK),
            ));
        } else if enabled {
            painter.rect_stroke(
                b.shrink(pt(0.5)),
                CornerRadius::same(pt(2.5) as u8),
                Stroke::new(pt(1.0), fill),
                StrokeKind::Middle,
            );
        }
        painter.galley(
            egui::pos2(
                rect.left() + pt(10.0) + gap,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            text_color,
        );
    }
    response
}

/// Photoshop's options-bar dropdown: a dark rounded field with a 1 pt
/// `#666666` border, the value on the left and a chevron on the right.
/// `menu` fills the popup.
pub fn dropdown(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    width: f32,
    text: &str,
    menu: impl FnOnce(&mut Ui),
) -> Response {
    let enabled = ui.is_enabled();
    dropdown_with(ui, id, width, text, enabled, menu)
}

/// [`dropdown`] that is drawn disabled (Photoshop's `#4d4d4d` field,
/// `#5e5e5e` border, `#878787` value) when `enabled` is false, without
/// egui's fading of disabled widgets; it then doesn't open.
pub fn dropdown_with(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    width: f32,
    text: &str,
    enabled: bool,
    menu: impl FnOnce(&mut Ui),
) -> Response {
    let response = dropdown_button(ui, width, text, enabled);
    if enabled {
        egui::Popup::menu(&response)
            .id(egui::Id::new(id))
            .show(menu);
    }
    response
}

/// The dropdown's box, value and chevron.
fn dropdown_button(ui: &mut Ui, width: f32, text: &str, enabled: bool) -> Response {
    use theme::pt;
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, pt(18.5)), sense);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, border, tint, chevron) = if enabled {
            (
                color::FIELD,
                if response.hovered() {
                    color::DROPDOWN_BORDER_HOVER
                } else {
                    color::DROPDOWN_BORDER
                },
                color::TEXT_BRIGHT,
                color::OPTIONS_ICON,
            )
        } else {
            (
                color::LIST_BG,
                Color32::from_gray(0x5e),
                Color32::from_gray(0x87),
                Color32::from_gray(0x6a),
            )
        };
        painter.rect(
            rect,
            CornerRadius::same(pt(2.5) as u8),
            fill,
            Stroke::new(pt(1.0), border),
            StrokeKind::Inside,
        );
        painter.text(
            rect.left_center() + Vec2::new(pt(7.0), 0.0),
            Align2::LEFT_CENTER,
            text,
            theme::body(),
            tint,
        );
        crate::ps_icons::paint(
            painter,
            egui::pos2(rect.right() - pt(7.75), rect.center().y + pt(0.25)),
            crate::ps_icons::Icon::Caret,
            chevron,
            fill,
        );
    }
    response
}

/// [`dropdown_with`] opening `entries` as a native macOS menu, as
/// Photoshop's options bar does (`native_popup`); returns the index of the
/// entry picked.
pub fn dropdown_entries(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    width: f32,
    text: &str,
    enabled: bool,
    entries: &[crate::native_popup::Entry],
) -> Option<usize> {
    let response = dropdown_button(ui, width, text, enabled);
    if !enabled {
        return None;
    }
    crate::native_popup::dropdown(ui, &response, egui::Id::new(id), entries)
}

/// An options-bar button drawing one of the traced Photoshop icons.
pub fn ps_icon_button(
    ui: &mut Ui,
    size: Vec2,
    icon: crate::ps_icons::Icon,
    tint: Color32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let enabled = ui.is_enabled();
        if enabled && (response.hovered() || response.is_pointer_button_down_on()) {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(4), color::HOVER);
        }
        crate::ps_icons::paint(
            ui.painter(),
            rect.center(),
            icon,
            if enabled {
                tint
            } else {
                color::OPTIONS_ICON_DISABLED
            },
            color::OPTIONS_BAR,
        );
    }
    response
}

/// Checkerboard (transparency background).
pub fn checkerboard(painter: &egui::Painter, rect: Rect, cell: f32) {
    painter.rect_filled(rect, 0, Color32::WHITE);
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;
    for y in 0..rows {
        for x in 0..cols {
            if (x + y) % 2 == 1 {
                let min = rect.min + Vec2::new(x as f32 * cell, y as f32 * cell);
                let r = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
                painter.rect_filled(r, 0, Color32::from_gray(204));
            }
        }
    }
}

/// An editable options-bar field at `rect`: Photoshop's `#454545` box with
/// a `#666666` border (dimmed when disabled), the value in the panel font,
/// 6 pt from the left. Returns the text edit's response.
pub fn text_box(
    ui: &mut Ui,
    rect: egui::Rect,
    text: &mut String,
    id: impl egui::AsIdSalt,
    enabled: bool,
) -> Response {
    text_box_inset(ui, rect, text, id, enabled, theme::pt(6.0))
}

/// [`text_box`] with the text `inset` from the box's left (the options
/// bar's fields: 4.5 pt).
pub fn text_box_inset(
    ui: &mut Ui,
    rect: egui::Rect,
    text: &mut String,
    id: impl egui::AsIdSalt,
    enabled: bool,
    inset: f32,
) -> Response {
    use theme::pt;
    let (fill, border, value) = if enabled {
        (color::FIELD, Color32::from_gray(0x66), color::TEXT)
    } else {
        (
            Color32::from_gray(0x4d),
            Color32::from_gray(0x5e),
            Color32::from_gray(0x87),
        )
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(pt(2.0) as u8),
        fill,
        Stroke::new(pt(1.0), border),
        StrokeKind::Inside,
    );
    let inner = Rect::from_min_max(
        egui::pos2(rect.left() + inset, rect.top() + pt(1.0)),
        egui::pos2(rect.right() - pt(4.0), rect.bottom() - pt(1.0)),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.add_enabled(
            enabled,
            egui::TextEdit::singleline(text)
                .id_salt(id)
                .frame(egui::Frame::NONE)
                .font(theme::body())
                .text_color(value)
                .desired_width(inner.width())
                .vertical_align(egui::Align::Center)
                .min_size(inner.size()),
        )
    })
    .inner
}
