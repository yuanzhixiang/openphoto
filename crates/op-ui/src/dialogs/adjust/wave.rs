//! Distort › Wave's dialog, as Photoshop 2026 lays it out (553 × 396 pt,
//! measured from a 2x capture): Number of Generators, Wavelength and
//! Amplitude (Min. and Max. fields over two sliders each) and Scale on the
//! left; Type, the preview, Randomize and Undefined Areas on the right.

use egui::{Align2, Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, vec2};

use super::{AdjustDialog, Outcome};
use crate::dialogs::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(553.0), pt(396.0));

/// The settings' order (`Kind::params`).
const GENERATORS: usize = 0;
const TYPE: usize = 7;
const UNDEFINED: usize = 8;

/// The preview box.
pub const PREVIEW: [f32; 4] = [294.5, 130.5, 540.5, 281.5];

/// A slider's track: x 10–277.5, 3 pt tall; pins travel from 16 to 272.5.
const TRACK: (f32, f32) = (10.0, 277.5);
const PINS: (f32, f32) = (16.0, 272.5);

impl AdjustDialog {
    pub(super) fn wave_ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Wave", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        let text = |ui: &Ui, x: f32, y: f32, align: Align2, s: &str| {
            appkit::text(ui, at(x, y), align, s, appkit::TEXT);
        };

        // Number of Generators
        text(ui, 15.5, 44.0, Align2::LEFT_CENTER, "Number of Generators:");
        self.wave_field(ui, GENERATORS, r(246.0, 35.5, 273.5, 53.5), true);
        self.wave_track(ui, GENERATORS, 62.0, false, frame);

        // Wavelength, Amplitude and Scale: two fields under their headers,
        // the first's slider (pin down) over the second's (pin up)
        for (label, label_y, field_y, heads, first, tracks, percent) in [
            (
                "Wavelength:",
                106.5,
                96.0,
                ["Min.", "Max."],
                1,
                (140.0, 155.0),
                false,
            ),
            (
                "Amplitude:",
                205.0,
                195.0,
                ["Min.", "Max."],
                3,
                (239.0, 254.0),
                false,
            ),
            (
                "Scale:",
                308.0,
                296.0,
                ["Horiz.", "Vert."],
                5,
                (337.0, 349.0),
                true,
            ),
        ] {
            text(ui, 15.5, label_y, Align2::LEFT_CENTER, label);
            let xs = if percent {
                [165.0, 236.0]
            } else {
                [165.0, 231.0]
            };
            for (k, x) in xs.into_iter().enumerate() {
                text(
                    ui,
                    x + 17.0,
                    field_y - 12.5,
                    Align2::CENTER_CENTER,
                    heads[k],
                );
                self.wave_field(
                    ui,
                    first + k,
                    r(x, field_y, x + 34.0, field_y + 19.0),
                    false,
                );
                if percent {
                    text(ui, x + 40.5, field_y + 9.5, Align2::LEFT_CENTER, "%");
                }
            }
            // Scale's sliders both point up
            self.wave_track(ui, first, tracks.0, !percent, frame);
            self.wave_track(ui, first + 1, tracks.1, false, frame);
        }

        // Type
        let group = |ui: &Ui, rect: Rect, title: &str, x: f32| {
            let galley = theme::tracked_galley(ui.painter(), title, appkit::font(), appkit::TEXT);
            let x = at(x, 0.0).x;
            appkit::group(
                ui.painter(),
                rect,
                (x - pt(4.0), x + galley.size().x + pt(4.0)),
            );
            appkit::label(ui, Pos2::new(x, rect.top() + pt(1.5)), title);
        };
        group(ui, r(298.5, 46.0, 378.5, 117.0), "Type:", 316.0);
        let kind = self.value(TYPE).unwrap_or(0.0) as usize;
        for (k, (y, label)) in [(64.0, "Sine"), (84.0, "Triangle"), (104.0, "Square")]
            .into_iter()
            .enumerate()
        {
            if appkit::radio_with(ui, at(312.0, y), label, kind == k, (13.0, appkit::font())) {
                self.values[TYPE] = k.to_string();
            }
        }

        // The preview, and Randomize under it: a new random pattern
        let preview = r(PREVIEW[0], PREVIEW[1], PREVIEW[2], PREVIEW[3]);
        painter.rect(
            preview,
            0,
            Color32::from_gray(0x4c),
            Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
            egui::StrokeKind::Inside,
        );
        self.pane_ui(ui, preview.shrink(pt(1.0)));
        if super::flat_button(
            ui,
            r(371.0, 291.0, 464.0, 317.0),
            "Randomize",
            "wave-randomize",
        ) {
            self.extra.seed = self
                .extra
                .seed
                .wrapping_mul(747_796_405)
                .wrapping_add(2_891_336_453);
        }

        // Undefined Areas
        group(ui, r(295.0, 333.5, 540.5, 386.0), "Undefined Areas:", 312.5);
        let undefined = self.value(UNDEFINED).unwrap_or(1.0) as usize;
        for (k, (y, label)) in [(352.0, "Wrap Around"), (372.0, "Repeat Edge Pixels")]
            .into_iter()
            .enumerate()
        {
            if appkit::radio_with(
                ui,
                at(308.0, y),
                label,
                undefined == k,
                (13.0, appkit::font()),
            ) {
                self.values[UNDEFINED] = k.to_string();
            }
        }

        // OK and Cancel
        let valid = self.effect().is_some();
        let ok = appkit::button_with(
            ui,
            r(392.5, 39.0, 531.0, 65.0),
            "OK",
            (true, valid),
            13.0,
            0.0,
        );
        let cancel = appkit::button_with(
            ui,
            r(392.5, 75.0, 531.0, 101.0),
            "Cancel",
            (false, true),
            13.0,
            0.0,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        let enter = !typing && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (ok.clicked() || enter)
            && let Some(effect) = self.effect()
        {
            return Outcome::Apply(effect);
        }
        Outcome::Open
    }

    fn wave_field(&mut self, ui: &mut Ui, i: usize, rect: Rect, focus: bool) {
        let p = &self.kind.params()[i];
        appkit::field(
            ui,
            rect,
            &mut self.values[i],
            ("wave-field", i),
            (p.min, p.max),
            1.0,
            p.decimals,
            focus && self.first_frame,
        );
    }

    /// Setting `i`'s slider: a 3 pt track at `top`, its pin pointing down
    /// onto it from above (`down`) or up into it from below. Dragging sets
    /// the value.
    fn wave_track(&mut self, ui: &mut Ui, i: usize, top: f32, down: bool, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let p = &self.kind.params()[i];
        let (min, max) = (p.min, p.max);
        let line = Rect::from_min_max(at(TRACK.0, top), at(TRACK.1, top + 3.0));
        ui.painter().rect_filled(line, 0, Color32::from_gray(0x75));
        let (x0, x1) = (at(PINS.0, 0.0).x, at(PINS.1, 0.0).x);
        let hit = Rect::from_min_max(
            Pos2::new(line.left(), line.top() - pt(if down { 5.0 } else { 1.0 })),
            Pos2::new(
                line.right(),
                line.bottom() + pt(if down { 1.0 } else { 5.0 }),
            ),
        );
        let response = ui.interact(
            hit,
            ui.id().with(("wave-track", i)),
            Sense::click_and_drag(),
        );
        if (response.dragged() || response.clicked())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let t = ((pointer.x - x0) / (x1 - x0)).clamp(0.0, 1.0);
            self.set(i, (min + (max - min) * t).round());
        }
        let v = self.value(i).unwrap_or(p.default);
        let x = x0 + (x1 - x0) * ((v - min) / (max - min)).clamp(0.0, 1.0);
        let tip = if down {
            Pos2::new(x, line.top() + pt(5.0))
        } else {
            Pos2::new(x, line.top() - pt(2.5))
        };
        small_pin(ui.painter(), tip, down);
    }
}

/// Wave's slider pins: 9 pt wide and 8 pt long, white with a dark edge,
/// their tip pointing down (`down`) or up.
fn small_pin(painter: &egui::Painter, tip: Pos2, down: bool) {
    let s = if down { -1.0 } else { 1.0 };
    let p = |x: f32, y: f32| tip + vec2(pt(x), pt(y * s));
    let outline = vec![
        p(0.0, 0.0),
        p(4.5, 4.5),
        p(4.5, 8.0),
        p(-4.5, 8.0),
        p(-4.5, 4.5),
    ];
    painter.add(Shape::convex_polygon(
        outline.clone(),
        Color32::from_gray(0xf0),
        Stroke::NONE,
    ));
    painter.add(Shape::closed_line(
        outline,
        Stroke::new(pt(1.0), Color32::from_gray(0x21)),
    ));
}
