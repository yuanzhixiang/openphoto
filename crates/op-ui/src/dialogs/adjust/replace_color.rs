//! Image › Adjustments › Replace Color..., as Photoshop 2026 lays it out
//! (436 × 474 pt, measured from a 2x capture): the eyedropper, plus and
//! minus tools, Localized Color Clusters, Color, Fuzziness, the selection
//! preview with Selection / Image, then Hue, Saturation, Lightness and the
//! Result swatch; OK, Cancel, Load..., Save... and Preview on the right.

use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Stroke, StrokeKind, vec2};

use super::filter_layout::Scale;
use super::{AdjustDialog, Outcome};
use crate::dialogs::{appkit, common, preset_files};
use crate::ps_icons::{self, Icon};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(436.0), pt(474.0));

/// The settings' order (`Kind::params`).
pub const FUZZINESS: usize = 0;
pub const LOCALIZED: usize = 4;

/// The preview area (the picture fitted and centered in it).
const PREVIEW: [f32; 4] = [10.0, 124.0, 331.0, 328.0];

impl AdjustDialog {
    pub(super) fn replace_color_ui(&mut self, ui: &mut egui::Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Replace Color", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();

        // The eyedropper, add and subtract tools, 30 pt apart; the chosen
        // one in a dark box
        for (k, icon) in [Icon::Eyedropper, Icon::EyedropperPlus, Icon::EyedropperMinus]
            .into_iter()
            .enumerate()
        {
            let center = at(24.75 + 30.0 * k as f32, 51.0);
            let rect = Rect::from_center_size(center, vec2(pt(29.5), pt(25.5)));
            if ui
                .interact(rect, ui.id().with(("rc-tool", k)), egui::Sense::click())
                .clicked()
            {
                self.extra.rc_tool = k;
            }
            let chosen = self.extra.rc_tool == k;
            if chosen {
                painter.rect_filled(rect, CornerRadius::same(pt(3.0) as u8), Color32::from_gray(0x38));
            }
            let fill = if chosen {
                Color32::from_gray(0x38)
            } else {
                theme::color::PANEL
            };
            ps_icons::paint(&painter, center, icon, Color32::from_gray(0xdd), fill);
        }

        // Localized Color Clusters
        let mut localized = self.value(LOCALIZED) == Some(1.0);
        appkit::checkbox_with(
            ui,
            at(10.0, 76.5),
            (12.0, 11.0),
            "Localized Color Clusters",
            &mut localized,
            true,
        );
        self.values[LOCALIZED] = (localized as u8).to_string();

        // Color: the last color picked
        appkit::text(ui, at(281.0, 64.25), Align2::RIGHT_CENTER, "Color:", appkit::TEXT);
        let [cr, cg, cb] = self.extra.replace.shown();
        painter.rect(
            r(290.5, 44.5, 330.0, 84.0),
            0,
            Color32::from_rgb(cr, cg, cb),
            Stroke::new(pt(1.0), Color32::from_gray(0x36)),
            StrokeKind::Inside,
        );

        // Fuzziness, the label at the left edge
        self.rc_row(ui, frame, FUZZINESS, "Fuzziness:", (287.0, 108.5), (89.5, 267.5, 103.0), true);

        // The selection preview and Selection / Image
        self.replace_preview(ui, r(PREVIEW[0], PREVIEW[1], PREVIEW[2], PREVIEW[3]));
        for (k, (label, x)) in [("Selection", 92.25), ("Image", 189.75)].into_iter().enumerate() {
            let chosen = self.extra.show_image == (k == 1);
            if appkit::radio_with(ui, at(x, 354.0), label, chosen, (17.5, appkit::font())) {
                self.extra.show_image = k == 1;
            }
        }
        painter.rect_filled(r(10.0, 381.0, 330.0, 382.0), 0, Color32::from_gray(0x3e));

        // Hue, Saturation, Lightness
        for (k, (label, y)) in [("Hue:", 400.5), ("Saturation:", 427.5), ("Lightness:", 454.5)]
            .into_iter()
            .enumerate()
        {
            self.rc_row(ui, frame, 1 + k, label, (239.0, y), (92.0, 219.5, y - 5.5), false);
        }
        // Result: the color picked as replaced
        let result = self
            .effect()
            .and_then(|e| match e {
                super::Effect::Adjustment(op_core::adjust::Adjustment::ReplaceColor { shift, .. }) => {
                    Some(shift)
                }
                _ => None,
            })
            .map_or([cr, cg, cb], |[h, s, l]| {
                let px = op_core::adjust::HueSaturation::master(h, s, l).apply([cr, cg, cb, 255]);
                [px[0], px[1], px[2]]
            });
        painter.rect(
            r(290.5, 396.0, 330.0, 435.5),
            0,
            Color32::from_rgb(result[0], result[1], result[2]),
            Stroke::new(pt(1.0), Color32::from_gray(0x36)),
            StrokeKind::Inside,
        );
        appkit::text(ui, at(310.5, 451.25), Align2::CENTER_CENTER, "Result", appkit::TEXT);

        // OK, Cancel, Load..., Save... and Preview
        let button = |ui: &mut egui::Ui, y: f32, label: &str, default: bool, enabled: bool| {
            appkit::button(ui, r(346.0, y, 426.5, y + 26.0), label, default, enabled)
        };
        let valid = self.effect().is_some();
        let ok = button(ui, 38.5, "OK", true, valid).clicked();
        let cancel = button(ui, 73.5, "Cancel", false, true).clicked();
        if button(ui, 108.5, "Load...", false, true).clicked()
            && let Some((_, bytes)) = preset_files::REPLACE_COLOR.load()
            && let Some((fuzz, shift)) = preset_files::decode_replace_color(&bytes)
        {
            self.values[FUZZINESS] = fuzz.to_string();
            for (k, v) in shift.into_iter().enumerate() {
                self.values[1 + k] = v.to_string();
            }
        }
        if button(ui, 143.5, "Save...", false, valid).clicked()
            && let (Some(fuzz), Some(h), Some(s), Some(l)) =
                (self.value(0), self.value(1), self.value(2), self.value(3))
        {
            let bytes = preset_files::encode_replace_color(
                fuzz as i32,
                [h as i32, s as i32, l as i32],
            );
            preset_files::REPLACE_COLOR.save(&bytes);
        }
        appkit::checkbox_with(ui, at(345.5, 181.0), (12.0, 11.0), "Preview", &mut self.preview, true);
        self.legacy_outcome(ui, (ok, cancel))
    }

    /// A row: the label at the dialog's left (x 11), the field (its left
    /// and center line), and the track (x0, x1, top).
    #[allow(clippy::too_many_arguments)]
    fn rc_row(
        &mut self,
        ui: &mut egui::Ui,
        frame: Rect,
        i: usize,
        label: &str,
        (fx, cy): (f32, f32),
        (x0, x1, top): (f32, f32, f32),
        focus: bool,
    ) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        appkit::text(ui, at(11.0, cy), Align2::LEFT_CENTER, label, appkit::TEXT);
        let p = &self.kind.params()[i];
        appkit::field(
            ui,
            Rect::from_min_max(at(fx, cy - 8.5), at(fx + 43.0, cy + 8.5)),
            &mut self.values[i],
            ("rc-field", i),
            (p.min, p.max),
            1.0,
            0,
            focus && self.first_frame,
        );
        self.classic_track(ui, i, at(x0, top), at(x1, 0.0).x, Scale::Linear);
    }

    /// The selection preview in `rect`: the active layer made small
    /// (`Extra::thumb`) as the selection (gray by how much each pixel is
    /// replaced) or as the image, fitted and centered, never shown larger
    /// than a thumbnail pixel a screen pixel.
    fn replace_preview(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let fuzziness = self.value(FUZZINESS).unwrap_or(40.0);
        let mut samples = self.extra.replace;
        samples.localized = self.value(LOCALIZED) == Some(1.0);
        let key = (samples, fuzziness.to_bits(), self.extra.show_image);
        if let Some((w, h, px)) = &self.extra.thumb
            && self
                .extra
                .preview_texture
                .as_ref()
                .is_none_or(|(k, _)| *k != key)
        {
            let pixels: Vec<Color32> = if self.extra.show_image {
                px.iter().map(|p| Color32::from_rgb(p[0], p[1], p[2])).collect()
            } else {
                // Localized clusters grow from the picked points, moved
                // onto the thumbnail
                let mut small = op_core::TiledImage::new(*w as u32, *h as u32);
                for (k, p) in px.iter().enumerate() {
                    small.set_pixel((k % w) as u32, (k / w) as u32, [p[0], p[1], p[2], 255]);
                }
                let (dw, dh) = self.extra.doc_size;
                let mut s = samples;
                for seed in s.seeds.iter_mut() {
                    seed.0 = (seed.0 as f32 * *w as f32 / dw.max(1.0)) as u32;
                    seed.1 = (seed.1 as f32 * *h as f32 / dh.max(1.0)) as u32;
                }
                let weights = if samples.localized {
                    op_core::color_match::localized_weights(&small, &s, fuzziness)
                } else {
                    px.iter().map(|&p| s.weight(p, fuzziness)).collect()
                };
                weights
                    .into_iter()
                    .map(|v| Color32::from_gray((v * 255.0).round() as u8))
                    .collect()
            };
            let image = egui::ColorImage::new([*w, *h], pixels);
            let texture =
                ui.ctx()
                    .load_texture("replace-color-preview", image, egui::TextureOptions::LINEAR);
            self.extra.preview_texture = Some((key, texture));
        }
        if let Some((_, texture)) = &self.extra.preview_texture {
            let size = texture.size_vec2();
            let ppp = ui.ctx().pixels_per_point();
            let k = (rect.width() / size.x)
                .min(rect.height() / size.y)
                .min(1.0 / ppp);
            let shown = Rect::from_center_size(rect.center(), size * k);
            ui.painter().image(
                texture.id(),
                shown,
                Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
}
