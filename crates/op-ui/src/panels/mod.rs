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
mod presets;
pub use color_panel::DEFAULT_SWATCHES;
pub mod history;
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
        }
    }
}

/// The divider between groups: a light line between two dark ones.
const GROUP_GAP: f32 = pt(3.0);
const MIN_GROUP: f32 = size::PANEL_TAB_BAR + 40.0;

impl Panels {
    pub fn show(&mut self, ui: &mut Ui, app: &mut AppState) {
        ui.spacing_mut().item_spacing.y = 0.0;
        crate::toolbar::header(ui, crate::toolbar::Collapse::Panels);

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
        let group = &mut self.groups[index];

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
            let response = ui.interact(tab_rect, ui.id().with(("tab", index, i)), Sense::click());
            if response.clicked() {
                group.active = i;
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
        match kind {
            PanelKind::Color => color_panel::show(&mut child, app),
            PanelKind::Swatches => color_panel::swatches(&mut child, app),
            PanelKind::Properties => properties::show(&mut child, app),
            PanelKind::Layers => layers::show(&mut child, app),
            PanelKind::Gradients => presets::gradients(&mut child, app),
            PanelKind::Channels => channels::show(&mut child, app),
            PanelKind::Patterns => presets::patterns(&mut child, app),
            other => placeholder(&mut child, other),
        }
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
pub fn icon_strip(ui: &mut Ui, app: &mut AppState) -> Rect {
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
    crate::toolbar::header(ui, crate::toolbar::Collapse::IconStrip);
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
    history.rect
}
