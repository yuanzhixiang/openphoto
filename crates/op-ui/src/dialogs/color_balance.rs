//! Image > Adjustments > Color Balance...: Photoshop 2026's UXP dialog,
//! 447 × 295 pt. A tone picker (Shadows, Midtones, Highlights) chooses
//! which three sliders show; each tone keeps its own values. Sizes are
//! Photoshop points from the dialog's top-left corner.

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};
use op_core::adjust::Adjustment;

use super::uxp;
use crate::theme::pt;

pub const SIZE: egui::Vec2 = vec2(pt(447.0), pt(295.0));
const RANGE: (f32, f32) = (-100.0, 100.0);
pub const TONES: [&str; 3] = ["Shadows", "Midtones", "Highlights"];
const SLIDERS: [&str; 3] = ["Cyan to Red", "Magenta to Green", "Yellow to Blue"];

// The tracks' colors at 17 even stops, sampled from Photoshop 2026
const CYAN_RED: [[u8; 3]; 17] = [
    [116, 251, 252],
    [111, 234, 236],
    [105, 220, 221],
    [103, 204, 206],
    [102, 188, 190],
    [105, 173, 174],
    [111, 158, 159],
    [118, 143, 144],
    [127, 128, 128],
    [138, 113, 112],
    [150, 100, 98],
    [162, 85, 84],
    [176, 73, 70],
    [190, 62, 56],
    [204, 56, 46],
    [218, 52, 38],
    [233, 51, 36],
];
const MAGENTA_GREEN: [[u8; 3]; 17] = [
    [233, 52, 246],
    [219, 51, 231],
    [205, 55, 215],
    [190, 63, 201],
    [176, 74, 186],
    [163, 85, 170],
    [150, 98, 156],
    [140, 113, 142],
    [128, 128, 128],
    [118, 143, 114],
    [111, 157, 102],
    [106, 172, 91],
    [102, 188, 83],
    [103, 204, 77],
    [106, 219, 74],
    [111, 234, 74],
    [117, 251, 76],
];
const YELLOW_BLUE: [[u8; 3]; 17] = [
    [254, 255, 84],
    [239, 238, 81],
    [223, 222, 79],
    [207, 207, 81],
    [191, 192, 86],
    [175, 175, 94],
    [159, 159, 104],
    [145, 144, 115],
    [128, 128, 128],
    [112, 112, 141],
    [96, 96, 154],
    [81, 80, 168],
    [64, 64, 184],
    [48, 49, 199],
    [33, 32, 214],
    [18, 17, 229],
    [2, 1, 244],
];
const TRACKS: [&[[u8; 3]]; 3] = [&CYAN_RED, &MAGENTA_GREEN, &YELLOW_BLUE];

/// The tone swatches' centers (x) and fills, and their labels' left edges.
const SWATCHES: [(f32, Color32, f32); 3] = [
    (68.0, Color32::BLACK, 85.5),
    (153.0, Color32::from_gray(0x77), 170.5),
    (240.0, Color32::from_gray(0xfd), 258.0),
];

#[derive(Clone)]
pub struct Dialog {
    /// Text of the three sliders for each tone.
    pub values: [[String; 3]; 3],
    /// The tone being edited (Midtones at first).
    pub tone: usize,
    pub preserve_luminosity: bool,
    focus: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            values: std::array::from_fn(|_| std::array::from_fn(|_| "0".to_string())),
            tone: 1,
            preserve_luminosity: true,
            focus: false,
        }
    }
}

fn parse(text: &str) -> Option<i32> {
    let v: f32 = text.trim().parse().ok()?;
    (RANGE.0..=RANGE.1).contains(&v).then_some(v.round() as i32)
}

impl Dialog {
    pub fn adjustment(&self) -> Option<Adjustment> {
        let tone = |t: usize| -> Option<[i32; 3]> {
            Some([
                parse(&self.values[t][0])?,
                parse(&self.values[t][1])?,
                parse(&self.values[t][2])?,
            ])
        };
        Some(Adjustment::ColorBalance {
            shadows: tone(0)?,
            midtones: tone(1)?,
            highlights: tone(2)?,
            preserve_luminosity: self.preserve_luminosity,
        })
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

        uxp::label(ui, at(20.0, 60.5), "Tone");
        for (i, &(cx, fill, label_x)) in SWATCHES.iter().enumerate() {
            let center = at(cx, 60.0);
            let label = uxp::label(ui, at(label_x, 60.5), TONES[i]);
            let hit = Rect::from_min_max(
                center - vec2(pt(12.0), pt(12.0)),
                Pos2::new(label.right(), center.y + pt(12.0)),
            );
            if ui
                .interact(hit, ui.id().with(("cb-tone", i)), Sense::click())
                .clicked()
            {
                self.tone = i;
                // The first box takes the focus with the new tone's values
                self.focus = true;
            }
            swatch(ui.painter(), center, fill, self.tone == i);
        }

        let focus_first = first_frame || std::mem::take(&mut self.focus);
        for k in 0..3 {
            let y = 92.0 + 48.0 * k as f32;
            uxp::label(ui, at(20.0, y + 12.0), SLIDERS[k]);
            let text = &mut self.values[self.tone][k];
            uxp::number_box(
                ui,
                r(249.0, y, 305.5, y + 24.0),
                text,
                ("cb-field", self.tone, k),
                RANGE,
                focus_first && k == 0,
            );
            let value = parse(text).unwrap_or(0) as f32;
            let colors = uxp::stops(TRACKS[k]);
            if let Some(v) = uxp::slider(
                ui,
                ("cb", k),
                at(20.0, 0.0).x,
                at(305.5, 0.0).x,
                at(0.0, y + 36.0).y,
                value,
                RANGE,
                &colors,
            ) {
                *text = format!("{v}");
            }
        }
        uxp::checkbox(
            ui,
            at(20.0, 257.0),
            "Preserve Luminosity",
            &mut self.preserve_luminosity,
        );
        uxp::preview(ui, at(318.0, 127.0), preview);
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), false)
    }
}

/// A tone swatch, 24 pt across: the chosen one inside a 2 pt white ring
/// and a 2 pt dark gap, the others with a fine gray edge.
fn swatch(painter: &egui::Painter, center: Pos2, fill: Color32, chosen: bool) {
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
