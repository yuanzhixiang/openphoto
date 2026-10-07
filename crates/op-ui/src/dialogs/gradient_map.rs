//! Image > Adjustments > Gradient Map...: Photoshop 2026's classic dialog,
//! 416 × 230 pt. Sizes are Photoshop points from the dialog's top-left
//! corner.

use egui::{Color32, Mesh, Pos2, Rect, Shape, Stroke, StrokeKind, Ui, vec2};
use op_core::adjust::Adjustment;
use op_core::gradient::{Method, blend_colors};

use super::appkit;
use super::uxp::Button;
use crate::ps_icons::{self, Icon};
use crate::theme::pt;

pub const SIZE: egui::Vec2 = vec2(pt(416.0), pt(230.0));

#[derive(Clone)]
pub struct Dialog {
    /// The gradient's two colors (the foreground and background colors
    /// when the dialog opens).
    pub colors: ([u8; 3], [u8; 3]),
    pub dither: bool,
    pub reverse: bool,
    pub method: Method,
}

impl Dialog {
    pub fn new(colors: ([u8; 3], [u8; 3])) -> Self {
        Self {
            colors,
            dither: false,
            reverse: false,
            method: Method::Smooth,
        }
    }

    pub fn adjustment(&self) -> Adjustment {
        let (a, b) = self.colors;
        let (from, to) = if self.reverse { (b, a) } else { (a, b) };
        Adjustment::GradientMap {
            from,
            to,
            method: self.method,
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
        let (from, to) = if self.reverse {
            (self.colors.1, self.colors.0)
        } else {
            self.colors
        };
        let mut mesh = Mesh::default();
        let steps = 64;
        for k in 0..=steps {
            let t = k as f32 / steps as f32;
            let [cr, cg, cb] = blend_colors(from, to, t, self.method);
            let c = Color32::from_rgb(cr, cg, cb);
            let x = bar.left() + bar.width() * t;
            mesh.colored_vertex(Pos2::new(x, bar.top()), c);
            mesh.colored_vertex(Pos2::new(x, bar.bottom()), c);
            if k > 0 {
                let i = (2 * k) as u32;
                mesh.add_triangle(i - 2, i - 1, i);
                mesh.add_triangle(i - 1, i + 1, i);
            }
        }
        painter.add(Shape::mesh(mesh));
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
        let colors = self.colors;
        let mut picked = None;
        egui::Popup::menu(&response)
            .id(ui.id().with("gradient-map-menu"))
            .show(|ui| {
                for (name, pair) in [
                    ("Foreground to Background", colors),
                    ("Black, White", ([0; 3], [255; 3])),
                    ("White, Black", ([255; 3], [0; 3])),
                ] {
                    if ui.button(name).clicked() {
                        picked = Some(pair);
                    }
                }
            });
        if let Some(pair) = picked {
            self.colors = pair;
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
    fn reverse_and_method() {
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
    }
}
