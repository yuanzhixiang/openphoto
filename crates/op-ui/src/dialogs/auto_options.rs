//! Auto Color Correction Options (Levels' and Curves' Options...): the
//! Auto algorithm, Snap Neutral Midtones, the shadow, midtone and
//! highlight target colors with the two clipping percentages, and Save as
//! defaults. Sizes are points from the dialog's top-left corner; the
//! layout follows Photoshop's arrangement and has not been measured
//! against Photoshop 2026 yet.

use egui::{Align2, Color32, Key, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use op_core::auto::{Algorithm, Options};

use super::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(400.0), pt(350.0));
/// The three target rows: label, y.
const TARGETS: [(&str, f32); 3] = [
    ("Shadows:", 222.0),
    ("Midtones:", 252.0),
    ("Highlights:", 282.0),
];

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

        // Algorithms
        let title = appkit::label(ui, at(28.0, 45.0), "Algorithms");
        appkit::group(
            &painter,
            r(18.0, 45.0, 300.0, 180.0),
            (title.left() - pt(4.0), title.right() + pt(4.0)),
        );
        for (k, algorithm) in Algorithm::ALL.into_iter().enumerate() {
            let y = 66.0 + k as f32 * 22.0;
            let chosen = self.options.algorithm == algorithm;
            if appkit::radio(ui, at(36.0, y), algorithm.label(), chosen) {
                self.options.algorithm = algorithm;
            }
        }
        appkit::checkbox(
            ui,
            at(29.5, 150.0),
            "Snap Neutral Midtones",
            &mut self.options.snap_neutral,
        );

        // Target Colors & Clipping
        let title = appkit::label(ui, at(28.0, 200.0), "Target Colors & Clipping");
        appkit::group(
            &painter,
            r(18.0, 200.0, 300.0, 302.0),
            (title.left() - pt(4.0), title.right() + pt(4.0)),
        );
        for (k, (label, y)) in TARGETS.into_iter().enumerate() {
            appkit::text(ui, at(100.0, y), Align2::RIGHT_CENTER, label, appkit::TEXT);
            let swatch = r(106.0, y - 10.0, 152.0, y + 10.0);
            let [cr, cg, cb] = self.target(k);
            painter.rect(
                swatch,
                0,
                Color32::from_rgb(cr, cg, cb),
                Stroke::new(pt(1.0), Color32::from_gray(0xb0)),
                StrokeKind::Inside,
            );
            if ui
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
                at(200.0, y),
                Align2::RIGHT_CENTER,
                "Clip:",
                appkit::TEXT,
            );
            let (text, id) = if k == 0 {
                (&mut self.shadow_clip, "auto-shadow-clip")
            } else {
                (&mut self.highlight_clip, "auto-highlight-clip")
            };
            let field = appkit::field(
                ui,
                r(206.0, y - 9.5, 256.0, y + 9.5),
                text,
                id,
                (0.0, 9.99),
                0.01,
                2,
                false,
            );
            appkit::text(ui, at(260.0, y), Align2::LEFT_CENTER, "%", appkit::TEXT);
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
        }

        appkit::checkbox(
            ui,
            at(18.0, 318.0),
            "Save as defaults",
            &mut self.save_defaults,
        );

        let ok = appkit::button(ui, r(312.0, 38.5, 382.0, 64.5), "OK", true, true);
        let cancel = appkit::button(ui, r(312.0, 73.5, 382.0, 99.5), "Cancel", false, true);
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
