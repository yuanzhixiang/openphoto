//! View > Guides > New Guide...: Orientation (Horizontal or Vertical),
//! Position and Color, laid out at the positions measured on Photoshop
//! 2026's "New guide" dialog (390 × 188 pt, a UXP dialog). Sizes are in
//! Photoshop points from the dialog's top-left corner.

use egui::{Align2, Color32, CornerRadius, Key, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use op_core::Guide;

use super::{common, uxp};
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(390.0), pt(188.0));
/// Photoshop's guide colors, Cyan first (the default).
const COLORS: [(&str, [u8; 3]); 9] = [
    ("Cyan", [0x85, 0xfb, 0xfd]),
    ("Light Blue", [0x4a, 0x9c, 0xff]),
    ("Light Red", [0xff, 0x6e, 0x6e]),
    ("Green", [0x4a, 0xff, 0x4a]),
    ("Medium Blue", [0x31, 0x31, 0xff]),
    ("Yellow", [0xff, 0xff, 0x4a]),
    ("Magenta", [0xff, 0x4a, 0xff]),
    ("Gray", [0x80, 0x80, 0x80]),
    ("Black", [0, 0, 0]),
];

pub enum Outcome {
    Open,
    Cancel,
    Apply(Guide),
}

pub struct NewGuideDialog {
    vertical: bool,
    position: String,
    /// Edit Selected Guides...: the guide being edited (its index).
    pub editing: Option<usize>,
    /// An index into [`COLORS`], or the custom color picked with the
    /// swatch (shown as "Custom").
    color: usize,
    custom: Option<[u8; 3]>,
    /// The swatch was clicked: `lib.rs` opens the Color Picker.
    wants_picker: bool,
}

/// What New Guide remembers for next time: the orientation and color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Remembered {
    pub vertical: bool,
    pub color: usize,
    pub custom: Option<[u8; 3]>,
}

impl Default for NewGuideDialog {
    /// Photoshop starts with a horizontal guide at 0.
    fn default() -> Self {
        Self {
            vertical: false,
            position: "0 px".into(),
            editing: None,
            color: 0,
            custom: None,
            wants_picker: false,
        }
    }
}

impl NewGuideDialog {
    /// View › Guides › Edit Selected Guides...: the dialog filled in with
    /// guide `index`, titled "Edit Guide"; its OK changes that guide.
    pub fn editing(guide: Guide, index: usize) -> Self {
        let (color, custom) = match guide.color {
            None => (0, None),
            Some(c) => match COLORS.iter().position(|(_, rgb)| *rgb == c) {
                Some(k) => (k, None),
                None => (0, Some(c)),
            },
        };
        Self {
            vertical: guide.vertical,
            position: format!("{} px", (guide.position * 32.0).round() / 32.0),
            editing: Some(index),
            color,
            custom,
            wants_picker: false,
        }
    }

    /// A new guide with the last orientation and color.
    pub fn remembered(last: Remembered) -> Self {
        Self {
            vertical: last.vertical,
            color: last.color.min(COLORS.len() - 1),
            custom: last.custom,
            ..Default::default()
        }
    }

    /// The orientation and color to remember.
    pub fn remember(&self) -> Remembered {
        Remembered {
            vertical: self.vertical,
            color: self.color,
            custom: self.custom,
        }
    }

    /// The custom swatch asks for the Color Picker; its starting color, once.
    pub fn take_picker_request(&mut self) -> Option<[u8; 3]> {
        std::mem::take(&mut self.wants_picker).then(|| self.custom.unwrap_or(COLORS[self.color].1))
    }

    /// The Color Picker was confirmed for the custom color.
    pub fn set_custom(&mut self, rgb: [u8; 3]) {
        self.custom = Some(rgb);
    }

    /// The guide's color: None for Cyan (the guides' own color), else the
    /// chosen or custom color.
    fn guide_color(&self) -> Option<[u8; 3]> {
        match (self.custom, self.color) {
            (Some(c), _) => Some(c),
            (None, 0) => None,
            (None, k) => Some(COLORS[k].1),
        }
    }

    fn guide(&self) -> Option<Guide> {
        let position: f32 = self
            .position
            .trim()
            .trim_end_matches("px")
            .trim()
            .parse()
            .ok()?;
        (position.abs() <= 30_000.0).then_some(Guide {
            vertical: self.vertical,
            position,
            color: self.guide_color(),
        })
    }

    /// `active` is false while the Color Picker is open on top.
    pub fn show(&mut self, ctx: &egui::Context, active: bool) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("new-guide"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect, active);
            });
        if !active {
            return Outcome::Open;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect, active: bool) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let title = if self.editing.is_some() {
            "Edit Guide"
        } else {
            "New guide"
        };
        common::frame(ui, frame, title, theme::dialog_bold(pt(13.0)));
        ui.painter().text(
            at(20.0, 57.0),
            Align2::LEFT_CENTER,
            "Orientation",
            theme::uxp_bold(pt(12.0)),
            uxp::TEXT,
        );
        for (vertical, label, x) in [(false, "Horizontal", 26.0), (true, "Vertical", 108.0)] {
            if uxp::radio(ui, at(x, 84.0), label, self.vertical == vertical) {
                self.vertical = vertical;
            }
        }
        let right_label = |ui: &Ui, cy: f32, text: &str| {
            ui.painter().text(
                at(59.5, cy),
                Align2::RIGHT_CENTER,
                text,
                uxp::font(),
                uxp::TEXT,
            );
        };
        right_label(ui, 120.5, "Position");
        common::text_field(
            ui,
            r(68.0, 108.0, 161.0, 132.0),
            &mut self.position,
            "guide-position",
            uxp::font(),
            pt(11.5),
            false,
        );

        right_label(ui, 156.5, "Color");
        let (name, [cr, cg, cb]) = match self.custom {
            Some(c) => ("Custom", c),
            None => COLORS[self.color],
        };
        let mut chosen = if self.custom.is_some() {
            usize::MAX
        } else {
            self.color
        };
        common::ps_dropdown(
            ui,
            r(68.0, 144.0, 228.0, 168.0),
            "guide-color",
            |painter, rect| {
                let swatch = Rect::from_min_size(
                    rect.min + vec2(pt(7.0), pt(4.0)),
                    vec2(pt(16.0), pt(16.0)),
                );
                painter.rect(
                    swatch,
                    CornerRadius::same(pt(2.0) as u8),
                    Color32::from_rgb(cr, cg, cb),
                    Stroke::new(pt(1.0), Color32::from_gray(0xa9)),
                    StrokeKind::Inside,
                );
                painter.text(
                    rect.left_center() + vec2(pt(30.5), pt(0.75)),
                    Align2::LEFT_CENTER,
                    name,
                    uxp::font(),
                    uxp::TEXT,
                );
            },
            |ui| {
                for (k, (label, _)) in COLORS.iter().enumerate() {
                    ui.selectable_value(&mut chosen, k, *label);
                }
            },
        );
        if chosen < COLORS.len() && (self.custom.is_some() || chosen != self.color) {
            self.color = chosen;
            self.custom = None;
        }
        // The custom color's swatch: a click opens the Color Picker
        let swatch = r(240.0, 144.0, 288.0, 168.0);
        let [sr, sg, sb] = self.custom.unwrap_or([255, 255, 255]);
        ui.painter().rect(
            swatch,
            CornerRadius::same(pt(3.0) as u8),
            Color32::from_rgb(sr, sg, sb),
            Stroke::new(pt(1.0), Color32::from_gray(0xa9)),
            StrokeKind::Inside,
        );
        if ui
            .interact(swatch, ui.id().with("guide-custom"), Sense::click())
            .clicked()
        {
            self.wants_picker = true;
        }

        // No field has the focus: OK does, as in Photoshop
        let guide = self.guide();
        let ok = common::ps_focused_button(
            ui,
            r(300.0, 48.0, 370.0, 72.0),
            "OK",
            theme::uxp_bold(pt(12.0)),
        );
        let cancel = common::ps_button(
            ui,
            r(300.0, 84.0, 370.0, 108.0),
            "Cancel",
            false,
            true,
            true,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = active && ui.input(|i| i.key_pressed(Key::Enter));
        if (ok.clicked() || enter)
            && let Some(guide) = guide
        {
            return Outcome::Apply(guide);
        }
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_accepts_a_px_suffix() {
        let mut d = NewGuideDialog {
            position: "120 px".into(),
            vertical: true,
            ..Default::default()
        };
        assert_eq!(
            d.guide(),
            Some(Guide {
                vertical: true,
                position: 120.0,
                color: None,
            })
        );
        d.position = "abc".into();
        assert_eq!(d.guide(), None);
    }

    #[test]
    fn colors_and_memory() {
        let mut d = NewGuideDialog::default();
        // Cyan is the guides' own color
        assert_eq!(d.guide().unwrap().color, None);
        d.color = 2;
        assert_eq!(d.guide().unwrap().color, Some(COLORS[2].1));
        d.set_custom([1, 2, 3]);
        assert_eq!(d.guide().unwrap().color, Some([1, 2, 3]));
        d.vertical = true;
        let again = NewGuideDialog::remembered(d.remember());
        assert!(again.vertical);
        assert_eq!(again.guide().unwrap().color, Some([1, 2, 3]));
        // Editing a guide shows its color
        let g = Guide {
            vertical: false,
            position: 5.0,
            color: Some(COLORS[5].1),
        };
        assert_eq!(NewGuideDialog::editing(g, 0).color, 5);
    }
}
