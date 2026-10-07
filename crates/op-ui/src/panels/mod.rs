//! Right-hand panel column: tabbed panel groups stacked vertically, with draggable
//! separators between groups.
//!
//! The three groups are fixed for now; a full docking system with drag-to-dock
//! and collapse-to-icons comes later.

pub(crate) mod brushes;
pub(crate) mod channels;
pub(crate) mod clone_source;
mod color_panel;
pub mod floating;
pub(crate) mod histogram;
pub(crate) mod info;
mod navigator;
pub(crate) mod panel_options;
mod presets;
pub(crate) mod tool_presets;
pub use color_panel::DEFAULT_SWATCHES;
pub mod history;
pub(crate) mod layer_comps;
mod layers;
pub use layers::{
    delete_active_layer, layer_color, layer_from_background_with, new_group_from, new_layer,
    new_layer_from, toggle_active_visibility,
};
mod properties;

use egui::{Align2, Color32, CursorIcon, Pos2, Rect, Sense, Ui, UiBuilder, Vec2};

use crate::state::AppState;
use crate::theme::{self, color, pt, size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelKind {
    Color,
    Swatches,
    Gradients,
    Patterns,
    Properties,
    Adjustments,
    Libraries,
    Layers,
    Channels,
    Paths,
}

impl PanelKind {
    fn title(self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Swatches => "Swatches",
            Self::Gradients => "Gradients",
            Self::Patterns => "Patterns",
            Self::Properties => "Properties",
            Self::Adjustments => "Adjustments",
            Self::Libraries => "Libraries",
            Self::Layers => "Layers",
            Self::Channels => "Channels",
            Self::Paths => "Paths",
        }
    }
}

struct Group {
    tabs: Vec<PanelKind>,
    active: usize,
}

pub struct Panels {
    groups: Vec<Group>,
    /// Group heights. The group at index `flex` ignores its value and takes the
    /// remaining space.
    heights: Vec<f32>,
    flex: usize,
    /// Panels dragged out of the column by their tab: each floats on its
    /// own, at its top-left corner.
    pub floating: Vec<(PanelKind, Pos2)>,
    /// The groups' tab bars this frame, where a floating panel docks.
    bars: Vec<(usize, Rect)>,
    /// With the column collapsed to icons: the panel open beside them.
    pub flyout: Option<PanelKind>,
}

impl Default for Panels {
    fn default() -> Self {
        use PanelKind::*;
        Self {
            groups: vec![
                Group {
                    tabs: vec![Color, Swatches, Gradients, Patterns],
                    active: 0,
                },
                Group {
                    tabs: vec![Properties, Adjustments, Libraries],
                    active: 0,
                },
                Group {
                    tabs: vec![Layers, Channels, Paths],
                    active: 0,
                },
            ],
            // Photoshop 2026's default Essentials layout, measured
            heights: vec![pt(147.0), 0.0, pt(286.0)],
            flex: 1,
            floating: Vec::new(),
            bars: Vec::new(),
            flyout: None,
        }
    }
}

/// The divider between groups: a light line between two dark ones.
const GROUP_GAP: f32 = pt(3.0);
const MIN_GROUP: f32 = size::PANEL_TAB_BAR + 40.0;

impl Panels {
    pub fn show(&mut self, ui: &mut Ui, app: &mut AppState) {
        ui.spacing_mut().item_spacing.y = 0.0;
        // The collapse bar's "»": the column collapses to icons
        if crate::toolbar::header(ui, crate::toolbar::Collapse::Panels).clicked() {
            app.panels_collapsed = true;
        }
        self.bars.clear();

        let area = ui.available_rect_before_wrap();
        let n = self.groups.len();
        let total_gaps = GROUP_GAP * (n - 1) as f32;
        let fixed: f32 = (0..n)
            .filter(|&i| i != self.flex)
            .map(|i| self.heights[i])
            .sum();
        let flex_h = (area.height() - fixed - total_gaps).max(MIN_GROUP);

        let mut y = area.top();
        for i in 0..n {
            let h = if i == self.flex {
                flex_h
            } else {
                self.heights[i]
            };
            let rect = Rect::from_min_size(Pos2::new(area.left(), y), Vec2::new(area.width(), h));
            self.show_group(ui, i, rect, app);
            y += h;

            if i + 1 < n {
                let gap = Rect::from_min_size(
                    Pos2::new(area.left(), y),
                    Vec2::new(area.width(), GROUP_GAP),
                );
                ui.painter().rect_filled(gap, 0, color::DIVIDER_DARK);
                ui.painter().rect_filled(
                    gap.shrink2(Vec2::new(0.0, pt(1.0))),
                    0,
                    color::DIVIDER_LIGHT,
                );
                let r = ui.interact(
                    gap.expand2(Vec2::new(0.0, 2.0)),
                    ui.id().with(("gap", i)),
                    Sense::drag(),
                );
                if r.hovered() || r.dragged() {
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
                }
                if r.dragged() {
                    // Resize the non-flex side; the flex group absorbs the difference
                    let dy = r.drag_delta().y;
                    if i == self.flex {
                        self.heights[i + 1] = (self.heights[i + 1] - dy).max(MIN_GROUP);
                    } else {
                        self.heights[i] = (self.heights[i] + dy).max(MIN_GROUP);
                    }
                }
                y += GROUP_GAP;
            }
        }
        ui.advance_cursor_after_rect(area);
    }

    fn show_group(&mut self, ui: &mut Ui, index: usize, rect: Rect, app: &mut AppState) {
        let bar = Rect::from_min_size(rect.min, Vec2::new(rect.width(), size::PANEL_TAB_BAR));
        let body = Rect::from_min_max(Pos2::new(rect.left(), bar.bottom()), rect.max);
        self.bars.push((index, bar));
        let column_left = rect.left();
        let mut torn = None;
        let group = &mut self.groups[index];
        if group.tabs.is_empty() {
            ui.painter().rect_filled(rect, 0, color::PANEL);
            return;
        }

        let painter = ui.painter_at(rect);
        painter.rect_filled(bar, 0, color::TAB_BAR);
        painter.rect_filled(body, 0, color::PANEL);
        // The bar's 1 pt bottom line, and the 1 pt line on the column's
        // right edge (Photoshop 2026)
        let tabs = Rect::from_min_max(bar.min, Pos2::new(bar.right(), bar.bottom() - pt(1.0)));
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(bar.left(), tabs.bottom()), bar.max),
            0,
            color::DIVIDER_DARK,
        );

        // Tabs: the title with 9 pt on its left and 10 pt on its right,
        // then a 1 pt line; the active tab takes the panel's color and is
        // joined to the body (no line under it)
        let mut x = bar.left();
        for (i, &tab) in group.tabs.iter().enumerate() {
            let font = theme::semibold(theme::font::BODY);
            let galley =
                ui.painter()
                    .layout_no_wrap(tab.title().into(), font, Color32::PLACEHOLDER);
            let w = (galley.size().x + pt(19.0)).round();
            let tab_rect =
                Rect::from_min_size(Pos2::new(x, tabs.top()), Vec2::new(w, tabs.height()));
            let response = ui.interact(
                tab_rect,
                ui.id().with(("tab", index, i)),
                Sense::click_and_drag(),
            );
            if response.clicked() || response.drag_started() {
                group.active = i;
            }
            // Dragged out of the column, the panel floats where it's let go
            if response.drag_stopped()
                && let Some(p) = response.interact_pointer_pos()
                && p.x < column_left - pt(30.0)
            {
                torn = Some((i, p));
            }
            let active = group.active == i;
            if active {
                painter.rect_filled(
                    Rect::from_min_max(tab_rect.min, Pos2::new(tab_rect.right(), bar.bottom())),
                    0,
                    color::TAB_ACTIVE,
                );
            }
            painter.rect_filled(
                Rect::from_min_size(tab_rect.right_top(), Vec2::new(pt(1.0), bar.height())),
                0,
                color::DIVIDER_DARK,
            );
            let text_color = if active || response.hovered() {
                color::TAB_TEXT_ACTIVE
            } else {
                color::TAB_TEXT
            };
            let pos = Pos2::new(
                tab_rect.left() + pt(9.0),
                tabs.center().y - galley.size().y / 2.0,
            );
            painter.galley(pos, galley, text_color);
            x += w + pt(1.0);
        }
        // Panel menu: four 10 × 1 pt lines, 2 pt apart (Photoshop 2026)
        for k in 0..4 {
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(
                        bar.right() - pt(15.5),
                        bar.top() + pt(10.0 + 2.0 * k as f32),
                    ),
                    Vec2::new(pt(10.0), pt(1.0)),
                ),
                0,
                Color32::from_gray(0xa8),
            );
        }

        let kind = group.tabs[group.active];
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(body)
                .id_salt(("panel-body", index)),
        );
        child.set_clip_rect(body);
        show_kind(&mut child, kind, app);
        if let Some((i, p)) = torn {
            let kind = group.tabs.remove(i);
            group.active = group.active.min(group.tabs.len().saturating_sub(1));
            self.floating
                .push((kind, p - Vec2::new(pt(45.0), pt(13.0))));
        }
    }

    /// The panels dragged out of the column: each in its own frame with a
    /// tab header, dragged by it; let go over a group's tab bar, it docks
    /// there; its × docks it back into the first group.
    pub fn show_floating(&mut self, ctx: &egui::Context, app: &mut AppState) {
        let size = Vec2::new(size::PANEL_COLUMN, pt(300.0));
        let mut dock: Option<(usize, usize)> = None;
        for k in 0..self.floating.len() {
            let (kind, pos) = self.floating[k];
            let mut moved = Vec2::ZERO;
            let mut dropped = None;
            let mut close = false;
            egui::Area::new(egui::Id::new(("docked-floating", kind.title())))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let rect = Rect::from_min_size(pos, size);
                    let bar = Rect::from_min_size(pos, Vec2::new(size.x, size::PANEL_TAB_BAR));
                    let painter = ui.painter();
                    painter.rect_filled(rect, 0, color::PANEL);
                    painter.rect_filled(bar, 0, color::TAB_BAR);
                    let header = ui.interact(
                        bar,
                        ui.id().with(("float-header", k)),
                        Sense::click_and_drag(),
                    );
                    moved = header.drag_delta();
                    if header.drag_stopped() {
                        dropped = header.interact_pointer_pos();
                    }
                    painter.text(
                        bar.left_center() + Vec2::new(pt(9.0), 0.0),
                        Align2::LEFT_CENTER,
                        kind.title(),
                        theme::semibold(theme::font::BODY),
                        color::TAB_TEXT_ACTIVE,
                    );
                    let x = Rect::from_center_size(
                        bar.right_center() - Vec2::new(pt(14.0), 0.0),
                        Vec2::splat(pt(18.0)),
                    );
                    let close_r = ui.interact(x, ui.id().with(("float-close", k)), Sense::click());
                    painter.text(
                        x.center(),
                        Align2::CENTER_CENTER,
                        crate::icons::X,
                        theme::icon(pt(12.0)),
                        if close_r.hovered() {
                            color::TEXT
                        } else {
                            color::TEXT_DIM
                        },
                    );
                    close = close_r.clicked();
                    let body = Rect::from_min_max(Pos2::new(rect.left(), bar.bottom()), rect.max);
                    let mut child =
                        ui.new_child(UiBuilder::new().max_rect(body).id_salt(("float-body", k)));
                    child.set_clip_rect(body);
                    show_kind(&mut child, kind, app);
                });
            self.floating[k].1 += moved;
            if close {
                dock = Some((k, 0));
            } else if let Some(p) = dropped
                && let Some((g, _)) = self.bars.iter().find(|(_, r)| r.contains(p))
            {
                dock = Some((k, *g));
            }
        }
        if let Some((k, g)) = dock {
            let (kind, _) = self.floating.remove(k);
            if let Some(group) = self.groups.get_mut(g) {
                group.tabs.push(kind);
                group.active = group.tabs.len() - 1;
            }
        }
    }

    /// The icon column, with the panels' buttons when the column is
    /// collapsed to icons (a click opens that panel beside them).
    pub fn icon_strip(&mut self, ui: &mut Ui, app: &mut AppState) -> Rect {
        let (history, expand) = icon_strip(ui, app);
        if expand {
            app.panels_collapsed = false;
            self.flyout = None;
        }
        if app.panels_collapsed {
            let content = ui.max_rect().shrink2(Vec2::new(pt(3.0), 0.0));
            let mut y = content.top() + pt(90.0);
            for g in &self.groups {
                for &kind in &g.tabs {
                    let rect = Rect::from_min_size(
                        Pos2::new(content.left() + pt(2.0), y),
                        Vec2::new(content.width() - pt(4.0), pt(24.0)),
                    );
                    let r = ui.interact(
                        rect,
                        ui.id().with(("strip-panel", kind.title())),
                        Sense::click(),
                    );
                    let on = self.flyout == Some(kind);
                    if on {
                        ui.painter().rect_filled(rect, 4, color::TOOL_ACTIVE);
                    } else if r.hovered() {
                        ui.painter().rect_filled(rect, 4, color::HOVER);
                    }
                    // (the panel's initials, where Photoshop draws its icon)
                    let short: String = kind.title().chars().take(2).collect();
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        short,
                        theme::small(),
                        color::TEXT,
                    );
                    let r = r.on_hover_text(kind.title());
                    if r.clicked() {
                        self.flyout = if on { None } else { Some(kind) };
                    }
                    y += pt(26.0);
                }
                y += pt(6.0);
            }
        }
        history
    }

    /// The flyout of a collapsed panel, to the left of the icon column; a
    /// click elsewhere closes it.
    pub fn show_flyout(&mut self, ctx: &egui::Context, app: &mut AppState, strip: Rect) {
        let Some(kind) = self.flyout.filter(|_| app.panels_collapsed) else {
            self.flyout = None;
            return;
        };
        let size = Vec2::new(size::PANEL_COLUMN, pt(320.0));
        let pos = Pos2::new(strip.left() - size.x, strip.top() + pt(16.0));
        let rect = Rect::from_min_size(pos, size);
        egui::Area::new(egui::Id::new("panel-flyout"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.interact(rect, ui.id().with("flyout-back"), Sense::click_and_drag());
                let bar = Rect::from_min_size(pos, Vec2::new(size.x, size::PANEL_TAB_BAR));
                ui.painter().rect_filled(rect, 0, color::PANEL);
                ui.painter().rect_filled(bar, 0, color::TAB_BAR);
                ui.painter().text(
                    bar.left_center() + Vec2::new(pt(9.0), 0.0),
                    Align2::LEFT_CENTER,
                    kind.title(),
                    theme::semibold(theme::font::BODY),
                    color::TAB_TEXT_ACTIVE,
                );
                let body = Rect::from_min_max(Pos2::new(rect.left(), bar.bottom()), rect.max);
                let mut child =
                    ui.new_child(UiBuilder::new().max_rect(body).id_salt("flyout-body"));
                child.set_clip_rect(body);
                show_kind(&mut child, kind, app);
            });
        let pressed = ctx.input(|i| {
            i.pointer
                .primary_pressed()
                .then(|| i.pointer.interact_pos())
                .flatten()
        });
        if let Some(p) = pressed
            && !rect.contains(p)
            && !strip.contains(p)
        {
            self.flyout = None;
        }
    }
}

/// A docked panel's contents in `ui`.
fn show_kind(ui: &mut Ui, kind: PanelKind, app: &mut AppState) {
    match kind {
        PanelKind::Color => color_panel::show(ui, app),
        PanelKind::Swatches => color_panel::swatches(ui, app),
        PanelKind::Properties => properties::show(ui, app),
        PanelKind::Layers => layers::show(ui, app),
        PanelKind::Gradients => presets::gradients(ui, app),
        PanelKind::Channels => channels::show(ui, app),
        PanelKind::Patterns => presets::patterns(ui, app),
        other => placeholder(ui, other),
    }
}

fn placeholder(ui: &mut Ui, kind: PanelKind) {
    ui.painter().text(
        ui.max_rect().center(),
        Align2::CENTER_CENTER,
        format!("{} — not implemented yet", kind.title()),
        theme::small(),
        color::TEXT_DISABLED,
    );
}

/// The icon column between the canvas and the panels (collapsed History, Comments, ...).
/// Returns the rect of the History button, which the History popout is anchored to.
fn icon_strip(ui: &mut Ui, app: &mut AppState) -> (Rect, bool) {
    use crate::theme::pt;
    // Photoshop: a 3 pt divider on each side (next to the document's
    // scrollbar and next to the panels), each a light line between two
    // dark ones
    let full = ui.max_rect();
    let painter = ui.painter();
    for left in [full.left(), full.right() - pt(3.0)] {
        let divider = Rect::from_min_max(
            Pos2::new(left, full.top()),
            Pos2::new(left + pt(3.0), full.bottom()),
        );
        painter.rect_filled(divider, 0, color::DIVIDER_DARK);
        painter.rect_filled(
            divider.shrink2(Vec2::new(pt(1.0), 0.0)),
            0,
            color::DIVIDER_LIGHT,
        );
    }
    let content = Rect::from_min_max(
        Pos2::new(full.left() + pt(3.0), full.top()),
        Pos2::new(full.right() - pt(3.0), full.bottom()),
    );
    let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(content));
    let ui = &mut inner;
    ui.spacing_mut().item_spacing.y = 0.0;
    // The collapse bar's "«": the collapsed column opens again
    let expand = crate::toolbar::header(ui, crate::toolbar::Collapse::IconStrip).clicked();
    // Below the collapse bar, at Photoshop 2026's positions: a drag grip,
    // the History and Comments buttons, and a line under them
    let top = full.top();
    let painter = ui.painter().clone();
    for k in 0..10 {
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(content.left() + pt(9.0 + 2.0 * k as f32), top + pt(16.0)),
                Vec2::new(pt(1.0), pt(4.0)),
            ),
            0,
            egui::Color32::from_gray(0x45),
        );
    }
    let button = Vec2::new(pt(30.0), pt(26.0));
    let strip_button = |ui: &mut Ui, y: f32, icon, tip: &str, on: bool| {
        let center = Pos2::new(content.center().x, top + pt(y));
        let rect = Rect::from_center_size(center, button);
        let response = ui.interact(rect, ui.id().with(tip), Sense::click());
        if on {
            painter.rect_filled(rect, 4, color::TOOL_ACTIVE);
        } else if response.hovered() {
            painter.rect_filled(rect, 4, color::HOVER);
        }
        crate::ps_icons::paint(&painter, center, icon, color::OPTIONS_ICON, color::PANEL);
        response.on_hover_text(tip)
    };
    let history = strip_button(
        ui,
        35.5,
        crate::ps_icons::Icon::History,
        "History",
        app.history_open,
    );
    if history.clicked() {
        app.history_open = !app.history_open;
    }
    strip_button(ui, 63.5, crate::ps_icons::Icon::Comments, "Comments", false);
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(content.left(), top + pt(81.0)),
            Vec2::new(content.width(), pt(1.0)),
        ),
        0,
        color::DIVIDER_DARK,
    );
    (history.rect, expand)
}
