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
    /// The Set Black, Gray or White Point eyedropper chosen (`sample`).
    pub eyedropper: Option<usize>,
    /// The pencil tool: the curves are drawn freehand as tables (one per
    /// channel), until the point tool turns them back into points.
    pub tables: Option<[[u8; 256]; 4]>,
    /// The pencil stroke's last point (curve values).
    last_pencil: Option<(f32, f32)>,
    /// The value of the image pixel under the pointer on the current
    /// channel (the mean of the three on RGB): a circle marks it on the
    /// curve.
    pub probe: Option<f32>,
    /// The selected point's Input and Output as typed, and the channel and
    /// point they were filled from.
    input_text: String,
    output_text: String,
    shown_point: Option<(usize, usize)>,
    /// Show Clipping while an end-point pin is dragged: the preview shows
    /// what clips to black (false) or white (true).
    pub clipping: Option<bool>,
    /// The targeted adjustment hand is on: a drag on the image moves the
    /// curve at the pressed pixel's value (`target_press`).
    pub targeting: bool,
    /// The hand's point (index in the current channel) and its output when
    /// pressed.
    target: Option<(usize, f32)>,
    /// Auto or Options... was clicked (`AdjustDialog` runs them).
    pub wants_auto: bool,
    pub wants_options: bool,
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
            eyedropper: None,
            tables: None,
            last_pencil: None,
            probe: None,
            input_text: String::new(),
            output_text: String::new(),
            shown_point: None,
            clipping: None,
            targeting: false,
            target: None,
            wants_auto: false,
            wants_options: false,
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

    /// The curves as tables (the pencil's, or the points' splines).
    fn current_tables(&self) -> [[u8; 256]; 4] {
        self.tables
            .unwrap_or_else(|| std::array::from_fn(|c| curve_table(&self.rounded(c))))
    }

    pub fn adjustment(&self) -> Adjustment {
        // Show Clipping while a pin is dragged: white (black) where no
        // channel clips, the clipped channels' colors elsewhere
        if let Some(highlights) = self.clipping {
            let t = self.current_tables();
            let comp: [[u8; 256]; 3] =
                std::array::from_fn(|c| std::array::from_fn(|v| t[0][t[c + 1][v] as usize]));
            let show = |table: &[u8; 256]| -> [u8; 256] {
                std::array::from_fn(|v| match (highlights, table[v]) {
                    (false, 0) => 0,
                    (false, _) => 255,
                    (true, 255) => 255,
                    (true, _) => 0,
                })
            };
            let identity: [u8; 256] = std::array::from_fn(|v| v as u8);
            return Adjustment::CurveTables([
                identity,
                show(&comp[0]),
                show(&comp[1]),
                show(&comp[2]),
            ]);
        }
        if let Some(t) = self.tables {
            return Adjustment::CurveTables(t);
        }
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

    /// The curves as a Curves preset file. Pencil-drawn curves go as the
    /// points the point tool would turn them into.
    fn encoded(&self) -> Vec<u8> {
        let points = match self.tables {
            Some(_) => {
                let mut copy = Self {
                    tables: self.tables,
                    points: self.points.clone(),
                    ..Self::new([[0; 256]; 3])
                };
                copy.set_pencil(false);
                copy.points
            }
            None => self.points.clone(),
        };
        super::preset_files::encode_curves(&points)
    }

    /// A preset file's curves (the point tool again).
    fn load_preset(&mut self, bytes: &[u8]) {
        if let Some(points) = super::preset_files::decode_curves(bytes) {
            self.tables = None;
            self.points = points;
            self.selected = None;
        }
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

    /// Picks the pencil (the curves become tables to draw on) or the point
    /// tool (the tables become points again, placed along the drawn curve
    /// every 32 levels).
    pub fn set_pencil(&mut self, on: bool) {
        if on && self.tables.is_none() {
            self.tables = Some(self.current_tables());
        } else if !on && let Some(t) = self.tables.take() {
            self.points = t.map(|table| {
                let pts: Vec<(f32, f32)> = (0..=8)
                    .map(|k| {
                        let x = (k * 32).min(255);
                        (x as f32, table[x] as f32)
                    })
                    .collect();
                // A straight table is the identity's two points
                if pts.iter().all(|&(x, y)| (x - y).abs() < 1.0) {
                    identity()
                } else {
                    pts
                }
            });
        }
        self.selected = None;
    }

    /// Smooth (pencil only): the current channel's table averaged over
    /// nine levels, as Photoshop's Smooth evens a drawn curve.
    pub fn smooth(&mut self) {
        let c = self.channel;
        if let Some(t) = &mut self.tables {
            let src = t[c];
            for (v, out) in t[c].iter_mut().enumerate() {
                let lo = v.saturating_sub(4);
                let hi = (v + 4).min(255);
                let sum: u32 = src[lo..=hi].iter().map(|&x| x as u32).sum();
                *out = (sum as f32 / (hi - lo + 1) as f32).round() as u8;
            }
        }
    }

    /// The chosen eyedropper's click on a pixel of color `rgb`: Set Black
    /// (White) Point moves each channel's black (white) end to that
    /// channel's value; Set Gray Point adds a point taking each channel's
    /// value to the color's mean, so it comes out neutral.
    pub fn sample(&mut self, rgb: [u8; 3]) {
        let Some(k) = self.eyedropper else {
            return;
        };
        let target = rgb.iter().map(|&v| v as f32).sum::<f32>() / 3.0;
        self.points[0] = identity();
        for (c, &value) in rgb.iter().enumerate() {
            let v = value as f32;
            let points = &mut self.points[c + 1];
            match k {
                0 => {
                    points.retain(|p| p.0 > v);
                    points.insert(0, (v.min(253.0), 0.0));
                }
                2 => {
                    points.retain(|p| p.0 < v);
                    points.push((v.max(2.0), 255.0));
                }
                _ => {
                    points.retain(|p| (p.0 - v).abs() >= 4.0);
                    let i = points.iter().position(|p| p.0 > v).unwrap_or(points.len());
                    points.insert(i, (v, target));
                }
            }
        }
        self.selected = None;
    }

    /// The hand pressed on a pixel of color `rgb`: the current channel's
    /// value there (the mean of the three on RGB) gets a point on the curve
    /// (or the point already within 4 levels), which the drag then moves.
    pub fn target_press(&mut self, rgb: [u8; 3]) {
        self.set_pencil(false);
        let v = if self.channel == 0 {
            (rgb.iter().map(|&v| v as f32).sum::<f32>() / 3.0).round()
        } else {
            rgb[self.channel - 1] as f32
        };
        let table = curve_table(&self.rounded(self.channel));
        let points = &mut self.points[self.channel];
        let i = match points.iter().position(|p| (p.0 - v).abs() <= 4.0) {
            Some(i) => i,
            None if points.len() < 16 => {
                let i = points.iter().position(|p| p.0 > v).unwrap_or(points.len());
                points.insert(i, (v, table[v as usize] as f32));
                i
            }
            None => return,
        };
        self.selected = Some(i);
        self.target = Some((i, points[i].1));
    }

    /// The image pixel under the pointer (None off the image): its value
    /// on the current channel is marked on the curve.
    pub fn set_probe(&mut self, rgb: Option<[u8; 3]>) {
        self.probe = rgb.map(|rgb| self.value_of(rgb));
    }

    /// The current channel's value of `rgb` (the mean of the three on RGB).
    fn value_of(&self, rgb: [u8; 3]) -> f32 {
        if self.channel == 0 {
            (rgb.iter().map(|&v| v as f32).sum::<f32>() / 3.0).round()
        } else {
            rgb[self.channel - 1] as f32
        }
    }

    /// ⌘-click on the image: a point on the current curve at the pixel's
    /// value, where the curve is now; with ⇧, one on each of the red,
    /// green and blue curves at that channel's value. A channel with 16
    /// points or a point within 4 levels gets no new one.
    pub fn add_points_at(&mut self, rgb: [u8; 3], each_channel: bool) {
        self.set_pencil(false);
        let targets: Vec<(usize, f32)> = if each_channel {
            (1..4).map(|c| (c, rgb[c - 1] as f32)).collect()
        } else {
            vec![(self.channel, self.value_of(rgb))]
        };
        for (c, v) in targets {
            let table = curve_table(&self.rounded(c));
            let points = &mut self.points[c];
            if points.len() >= 16 || points.iter().any(|p| (p.0 - v).abs() <= 4.0) {
                continue;
            }
            let i = points.iter().position(|p| p.0 > v).unwrap_or(points.len());
            points.insert(i, (v, table[v as usize] as f32));
            if c == self.channel {
                self.selected = Some(i);
            }
        }
    }

    /// The hand dragged `dy` points down from where it was pressed: the
    /// point's output goes up a level per point dragged up.
    pub fn target_drag(&mut self, dy: f32) {
        if let Some((i, start)) = self.target
            && let Some(p) = self.points[self.channel].get_mut(i)
        {
            p.1 = (start - dy).round().clamp(0.0, 255.0);
        }
    }

    pub fn target_release(&mut self) {
        self.target = None;
    }

    /// Auto (computed by `op_core::auto` with the Auto Color Correction
    /// Options): each channel's black and white become end points and its
    /// gamma a point halfway between them; the same curve for all three
    /// channels goes on the composite.
    pub fn auto(&mut self, channels: [op_core::auto::Channel; 3]) {
        let curve = |c: &op_core::auto::Channel| {
            let mut p = vec![
                (c.black.round(), c.out_black as f32),
                (c.white.round(), c.out_white as f32),
            ];
            if (c.gamma - 1.0).abs() > 0.005 {
                let x = ((c.black + c.white) / 2.0).round();
                p.insert(1, (x, c.map(x).round()));
            }
            p
        };
        self.points = std::array::from_fn(|_| identity());
        self.tables = None;
        if channels[0] == channels[1] && channels[1] == channels[2] {
            self.points[0] = curve(&channels[0]);
        } else {
            for (points, c) in self.points[1..].iter_mut().zip(&channels) {
                *points = curve(c);
            }
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
        use super::preset_files as files;
        let encoded = self.encoded();
        let saved_name = files::shown(files::CURVES, Some(&encoded));
        let preset = saved_name.as_deref().unwrap_or(self.preset());
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
        let before = entries.len();
        let saved = files::add_saved(&mut entries, files::CURVES, preset);
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
            Some(k) if k >= before => {
                if let Some(bytes) = files::picked_saved(k, before, &saved, files::CURVES) {
                    self.load_preset(&bytes);
                }
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
        let gear = Rect::from_center_size(at(365.5, 56.5), vec2(pt(18.0), pt(18.0)));
        if let Some(bytes) = files::gear(ui, gear, "curves-gear", files::CURVES, Some(encoded)) {
            self.load_preset(&bytes);
        }

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

        // Point and pencil tools, the chosen one pressed
        let pencil = self.tables.is_some();
        for (k, (rect, icon)) in [
            (r(20.0, 107.5, 50.0, 133.5), Icon::CurvePoints),
            (r(49.5, 107.5, 79.5, 133.5), Icon::Pencil),
        ]
        .into_iter()
        .enumerate()
        {
            let chosen = pencil == (k == 1);
            let fill = if chosen {
                painter.rect(
                    rect,
                    pt(3.0),
                    Color32::from_gray(0x38),
                    Stroke::new(pt(1.0), Color32::from_gray(0x63)),
                    StrokeKind::Inside,
                );
                Color32::from_gray(0x38)
            } else {
                color::PANEL
            };
            ps_icons::paint(&painter, rect.center(), icon, appkit::TEXT, fill);
            if ui
                .interact(rect, ui.id().with(("curves-tool", k)), Sense::click())
                .clicked()
                && !chosen
            {
                self.set_pencil(k == 1);
            }
        }

        self.graph(ui, frame);

        // Readouts: the selected point, or where the pointer is
        let readout = self
            .selected
            .and_then(|i| self.points[self.channel].get(i).copied())
            .or(self.hover);
        appkit::label(ui, at(20.5, 339.75), "Output:");
        appkit::label(ui, at(90.0, 389.75), "Input:");
        // A selected point's values can be typed
        let selected = self
            .selected
            .filter(|&i| self.tables.is_none() && i < self.points[self.channel].len())
            .map(|i| (self.channel, i));
        if let Some((c, i)) = selected {
            if self.shown_point != Some((c, i)) || self.drag.is_some() {
                let (x, y) = self.points[c][i];
                self.input_text = self.shown(x);
                self.output_text = self.shown(y);
                self.shown_point = Some((c, i));
            }
            let output = appkit::field(
                ui,
                r(18.0, 346.5, 62.0, 365.5),
                &mut self.output_text,
                "curves-output",
                (0.0, 255.0),
                1.0,
                0,
                false,
            );
            let input = appkit::field(
                ui,
                r(87.5, 401.0, 131.5, 420.0),
                &mut self.input_text,
                "curves-input",
                (0.0, 255.0),
                1.0,
                0,
                false,
            );
            let pigment = self.pigment;
            let read = |t: &str| {
                let v: f32 = t.trim().parse().ok()?;
                Some(if pigment { 255.0 - v * 2.55 } else { v }.clamp(0.0, 255.0))
            };
            let (out_v, in_v) = (read(&self.output_text), read(&self.input_text));
            let pts = &mut self.points[c];
            if output.changed()
                && let Some(y) = out_v
            {
                pts[i].1 = y;
            }
            if input.changed()
                && let Some(x) = in_v
            {
                let lo = if i > 0 { pts[i - 1].0 + 1.0 } else { 0.0 };
                let hi = if i + 1 < pts.len() {
                    pts[i + 1].0 - 1.0
                } else {
                    255.0
                };
                pts[i].0 = x.clamp(lo, hi.max(lo));
            }
        } else {
            self.shown_point = None;
            if let Some((x, y)) = readout {
                appkit::label(ui, at(20.5, 355.75), &self.shown(y));
                appkit::label(ui, at(89.5, 410.25), &self.shown(x));
            }
        }
        // The targeted adjustment hand: a toggle, pressed while on
        let hand = Rect::from_center_size(at(34.0, 412.0), vec2(pt(30.0), pt(26.0)));
        let fill = if self.targeting {
            painter.rect(
                hand,
                pt(3.0),
                Color32::from_gray(0x38),
                Stroke::new(pt(1.0), Color32::from_gray(0x63)),
                StrokeKind::Inside,
            );
            Color32::from_gray(0x38)
        } else {
            color::PANEL
        };
        ps_icons::paint_scaled(
            &painter,
            at(34.0, 412.0),
            Icon::TargetedHandVertical,
            appkit::TEXT,
            fill,
            1.4,
        );
        if ui
            .interact(hand, ui.id().with("curves-hand"), Sense::click())
            .clicked()
        {
            self.targeting = !self.targeting;
            self.eyedropper = None;
        }
        for (k, (x, icon)) in [
            (158.0, Icon::EyedropperBlack),
            (188.0, Icon::EyedropperGray),
            (218.0, Icon::EyedropperWhite),
        ]
        .into_iter()
        .enumerate()
        {
            let chosen = self.eyedropper == Some(k);
            if appkit::eyedropper(ui, at(x, 411.5), icon, chosen, 1.1) {
                self.eyedropper = (!chosen).then_some(k);
                self.targeting = false;
            }
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
        if button(ui, 122.0, "Smooth", false, self.tables.is_some()).clicked() {
            self.smooth();
        }
        let auto = button(ui, 164.0, "Auto", false, true);
        if button(ui, 199.0, "Options...", false, true).clicked() {
            self.wants_options = true;
        }
        appkit::checkbox(ui, at(563.5, 243.5), "Preview", preview);

        if cancel.clicked() {
            return Some(Button::Cancel);
        }
        if auto.clicked() {
            self.wants_auto = true;
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
        // The pencil draws the curve: each column the pointer crosses
        // takes its height
        if let Some(tables) = &mut self.tables {
            let drawing = response.dragged() || response.clicked() || response.drag_started();
            if let Some(p) = response.interact_pointer_pos().filter(|_| drawing) {
                let (x, y) = to_curve(p);
                let (x0, y0) = self.last_pencil.unwrap_or((x, y));
                let (a, b) = if x0 <= x {
                    ((x0, y0), (x, y))
                } else {
                    ((x, y), (x0, y0))
                };
                for col in a.0.round() as usize..=b.0.round() as usize {
                    let t = if b.0 > a.0 {
                        (col as f32 - a.0) / (b.0 - a.0)
                    } else {
                        0.0
                    };
                    tables[c][col.min(255)] = (a.1 + (b.1 - a.1) * t.clamp(0.0, 1.0)).round() as u8;
                }
                self.last_pencil = Some((x, y));
            }
            if !response.dragged() {
                self.last_pencil = None;
            }
            let line: Vec<Pos2> = (0..256)
                .map(|x| to_screen((x as f32, tables[c][x] as f32)))
                .collect();
            painter.add(Shape::line(line, Stroke::new(pt(1.75), CHANNEL_COLORS[c])));
        }
        let pencil = self.tables.is_some();
        if !pencil {
            self.point_tool(
                ui,
                &response,
                &painter,
                (c, flip),
                (&to_screen, &to_curve),
                graph,
            );
        }
        // The pixel under the pointer on the image: a circle on the curve
        // at its value
        if let Some(v) = self.probe {
            let v = v.round().clamp(0.0, 255.0) as usize;
            let out = match &self.tables {
                Some(t) => t[c][v],
                None => curve_table(&self.rounded(c))[v],
            };
            painter.circle_stroke(
                to_screen((v as f32, out as f32)),
                pt(4.0),
                Stroke::new(pt(1.0), appkit::TEXT),
            );
        }
        self.ramps_and_pins(
            ui,
            &painter,
            frame,
            (c, flip, pencil),
            (&to_screen, &to_curve),
            graph,
        );
    }

    /// The point tool's part of the graph: adding, selecting, dragging and
    /// deleting points, the curves and the points.
    fn point_tool(
        &mut self,
        ui: &Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        (c, flip): (usize, bool),
        (to_screen, to_curve): (&dyn Fn((f32, f32)) -> Pos2, &dyn Fn(Pos2) -> (f32, f32)),
        graph: Rect,
    ) {
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
    }

    /// The ramps: output on the left (white at the top in Light), input
    /// along the bottom, with the end-point pins (fixed at the ends while
    /// the pencil draws).
    fn ramps_and_pins(
        &mut self,
        ui: &Ui,
        painter: &egui::Painter,
        frame: Rect,
        (c, flip, pencil): (usize, bool, bool),
        (to_screen, to_curve): (&dyn Fn((f32, f32)) -> Pos2, &dyn Fn(Pos2) -> (f32, f32)),
        graph: Rect,
    ) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let pts = &self.points[c];
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
        let (first, last) = if pencil {
            ((0.0, 0.0), (255.0, 255.0))
        } else {
            (
                pts.first().copied().unwrap_or((0.0, 0.0)),
                pts.last().copied().unwrap_or((255.0, 255.0)),
            )
        };
        let pin_y = at(0.0, 369.5).y;
        let pins = [to_screen((first.0, 0.0)).x, to_screen((last.0, 0.0)).x];
        let area = Rect::from_min_max(
            Pos2::new(graph.left() - pt(6.0), pin_y),
            Pos2::new(graph.right() + pt(6.0), pin_y + pt(11.0)),
        );
        let sense = if pencil {
            Sense::hover()
        } else {
            Sense::drag()
        };
        let response = ui.interact(area, ui.id().with("curves-pins"), sense);
        if response.drag_started()
            && let Some(p) = response.interact_pointer_pos()
        {
            self.pin_drag = Some(((pins[1] - p.x).abs() < (pins[0] - p.x).abs()) as usize);
        }
        // Show Clipping: the preview shows what the dragged pin clips
        self.clipping = self
            .pin_drag
            .filter(|_| self.show_clipping)
            .map(|k| (k == 1) != flip);
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
                self.clipping = None;
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
    fn command_click_adds_points() {
        let mut d = Dialog::new([[0; 256]; 3]);
        // ⌘-click on RGB: one point at the mean, on the curve
        d.add_points_at([30, 60, 90], false);
        assert_eq!(d.points[0], vec![(0.0, 0.0), (60.0, 60.0), (255.0, 255.0)]);
        assert_eq!(d.selected, Some(1));
        // ⌘⇧-click: each color channel at its own value
        d.add_points_at([30, 60, 90], true);
        assert_eq!(d.points[1][1], (30.0, 30.0));
        assert_eq!(d.points[2][1], (60.0, 60.0));
        assert_eq!(d.points[3][1], (90.0, 90.0));
        // Not again within 4 levels
        d.add_points_at([32, 60, 90], true);
        assert_eq!(d.points[1].len(), 3);
        // The probe follows the current channel
        d.channel = 3;
        d.set_probe(Some([30, 60, 90]));
        assert_eq!(d.probe, Some(90.0));
        d.set_probe(None);
        assert_eq!(d.probe, None);
    }

    #[test]
    fn preset_files_round_trip() {
        let mut d = Dialog::new([[0; 256]; 3]);
        d.points[2] = vec![(0.0, 20.0), (128.0, 150.0), (255.0, 255.0)];
        let mut e = Dialog::new([[0; 256]; 3]);
        e.load_preset(&d.encoded());
        assert_eq!(e.points, d.points);
    }

    #[test]
    fn pencil_smooth_and_show_clipping() {
        let mut d = Dialog::new([[0; 256]; 3]);
        // The pencil takes the curve as a table; a step drawn on it
        d.set_pencil(true);
        let t = d.tables.as_mut().unwrap();
        t[0][128..].fill(255);
        t[0][..128].fill(0);
        assert_eq!(d.adjustment(), Adjustment::CurveTables(d.tables.unwrap()));
        // Smooth eases the step
        d.smooth();
        let t = d.tables.unwrap()[0];
        assert!(t[127] > 0 && t[128] < 255 && t[100] == 0 && t[200] == 255);
        // Back to points: placed along the drawn curve
        d.set_pencil(false);
        assert!(d.tables.is_none());
        assert_eq!(d.points[0].len(), 9);
        assert_eq!(d.points[0][0], (0.0, 0.0));
        assert_eq!(d.points[1], identity());
        // Show Clipping while the black pin is dragged: black where every
        // channel clips to 0, white elsewhere
        let mut d = Dialog::new([[0; 256]; 3]);
        d.points[0][0] = (50.0, 0.0);
        d.clipping = Some(false);
        let Adjustment::CurveTables(t) = d.adjustment() else {
            panic!("tables");
        };
        assert_eq!((t[1][40], t[1][60]), (0, 255));
    }

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
    fn the_hand_adds_and_moves_a_point() {
        let mut d = Dialog::new([[0; 256]; 3]);
        d.target_press([90, 100, 110]);
        assert_eq!(
            d.points[0],
            vec![(0.0, 0.0), (100.0, 100.0), (255.0, 255.0)]
        );
        d.target_drag(-30.0);
        assert_eq!(d.points[0][1], (100.0, 130.0));
        d.target_release();
        // On a channel, that channel's value; a point within 4 is reused
        d.channel = 1;
        d.target_press([52, 0, 0]);
        d.target_drag(20.0);
        d.target_press([50, 0, 0]);
        assert_eq!(d.points[1], vec![(0.0, 0.0), (52.0, 32.0), (255.0, 255.0)]);
    }

    #[test]
    fn auto_moves_each_channels_end_points() {
        use op_core::auto::{Options, Targets, compute};
        let mut d = Dialog::new([[0; 256]; 3]);
        let px: Vec<[u8; 3]> = (0..1000)
            .map(|i| [30 + (i % 191) as u8, 60 + (i % 100) as u8, 90])
            .collect();
        d.auto(compute(&px, &Options::auto_tone(Targets::default())));
        assert_eq!(d.points[1], vec![(30.0, 0.0), (220.0, 255.0)]);
        assert_eq!(d.points[0], identity());
        // The default (Enhance Brightness and Contrast): one curve on RGB
        d.auto(compute(&px, &Options::default()));
        assert_eq!(d.points[1], identity());
        assert!(d.points[0].len() >= 2);
    }
}
