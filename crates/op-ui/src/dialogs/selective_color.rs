//! Image > Adjustments > Selective Color...: Photoshop 2026's UXP dialog,
//! 473 × 400 pt. Nine swatches pick the color range (Reds … Blacks); each
//! keeps its own cyan, magenta, yellow and black. Sizes are Photoshop
//! points from the dialog's top-left corner.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};
use op_core::adjust::Adjustment;

use super::{common, uxp};
use crate::ps_icons::{self, Icon};
use crate::theme::{color, pt};

pub const SIZE: egui::Vec2 = vec2(pt(473.0), pt(400.0));
const RANGE: (f32, f32) = (-100.0, 100.0);
pub const COLORS: [&str; 9] = [
    "Reds", "Yellows", "Greens", "Cyans", "Blues", "Magentas", "Whites", "Neutrals", "Blacks",
];
const FILLS: [[u8; 3]; 9] = [
    [234, 51, 36],
    [255, 255, 85],
    [117, 252, 76],
    [117, 251, 253],
    [0, 0, 245],
    [234, 51, 247],
    [255, 255, 255],
    [128, 127, 127],
    [0, 0, 1],
];
// Track colors at 17 even stops, sampled from Photoshop 2026
const CYAN_TRACK: [[u8; 3]; 17] = [
    [226, 49, 37],
    [220, 51, 39],
    [205, 55, 46],
    [191, 63, 55],
    [176, 73, 69],
    [163, 85, 83],
    [151, 99, 98],
    [138, 112, 112],
    [128, 128, 128],
    [119, 142, 142],
    [111, 157, 158],
    [106, 173, 173],
    [102, 189, 190],
    [103, 205, 205],
    [105, 220, 221],
    [110, 236, 237],
    [112, 241, 242],
];
const MAGENTA_TRACK: [[u8; 3]; 17] = [
    [112, 241, 73],
    [110, 236, 73],
    [105, 220, 73],
    [103, 204, 76],
    [103, 189, 82],
    [105, 173, 91],
    [110, 158, 103],
    [118, 142, 114],
    [128, 128, 128],
    [138, 114, 141],
    [150, 100, 155],
    [164, 86, 170],
    [177, 73, 185],
    [191, 63, 200],
    [205, 56, 215],
    [220, 52, 230],
    [226, 51, 235],
];
const YELLOW_TRACK: [[u8; 3]; 17] = [
    [2, 2, 244],
    [16, 16, 229],
    [32, 33, 214],
    [49, 49, 199],
    [64, 64, 184],
    [80, 79, 169],
    [96, 95, 155],
    [112, 112, 141],
    [128, 128, 128],
    [143, 143, 115],
    [159, 159, 104],
    [175, 175, 95],
    [191, 191, 86],
    [206, 207, 81],
    [222, 223, 80],
    [240, 240, 81],
    [254, 255, 84],
];
const BLACK_TRACK: [[u8; 3]; 17] = [
    [254, 254, 254],
    [239, 239, 239],
    [223, 224, 223],
    [207, 207, 207],
    [191, 191, 191],
    [174, 176, 176],
    [160, 160, 158],
    [143, 143, 144],
    [128, 128, 128],
    [111, 111, 112],
    [96, 96, 95],
    [80, 80, 79],
    [64, 64, 64],
    [48, 48, 48],
    [32, 32, 32],
    [16, 16, 16],
    [1, 1, 1],
];

#[derive(Clone)]
pub struct Dialog {
    /// Cyan, magenta, yellow and black text for each of the nine ranges.
    pub values: [[String; 4]; 9],
    pub selected: usize,
    pub absolute: bool,
    focus: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            values: std::array::from_fn(|_| std::array::from_fn(|_| "0".to_string())),
            selected: 0,
            absolute: false,
            focus: false,
        }
    }
}

fn parse(text: &str) -> Option<i32> {
    let v: f32 = text.trim().trim_end_matches('%').trim().parse().ok()?;
    (RANGE.0..=RANGE.1).contains(&v).then_some(v.round() as i32)
}

impl Dialog {
    pub fn adjustment(&self) -> Option<Adjustment> {
        let mut colors = [[0; 4]; 9];
        for (c, v) in colors.iter_mut().zip(&self.values) {
            for k in 0..4 {
                c[k] = parse(&v[k])?;
            }
        }
        Some(Adjustment::SelectiveColor {
            colors,
            absolute: self.absolute,
        })
    }

    fn preset(&self) -> &'static str {
        if self.values.iter().flatten().all(|v| parse(v) == Some(0)) {
            "Default"
        } else {
            "Custom"
        }
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

        uxp::label(ui, at(20.0, 61.0), "Preset");
        let preset = self.preset();
        let mut reset = false;
        common::ps_dropdown(
            ui,
            r(56.0, 48.5, 302.0, 73.5),
            "selective-preset",
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
            },
        );
        if reset {
            self.values = Self::default().values;
        }
        ps_icons::paint(
            &painter,
            at(319.0, 61.0),
            Icon::PresetMenu,
            uxp::TEXT,
            color::PANEL,
        );

        for (i, [r, g, b]) in FILLS.into_iter().enumerate() {
            let center = at(32.0 + 36.0 * i as f32, 96.0);
            let hit = Rect::from_center_size(center, vec2(pt(24.0), pt(24.0)));
            let response = ui.interact(hit, ui.id().with(("selective-range", i)), Sense::click());
            if response.clicked() {
                self.selected = i;
                self.focus = true;
            }
            response.on_hover_text(COLORS[i]);
            let fill = Color32::from_rgb(r, g, b);
            if self.selected == i {
                painter.circle_filled(center, pt(12.0), Color32::WHITE);
                painter.circle_filled(center, pt(10.0), Color32::from_gray(0x45));
                painter.circle_filled(center, pt(8.0), fill);
            } else {
                painter.circle(
                    center,
                    pt(11.75),
                    fill,
                    Stroke::new(pt(0.5), Color32::from_gray(0xa7)),
                );
            }
        }

        let focus = first_frame || std::mem::take(&mut self.focus);
        let tracks: [&'static [[u8; 3]]; 4] =
            [&CYAN_TRACK, &MAGENTA_TRACK, &YELLOW_TRACK, &BLACK_TRACK];
        for (k, name) in ["Cyan", "Magenta", "Yellow", "Black"]
            .into_iter()
            .enumerate()
        {
            let y = 120.0 + 55.0 * k as f32;
            uxp::label(ui, at(20.0, y + 12.5), name);
            let text = &mut self.values[self.selected][k];
            let field = r(277.0, y, 331.5, y + 24.0);
            uxp::number_box(
                ui,
                field,
                text,
                ("selective-field", self.selected, k),
                RANGE,
                focus && k == 0,
            );
            // The percent sign follows the number
            let shown = painter.layout_no_wrap(text.clone(), uxp::font(), uxp::TEXT);
            if !text.contains('%') {
                painter.text(
                    Pos2::new(
                        field.left() + pt(11.0) + shown.size().x,
                        field.center().y + pt(0.75),
                    ),
                    Align2::LEFT_CENTER,
                    "%",
                    uxp::font(),
                    uxp::TEXT,
                );
            }
            let value = parse(text).unwrap_or(0) as f32;
            let colors = uxp::stops(tracks[k]);
            if let Some(v) = uxp::slider(
                ui,
                ("selective", k),
                at(20.0, 0.0).x,
                at(331.5, 0.0).x,
                at(0.0, y + 36.0).y,
                value,
                RANGE,
                &colors,
            ) {
                *text = format!("{v}");
            }
        }

        uxp::label(ui, at(20.0, 348.5), "Method");
        if uxp::radio(ui, at(26.0, 368.5), "Relative", !self.absolute) {
            self.absolute = false;
        }
        if uxp::radio(ui, at(98.0, 368.5), "Absolute", self.absolute) {
            self.absolute = true;
        }
        uxp::preview(ui, at(344.0, 127.0), preview);
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_keep_their_values() {
        let mut d = Dialog::default();
        assert_eq!(d.preset(), "Default");
        d.values[7][3] = "30%".into();
        d.values[0][0] = "-20".into();
        d.absolute = true;
        let Some(Adjustment::SelectiveColor { colors, absolute }) = d.adjustment() else {
            panic!()
        };
        assert!(absolute);
        assert_eq!((colors[7], colors[0][0]), ([0, 0, 0, 30], -20));
        assert_eq!(d.preset(), "Custom");
        d.values[2][1] = "101".into();
        assert_eq!(d.adjustment(), None);
    }
}
