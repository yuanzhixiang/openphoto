//! Controls of Photoshop 2026's classic (AppKit-drawn) adjustment dialogs,
//! Levels and Curves, measured on Photoshop: 12 pt system-font labels,
//! 19 pt number fields, 21 pt pop-up menus, 26 pt push buttons, light
//! checkboxes and the slider "pins" under histograms.

use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, vec2};

use super::common;
use crate::native_popup::{self, Entry};
use crate::ps_icons::{self, Icon};
use crate::theme::{self, color, pt};

pub const TEXT: Color32 = Color32::from_gray(0xf0);
pub const TEXT_OFF: Color32 = Color32::from_gray(0x8e);
pub const FIELD: Color32 = Color32::from_gray(0x45);
const FIELD_BORDER: Color32 = Color32::from_gray(0x66);
/// Group box frames.
pub const GROUP_LINE: Color32 = Color32::from_gray(0x42);
const PIN_OUTLINE: Color32 = Color32::from_gray(0x1a);

pub fn font() -> egui::FontId {
    theme::dialog(pt(12.0))
}

/// 12 pt text with its capitals centered on `left_center`, tracked as
/// macOS tracks it; `align` picks which end `pos` is.
pub fn text(ui: &Ui, pos: Pos2, align: Align2, text: &str, color: Color32) -> Rect {
    let galley = theme::tracked_galley(ui.painter(), text, font(), color);
    let rect = align.anchor_size(pos, galley.size());
    ui.painter().galley(rect.min, galley, color);
    rect
}

pub fn label(ui: &Ui, left_center: Pos2, s: &str) -> Rect {
    text(ui, left_center, Align2::LEFT_CENTER, s, TEXT)
}

/// A number field (`#454545`, 1 pt `#666666` border, value 4 pt in). With
/// `focus` it takes the keyboard focus with its text selected. Up and Down
/// arrows step by `step` (10 × with Shift) within `range`.
#[allow(clippy::too_many_arguments)]
pub fn field(
    ui: &mut Ui,
    rect: Rect,
    value: &mut String,
    id: impl egui::AsIdSalt + Copy,
    range: (f32, f32),
    step: f32,
    decimals: usize,
    focus: bool,
) -> egui::Response {
    ui.painter().rect(
        rect,
        0,
        FIELD,
        Stroke::new(pt(1.0), FIELD_BORDER),
        StrokeKind::Inside,
    );
    let response = common::text_field(ui, rect, value, id, font(), pt(4.0), focus);
    if response.has_focus() {
        let (up, down, shift) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.modifiers.shift,
            )
        });
        let step = if shift { step * 10.0 } else { step };
        let delta = if up {
            step
        } else if down {
            -step
        } else {
            0.0
        };
        if delta != 0.0
            && let Ok(v) = value.trim().parse::<f32>()
        {
            *value = format!("{:.decimals$}", (v + delta).clamp(range.0, range.1));
        }
    }
    response
}

/// A pop-up menu button: `#454545` with a `#666666` border, its value 8.5 pt
/// in and a chevron 7.75 pt from the right; clicking opens `entries` as a
/// native menu (`native_popup`). Returns the index of the entry picked.
pub fn popup(ui: &mut Ui, rect: Rect, id: &str, value: &str, entries: &[Entry]) -> Option<usize> {
    let response = ui.interact(rect, ui.id().with(id), Sense::click());
    let fill = if response.hovered() {
        Color32::from_gray(0x4c)
    } else {
        FIELD
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(pt(2.0) as u8),
        fill,
        Stroke::new(pt(1.0), FIELD_BORDER),
        StrokeKind::Inside,
    );
    text(
        ui,
        rect.left_center() + vec2(pt(8.5), 0.0),
        Align2::LEFT_CENTER,
        value,
        TEXT,
    );
    ps_icons::paint(
        ui.painter(),
        Pos2::new(rect.right() - pt(7.75), rect.center().y + pt(0.25)),
        Icon::Caret,
        TEXT,
        fill,
    );
    native_popup::dropdown(ui, &response, ui.id().with((id, "menu")), entries)
}

/// Menu entries for `options`, `chosen` checked.
pub fn choices<'a>(options: impl IntoIterator<Item = &'a str>, chosen: usize) -> Vec<Entry> {
    options
        .into_iter()
        .enumerate()
        .map(|(k, o)| Entry::item(o, k == chosen))
        .collect()
}

/// A 26 pt push button: the default one with a light border.
pub fn button(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    default: bool,
    enabled: bool,
) -> egui::Response {
    button_with(ui, rect, label, (default, enabled), 12.0, 1.0)
}

/// [`button`] with a `size` pt label raised `lift` pt from the center (the
/// plug-in style filter dialogs label theirs in 13 pt, centered).
pub fn button_with(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    (default, enabled): (bool, bool),
    size: f32,
    lift: f32,
) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.interact(rect, ui.id().with(("appkit-button", label)), sense);
    let fill = if enabled && response.is_pointer_button_down_on() {
        color::TOOL_ACTIVE
    } else {
        color::PANEL
    };
    let border = match (enabled, default) {
        (false, _) => Color32::from_gray(0x60),
        (true, true) => Color32::from_gray(0xe1),
        (true, false) => Color32::from_gray(0x7d),
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(255),
        fill,
        Stroke::new(pt(1.0), border),
        StrokeKind::Inside,
    );
    let color = if enabled { TEXT } else { TEXT_OFF };
    let galley = theme::tracked_galley(ui.painter(), label, theme::dialog(pt(size)), color);
    let at = Align2::CENTER_CENTER.anchor_size(rect.center() - vec2(0.0, pt(lift)), galley.size());
    ui.painter().galley(at.min, galley, color);
    response
}

/// A 13 pt checkbox: light gray with a dark check when on, a dark box
/// with a light edge when off; the label 10.5 pt to its right.
pub fn checkbox(ui: &mut Ui, min: Pos2, label: &str, checked: &mut bool) -> egui::Response {
    checkbox_with(ui, min, (13.0, 10.5), label, checked, true)
}

/// [`checkbox`] `size` pt square with its label `gap` pt after it (the
/// Fill dialog's are 12 pt, 11 pt); disabled, a `#4d4d4d` box in a
/// `#5d5d5d` edge with a dimmed label.
pub fn checkbox_with(
    ui: &mut Ui,
    min: Pos2,
    (size, gap): (f32, f32),
    label: &str,
    checked: &mut bool,
    enabled: bool,
) -> egui::Response {
    let b = Rect::from_min_size(min, vec2(pt(size), pt(size)));
    let ink = if enabled { TEXT } else { TEXT_OFF };
    let galley = theme::tracked_galley(ui.painter(), label, font(), ink);
    let hit = Rect::from_min_max(
        b.min,
        Pos2::new(b.right() + pt(gap) + galley.size().x, b.bottom()),
    );
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.interact(hit, ui.id().with(("appkit-check", label)), sense);
    if response.clicked() {
        *checked = !*checked;
    }
    let painter = ui.painter();
    let k = size / 13.0;
    if !enabled {
        painter.rect(
            b,
            pt(3.0),
            Color32::from_gray(0x4d),
            Stroke::new(pt(1.0), Color32::from_gray(0x5d)),
            StrokeKind::Inside,
        );
    } else if *checked {
        painter.rect_filled(b, pt(3.0), Color32::from_gray(0xd4));
        let p = |x: f32, y: f32| b.min + vec2(pt(x * k), pt(y * k));
        painter.add(Shape::line(
            vec![p(3.0, 6.75), p(5.5, 9.25), p(10.0, 3.75)],
            Stroke::new(pt(1.75), Color32::from_gray(0x32)),
        ));
    } else {
        painter.rect(
            b,
            pt(3.0),
            Color32::from_gray(0x45),
            Stroke::new(pt(1.0), Color32::from_gray(0xa0)),
            StrokeKind::Inside,
        );
    }
    painter.galley(
        Pos2::new(b.right() + pt(gap), b.center().y - galley.size().y / 2.0),
        galley,
        ink,
    );
    response
}

/// How a slider pin is filled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pin {
    /// Black point: hollow.
    Black,
    Gray,
    White,
}

/// A slider pin under a histogram or ramp: a 12 × 10.5 pt house shape
/// (a point on top, rounded corners below) outlined in near black, with
/// its tip at `tip`.
pub fn pin(painter: &egui::Painter, tip: Pos2, kind: Pin) {
    let p = |x: f32, y: f32| tip + vec2(pt(x), pt(y));
    let outline = vec![
        p(0.0, 0.0),
        p(5.75, 6.5),
        p(5.75, 9.0),
        p(5.0, 10.0),
        p(-5.0, 10.0),
        p(-5.75, 9.0),
        p(-5.75, 6.5),
    ];
    let fill = match kind {
        Pin::Black => Color32::TRANSPARENT,
        Pin::Gray => Color32::from_gray(0xa0),
        Pin::White => Color32::from_gray(0xe6),
    };
    painter.add(Shape::convex_polygon(outline.clone(), fill, Stroke::NONE));
    painter.add(Shape::closed_line(
        outline,
        Stroke::new(pt(1.0), PIN_OUTLINE),
    ));
}

/// A group box: a 1 pt `#424242` frame whose top edge is broken from
/// `gap.0` to `gap.1` (x) for its title.
pub fn group(painter: &egui::Painter, rect: Rect, gap: (f32, f32)) {
    let stroke = Stroke::new(pt(1.0), GROUP_LINE);
    let (l, r, t, b) = (rect.left(), rect.right(), rect.top(), rect.bottom());
    painter.line_segment([Pos2::new(l, t), Pos2::new(gap.0, t)], stroke);
    painter.line_segment([Pos2::new(gap.1, t), Pos2::new(r, t)], stroke);
    painter.line_segment([Pos2::new(l, t), Pos2::new(l, b)], stroke);
    painter.line_segment([Pos2::new(r, t), Pos2::new(r, b)], stroke);
    painter.line_segment([Pos2::new(l, b), Pos2::new(r, b)], stroke);
}

/// A radio button centered on `center` with its label 16.5 pt to the
/// right: chosen, a light disc with a dark dot; otherwise a dark disc in
/// a gray ring.
pub fn radio(ui: &mut Ui, center: Pos2, label: &str, chosen: bool) -> bool {
    radio_with(ui, center, label, chosen, (16.5, font()))
}

/// [`radio`] with its label `gap` pt after the button's center, in `font`.
pub fn radio_with(
    ui: &mut Ui,
    center: Pos2,
    label: &str,
    chosen: bool,
    (gap, font): (f32, egui::FontId),
) -> bool {
    let galley = theme::tracked_galley(ui.painter(), label, font, TEXT);
    let hit = Rect::from_min_max(
        center - vec2(pt(7.0), pt(8.0)),
        Pos2::new(center.x + pt(gap) + galley.size().x, center.y + pt(8.0)),
    );
    let clicked = ui
        .interact(
            hit,
            ui.id().with((
                "appkit-radio",
                label,
                center.x.to_bits(),
                center.y.to_bits(),
            )),
            Sense::click(),
        )
        .clicked();
    let painter = ui.painter();
    if chosen {
        painter.circle_filled(center, pt(6.5), Color32::from_gray(0xd4));
        painter.circle_filled(center, pt(2.5), Color32::from_gray(0x32));
    } else {
        painter.circle_filled(center, pt(6.0), Color32::from_gray(0x47));
        painter.circle_stroke(
            center,
            pt(6.0),
            Stroke::new(pt(1.0), Color32::from_gray(0x84)),
        );
    }
    painter.galley(
        Pos2::new(center.x + pt(gap), center.y - galley.size().y / 2.0),
        galley,
        TEXT,
    );
    clicked
}

/// Blend mode menu entries in Photoshop's groups, `chosen` checked, and the
/// mode each entry stands for (`None` for the separators).
pub fn blend_modes(chosen: op_core::BlendMode) -> (Vec<Entry>, Vec<Option<op_core::BlendMode>>) {
    let mut entries = Vec::new();
    let mut modes = Vec::new();
    for (gi, group) in op_core::BlendMode::GROUPS.iter().enumerate() {
        if gi > 0 {
            entries.push(Entry::Separator);
            modes.push(None);
        }
        for &m in *group {
            entries.push(Entry::item(m.label(), m == chosen));
            modes.push(Some(m));
        }
    }
    (entries, modes)
}

/// One of the Set Black, Gray and White Point eyedroppers: the icon, with
/// a pressed `#383838` box in a `#636363` edge while chosen. Returns
/// whether it was clicked.
pub fn eyedropper(ui: &mut Ui, center: Pos2, icon: Icon, chosen: bool, scale: f32) -> bool {
    let rect = Rect::from_center_size(center, vec2(pt(26.0), pt(24.0)));
    let response = ui.interact(
        rect,
        ui.id()
            .with(("eyedropper", center.x.to_bits(), center.y.to_bits())),
        Sense::click(),
    );
    let fill = if chosen {
        ui.painter().rect(
            rect,
            pt(3.0),
            Color32::from_gray(0x38),
            Stroke::new(pt(1.0), Color32::from_gray(0x63)),
            StrokeKind::Inside,
        );
        Color32::from_gray(0x38)
    } else {
        color::PANEL
    };
    ps_icons::paint_scaled(
        ui.painter(),
        center,
        icon,
        Color32::from_gray(0xdd),
        fill,
        scale,
    );
    response.clicked()
}
