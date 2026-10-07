//! Image > Canvas Size dialog, laid out at the positions measured on
//! Photoshop 2026's dialog (458 × 372 pt, the New Layer dialog's family):
//! sizes are in Photoshop points from the dialog's top-left corner.

use egui::{
    Align2, Color32, CornerRadius, FontId, Key, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
    pos2, vec2,
};
use op_core::{Anchor, Color, Document};

use super::common;
use crate::theme::{self, color, pt};

/// Photoshop's maximum canvas dimension for regular documents.
pub const MAX_DIMENSION: u32 = 30_000;

const SIZE: Vec2 = vec2(pt(458.0), pt(372.0));
const FONT: f32 = pt(12.0);
const TEXT: Color32 = Color32::from_gray(0xf1);
const TEXT_OFF: Color32 = Color32::from_gray(0x8e);
/// The section rules and the anchor grid, measured.
const RULE: Color32 = Color32::from_gray(0x73);
const ANCHOR_LINE: Color32 = Color32::from_gray(0x78);
/// The rules end here; the anchor grid's cells are 23 pt.
const RULE_END: f32 = 356.0;
const ANCHOR_CELL: f32 = 23.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Pixels,
    Percent,
    Inches,
    Centimeters,
    Millimeters,
    Points,
    Picas,
}

impl Unit {
    const ALL: [Self; 7] = [
        Self::Pixels,
        Self::Percent,
        Self::Inches,
        Self::Centimeters,
        Self::Millimeters,
        Self::Points,
        Self::Picas,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Pixels => "Pixels",
            Self::Percent => "Percent",
            Self::Inches => "Inches",
            Self::Centimeters => "Centimeters",
            Self::Millimeters => "Millimeters",
            Self::Points => "Points",
            Self::Picas => "Picas",
        }
    }

    /// Pixels per unit. Percent is relative to the original dimension.
    fn pixels_per_unit(self, original: u32, resolution: f64) -> f64 {
        match self {
            Self::Pixels => 1.0,
            Self::Percent => original as f64 / 100.0,
            Self::Inches => resolution,
            Self::Centimeters => resolution / 2.54,
            Self::Millimeters => resolution / 25.4,
            Self::Points => resolution / 72.0,
            Self::Picas => resolution / 6.0,
        }
    }

    fn format(self, value: f64) -> String {
        let decimals = match self {
            Self::Pixels => 0,
            Self::Percent => 2,
            _ => 3,
        };
        let s = format!("{value:.decimals$}");
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            s
        }
    }
}

/// What fills the area added around the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extension {
    Foreground,
    Background,
    White,
    Black,
    Gray,
    Other,
}

impl Extension {
    const ALL: [Self; 6] = [
        Self::Foreground,
        Self::Background,
        Self::White,
        Self::Black,
        Self::Gray,
        Self::Other,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Foreground => "Foreground",
            Self::Background => "Background",
            Self::White => "White",
            Self::Black => "Black",
            Self::Gray => "Gray",
            Self::Other => "Other...",
        }
    }
}

/// One dimension field (width or height) with its unit.
struct Dimension {
    text: String,
    unit: Unit,
    original: u32,
}

impl Dimension {
    fn new(original: u32) -> Self {
        Self {
            text: original.to_string(),
            unit: Unit::Pixels,
            original,
        }
    }

    /// The typed value in pixels (a delta when `relative`), if it parses.
    fn typed_pixels(&self, resolution: f64) -> Option<f64> {
        let v: f64 = self.text.trim().parse().ok()?;
        Some(v * self.unit.pixels_per_unit(self.original, resolution))
    }

    /// The resulting dimension in pixels, if valid.
    fn pixels(&self, relative: bool, resolution: f64) -> Option<u32> {
        let typed = self.typed_pixels(resolution)?;
        let px = if relative {
            self.original as f64 + typed
        } else {
            typed
        }
        .round();
        (px >= 1.0 && px <= MAX_DIMENSION as f64).then_some(px as u32)
    }

    fn show_pixels(&mut self, px: f64, resolution: f64) {
        let per_unit = self.unit.pixels_per_unit(self.original, resolution);
        self.text = self.unit.format(px / per_unit);
    }

    /// Changes the unit, keeping the value the field represents.
    fn set_unit(&mut self, unit: Unit, resolution: f64) {
        let px = self.typed_pixels(resolution);
        self.unit = unit;
        if let Some(px) = px {
            self.show_pixels(px, resolution);
        }
    }

    /// Switches between absolute and relative entry, keeping the result.
    fn set_relative(&mut self, relative: bool, resolution: f64) {
        let Some(typed) = self.typed_pixels(resolution) else {
            return;
        };
        let original = self.original as f64;
        let px = if relative {
            typed - original
        } else {
            original + typed
        };
        self.show_pixels(px, resolution);
    }
}

pub enum Outcome {
    Open,
    Cancel,
    Apply {
        width: u32,
        height: u32,
        anchor: Anchor,
        fill: Color,
    },
}

pub struct CanvasSizeDialog {
    width: Dimension,
    height: Dimension,
    resolution: f64,
    /// Bytes per pixel used for the "Current Size: 1.70M" readout.
    bytes_per_pixel: u64,
    relative: bool,
    anchor: Anchor,
    extension: Extension,
    other: Color,
    /// Set when "Other..." or the swatch asks for the Color Picker.
    wants_color_picker: bool,
    /// The extension color only applies to a background layer.
    has_background: bool,
    first_frame: bool,
}

impl CanvasSizeDialog {
    pub fn new(doc: &Document) -> Self {
        Self {
            width: Dimension::new(doc.width),
            height: Dimension::new(doc.height),
            resolution: doc.resolution as f64,
            // Photoshop counts the color channels only (RGB, 8 bits each)
            bytes_per_pixel: 3 * doc.bit_depth.bits() as u64 / 8,
            relative: false,
            anchor: Anchor::CENTER,
            extension: Extension::Background,
            other: Color::WHITE,
            wants_color_picker: false,
            has_background: doc.has_background(),
            first_frame: true,
        }
    }

    fn new_size(&self) -> Option<(u32, u32)> {
        Some((
            self.width.pixels(self.relative, self.resolution)?,
            self.height.pixels(self.relative, self.resolution)?,
        ))
    }

    fn extension_color(&self, foreground: Color, background: Color) -> Color {
        match self.extension {
            Extension::Foreground => foreground,
            Extension::Background => background,
            Extension::White => Color::WHITE,
            Extension::Black => Color::BLACK,
            Extension::Gray => Color::from_rgba8([128, 128, 128, 255]),
            Extension::Other => self.other,
        }
    }

    /// The Color Picker the dialog asked for, with the color to start from.
    /// Returns it once per request.
    pub fn take_color_picker_request(
        &mut self,
        foreground: Color,
        background: Color,
    ) -> Option<Color> {
        std::mem::take(&mut self.wants_color_picker)
            .then(|| self.extension_color(foreground, background))
    }

    /// The Color Picker was confirmed: the extension becomes "Other..." with
    /// that color. (Cancelling leaves the previous choice in place.)
    pub fn set_other_color(&mut self, color: Color) {
        self.extension = Extension::Other;
        self.other = color;
    }

    /// `active` is false while a dialog on top (the Color Picker) has the
    /// keyboard, so Enter and Esc go to that dialog only.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        foreground: Color,
        background: Color,
        active: bool,
    ) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("canvas-size"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect, foreground, background, active);
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
        foreground: Color,
        background: Color,
        active: bool,
    ) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let font = theme::uxp(FONT);
        let bold = theme::uxp_bold(FONT);
        let painter = ui.painter().clone();
        common::frame(ui, frame, "Canvas Size", theme::dialog_bold(pt(13.0)));
        let label = |right: f32, cy: f32, text: &str| {
            painter.text(
                at(right, cy),
                Align2::RIGHT_CENTER,
                text,
                font.clone(),
                TEXT,
            );
        };
        let new_size = self.new_size();

        // Current size: the readout with a rule to the right, then the
        // original width and height
        section(
            &painter,
            at(20.0, 57.5),
            &format!(
                "Current Size: {}",
                self.size_label(self.width.original, self.height.original)
            ),
            &bold,
            at(RULE_END, 0.0).x,
        );
        for (text, value, cy) in [
            ("Width", self.width.original, 81.0),
            ("Height", self.height.original, 102.5),
        ] {
            label(55.0, cy, text);
            painter.text(
                at(64.0, cy),
                Align2::LEFT_CENTER,
                format!("{value} px"),
                font.clone(),
                TEXT,
            );
        }

        let new_label = match new_size {
            Some((w, h)) => self.size_label(w, h),
            None => "—".into(),
        };
        section(
            &painter,
            at(20.0, 130.5),
            &format!("New Size: {new_label}"),
            &bold,
            at(RULE_END, 0.0).x,
        );

        let resolution = self.resolution;
        let first_frame = self.first_frame;
        for (i, (text, y0)) in [("Width", 145.0), ("Height", 174.0)]
            .into_iter()
            .enumerate()
        {
            label(55.0, y0 + 13.0, text);
            let dim = if i == 0 {
                &mut self.width
            } else {
                &mut self.height
            };
            common::text_field(
                ui,
                r(64.0, y0, 145.0, y0 + 24.0),
                &mut dim.text,
                ("canvas-size-field", i),
                font.clone(),
                pt(10.5),
                first_frame && i == 0,
            );
            let mut unit = dim.unit;
            let shown = unit.label();
            common::ps_dropdown(
                ui,
                r(153.0, y0, 313.0, y0 + 24.0),
                if i == 0 {
                    "canvas-size-unit-w"
                } else {
                    "canvas-size-unit-h"
                },
                |painter, rect| {
                    painter.text(
                        rect.left_center() + vec2(pt(9.0), pt(0.75)),
                        Align2::LEFT_CENTER,
                        shown,
                        theme::uxp(FONT),
                        TEXT,
                    );
                },
                |ui| {
                    for u in Unit::ALL {
                        ui.selectable_value(&mut unit, u, u.label());
                    }
                },
            );
            if unit != dim.unit {
                dim.set_unit(unit, resolution);
            }
        }

        let mut relative = self.relative;
        common::ps_checkbox(
            ui,
            at(64.0, 216.0),
            "Relative to current dimension",
            &mut relative,
            true,
        );
        if relative != self.relative {
            self.width.set_relative(relative, resolution);
            self.height.set_relative(relative, resolution);
            self.relative = relative;
        }

        // Anchor: its label at the left, the 3 × 3 grid under the fields
        painter.text(
            at(20.0, 255.0),
            Align2::LEFT_CENTER,
            "Anchor",
            font.clone(),
            TEXT,
        );
        let grid = Rect::from_min_size(at(64.0, 246.0), Vec2::splat(pt(ANCHOR_CELL * 3.0 + 1.0)));
        let growth = match new_size {
            Some((w, h)) => (
                (w as i64 - self.width.original as i64).signum(),
                (h as i64 - self.height.original as i64).signum(),
            ),
            None => (1, 1),
        };
        self.anchor_grid(ui, grid, growth);

        // Canvas extension color: only for a document with a background
        let on = self.has_background;
        painter.text(
            at(20.5, 341.0),
            Align2::LEFT_CENTER,
            "Canvas extension color",
            font.clone(),
            if on { TEXT } else { TEXT_OFF },
        );
        let ext_rect = r(140.0, 328.0, 300.0, 352.0);
        let shown = self.extension.label();
        let content = |painter: &egui::Painter, rect: Rect, ink: Color32| {
            painter.text(
                rect.left_center() + vec2(pt(9.0), pt(0.75)),
                Align2::LEFT_CENTER,
                shown,
                theme::uxp(FONT),
                ink,
            );
        };
        if on {
            let mut extension = self.extension;
            common::ps_dropdown(
                ui,
                ext_rect,
                "canvas-size-extension",
                |painter, rect| content(painter, rect, TEXT),
                |ui| {
                    for (i, e) in Extension::ALL.into_iter().enumerate() {
                        if i == 5 {
                            ui.separator();
                        }
                        ui.selectable_value(&mut extension, e, e.label());
                    }
                },
            );
            if extension == Extension::Other {
                // "Other..." opens the Color Picker; the choice only changes
                // once a color is confirmed there, as in Photoshop
                self.wants_color_picker = true;
            } else {
                self.extension = extension;
            }
        } else {
            // Disabled: a flat #5c5c5c box, dimmed text and chevron
            let fill = Color32::from_gray(0x5c);
            painter.rect_filled(ext_rect, CornerRadius::same(pt(3.0) as u8), fill);
            content(&painter, ext_rect, TEXT_OFF);
            crate::ps_icons::paint(
                &painter,
                Pos2::new(ext_rect.right() - pt(13.5), ext_rect.center().y),
                crate::ps_icons::Icon::DialogChevron,
                TEXT_OFF,
                fill,
            );
        }
        let swatch = r(308.0, 328.0, 356.0, 352.0);
        let [cr, cg, cb, _] = self.extension_color(foreground, background).to_rgba8();
        painter.rect(
            swatch,
            CornerRadius::same(pt(3.0) as u8),
            Color32::from_rgb(cr, cg, cb),
            Stroke::new(pt(1.0), Color32::from_gray(0x8e)),
            StrokeKind::Inside,
        );
        if on
            && ui
                .interact(swatch, ui.id().with("canvas-size-swatch"), Sense::click())
                .clicked()
        {
            self.wants_color_picker = true;
        }

        let ok = common::ps_button(
            ui,
            r(368.0, 48.0, 438.0, 72.0),
            "OK",
            true,
            new_size.is_some(),
            true,
        );
        let cancel = common::ps_button(
            ui,
            r(368.0, 84.0, 438.0, 108.0),
            "Cancel",
            false,
            true,
            true,
        );
        let enter = active && ui.input(|i| i.key_pressed(Key::Enter));
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        if (ok.clicked() || enter)
            && let Some((width, height)) = new_size
        {
            return Outcome::Apply {
                width,
                height,
                anchor: self.anchor,
                fill: self.extension_color(foreground, background),
            };
        }
        Outcome::Open
    }

    /// "1.70M" style size readout, as in Photoshop.
    fn size_label(&self, width: u32, height: u32) -> String {
        let bytes = width as f64 * height as f64 * self.bytes_per_pixel as f64;
        const K: f64 = 1024.0;
        if bytes >= K * K * K {
            format!("{:.2}G", bytes / (K * K * K))
        } else if bytes >= K * K {
            format!("{:.2}M", bytes / (K * K))
        } else {
            format!("{:.1}K", bytes / K)
        }
    }

    /// 3×3 anchor picker (23 pt cells in 1 pt lines). Arrows point away
    /// from the anchor when the canvas grows along that axis and toward it
    /// when it shrinks.
    fn anchor_grid(&mut self, ui: &mut Ui, grid: Rect, growth: (i64, i64)) {
        let cell = pt(ANCHOR_CELL);
        let line = |painter: &egui::Painter, r: Rect| painter.rect_filled(r, 0, ANCHOR_LINE);
        for i in 0..4 {
            let d = i as f32 * cell;
            line(
                ui.painter(),
                Rect::from_min_size(grid.min + vec2(d, 0.0), vec2(pt(1.0), grid.height())),
            );
            line(
                ui.painter(),
                Rect::from_min_size(grid.min + vec2(0.0, d), vec2(grid.width(), pt(1.0))),
            );
        }
        for cy in 0..3u8 {
            for cx in 0..3u8 {
                let rect = Rect::from_min_size(
                    grid.min + vec2(cx as f32 * cell + pt(1.0), cy as f32 * cell + pt(1.0)),
                    Vec2::splat(cell - pt(1.0)),
                );
                let response = ui.interact(rect, ui.id().with(("anchor", cx, cy)), Sense::click());
                if response.clicked() {
                    self.anchor = Anchor { x: cx, y: cy };
                }
                if response.hovered() {
                    ui.painter().rect_filled(rect, 0, color::HOVER);
                }
                let dx = cx as i64 - self.anchor.x as i64;
                let dy = cy as i64 - self.anchor.y as i64;
                let c = rect.center();
                if dx == 0 && dy == 0 {
                    ui.painter().circle_filled(c, pt(3.5), TEXT);
                } else if dx.abs() <= 1 && dy.abs() <= 1 {
                    let sx = if growth.0 < 0 { -1 } else { 1 };
                    let sy = if growth.1 < 0 { -1 } else { 1 };
                    let dir = vec2((dx * sx) as f32, (dy * sy) as f32).normalized();
                    arrow(ui.painter(), c, dir);
                }
            }
        }
    }
}

/// A bold section heading at `left_center` with a 1 pt rule from 8 pt
/// after it to `rule_end`.
fn section(painter: &egui::Painter, left_center: Pos2, text: &str, font: &FontId, rule_end: f32) {
    let r = painter.text(left_center, Align2::LEFT_CENTER, text, font.clone(), TEXT);
    painter.rect_filled(
        Rect::from_min_max(
            pos2(r.right() + pt(8.0), left_center.y),
            pos2(rule_end, left_center.y + pt(1.0)),
        ),
        0,
        RULE,
    );
}

/// An arrow centered on `c`, pointing along `dir`, measured: 13.25 pt
/// long, a 7.75 pt head 7.5 pt wide and a 2 pt shaft.
fn arrow(painter: &egui::Painter, c: Pos2, dir: Vec2) {
    let tip = c + dir * pt(6.5);
    let base = c - dir * pt(1.25);
    let tail = c - dir * pt(6.75);
    painter.line_segment([tail, base], Stroke::new(pt(2.0), TEXT));
    let side = vec2(-dir.y, dir.x) * pt(3.75);
    painter.add(egui::Shape::convex_polygon(
        vec![tip, base + side, base - side],
        TEXT,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_round_trip() {
        let mut d = Dimension::new(734);
        d.set_unit(Unit::Inches, 96.0);
        assert_eq!(d.text, "7.646");
        d.set_unit(Unit::Percent, 96.0);
        assert_eq!(d.text, "100");
        d.set_unit(Unit::Pixels, 96.0);
        assert_eq!(d.text, "734");
        assert_eq!(d.pixels(false, 96.0), Some(734));
    }

    #[test]
    fn relative_entry() {
        let mut d = Dimension::new(734);
        d.set_relative(true, 96.0);
        assert_eq!(d.text, "0");
        d.text = "266".into();
        assert_eq!(d.pixels(true, 96.0), Some(1000));
        d.set_relative(false, 96.0);
        assert_eq!(d.text, "1000");
    }

    #[test]
    fn rejects_out_of_range() {
        let mut d = Dimension::new(10);
        d.text = "0".into();
        assert_eq!(d.pixels(false, 72.0), None);
        d.text = "abc".into();
        assert_eq!(d.pixels(false, 72.0), None);
        d.text = (MAX_DIMENSION + 1).to_string();
        assert_eq!(d.pixels(false, 72.0), None);
    }
}
