//! Auto Color Correction Options (Levels' and Curves' Options...): the
//! Auto algorithm, Snap Neutral Midtones, the shadow, midtone and
//! highlight target colors with the two clipping percentages, and Save as
//! defaults. Sizes are points from the dialog's top-left corner, measured
//! from 2x captures of Photoshop 2026's dialog (346 × 385 pt).

use egui::{Align2, Color32, Key, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use op_core::auto::{Algorithm, Options};

use super::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(346.0), pt(385.0));
/// The three target rows: label, center line.
const TARGETS: [(&str, f32); 3] = [
    ("Shadows:", 257.25),
    ("Midtones:", 288.25),
    ("Highlights:", 319.25),
];
/// The target rows' labels (dimmer than the radio labels).
const TARGET_TEXT: Color32 = Color32::from_gray(0xd6);

pub enum Outcome {
    Open,
    Cancel,
    /// The options, and whether they become the defaults.
    Ok(Options, bool),
}

#[derive(Clone)]
pub struct AutoOptionsDialog {
    pub options: Options,
    pub save_defaults: bool,
    shadow_clip: String,
    highlight_clip: String,
    /// A target swatch was clicked (0 shadows, 1 midtones, 2 highlights):
    /// the app opens the Color Picker (`take_color_request`).
    wants_color: Option<usize>,
    first_frame: bool,
}

fn clip_text(v: f32) -> String {
    format!("{v:.2}")
}

impl AutoOptionsDialog {
    pub fn new(options: Options) -> Self {
        Self {
            shadow_clip: clip_text(options.targets.shadow_clip),
            highlight_clip: clip_text(options.targets.highlight_clip),
            options,
            save_defaults: false,
            wants_color: None,
            first_frame: true,
        }
    }

    /// The Color Picker's request: which target and its color.
    pub fn take_color_request(&mut self) -> Option<(usize, [u8; 3])> {
        let k = self.wants_color.take()?;
        Some((k, self.target(k)))
    }

    fn target(&self, k: usize) -> [u8; 3] {
        let t = &self.options.targets;
        [t.shadows, t.midtones, t.highlights][k]
    }

    /// The Color Picker's OK for target `k`.
    pub fn set_target(&mut self, k: usize, rgb: [u8; 3]) {
        let t = &mut self.options.targets;
        match k {
            0 => t.shadows = rgb,
            1 => t.midtones = rgb,
            _ => t.highlights = rgb,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, active: bool) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("auto-color-options"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        self.first_frame = false;
        if !active {
            return Outcome::Open;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(
            ui,
            frame,
            "Auto Color Correction Options",
            theme::dialog_bold(pt(13.0)),
        );
        let painter = ui.painter().clone();
        // A group box, its title at x 30.5 breaking the top edge (6 pt
        // before the title, 5 after)
        let group = |ui: &Ui, rect: Rect, title: &str| {
            let t = appkit::label(ui, egui::Pos2::new(at(30.5, 0.0).x, rect.top()), title);
            appkit::group(&painter, rect, (t.left() - pt(6.0), t.right() + pt(5.0)));
        };

        // Algorithms: four radios 27 pt apart, a rule, Snap Neutral Midtones
        group(ui, r(10.75, 46.75, 259.75, 206.75), "Algorithms");
        for (k, algorithm) in Algorithm::ALL.into_iter().enumerate() {
            let y = 71.25 + k as f32 * 27.0;
            let chosen = self.options.algorithm == algorithm;
            if appkit::radio_with(
                ui,
                at(27.25, y),
                algorithm.label(),
                chosen,
                (16.25, appkit::font()),
            ) {
                self.options.algorithm = algorithm;
            }
        }
        painter.rect_filled(r(20.0, 169.25, 251.0, 170.25), 0, Color32::from_gray(0x3e));
        // Enhance Brightness and Contrast has no options of its own
        let options_on = self.options.algorithm != Algorithm::BrightnessContrast;
        appkit::checkbox_with(
            ui,
            at(20.0, 182.0),
            (12.5, 10.0),
            "Snap Neutral Midtones",
            &mut self.options.snap_neutral,
            options_on,
        );

        // Target Colors & Clipping
        group(
            ui,
            r(10.75, 231.75, 259.75, 339.75),
            "Target Colors & Clipping",
        );
        let ink = |color: Color32| if options_on { color } else { appkit::TEXT_OFF };
        for (k, (label, y)) in TARGETS.into_iter().enumerate() {
            appkit::text(
                ui,
                at(81.0, y),
                Align2::RIGHT_CENTER,
                label,
                ink(TARGET_TEXT),
            );
            let swatch = r(85.5, y - 11.25, 128.0, y + 11.25);
            let [cr, cg, cb] = self.target(k);
            // Off, the swatch is an empty frame
            let fill = if options_on {
                Color32::from_rgb(cr, cg, cb)
            } else {
                Color32::TRANSPARENT
            };
            painter.rect(
                swatch,
                0,
                fill,
                Stroke::new(pt(1.0), Color32::from_gray(0x36)),
                StrokeKind::Inside,
            );
            if options_on
                && ui
                    .interact(swatch, ui.id().with(("auto-target", k)), Sense::click())
                    .clicked()
            {
                self.wants_color = Some(k);
            }
            if k == 1 {
                continue;
            }
            appkit::text(
                ui,
                at(178.0, y),
                Align2::RIGHT_CENTER,
                "Clip:",
                ink(TARGET_TEXT),
            );
            let (text, id) = if k == 0 {
                (&mut self.shadow_clip, "auto-shadow-clip")
            } else {
                (&mut self.highlight_clip, "auto-highlight-clip")
            };
            let rect = r(182.0, y - 9.25, 236.0, y + 9.25);
            if options_on {
                let field = appkit::field(ui, rect, text, id, (0.0, 9.99), 0.01, 2, false);
                if field.changed()
                    && let Ok(v) = text.trim().parse::<f32>()
                {
                    let v = v.clamp(0.0, 9.99);
                    if k == 0 {
                        self.options.targets.shadow_clip = v;
                    } else {
                        self.options.targets.highlight_clip = v;
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
                appkit::text(
                    ui,
                    rect.left_center() + vec2(pt(4.0), 0.0),
                    Align2::LEFT_CENTER,
                    text,
                    appkit::TEXT_OFF,
                );
            }
            appkit::text(ui, at(239.5, y), Align2::LEFT_CENTER, "%", ink(TARGET_TEXT));
        }

        appkit::checkbox_with(
            ui,
            at(10.0, 359.0),
            (12.5, 11.0),
            "Save as defaults",
            &mut self.save_defaults,
            true,
        );

        let ok = appkit::button(ui, r(276.0, 45.0, 336.5, 71.0), "OK", true, true);
        let cancel = appkit::button(ui, r(276.0, 80.0, 336.5, 106.0), "Cancel", false, true);
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        let enter = !typing && ui.input(|i| i.key_pressed(Key::Enter));
        if ok.clicked() || enter {
            return Outcome::Ok(self.options, self.save_defaults);
        }
        Outcome::Open
    }
}
