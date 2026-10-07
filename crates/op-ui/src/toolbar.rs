//! Left toolbar.

use egui::{Align2, Color32, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use op_core::Color;
use op_tools::{TOOLBAR, Tool};

use crate::icons;
use crate::state::{AppState, PickerTarget};
use crate::theme::{self, color, size};
use crate::widgets;

/// Dark border on the toolbar's right edge, next to the canvas.
const RIGHT_BORDER: f32 = crate::theme::pt(3.0);

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let full = ui.max_rect();
    // The 3 pt divider next to the canvas: a light line between dark ones
    let border = Rect::from_min_max(Pos2::new(full.right() - RIGHT_BORDER, full.top()), full.max);
    ui.painter().rect_filled(border, 0, color::DIVIDER_DARK);
    ui.painter().rect_filled(
        border.shrink2(Vec2::new(crate::theme::pt(1.0), 0.0)),
        0,
        color::DIVIDER_LIGHT,
    );
    ui.set_max_width(full.width() - RIGHT_BORDER);
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

    // Collapse arrows and drag grip at the top
    header(ui, Collapse::Toolbar);
    grip(ui);
    // Measured on Photoshop 2026: the first tool's center 43 pt below the
    // toolbar's top, then one tool every 25.9 pt, with no gaps between groups
    let pitch = crate::theme::pt(25.9);
    // Tool buttons are 30.5 pt wide, centered in the bar
    let button_w = crate::theme::pt(30.5);
    let tool_pad = (size::TOOLBAR - RIGHT_BORDER - button_w) / 2.0;
    let first_top = full.top() + crate::theme::pt(43.0) - pitch / 2.0;
    ui.add_space((first_top - ui.cursor().top()).max(0.0));

    for (slot, group) in TOOLBAR.iter().enumerate() {
        // Each slot shows the tool of its group that was used last
        let shown = app.tool_slots[slot];
        ui.horizontal(|ui| {
            ui.add_space(tool_pad);
            let selected = app.tool.slot() == slot;
            // Photoshop's tool icons are about 15 pt across
            let r = widgets::icon_button_font(
                ui,
                "",
                Vec2::new(button_w, pitch - 1.0),
                theme::tool_icon(crate::theme::pt(17.5)),
                selected,
            );
            // Photoshop-style drawings, else the icon font
            // The icon's center by Photoshop's pitch, free of the buttons'
            // rounding
            let icon_center = Pos2::new(
                r.rect.center().x,
                full.top() + crate::theme::pt(43.0) + pitch * slot as f32,
            );
            let bg = if selected || r.is_pointer_button_down_on() {
                color::TOOL_ACTIVE
            } else if r.hovered() {
                color::HOVER
            } else {
                color::PANEL
            };
            let tint = crate::tool_icons::COLOR;
            if !crate::tool_icons::paint(ui.painter(), icon_center, shown, tint, bg) {
                ui.painter().text(
                    r.rect.center(),
                    Align2::CENTER_CENTER,
                    icons::tool(shown),
                    theme::tool_icon(crate::theme::pt(17.5)),
                    color::ICON,
                );
            }
            // The selected tool's box has a faint light outline
            if selected {
                ui.painter().rect_stroke(
                    r.rect,
                    4,
                    egui::Stroke::new(1.0, Color32::from_gray(0x60)),
                    egui::StrokeKind::Outside,
                );
            }
            if group.len() > 1 {
                group_marker(ui, r.rect);
            }
            let r = r.on_hover_text(tool_tip(shown));
            // A click picks the tool; the release ending a hold doesn't
            let was_held = ui
                .ctx()
                .data(|d| d.get_temp::<bool>(r.id.with("held")))
                .unwrap_or(false);
            if r.clicked() && !was_held {
                app.select_tool(shown);
            }
            if group.len() > 1 {
                flyout(&r, group, shown, app);
            }
        });
        ui.add_space(1.0);
    }

    bottom(ui, app, full, Vec2::new(button_w, pitch - 1.0));
}

/// Below the tools, at Photoshop 2026's positions: Edit Toolbar (•••), the
/// default-colors and swap icons, the color swatches, Quick Mask and Change
/// Screen Mode. Shapes are traced from Photoshop at 2x; `p` takes 2x
/// device pixels measured from the toolbar's top-left corner.
fn bottom(ui: &mut Ui, app: &mut AppState, full: Rect, button: Vec2) {
    use crate::ps_icons::Icon;
    use crate::theme::pt;
    let p = |x: f32, y: f32| Pos2::new(full.left() + pt(x / 2.0), full.top() + pt(y / 2.0));
    let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(p(x0, y0), p(x1, y1));
    let painter = ui.painter().clone();
    let icon_color = color::OPTIONS_ICON;
    let marker = |y: f32| {
        painter.add(Shape::convex_polygon(
            vec![p(62.5, y), p(62.5, y + 6.0), p(56.5, y + 6.0)],
            Color32::from_gray(0xbc),
            Stroke::NONE,
        ));
    };
    let button_at = |ui: &mut Ui, center: Pos2, id: &str| {
        let rect = Rect::from_center_size(center, button);
        let response = ui.interact(rect, ui.id().with(id), Sense::click());
        if response.hovered() {
            painter.rect_filled(rect, 4, color::HOVER);
        }
        response
    };

    // Edit Toolbar
    button_at(ui, p(38.5, 1227.5), "edit-toolbar").on_hover_text("Edit Toolbar");
    crate::ps_icons::paint(
        &painter,
        p(38.5, 1227.5),
        Icon::More,
        icon_color,
        color::PANEL,
    );
    marker(1240.0);

    // Default colors: two small overlapping squares, black over white
    let reset_rect = r(4.0, 1262.0, 32.0, 1290.0);
    let reset = ui.interact(reset_rect, ui.id().with("reset"), Sense::click());
    let light = Color32::from_gray(0xc3);
    painter.rect_filled(r(6.0, 1264.0, 24.0, 1282.0), 0, light);
    painter.rect_filled(r(12.0, 1270.0, 30.0, 1288.0), 0, light);
    painter.rect_filled(r(14.0, 1272.0, 28.0, 1286.0), 0, Color32::BLACK);
    painter.rect_filled(r(16.0, 1274.0, 26.0, 1284.0), 0, Color32::WHITE);
    painter.rect_filled(r(8.0, 1266.0, 22.0, 1280.0), 0, Color32::BLACK);
    if reset
        .on_hover_text("Default Foreground and Background Colors (D)")
        .clicked()
    {
        reset_colors(app);
    }

    // Switch colors: a quarter-circle arrow from left to down
    let swap_rect = r(40.0, 1262.0, 70.0, 1288.0);
    let swap = ui.interact(swap_rect, ui.id().with("swap"), Sense::click());
    let center = p(52.0, 1280.0);
    let arc: Vec<Pos2> = (0..=12)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 * (1.0 - k as f32 / 12.0);
            center + Vec2::new(a.cos(), a.sin()) * pt(5.0)
        })
        .collect();
    painter.add(Shape::line(arc, Stroke::new(pt(2.0), icon_color)));
    painter.add(Shape::convex_polygon(
        vec![p(52.0, 1263.5), p(52.0, 1276.5), p(43.5, 1270.0)],
        icon_color,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![p(55.5, 1280.0), p(67.5, 1280.0), p(61.5, 1286.5)],
        icon_color,
        Stroke::NONE,
    ));
    if swap
        .on_hover_text("Switch Foreground and Background Colors (X)")
        .clicked()
    {
        swap_colors(app);
    }

    // The swatches: background (dark and white frame) under foreground
    // (dark frame)
    let bg_rect = r(28.0, 1316.0, 68.0, 1356.0);
    let fg_rect = r(8.0, 1296.0, 48.0, 1336.0);
    let bg = ui.interact(bg_rect, ui.id().with("bg"), Sense::click());
    let fg = ui.interact(fg_rect, ui.id().with("fg"), Sense::click());
    swatch(&painter, bg_rect, app.background, true);
    swatch(&painter, fg_rect, app.foreground, false);
    if fg.on_hover_text("Set foreground color").clicked() {
        app.editing_background = false;
        app.open_color_picker(PickerTarget::Foreground);
    }
    if bg.on_hover_text("Set background color").clicked() {
        app.editing_background = true;
        app.open_color_picker(PickerTarget::Background);
    }

    // Quick Mask: a rounded frame around a dotted circle
    let on = app
        .active_doc
        .and_then(|id| app.docs.get(&id))
        .is_some_and(|d| d.doc.quick_mask.is_some());
    let qm_center = p(36.0, 1391.0);
    let qm = button_at(ui, qm_center, "quick-mask");
    if on {
        painter.rect_filled(
            Rect::from_center_size(qm_center, button),
            4,
            color::TOOL_ACTIVE,
        );
    }
    painter.rect_stroke(
        r(18.0, 1378.0, 54.0, 1404.0),
        1,
        Stroke::new(pt(1.0), icon_color),
        StrokeKind::Inside,
    );
    for (x, y) in [
        (33.0, 1383.0),
        (37.0, 1383.0),
        (29.0, 1385.0),
        (41.0, 1385.0),
        (27.0, 1389.0),
        (43.0, 1389.0),
        (27.0, 1393.0),
        (43.0, 1393.0),
        (29.0, 1397.0),
        (41.0, 1397.0),
        (33.0, 1399.0),
        (37.0, 1399.0),
    ] {
        painter.rect_filled(
            Rect::from_center_size(p(x, y), Vec2::splat(pt(1.0))),
            0,
            icon_color,
        );
    }
    if qm.on_hover_text("Edit in Quick Mask Mode (Q)").clicked()
        && let Some(state) = app.active()
    {
        toggle_quick_mask(state);
    }

    // Change Screen Mode: two overlapping windows, the back one on top
    let sm_center = p(36.0, 1443.0);
    button_at(ui, sm_center, "screen-mode").on_hover_text("Change Screen Mode (F)");
    let window = |rect: Rect, fill: bool| {
        if fill {
            painter.rect_filled(rect, 0, color::PANEL);
        }
        painter.rect_stroke(
            rect,
            0,
            Stroke::new(pt(1.0), icon_color),
            StrokeKind::Inside,
        );
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(rect.width(), pt(2.0))),
            0,
            icon_color,
        );
    };
    window(r(18.0, 1436.0, 42.0, 1456.0), false);
    window(r(28.0, 1430.0, 54.0, 1450.0), true);
    marker(1456.0);
}

/// Which collapse bar a column has: the toolbar's bold "»", the panel
/// column's thin "»" or the icon strip's thin "«".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collapse {
    Toolbar,
    Panels,
    IconStrip,
}

/// Height of the collapse bar at the top of the toolbar, the icon strip and
/// the panel column, its two 1 pt lines included (Photoshop 2026).
pub const COLLAPSE_BAR: f32 = crate::theme::pt(13.0);

/// The collapse bar: `#424242` between two `#383838` lines, with the
/// chevrons Photoshop draws in it.
pub fn header(ui: &mut Ui, kind: Collapse) -> egui::Response {
    use crate::theme::pt;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), COLLAPSE_BAR),
        Sense::click(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, 0, color::DIVIDER_DARK);
    painter.rect_filled(
        rect.shrink2(Vec2::new(0.0, pt(1.0))),
        0,
        color::COLLAPSE_BAR,
    );
    let (center, icon) = match kind {
        Collapse::Toolbar => (
            Pos2::new(rect.left() + pt(9.25), rect.top() + pt(6.5)),
            crate::ps_icons::Icon::CollapseToolbar,
        ),
        Collapse::Panels => (
            Pos2::new(rect.right() - pt(11.0), rect.top() + pt(6.5)),
            crate::ps_icons::Icon::CollapseRight,
        ),
        Collapse::IconStrip => (
            Pos2::new(rect.right() - pt(9.25), rect.top() + pt(6.5)),
            crate::ps_icons::Icon::CollapseLeft,
        ),
    };
    crate::ps_icons::paint(
        painter,
        center,
        icon,
        color::COLLAPSE_CHEVRON,
        color::COLLAPSE_BAR,
    );
    response
}

/// The toolbar's drag grip under the collapse bar: ten 1 × 4 pt dashes.
fn grip(ui: &mut Ui) {
    use crate::theme::pt;
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), pt(7.0)), Sense::hover());
    for k in 0..10 {
        ui.painter().rect_filled(
            Rect::from_min_size(
                Pos2::new(
                    rect.left() + pt(10.0 + 2.0 * k as f32),
                    rect.top() + pt(3.0),
                ),
                Vec2::new(pt(1.0), pt(4.0)),
            ),
            0,
            Color32::from_gray(0x45),
        );
    }
}

fn tool_tip(tool: Tool) -> String {
    match tool.shortcut() {
        Some(k) => format!("{} ({k})", tool.name()),
        None => tool.name().to_owned(),
    }
}

/// Photoshop's tool flyout: right-clicking a slot lists the tools of its
/// group to the right of the button, with the shown tool marked.
/// The flyout's measurements on Photoshop 2026 (points): 19 pt rows in a
/// 1 pt `#3e3e3e` frame, the current tool's 4 pt square at x 7, the icon
/// centered at 28.5, the name from 41, the shortcut ending 7 pt from the
/// right and at least 11 pt after the longest name; as wide as that needs
/// (Move Tool / Artboard Tool: 130 × 40), its top-left 1.5 pt right of the
/// button, level with its top (the Move tool's: 36, 92 in the window).
pub mod flyout_metrics {
    pub const ROW: f32 = 19.0;
    pub const FRAME: f32 = 1.0;
    pub const MARK_X: f32 = 7.0;
    pub const ICON_X: f32 = 28.5;
    pub const NAME_X: f32 = 41.0;
    pub const GAP: f32 = 11.0;
    pub const RIGHT: f32 = 7.0;
    pub const OFFSET: (f32, f32) = (1.5, 0.0);
    /// Text size: Photoshop's "Move Tool" is 47.5 pt wide and "Artboard
    /// Tool" 65 in Adobe Clean; Source Sans 3 at 11.5 pt gives 48.7 and
    /// 65.7.
    pub const TEXT: f32 = 11.5;
    pub const ICON: f32 = 16.3;
    /// The drawn icons at the flyout's size: Photoshop's Move icon is 14 pt
    /// across there, 15 in the toolbar.
    pub const ICON_SCALE: f32 = 0.93;
}

/// The flyout's text font (the panels' font).
fn flyout_font() -> egui::FontId {
    egui::FontId::proportional(crate::theme::pt(flyout_metrics::TEXT))
}

fn flyout_galley(painter: &egui::Painter, text: &str) -> std::sync::Arc<egui::Galley> {
    painter.layout_no_wrap(text.to_owned(), flyout_font(), color::TEXT)
}

/// The flyout's size for `group`, in points.
pub fn flyout_size(painter: &egui::Painter, group: &[Tool]) -> Vec2 {
    use flyout_metrics::*;
    let width = |t: &str| flyout_galley(painter, t).size().x;
    let pt = crate::theme::pt;
    let name = group.iter().map(|t| width(t.name())).fold(0.0, f32::max);
    let key = group
        .iter()
        .filter_map(|t| t.shortcut())
        .map(|k| width(&k.to_string()))
        .fold(0.0, f32::max);
    Vec2::new(
        pt(NAME_X + GAP + RIGHT) + name + key,
        pt(ROW * group.len() as f32 + 2.0 * FRAME),
    )
}

/// How long a toolbar button must be held down to open its flyout.
pub const HOLD_SECONDS: f64 = 0.4;

fn flyout(button: &egui::Response, group: &[Tool], shown: Tool, app: &mut AppState) {
    use crate::theme::pt;
    use flyout_metrics::*;
    // Right-clicking, or holding the button down (Photoshop's press and
    // hold), opens it
    let held = button.is_pointer_button_down_on()
        && button.ctx.input(|i| {
            i.pointer.primary_down()
                && i.pointer
                    .press_start_time()
                    .is_some_and(|t| i.time - t >= HOLD_SECONDS)
        });
    // The release ending a hold keeps the flyout open (it isn't a click)
    let held_id = button.id.with("held");
    if held {
        button.ctx.data_mut(|d| d.insert_temp(held_id, true));
    }
    let released_hold = !button.is_pointer_button_down_on()
        && button
            .ctx
            .data_mut(|d| d.remove_temp::<bool>(held_id))
            .unwrap_or(false);
    if button.is_pointer_button_down_on() && !held {
        // Wake up when the hold is long enough
        button
            .ctx
            .request_repaint_after(std::time::Duration::from_secs_f64(0.05));
    }
    let open = (button.secondary_clicked() || held || released_hold)
        .then_some(egui::SetOpenCommand::Bool(true));
    let size = flyout_size(&button.ctx.layer_painter(button.layer_id), group);
    let corner = button.rect.right_top() + Vec2::new(pt(OFFSET.0), pt(OFFSET.1));
    egui::Popup::from_response(button)
        .anchor(corner)
        .align(egui::RectAlign::RIGHT_START)
        .gap(0.0)
        .open_memory(open)
        .close_behavior(if released_hold {
            egui::PopupCloseBehavior::IgnoreClicks
        } else {
            egui::PopupCloseBehavior::CloseOnClick
        })
        .frame(
            egui::Frame::new()
                .fill(color::PANEL)
                .stroke(Stroke::new(pt(FRAME), Color32::from_gray(0x3e)))
                .shadow(egui::Shadow {
                    offset: [0, 4],
                    blur: 12,
                    spread: 0,
                    color: Color32::from_black_alpha(90),
                }),
        )
        .show(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let inner = size.x - pt(2.0 * FRAME);
            ui.set_width(inner);
            for &tool in group {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(inner, pt(ROW)), Sense::click());
                // x measured from the frame's outer edge
                let x = |v: f32| rect.left() - pt(FRAME) + pt(v);
                let painter = ui.painter();
                if response.hovered() {
                    painter.rect_filled(rect, 0, color::ACCENT);
                }
                if tool == shown {
                    let mark = Rect::from_min_size(
                        Pos2::new(x(MARK_X), rect.center().y - pt(2.0)),
                        Vec2::splat(pt(4.0)),
                    );
                    painter.rect_filled(mark, 0, color::TEXT);
                }
                let icon_at = Pos2::new(x(ICON_X), rect.center().y);
                let bg = if response.hovered() {
                    color::ACCENT
                } else {
                    color::PANEL
                };
                let tint = crate::tool_icons::COLOR;
                if !crate::tool_icons::paint_scaled(painter, icon_at, tool, tint, bg, ICON_SCALE) {
                    painter.text(
                        icon_at,
                        Align2::CENTER_CENTER,
                        icons::tool(tool),
                        theme::tool_icon(pt(ICON)),
                        color::ICON,
                    );
                }
                let galley = flyout_galley(painter, tool.name());
                let name_pos = Pos2::new(x(NAME_X), rect.center().y - galley.size().y / 2.0);
                painter.galley(name_pos, galley, color::TEXT);
                if let Some(k) = tool.shortcut() {
                    let galley = flyout_galley(painter, &k.to_string());
                    let pos = Pos2::new(
                        rect.right() + pt(FRAME) - pt(RIGHT) - galley.size().x,
                        rect.center().y - galley.size().y / 2.0,
                    );
                    painter.galley(pos, galley, color::TEXT);
                }
                if response.clicked() {
                    app.select_tool(tool);
                }
            }
        });
}

/// Small corner triangle indicating more tools in the group.
fn group_marker(ui: &Ui, rect: Rect) {
    let br = rect.right_bottom() + Vec2::new(-3.0, -3.0);
    ui.painter().add(Shape::convex_polygon(
        vec![br, br + Vec2::new(-4.0, 0.0), br + Vec2::new(0.0, -4.0)],
        color::ICON,
        Stroke::NONE,
    ));
}

/// A color swatch with Photoshop's 1 pt dark frame; `white_frame` adds the
/// 1 pt white frame inside it that the background swatch has.
pub fn swatch(painter: &egui::Painter, rect: Rect, c: Color, white_frame: bool) {
    use crate::theme::pt;
    let [r, g, b, _] = c.to_rgba8();
    painter.rect_filled(rect, 0, Color32::from_gray(0x36));
    let mut inner = rect.shrink(pt(1.0));
    if white_frame {
        painter.rect_filled(inner, 0, Color32::WHITE);
        inner = inner.shrink(pt(1.0));
    }
    painter.rect_filled(inner, 0, Color32::from_rgb(r, g, b));
}

/// Q: enters or leaves Quick Mask, recorded as "Quick Mask" either way.
pub fn toggle_quick_mask(state: &mut crate::state::DocState) {
    if state.doc.quick_mask.is_some() {
        state.doc.exit_quick_mask();
    } else {
        state.doc.enter_quick_mask();
    }
    state.record("Quick Mask");
}

pub fn reset_colors(app: &mut AppState) {
    app.foreground = Color::BLACK;
    app.background = Color::WHITE;
}

pub fn swap_colors(app: &mut AppState) {
    std::mem::swap(&mut app.foreground, &mut app.background);
}
