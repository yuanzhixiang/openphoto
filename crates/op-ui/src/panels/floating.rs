//! Panels opened from the Window menu that aren't docked: Info, Navigator
//! and Histogram. Each floats in its own frame with a tab header (the
//! panel's name and a close button) and can be dragged by the header.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::icons;
use crate::state::AppState;
use crate::theme::{self, color, pt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Floating {
    Info,
    Navigator,
    Histogram,
    CloneSource,
    Brushes,
    BrushSettings,
    ToolPresets,
}

impl Floating {
    pub const ALL: [Self; 7] = [
        Self::Navigator,
        Self::Histogram,
        Self::Info,
        Self::CloneSource,
        Self::Brushes,
        Self::BrushSettings,
        Self::ToolPresets,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Navigator => "Navigator",
            Self::Histogram => "Histogram",
            Self::CloneSource => "Clone Source",
            Self::Brushes => "Brushes",
            Self::BrushSettings => "Brush Settings",
            Self::ToolPresets => "Tool Presets",
        }
    }

    fn size(self, app: &AppState) -> Vec2 {
        match self {
            // Taller by three lines for each row of two color samplers
            Self::Info => {
                let rows = super::info::sampler_rows(app);
                Vec2::new(
                    pt(250.0),
                    pt(200.0) + rows as f32 * super::info::SAMPLER_ROW,
                )
            }
            Self::Navigator => Vec2::new(pt(250.0), pt(230.0)),
            Self::Histogram if app.histogram_expanded => {
                Vec2::new(pt(250.0), super::histogram::EXPANDED)
            }
            Self::Histogram => Vec2::new(pt(250.0), pt(150.0)),
            Self::CloneSource => Vec2::new(pt(250.0), pt(250.0)),
            Self::Brushes => Vec2::new(pt(250.0), pt(260.0)),
            Self::BrushSettings => Vec2::new(pt(300.0), pt(240.0)),
            Self::ToolPresets => Vec2::new(pt(250.0), pt(200.0)),
        }
    }
}

/// Which floating panels are open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FloatingPanels {
    pub info: bool,
    pub navigator: bool,
    pub histogram: bool,
    pub clone_source: bool,
    pub brushes: bool,
    pub brush_settings: bool,
    pub tool_presets: bool,
}

impl FloatingPanels {
    pub fn is_open(&self, panel: Floating) -> bool {
        match panel {
            Floating::Info => self.info,
            Floating::Navigator => self.navigator,
            Floating::Histogram => self.histogram,
            Floating::CloneSource => self.clone_source,
            Floating::Brushes => self.brushes,
            Floating::BrushSettings => self.brush_settings,
            Floating::ToolPresets => self.tool_presets,
        }
    }

    pub fn toggle(&mut self, panel: Floating) {
        let open = match panel {
            Floating::Info => &mut self.info,
            Floating::Navigator => &mut self.navigator,
            Floating::Histogram => &mut self.histogram,
            Floating::CloneSource => &mut self.clone_source,
            Floating::Brushes => &mut self.brushes,
            Floating::BrushSettings => &mut self.brush_settings,
            Floating::ToolPresets => &mut self.tool_presets,
        };
        *open = !*open;
    }
}

const HEADER: f32 = pt(26.0);

/// Shows the open floating panels, to the left of the panel column at
/// first and then wherever they are dragged.
pub fn show(ctx: &egui::Context, app: &mut AppState, panel_column_left: f32, top: f32) {
    // Each panel has its own spot in a column, open or not, so panels
    // opened later never land on top of one another
    let mut y = top + pt(8.0);
    for panel in Floating::ALL {
        let size = panel.size(app) + Vec2::new(0.0, HEADER);
        let default = Pos2::new(panel_column_left - size.x - pt(12.0), y);
        y += size.y + pt(8.0);
        if !app.floating.is_open(panel) {
            continue;
        }
        let mut close = false;
        let mut options = None;
        egui::Area::new(egui::Id::new(("floating-panel", panel.title())))
            .default_pos(default)
            .movable(true)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                let painter = ui.painter();
                painter.add(
                    egui::Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 0,
                        color: Color32::from_black_alpha(90),
                    }
                    .as_shape(rect, 0),
                );
                painter.rect(
                    rect,
                    0,
                    color::PANEL,
                    Stroke::new(1.0, color::SEPARATOR),
                    StrokeKind::Outside,
                );
                // Header: a tab with the panel's name, and a close button
                let header = Rect::from_min_size(rect.min, Vec2::new(rect.width(), HEADER));
                painter.rect_filled(header, 0, color::TAB_BAR);
                let tab = Rect::from_min_size(header.min, Vec2::new(pt(90.0), HEADER));
                painter.rect_filled(tab, 0, color::PANEL);
                painter.text(
                    tab.center(),
                    Align2::CENTER_CENTER,
                    panel.title(),
                    theme::semibold(theme::font::BODY),
                    color::TEXT,
                );
                let x = Rect::from_center_size(
                    header.right_center() - Vec2::new(pt(14.0), 0.0),
                    Vec2::splat(pt(18.0)),
                );
                let close_response =
                    ui.interact(x, ui.id().with(("close", panel.title())), Sense::click());
                painter.text(
                    x.center(),
                    Align2::CENTER_CENTER,
                    icons::X,
                    theme::icon(pt(12.0)),
                    if close_response.hovered() {
                        color::TEXT
                    } else {
                        color::TEXT_DIM
                    },
                );
                if close_response.clicked() {
                    close = true;
                }
                // Info and Navigator: a panel menu with Panel Options...
                if matches!(panel, Floating::Info | Floating::Navigator) {
                    let menu = Rect::from_center_size(
                        x.center() - Vec2::new(pt(22.0), 0.0),
                        Vec2::splat(pt(18.0)),
                    );
                    let menu_response =
                        ui.interact(menu, ui.id().with(("menu", panel.title())), Sense::click());
                    for k in 0..4 {
                        painter.rect_filled(
                            Rect::from_center_size(
                                menu.center() + Vec2::new(0.0, pt(2.0 * k as f32 - 3.0)),
                                Vec2::new(pt(10.0), pt(1.0)),
                            ),
                            0,
                            if menu_response.hovered() {
                                color::TEXT
                            } else {
                                Color32::from_gray(0xa8)
                            },
                        );
                    }
                    egui::Popup::menu(&menu_response)
                        .id(ui.id().with(("panel-menu", panel.title())))
                        .show(|ui| {
                            if ui.button("Panel Options...").clicked() {
                                options = Some(panel);
                            }
                        });
                }
                let body = Rect::from_min_max(Pos2::new(rect.left(), header.bottom()), rect.max)
                    .shrink(pt(10.0));
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));
                match panel {
                    Floating::Info => super::info::show(&mut child, app),
                    Floating::Navigator => super::navigator::show(&mut child, app),
                    Floating::Histogram => super::histogram::show(&mut child, app),
                    Floating::CloneSource => super::clone_source::show(&mut child, app),
                    Floating::Brushes => super::brushes::brushes(&mut child, app),
                    Floating::BrushSettings => super::brushes::brush_settings(&mut child, app),
                    Floating::ToolPresets => super::tool_presets::show(&mut child, app),
                }
            });
        if close {
            app.floating.toggle(panel);
        }
        if let Some(panel) = options {
            app.panel_options = Some((panel, app.info_options, app.navigator_box));
        }
    }
}

/// The Panel Options dialog of the Info or Navigator panel: OK keeps the
/// options, Cancel or Escape drops them.
pub fn panel_options(ctx: &egui::Context, app: &mut AppState) {
    let Some((panel, mut info, mut view_box)) = app.panel_options.take() else {
        return;
    };
    let mut done = None;
    egui::Modal::new(egui::Id::new("panel-options")).show(ctx, |ui| {
        ui.set_min_width(pt(300.0));
        let title = match panel {
            Floating::Info => "Info Panel Options",
            _ => "Navigator Panel Options",
        };
        ui.heading(title);
        if panel == Floating::Info {
            super::panel_options::info_options_ui(ui, &mut info);
        } else {
            ui.strong("Color:");
            egui::ComboBox::from_id_salt("navigator-box")
                .selected_text(super::panel_options::VIEW_BOX_COLORS[view_box].0)
                .show_ui(ui, |ui| {
                    for (k, (name, _)) in super::panel_options::VIEW_BOX_COLORS.iter().enumerate() {
                        ui.selectable_value(&mut view_box, k, *name);
                    }
                });
        }
        ui.horizontal(|ui| {
            if ui.button("OK").clicked() {
                done = Some(true);
            }
            if ui.button("Cancel").clicked() {
                done = Some(false);
            }
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        done = Some(false);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !ctx.egui_wants_keyboard_input() {
        done = Some(true);
    }
    match done {
        Some(true) => {
            app.info_options = info;
            app.navigator_box = view_box;
        }
        Some(false) => {}
        None => app.panel_options = Some((panel, info, view_box)),
    }
}

/// Small text in the panels.
pub fn small() -> FontId {
    FontId::proportional(theme::font::BODY)
}
