//! Image > Adjustments > Curves... (⌘M): Photoshop 2026's classic dialog,
//! 658 × 445 pt. A Channel menu picks RGB or one channel, each with its
//! own curve; the display options on the right change only how the graph
//! is drawn. Sizes are Photoshop points from the dialog's top-left corner.

use crate::native_popup::Entry;
use egui::{Color32, Key, Mesh, Modifiers, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, vec2};
use op_core::adjust::{Adjustment, curve_table};

use super::appkit::{self, Pin};
use super::uxp::Button;
use crate::ps_icons::{self, Icon};
use crate::theme::{color, pt};

pub const SIZE: egui::Vec2 = vec2(pt(658.0), pt(445.0));
const CHANNELS: [&str; 4] = ["RGB", "Red", "Green", "Blue"];
const CHANNEL_COLORS: [Color32; 4] = [
    Color32::WHITE,
    Color32::from_rgb(0xff, 0x45, 0x45),
    Color32::from_rgb(0x45, 0xe0, 0x45),
    Color32::from_rgb(0x4d, 0x8c, 0xff),
];
/// The graph (Photoshop points).
const GRAPH: [f32; 4] = [89.0, 108.0, 346.0, 365.0];
const SHOW: [&str; 4] = [
    "Channel Overlays",
    "Histogram",
    "Baseline",
    "Intersection Line",
];

#[derive(Clone)]
pub struct Dialog {
    /// RGB, Red, Green, Blue.
    pub channel: usize,
    /// Each channel's points (input, output), sorted by input.
    pub points: [Vec<(f32, f32)>; 4],
    selected: Option<usize>,
    drag: Option<usize>,
    /// The end-point pin being dragged (0 black, 1 white).
    pin_drag: Option<usize>,
    histograms: [[u64; 256]; 4],
    /// Show Amount of: Pigment/Ink % instead of Light (0–255).
    pub pigment: bool,
    /// Grid size: ten-by-ten instead of quarters.
    pub fine_grid: bool,
    /// Channel Overlays, Histogram, Baseline, Intersection Line.
    pub show: [bool; 4],
    pub show_clipping: bool,
    /// The point under the pointer, in curve values.
    hover: Option<(f32, f32)>,
}

fn identity() -> Vec<(f32, f32)> {
    vec![(0.0, 0.0), (255.0, 255.0)]
}

impl Dialog {
    /// New histograms, keeping the settings (opening with the last ones).
    pub fn set_histograms(&mut self, channels: [[u64; 256]; 3]) {
        self.histograms = Self::new(channels).histograms;
    }

    pub fn new(channels: [[u64; 256]; 3]) -> Self {
        let mut composite = [0u64; 256];
        for h in &channels {
            for (c, v) in composite.iter_mut().zip(h) {
                *c += v;
            }
        }
        Self {
            channel: 0,
            points: std::array::from_fn(|_| identity()),
            selected: None,
            drag: None,
            pin_drag: None,
            histograms: [composite, channels[0], channels[1], channels[2]],
            pigment: false,
            fine_grid: false,
            show: [true; 4],
            show_clipping: false,
            hover: None,
        }
    }

    fn rounded(&self, c: usize) -> Vec<(u8, u8)> {
        self.points[c]
            .iter()
            .map(|&(x, y)| {
                (
                    x.round().clamp(0.0, 255.0) as u8,
                    y.round().clamp(0.0, 255.0) as u8,
                )
            })
            .collect()
    }

    pub fn adjustment(&self) -> Adjustment {
        let p: [Vec<(u8, u8)>; 4] = std::array::from_fn(|c| {
            let p = self.rounded(c);
            // An untouched channel leaves its values alone
            if p == [(0, 0), (255, 255)] {
                Vec::new()
            } else {
                p
            }
        });
        Adjustment::curves_per_channel([&p[0], &p[1], &p[2], &p[3]])
    }

    /// A preset's curves (`adjust_presets::CURVES`) as points.
    fn preset_points(p: &[&[(u8, u8)]; 4]) -> [Vec<(f32, f32)>; 4] {
        p.map(|c| c.iter().map(|&(i, o)| (i as f32, o as f32)).collect())
    }

    /// "Default" or the preset the curves match, else "Custom".
    fn preset(&self) -> &'static str {
        if self.points.iter().all(|p| *p == identity()) {
            return "Default";
        }
        super::adjust_presets::CURVES
            .iter()
            .find(|(_, p)| self.points == Self::preset_points(p))
            .map_or("Custom", |(name, _)| name)
    }

    /// Auto: each channel's darkest and lightest 0.1% become black and
    /// white (as Levels' Auto).
    pub fn auto(&mut self) {
        self.points[0] = identity();
        for c in 1..4 {
            let hist = &self.histograms[c];
            let total: u64 = hist.iter().sum();
            if total == 0 {
                continue;
            }
            let clip = total / 1000;
            let mut acc = 0;
            let lo = (0..256).find(|&i| {
                acc += hist[i];
                acc > clip
            });
            acc = 0;
            let hi = (0..256).rev().find(|&i| {
                acc += hist[i];
                acc > clip
            });
            let (lo, hi) = (lo.unwrap_or(0) as f32, hi.unwrap_or(255) as f32);
            self.points[c] = if hi > lo {
                vec![(lo, 0.0), (hi, 255.0)]
            } else {
                identity()
            };
        }
        self.selected = None;
    }

    /// A value as Show Amount of displays it.
    fn shown(&self, v: f32) -> String {
        if self.pigment {
            format!("{}", (100.0 - v / 2.55).round())
        } else {
            format!("{}", v.round())
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, frame: Rect, preview: &mut bool) -> Option<Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();

        for (k, key) in [Key::Num2, Key::Num3, Key::Num4, Key::Num5]
            .into_iter()
            .enumerate()
        {
            if ui.input_mut(|i| i.consume_key(Modifiers::ALT, key)) {
                self.channel = k;
                self.selected = None;
            }
        }

        // Preset
        appkit::label(ui, at(11.0, 56.25), "Preset:");
        let preset = self.preset();
        // Default, Custom, then Photoshop's presets
        let mut entries = vec![
            Entry::item("Default", preset == "Default"),
            Entry::item("Custom", preset == "Custom").enabled(false),
            Entry::Separator,
        ];
        entries.extend(
            super::adjust_presets::CURVES
                .iter()
                .map(|(name, _)| Entry::item(*name, preset == *name)),
        );
        match appkit::popup(
            ui,
            r(56.0, 46.0, 345.0, 67.0),
            "curves-preset",
            preset,
            &entries,
        ) {
            Some(0) => {
                self.points = std::array::from_fn(|_| identity());
                self.selected = None;
            }
            Some(k) if k >= 3 => {
                self.points = Self::preset_points(&super::adjust_presets::CURVES[k - 3].1);
                self.selected = None;
            }
            _ => {}
        }
        ps_icons::paint_scaled(
            &painter,
            at(365.5, 56.5),
            Icon::Gear,
            appkit::TEXT,
            color::PANEL,
            0.77,
        );

        // The channel group
        appkit::group(
            &painter,
            r(11.0, 87.5, 368.0, 434.5),
            (at(25.5, 0.0).x, at(172.5, 0.0).x),
        );
        appkit::label(ui, at(30.5, 86.75), "Channel:");
        let channels: Vec<Entry> = CHANNELS
            .iter()
            .enumerate()
            .map(|(k, name)| Entry::item(format!("{name}    ⌥{}", k + 2), k == self.channel))
            .collect();
        let channel = appkit::popup(
            ui,
            r(85.0, 76.5, 170.0, 97.5),
            "curves-channel",
            CHANNELS[self.channel],
            &channels,
        )
        .unwrap_or(self.channel);
        if channel != self.channel {
            self.channel = channel;
            self.selected = None;
        }

        // Point and pencil tools (the pencil is not available)
        let tool = r(20.0, 107.5, 50.0, 133.5);
        painter.rect(
            tool,
            pt(3.0),
            Color32::from_gray(0x38),
            Stroke::new(pt(1.0), Color32::from_gray(0x63)),
            StrokeKind::Inside,
        );
        ps_icons::paint(
            &painter,
            tool.center(),
            Icon::CurvePoints,
            appkit::TEXT,
            Color32::from_gray(0x38),
        );
        ps_icons::paint(
            &painter,
            at(64.5, 120.0),
            Icon::Pencil,
            appkit::TEXT,
            color::PANEL,
        );

        self.graph(ui, frame);

        // Readouts: the selected point, or where the pointer is
        let readout = self
            .selected
            .and_then(|i| self.points[self.channel].get(i).copied())
            .or(self.hover);
        appkit::label(ui, at(20.5, 339.75), "Output:");
        appkit::label(ui, at(90.0, 389.75), "Input:");
        if let Some((x, y)) = readout {
            appkit::label(ui, at(20.5, 355.75), &self.shown(y));
            appkit::label(ui, at(89.5, 410.25), &self.shown(x));
        }
        ps_icons::paint_scaled(
            &painter,
            at(34.0, 412.0),
            Icon::TargetedHandVertical,
            appkit::TEXT,
            color::PANEL,
            1.4,
        );
        for (x, icon) in [
            (158.0, Icon::EyedropperBlack),
            (188.0, Icon::EyedropperGray),
            (218.0, Icon::EyedropperWhite),
        ] {
            ps_icons::paint_scaled(
                &painter,
                at(x, 411.5),
                icon,
                Color32::from_gray(0xdd),
                color::PANEL,
                1.1,
            );
        }
        appkit::checkbox(
            ui,
            at(240.5, 404.5),
            "Show Clipping",
            &mut self.show_clipping,
        );

        // Display options
        appkit::group(
            &painter,
            r(394.0, 46.5, 547.0, 122.5),
            (at(406.0, 0.0).x, at(511.0, 0.0).x),
        );
        appkit::label(ui, at(413.5, 45.25), "Show Amount of:");
        if appkit::radio(ui, at(410.0, 71.25), "Light  (0-255)", !self.pigment) {
            self.pigment = false;
        }
        if appkit::radio(ui, at(410.0, 105.25), "Pigment/Ink %", self.pigment) {
            self.pigment = true;
        }
        appkit::group(
            &painter,
            r(394.0, 147.5, 547.0, 203.5),
            (at(406.0, 0.0).x, at(468.0, 0.0).x),
        );
        appkit::label(ui, at(413.5, 146.25), "Grid size:");
        for (k, (cx, icon)) in [(416.0, Icon::GridQuarters), (442.0, Icon::GridTenths)]
            .into_iter()
            .enumerate()
        {
            let rect = Rect::from_center_size(at(cx, 181.0), vec2(pt(25.0), pt(25.0)));
            let chosen = self.fine_grid == (k == 1);
            if chosen {
                painter.rect(
                    rect,
                    pt(3.0),
                    Color32::from_gray(0x38),
                    Stroke::new(pt(1.0), Color32::from_gray(0x63)),
                    StrokeKind::Inside,
                );
            }
            let tint = if chosen {
                appkit::TEXT
            } else {
                Color32::from_gray(0xc8)
            };
            ps_icons::paint_scaled(&painter, rect.center(), icon, tint, color::PANEL, 0.75);
            if ui
                .interact(rect, ui.id().with(("curves-grid", k)), Sense::click())
                .clicked()
            {
                self.fine_grid = k == 1;
            }
        }
        appkit::group(
            &painter,
            r(394.0, 228.0, 547.0, 351.0),
            (at(406.0, 0.0).x, at(450.0, 0.0).x),
        );
        appkit::label(ui, at(413.5, 226.75), "Show:");
        for (k, y) in [246.0, 272.75, 299.5, 326.25].into_iter().enumerate() {
            appkit::checkbox(ui, at(402.5, y), SHOW[k], &mut self.show[k]);
        }

        // Buttons and Preview
        let button = |ui: &mut Ui, y: f32, label: &str, default: bool, enabled: bool| {
            appkit::button(ui, r(563.5, y, 647.5, y + 26.0), label, default, enabled)
        };
        let ok = button(ui, 45.0, "OK", true, true);
        let cancel = button(ui, 80.0, "Cancel", false, true);
        button(ui, 122.0, "Smooth", false, false);
        let auto = button(ui, 164.0, "Auto", false, true);
        button(ui, 199.0, "Options...", false, true);
        appkit::checkbox(ui, at(563.5, 243.5), "Preview", preview);

        if cancel.clicked() {
            return Some(Button::Cancel);
        }
        if auto.clicked() {
            self.auto();
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        (ok.clicked() || enter).then_some(Button::Ok)
    }

    /// The graph: histogram, grid, baseline, overlays, the curve and its
    /// points, the ramps along its edges and the end-point pins. Click to
    /// add a point, drag to move it (a point stays between its
    /// neighbours), drag it off the graph to remove it (the end points
    /// stay); arrows nudge the selected point, Delete removes it.
    fn graph(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let graph = Rect::from_min_max(at(GRAPH[0], GRAPH[1]), at(GRAPH[2], GRAPH[3]));
        let painter = ui.painter_at(frame);
        let c = self.channel;
        let flip = self.pigment;
        // Curve values (0–255, output up) to the screen and back; Pigment
        // turns both axes around
        let to_screen = |(x, y): (f32, f32)| {
            let (x, y) = if flip { (255.0 - x, 255.0 - y) } else { (x, y) };
            Pos2::new(
                graph.left() + x / 255.0 * graph.width(),
                graph.bottom() - y / 255.0 * graph.height(),
            )
        };
        let to_curve = |p: Pos2| {
            let x = ((p.x - graph.left()) / graph.width() * 255.0).clamp(0.0, 255.0);
            let y = ((graph.bottom() - p.y) / graph.height() * 255.0).clamp(0.0, 255.0);
            if flip { (255.0 - x, 255.0 - y) } else { (x, y) }
        };

        painter.rect(
            graph,
            0,
            appkit::FIELD,
            Stroke::new(pt(1.0), Color32::from_gray(0x30)),
            StrokeKind::Outside,
        );
        if self.show[1] {
            let hist = &self.histograms[c];
            let max = hist.iter().copied().max().unwrap_or(0).max(1) as f32;
            for (i, &n) in hist.iter().enumerate() {
                if n == 0 {
                    continue;
                }
                let x0 = to_screen((i as f32, 0.0)).x;
                let x1 = to_screen((i as f32 + 1.0, 0.0)).x;
                let h = n as f32 / max * graph.height();
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(x0.min(x1), graph.bottom() - h),
                        Pos2::new(x0.max(x1), graph.bottom()),
                    ),
                    0,
                    Color32::from_gray(0x86),
                );
            }
        }
        let grid = Stroke::new(pt(1.0), Color32::from_gray(0x38));
        let lines = if self.fine_grid { 10 } else { 4 };
        for k in 1..lines {
            let t = k as f32 / lines as f32;
            let x = graph.left() + graph.width() * t;
            let y = graph.top() + graph.height() * t;
            painter.line_segment(
                [Pos2::new(x, graph.top()), Pos2::new(x, graph.bottom())],
                grid,
            );
            painter.line_segment(
                [Pos2::new(graph.left(), y), Pos2::new(graph.right(), y)],
                grid,
            );
        }
        if self.show[2] {
            painter.line_segment(
                [to_screen((0.0, 0.0)), to_screen((255.0, 255.0))],
                Stroke::new(pt(1.0), Color32::from_gray(0x80)),
            );
        }

        // Pointer: add, select and drag points
        let id = ui.id().with("curves-graph");
        let response = ui.interact(graph.expand(pt(4.0)), id, Sense::click_and_drag());
        self.hover = response
            .hover_pos()
            .filter(|p| graph.contains(*p))
            .map(to_curve);
        let pts = &mut self.points[c];
        if (response.drag_started() || response.clicked())
            && let Some(p) = response.interact_pointer_pos()
        {
            let near = pts
                .iter()
                .position(|&q| to_screen(q).distance(p) <= pt(5.0));
            let index = near.unwrap_or_else(|| {
                // A new point where the pointer is (the curve jumps to it)
                let point = to_curve(p);
                let x = point.0;
                let i = pts.iter().position(|&(px, _)| px > x).unwrap_or(pts.len());
                if pts.iter().any(|&(px, _)| (px - x).abs() < 1.0) || pts.len() >= 16 {
                    return pts
                        .iter()
                        .position(|&(px, _)| (px - x).abs() < 1.0)
                        .unwrap_or(0);
                }
                pts.insert(i, point);
                i
            });
            self.selected = Some(index);
            self.drag = response.drag_started().then_some(index);
        }
        if let (Some(i), Some(p)) = (self.drag, response.interact_pointer_pos()) {
            let (mut x, y) = to_curve(p);
            let lo = if i > 0 { pts[i - 1].0 + 1.0 } else { 0.0 };
            let hi = if i + 1 < pts.len() {
                pts[i + 1].0 - 1.0
            } else {
                255.0
            };
            x = x.clamp(lo, hi.max(lo));
            if i < pts.len() {
                pts[i] = (x, y);
            }
            if response.drag_stopped() {
                let outside = !graph.expand(pt(20.0)).contains(p);
                if outside && i != 0 && i + 1 != pts.len() {
                    pts.remove(i);
                    self.selected = None;
                }
                self.drag = None;
            }
        }
        if let Some(i) = self.selected.filter(|&i| i < pts.len()) {
            let (dx, dy, delete, shift) = ui.input(|inp| {
                let k = |key| inp.key_pressed(key) as i32 as f32;
                (
                    k(Key::ArrowRight) - k(Key::ArrowLeft),
                    k(Key::ArrowUp) - k(Key::ArrowDown),
                    inp.key_pressed(Key::Delete) || inp.key_pressed(Key::Backspace),
                    inp.modifiers.shift,
                )
            });
            let step = if shift { 10.0 } else { 1.0 };
            let (dx, dy) = if flip { (-dx, -dy) } else { (dx, dy) };
            if dx != 0.0 || dy != 0.0 {
                let lo = if i > 0 { pts[i - 1].0 + 1.0 } else { 0.0 };
                let hi = if i + 1 < pts.len() {
                    pts[i + 1].0 - 1.0
                } else {
                    255.0
                };
                let (x, y) = pts[i];
                pts[i] = (
                    (x + dx * step).clamp(lo, hi),
                    (y + dy * step).clamp(0.0, 255.0),
                );
            }
            if delete && i != 0 && i + 1 != pts.len() {
                pts.remove(i);
                self.selected = None;
            }
        }

        // Other channels' curves in their colors, then this one
        let curve_line = |points: &[(f32, f32)], color: Color32, width: f32| {
            let rounded: Vec<(u8, u8)> = points
                .iter()
                .map(|&(x, y)| (x.round() as u8, y.round() as u8))
                .collect();
            let table = curve_table(&rounded);
            let line: Vec<Pos2> = (0..256)
                .map(|x| to_screen((x as f32, table[x] as f32)))
                .collect();
            painter.add(Shape::line(line, Stroke::new(width, color)));
        };
        if self.show[0] {
            for (k, color) in CHANNEL_COLORS.into_iter().enumerate() {
                if k != c && self.points[k] != identity() {
                    curve_line(&self.points[k], color, pt(1.0));
                }
            }
        }
        curve_line(&self.points[c], CHANNEL_COLORS[c], pt(1.75));
        let pts = &self.points[c];
        if self.show[3]
            && let Some(i) = self.drag
            && let Some(&q) = pts.get(i)
        {
            let p = to_screen(q);
            let s = Stroke::new(pt(0.5), Color32::from_gray(0xc0));
            painter.line_segment(
                [Pos2::new(graph.left(), p.y), Pos2::new(graph.right(), p.y)],
                s,
            );
            painter.line_segment(
                [Pos2::new(p.x, graph.top()), Pos2::new(p.x, graph.bottom())],
                s,
            );
        }
        for (i, &q) in pts.iter().enumerate() {
            let rect = Rect::from_center_size(to_screen(q), vec2(pt(4.0), pt(4.0)));
            if self.selected == Some(i) {
                painter.rect_filled(rect, 0, appkit::TEXT);
            } else {
                painter.rect(
                    rect,
                    0,
                    appkit::FIELD,
                    Stroke::new(pt(1.0), appkit::TEXT),
                    StrokeKind::Inside,
                );
            }
        }

        // The ramps: output on the left (white at the top in Light), input
        // along the bottom, with the end-point pins
        let (light, dark) = if flip {
            (Color32::BLACK, Color32::WHITE)
        } else {
            (Color32::WHITE, Color32::BLACK)
        };
        // A ramp from `a` to `b`, top to bottom or left to right
        let gradient = |rect: Rect, a: Color32, b: Color32, vertical: bool| {
            let mut mesh = Mesh::default();
            let (tr, bl) = if vertical { (a, b) } else { (b, a) };
            mesh.colored_vertex(rect.left_top(), a);
            mesh.colored_vertex(rect.right_top(), tr);
            mesh.colored_vertex(rect.right_bottom(), b);
            mesh.colored_vertex(rect.left_bottom(), bl);
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(0, 2, 3);
            painter.add(Shape::mesh(mesh));
        };
        gradient(
            Rect::from_min_max(at(84.0, GRAPH[1]), at(87.5, GRAPH[3])),
            light,
            dark,
            true,
        );
        gradient(
            Rect::from_min_max(at(GRAPH[0], 366.0), at(GRAPH[2], 369.5)),
            dark,
            light,
            false,
        );
        let first = pts.first().copied().unwrap_or((0.0, 0.0));
        let last = pts.last().copied().unwrap_or((255.0, 255.0));
        let pin_y = at(0.0, 369.5).y;
        let pins = [to_screen((first.0, 0.0)).x, to_screen((last.0, 0.0)).x];
        let area = Rect::from_min_max(
            Pos2::new(graph.left() - pt(6.0), pin_y),
            Pos2::new(graph.right() + pt(6.0), pin_y + pt(11.0)),
        );
        let response = ui.interact(area, ui.id().with("curves-pins"), Sense::drag());
        if response.drag_started()
            && let Some(p) = response.interact_pointer_pos()
        {
            self.pin_drag = Some(((pins[1] - p.x).abs() < (pins[0] - p.x).abs()) as usize);
        }
        if let (Some(k), Some(p)) = (self.pin_drag, response.interact_pointer_pos()) {
            let (x, _) = to_curve(p);
            let pts = &mut self.points[c];
            let n = pts.len();
            if k == 0 {
                let hi = pts[1].0 - 1.0;
                pts[0].0 = x.min(hi);
            } else {
                let lo = pts[n - 2].0 + 1.0;
                pts[n - 1].0 = x.max(lo);
            }
            if response.drag_stopped() {
                self.pin_drag = None;
            }
        }
        let (black, white) = if flip {
            (Pin::White, Pin::Black)
        } else {
            (Pin::Black, Pin::White)
        };
        let painter = ui.painter();
        appkit::pin(painter, Pos2::new(pins[0], pin_y), black);
        appkit::pin(painter, Pos2::new(pins[1], pin_y), white);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photoshops_presets() {
        let mut d = Dialog::new([[0; 256]; 3]);
        let negative = super::super::adjust_presets::CURVES
            .iter()
            .find(|(n, _)| *n == "Negative")
            .unwrap();
        d.points = Dialog::preset_points(&negative.1);
        assert_eq!(d.preset(), "Negative");
        assert_eq!(d.points[0], vec![(0.0, 255.0), (255.0, 0.0)]);
        d.points[0][0].1 = 250.0;
        assert_eq!(d.preset(), "Custom");
    }

    #[test]
    fn channels_keep_their_curves() {
        let mut d = Dialog::new([[1; 256]; 3]);
        assert_eq!(
            d.adjustment(),
            Adjustment::curves_per_channel([&[], &[], &[], &[]])
        );
        assert_eq!(d.preset(), "Default");
        d.points[2].insert(1, (64.0, 128.0));
        assert_eq!(d.preset(), "Custom");
        assert_eq!(
            d.adjustment(),
            Adjustment::curves_per_channel([&[], &[], &[(0, 0), (64, 128), (255, 255)], &[]])
        );
    }

    #[test]
    fn pigment_shows_ink_percent() {
        let mut d = Dialog::new([[0; 256]; 3]);
        assert_eq!(d.shown(255.0), "255");
        d.pigment = true;
        assert_eq!(
            (d.shown(255.0), d.shown(0.0)),
            ("0".to_string(), "100".to_string())
        );
    }

    #[test]
    fn auto_moves_each_channels_end_points() {
        let mut h = [0u64; 256];
        h[30] = 10;
        h[220] = 10;
        let mut d = Dialog::new([h, h, h]);
        d.auto();
        assert_eq!(d.points[1], vec![(30.0, 0.0), (220.0, 255.0)]);
        assert_eq!(d.points[0], identity());
    }
}
