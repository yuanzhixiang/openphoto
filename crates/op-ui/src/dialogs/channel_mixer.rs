//! Image > Adjustments > Channel Mixer...: Photoshop 2026's UXP dialog,
//! 401 × 426 pt. The output channel swatches pick which channel's mix the
//! sliders show; Monochrome mixes one gray instead. Sizes are Photoshop
//! points from the dialog's top-left corner.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};
use op_core::adjust::Adjustment;

use super::{common, uxp};
use crate::ps_icons::{self, Icon};
use crate::theme::{color, pt};

pub const SIZE: egui::Vec2 = vec2(pt(401.0), pt(426.0));
const RANGE: (f32, f32) = (-200.0, 200.0);
const SWATCHES: [(f32, [u8; 3]); 3] = [
    (121.0, [234, 51, 35]),
    (150.0, [117, 251, 76]),
    (178.75, [0, 0, 245]),
];

// Track colors at 17 even stops, sampled from Photoshop 2026
const RED_TRACK: [[u8; 3]; 17] = [
    [0, 0, 0],
    [28, 2, 1],
    [57, 7, 4],
    [88, 13, 7],
    [117, 20, 11],
    [147, 28, 17],
    [176, 35, 24],
    [205, 44, 29],
    [234, 51, 35],
    [234, 61, 49],
    [235, 80, 73],
    [236, 106, 101],
    [238, 134, 131],
    [241, 163, 161],
    [244, 194, 192],
    [249, 223, 224],
    [255, 255, 255],
];
const GREEN_TRACK: [[u8; 3]; 17] = [
    [0, 0, 0],
    [9, 32, 3],
    [25, 64, 11],
    [39, 95, 23],
    [55, 126, 33],
    [70, 157, 44],
    [85, 189, 55],
    [102, 221, 66],
    [117, 251, 76],
    [121, 251, 82],
    [128, 251, 97],
    [142, 252, 117],
    [159, 252, 141],
    [181, 253, 168],
    [204, 253, 196],
    [229, 255, 224],
    [255, 255, 255],
];
const BLUE_TRACK: [[u8; 3]; 17] = [
    [0, 0, 0],
    [0, 1, 31],
    [0, 0, 62],
    [0, 1, 92],
    [0, 0, 122],
    [0, 1, 153],
    [0, 0, 184],
    [0, 0, 215],
    [0, 0, 245],
    [32, 31, 245],
    [64, 63, 246],
    [96, 95, 246],
    [126, 126, 246],
    [159, 159, 248],
    [190, 191, 250],
    [222, 222, 252],
    [255, 255, 255],
];
const CONSTANT_TRACK: [[u8; 3]; 3] = [[0, 0, 0], [128, 128, 128], [255, 255, 255]];

#[derive(Clone)]
pub struct Dialog {
    /// Red, green, blue and constant text for each output channel.
    pub rows: [[String; 4]; 3],
    /// The gray's mix with Monochrome (Photoshop starts it at 40/40/20).
    pub gray: [String; 4],
    pub output: usize,
    pub monochrome: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        let row = |c: usize| std::array::from_fn(|i| if i == c { "100" } else { "0" }.to_string());
        Self {
            rows: [row(0), row(1), row(2)],
            gray: ["40", "40", "20", "0"].map(String::from),
            output: 0,
            monochrome: false,
        }
    }
}

fn parse(text: &str) -> Option<i32> {
    let v: f32 = text.trim().parse().ok()?;
    (RANGE.0..=RANGE.1).contains(&v).then_some(v.round() as i32)
}

impl Dialog {
    pub fn adjustment(&self) -> Option<Adjustment> {
        let row = |r: &[String; 4]| -> Option<[i32; 4]> {
            Some([parse(&r[0])?, parse(&r[1])?, parse(&r[2])?, parse(&r[3])?])
        };
        Some(if self.monochrome {
            Adjustment::ChannelMixer {
                rows: [row(&self.gray)?, [0; 4], [0; 4]],
                monochrome: true,
            }
        } else {
            Adjustment::ChannelMixer {
                rows: [
                    row(&self.rows[0])?,
                    row(&self.rows[1])?,
                    row(&self.rows[2])?,
                ],
                monochrome: false,
            }
        })
    }

    fn preset(&self) -> &'static str {
        if self.rows == Self::default().rows && !self.monochrome {
            return "Default";
        }
        super::adjust_presets::CHANNEL_MIXER
            .iter()
            .find(|(_, [r, g, b])| {
                self.monochrome
                    && self.rows == Self::default().rows
                    && self
                        .gray
                        .iter()
                        .zip([*r, *g, *b, 0])
                        .all(|(t, v)| parse(t) == Some(v))
            })
            .map_or("Custom", |(name, _)| name)
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
        let mut picked = None;
        common::ps_dropdown(
            ui,
            r(56.0, 48.5, 231.0, 73.5),
            "mixer-preset",
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
                for (k, (name, _)) in super::adjust_presets::CHANNEL_MIXER.iter().enumerate() {
                    if ui.selectable_label(preset == *name, *name).clicked() {
                        picked = Some(k);
                    }
                }
            },
        );
        if reset {
            *self = Self::default();
        }
        if let Some(k) = picked {
            *self = Self::default();
            let [r, g, b] = super::adjust_presets::CHANNEL_MIXER[k].1;
            self.monochrome = true;
            self.gray = [r, g, b, 0].map(|v| v.to_string());
        }
        ps_icons::paint(
            &painter,
            at(248.0, 61.0),
            Icon::PresetMenu,
            uxp::TEXT,
            color::PANEL,
        );

        uxp::label(ui, at(20.0, 97.0), "Output channel");
        if self.monochrome {
            swatch(&painter, at(121.0, 96.0), [128, 128, 128], true);
        } else {
            for (i, &(x, fill)) in SWATCHES.iter().enumerate() {
                let center = at(x, 96.0);
                let hit = Rect::from_center_size(center, vec2(pt(24.0), pt(24.0)));
                if ui
                    .interact(hit, ui.id().with(("mixer-output", i)), Sense::click())
                    .clicked()
                {
                    self.output = i;
                }
                swatch(&painter, center, fill, self.output == i);
            }
        }
        uxp::checkbox(ui, at(20.0, 127.0), "Monochrome", &mut self.monochrome);

        let row = if self.monochrome {
            &mut self.gray
        } else {
            &mut self.rows[self.output]
        };
        let tracks: [&'static [[u8; 3]]; 4] =
            [&RED_TRACK, &GREEN_TRACK, &BLUE_TRACK, &CONSTANT_TRACK];
        for (k, (name, y)) in [
            ("Red", 156.0),
            ("Green", 211.0),
            ("Blue", 266.0),
            ("Constant", 363.0),
        ]
        .into_iter()
        .enumerate()
        {
            uxp::label(ui, at(20.0, y + 12.5), name);
            uxp::number_box(
                ui,
                r(203.0, y, 260.0, y + 24.0),
                &mut row[k],
                ("mixer-field", k),
                RANGE,
                first_frame && k == 0,
            );
            let value = parse(&row[k]).unwrap_or(0) as f32;
            let colors = uxp::stops(tracks[k]);
            if let Some(v) = uxp::slider(
                ui,
                ("mixer", k),
                at(20.0, 0.0).x,
                at(260.0, 0.0).x,
                at(0.0, y + 36.0).y,
                value,
                RANGE,
                &colors,
            ) {
                row[k] = format!("{v}");
            }
        }
        // Total of the three sources, then a rule above Constant
        let total: i32 = (0..3).map(|k| parse(&row[k]).unwrap_or(0)).sum();
        uxp::label(ui, at(20.0, 330.0), "Total");
        painter.text(
            at(238.5, 330.75),
            Align2::RIGHT_CENTER,
            total.to_string(),
            uxp::font(),
            uxp::TEXT,
        );
        painter.text(
            at(250.0, 330.75),
            Align2::LEFT_CENTER,
            "%",
            uxp::font(),
            uxp::TEXT,
        );
        painter.line_segment(
            [at(20.0, 350.0), at(260.0, 350.0)],
            Stroke::new(pt(1.0), uxp::TRACK),
        );

        uxp::preview(ui, at(272.0, 127.0), preview);
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), false)
    }
}

/// A 24 pt swatch, as in Color Balance.
fn swatch(painter: &egui::Painter, center: Pos2, [r, g, b]: [u8; 3], chosen: bool) {
    let fill = Color32::from_rgb(r, g, b);
    if chosen {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photoshops_presets() {
        let mut d = Dialog {
            monochrome: true,
            gray: ["-70", "200", "-30", "0"].map(String::from),
            ..Default::default()
        };
        assert_eq!(d.preset(), "Black & White Infrared");
        d.gray[3] = "5".into();
        assert_eq!(d.preset(), "Custom");
    }

    #[test]
    fn rows_and_monochrome() {
        let mut d = Dialog::default();
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::ChannelMixer {
                rows: [[100, 0, 0, 0], [0, 100, 0, 0], [0, 0, 100, 0]],
                monochrome: false
            })
        );
        assert_eq!(d.preset(), "Default");
        d.rows[1][0] = "-40".into();
        assert_eq!(d.preset(), "Custom");
        d.monochrome = true;
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::ChannelMixer {
                rows: [[40, 40, 20, 0], [0; 4], [0; 4]],
                monochrome: true
            })
        );
        d.gray[3] = "201".into();
        assert_eq!(d.adjustment(), None);
    }
}
