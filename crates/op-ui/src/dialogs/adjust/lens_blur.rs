//! Blur › Lens Blur's dialog, as Photoshop 2026 lays it out: a window
//! nearly the screen's size (1508 × 880 pt at most) with the preview on the
//! left and a column of groups 13.5 pt from the right edge: Preview,
//! Depth Map, Iris, Specular Highlights and Noise. Measured from a 2x
//! capture; x positions in the column are from the window's right edge.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};

use super::{AdjustDialog, Outcome};
use crate::dialogs::{appkit, common, distort};
use crate::theme::{self, pt};

/// The settings' order (`Kind::params`).
pub const MODE: usize = 0;
pub const SOURCE: usize = 1;

/// The depth source's setting (for the app, `AdjustDialog::value_of`).
pub fn source() -> usize {
    SOURCE
}
pub const FOCAL: usize = 2;
pub const INVERT: usize = 3;
pub const SHAPE: usize = 4;
pub const RADIUS: usize = 5;
pub const CURVATURE: usize = 6;
pub const ROTATION: usize = 7;
pub const BRIGHTNESS: usize = 8;
pub const THRESHOLD: usize = 9;
pub const AMOUNT: usize = 10;
pub const DISTRIBUTION: usize = 11;
pub const MONOCHROMATIC: usize = 12;

/// The window's size: Photoshop's 1508 × 880, or less on a smaller screen.
pub fn size(screen: Rect) -> egui::Vec2 {
    vec2(
        (screen.width() - pt(4.0)).min(pt(1508.0)),
        (screen.height() - pt(40.0)).min(pt(880.0)),
    )
}

/// The preview's image area within the window (left of the column, above
/// the zoom bar).
pub fn image_rect(frame: Rect) -> Rect {
    Rect::from_min_max(
        frame.min + vec2(pt(9.0), pt(37.0)),
        frame.max - vec2(pt(267.0), pt(24.0)),
    )
}

impl AdjustDialog {
    pub(super) fn lens_ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        // Window points: x from the left (`at`) or the right edge (`rt`)
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let rt = |x: f32, y: f32| Pos2::new(frame.right() - pt(x), frame.top() + pt(y));
        let rr = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(rt(x0, y0), rt(x1, y1));
        let title = format!("Lens Blur ({})", super::zoom_label(self.pane_zoom));
        common::frame(ui, frame, &title, theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();

        // The preview, its zoom bar under it
        let preview = Rect::from_min_max(at(8.0, 36.0), frame.max - vec2(pt(252.0), pt(9.0)));
        painter.rect(
            preview,
            0,
            Color32::from_gray(0x4d),
            Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
            egui::StrokeKind::Inside,
        );
        let image = image_rect(frame);
        self.pane_ui(ui, image);
        self.focal_pick(ui, image);
        let dy = frame.height() / pt(1.0) - 317.0;
        let zoom = distort::zoom_bar(ui, |x, y| at(x, y + dy), &super::zoom_label(self.pane_zoom));
        self.zoom_pane(zoom);

        // OK and Cancel
        let valid = self.effect().is_some();
        let ok = appkit::button(ui, rr(187.5, 42.0, 63.5, 68.0), "OK", true, valid);
        let cancel = appkit::button(ui, rr(187.5, 78.0, 63.5, 104.0), "Cancel", false, true);

        let group = |ui: &Ui, title: &str, x_title: f32, rect: Rect| {
            let galley = theme::tracked_galley(ui.painter(), title, appkit::font(), appkit::TEXT);
            let x = rt(x_title, 0.0).x;
            appkit::group(
                ui.painter(),
                rect,
                (x - pt(5.0), x + galley.size().x + pt(5.0)),
            );
            appkit::label(ui, Pos2::new(x, rect.top()), title);
        };
        let radio = |ui: &mut Ui, center: Pos2, label: &str, chosen: bool| {
            appkit::radio_with(ui, center, label, chosen, (13.0, appkit::font()))
        };

        // Preview: the checkbox on the group's edge, Faster or More Accurate
        let g = rr(238.0, 123.5, 13.5, 160.5);
        appkit::group(&painter, g, (rt(222.0, 0.0).x, rt(146.0, 0.0).x));
        appkit::checkbox(ui, rt(223.5, 116.0), "Preview", &mut self.preview);
        let mode = self.value(MODE).unwrap_or(0.0) as usize;
        for (k, (x, label)) in [(218.0, "Faster"), (150.0, "More Accurate")]
            .into_iter()
            .enumerate()
        {
            if radio(ui, rt(x, 145.0), label, mode == k) {
                self.values[MODE] = k.to_string();
            }
        }

        // Depth Map
        group(ui, "Depth Map", 218.0, rr(238.0, 172.5, 13.5, 316.0));
        let source = self.value(SOURCE).unwrap_or(0.0) as usize;
        self.popup_row(
            ui,
            SOURCE,
            "Source:",
            rt(154.5, 193.0),
            rr(143.5, 184.5, 26.5, 201.5),
        );
        let depth = source != 0;
        let ink = if depth {
            appkit::TEXT
        } else {
            appkit::TEXT_OFF
        };
        // Set Focal Point: a toggle; then a click in the preview picks the
        // depth there
        let button = rr(227.0, 215.0, 202.0, 240.0);
        let response = ui.interact(button, ui.id().with("lens-focal"), Sense::click());
        if depth && response.clicked() {
            self.extra.lens_pick = !self.extra.lens_pick;
        }
        if !depth {
            self.extra.lens_pick = false;
        }
        painter.rect(
            button,
            pt(2.0),
            if self.extra.lens_pick {
                Color32::from_gray(0x6a)
            } else {
                Color32::from_gray(0x4d)
            },
            Stroke::new(pt(1.0), Color32::from_gray(0x5d)),
            egui::StrokeKind::Inside,
        );
        focal_icon(&painter, button.center(), ink);
        appkit::text(
            ui,
            rt(198.0, 227.0),
            Align2::LEFT_CENTER,
            "Set Focal Point",
            ink,
        );
        self.slider_row(ui, FOCAL, "Blur Focal Distance", 257.0, depth);
        let mut invert = self.value(INVERT) == Some(1.0);
        appkit::checkbox_with(
            ui,
            rt(228.5, 292.5),
            (13.0, 10.5),
            "Invert",
            &mut invert,
            depth,
        );
        if depth {
            self.values[INVERT] = (invert as u8).to_string();
        }

        // Iris
        group(ui, "Iris", 218.0, rr(238.0, 328.5, 13.5, 485.0));
        self.popup_row(
            ui,
            SHAPE,
            "Shape:",
            rt(154.5, 349.0),
            rr(143.5, 341.5, 26.5, 357.5),
        );
        self.slider_row(ui, RADIUS, "Radius", 378.0, true);
        self.slider_row(ui, CURVATURE, "Blade Curvature", 416.0, true);
        self.slider_row(ui, ROTATION, "Rotation", 454.0, true);

        // Specular Highlights
        group(
            ui,
            "Specular Highlights",
            218.0,
            rr(238.0, 497.5, 13.5, 587.0),
        );
        self.slider_row(ui, BRIGHTNESS, "Brightness", 518.0, true);
        self.slider_row(ui, THRESHOLD, "Threshold", 556.0, true);

        // Noise
        group(ui, "Noise", 218.0, rr(238.0, 599.5, 13.5, 729.0));
        self.slider_row(ui, AMOUNT, "Amount", 620.0, true);
        group(ui, "Distribution", 212.0, rr(232.0, 657.0, 19.5, 704.5));
        let distribution = self.value(DISTRIBUTION).unwrap_or(0.0) as usize;
        for (k, (y, label)) in [(676.0, "Uniform"), (692.0, "Gaussian")]
            .into_iter()
            .enumerate()
        {
            if radio(ui, rt(212.0, y), label, distribution == k) {
                self.values[DISTRIBUTION] = k.to_string();
            }
        }
        let mut mono = self.value(MONOCHROMATIC) == Some(1.0);
        appkit::checkbox(ui, rt(230.5, 710.5), "Monochromatic", &mut mono);
        self.values[MONOCHROMATIC] = (mono as u8).to_string();

        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        let enter = !typing && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (ok.clicked() || enter)
            && let Some(effect) = self.effect()
        {
            return Outcome::Apply(effect);
        }
        Outcome::Open
    }

    /// A label right-aligned to `label` and a pop-up menu for setting `i`.
    fn popup_row(&mut self, ui: &mut Ui, i: usize, text: &str, label: Pos2, rect: Rect) {
        appkit::text(ui, label, Align2::RIGHT_CENTER, text, appkit::TEXT);
        if let super::ParamKind::Choice(options) = self.kind.params()[i].kind {
            let chosen = (self.value(i).unwrap_or(0.0) as usize).min(options.len() - 1);
            if let Some(k) = appkit::popup(
                ui,
                rect,
                &format!("lens-popup-{i}"),
                options[chosen],
                &appkit::choices(options.iter().copied(), chosen),
            ) {
                self.values[i] = k.to_string();
            }
        }
    }

    /// A slider row: the label 225 pt from the right edge on `y`, the field
    /// 61–25.5 pt from it, and the track 19.5 pt lower (pins from 226 to
    /// 26 pt from the right edge). Disabled, the label dims and the field
    /// shows its value without taking input.
    fn slider_row(&mut self, ui: &mut Ui, i: usize, text: &str, y: f32, enabled: bool) {
        let frame = self.rect;
        let rt = |x: f32, yy: f32| Pos2::new(frame.right() - pt(x), frame.top() + pt(yy));
        let ink = if enabled {
            appkit::TEXT
        } else {
            appkit::TEXT_OFF
        };
        appkit::text(ui, rt(225.0, y), Align2::LEFT_CENTER, text, ink);
        let field = Rect::from_min_max(rt(61.0, y - 9.5), rt(25.5, y + 9.5));
        let p = &self.kind.params()[i];
        if enabled {
            appkit::field(
                ui,
                field,
                &mut self.values[i],
                ("lens-field", i),
                (p.min, p.max),
                1.0,
                p.decimals,
                false,
            );
        } else {
            ui.painter().rect(
                field,
                0,
                Color32::from_gray(0x4d),
                Stroke::new(pt(1.0), Color32::from_gray(0x5d)),
                egui::StrokeKind::Inside,
            );
            appkit::text(
                ui,
                field.left_center() + vec2(pt(4.0), 0.0),
                Align2::LEFT_CENTER,
                &self.values[i],
                appkit::TEXT_OFF,
            );
        }
        let track_y = y + 18.0;
        let pins = (rt(226.0, 0.0).x, rt(26.0, 0.0).x);
        if enabled {
            self.plain_track(ui, i, (rt(230.0, track_y), rt(21.5, 0.0).x), pins);
        } else {
            let line = Rect::from_min_max(rt(230.0, track_y), rt(21.5, track_y + 3.0));
            ui.painter().rect_filled(line, 0, Color32::from_gray(0x75));
            let v = self.value(i).unwrap_or(p.default);
            let x = pins.0 + (pins.1 - pins.0) * ((v - p.min) / (p.max - p.min)).clamp(0.0, 1.0);
            appkit::pin(
                ui.painter(),
                Pos2::new(x, line.top() - pt(3.0)),
                appkit::Pin::White,
            );
        }
    }

    /// With Set Focal Point on, a click in the preview marks that document
    /// pixel; the app reads its depth into Blur Focal Distance
    /// (`Extra::lens_focal_at`).
    fn focal_pick(&mut self, ui: &mut Ui, image: Rect) {
        if !self.extra.lens_pick {
            return;
        }
        let response = ui.interact(image, ui.id().with("lens-pick"), Sense::click());
        if response.clicked()
            && let (Some(p), Some((cx, cy))) = (response.interact_pointer_pos(), self.pane_center)
        {
            let k = pt(0.5) * self.pane_zoom;
            self.extra.lens_focal_at = Some((
                cx + (p.x - image.center().x) / k,
                cy + (p.y - image.center().y) / k,
            ));
        }
    }

    /// A setting's value (Lens Blur's depth source, for the app).
    pub fn value_of(&self, i: usize) -> Option<f32> {
        self.value(i)
    }

    /// Blur Focal Distance from the depth the app read at the point picked.
    pub fn set_focal_distance(&mut self, depth: u8) {
        self.set(FOCAL, depth as f32);
    }
}

/// Set Focal Point's icon: a dashed square with a cross in it.
fn focal_icon(painter: &egui::Painter, c: Pos2, ink: Color32) {
    let s = Stroke::new(pt(1.0), ink);
    let r = Rect::from_center_size(c, vec2(pt(13.0), pt(13.0)));
    for (a, b) in [
        (r.left_top(), r.right_top()),
        (r.right_top(), r.right_bottom()),
        (r.right_bottom(), r.left_bottom()),
        (r.left_bottom(), r.left_top()),
    ] {
        painter.extend(egui::Shape::dashed_line(&[a, b], s, pt(3.0), pt(2.0)));
    }
    painter.line_segment([c - vec2(pt(3.0), 0.0), c + vec2(pt(3.0), 0.0)], s);
    painter.line_segment([c - vec2(0.0, pt(3.0)), c + vec2(0.0, pt(3.0))], s);
}
