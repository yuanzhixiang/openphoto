//! Image › Adjustments › HDR Toning..., as Photoshop 2026 lays it out
//! (441 pt wide, measured from 2x captures): Preset with its gear, the
//! Method group, and for Local Adaptation the Edge Glow, Tone and Detail,
//! Advanced and Toning Curve and Histogram sections, each folding under
//! its triangle; OK, Cancel and Preview on the right.

use egui::{Align2, Color32, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, vec2};

use super::filter_layout::Scale;
use super::{AdjustDialog, Outcome};
use crate::dialogs::{appkit, common, preset_files};
use crate::native_popup::Entry;
use crate::theme::{self, pt};

/// The settings' order (`Kind::params`).
const METHOD: usize = 0;
const RADIUS: usize = 1;
const STRENGTH: usize = 2;
const GAMMA: usize = 3;
const EXPOSURE: usize = 4;
const DETAIL: usize = 5;
const SMOOTH_EDGES: usize = 10;
/// Local Adaptation and Exposure and Gamma in `HdrMethod::ALL`'s order.
const LOCAL: usize = 3;
const EXPOSURE_GAMMA: usize = 0;

/// The four sections, their open heights (from the title line to the
/// box's bottom) and the step to the next title when open (Photoshop's
/// titles sit at 115.5, 215.5, 318.25 and 446).
const SECTIONS: [(&str, f32, f32); 4] = [
    ("Edge Glow", 87.25, 100.0),
    ("Tone and Detail", 90.25, 102.75),
    ("Advanced", 115.5, 127.75),
    ("Toning Curve and Histogram", 334.75, 347.5),
];

/// HDR Toning's state beyond its fields: the Toning Curve, the point
/// picked on it, and which sections are open.
#[derive(Clone, Debug)]
pub struct HdrExtra {
    pub curve: Vec<(u8, u8)>,
    pub corners: Vec<bool>,
    pub selected: Option<usize>,
    drag: Option<usize>,
    pub open: [bool; 4],
    input: String,
    output: String,
    shown: Option<usize>,
}

impl Default for HdrExtra {
    /// Photoshop opens with the first three sections open, the curve
    /// folded and straight.
    fn default() -> Self {
        Self {
            curve: vec![(0, 0), (255, 255)],
            corners: vec![false, false],
            selected: None,
            drag: None,
            open: [true, true, true, false],
            input: String::new(),
            output: String::new(),
            shown: None,
        }
    }
}

impl AdjustDialog {
    /// The sections' title lines (Local Adaptation) and the outer group's
    /// bottom.
    fn hdr_layout(&self) -> ([f32; 4], f32) {
        let method = self.value(METHOD).unwrap_or(LOCAL as f32) as usize;
        if method == EXPOSURE_GAMMA {
            return ([0.0; 4], 164.0);
        }
        if method != LOCAL {
            return ([0.0; 4], 110.0);
        }
        let mut titles = [0.0; 4];
        let mut y = 115.5;
        let mut bottom = 0.0;
        for (k, (_, height, step)) in SECTIONS.into_iter().enumerate() {
            titles[k] = y;
            if self.extra.hdr.open[k] {
                bottom = y + height + 10.0;
                y += step;
            } else {
                bottom = y + 22.0;
                y += 25.0;
            }
        }
        (titles, bottom)
    }

    pub(super) fn hdr_size(&self) -> egui::Vec2 {
        let (_, bottom) = self.hdr_layout();
        vec2(pt(441.0), pt(bottom + 11.0))
    }

    pub(super) fn hdr_toning_ui(&mut self, ui: &mut egui::Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "HDR Toning", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();

        self.hdr_presets(ui, frame);

        // Method, its pop-up breaking the outer group's top line
        let (titles, bottom) = self.hdr_layout();
        let method = self.value(METHOD).unwrap_or(LOCAL as f32) as usize;
        let t = appkit::text(
            ui,
            at(71.0, 89.0),
            Align2::RIGHT_CENTER,
            "Method:",
            appkit::TEXT,
        );
        let labels = op_core::tone::HdrMethod::ALL.map(|m| m.label());
        let entries: Vec<Entry> = labels
            .iter()
            .enumerate()
            .map(|(k, l)| Entry::item(*l, k == method))
            .collect();
        if let Some(k) = appkit::popup(
            ui,
            r(75.0, 80.0, 208.0, 98.5),
            "hdr-method",
            labels[method],
            &entries,
        ) {
            self.values[METHOD] = k.to_string();
        }
        appkit::group(
            &painter,
            r(10.75, 90.25, 333.75, bottom),
            (t.left() - pt(6.0), at(213.0, 0.0).x),
        );

        if method == EXPOSURE_GAMMA {
            self.hdr_row(ui, frame, GAMMA, "Gamma:", 121.0, None, true);
            self.hdr_row(ui, frame, EXPOSURE, "Exposure:", 146.0, None, false);
        }
        if method == LOCAL {
            for (k, (title, height, _)) in SECTIONS.into_iter().enumerate() {
                let y = titles[k];
                let open = self.extra.hdr.open[k];
                // The title with its triangle, which folds the section
                let tri = at(29.75, y);
                let head = Rect::from_min_max(
                    at(22.0, y - 8.0),
                    at(40.5 + 7.0 * title.len() as f32, y + 8.0),
                );
                if ui
                    .interact(head, ui.id().with(("hdr-section", k)), Sense::click())
                    .clicked()
                {
                    self.extra.hdr.open[k] = !open;
                }
                let s = Stroke::new(pt(1.25), Color32::from_gray(0xa1));
                let points = if open {
                    vec![
                        tri + vec2(pt(-3.5), pt(-1.75)),
                        tri + vec2(0.0, pt(1.75)),
                        tri + vec2(pt(3.5), pt(-1.75)),
                    ]
                } else {
                    vec![
                        tri + vec2(pt(-1.75), pt(-3.5)),
                        tri + vec2(pt(1.75), 0.0),
                        tri + vec2(pt(-1.75), pt(3.5)),
                    ]
                };
                painter.add(Shape::line(points, s));
                let label = appkit::text(ui, at(40.5, y), Align2::LEFT_CENTER, title, appkit::TEXT);
                let right = at(323.75, 0.0).x;
                let line = Stroke::new(pt(1.0), appkit::GROUP_LINE);
                if open {
                    appkit::group(
                        &painter,
                        r(20.75, y, 323.75, y + height),
                        (tri.x - pt(6.0), label.right() + pt(5.0)),
                    );
                } else {
                    // Folded: the title line alone
                    painter.line_segment(
                        [at(20.75, y), Pos2::new(tri.x - pt(6.0), at(0.0, y).y)],
                        line,
                    );
                    painter.line_segment(
                        [
                            Pos2::new(label.right() + pt(5.0), at(0.0, y).y),
                            Pos2::new(right, at(0.0, y).y),
                        ],
                        line,
                    );
                    continue;
                }
                match k {
                    0 => {
                        self.hdr_row(ui, frame, RADIUS, "Radius:", y + 23.5, Some("px"), true);
                        self.hdr_row(ui, frame, STRENGTH, "Strength:", y + 48.5, None, false);
                        let mut on = self.value(SMOOTH_EDGES) == Some(1.0);
                        appkit::checkbox_with(
                            ui,
                            at(123.5, y + 67.5),
                            (10.5, 8.0),
                            "Smooth Edges",
                            &mut on,
                            true,
                        );
                        self.values[SMOOTH_EDGES] = (on as u8).to_string();
                    }
                    1 => {
                        for (j, (i, label, unit)) in [
                            (GAMMA, "Gamma:", None),
                            (EXPOSURE, "Exposure:", None),
                            (DETAIL, "Detail:", Some("%")),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            self.hdr_row(
                                ui,
                                frame,
                                i,
                                label,
                                y + 23.0 + 25.0 * j as f32,
                                unit,
                                false,
                            );
                        }
                    }
                    2 => {
                        for (j, label) in ["Shadow:", "Highlight:", "Vibrance:", "Saturation:"]
                            .into_iter()
                            .enumerate()
                        {
                            self.hdr_row(
                                ui,
                                frame,
                                6 + j,
                                label,
                                y + 23.25 + 25.0 * j as f32,
                                Some("%"),
                                false,
                            );
                        }
                    }
                    _ => self.hdr_curve(ui, frame, y),
                }
            }
        }

        // OK, Cancel and Preview
        let valid = self.effect().is_some();
        let ok = appkit::button(ui, r(351.0, 38.5, 430.0, 64.0), "OK", true, valid).clicked();
        let cancel =
            appkit::button(ui, r(351.0, 73.5, 430.0, 99.0), "Cancel", false, true).clicked();
        appkit::checkbox_with(
            ui,
            at(350.0, 118.5),
            (12.0, 10.0),
            "Preview",
            &mut self.preview,
            true,
        );
        self.legacy_outcome(ui, (ok, cancel))
    }

    /// A row: the label right-aligned at 105 on line `cy`, the field
    /// 236–279.5 with an optional unit after it, the track 117–224.5 with
    /// its top 4.5 pt above the line.
    #[allow(clippy::too_many_arguments)]
    fn hdr_row(
        &mut self,
        ui: &mut egui::Ui,
        frame: Rect,
        i: usize,
        label: &str,
        cy: f32,
        unit: Option<&str>,
        focus: bool,
    ) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        appkit::text(ui, at(105.0, cy), Align2::RIGHT_CENTER, label, appkit::TEXT);
        let p = &self.kind.params()[i];
        appkit::field(
            ui,
            Rect::from_min_max(at(236.0, cy - 8.5), at(279.5, cy + 8.5)),
            &mut self.values[i],
            ("hdr-field", i),
            (p.min, p.max),
            10f32.powi(-(p.decimals as i32)),
            p.decimals,
            focus && self.first_frame,
        );
        if let Some(unit) = unit {
            appkit::text(ui, at(281.0, cy), Align2::LEFT_CENTER, unit, appkit::TEXT);
        }
        // Gamma and Detail center their neutral value, as Photoshop's do
        let scale = match i {
            GAMMA => Scale::Table(&[(0.1, 0.0), (1.0, 0.5), (2.0, 1.0)]),
            DETAIL => Scale::Table(&[(-100.0, 0.0), (0.0, 0.5), (300.0, 1.0)]),
            _ => Scale::Linear,
        };
        self.classic_track(ui, i, at(117.0, cy - 4.5), at(224.5, 0.0).x, scale);
    }

    /// Preset: Default, then Photoshop's presets (when installed) and the
    /// saved ones; the gear saves, loads and deletes `.hdt` files.
    fn hdr_presets(&mut self, ui: &mut egui::Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        appkit::text(
            ui,
            at(44.5, 49.5),
            Align2::RIGHT_CENTER,
            "Preset:",
            appkit::TEXT,
        );
        use preset_files as files;
        let encoded = files::encode_hdr(&self.hdr_preset());
        let shown = files::shown(files::HDR_TONING, Some(&encoded));
        let default =
            self.values == self.default_values() && self.extra.hdr.curve == [(0, 0), (255, 255)];
        let name = shown.unwrap_or_else(|| {
            if default {
                "Default".into()
            } else {
                "Custom".into()
            }
        });
        let photoshop = files::HDR_TONING.saved_in(std::path::Path::new(
            "/Applications/Adobe Photoshop 2026/Presets/HDR Toning",
        ));
        let mut entries = vec![
            Entry::item("Default", name == "Default"),
            Entry::item("Custom", name == "Custom").enabled(false),
        ];
        if !photoshop.is_empty() {
            entries.push(Entry::Separator);
            entries.extend(
                photoshop
                    .iter()
                    .map(|(n, _)| Entry::item(n.clone(), *n == name)),
            );
        }
        let before = entries.len();
        let saved = files::add_saved(&mut entries, files::HDR_TONING, &name);
        let rect = Rect::from_min_max(at(49.0, 40.5), at(306.0, 59.0));
        match appkit::popup(ui, rect, "hdr-preset", &name, &entries) {
            Some(0) => {
                self.values = self.default_values();
                self.extra.hdr = HdrExtra {
                    open: self.extra.hdr.open,
                    ..Default::default()
                };
            }
            Some(k) if k >= before => {
                if let Some(bytes) = files::picked_saved(k, before, &saved, files::HDR_TONING) {
                    self.load_hdr(&bytes);
                }
            }
            Some(k) if k >= 3 => {
                if let Some(entry) = photoshop.get(k - 3)
                    && let Some(bytes) = files::load_saved(files::HDR_TONING, entry)
                {
                    self.load_hdr(&bytes);
                }
            }
            _ => {}
        }
        crate::ps_icons::paint_scaled(
            ui.painter(),
            at(320.0, 50.0),
            crate::ps_icons::Icon::Gear,
            appkit::TEXT,
            theme::color::PANEL,
            0.77,
        );
        let gear = Rect::from_center_size(at(320.0, 50.0), vec2(pt(18.0), pt(18.0)));
        if let Some(bytes) = files::gear(ui, gear, "hdr-gear", files::HDR_TONING, Some(encoded)) {
            self.load_hdr(&bytes);
        }
    }

    /// The settings' defaults as the fields show them.
    fn default_values(&self) -> Vec<String> {
        self.kind
            .params()
            .iter()
            .map(|p| super::format(self.kind, p.default, p.decimals))
            .collect()
    }

    /// The settings as an HDR Toning preset.
    fn hdr_preset(&self) -> preset_files::HdrPreset {
        let v = |i: usize| self.value(i).unwrap_or(self.kind.params()[i].default);
        preset_files::HdrPreset {
            local: v(METHOD) as usize == LOCAL,
            radius: v(RADIUS),
            strength: v(STRENGTH),
            gamma: v(GAMMA),
            exposure: v(EXPOSURE),
            detail: v(DETAIL),
            shadow: v(6),
            highlight: v(7),
            vibrance: v(8),
            saturation: v(9),
            curve: self.extra.hdr.curve.clone(),
            corners: self.extra.hdr.corners.clone(),
        }
    }

    fn load_hdr(&mut self, bytes: &[u8]) {
        let Some(p) = preset_files::decode_hdr(bytes) else {
            return;
        };
        let set = |values: &mut Vec<String>, i: usize, x: f32, decimals: usize| {
            values[i] = format!("{x:.decimals$}");
        };
        if p.local {
            self.values[METHOD] = LOCAL.to_string();
        }
        set(&mut self.values, RADIUS, p.radius, 0);
        set(&mut self.values, STRENGTH, p.strength, 2);
        set(&mut self.values, GAMMA, p.gamma, 2);
        set(&mut self.values, EXPOSURE, p.exposure, 2);
        set(&mut self.values, DETAIL, p.detail, 0);
        set(&mut self.values, 6, p.shadow, 0);
        set(&mut self.values, 7, p.highlight, 0);
        set(&mut self.values, 8, p.vibrance, 0);
        set(&mut self.values, 9, p.saturation, 0);
        if p.curve.len() >= 2 {
            self.extra.hdr.corners = (0..p.curve.len())
                .map(|k| p.corners.get(k).copied().unwrap_or(false))
                .collect();
            self.extra.hdr.curve = p.curve;
            self.extra.hdr.selected = None;
        }
    }

    /// The Toning Curve and Histogram section under its title line `t`:
    /// the graph with the histogram and the curve, the ramps, and Input,
    /// Output, Corner and the reset button.
    fn hdr_curve(&mut self, ui: &mut egui::Ui, frame: Rect, t: f32) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let graph = Rect::from_min_max(at(49.0, t + 25.0), at(305.0, t + 280.5));
        let painter = ui.painter().clone();
        painter.rect_filled(graph, 0, Color32::from_gray(0x45));
        // The histogram, then the quarter grid
        let hist = &self.histogram;
        let max = hist.iter().copied().max().unwrap_or(0).max(1) as f32;
        for (i, &n) in hist.iter().enumerate() {
            if n == 0 {
                continue;
            }
            let x0 = graph.left() + graph.width() * i as f32 / 256.0;
            let h = n as f32 / max * graph.height();
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x0, graph.bottom() - h),
                    Pos2::new(x0 + graph.width() / 256.0, graph.bottom()),
                ),
                0,
                Color32::from_gray(0x33),
            );
        }
        let grid = Stroke::new(pt(1.0), Color32::from_gray(0x33));
        for k in 1..4 {
            let f = k as f32 / 4.0;
            let x = graph.left() + graph.width() * f;
            let y = graph.top() + graph.height() * f;
            painter.line_segment(
                [Pos2::new(x, graph.top()), Pos2::new(x, graph.bottom())],
                grid,
            );
            painter.line_segment(
                [Pos2::new(graph.left(), y), Pos2::new(graph.right(), y)],
                grid,
            );
        }
        // The ramps: white to black down the left, black to white along
        // the bottom
        let ramp = |rect: Rect, a: Color32, b: Color32, vertical: bool| {
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
        ramp(
            Rect::from_min_max(at(40.0, t + 25.0), at(49.0, t + 280.5)),
            Color32::WHITE,
            Color32::BLACK,
            true,
        );
        ramp(
            Rect::from_min_max(at(49.0, t + 280.5), at(305.0, t + 289.5)),
            Color32::BLACK,
            Color32::WHITE,
            false,
        );

        let to_screen = |(x, y): (u8, u8)| {
            Pos2::new(
                graph.left() + x as f32 / 255.0 * graph.width(),
                graph.bottom() - y as f32 / 255.0 * graph.height(),
            )
        };
        let to_curve = |p: Pos2| {
            let x = ((p.x - graph.left()) / graph.width() * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            let y = ((graph.bottom() - p.y) / graph.height() * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            (x, y)
        };
        // Points: press near one to pick and drag it, elsewhere to add one;
        // dragged out of the graph a middle point goes
        let response = ui.interact(
            graph.expand(pt(4.0)),
            ui.id().with("hdr-curve"),
            Sense::click_and_drag(),
        );
        let c = &mut self.extra.hdr;
        if (response.drag_started() || response.clicked())
            && let Some(p) = response.interact_pointer_pos()
        {
            let near = c
                .curve
                .iter()
                .position(|&q| to_screen(q).distance(p) <= pt(5.0));
            let k = match near {
                Some(k) => Some(k),
                None if c.curve.len() < 16 => {
                    let q = to_curve(p);
                    let k = c
                        .curve
                        .iter()
                        .position(|&(x, _)| x > q.0)
                        .unwrap_or(c.curve.len());
                    if c.curve.iter().any(|&(x, _)| x == q.0) {
                        None
                    } else {
                        c.curve.insert(k, q);
                        c.corners.insert(k, false);
                        Some(k)
                    }
                }
                None => None,
            };
            c.selected = k;
            c.drag = k.filter(|_| response.drag_started());
        }
        if let (Some(k), Some(p)) = (c.drag, response.interact_pointer_pos()) {
            let n = c.curve.len();
            if !graph.expand(pt(20.0)).contains(p) && k != 0 && k + 1 != n {
                c.curve.remove(k);
                c.corners.remove(k);
                c.drag = None;
                c.selected = None;
            } else {
                let (mut x, y) = to_curve(p);
                let lo = if k > 0 {
                    c.curve[k - 1].0.saturating_add(1)
                } else {
                    0
                };
                let hi = if k + 1 < n {
                    c.curve[k + 1].0.saturating_sub(1)
                } else {
                    255
                };
                x = x.clamp(lo, hi.max(lo));
                c.curve[k] = (x, y);
            }
        }
        if response.drag_stopped() {
            c.drag = None;
        }
        // The curve and its points
        let mut tone = op_core::tone::HdrToning::default();
        for (k, &p) in c.curve.iter().take(16).enumerate() {
            tone.curve[k] = p;
        }
        tone.curve_len = c.curve.len().min(16) as u8;
        tone.corners = c
            .corners
            .iter()
            .enumerate()
            .fold(0, |m, (k, &v)| if v { m | 1 << k } else { m });
        let table = tone.curve_table();
        let line: Vec<Pos2> = (0..256).map(|x| to_screen((x as u8, table[x]))).collect();
        painter.add(Shape::line(line, Stroke::new(pt(1.5), Color32::WHITE)));
        for (k, &q) in c.curve.iter().enumerate() {
            let rect = Rect::from_center_size(to_screen(q), vec2(pt(5.0), pt(5.0)));
            if c.selected == Some(k) {
                painter.rect_filled(rect, 0, Color32::WHITE);
            } else {
                painter.rect(
                    rect,
                    0,
                    Color32::from_gray(0x45),
                    Stroke::new(pt(1.0), Color32::WHITE),
                    StrokeKind::Inside,
                );
            }
        }

        // Input, Output (percent), Corner and the reset button
        let row = t + 306.5;
        let on = c.selected.is_some();
        let ink = if on { appkit::TEXT } else { appkit::TEXT_OFF };
        if c.shown != c.selected || c.drag.is_some() {
            c.shown = c.selected;
            let (i, o) = c.selected.and_then(|k| c.curve.get(k).copied()).map_or(
                (String::new(), String::new()),
                |(x, y)| {
                    (
                        format!("{}", (x as f32 / 2.55).round()),
                        format!("{}", (y as f32 / 2.55).round()),
                    )
                },
            );
            c.input = i;
            c.output = o;
        }
        appkit::text(
            ui,
            at(40.0, row),
            Align2::LEFT_CENTER,
            "Input:",
            appkit::TEXT,
        );
        appkit::text(
            ui,
            at(138.0, row),
            Align2::LEFT_CENTER,
            "Output:",
            appkit::TEXT,
        );
        appkit::text(ui, at(120.0, row), Align2::LEFT_CENTER, "%", ink);
        appkit::text(ui, at(218.5, row), Align2::LEFT_CENTER, "%", ink);
        for (j, x) in [85.5, 183.5].into_iter().enumerate() {
            let rect = Rect::from_min_max(at(x, row - 8.5), at(x + 32.5, row + 8.5));
            let selected = self.extra.hdr.selected;
            let text = if j == 0 {
                &mut self.extra.hdr.input
            } else {
                &mut self.extra.hdr.output
            };
            if let Some(k) = selected {
                let response = appkit::field(
                    ui,
                    rect,
                    text,
                    ("hdr-curve-field", j),
                    (0.0, 100.0),
                    1.0,
                    0,
                    false,
                );
                if response.changed()
                    && let Ok(v) = text.trim().parse::<f32>()
                {
                    let v = (v.clamp(0.0, 100.0) * 2.55).round() as u8;
                    let c = &mut self.extra.hdr;
                    if j == 0 {
                        let lo = if k > 0 {
                            c.curve[k - 1].0.saturating_add(1)
                        } else {
                            0
                        };
                        let hi = if k + 1 < c.curve.len() {
                            c.curve[k + 1].0.saturating_sub(1)
                        } else {
                            255
                        };
                        c.curve[k].0 = v.clamp(lo, hi.max(lo));
                    } else {
                        c.curve[k].1 = v;
                    }
                }
            } else {
                painter.rect(
                    rect,
                    0,
                    Color32::from_gray(0x4e),
                    Stroke::new(pt(1.0), Color32::from_gray(0x5d)),
                    StrokeKind::Inside,
                );
            }
        }
        let c = &mut self.extra.hdr;
        let mut corner = c
            .selected
            .and_then(|k| c.corners.get(k).copied())
            .unwrap_or(false);
        appkit::checkbox_with(
            ui,
            at(232.0, row - 4.5),
            (9.0, 9.0),
            "Corner",
            &mut corner,
            on,
        );
        if let Some(k) = c.selected
            && let Some(flag) = c.corners.get_mut(k)
        {
            *flag = corner;
        }
        // Reset: the straight curve
        let reset = Rect::from_center_size(at(296.0, row), vec2(pt(16.0), pt(16.0)));
        if ui
            .interact(reset, ui.id().with("hdr-curve-reset"), Sense::click())
            .clicked()
        {
            c.curve = vec![(0, 0), (255, 255)];
            c.corners = vec![false, false];
            c.selected = None;
        }
        painter.text(
            reset.center(),
            Align2::CENTER_CENTER,
            "↺",
            appkit::font(),
            appkit::TEXT,
        );
    }
}
