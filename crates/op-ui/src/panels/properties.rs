//! Properties panel: a document's Canvas, Rulers & Grids, Guides and
//! Quick Actions sections, laid out at the positions measured on Photoshop
//! 2026 (points from the panel body's top-left corner) and drawn with
//! shapes traced from it at 2x.

use egui::{Align2, Color32, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use op_core::{BitDepth, ColorMode};

use crate::commands::Command;
use crate::state::AppState;
use crate::theme::{self, color, pt};
use crate::widgets;

/// Labels in the panel.
const LABEL: Color32 = Color32::from_gray(0xd6);
/// Values in fields and dropdowns, and section titles.
const VALUE: Color32 = Color32::from_gray(0xf0);
/// The document icon's and pressed buttons' boxes.
const BOX_FILL: Color32 = Color32::from_gray(0x38);
const BOX_BORDER: Color32 = Color32::from_gray(0x63);
const ICON: Color32 = Color32::from_gray(0xd7);
/// Icons of options that can't be used.
const ICON_OFF: Color32 = Color32::from_gray(0x98);

/// The sections below the Document row, each starting at a 1 pt rule (the
/// first at y 33) with its title 16.25 pt below it, and their heights when
/// open, measured. A closed section keeps only its title.
const SECTIONS: [(&str, f32); 4] = [
    ("Canvas", 225.25),
    ("Rulers & Grids", 68.0),
    ("Guides", 68.0),
    ("Quick Actions", 91.0),
];
const CLOSED_H: f32 = 32.0;
const FIRST_RULE: f32 = 33.0;

/// What a click in the panel asks for, done once the document is let go.
enum Action {
    Run(Command),
    Crop,
}

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let view = app.view;
    let Some(state) = app.active() else {
        empty(ui);
        return;
    };
    let doc = &mut state.doc;
    let has_guides = !doc.guides.is_empty();
    let mut view_after = view;
    let mut action = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        let ids: Vec<egui::Id> = SECTIONS
            .iter()
            .map(|(name, _)| ui.id().with(("section", *name)))
            .collect();
        let open: Vec<bool> = ids
            .iter()
            .map(|id| ui.data_mut(|d| *d.get_temp_mut_or(*id, true)))
            .collect();
        let height = FIRST_RULE
            + SECTIONS
                .iter()
                .zip(&open)
                .map(|((_, h), o)| if *o { *h } else { CLOSED_H })
                .sum::<f32>();
        let (content, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), pt(height)), Sense::hover());
        let at = |x: f32, y: f32| content.min + Vec2::new(pt(x), pt(y));
        let rect = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();
        let text = |x: f32, cy: f32, s: &str, align: Align2, font: egui::FontId, c: Color32| {
            painter.text(
                Pos2::new(at(x, 0.0).x, at(0.0, cy).y + TEXT_DY),
                align,
                s,
                font,
                c,
            );
        };
        let rule = |y: f32| {
            painter.rect_filled(
                Rect::from_min_size(at(0.0, y), Vec2::new(content.width(), pt(1.0))),
                0,
                color::OPTIONS_SEPARATOR,
            );
        };

        // The kind of properties: a document
        let doc_box = rect(10.0, 5.0, 34.0, 29.0);
        painter.rect(
            doc_box,
            0,
            BOX_FILL,
            Stroke::new(pt(1.0), BOX_BORDER),
            StrokeKind::Inside,
        );
        document_icon(&painter, doc_box.center());
        text(
            37.5,
            17.75,
            "Document",
            Align2::LEFT_CENTER,
            theme::body(),
            LABEL,
        );

        let mut top = FIRST_RULE;
        for (k, (name, h)) in SECTIONS.iter().enumerate() {
            rule(top);
            // The title, with a chevron, toggles the section
            let header = ui.interact(
                rect(0.0, top + 5.0, 140.0, top + 27.0),
                ids[k].with("header"),
                Sense::click(),
            );
            if header.clicked() {
                ui.data_mut(|d| d.insert_temp(ids[k], !open[k]));
            }
            let c = at(11.0, top + 16.5);
            let chevron = if open[k] {
                vec![c + v(-4.0, -2.0), c + v(0.0, 2.0), c + v(4.0, -2.0)]
            } else {
                vec![c + v(-2.0, -4.0), c + v(2.0, 0.0), c + v(-2.0, 4.0)]
            };
            painter.add(Shape::line(
                chevron,
                Stroke::new(pt(1.1), Color32::from_gray(0xe0)),
            ));
            text(
                21.0,
                top + 16.25,
                name,
                Align2::LEFT_CENTER,
                theme::semibold(theme::font::BODY),
                VALUE,
            );
            if !open[k] {
                top += CLOSED_H;
                continue;
            }
            // Measured positions are for a section whose rule is at `base`
            let dy = top - [FIRST_RULE, 258.25, 326.25, 394.25][k];
            let at = |x: f32, y: f32| at(x, y + dy);
            let rect = |x0: f32, y0: f32, x1: f32, y1: f32| rect(x0, y0 + dy, x1, y1 + dy);
            let text = |x: f32, cy: f32, s: &str, align: Align2, font: egui::FontId, c: Color32| {
                text(x, cy + dy, s, align, font, c)
            };
            match k {
                0 => canvas_section(ui, &painter, doc, &at, &rect, &text),
                1 => {
                    // Rulers, the grid and the pixel grid; the units
                    let toggles = [
                        (23.0, view.rulers, "Toggle rulers"),
                        (56.0, view.grid, "Toggle grid"),
                        (89.0, false, "Toggle pixel grid"),
                    ];
                    for (i, (x, on, tip)) in toggles.into_iter().enumerate() {
                        let b = rect(x - 13.0, 289.75, x + 13.0, 315.75);
                        if toggle(ui, &painter, b, on, (name, i), tip) {
                            match i {
                                0 => view_after.rulers = !view.rulers,
                                1 => view_after.grid = !view.grid,
                                _ => {}
                            }
                        }
                        match i {
                            0 => ruler_icon(&painter, b.center()),
                            1 => grid_icon(&painter, b.center()),
                            _ => checker_icon(&painter, b.center()),
                        }
                    }
                    let units = rect(119.5, 293.25, 196.5, 312.25);
                    ui.scope_builder(egui::UiBuilder::new().max_rect(units), |ui| {
                        widgets::dropdown(ui, "ruler-units", units.width(), "Pixels", |ui| {
                            for u in RULER_UNITS {
                                ui.add_enabled_ui(u == "Pixels", |ui| {
                                    let _ = ui.selectable_label(u == "Pixels", u);
                                });
                            }
                        });
                    });
                }
                2 => {
                    // Show guides (only with guides), lock guides, smart
                    // guides; the guides' line style
                    let toggles = [
                        (23.0, view.guides && has_guides, "Toggle guides"),
                        (56.0, view.lock_guides, "Lock guides"),
                        (89.0, view.smart_guides, "Toggle smart guides"),
                    ];
                    for (i, (x, on, tip)) in toggles.into_iter().enumerate() {
                        let b = rect(x - 13.0, 358.25, x + 13.0, 384.25);
                        let enabled = i != 0 || has_guides;
                        if enabled && toggle(ui, &painter, b, on, (name, i), tip) {
                            match i {
                                0 => view_after.guides = !view.guides,
                                1 => view_after.lock_guides = !view.lock_guides,
                                _ => view_after.smart_guides = !view.smart_guides,
                            }
                        }
                        let ink = if enabled { ICON } else { ICON_OFF };
                        match i {
                            0 => guides_icon(&painter, b.center(), ink, false),
                            1 => guides_icon(&painter, b.center(), ink, true),
                            _ => smart_guides_icon(&painter, b.center()),
                        }
                    }
                    let style = rect(119.5, 361.25, 196.5, 380.25);
                    ui.scope_builder(egui::UiBuilder::new().max_rect(style), |ui| {
                        widgets::dropdown(ui, "guide-style", style.width(), "", |ui| {
                            let _ = ui.selectable_label(true, "Lines");
                            ui.add_enabled_ui(false, |ui| {
                                let _ = ui.selectable_label(false, "Dashed Lines");
                            });
                        });
                    });
                    painter.rect_filled(
                        Rect::from_min_max(at(127.5, 370.25), at(170.5, 371.25)),
                        0,
                        LABEL,
                    );
                }
                _ => {
                    let buttons = [
                        (10.5, 426.75, "Image Size", Action::Run(Command::ImageSize)),
                        (106.5, 426.75, "Crop", Action::Crop),
                        (10.5, 456.75, "Trim", Action::Run(Command::Trim)),
                        (
                            106.5,
                            456.75,
                            "Rotate",
                            Action::Run(Command::RotateArbitrary),
                        ),
                    ];
                    for (x, y, label, a) in buttons {
                        if button(ui, &painter, rect(x, y, x + 90.0, y + 24.0), label) {
                            action = Some(a);
                        }
                    }
                }
            }
            top += h;
        }
    });

    app.view = view_after;
    match action {
        Some(Action::Run(command)) => crate::commands::run(command, ui.ctx(), app),
        // Photoshop's Crop quick action picks the Crop tool
        Some(Action::Crop) => app.select_tool(op_tools::Tool::Crop),
        None => {}
    }
}

/// Draws text: x, the capitals' center y, the text, alignment, font, color.
type TextFn<'a> = dyn Fn(f32, f32, &str, Align2, egui::FontId, Color32) + 'a;

/// The Canvas section (positions for its rule at y 33).
fn canvas_section(
    ui: &mut Ui,
    painter: &egui::Painter,
    doc: &mut op_core::Document,
    at: &dyn Fn(f32, f32) -> Pos2,
    rect: &dyn Fn(f32, f32, f32, f32) -> Rect,
    text: &TextFn<'_>,
) {
    // W, H (with the constraint link) and X, Y
    link_icon(painter, at(23.25, 84.25));
    let (w, h) = (format!("{} px", doc.width), format!("{} px", doc.height));
    for (row, (axis, value, pos_axis)) in [("W", &w, "X"), ("H", &h, "Y")].into_iter().enumerate() {
        let y = 64.0 + 24.0 * row as f32;
        text(
            59.5,
            y + 8.25,
            axis,
            Align2::RIGHT_CENTER,
            theme::body(),
            LABEL,
        );
        field(painter, rect(64.0, y, 118.0, y + 17.0), value, true);
        text(
            138.0,
            y + 8.25,
            pos_axis,
            Align2::RIGHT_CENTER,
            theme::body(),
            LABEL,
        );
        field(painter, rect(142.5, y, 196.5, y + 17.0), "0 px", false);
    }

    // Orientation: the current one has a box
    let portrait = doc.height >= doc.width;
    if portrait {
        painter.rect(
            rect(65.0, 115.0, 91.0, 141.0),
            0,
            BOX_FILL,
            Stroke::new(pt(1.0), BOX_BORDER),
            StrokeKind::Inside,
        );
    } else {
        painter.rect(
            rect(91.0, 115.0, 117.0, 141.0),
            0,
            BOX_FILL,
            Stroke::new(pt(1.0), BOX_BORDER),
            StrokeKind::Inside,
        );
    }
    orientation_icon(painter, at(78.0, 128.0), true);
    orientation_icon(painter, at(104.0, 128.0), false);

    text(
        64.5,
        156.75,
        &format!("Resolution: {} pixels/inch", doc.resolution.round()),
        Align2::LEFT_CENTER,
        theme::body(),
        LABEL,
    );

    // Mode and bit depth: only RGB, 8 bits is implemented so far
    text(
        59.0,
        183.25,
        "Mode",
        Align2::RIGHT_CENTER,
        theme::body(),
        LABEL,
    );
    let mode_rect = rect(65.0, 174.0, 197.0, 193.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(mode_rect), |ui| {
        widgets::dropdown(
            ui,
            "doc-mode",
            mode_rect.width(),
            doc.color_mode.label(),
            |ui| {
                for m in ColorMode::ALL {
                    ui.add_enabled_ui(m == ColorMode::Rgb, |ui| {
                        ui.selectable_value(&mut doc.color_mode, m, m.label());
                    });
                }
            },
        );
    });
    let depth_rect = rect(65.0, 200.0, 197.0, 219.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(depth_rect), |ui| {
        widgets::dropdown(
            ui,
            "doc-depth",
            depth_rect.width(),
            doc.bit_depth.label(),
            |ui| {
                for d in BitDepth::ALL {
                    ui.add_enabled_ui(d == BitDepth::U8, |ui| {
                        ui.selectable_value(&mut doc.bit_depth, d, d.label());
                    });
                }
            },
        );
    });

    // Fill: the canvas behind the layers, transparent without a background
    let background = doc.has_background();
    text(
        59.0,
        238.5,
        "Fill",
        Align2::RIGHT_CENTER,
        theme::body(),
        LABEL,
    );
    let swatch = rect(64.5, 229.25, 82.5, 248.25);
    if background {
        painter.rect_filled(swatch, 0, Color32::WHITE);
    }
    painter.rect_stroke(
        swatch,
        0,
        Stroke::new(pt(1.0), Color32::from_gray(0x36)),
        StrokeKind::Inside,
    );
    let shown = if background { "White" } else { "Transparent" };
    let fill_rect = rect(86.5, 229.25, 196.5, 248.25);
    ui.scope_builder(egui::UiBuilder::new().max_rect(fill_rect), |ui| {
        widgets::dropdown(ui, "doc-fill", fill_rect.width(), shown, |ui| {
            for f in [
                "White",
                "Black",
                "Background Color",
                "Transparent",
                "Custom...",
            ] {
                ui.add_enabled_ui(f == shown, |ui| {
                    let _ = ui.selectable_label(f == shown, f);
                });
            }
        });
    });
}

const RULER_UNITS: [&str; 7] = [
    "Pixels",
    "Inches",
    "Centimeters",
    "Millimeters",
    "Points",
    "Picas",
    "Percent",
];

/// A 26 pt icon button that's boxed when `on`; returns whether it was
/// clicked.
fn toggle(
    ui: &mut Ui,
    painter: &egui::Painter,
    b: Rect,
    on: bool,
    id: impl egui::AsIdSalt,
    tip: &str,
) -> bool {
    let r = ui.interact(b, ui.id().with(id), Sense::click());
    if on {
        painter.rect(
            b,
            pt(3.0),
            BOX_FILL,
            Stroke::new(pt(1.0), BOX_BORDER),
            StrokeKind::Inside,
        );
    } else if r.hovered() {
        painter.rect_filled(b, pt(3.0), color::HOVER);
    }
    r.on_hover_text(tip).clicked()
}

/// A Quick Actions button: `#454545` with a 1 pt `#666666` edge.
fn button(ui: &mut Ui, painter: &egui::Painter, b: Rect, label: &str) -> bool {
    let r = ui.interact(b, ui.id().with(("quick", label)), Sense::click());
    let fill = if r.is_pointer_button_down_on() {
        color::TOOL_ACTIVE
    } else if r.hovered() {
        Color32::from_gray(0x50)
    } else {
        color::FIELD
    };
    painter.rect(
        b,
        pt(2.0),
        fill,
        Stroke::new(pt(1.0), Color32::from_gray(0x66)),
        StrokeKind::Inside,
    );
    painter.text(
        b.center(),
        Align2::CENTER_CENTER,
        label,
        theme::body(),
        VALUE,
    );
    r.clicked()
}

/// Traced at 2x: an L-shaped ruler with notches.
fn ruler_icon(painter: &egui::Painter, c: Pos2) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    let r =
        |x0, y0, x1, y1, col| painter.rect_filled(Rect::from_min_max(p(x0, y0), p(x1, y1)), 0, col);
    r(-16.0, -16.0, 16.0, -8.0, ICON);
    r(-16.0, -8.0, -8.0, 16.0, ICON);
    for k in 0..4 {
        let d = -6.0 + 6.0 * k as f32;
        r(d, -13.0, d + 2.0, -10.0, color::PANEL);
        r(-13.0, d, -10.0, d + 2.0, color::PANEL);
    }
}

/// Traced at 2x: a 3 × 3 grid in 1 pt lines.
fn grid_icon(painter: &egui::Painter, c: Pos2) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    for k in 0..4 {
        let d = -16.0 + 10.0 * k as f32;
        painter.rect_filled(Rect::from_min_max(p(d, -16.0), p(d + 2.0, 16.0)), 0, ICON);
        painter.rect_filled(Rect::from_min_max(p(-16.0, d), p(16.0, d + 2.0)), 0, ICON);
    }
}

/// Traced at 2x: a framed checkerboard of 3 pt cells.
fn checker_icon(painter: &egui::Painter, c: Pos2) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    for i in 0..5 {
        for j in 0..5 {
            if (i + j) % 2 == 0 {
                let (x, y) = (-14.0 + 6.0 * i as f32, -14.0 + 6.0 * j as f32);
                painter.rect_filled(
                    Rect::from_min_max(p(x, y), p((x + 6.0).min(14.0), (y + 6.0).min(14.0))),
                    0,
                    Color32::from_gray(0x8c),
                );
            }
        }
    }
    painter.rect_stroke(
        Rect::from_min_max(p(-16.0, -16.0), p(16.0, 16.0)),
        0,
        Stroke::new(pt(1.0), ICON),
        StrokeKind::Inside,
    );
}

/// Traced at 2x: two vertical and two horizontal guides; with a padlock
/// at the bottom right for Lock Guides.
fn guides_icon(painter: &egui::Painter, c: Pos2, ink: Color32, lock: bool) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    let r = |x0, y0, x1, y1| painter.rect_filled(Rect::from_min_max(p(x0, y0), p(x1, y1)), 0, ink);
    let bottom = if lock { 15.0 } else { 16.0 };
    r(-10.0, -16.0, -8.0, bottom);
    r(-4.0, -16.0, -2.0, bottom);
    r(-16.0, -6.0, 16.0, -4.0);
    r(-16.0, 8.0, if lock { 2.0 } else { 16.0 }, 10.0);
    if lock {
        r(4.0, 8.0, 18.0, 18.0);
        painter.add(Shape::line(
            (0..=12)
                .map(|k| {
                    let a = std::f32::consts::PI * (1.0 + k as f32 / 12.0);
                    p(11.0 + 4.0 * a.cos(), 7.0 + 4.5 * a.sin())
                })
                .collect(),
            Stroke::new(pt(1.0), ink),
        ));
    }
}

/// Traced at 2x: a guide crossing with an arrow to it and a lightning
/// bolt (Smart Guides).
fn smart_guides_icon(painter: &egui::Painter, c: Pos2) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    let stroke = Stroke::new(pt(1.0), ICON);
    painter.rect_filled(Rect::from_min_max(p(-10.0, -17.0), p(-8.0, 15.0)), 0, ICON);
    painter.rect_filled(Rect::from_min_max(p(-16.0, -7.0), p(16.0, -5.0)), 0, ICON);
    painter.add(Shape::line(
        vec![p(0.0, -12.0), p(-7.0, -6.0), p(0.0, 0.0)],
        stroke,
    ));
    // The bolt: two slanted wedges
    for tri in [
        [p(7.0, 0.0), p(13.0, 0.0), p(6.0, 10.0)],
        [p(10.0, 6.0), p(16.0, 6.0), p(5.0, 17.0)],
    ] {
        painter.add(Shape::convex_polygon(tri.to_vec(), ICON, Stroke::NONE));
    }
}

/// Moves text so its capital letters' center lands on the measured line.
const TEXT_DY: f32 = 0.0;

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(pt(x), pt(y))
}

/// A Photoshop text field: 1 pt `#666666` border on `#454545`, or the
/// dimmer disabled look.
fn field(painter: &egui::Painter, rect: Rect, value: &str, enabled: bool) {
    let (fill, border, text) = if enabled {
        (color::FIELD, color::DROPDOWN_BORDER, VALUE)
    } else {
        (
            Color32::from_gray(0x4d),
            Color32::from_gray(0x5e),
            Color32::from_gray(0x6a),
        )
    };
    painter.rect(
        rect,
        0,
        fill,
        Stroke::new(pt(1.0), border),
        StrokeKind::Inside,
    );
    painter.text(
        Pos2::new(rect.left() + pt(4.0), rect.center().y + TEXT_DY),
        Align2::LEFT_CENTER,
        value,
        theme::body(),
        text,
    );
}

/// Traced at 2x around the box's center: a page with a folded corner.
fn document_icon(painter: &egui::Painter, c: Pos2) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    let stroke = Stroke::new(pt(1.0), ICON);
    painter.add(Shape::closed_line(
        vec![
            p(-9.0, 11.0),
            p(-9.0, -13.0),
            p(5.0, -13.0),
            p(11.0, -7.0),
            p(11.0, 11.0),
        ],
        stroke,
    ));
    painter.add(Shape::line(
        vec![p(3.0, -13.0), p(3.0, -5.0), p(11.0, -5.0)],
        stroke,
    ));
}

/// The W/H constraint link, traced at 2x: two links of a chain, upright.
fn link_icon(painter: &egui::Painter, c: Pos2) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    let stroke = Stroke::new(pt(1.0), ICON);
    for (cy, start) in [(-7.5f32, 120.0f32), (7.5, -60.0)] {
        let arc: Vec<Pos2> = (0..=16)
            .map(|k| {
                let a = (start + 300.0 * k as f32 / 16.0).to_radians();
                p(6.0 * a.cos(), cy + 6.0 * a.sin())
            })
            .collect();
        painter.add(Shape::line(arc, stroke));
    }
    painter.rect_filled(Rect::from_min_max(p(-1.5, -6.0), p(1.5, 6.0)), 0, ICON);
}

/// The orientation icons, traced at 2x: a frame with a person in it.
fn orientation_icon(painter: &egui::Painter, c: Pos2, portrait: bool) {
    let p = |x: f32, y: f32| c + v(x / 2.0, y / 2.0);
    let stroke = Stroke::new(pt(1.0), ICON);
    if portrait {
        painter.rect_stroke(
            Rect::from_min_max(p(-14.0, -16.0), p(14.0, 16.0)),
            0,
            stroke,
            StrokeKind::Inside,
        );
        painter.circle_filled(p(0.0, -7.5), pt(2.0), ICON);
        painter.rect_filled(
            Rect::from_min_max(p(-8.0, -2.0), p(8.0, 10.0)),
            pt(1.5),
            ICON,
        );
        painter.rect_filled(Rect::from_min_max(p(-6.0, 9.0), p(6.0, 14.0)), 0, ICON);
    } else {
        painter.rect_stroke(
            Rect::from_min_max(p(-16.0, -12.0), p(16.0, 12.0)),
            0,
            stroke,
            StrokeKind::Inside,
        );
        painter.circle_filled(p(0.0, -4.5), pt(1.75), ICON);
        painter.rect_filled(
            Rect::from_min_max(p(-8.0, 2.0), p(8.0, 10.0)),
            pt(1.5),
            ICON,
        );
    }
}

fn empty(ui: &mut Ui) {
    ui.painter().text(
        ui.max_rect().center(),
        Align2::CENTER_CENTER,
        "No Properties",
        theme::small(),
        color::TEXT_DISABLED,
    );
}
