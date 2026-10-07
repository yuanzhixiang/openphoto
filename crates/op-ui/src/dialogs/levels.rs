//! Image > Adjustments > Levels... (⌘L): Photoshop 2026's classic dialog,
//! 412 × 371 pt. A Channel menu picks RGB or one channel; each keeps its
//! own levels. Sizes are Photoshop points from the dialog's top-left
//! corner.

use crate::native_popup::Entry;
use egui::{Color32, Key, Mesh, Modifiers, Pos2, Rect, Sense, Ui, vec2};
use op_core::adjust::{Adjustment, Levels};

use super::appkit::{self, Pin};
use crate::ps_icons::{self, Icon};
use crate::theme::{color, pt};

pub const SIZE: egui::Vec2 = vec2(pt(412.0), pt(371.0));
pub const CHANNELS: [&str; 4] = ["RGB", "Red", "Green", "Blue"];
/// Input black, gamma, input white, output black, output white.
const RANGES: [(f32, f32); 5] = [
    (0.0, 253.0),
    (0.01, 9.99),
    (2.0, 255.0),
    (0.0, 255.0),
    (0.0, 255.0),
];
const DEFAULTS: [&str; 5] = ["0", "1.00", "255", "0", "255"];
/// Where 0 and 255 sit under the histogram and the output ramp.
const PIN_X: (f32, f32) = (25.75, 279.25);
const HISTOGRAM: [f32; 4] = [25.0, 131.0, 281.0, 230.0];

#[derive(Clone)]
pub struct Dialog {
    /// RGB, Red, Green, Blue.
    pub channel: usize,
    /// Each channel's five values as typed.
    pub values: [[String; 5]; 4],
    /// The composite (all three channels together) and each channel's
    /// histogram.
    histograms: [[u64; 256]; 4],
    /// The pin being dragged: 0–2 input black/gamma/white, 3–4 output.
    drag: Option<usize>,
    /// The Set Black, Gray or White Point eyedropper chosen: a click on
    /// the image sets that point (`sample`).
    pub eyedropper: Option<usize>,
    /// Auto or Options... was clicked (`AdjustDialog` runs them).
    pub wants_auto: bool,
    pub wants_options: bool,
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
            values: std::array::from_fn(|_| DEFAULTS.map(String::from)),
            histograms: [composite, channels[0], channels[1], channels[2]],
            drag: None,
            eyedropper: None,
            wants_auto: false,
            wants_options: false,
        }
    }

    fn levels(&self, c: usize) -> Option<Levels> {
        let v = &self.values[c];
        let mut n = [0f32; 5];
        for i in 0..5 {
            let x: f32 = v[i].trim().parse().ok()?;
            let (lo, hi) = RANGES[i];
            if !(lo..=hi).contains(&x) {
                return None;
            }
            n[i] = x;
        }
        // The black point stays at least 2 below the white point
        if n[0] + 2.0 > n[2] {
            return None;
        }
        Some(Levels {
            input_black: n[0].round() as u8,
            gamma: n[1],
            input_white: n[2].round() as u8,
            output_black: n[3].round() as u8,
            output_white: n[4].round() as u8,
        })
    }

    pub fn adjustment(&self) -> Option<Adjustment> {
        Some(Adjustment::Levels([
            self.levels(0)?,
            self.levels(1)?,
            self.levels(2)?,
            self.levels(3)?,
        ]))
    }

    /// The values of a preset (`adjust_presets::LEVELS`) as the fields show
    /// them.
    fn preset_values(p: &[[i32; 5]; 4]) -> [[String; 5]; 4] {
        p.map(|[black, gamma, white, out_black, out_white]| {
            [
                black.to_string(),
                format!("{:.2}", gamma as f32 / 100.0),
                white.to_string(),
                out_black.to_string(),
                out_white.to_string(),
            ]
        })
    }

    /// The values as a Levels preset file (None while a field is invalid).
    fn encoded(&self) -> Option<Vec<u8>> {
        let mut levels = [[0f32; 5]; 4];
        for (l, v) in levels.iter_mut().zip(&self.values) {
            for (x, t) in l.iter_mut().zip(v) {
                *x = t.trim().parse().ok()?;
            }
        }
        Some(super::preset_files::encode_levels(&levels))
    }

    /// A preset file's values into the fields.
    fn load_preset(&mut self, bytes: &[u8]) {
        if let Some(levels) = super::preset_files::decode_levels(bytes) {
            self.values = levels.map(|[black, gamma, white, out_black, out_white]| {
                [
                    black.to_string(),
                    format!("{gamma:.2}"),
                    white.to_string(),
                    out_black.to_string(),
                    out_white.to_string(),
                ]
            });
        }
    }

    /// "Default" or the preset the values match, else "Custom".
    fn preset(&self) -> &'static str {
        let same = |want: &[[String; 5]; 4]| {
            self.values.iter().zip(want).all(|(v, w)| {
                v.iter()
                    .zip(w)
                    .all(|(a, b)| a.trim().parse::<f32>().ok() == b.parse().ok())
            })
        };
        if same(&std::array::from_fn(|_| DEFAULTS.map(String::from))) {
            return "Default";
        }
        super::adjust_presets::LEVELS
            .iter()
            .find(|(_, p)| same(&Self::preset_values(p)))
            .map_or("Custom", |(name, _)| name)
    }

    /// The chosen eyedropper's click on a pixel of color `rgb` (the image
    /// before adjusting): Set Black Point makes each channel's value its
    /// input black, Set White Point its input white, and Set Gray Point
    /// sets each channel's gamma so the color comes out neutral (at its
    /// mean), as Photoshop's eyedroppers do. The composite channel goes back
    /// to its defaults.
    pub fn sample(&mut self, rgb: [u8; 3]) {
        let Some(k) = self.eyedropper else {
            return;
        };
        let num = |s: &String, d: f32| s.trim().parse::<f32>().unwrap_or(d);
        self.values[0] = DEFAULTS.map(String::from);
        let target = rgb.iter().map(|&v| v as f32).sum::<f32>() / 3.0;
        for (c, &value) in rgb.iter().enumerate() {
            let v = value as f32;
            let row = &mut self.values[c + 1];
            match k {
                0 => row[0] = format!("{}", v.min(253.0).round()),
                2 => row[2] = format!("{}", v.max(2.0).round()),
                _ => {
                    let (black, white) = (num(&row[0], 0.0), num(&row[2], 255.0));
                    let x = ((v - black) / (white - black).max(1.0)).clamp(0.001, 0.999);
                    let t = (target / 255.0).clamp(0.001, 0.999);
                    // out = x ^ (1 / gamma) = t
                    let gamma = (x.ln() / t.ln()).clamp(0.01, 9.99);
                    row[1] = format!("{gamma:.2}");
                }
            }
        }
    }

    /// Auto (with the Auto Color Correction Options' algorithm, computed
    /// by `op_core::auto`): the same values for all three channels go on
    /// the composite, otherwise on each channel.
    pub fn auto(&mut self, channels: [op_core::auto::Channel; 3]) {
        let row = |c: &op_core::auto::Channel| {
            [
                format!("{}", c.black.round()),
                format!("{:.2}", c.gamma),
                format!("{}", c.white.round()),
                c.out_black.to_string(),
                c.out_white.to_string(),
            ]
        };
        self.values = std::array::from_fn(|_| DEFAULTS.map(String::from));
        if channels[0] == channels[1] && channels[1] == channels[2] {
            self.values[0] = row(&channels[0]);
        } else {
            for (values, c) in self.values[1..].iter_mut().zip(&channels) {
                *values = row(c);
            }
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> Option<super::uxp::Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();

        // ⌥2–⌥5 pick RGB, Red, Green, Blue
        for (k, key) in [Key::Num2, Key::Num3, Key::Num4, Key::Num5]
            .into_iter()
            .enumerate()
        {
            if ui.input_mut(|i| i.consume_key(Modifiers::ALT, key)) {
                self.channel = k;
            }
        }

        // Preset
        appkit::label(ui, at(11.0, 50.0), "Preset:");
        use super::preset_files as files;
        let encoded = self.encoded();
        let saved_name = files::shown(files::LEVELS, encoded.as_deref());
        let preset = saved_name.as_deref().unwrap_or(self.preset());
        // Default, Custom, then Photoshop's presets, then the saved ones
        let mut entries = vec![
            Entry::item("Default", preset == "Default"),
            Entry::item("Custom", preset == "Custom").enabled(false),
            Entry::Separator,
        ];
        entries.extend(
            super::adjust_presets::LEVELS
                .iter()
                .map(|(name, _)| Entry::item(*name, preset == *name)),
        );
        let before = entries.len();
        let saved = files::add_saved(&mut entries, files::LEVELS, preset);
        match appkit::popup(
            ui,
            r(59.0, 39.5, 266.0, 60.5),
            "levels-preset",
            preset,
            &entries,
        ) {
            Some(0) => self.values = std::array::from_fn(|_| DEFAULTS.map(String::from)),
            Some(k) if k >= before => {
                if let Some(bytes) = files::picked_saved(k, before, &saved, files::LEVELS) {
                    self.load_preset(&bytes);
                }
            }
            Some(k) if k >= 3 => {
                self.values = Self::preset_values(&super::adjust_presets::LEVELS[k - 3].1);
            }
            _ => {}
        }
        // The gear: Save Preset..., Load Preset..., Delete Current Preset
        let gear = Rect::from_center_size(at(283.5, 50.0), vec2(pt(18.0), pt(18.0)));
        if let Some(bytes) = files::gear(ui, gear, "levels-gear", files::LEVELS, encoded) {
            self.load_preset(&bytes);
        }
        ps_icons::paint_scaled(
            &painter,
            at(283.5, 50.0),
            Icon::Gear,
            appkit::TEXT,
            color::PANEL,
            0.77,
        );

        // The channel group's frame, broken by its title
        let line = |a: Pos2, b: Pos2| {
            painter.line_segment([a, b], egui::Stroke::new(pt(1.0), appkit::GROUP_LINE))
        };
        line(at(11.0, 90.5), at(25.5, 90.5));
        line(at(208.5, 90.5), at(295.5, 90.5));
        line(at(11.0, 90.5), at(11.0, 359.5));
        line(at(295.5, 90.5), at(295.5, 359.5));
        line(at(11.0, 359.5), at(295.5, 359.5));
        appkit::label(ui, at(30.5, 90.5), "Channel:");
        let channels: Vec<Entry> = CHANNELS
            .iter()
            .enumerate()
            .map(|(k, name)| Entry::item(format!("{name}    ⌥{}", k + 2), k == self.channel))
            .collect();
        if let Some(k) = appkit::popup(
            ui,
            r(85.0, 80.0, 206.5, 101.0),
            "levels-channel",
            CHANNELS[self.channel],
            &channels,
        ) {
            self.channel = k;
        }
        let c = self.channel;

        appkit::label(ui, at(20.5, 118.5), "Input Levels:");
        self.histogram(
            ui,
            r(HISTOGRAM[0], HISTOGRAM[1], HISTOGRAM[2], HISTOGRAM[3]),
        );
        for (i, x0) in [(0, 60.0), (1, 150.0), (2, 240.0)] {
            let decimals = if i == 1 { 2 } else { 0 };
            let step = if i == 1 { 0.01 } else { 1.0 };
            appkit::field(
                ui,
                r(x0, 249.0, x0 + 45.0, 268.0),
                &mut self.values[c][i],
                ("levels", c, i),
                RANGES[i],
                step,
                decimals,
                first_frame && i == 0,
            );
        }
        painter.line_segment(
            [at(20.0, 277.0), at(286.0, 277.0)],
            egui::Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
        );
        appkit::label(ui, at(20.5, 290.5), "Output Levels:");
        let ramp = r(24.0, 302.0, 282.0, 315.0);
        let mut mesh = Mesh::default();
        mesh.colored_vertex(ramp.left_top(), Color32::BLACK);
        mesh.colored_vertex(ramp.right_top(), Color32::WHITE);
        mesh.colored_vertex(ramp.right_bottom(), Color32::WHITE);
        mesh.colored_vertex(ramp.left_bottom(), Color32::BLACK);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        painter.add(egui::Shape::mesh(mesh));
        for (i, x0) in [(3, 60.0), (4, 240.0)] {
            appkit::field(
                ui,
                r(x0, 332.0, x0 + 45.0, 351.0),
                &mut self.values[c][i],
                ("levels", c, i),
                RANGES[i],
                1.0,
                0,
                false,
            );
        }
        self.pins(ui, frame);

        // Buttons, eyedroppers (not available here) and Preview
        let button = |ui: &mut Ui, y: f32, label: &str, default: bool| {
            appkit::button(ui, r(313.5, y, 402.5, y + 26.0), label, default, true)
        };
        let ok = button(ui, 38.5, "OK", true);
        let cancel = button(ui, 73.5, "Cancel", false);
        let auto = button(ui, 115.5, "Auto", false);
        if button(ui, 157.5, "Options...", false).clicked() {
            self.wants_options = true;
        }
        for (k, (x, icon)) in [
            (328.5, Icon::EyedropperBlack),
            (358.5, Icon::EyedropperGray),
            (388.5, Icon::EyedropperWhite),
        ]
        .into_iter()
        .enumerate()
        {
            let chosen = self.eyedropper == Some(k);
            if appkit::eyedropper(ui, at(x - 1.5, 213.0), icon, chosen, 1.1) {
                self.eyedropper = (!chosen).then_some(k);
            }
        }
        appkit::checkbox(ui, at(312.5, 243.0), "Preview", preview);

        if cancel.clicked() {
            return Some(super::uxp::Button::Cancel);
        }
        if auto.clicked() {
            self.wants_auto = true;
        }
        let valid = self.adjustment().is_some();
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        ((ok.clicked() || enter) && valid).then_some(super::uxp::Button::Ok)
    }

    fn histogram(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 0, appkit::FIELD);
        let hist = &self.histograms[self.channel];
        let max = hist.iter().copied().max().unwrap_or(0).max(1) as f32;
        let bar = rect.width() / 256.0;
        for (i, &n) in hist.iter().enumerate() {
            if n == 0 {
                continue;
            }
            let h = n as f32 / max * rect.height();
            let x = rect.left() + i as f32 * bar;
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, rect.bottom() - h),
                    Pos2::new(x + bar, rect.bottom()),
                ),
                0,
                Color32::from_gray(0xd0),
            );
        }
    }

    /// The input pins (black, gamma, white) under the histogram and the
    /// output pins under the ramp; dragging one sets its value.
    fn pins(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let (x0, x1) = (at(PIN_X.0, 0.0).x, at(PIN_X.1, 0.0).x);
        let to_x = |v: f32| x0 + (x1 - x0) * v / 255.0;
        let to_v = |x: f32| ((x - x0) / (x1 - x0) * 255.0).round().clamp(0.0, 255.0);
        let c = self.channel;
        let num = |s: &String, d: f32| s.trim().parse::<f32>().unwrap_or(d);
        let [b, g, w, ob, ow] =
            [0, 1, 2, 3, 4].map(|i| num(&self.values[c][i], DEFAULTS[i].parse().unwrap()));
        // The gamma pin sits between the black and white ones, at
        // 0.5^gamma of the way
        let gx = to_x(b + (w - b) * 0.5f32.powf(g));
        let xs = [to_x(b), gx, to_x(w), to_x(ob), to_x(ow)];
        let (top_in, top_out) = (at(0.0, 233.0).y, at(0.0, 315.0).y);
        for (row, top) in [(0..3, top_in), (3..5, top_out)] {
            let area = Rect::from_min_max(
                Pos2::new(x0 - pt(6.0), top),
                Pos2::new(x1 + pt(6.0), top + pt(11.0)),
            );
            let response = ui.interact(
                area,
                ui.id().with(("levels-pins", row.start)),
                Sense::drag(),
            );
            if response.drag_started()
                && let Some(p) = response.interact_pointer_pos()
            {
                self.drag = row
                    .clone()
                    .min_by(|&a, &b| (xs[a] - p.x).abs().total_cmp(&(xs[b] - p.x).abs()));
            }
            if let (Some(i), Some(p)) = (self.drag, response.interact_pointer_pos())
                && row.contains(&i)
                && response.dragged()
            {
                let v = to_v(p.x);
                self.values[c][i] = match i {
                    0 => format!("{}", v.min(w - 2.0)),
                    2 => format!("{}", v.max(b + 2.0)),
                    1 => {
                        let t = ((v - b) / (w - b)).clamp(0.001, 0.999);
                        format!("{:.2}", (t.ln() / 0.5f32.ln()).clamp(0.01, 9.99))
                    }
                    _ => format!("{v}"),
                };
            }
            if response.drag_stopped() {
                self.drag = None;
            }
        }
        let painter = ui.painter();
        for (i, kind) in [Pin::Black, Pin::Gray, Pin::White, Pin::Black, Pin::White]
            .into_iter()
            .enumerate()
        {
            let top = if i < 3 { top_in } else { top_out };
            appkit::pin(painter, Pos2::new(xs[i], top), kind);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_files_round_trip() {
        let mut d = Dialog::new([[0; 256]; 3]);
        d.values[0] = ["15", "1.20", "240", "5", "250"].map(String::from);
        let bytes = d.encoded().unwrap();
        let mut e = Dialog::new([[0; 256]; 3]);
        e.load_preset(&bytes);
        assert_eq!(e.adjustment(), d.adjustment());
    }

    #[test]
    fn photoshops_presets() {
        let mut d = Dialog::new([[1; 256]; 3]);
        d.values = Dialog::preset_values(&super::super::adjust_presets::LEVELS[2].1);
        assert_eq!(d.preset(), "Increase Contrast 2");
        assert_eq!(d.values[0][0], "20");
        assert_eq!(d.values[0][1], "1.00");
        d.values[0][1] = "1.6".into();
        d.values[0][0] = "0".into();
        d.values[0][2] = "255".into();
        assert_eq!(d.preset(), "Lighten Shadows");
        d.values[1][0] = "3".into();
        assert_eq!(d.preset(), "Custom");
    }

    #[test]
    fn channels_keep_their_levels() {
        let mut d = Dialog::new([[1; 256]; 3]);
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::Levels(Levels::IDENTITY.composite()))
        );
        assert_eq!(d.preset(), "Default");
        d.values[1][0] = "20".into();
        d.values[0][1] = "1.50".into();
        let Some(Adjustment::Levels(l)) = d.adjustment() else {
            panic!()
        };
        assert_eq!(
            (l[0].gamma, l[1].input_black, l[2].input_black),
            (1.5, 20, 0)
        );
        assert_eq!(d.preset(), "Custom");
        // Black too close to white
        d.values[2][0] = "250".into();
        d.values[2][2] = "251".into();
        assert_eq!(d.adjustment(), None);
    }

    #[test]
    fn auto_fills_the_composite_or_each_channel() {
        use op_core::auto::{Channel, Options, Targets, compute};
        let mut d = Dialog::new([[0; 256]; 3]);
        let px: Vec<[u8; 3]> = (0..1000).map(|i| [40 + (i % 160) as u8, 60, 80]).collect();
        d.auto(compute(&px, &Options::auto_tone(Targets::default())));
        assert_eq!(d.values[1][0], "40");
        assert_eq!(d.values[1][2], "199");
        assert_eq!(d.values[0][0], "0");
        let same = Channel {
            black: 10.0,
            gamma: 1.2,
            white: 240.0,
            out_black: 0,
            out_white: 255,
        };
        d.auto([same; 3]);
        assert_eq!(d.values[0][..3], ["10", "1.20", "240"]);
        assert_eq!(d.values[1][0], "0");
    }
}
