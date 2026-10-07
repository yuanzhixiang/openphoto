//! Filter › Blur › Shape Blur..., as Photoshop 2026 lays it out (359 × 485
//! pt, measured from a 2x capture): the preview at the top left, OK,
//! Cancel and Preview beside it, the chosen shape's swatch under
//! "Shape:", Radius with its slider, and the list of shapes.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};
use op_core::more_filters::BlurShape;

use super::filter_layout::Scale;
use super::legacy::Row;
use super::{AdjustDialog, Outcome};
use crate::dialogs::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(359.0), pt(485.0));

/// The settings' order (`Kind::params`).
pub const RADIUS: usize = 0;
pub const SHAPE: usize = 1;

/// The list of shapes: its box, and its rows' height.
const LIST: [f32; 4] = [10.5, 344.0, 316.5, 473.0];
const ROW_H: f32 = 29.0;

/// A shape white on black, `n` pixels square.
fn shape_image(shape: BlurShape, n: usize) -> egui::ColorImage {
    let pixels = (0..n * n)
        .map(|i| {
            let (x, y) = (i % n, i / n);
            let u = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            if shape.contains(u / 0.9, v / 0.9) {
                Color32::WHITE
            } else {
                Color32::BLACK
            }
        })
        .collect();
    egui::ColorImage::new([n, n], pixels)
}

impl AdjustDialog {
    pub(super) fn shape_blur_ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Shape Blur", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        self.legacy_preview(ui, frame, 261.0);
        let buttons = self.legacy_buttons(ui, frame, 267.0);
        self.legacy_preview_check(ui, frame, (268.0, 118.5));
        if self.extra.shape_thumbs.is_empty() {
            self.extra.shape_thumbs = BlurShape::ALL
                .iter()
                .map(|&s| {
                    ui.ctx().load_texture(
                        format!("shape-blur-{}", s.label()),
                        shape_image(s, 96),
                        egui::TextureOptions::LINEAR,
                    )
                })
                .collect();
        }
        let chosen = (self.value(SHAPE).unwrap_or(0.0) as usize).min(BlurShape::ALL.len() - 1);
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));

        // The chosen shape
        appkit::text(
            ui,
            at(268.0, 172.0),
            Align2::LEFT_CENTER,
            "Shape:",
            appkit::TEXT,
        );
        let swatch = r(268.5, 196.5, 330.0, 231.5);
        painter.rect_filled(swatch, 0, Color32::BLACK);
        let side = swatch.height();
        painter.image(
            self.extra.shape_thumbs[chosen].id(),
            Rect::from_center_size(swatch.center(), vec2(side, side)),
            uv,
            Color32::WHITE,
        );

        // Radius
        let row = Row {
            label: "Radius:",
            label_right: 0.0,
            field: [65.5, 288.0, 125.0, 305.5],
            unit: Some(("Pixels", 129.5)),
            track: (20.5, 207.0, 312.5),
            scale: Scale::Linear,
        };
        // Its label is left-aligned, as in Photoshop
        appkit::text(
            ui,
            at(19.5, 296.75),
            Align2::LEFT_CENTER,
            "Radius:",
            appkit::TEXT,
        );
        self.legacy_row(ui, frame, RADIUS, &Row { label: "", ..row }, true, true);

        // The shapes: a row each, the chosen one highlighted
        let list = r(LIST[0], LIST[1], LIST[2], LIST[3]);
        painter.rect(
            list,
            0,
            Color32::from_gray(0x3f),
            Stroke::new(pt(1.0), Color32::from_gray(0x36)),
            egui::StrokeKind::Inside,
        );
        for (k, shape) in BlurShape::ALL.iter().enumerate() {
            let top = LIST[1] + 1.0 + k as f32 * ROW_H;
            if top + ROW_H > LIST[3] {
                break;
            }
            let rect = r(LIST[0] + 1.0, top, LIST[2] - 17.0, top + ROW_H - 1.0);
            let response = ui.interact(rect, ui.id().with(("shape-row", k)), Sense::click());
            let fill = if k == chosen {
                Color32::from_gray(0x5c)
            } else if response.hovered() {
                Color32::from_gray(0x4c)
            } else {
                Color32::from_gray(0x46)
            };
            painter.rect_filled(rect, 0, fill);
            let thumb = Rect::from_center_size(
                at(LIST[0] + 22.0, top + ROW_H / 2.0 - 0.5),
                vec2(pt(18.0), pt(18.0)),
            );
            painter.rect_filled(thumb, pt(2.0), Color32::BLACK);
            painter.image(self.extra.shape_thumbs[k].id(), thumb, uv, Color32::WHITE);
            appkit::text(
                ui,
                at(LIST[0] + 43.5, top + ROW_H / 2.0 - 0.5),
                Align2::LEFT_CENTER,
                shape.label(),
                appkit::TEXT,
            );
            if response.clicked() {
                self.values[SHAPE] = k.to_string();
            }
        }
        // The scroll gutter at the list's right
        painter.rect_filled(
            r(LIST[2] - 16.5, LIST[1] + 1.0, LIST[2] - 1.0, LIST[3] - 1.0),
            0,
            Color32::from_gray(0x3a),
        );
        self.legacy_outcome(ui, buttons)
    }
}
