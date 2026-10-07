//! Image > Adjustments > Hue/Saturation...: Photoshop 2026's UXP dialog,
//! 437 × 413 pt. A row of swatches picks Master or one of six color ranges
//! (each keeps its own values and hue range, edited on the bar under the
//! Before – After strip); Colorize swaps the swatches for one showing the
//! colorize color. Sizes are Photoshop points from the dialog's top-left
//! corner.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use op_core::adjust::{Adjustment, HUE_RANGES, HueRange, HueSaturation};

use super::{common, uxp};
use crate::ps_icons::{self, Icon};
use crate::theme::{color, pt};

pub const SIZE: egui::Vec2 = vec2(pt(437.0), pt(413.0));
const DIM: Color32 = Color32::from_gray(0x8e);
const HUE: (f32, f32) = (-180.0, 180.0);
const AMOUNT: (f32, f32) = (-100.0, 100.0);
const COLORIZE_HUE: (f32, f32) = (0.0, 360.0);
const COLORIZE_SATURATION: (f32, f32) = (0.0, 100.0);
/// The Before – After strip and the range bar: hue 180° at the left end,
/// 0° in the middle.
const STRIP_X: (f32, f32) = (20.0, 296.0);

/// The swatches' fills, as Photoshop draws them.
const SWATCH_FILLS: [[u8; 3]; 6] = [
    [234, 51, 35],
    [254, 252, 84],
    [117, 251, 76],
    [116, 251, 252],
    [0, 0, 244],
    [234, 51, 247],
];

// Track colors at 17 even stops, sampled from Photoshop 2026
const HUE_TRACK: [[u8; 3]; 17] = [
    [218, 55, 41],
    [236, 106, 44],
    [245, 196, 67],
    [229, 254, 82],
    [159, 251, 79],
    [121, 251, 76],
    [117, 251, 98],
    [117, 251, 168],
    [101, 218, 222],
    [71, 158, 248],
    [25, 64, 245],
    [28, 1, 245],
    [115, 20, 245],
    [205, 43, 246],
    [233, 51, 188],
    [233, 51, 97],
    [217, 54, 41],
];
const SATURATION_TRACK: [[u8; 3]; 17] = [
    [123, 123, 122],
    [134, 133, 128],
    [141, 140, 129],
    [143, 146, 130],
    [139, 152, 130],
    [136, 157, 130],
    [135, 163, 142],
    [137, 168, 159],
    [126, 156, 174],
    [104, 129, 189],
    [70, 80, 204],
    [68, 45, 217],
    [131, 42, 223],
    [198, 48, 229],
    [223, 51, 177],
    [230, 51, 91],
    [218, 54, 41],
];
/// The Before strip at 25 even stops, cyan (180°) to cyan.
const STRIP: [[u8; 3]; 25] = [
    [117, 251, 252],
    [88, 193, 250],
    [55, 127, 246],
    [26, 66, 246],
    [0, 0, 244],
    [53, 5, 245],
    [116, 20, 245],
    [170, 35, 246],
    [234, 51, 247],
    [234, 51, 191],
    [235, 51, 128],
    [235, 51, 74],
    [233, 51, 35],
    [235, 77, 39],
    [237, 127, 50],
    [244, 190, 65],
    [253, 247, 83],
    [209, 253, 81],
    [167, 252, 78],
    [130, 252, 76],
    [118, 251, 76],
    [117, 251, 93],
    [117, 251, 135],
    [117, 251, 191],
    [116, 251, 245],
];
const LIGHTNESS_TRACK: [[u8; 3]; 3] = [[10, 9, 9], [128, 128, 128], [239, 239, 239]];

#[derive(Clone)]
pub struct Dialog {
    /// Hue, saturation and lightness text for Master and the six ranges.
    pub values: [[String; 3]; 7],
    /// Each range's bounds (degrees).
    pub bounds: [[i32; 4]; 6],
    /// 0 for Master, 1–6 for Reds … Magentas.
    pub selected: usize,
    pub colorize: bool,
    /// Colorize's hue (0–360), saturation (0–100) and lightness.
    pub colorize_values: [String; 3],
    /// The range bar handle being dragged (0–3), or 4 for the whole range.
    drag: Option<(usize, f32, [i32; 4])>,
    /// The eyedropper chosen (0 pick a range, 1 add to it, 2 subtract):
    /// a click on the image samples (`sample`).
    pub eyedropper: Option<usize>,
    /// The targeted adjustment hand: dragging on the image changes the
    /// saturation (hue with Command) of the range under the pointer.
    pub targeting: bool,
    /// A hand drag: the range and its value when it started.
    target: Option<(usize, i32)>,
}

impl Dialog {
    /// A fresh dialog; Colorize starts from `hue` (the foreground color's).
    pub fn new(hue: i32) -> Self {
        let zero = || std::array::from_fn(|_| "0".to_string());
        Self {
            values: std::array::from_fn(|_| zero()),
            bounds: HUE_RANGES,
            selected: 0,
            colorize: false,
            colorize_values: [hue.to_string(), "25".into(), "0".into()],
            drag: None,
            eyedropper: None,
            targeting: false,
            target: None,
        }
    }
}

fn parse(text: &str, (min, max): (f32, f32)) -> Option<i32> {
    let v: f32 = text.trim().parse().ok()?;
    (min..=max).contains(&v).then_some(v.round() as i32)
}

/// The x of `hue` on the strip (degrees, 180° at the left end).
fn strip_x(frame: Rect, hue: f32) -> f32 {
    let t = (hue - 180.0).rem_euclid(360.0) / 360.0;
    frame.left() + pt(STRIP_X.0 + (STRIP_X.1 - STRIP_X.0) * t)
}

fn hue_color(hue: f32) -> [u8; 3] {
    let h = hue.rem_euclid(360.0) / 60.0;
    let f = h.fract();
    let (up, down) = ((f * 255.0) as u8, ((1.0 - f) * 255.0) as u8);
    match h as usize {
        0 => [255, up, 0],
        1 => [down, 255, 0],
        2 => [0, 255, up],
        3 => [0, down, 255],
        4 => [up, 0, 255],
        _ => [255, 0, down],
    }
}

impl Dialog {
    fn ranges(&self) -> Option<[HueRange; 6]> {
        let mut out = [HueRange {
            bounds: [0; 4],
            hue: 0,
            saturation: 0,
            lightness: 0,
        }; 6];
        for (i, r) in out.iter_mut().enumerate() {
            let v = &self.values[i + 1];
            *r = HueRange {
                bounds: self.bounds[i],
                hue: parse(&v[0], HUE)?,
                saturation: parse(&v[1], AMOUNT)?,
                lightness: parse(&v[2], AMOUNT)?,
            };
        }
        Some(out)
    }

    pub fn settings(&self) -> Option<HueSaturation> {
        if self.colorize {
            let v = &self.colorize_values;
            return Some(HueSaturation {
                colorize: true,
                ..HueSaturation::master(
                    parse(&v[0], COLORIZE_HUE)?,
                    parse(&v[1], COLORIZE_SATURATION)?,
                    parse(&v[2], AMOUNT)?,
                )
            });
        }
        let v = &self.values[0];
        Some(HueSaturation {
            ranges: self.ranges()?,
            ..HueSaturation::master(
                parse(&v[0], HUE)?,
                parse(&v[1], AMOUNT)?,
                parse(&v[2], AMOUNT)?,
            )
        })
    }

    pub fn adjustment(&self) -> Option<Adjustment> {
        self.settings().map(Adjustment::HueSaturation)
    }

    /// "Default" until anything changes, then "Custom".
    /// The settings as a Hue/Saturation preset file (None while a field
    /// is invalid).
    fn encoded(&self) -> Option<Vec<u8>> {
        let three = |t: &[String; 3], ranges: [(f32, f32); 3]| -> Option<[i32; 3]> {
            Some([
                parse(&t[0], ranges[0])?,
                parse(&t[1], ranges[1])?,
                parse(&t[2], ranges[2])?,
            ])
        };
        let mut values = [[0; 3]; 7];
        for (v, t) in values.iter_mut().zip(&self.values) {
            *v = three(t, [HUE, AMOUNT, AMOUNT])?;
        }
        let colorize = [COLORIZE_HUE, COLORIZE_SATURATION, AMOUNT];
        Some(super::preset_files::encode_hue_saturation(
            &super::preset_files::HueSaturation {
                colorize: self.colorize,
                colorize_values: three(&self.colorize_values, colorize)?,
                values,
                bounds: self.bounds,
            },
        ))
    }

    /// A preset file's settings into the dialog.
    fn load_preset(&mut self, bytes: &[u8]) {
        if let Some(h) = super::preset_files::decode_hue_saturation(bytes) {
            self.colorize = h.colorize;
            self.colorize_values = h.colorize_values.map(|v| v.to_string());
            self.values = h.values.map(|t| t.map(|v| v.to_string()));
            self.bounds = h.bounds;
        }
    }

    fn preset(&self) -> &'static str {
        let untouched = !self.colorize
            && self.bounds == HUE_RANGES
            && self
                .values
                .iter()
                .flatten()
                .all(|v| parse(v, HUE) == Some(0));
        if untouched {
            return "Default";
        }
        let ranges_flat = self.bounds == HUE_RANGES
            && self.values[1..]
                .iter()
                .flatten()
                .all(|v| parse(v, HUE) == Some(0));
        let same = |text: &[String; 3], want: [i32; 3]| {
            text.iter().zip(want).all(|(t, w)| parse(t, HUE) == Some(w))
        };
        super::adjust_presets::HUE_SATURATION
            .iter()
            .find(|(_, master, colorize)| {
                ranges_flat
                    && match colorize {
                        Some(c) => self.colorize && same(&self.colorize_values, *c),
                        None => !self.colorize && same(&self.values[0], *master),
                    }
            })
            .map_or("Custom", |(name, ..)| name)
    }

    /// Picks Photoshop's preset `k`: its Master values, or Colorize with
    /// its values.
    fn apply_preset(&mut self, k: usize) {
        let (_, master, colorize) = super::adjust_presets::HUE_SATURATION[k];
        self.reset();
        match colorize {
            Some(c) => {
                self.colorize = true;
                self.colorize_values = c.map(|v| v.to_string());
            }
            None => self.values[0] = master.map(|v| v.to_string()),
        }
    }

    /// The range (1–6) whose full-strength part holds `hue`, else the one
    /// nearest it.
    fn range_of(&self, hue: f32) -> usize {
        let dist = |b: &[i32; 4]| {
            let mid =
                (b[1] as f32 + ((b[2] - b[1]).rem_euclid(360)) as f32 / 2.0).rem_euclid(360.0);
            let d = (hue - mid).rem_euclid(360.0);
            d.min(360.0 - d)
        };
        (0..6)
            .min_by(|&a, &b| dist(&self.bounds[a]).total_cmp(&dist(&self.bounds[b])))
            .map_or(1, |k| k + 1)
    }

    /// The chosen eyedropper's click on a pixel of color `rgb`: picking
    /// centers the range (the nearest one, with Master selected) on its
    /// hue with Photoshop's widths (30° full, 30° falloffs); adding widens
    /// the full-strength part to take it in; subtracting narrows it to
    /// leave it out.
    pub fn sample(&mut self, rgb: [u8; 3]) {
        let Some(k) = self.eyedropper else {
            return;
        };
        if self.colorize {
            return;
        }
        let h = op_core::adjust::hue_of(rgb);
        if self.selected == 0 || k == 0 && self.selected == 0 {
            self.selected = self.range_of(h);
        }
        let i = self.selected - 1;
        let b = &mut self.bounds[i];
        let wrap = |v: f32| (v.round() as i32).rem_euclid(360);
        // Degrees from the range's start, so comparisons don't wrap
        let off = |v: i32| (v - b[0]).rem_euclid(360) as f32;
        match k {
            0 => {
                *b = [
                    wrap(h - 45.0),
                    wrap(h - 15.0),
                    wrap(h + 15.0),
                    wrap(h + 45.0),
                ]
            }
            1 => {
                let x = (h - b[0] as f32).rem_euclid(360.0);
                let (s0, s1) = (off(b[1]), off(b[2]));
                if x < s0 {
                    let d = s0 - x;
                    *b = [wrap(b[0] as f32 - d), wrap(h), b[2], b[3]];
                } else if x > s1 {
                    let d = x - s1;
                    *b = [b[0], b[1], wrap(h), wrap(b[3] as f32 + d)];
                }
            }
            _ => {
                let x = (h - b[0] as f32).rem_euclid(360.0);
                let (s0, s1) = (off(b[1]), off(b[2]));
                if (s0..=s1).contains(&x) {
                    // Off the nearer end of the full-strength part
                    if x - s0 < s1 - x {
                        let to = (x + 1.0).min(s1);
                        b[1] = wrap(b[0] as f32 + to);
                    } else {
                        let to = (x - 1.0).max(s0);
                        b[2] = wrap(b[0] as f32 + to);
                    }
                }
            }
        }
    }

    /// The targeted adjustment hand pressed on a pixel of color `rgb`:
    /// the range of its hue is selected.
    pub fn target_press(&mut self, rgb: [u8; 3], hue_mode: bool) {
        if self.colorize {
            return;
        }
        let i = self.range_of(op_core::adjust::hue_of(rgb));
        self.selected = i;
        let slot = if hue_mode { 0 } else { 1 };
        let start = parse(&self.values[i][slot], HUE).unwrap_or(0);
        self.target = Some((i, start));
    }

    /// The hand dragged `dx` points sideways: saturation (hue with
    /// Command) of that range goes up to the right, a point per point.
    pub fn target_drag(&mut self, dx: f32, hue_mode: bool) {
        if let Some((i, start)) = self.target {
            let (slot, range) = if hue_mode { (0, HUE) } else { (1, AMOUNT) };
            let v = (start as f32 + dx).round().clamp(range.0, range.1) as i32;
            self.values[i][slot] = v.to_string();
        }
    }

    pub fn target_release(&mut self) {
        self.target = None;
    }

    /// Invert: the selected range covers every other hue instead.
    pub fn invert(&mut self) {
        if self.selected > 0 {
            let b = self.bounds[self.selected - 1];
            self.bounds[self.selected - 1] = [b[2], b[3], b[0], b[1]];
        }
    }

    fn reset(&mut self) {
        let colorize = self.colorize_values.clone();
        *self = Self::new(0);
        self.colorize_values = colorize;
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> Option<uxp::Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();

        // Preset
        uxp::label(ui, at(20.0, 61.0), "Preset");
        use super::preset_files as files;
        let encoded = self.encoded();
        let saved_name = files::shown(files::HUE_SATURATION, encoded.as_deref());
        let preset = saved_name.as_deref().unwrap_or(self.preset());
        let saved = files::HUE_SATURATION.saved();
        let mut reset = false;
        let mut picked = None;
        let mut picked_saved = None;
        common::ps_dropdown(
            ui,
            r(56.0, 48.5, 266.0, 73.5),
            "hs-preset",
            |p, rect| {
                p.text(
                    rect.left_center() + vec2(pt(9.0), pt(0.75)),
                    Align2::LEFT_CENTER,
                    preset,
                    uxp::font(),
                    uxp::TEXT,
                );
            },
            |ui| {
                if ui.button("Default").clicked() {
                    reset = true;
                }
                ui.add_enabled(false, egui::Button::new("Custom"));
                ui.separator();
                for (k, (name, ..)) in super::adjust_presets::HUE_SATURATION.iter().enumerate() {
                    if ui.selectable_label(preset == *name, *name).clicked() {
                        picked = Some(k);
                    }
                }
                // The saved presets
                if !saved.is_empty() {
                    ui.separator();
                }
                for (k, (name, _)) in saved.iter().enumerate() {
                    if ui.selectable_label(preset == name, name).clicked() {
                        picked_saved = Some(k);
                    }
                }
            },
        );
        if reset {
            self.reset();
        }
        if let Some(k) = picked {
            self.apply_preset(k);
        }
        if let Some(bytes) =
            picked_saved.and_then(|k| files::load_saved(files::HUE_SATURATION, &saved[k]))
        {
            self.load_preset(&bytes);
        }
        let gear = Rect::from_center_size(at(284.0, 61.0), vec2(pt(18.0), pt(18.0)));
        if let Some(bytes) = files::gear(ui, gear, "hs-gear", files::HUE_SATURATION, encoded) {
            self.load_preset(&bytes);
        }
        ps_icons::paint(
            &painter,
            at(284.0, 61.0),
            Icon::PresetMenu,
            uxp::TEXT,
            color::PANEL,
        );
        // The targeted adjustment hand: a toggle
        let hand = Rect::from_center_size(at(32.5, 95.0), vec2(pt(26.0), pt(24.0)));
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
        ps_icons::paint(&painter, hand.center(), Icon::TargetedHand, uxp::TEXT, fill);
        if !self.colorize
            && ui
                .interact(hand, ui.id().with("hs-hand"), Sense::click())
                .clicked()
        {
            self.targeting = !self.targeting;
            self.eyedropper = None;
        }

        // Master, the six ranges, or the colorize color
        if self.colorize {
            let [h, s] = [0, 1].map(|i| parse(&self.colorize_values[i], (0.0, 360.0)).unwrap_or(0));
            let rgb = op_core::adjust::hsl_color(h as f32, s as f32 / 100.0, 0.5);
            swatch(&painter, at(68.0, 96.0), Some(rgb), true);
        } else {
            for i in 0..7 {
                let center = at(68.0 + 36.0 * i as f32, 96.0);
                let hit = Rect::from_center_size(center, vec2(pt(24.0), pt(24.0)));
                if ui
                    .interact(hit, ui.id().with(("hs-range", i)), Sense::click())
                    .clicked()
                {
                    self.selected = i;
                }
                swatch(
                    &painter,
                    center,
                    i.checked_sub(1).map(|k| SWATCH_FILLS[k]),
                    self.selected == i,
                );
            }
        }

        // The three sliders
        let editing = if self.colorize { 7 } else { self.selected };
        let ranges = if self.colorize {
            [COLORIZE_HUE, COLORIZE_SATURATION, AMOUNT]
        } else {
            [HUE, AMOUNT, AMOUNT]
        };
        let colorize_hue = parse(&self.colorize_values[0], COLORIZE_HUE).unwrap_or(0) as f32;
        for (k, name) in ["Hue", "Saturation", "Lightness"].into_iter().enumerate() {
            let y = 120.0 + 55.0 * k as f32;
            uxp::label(ui, at(20.0, y + 12.5), name);
            let text = if editing == 7 {
                &mut self.colorize_values[k]
            } else {
                &mut self.values[editing][k]
            };
            uxp::number_box(
                ui,
                r(251.0, y, 295.5, y + 24.0),
                text,
                ("hs-field", editing, k),
                ranges[k],
                false,
            );
            let value = parse(text, ranges[k]).unwrap_or(0) as f32;
            let saturated = hue_color(colorize_hue);
            let colorize_saturation = move |t: f32| {
                let [r, g, b] = saturated;
                Color32::from_gray(0x76).lerp_to_gamma(Color32::from_rgb(r, g, b), t)
            };
            let colors: Box<dyn Fn(f32) -> Color32> = match (k, self.colorize) {
                (0, _) => Box::new(uxp::stops(&HUE_TRACK)),
                (1, false) => Box::new(uxp::stops(&SATURATION_TRACK)),
                (1, true) => Box::new(colorize_saturation),
                _ => Box::new(uxp::stops(&LIGHTNESS_TRACK)),
            };
            if let Some(v) = uxp::slider(
                ui,
                ("hs", k),
                at(20.0, 0.0).x,
                at(296.0, 0.0).x,
                at(0.0, y + 36.0).y,
                value,
                ranges[k],
                &colors,
            ) {
                *text = format!("{v}");
            }
        }

        // A range's hue, next to its Hue box
        if !self.colorize && self.selected > 0 {
            let [_, b, c, _] = self.bounds[self.selected - 1];
            let center = (b as f32 + (c - b).rem_euclid(360) as f32 / 2.0).rem_euclid(360.0);
            painter.text(
                at(243.5, 132.75),
                Align2::RIGHT_CENTER,
                format!("{}°", center.round()),
                uxp::font(),
                uxp::TEXT,
            );
        }

        // Colorize, the eyedroppers and Invert (not available here)
        let was = self.colorize;
        uxp::checkbox(ui, at(20.0, 292.0), "Colorize", &mut self.colorize);
        if self.colorize != was {
            self.selected = 0;
        }
        // The eyedroppers work with a range (or Master, which picks one);
        // add and subtract need a range, as does Invert
        let ranged = !self.colorize && self.selected > 0;
        for (k, (x, icon)) in [
            (123.5, Icon::Eyedropper),
            (160.0, Icon::EyedropperPlus),
            (196.0, Icon::EyedropperMinus),
        ]
        .into_iter()
        .enumerate()
        {
            let enabled = !self.colorize && (k == 0 || ranged);
            let rect = Rect::from_center_size(at(x, 298.0), vec2(pt(26.0), pt(24.0)));
            let chosen = enabled && self.eyedropper == Some(k);
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
            let ink = if enabled {
                uxp::TEXT
            } else {
                Color32::from_gray(0x8e)
            };
            ps_icons::paint(&painter, rect.center(), icon, ink, fill);
            if enabled
                && ui
                    .interact(rect, ui.id().with(("hs-eyedropper", k)), Sense::click())
                    .clicked()
            {
                self.eyedropper = (!chosen).then_some(k);
                self.targeting = false;
            }
        }
        if common::ps_button_with(
            ui,
            r(233.0, 286.0, 296.0, 310.0),
            "Invert",
            false,
            ranged,
            crate::theme::uxp_bold(pt(12.0)),
        )
        .clicked()
        {
            self.invert();
        }

        self.before_after(ui, frame);
        uxp::preview(ui, at(308.0, 127.0), preview);
        // OK holds the keyboard focus while no box does
        let _ = first_frame;
        let ok_focused = ui.memory(|m| m.focused().is_none());
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), ok_focused)
    }

    /// The Before – After strip (the hues before on top, after below), the
    /// range bar and the range's bounds under it.
    fn before_after(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let painter = ui.painter().clone();
        uxp::label(ui, at(20.0, 331.0), "Before - After");
        let settings = self.settings();
        let x0 = at(STRIP_X.0, 0.0).x;
        let x1 = at(STRIP_X.1, 0.0).x;
        let steps = 138;
        for i in 0..steps {
            let (a, b) = (
                x0 + (x1 - x0) * i as f32 / steps as f32,
                x0 + (x1 - x0) * (i + 1) as f32 / steps as f32,
            );
            let before = uxp::stops(&STRIP)((i as f32 + 0.5) / steps as f32);
            let [r, g, bl, _] = before.to_array();
            let after = settings.map_or(before, |s| {
                let [r, g, b, _] = s.apply([r, g, bl, 255]);
                Color32::from_rgb(r, g, b)
            });
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(a, at(0.0, 343.0).y),
                    Pos2::new(b + 0.5, at(0.0, 349.5).y),
                ),
                0,
                before,
            );
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(a, at(0.0, 349.5).y),
                    Pos2::new(b + 0.5, at(0.0, 356.0).y),
                ),
                0,
                after,
            );
        }
        painter.rect_filled(
            Rect::from_min_max(at(STRIP_X.0, 364.0), at(STRIP_X.1, 366.0)),
            0,
            Color32::from_gray(0xa0),
        );

        if self.colorize || self.selected == 0 {
            painter.text(
                at(20.0, 386.0),
                Align2::LEFT_CENTER,
                "0° / 0°",
                uxp::font(),
                DIM,
            );
            painter.text(
                at(296.0, 386.0),
                Align2::RIGHT_CENTER,
                "0° / 0°",
                uxp::font(),
                DIM,
            );
            return;
        }
        let range = self.selected - 1;
        let bounds = self.bounds[range];
        let xs = bounds.map(|b| strip_x(frame, b as f32));
        let y = |v: f32| at(0.0, v).y;
        // The fades darker, the full range lighter, then the handles
        let span = |a: f32, b: f32, top: f32, bottom: f32, fill: Color32| {
            let r = Rect::from_min_max(Pos2::new(a, y(top)), Pos2::new(b, y(bottom)));
            if b >= a {
                painter.rect_filled(r, 0, fill);
            } else {
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(a, y(top)), Pos2::new(x1, y(bottom))),
                    0,
                    fill,
                );
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(x0, y(top)), Pos2::new(b, y(bottom))),
                    0,
                    fill,
                );
            }
        };
        span(xs[0], xs[1], 361.0, 369.0, Color32::from_gray(0x7a));
        span(xs[2], xs[3], 361.0, 369.0, Color32::from_gray(0x7a));
        span(xs[1], xs[2], 358.0, 372.0, Color32::from_gray(0xa0));
        for (k, &x) in xs.iter().enumerate() {
            let w = if k == 1 || k == 2 { pt(1.5) } else { pt(1.0) };
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(x - w, y(358.0)), Pos2::new(x + w, y(372.0))),
                0,
                Color32::from_gray(0xf0),
            );
        }
        let label = |b: [i32; 2]| format!("{}°/{}°", b[0].rem_euclid(360), b[1].rem_euclid(360));
        let left = elide(ui, &label([bounds[0], bounds[1]]), pt(34.0));
        let right = elide(ui, &label([bounds[2], bounds[3]]), pt(28.0));
        painter.text(
            at(20.0, 386.0),
            Align2::LEFT_CENTER,
            left,
            uxp::font(),
            uxp::TEXT,
        );
        painter.text(
            at(296.0, 386.0),
            Align2::RIGHT_CENTER,
            right,
            uxp::font(),
            uxp::TEXT,
        );

        // Dragging a handle moves that bound; inside the range moves it all
        let bar = Rect::from_min_max(Pos2::new(x0, y(356.0)), Pos2::new(x1, y(374.0)));
        let response = ui.interact(bar, ui.id().with("hs-range-bar"), Sense::drag());
        let to_hue = |x: f32| 180.0 + 360.0 * (x - x0) / (x1 - x0);
        if response.drag_started()
            && let Some(p) = response.interact_pointer_pos()
        {
            let nearest = (0..4)
                .min_by(|&a, &b| (xs[a] - p.x).abs().total_cmp(&(xs[b] - p.x).abs()))
                .unwrap_or(0);
            let handle = if (xs[nearest] - p.x).abs() <= pt(4.0) {
                nearest
            } else {
                4
            };
            self.drag = Some((handle, to_hue(p.x), bounds));
        }
        if let (Some((handle, from, start)), Some(p)) = (self.drag, response.interact_pointer_pos())
        {
            let delta = (to_hue(p.x) - from).round() as i32;
            let mut b = start;
            if handle == 4 {
                b = b.map(|v| (v + delta).rem_euclid(360));
            } else {
                // A bound stays between its neighbours
                let around = |v: i32| (v - start[0]).rem_euclid(360);
                let lo = if handle == 0 {
                    around(start[1]) - 360
                } else {
                    around(start[handle - 1])
                };
                let hi = if handle == 3 {
                    360 + around(start[0]) - 1
                } else {
                    around(start[handle + 1])
                };
                let v = (around(start[handle]) + delta).clamp(lo, hi);
                b[handle] = (start[0] + v).rem_euclid(360);
            }
            self.bounds[range] = b;
        }
        if response.drag_stopped() {
            self.drag = None;
        }
    }
}

/// `text`, or as much as fits in `width` followed by "...".
fn elide(ui: &Ui, text: &str, width: f32) -> String {
    let fits = |t: &str| {
        ui.painter()
            .layout_no_wrap(t.to_string(), uxp::font(), uxp::TEXT)
            .size()
            .x
            <= width
    };
    if fits(text) {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    (0..chars.len())
        .rev()
        .map(|n| format!("{}...", chars[..n].iter().collect::<String>()))
        .find(|t| fits(t))
        .unwrap_or_else(|| "...".into())
}

/// A 24 pt swatch: a fill (or Master's color wheel when `None`), the
/// chosen one inside a 2 pt white ring and a 2 pt dark gap, the others
/// with a fine gray edge.
fn swatch(painter: &egui::Painter, center: Pos2, fill: Option<[u8; 3]>, chosen: bool) {
    let radius = if chosen {
        painter.circle_filled(center, pt(12.0), Color32::WHITE);
        painter.circle_filled(center, pt(10.0), Color32::from_gray(0x45));
        pt(8.0)
    } else {
        pt(11.75)
    };
    match fill {
        Some([r, g, b]) => {
            painter.circle_filled(center, radius, Color32::from_rgb(r, g, b));
        }
        None => {
            // Eight slices from red at the top, clockwise
            const WHEEL: [[u8; 3]; 8] = [
                [234, 51, 35],
                [240, 140, 50],
                [253, 230, 80],
                [117, 220, 76],
                [100, 200, 230],
                [40, 90, 240],
                [140, 60, 240],
                [234, 51, 200],
            ];
            for (i, [r, g, b]) in WHEEL.into_iter().enumerate() {
                let a0 = -std::f32::consts::FRAC_PI_2 - std::f32::consts::PI / 8.0
                    + i as f32 * std::f32::consts::FRAC_PI_4;
                let mut points = vec![center];
                for s in 0..=8 {
                    let a = a0 + std::f32::consts::FRAC_PI_4 * s as f32 / 8.0;
                    points.push(center + vec2(a.cos(), a.sin()) * radius);
                }
                painter.add(egui::Shape::convex_polygon(
                    points,
                    Color32::from_rgb(r, g, b),
                    Stroke::NONE,
                ));
            }
        }
    }
    if !chosen {
        painter.circle_stroke(
            center,
            pt(11.75),
            Stroke::new(pt(0.5), Color32::from_gray(0xa7)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_files_round_trip() {
        let mut d = Dialog::new(0);
        d.values[2] = ["10", "-20", "5"].map(String::from);
        d.colorize = true;
        d.colorize_values = ["215", "25", "0"].map(String::from);
        let mut e = Dialog::new(0);
        e.load_preset(&d.encoded().unwrap());
        assert_eq!(
            (e.values.clone(), e.colorize, e.colorize_values.clone()),
            (d.values.clone(), true, d.colorize_values.clone())
        );
    }

    #[test]
    fn eyedroppers_invert_and_the_hand() {
        let mut d = Dialog::new(0);
        // Picking a blue with Master selected chooses Blues, centered there
        d.eyedropper = Some(0);
        d.sample([0, 0, 255]);
        assert_eq!(d.selected, 5);
        assert_eq!(d.bounds[4], [195, 225, 255, 285]);
        // Adding a purple widens the full-strength part up to it
        d.eyedropper = Some(1);
        d.sample([128, 0, 255]);
        let b = d.bounds[4];
        assert_eq!((b[1], b[2]), (225, 270));
        assert_eq!(b[3], 300);
        // Subtracting the blue again cuts it out of the near end
        d.eyedropper = Some(2);
        d.sample([0, 0, 255]);
        assert!(d.bounds[4][1] > 240, "{:?}", d.bounds[4]);
        // Invert swaps the range for the rest of the hues
        let before = d.bounds[4];
        d.invert();
        assert_eq!(d.bounds[4], [before[2], before[3], before[0], before[1]]);
        // The hand on a red drags Reds' saturation up
        let mut d = Dialog::new(0);
        d.targeting = true;
        d.target_press([220, 40, 40], false);
        assert_eq!(d.selected, 1);
        d.target_drag(30.0, false);
        assert_eq!(d.values[1][1], "30");
        d.target_drag(-500.0, false);
        assert_eq!(d.values[1][1], "-100");
        d.target_release();
    }

    #[test]
    fn photoshops_presets() {
        let mut d = Dialog::new(200);
        let k = |name: &str| {
            super::super::adjust_presets::HUE_SATURATION
                .iter()
                .position(|(n, ..)| *n == name)
                .unwrap()
        };
        d.apply_preset(k("Sepia"));
        assert!(d.colorize);
        assert_eq!(d.colorize_values, ["35", "25", "0"].map(String::from));
        assert_eq!(d.preset(), "Sepia");
        d.apply_preset(k("Old Style"));
        assert!(!d.colorize);
        assert_eq!(d.values[0], ["0", "-40", "5"].map(String::from));
        assert_eq!(d.preset(), "Old Style");
    }

    #[test]
    fn values_ranges_and_preset() {
        let mut d = Dialog::new(200);
        assert_eq!(d.preset(), "Default");
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::HueSaturation(HueSaturation::master(0, 0, 0)))
        );
        // Each range keeps its own values
        d.values[2][1] = "-40".into();
        assert_eq!(d.preset(), "Custom");
        let s = d.settings().unwrap();
        assert_eq!(s.ranges[1].saturation, -40);
        assert_eq!(s.ranges[1].bounds, HUE_RANGES[1]);
        // Out of range text disables OK
        d.values[0][0] = "181".into();
        assert_eq!(d.adjustment(), None);
        d.values[0][0] = "0".into();
        // Colorize uses its own values: the foreground's hue, 25, 0
        d.colorize = true;
        let s = d.settings().unwrap();
        assert!(s.colorize);
        assert_eq!(s.master, [200, 25, 0]);
        d.colorize_values[1] = "-5".into();
        assert_eq!(d.adjustment(), None);
    }

    #[test]
    fn the_strip_puts_red_in_the_middle() {
        let frame = Rect::from_min_size(Pos2::ZERO, SIZE);
        let mid = (pt(STRIP_X.0) + pt(STRIP_X.1)) / 2.0;
        assert!((strip_x(frame, 0.0) - mid).abs() < 0.01);
        assert!((strip_x(frame, 180.0) - pt(STRIP_X.0)).abs() < 0.01);
    }
}
