//! The Gradient Editor (from the Gradient tool's options bar and Gradient
//! Map's gradient): presets, the gradient's name, Smoothness, and the bar
//! with its opacity stops above and color stops below. Sizes are points
//! from the dialog's top-left corner. The layout follows Photoshop's
//! arrangement; it has not been measured against Photoshop 2026 yet.

use egui::{Align2, Color32, Key, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, vec2};
use op_core::gradient::{ColorStop, Gradient, Method, OpacityStop};

use super::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(516.0), pt(470.0));
/// The gradient bar.
const BAR: [f32; 4] = [30.0, 286.0, 486.0, 314.0];
/// Presets: columns, swatch size and gap.
const PRESET_COLUMNS: usize = 8;
const SWATCH: f32 = 40.0;
const SWATCH_GAP: f32 = 7.0;

/// Which stop is selected: a color stop or an opacity stop, by index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selected {
    Color(usize),
    Opacity(usize),
}

pub enum Outcome {
    Open,
    Cancel,
    Ok(Gradient),
}

#[derive(Clone)]
pub struct GradientEditor {
    pub gradient: Gradient,
    pub selected: Option<Selected>,
    pub presets: Vec<Gradient>,
    name: String,
    smoothness: String,
    /// The selected stop's fields as typed.
    location: String,
    opacity: String,
    /// What the fields were filled from.
    shown: Option<Selected>,
    /// A stop or midpoint being dragged: (kind, index, is midpoint).
    drag: Option<(Selected, bool)>,
    /// The selected color stop's swatch was clicked: the app opens the
    /// Color Picker (`take_color_request`).
    wants_color: bool,
    first_frame: bool,
}

/// Photoshop's Basics and a few more two- and three-color gradients;
/// Foreground to Background and Foreground to Transparent follow the
/// colors given.
pub fn presets(fg: [u8; 3], bg: [u8; 3]) -> Vec<Gradient> {
    let mut list = vec![Gradient::two("Foreground to Background", fg, bg)];
    let mut clear = Gradient::two("Foreground to Transparent", fg, fg);
    clear.opacities[1].opacity = 0.0;
    list.push(clear);
    list.push(Gradient::two("Black, White", [0; 3], [255; 3]));
    let three = |name: &str, a: [u8; 3], b: [u8; 3], c: [u8; 3]| {
        let mut g = Gradient::two(name, a, c);
        g.colors.insert(
            1,
            ColorStop {
                location: 0.5,
                color: b,
                midpoint: 0.5,
            },
        );
        g
    };
    list.push(Gradient::two("Blue, Red", [10, 0, 178], [255, 0, 0]));
    list.push(Gradient::two("Violet, Orange", [41, 10, 89], [255, 124, 0]));
    list.push(three(
        "Yellow, Violet, Orange, Blue",
        [255, 230, 0],
        [128, 0, 128],
        [0, 70, 255],
    ));
    list.push(three(
        "Copper",
        [151, 70, 26],
        [251, 216, 197],
        [108, 46, 22],
    ));
    list.push(three(
        "Chrome",
        [41, 137, 204],
        [255, 255, 255],
        [144, 106, 0],
    ));
    list.push(Gradient::two("Spectrum", [255, 0, 0], [255, 0, 255]));
    if let Some(s) = list.last_mut() {
        s.colors = [
            [255, 0, 0],
            [255, 255, 0],
            [0, 255, 0],
            [0, 255, 255],
            [0, 0, 255],
            [255, 0, 255],
            [255, 0, 0],
        ]
        .iter()
        .enumerate()
        .map(|(i, &color)| ColorStop {
            location: i as f32 / 6.0,
            color,
            midpoint: 0.5,
        })
        .collect();
    }
    list
}

fn percent(v: f32) -> String {
    format!("{}", (v * 100.0).round())
}

impl GradientEditor {
    pub fn new(gradient: Gradient, presets: Vec<Gradient>) -> Self {
        let mut editor = Self {
            name: gradient.name.clone(),
            smoothness: percent(gradient.smoothness),
            gradient,
            selected: None,
            presets,
            location: String::new(),
            opacity: String::new(),
            shown: None,
            drag: None,
            wants_color: false,
            first_frame: true,
        };
        editor.select(Some(Selected::Color(0)));
        editor
    }

    /// The Color Picker's request for the selected color stop (its color).
    pub fn take_color_request(&mut self) -> Option<[u8; 3]> {
        if !std::mem::take(&mut self.wants_color) {
            return None;
        }
        match self.selected {
            Some(Selected::Color(i)) => self.gradient.colors.get(i).map(|s| s.color),
            _ => None,
        }
    }

    /// The Color Picker's OK: the selected color stop takes the color.
    pub fn set_stop_color(&mut self, rgb: [u8; 3]) {
        if let Some(Selected::Color(i)) = self.selected
            && let Some(s) = self.gradient.colors.get_mut(i)
        {
            s.color = rgb;
        }
    }

    fn select(&mut self, s: Option<Selected>) {
        self.selected = s;
        self.shown = None;
    }

    /// A stop added at `location` (color below the bar, opacity above),
    /// taking the gradient's own color or opacity there.
    pub fn add_stop(&mut self, location: f32, color: bool) -> Selected {
        let location = location.clamp(0.0, 1.0);
        let (c, o) = self.gradient.sample(location);
        if color {
            let i = self
                .gradient
                .colors
                .partition_point(|s| s.location <= location);
            self.gradient.colors.insert(
                i,
                ColorStop {
                    location,
                    color: c,
                    midpoint: 0.5,
                },
            );
            Selected::Color(i)
        } else {
            let i = self
                .gradient
                .opacities
                .partition_point(|s| s.location <= location);
            self.gradient.opacities.insert(
                i,
                OpacityStop {
                    location,
                    opacity: o,
                    midpoint: 0.5,
                },
            );
            Selected::Opacity(i)
        }
    }

    /// Delete: a stop goes (two of each kind always stay).
    pub fn delete(&mut self) {
        match self.selected {
            Some(Selected::Color(i)) if self.gradient.colors.len() > 2 => {
                self.gradient.colors.remove(i);
                self.select(Some(Selected::Color(i.saturating_sub(1))));
            }
            Some(Selected::Opacity(i)) if self.gradient.opacities.len() > 2 => {
                self.gradient.opacities.remove(i);
                self.select(Some(Selected::Opacity(i.saturating_sub(1))));
            }
            _ => {}
        }
    }

    /// Moves a stop to `location`, keeping the stops in order (the
    /// selection follows it).
    pub fn move_stop(&mut self, s: Selected, location: f32) -> Selected {
        let location = location.clamp(0.0, 1.0);
        match s {
            Selected::Color(i) => {
                let mut stop = self.gradient.colors.remove(i);
                stop.location = location;
                let j = self
                    .gradient
                    .colors
                    .partition_point(|s| s.location <= location);
                self.gradient.colors.insert(j, stop);
                Selected::Color(j)
            }
            Selected::Opacity(i) => {
                let mut stop = self.gradient.opacities.remove(i);
                stop.location = location;
                let j = self
                    .gradient
                    .opacities
                    .partition_point(|s| s.location <= location);
                self.gradient.opacities.insert(j, stop);
                Selected::Opacity(j)
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, active: bool) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("gradient-editor"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        self.first_frame = false;
        if !active {
            return Outcome::Open;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Gradient Editor", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();

        // Presets
        appkit::label(ui, at(20.0, 48.0), "Presets");
        let mut picked = None;
        for (k, g) in self.presets.iter().enumerate() {
            let (col, row) = (k % PRESET_COLUMNS, k / PRESET_COLUMNS);
            let x = 20.0 + col as f32 * (SWATCH + SWATCH_GAP);
            let y = 62.0 + row as f32 * (SWATCH + SWATCH_GAP);
            let cell = r(x, y, x + SWATCH, y + SWATCH);
            paint_gradient(&painter, cell, g);
            let response = ui
                .interact(cell, ui.id().with(("gradient-preset", k)), Sense::click())
                .on_hover_text(&g.name);
            let chosen = g.colors == self.gradient.colors && g.opacities == self.gradient.opacities;
            let edge = if chosen || response.hovered() {
                Color32::WHITE
            } else {
                Color32::from_gray(0x30)
            };
            painter.rect_stroke(cell, 0, Stroke::new(pt(1.0), edge), StrokeKind::Outside);
            if response.clicked() {
                picked = Some(g.clone());
            }
        }
        if let Some(g) = picked {
            self.name = g.name.clone();
            self.smoothness = percent(g.smoothness);
            self.gradient = g;
            self.select(Some(Selected::Color(0)));
        }

        // Name and New
        appkit::text(
            ui,
            at(65.0, 199.5),
            Align2::RIGHT_CENTER,
            "Name:",
            appkit::TEXT,
        );
        let name = appkit::field(
            ui,
            r(70.0, 190.0, 330.0, 209.0),
            &mut self.name,
            "gradient-name",
            (0.0, 0.0),
            0.0,
            0,
            false,
        );
        if name.changed() {
            self.gradient.name = self.name.clone();
        }
        if appkit::button(ui, r(340.0, 186.5, 400.0, 212.5), "New", false, true).clicked() {
            let mut g = self.gradient.clone();
            if g.name.trim().is_empty() {
                g.name = "Custom".into();
            }
            self.presets.push(g);
        }

        // Gradient Type (Solid) and Smoothness
        appkit::text(
            ui,
            at(105.0, 235.5),
            Align2::RIGHT_CENTER,
            "Gradient Type:",
            appkit::TEXT,
        );
        appkit::popup(
            ui,
            r(110.0, 225.0, 200.0, 246.0),
            "gradient-type",
            "Solid",
            &[crate::native_popup::Entry::item("Solid", true)],
        );
        appkit::text(
            ui,
            at(295.0, 235.5),
            Align2::RIGHT_CENTER,
            "Smoothness:",
            appkit::TEXT,
        );
        let smooth = appkit::field(
            ui,
            r(300.0, 226.0, 345.0, 245.0),
            &mut self.smoothness,
            "gradient-smoothness",
            (0.0, 100.0),
            1.0,
            0,
            false,
        );
        appkit::text(ui, at(349.0, 235.5), Align2::LEFT_CENTER, "%", appkit::TEXT);
        if smooth.changed()
            && let Ok(v) = self.smoothness.trim().parse::<f32>()
        {
            self.gradient.smoothness = (v / 100.0).clamp(0.0, 1.0);
        }

        self.bar(ui, frame);
        self.stop_fields(ui, frame);

        // OK, Cancel, Import..., Export...
        let ok = appkit::button(ui, r(420.0, 38.5, 500.0, 64.5), "OK", true, true);
        let cancel = appkit::button(ui, r(420.0, 73.5, 500.0, 99.5), "Cancel", false, true);
        if appkit::button(ui, r(420.0, 115.0, 500.0, 141.0), "Import...", false, true).clicked() {
            self.import();
        }
        if appkit::button(ui, r(420.0, 150.0, 500.0, 176.0), "Export...", false, true).clicked() {
            self.export();
        }
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        let enter = !typing && ui.input(|i| i.key_pressed(Key::Enter));
        if ok.clicked() || enter {
            return Outcome::Ok(self.gradient.clone());
        }
        Outcome::Open
    }

    /// The bar: the gradient over a checkerboard, opacity stops above it,
    /// color stops below, the selected stop's midpoints as diamonds.
    fn bar(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let bar = Rect::from_min_max(at(BAR[0], BAR[1]), at(BAR[2], BAR[3]));
        let painter = ui.painter().clone();
        crate::widgets::checkerboard(&painter, bar, pt(4.0));
        paint_gradient(&painter, bar, &self.gradient);
        painter.rect_stroke(
            bar,
            0,
            Stroke::new(pt(1.0), Color32::from_gray(0x30)),
            StrokeKind::Outside,
        );
        let x_of = |t: f32| bar.left() + bar.width() * t;
        let t_of = |x: f32| ((x - bar.left()) / bar.width()).clamp(0.0, 1.0);

        // Pointer: stops (above for opacity, below for color), midpoints
        let zones = [
            (
                Rect::from_min_max(at(BAR[0] - 8.0, BAR[1] - 18.0), at(BAR[2] + 8.0, BAR[1])),
                false,
            ),
            (
                Rect::from_min_max(at(BAR[0] - 8.0, BAR[3]), at(BAR[2] + 8.0, BAR[3] + 18.0)),
                true,
            ),
        ];
        for (zone, color) in zones {
            let response = ui.interact(
                zone,
                ui.id().with(("gradient-stops", color)),
                Sense::click_and_drag(),
            );
            let Some(p) = response.interact_pointer_pos() else {
                continue;
            };
            if response.drag_started() || response.clicked() {
                // The nearest stop or midpoint within reach, else a new stop
                let stops: Vec<f32> = if color {
                    self.gradient.colors.iter().map(|s| s.location).collect()
                } else {
                    self.gradient.opacities.iter().map(|s| s.location).collect()
                };
                let kind = |i| {
                    if color {
                        Selected::Color(i)
                    } else {
                        Selected::Opacity(i)
                    }
                };
                let near = stops
                    .iter()
                    .enumerate()
                    .map(|(i, &t)| (i, (x_of(t) - p.x).abs()))
                    .filter(|&(_, d)| d <= pt(6.0))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(i, _)| i);
                let midpoint = self.midpoint_at(color, p.x, &x_of);
                let target = match (near, midpoint) {
                    (Some(i), _) => (kind(i), false),
                    (None, Some(i)) => (kind(i), true),
                    (None, None) => (self.add_stop(t_of(p.x), color), false),
                };
                if !target.1 {
                    self.select(Some(target.0));
                }
                self.drag = response.drag_started().then_some(target);
            }
            if let Some((s, midpoint)) = self.drag
                && response.dragged()
            {
                if midpoint {
                    self.drag_midpoint(s, t_of(p.x));
                } else if !zone.expand(pt(30.0)).contains(p) {
                    // Pulled away: the stop goes
                    self.select(Some(s));
                    self.delete();
                    self.drag = None;
                } else {
                    let moved = self.move_stop(s, t_of(p.x));
                    self.selected = Some(moved);
                    self.shown = None;
                    self.drag = Some((moved, false));
                }
            }
            if response.drag_stopped() {
                self.drag = None;
            }
            // Double-clicking a color stop opens the Color Picker
            if color && response.double_clicked() {
                self.wants_color = true;
            }
        }

        // The stops: house shapes pointing at the bar
        for (i, s) in self.gradient.opacities.iter().enumerate() {
            let gray = (s.opacity * 255.0).round() as u8;
            let selected = self.selected == Some(Selected::Opacity(i));
            stop_marker(
                &painter,
                Pos2::new(x_of(s.location), bar.top()),
                false,
                Color32::from_gray(255 - gray),
                selected,
            );
        }
        for (i, s) in self.gradient.colors.iter().enumerate() {
            let [cr, cg, cb] = s.color;
            let selected = self.selected == Some(Selected::Color(i));
            stop_marker(
                &painter,
                Pos2::new(x_of(s.location), bar.bottom()),
                true,
                Color32::from_rgb(cr, cg, cb),
                selected,
            );
        }
        // The selected stop's midpoints (to its neighbours)
        let (locations, mids, color) = match self.selected {
            Some(Selected::Color(_)) => (
                self.gradient
                    .colors
                    .iter()
                    .map(|s| s.location)
                    .collect::<Vec<_>>(),
                self.gradient
                    .colors
                    .iter()
                    .map(|s| s.midpoint)
                    .collect::<Vec<_>>(),
                true,
            ),
            Some(Selected::Opacity(_)) => (
                self.gradient.opacities.iter().map(|s| s.location).collect(),
                self.gradient.opacities.iter().map(|s| s.midpoint).collect(),
                false,
            ),
            None => (Vec::new(), Vec::new(), true),
        };
        let y = if color {
            bar.bottom() + pt(4.0)
        } else {
            bar.top() - pt(4.0)
        };
        for i in 0..locations.len().saturating_sub(1) {
            let t = locations[i] + (locations[i + 1] - locations[i]) * mids[i];
            let c = Pos2::new(x_of(t), y);
            let d = pt(3.0);
            painter.add(Shape::convex_polygon(
                vec![
                    c + vec2(0.0, -d),
                    c + vec2(d, 0.0),
                    c + vec2(0.0, d),
                    c + vec2(-d, 0.0),
                ],
                Color32::from_gray(0xd0),
                Stroke::NONE,
            ));
        }
    }

    /// The midpoint near screen x `x` on the selected stop's row, if any:
    /// the index of the stop it follows.
    fn midpoint_at(&self, color: bool, x: f32, x_of: &dyn Fn(f32) -> f32) -> Option<usize> {
        let ok = matches!(
            (self.selected, color),
            (Some(Selected::Color(_)), true) | (Some(Selected::Opacity(_)), false)
        );
        if !ok {
            return None;
        }
        let pairs: Vec<(f32, f32)> = if color {
            self.gradient
                .colors
                .iter()
                .map(|s| (s.location, s.midpoint))
                .collect()
        } else {
            self.gradient
                .opacities
                .iter()
                .map(|s| (s.location, s.midpoint))
                .collect()
        };
        (0..pairs.len().saturating_sub(1)).find(|&i| {
            let t = pairs[i].0 + (pairs[i + 1].0 - pairs[i].0) * pairs[i].1;
            (x_of(t) - x).abs() <= pt(4.0)
        })
    }

    fn drag_midpoint(&mut self, s: Selected, t: f32) {
        let set = |a: f32, b: f32| ((t - a) / (b - a).max(1e-6)).clamp(0.05, 0.95);
        match s {
            Selected::Color(i) if i + 1 < self.gradient.colors.len() => {
                let (a, b) = (
                    self.gradient.colors[i].location,
                    self.gradient.colors[i + 1].location,
                );
                self.gradient.colors[i].midpoint = set(a, b);
            }
            Selected::Opacity(i) if i + 1 < self.gradient.opacities.len() => {
                let (a, b) = (
                    self.gradient.opacities[i].location,
                    self.gradient.opacities[i + 1].location,
                );
                self.gradient.opacities[i].midpoint = set(a, b);
            }
            _ => {}
        }
    }

    /// The Stops group: the selected opacity stop's Opacity and Location,
    /// or the color stop's Color and Location, and Delete.
    fn stop_fields(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();
        let title = appkit::label(ui, at(35.0, 350.5), "Stops");
        appkit::group(
            &painter,
            r(20.0, 350.5, 496.0, 456.0),
            (title.left() - pt(5.0), title.right() + pt(5.0)),
        );
        if self.shown != self.selected {
            self.shown = self.selected;
            match self.selected {
                Some(Selected::Color(i)) => {
                    self.location = percent(self.gradient.colors[i].location);
                }
                Some(Selected::Opacity(i)) => {
                    let s = self.gradient.opacities[i];
                    self.location = percent(s.location);
                    self.opacity = percent(s.opacity);
                }
                None => {}
            }
        }
        let opacity_row = matches!(self.selected, Some(Selected::Opacity(_)));
        let color_row = matches!(self.selected, Some(Selected::Color(_)));
        // Opacity row (y 381.5), Color row (y 423.5)
        let ink = |on: bool| if on { appkit::TEXT } else { appkit::TEXT_OFF };
        appkit::text(
            ui,
            at(85.0, 381.5),
            Align2::RIGHT_CENTER,
            "Opacity:",
            ink(opacity_row),
        );
        appkit::text(
            ui,
            at(85.0, 423.5),
            Align2::RIGHT_CENTER,
            "Color:",
            ink(color_row),
        );
        appkit::text(
            ui,
            at(265.0, 381.5),
            Align2::RIGHT_CENTER,
            "Location:",
            ink(opacity_row),
        );
        appkit::text(
            ui,
            at(265.0, 423.5),
            Align2::RIGHT_CENTER,
            "Location:",
            ink(color_row),
        );
        let row_y = if opacity_row { 372.0 } else { 414.0 };
        if opacity_row {
            let f = appkit::field(
                ui,
                r(90.0, 372.0, 140.0, 391.0),
                &mut self.opacity,
                "stop-opacity",
                (0.0, 100.0),
                1.0,
                0,
                false,
            );
            if f.changed()
                && let (Ok(v), Some(Selected::Opacity(i))) =
                    (self.opacity.trim().parse::<f32>(), self.selected)
            {
                self.gradient.opacities[i].opacity = (v / 100.0).clamp(0.0, 1.0);
            }
            appkit::text(ui, at(144.0, 381.5), Align2::LEFT_CENTER, "%", appkit::TEXT);
        }
        // The color swatch: clicking it opens the Color Picker
        let swatch = r(90.0, 414.0, 140.0, 433.0);
        if let Some(Selected::Color(i)) = self.selected {
            let [cr, cg, cb] = self.gradient.colors[i].color;
            painter.rect(
                swatch,
                0,
                Color32::from_rgb(cr, cg, cb),
                Stroke::new(pt(1.0), Color32::from_gray(0x66)),
                StrokeKind::Inside,
            );
            if ui
                .interact(swatch, ui.id().with("stop-color"), Sense::click())
                .clicked()
            {
                self.wants_color = true;
            }
        }
        if self.selected.is_some() {
            let f = appkit::field(
                ui,
                r(270.0, row_y, 320.0, row_y + 19.0),
                &mut self.location,
                "stop-location",
                (0.0, 100.0),
                1.0,
                0,
                false,
            );
            appkit::text(
                ui,
                at(324.0, row_y + 9.5),
                Align2::LEFT_CENTER,
                "%",
                appkit::TEXT,
            );
            if f.changed()
                && let (Ok(v), Some(s)) = (self.location.trim().parse::<f32>(), self.selected)
            {
                let moved = self.move_stop(s, v / 100.0);
                self.selected = Some(moved);
                self.shown = Some(moved);
            }
            let can_delete = match self.selected {
                Some(Selected::Color(_)) => self.gradient.colors.len() > 2,
                Some(Selected::Opacity(_)) => self.gradient.opacities.len() > 2,
                None => false,
            };
            if appkit::button(
                ui,
                r(400.0, row_y - 3.5, 480.0, row_y + 22.5),
                "Delete",
                false,
                can_delete,
            )
            .clicked()
            {
                self.delete();
            }
        }
    }

    /// Import...: a gradient saved by Export... (OpenPhoto's text form).
    fn import(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Gradient", &["opgrd"])
            .pick_file()
        else {
            return;
        };
        if let Some(g) = std::fs::read_to_string(&path).ok().and_then(|t| parse(&t)) {
            self.presets.push(g);
        }
    }

    /// Export...: the gradient as text (stops, method, smoothness).
    fn export(&self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Gradient", &["opgrd"])
            .set_file_name(format!("{}.opgrd", self.gradient.name))
            .save_file()
        else {
            return;
        };
        let _ = std::fs::write(path, serialize(&self.gradient));
    }
}

/// A gradient as lines: its name, `method smoothness`, then `c location r
/// g b midpoint` and `o location opacity midpoint` per stop.
pub fn serialize(g: &Gradient) -> String {
    let mut out = format!("{}\n{} {}\n", g.name, g.method.label(), g.smoothness);
    for s in &g.colors {
        out += &format!(
            "c {} {} {} {} {}\n",
            s.location, s.color[0], s.color[1], s.color[2], s.midpoint
        );
    }
    for s in &g.opacities {
        out += &format!("o {} {} {}\n", s.location, s.opacity, s.midpoint);
    }
    out
}

pub fn parse(text: &str) -> Option<Gradient> {
    let mut lines = text.lines();
    let name = lines.next()?.to_owned();
    let mut head = lines.next()?.split_whitespace();
    let method_label = head.next()?;
    let method = Method::ALL
        .into_iter()
        .find(|m| m.label() == method_label)?;
    let smoothness: f32 = head.next()?.parse().ok()?;
    let mut g = Gradient::two(&name, [0; 3], [0; 3]);
    g.method = method;
    g.smoothness = smoothness;
    g.colors.clear();
    g.opacities.clear();
    for line in lines {
        let v: Vec<&str> = line.split_whitespace().collect();
        match v.first() {
            Some(&"c") if v.len() == 6 => g.colors.push(ColorStop {
                location: v[1].parse().ok()?,
                color: [v[2].parse().ok()?, v[3].parse().ok()?, v[4].parse().ok()?],
                midpoint: v[5].parse().ok()?,
            }),
            Some(&"o") if v.len() == 4 => g.opacities.push(OpacityStop {
                location: v[1].parse().ok()?,
                opacity: v[2].parse().ok()?,
                midpoint: v[3].parse().ok()?,
            }),
            _ => {}
        }
    }
    (g.colors.len() >= 2 && g.opacities.len() >= 2).then_some(g)
}

/// The gradient across `rect` (with its opacity over what is under it).
pub fn paint_gradient(painter: &egui::Painter, rect: Rect, g: &Gradient) {
    let mut mesh = Mesh::default();
    let steps = 96;
    for k in 0..=steps {
        let t = k as f32 / steps as f32;
        let ([cr, cg, cb], o) = g.sample(t);
        let c = Color32::from_rgba_unmultiplied(cr, cg, cb, (o * 255.0).round() as u8);
        let x = rect.left() + rect.width() * t;
        mesh.colored_vertex(Pos2::new(x, rect.top()), c);
        mesh.colored_vertex(Pos2::new(x, rect.bottom()), c);
        if k > 0 {
            let i = (2 * k) as u32;
            mesh.add_triangle(i - 2, i - 1, i);
            mesh.add_triangle(i - 1, i + 1, i);
        }
    }
    painter.add(Shape::mesh(mesh));
}

/// A stop: a little house pointing at the bar from below (color) or
/// above (opacity), filled with its color, its edge light when selected.
fn stop_marker(painter: &egui::Painter, tip: Pos2, below: bool, fill: Color32, selected: bool) {
    let s = if below { 1.0 } else { -1.0 };
    let p = |x: f32, y: f32| tip + vec2(pt(x), pt(y * s));
    let shape = vec![
        p(0.0, 0.0),
        p(5.5, 6.0),
        p(5.5, 15.0),
        p(-5.5, 15.0),
        p(-5.5, 6.0),
    ];
    painter.add(Shape::convex_polygon(
        shape.clone(),
        Color32::from_gray(0x5a),
        Stroke::NONE,
    ));
    let inner = Rect::from_center_size(p(0.0, 10.5), vec2(pt(8.0), pt(6.5)));
    painter.rect_filled(inner, 0, fill);
    let edge = if selected {
        Color32::WHITE
    } else {
        Color32::from_gray(0x20)
    };
    painter.add(Shape::closed_line(shape, Stroke::new(pt(1.0), edge)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_are_added_moved_and_deleted() {
        let mut e = GradientEditor::new(Gradient::two("t", [0; 3], [255; 3]), Vec::new());
        // A color stop in the middle takes the gradient's color there
        let s = e.add_stop(0.5, true);
        assert_eq!(s, Selected::Color(1));
        assert_eq!(e.gradient.colors.len(), 3);
        // Moving it past the end keeps the stops in order
        let moved = e.move_stop(s, 1.0);
        assert_eq!(moved, Selected::Color(2));
        assert!(
            e.gradient
                .colors
                .windows(2)
                .all(|w| w[0].location <= w[1].location)
        );
        // Delete takes it out; two stops always stay
        e.select(Some(moved));
        e.delete();
        assert_eq!(e.gradient.colors.len(), 2);
        e.delete();
        assert_eq!(e.gradient.colors.len(), 2);
        // An opacity stop
        let o = e.add_stop(0.25, false);
        assert_eq!(o, Selected::Opacity(1));
        // The Color Picker's color for the selected color stop
        e.select(Some(Selected::Color(0)));
        e.wants_color = true;
        assert_eq!(e.take_color_request(), Some([0; 3]));
        e.set_stop_color([200, 10, 10]);
        assert_eq!(e.gradient.colors[0].color, [200, 10, 10]);
    }

    #[test]
    fn export_and_import() {
        let mut g = presets([1, 2, 3], [4, 5, 6])[5].clone();
        g.opacities[1].opacity = 0.25;
        assert_eq!(parse(&serialize(&g)), Some(g));
        assert_eq!(parse("x"), None);
    }
}
