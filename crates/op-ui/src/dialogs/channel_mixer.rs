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

    /// The settings as a Channel Mixer preset file (None while a field is
    /// invalid).
    fn encoded(&self) -> Option<Vec<u8>> {
        let four = |t: &[String; 4]| -> Option<[i32; 4]> {
            Some([parse(&t[0])?, parse(&t[1])?, parse(&t[2])?, parse(&t[3])?])
        };
        let rows = [
            four(&self.rows[0])?,
            four(&self.rows[1])?,
            four(&self.rows[2])?,
        ];
        Some(super::preset_files::encode_channel_mixer(
            &rows,
            &four(&self.gray)?,
            self.monochrome,
        ))
    }

    /// A preset file's settings into the dialog.
    fn load_preset(&mut self, bytes: &[u8]) {
        if let Some((rows, gray, monochrome)) = super::preset_files::decode_channel_mixer(bytes) {
            self.rows = rows.map(|r| r.map(|v| v.to_string()));
            self.gray = gray.map(|v| v.to_string());
            self.monochrome = monochrome;
        }
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
        use super::preset_files as files;
        let encoded = self.encoded();
        let saved_name = files::shown(files::CHANNEL_MIXER, encoded.as_deref());
        let preset = saved_name.as_deref().unwrap_or(self.preset());
        let saved = files::CHANNEL_MIXER.saved();
        let mut reset = false;
        let mut picked = None;
        let mut picked_saved = None;
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
        if let Some(bytes) =
            picked_saved.and_then(|k| files::load_saved(files::CHANNEL_MIXER, &saved[k]))
        {
            self.load_preset(&bytes);
        }
        let gear = Rect::from_center_size(at(248.0, 61.0), vec2(pt(18.0), pt(18.0)));
        if let Some(bytes) = files::gear(ui, gear, "mixer-gear", files::CHANNEL_MIXER, encoded) {
            self.load_preset(&bytes);
        }

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
        // Over 100% Photoshop warns that the mix may clip: a yellow
        // caution triangle before the total
        if total > 100 {
            let width = painter
                .layout_no_wrap(total.to_string(), uxp::font(), uxp::TEXT)
                .size()
                .x;
            let c = at(238.5, 330.0) - vec2(width + pt(12.0), 0.0);
            let (h, w) = (pt(11.0), pt(12.0));
            painter.add(egui::Shape::convex_polygon(
                vec![
                    c + vec2(0.0, -h / 2.0),
                    c + vec2(w / 2.0, h / 2.0),
                    c + vec2(-w / 2.0, h / 2.0),
                ],
                Color32::from_rgb(0xf5, 0xc5, 0x18),
                Stroke::NONE,
            ));
            painter.text(
                c + vec2(0.0, pt(1.5)),
                Align2::CENTER_CENTER,
                "!",
                crate::theme::uxp_bold(pt(8.5)),
                Color32::BLACK,
            );
        }
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
    fn preset_files_round_trip() {
        let mut d = Dialog::default();
        d.rows[1] = ["10", "80", "10", "-5"].map(String::from);
        let mut e = Dialog::default();
        e.load_preset(&d.encoded().unwrap());
        assert_eq!(e.rows, d.rows);
    }

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
