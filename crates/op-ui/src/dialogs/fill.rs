//! Edit > Fill (Shift+F5): Contents, the Options group (Color
//! Adaptation, for Content-Aware) and the Blending group (Mode, Opacity,
//! Preserve Transparency), laid out at the positions measured on Photoshop
//! 2026's dialog (348 × 293 pt, a classic AppKit dialog). Sizes are in
//! Photoshop points from the dialog's top-left corner.

use crate::native_popup::Entry;
use egui::{Align2, Color32, Key, Rect, Sense, Ui, vec2};
use op_core::fill::FillOptions;
use op_core::{BlendMode, Color};

use super::{appkit, common};
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(348.0), pt(293.0));

/// What the dialog fills with ("Contents").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contents {
    Foreground,
    Background,
    Color,
    ContentAware,
    Pattern,
    History,
    Black,
    Gray,
    White,
}

impl Contents {
    const ALL: [Self; 9] = [
        Self::Foreground,
        Self::Background,
        Self::Color,
        Self::ContentAware,
        Self::Pattern,
        Self::History,
        Self::Black,
        Self::Gray,
        Self::White,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Foreground => "Foreground Color",
            Self::Background => "Background Color",
            Self::Color => "Color...",
            Self::ContentAware => "Content-Aware",
            Self::Pattern => "Pattern",
            Self::History => "History",
            Self::Black => "Black",
            Self::Gray => "50% Gray",
            Self::White => "White",
        }
    }

    /// Content-Aware isn't implemented.
    fn available(self) -> bool {
        self != Self::ContentAware
    }
}

/// What OK fills with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FillWith {
    Color(Color),
    /// The pattern chosen under Custom Pattern (`AppState::pattern`).
    Pattern,
    /// The History panel's source state of the layer.
    History,
}

pub enum Outcome {
    Open,
    Cancel,
    Apply {
        with: FillWith,
        options: FillOptions,
    },
}

pub struct FillDialog {
    contents: Contents,
    /// The color chosen with "Color...".
    color: Color,
    mode: BlendMode,
    opacity: String,
    preserve_transparency: bool,
    wants_color_picker: bool,
    first_frame: bool,
}

impl Default for FillDialog {
    fn default() -> Self {
        Self {
            contents: Contents::Foreground,
            color: Color::WHITE,
            mode: BlendMode::Normal,
            opacity: "100".into(),
            preserve_transparency: false,
            wants_color_picker: false,
            first_frame: true,
        }
    }
}

impl FillDialog {
    /// "Color..." asks for the Color Picker; returns its starting color once.
    pub fn take_color_picker_request(&mut self) -> Option<Color> {
        std::mem::take(&mut self.wants_color_picker).then_some(self.color)
    }

    #[cfg(test)]
    pub fn set_contents_for_test(&mut self, contents: Contents) {
        self.contents = contents;
    }

    /// The Color Picker was confirmed for "Color...".
    pub fn set_color(&mut self, color: Color) {
        self.contents = Contents::Color;
        self.color = color;
    }

    fn fill_with(&self, foreground: Color, background: Color) -> FillWith {
        match self.contents {
            Contents::Pattern => FillWith::Pattern,
            Contents::History => FillWith::History,
            _ => FillWith::Color(self.fill_color(foreground, background)),
        }
    }

    fn fill_color(&self, foreground: Color, background: Color) -> Color {
        match self.contents {
            Contents::Foreground => foreground,
            Contents::Background => background,
            Contents::Color => self.color,
            Contents::Black => Color::BLACK,
            Contents::Gray => Color::from_rgba8([128, 128, 128, 255]),
            Contents::White | Contents::ContentAware | Contents::Pattern | Contents::History => {
                Color::WHITE
            }
        }
    }

    fn opacity(&self) -> Option<f32> {
        let v: f32 = self.opacity.trim().parse().ok()?;
        (0.0..=100.0).contains(&v).then_some(v / 100.0)
    }

    /// `active` is false while the Color Picker is open on top. `patterns`
    /// are the Custom Pattern picker's patterns and `pattern` the chosen
    /// one.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        foreground: Color,
        background: Color,
        active: bool,
        (patterns, pattern): (&[crate::state::Pattern], &mut usize),
    ) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("fill-dialog"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(
                    ui,
                    rect,
                    (foreground, background),
                    active,
                    (patterns, pattern),
                );
            });
        self.first_frame = false;
        if active && ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        (foreground, background): (Color, Color),
        active: bool,
        (patterns, pattern): (&[crate::state::Pattern], &mut usize),
    ) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Fill", theme::dialog_bold(pt(13.0)));
        let label = |ui: &Ui, cy: f32, text: &str| {
            appkit::text(ui, at(112.5, cy), Align2::RIGHT_CENTER, text, appkit::TEXT);
        };

        label(ui, 48.5, "Contents:");
        let mut entries = Vec::new();
        let mut kinds = Vec::new();
        for (i, c) in Contents::ALL.into_iter().enumerate() {
            if i == 3 || i == 6 {
                entries.push(Entry::Separator);
                kinds.push(None);
            }
            entries.push(Entry::item(c.label(), c == self.contents).enabled(c.available()));
            kinds.push(Some(c));
        }
        let contents = appkit::popup(
            ui,
            r(118.0, 38.0, 263.0, 59.0),
            "fill-contents",
            self.contents.label(),
            &entries,
        )
        .and_then(|k| kinds[k])
        .unwrap_or(self.contents);
        if contents == Contents::Color && self.contents != Contents::Color {
            // "Color..." opens the Color Picker; the choice applies once a
            // color is confirmed there
            self.wants_color_picker = true;
        } else {
            self.contents = contents;
        }

        // The two group boxes, their titles breaking the top edge 5 pt
        // either side
        for (top, bottom, title) in [(116.0, 166.0, "Options"), (184.0, 282.0, "Blending")] {
            let t = appkit::text(
                ui,
                at(30.5, top - 1.5),
                Align2::LEFT_CENTER,
                title,
                appkit::TEXT,
            );
            appkit::group(
                ui.painter(),
                r(11.5, top, 262.5, bottom),
                (t.left() - pt(5.0), t.right() + pt(5.0)),
            );
        }
        if self.contents == Contents::Pattern && !patterns.is_empty() {
            // Custom Pattern: the pattern, its chevron the picker
            label(ui, 141.5, "Custom Pattern:");
            let k = (*pattern).min(patterns.len() - 1);
            let image = &patterns[k].image;
            let swatch = r(118.0, 128.5, 148.0, 155.5);
            let painter = ui.painter_at(swatch);
            let (pw, ph) = (image.width().max(1), image.height().max(1));
            let (cols, rows) = (
                (swatch.width() / pt(1.0)).ceil() as u32,
                (swatch.height() / pt(1.0)).ceil() as u32,
            );
            for y in 0..rows {
                for x in 0..cols {
                    let [r, g, b, _] = image.pixel(x % pw, y % ph);
                    let cell = Rect::from_min_size(
                        swatch.min + vec2(x as f32 * pt(1.0), y as f32 * pt(1.0)),
                        vec2(pt(1.0), pt(1.0)),
                    );
                    painter.rect_filled(cell, 0, Color32::from_rgb(r, g, b));
                }
            }
            ui.painter().rect_stroke(
                swatch,
                0,
                egui::Stroke::new(pt(1.0), appkit::FIELD_BORDER),
                egui::StrokeKind::Inside,
            );
            let chevron_rect = r(147.0, 128.5, 160.0, 155.5);
            let chevron = ui.interact(chevron_rect, ui.id().with("fill-pattern"), Sense::click());
            ui.painter().rect(
                chevron_rect,
                0,
                appkit::FIELD,
                egui::Stroke::new(pt(1.0), appkit::FIELD_BORDER),
                egui::StrokeKind::Inside,
            );
            crate::ps_icons::paint(
                ui.painter(),
                chevron_rect.center(),
                crate::ps_icons::Icon::Caret,
                Color32::from_gray(0xd0),
                appkit::FIELD,
            );
            let entries: Vec<_> = patterns
                .iter()
                .enumerate()
                .map(|(i, p)| Entry::item(p.name.clone(), i == k))
                .collect();
            if let Some(i) =
                crate::native_popup::dropdown(ui, &chevron, ui.id().with("fill-patterns"), &entries)
            {
                *pattern = i;
            }
        } else {
            // Color Adaptation only applies to Content-Aware, which isn't
            // available
            appkit::checkbox_with(
                ui,
                at(20.0, 141.5),
                (12.0, 11.0),
                "Color Adaptation",
                &mut false,
                false,
            );
        }

        label(ui, 208.5, "Mode:");
        let (entries, modes) = appkit::blend_modes(self.mode);
        if let Some(k) = appkit::popup(
            ui,
            r(118.0, 198.0, 253.0, 219.0),
            "fill-mode",
            self.mode.label(),
            &entries,
        ) {
            self.mode = modes[k].unwrap_or(self.mode);
        }

        label(ui, 236.5, "Opacity:");
        appkit::field(
            ui,
            r(117.0, 227.0, 169.0, 246.0),
            &mut self.opacity,
            "fill-opacity",
            (0.0, 100.0),
            1.0,
            0,
            self.first_frame,
        );
        appkit::text(ui, at(177.5, 236.5), Align2::LEFT_CENTER, "%", appkit::TEXT);
        appkit::checkbox_with(
            ui,
            at(20.0, 257.5),
            (12.0, 11.0),
            "Preserve Transparency",
            &mut self.preserve_transparency,
            true,
        );

        let opacity = self.opacity();
        let ok = appkit::button(
            ui,
            r(278.5, 38.5, 338.5, 64.5),
            "OK",
            true,
            opacity.is_some(),
        );
        let cancel = appkit::button(ui, r(278.5, 73.5, 338.5, 99.5), "Cancel", false, true);
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = active && ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter)
            && let Some(opacity) = opacity
        {
            return Outcome::Apply {
                with: self.fill_with(foreground, background),
                options: FillOptions {
                    mode: self.mode,
                    opacity,
                    preserve_transparency: self.preserve_transparency,
                },
            };
        }
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contents_pick_the_color() {
        let mut d = FillDialog::default();
        let (fg, bg) = (
            Color::from_rgba8([1, 2, 3, 255]),
            Color::from_rgba8([4, 5, 6, 255]),
        );
        assert_eq!(d.fill_color(fg, bg), fg);
        d.contents = Contents::Gray;
        assert_eq!(d.fill_color(fg, bg).to_rgba8(), [128, 128, 128, 255]);
        d.set_color(Color::from_rgba8([7, 8, 9, 255]));
        assert_eq!(d.fill_color(fg, bg).to_rgba8(), [7, 8, 9, 255]);
    }

    #[test]
    fn opacity_must_be_a_percentage() {
        let mut d = FillDialog::default();
        assert_eq!(d.opacity(), Some(1.0));
        d.opacity = "150".into();
        assert_eq!(d.opacity(), None);
        d.opacity = "x".into();
        assert_eq!(d.opacity(), None);
    }
}
