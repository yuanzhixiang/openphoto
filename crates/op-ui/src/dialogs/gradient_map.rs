//! Image > Adjustments > Gradient Map...: Photoshop 2026's classic dialog,
//! 416 × 230 pt. Sizes are Photoshop points from the dialog's top-left
//! corner.

use egui::{Color32, Rect, Stroke, StrokeKind, Ui, vec2};
use op_core::adjust::Adjustment;
use op_core::gradient::{Gradient, Method};

use super::appkit;
use super::uxp::Button;
use crate::ps_icons::{self, Icon};
use crate::theme::pt;

pub const SIZE: egui::Vec2 = vec2(pt(416.0), pt(230.0));

#[derive(Clone)]
pub struct Dialog {
    /// The gradient (Foreground to Background when the dialog opens).
    pub gradient: Gradient,
    pub dither: bool,
    pub reverse: bool,
    pub method: Method,
    /// The gradient was clicked: the app opens the Gradient Editor.
    pub wants_editor: bool,
}

impl Dialog {
    pub fn new(colors: ([u8; 3], [u8; 3])) -> Self {
        Self {
            gradient: Gradient::two("Foreground to Background", colors.0, colors.1),
            dither: false,
            reverse: false,
            method: Method::Smooth,
            wants_editor: false,
        }
    }

    /// The gradient as mapped: reversed, with the dialog's Method.
    fn used(&self) -> Gradient {
        let mut g = if self.reverse {
            self.gradient.reversed()
        } else {
            self.gradient.clone()
        };
        g.method = self.method;
        g
    }

    /// Two opaque stops at the ends with the midpoint halfway map through
    /// `Adjustment::GradientMap` (measured against Photoshop); anything
    /// else, or Dither, through the gradient's table.
    pub fn adjustment(&self) -> Adjustment {
        let g = self.used();
        let simple = g.colors.len() == 2
            && g.colors[0].location == 0.0
            && g.colors[1].location == 1.0
            && g.colors[0].midpoint == 0.5
            && g.opacities.iter().all(|o| o.opacity >= 1.0)
            && g.smoothness >= 1.0;
        if simple && !self.dither {
            return Adjustment::GradientMap {
                from: g.colors[0].color,
                to: g.colors[1].color,
                method: self.method,
            };
        }
        Adjustment::GradientTable {
            table: g.table(),
            dither: self.dither,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, frame: Rect, preview: &mut bool) -> Option<Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();

        // The gradient, as the map will use it
        appkit::group(
            &painter,
            r(11.0, 46.5, 309.0, 97.0),
            (at(25.5, 0.0).x, at(246.0, 0.0).x),
        );
        appkit::label(ui, at(30.5, 46.0), "Gradient Used for Grayscale Mapping");
        let field = r(20.0, 67.5, 300.0, 87.5);
        painter.rect(
            field,
            0,
            appkit::FIELD,
            Stroke::new(pt(1.0), Color32::from_gray(0x66)),
            StrokeKind::Inside,
        );
        let bar = r(21.0, 68.5, 286.0, 86.5);
        super::gradient_editor::paint_gradient(&painter, bar, &self.used());
        // Clicking the gradient opens the Gradient Editor
        if ui
            .interact(bar, ui.id().with("gradient-map-bar"), egui::Sense::click())
            .clicked()
        {
            self.wants_editor = true;
        }
        // The presets button: a few two-color gradients
        let chevron = r(286.0, 68.5, 299.5, 87.0);
        ps_icons::paint_scaled(
            &painter,
            chevron.center(),
            Icon::Caret,
            appkit::TEXT,
            appkit::FIELD,
            0.8,
        );
        let response = ui.interact(
            chevron,
            ui.id().with("gradient-map-presets"),
            egui::Sense::click(),
        );
        // The presets (Photoshop's basics with the dialog's first colors)
        let first = (
            self.gradient.colors.first().map_or([0; 3], |s| s.color),
            self.gradient.colors.last().map_or([255; 3], |s| s.color),
        );
        let presets = super::gradient_editor::presets(first.0, first.1);
        let entries: Vec<_> = presets
            .iter()
            .map(|p| {
                crate::native_popup::Entry::item(p.name.clone(), p.colors == self.gradient.colors)
            })
            .collect();
        if let Some(k) = crate::native_popup::dropdown(
            ui,
            &response,
            ui.id().with("gradient-map-menu"),
            &entries,
        ) {
            self.gradient = presets[k].clone();
        }

        appkit::group(
            &painter,
            r(11.0, 121.0, 309.0, 219.0),
            (at(25.5, 0.0).x, at(130.0, 0.0).x),
        );
        appkit::label(ui, at(30.5, 120.5), "Gradient Options");
        appkit::checkbox(ui, at(20.0, 139.0), "Dither", &mut self.dither);
        appkit::checkbox(ui, at(20.0, 166.0), "Reverse", &mut self.reverse);
        appkit::label(ui, at(20.5, 200.0), "Method:");
        let chosen = Method::ALL
            .iter()
            .position(|&m| m == self.method)
            .unwrap_or(0);
        if let Some(k) = appkit::popup(
            ui,
            r(72.0, 189.5, 157.0, 210.5),
            "gradient-map-method",
            self.method.label(),
            &appkit::choices(Method::ALL.iter().map(|m| m.label()), chosen),
        ) {
            self.method = Method::ALL[k];
        }

        let ok = appkit::button(ui, r(325.5, 45.0, 385.5, 71.0), "OK", true, true);
        let cancel = appkit::button(ui, r(325.5, 80.0, 385.5, 106.0), "Cancel", false, true);
        appkit::checkbox(ui, at(325.0, 124.5), "Preview", preview);
        if cancel.clicked() {
            return Some(Button::Cancel);
        }
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
        (ok.clicked() || enter).then_some(Button::Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_method_and_stops() {
        let mut d = Dialog::new(([0; 3], [255, 0, 0]));
        assert_eq!(
            d.adjustment(),
            Adjustment::GradientMap {
                from: [0; 3],
                to: [255, 0, 0],
                method: Method::Smooth
            }
        );
        d.reverse = true;
        d.method = Method::Classic;
        assert_eq!(
            d.adjustment(),
            Adjustment::GradientMap {
                from: [255, 0, 0],
                to: [0; 3],
                method: Method::Classic
            }
        );
        // Dither, or a third stop, maps through the table
        d.dither = true;
        assert!(matches!(
            d.adjustment(),
            Adjustment::GradientTable { dither: true, .. }
        ));
        d.dither = false;
        d.reverse = false;
        d.method = Method::Smooth;
        d.gradient = super::super::gradient_editor::presets([0; 3], [255; 3])[5].clone();
        let Adjustment::GradientTable { table, .. } = d.adjustment() else {
            panic!("a table");
        };
        assert_eq!(table[128], d.gradient.sample(128.0 / 255.0).0);
    }
}
