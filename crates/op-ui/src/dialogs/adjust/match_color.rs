//! Image › Adjustments › Match Color..., as Photoshop 2026 lays it out
//! (481 × 559 pt, measured from a 2x capture): Destination Image with the
//! target and Image Options (Luminance, Color Intensity, Fade,
//! Neutralize), Image Statistics with the selection switches, Source and
//! Layer, Load / Save Statistics and the source's thumbnail; OK, Cancel
//! and Preview on the right.

use egui::{Align2, Color32, Pos2, Rect, Stroke, StrokeKind, vec2};

use super::filter_layout::Scale;
use super::{AdjustDialog, MatchSource, Outcome};
use crate::dialogs::{appkit, common, preset_files};
use crate::native_popup::Entry;
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(481.0), pt(559.0));

/// The settings' order (`Kind::params`).
const NEUTRALIZE: usize = 3;
const SOURCE: usize = 4;
const LAYER: usize = 5;
const IGNORE_SELECTION: usize = 6;
const SOURCE_SELECTION: usize = 7;
const TARGET_SELECTION: usize = 8;

impl AdjustDialog {
    pub(super) fn match_color_ui(&mut self, ui: &mut egui::Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Match Color", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        let group = |ui: &egui::Ui, rect: Rect, x: f32, title: &str| {
            let t = appkit::label(ui, Pos2::new(at(x, 0.0).x, rect.top()), title);
            appkit::group(&painter, rect, (t.left() - pt(6.0), t.right() + pt(5.0)));
        };
        let has_selection = self.extra.target_selected.is_some();

        // Destination Image: the target, Ignore Selection, Image Options
        group(
            ui,
            r(10.75, 46.75, 381.25, 318.75),
            31.0,
            "Destination Image",
        );
        appkit::text(
            ui,
            at(24.5, 69.25),
            Align2::LEFT_CENTER,
            "Target:",
            appkit::TEXT,
        );
        let target = format!("{} (RGB/8)", self.extra.target_name);
        appkit::text(
            ui,
            at(71.5, 69.25),
            Align2::LEFT_CENTER,
            &target,
            appkit::TEXT,
        );
        self.mc_check(ui, frame, IGNORE_SELECTION, (68.0, 88.0), has_selection);
        group(ui, r(20.75, 119.75, 371.75, 308.75), 40.5, "Image Options");
        for (k, label) in ["Luminance", "Color Intensity", "Fade"]
            .into_iter()
            .enumerate()
        {
            let cy = 143.5 + 49.0 * k as f32;
            appkit::text(ui, at(68.5, cy), Align2::LEFT_CENTER, label, appkit::TEXT);
            let p = &self.kind.params()[k];
            appkit::field(
                ui,
                r(242.5, cy - 8.5, 286.0, cy + 8.5),
                &mut self.values[k],
                ("mc-field", k),
                (p.min, p.max),
                1.0,
                0,
                k == 2 && self.first_frame,
            );
            self.classic_track(ui, k, at(78.5, cy + 16.5), at(275.0, 0.0).x, Scale::Linear);
        }
        self.mc_check(ui, frame, NEUTRALIZE, (68.0, 285.0), true);

        // Image Statistics
        group(
            ui,
            r(10.75, 336.75, 381.25, 548.0),
            31.0,
            "Image Statistics",
        );
        let source = self.value(SOURCE).unwrap_or(0.0) as usize;
        let source_has_selection = source > 0
            && self
                .extra
                .sources
                .get(source - 1)
                .and_then(|s| s.layers.first())
                .is_some_and(|l| l.2.is_some());
        self.mc_check(
            ui,
            frame,
            SOURCE_SELECTION,
            (68.0, 355.0),
            source_has_selection,
        );
        self.mc_check(ui, frame, TARGET_SELECTION, (68.0, 389.0), has_selection);
        // Source and Layer, labels right-aligned at 61.5
        for (i, cy) in [(SOURCE, 431.0), (LAYER, 460.0)] {
            let label = if i == SOURCE { "Source:" } else { "Layer:" };
            let on = i == SOURCE || source > 0;
            let ink = if on { appkit::TEXT } else { appkit::TEXT_OFF };
            appkit::text(ui, at(61.5, cy), Align2::RIGHT_CENTER, label, ink);
            let labels = self
                .extra
                .labels(self.kind, i, Some(source))
                .unwrap_or_default();
            let chosen = self.value(i).unwrap_or(0.0) as usize;
            let shown = labels.get(chosen).cloned().unwrap_or_default();
            let rect = r(68.0, cy - 9.0, 213.5, cy + 9.0);
            if on {
                let entries: Vec<Entry> = labels
                    .iter()
                    .enumerate()
                    .map(|(k, l)| Entry::item(l.clone(), k == chosen))
                    .collect();
                if let Some(k) = appkit::popup(ui, rect, &format!("mc-popup-{i}"), &shown, &entries)
                {
                    self.values[i] = k.to_string();
                    // A new source starts with its merged image
                    if i == SOURCE {
                        self.values[LAYER] = "0".into();
                    }
                }
            } else {
                painter.rect(
                    rect,
                    egui::CornerRadius::same(pt(2.0) as u8),
                    Color32::from_gray(0x4e),
                    Stroke::new(pt(1.0), Color32::from_gray(0x5d)),
                    StrokeKind::Inside,
                );
                appkit::text(
                    ui,
                    rect.left_center() + vec2(pt(8.0), 0.0),
                    Align2::LEFT_CENTER,
                    &shown,
                    appkit::TEXT_OFF,
                );
                crate::ps_icons::paint(
                    &painter,
                    Pos2::new(rect.right() - pt(8.0), rect.center().y),
                    crate::ps_icons::Icon::Caret,
                    appkit::TEXT_OFF,
                    Color32::from_gray(0x4e),
                );
            }
        }
        // Load / Save Statistics
        // (flat buttons, as Photoshop draws these two)
        let load = super::flat_button(
            ui,
            r(67.5, 477.5, 191.5, 503.0),
            "Load Statistics...",
            "mc-load",
        );
        let save = super::flat_button(
            ui,
            r(67.5, 512.5, 191.5, 538.0),
            "Save Statistics...",
            "mc-save",
        );
        if load
            && let Some((name, bytes)) = preset_files::MATCH_STATISTICS.load()
            && let Some(stats) = preset_files::decode_match_statistics(&bytes)
        {
            self.extra.sources.push(MatchSource {
                name,
                layers: vec![("Merged".to_owned(), stats, None)],
                thumb: None,
            });
            self.values[SOURCE] = self.extra.sources.len().to_string();
            self.values[LAYER] = "0".into();
        }
        if save {
            // The source's statistics (the target's with Source None)
            let stats = match source {
                0 => self.extra.target,
                k => self
                    .extra
                    .sources
                    .get(k - 1)
                    .and_then(|s| s.layers.get(self.value(LAYER).unwrap_or(0.0) as usize))
                    .map(|l| l.1),
            };
            if let Some(stats) = stats {
                preset_files::MATCH_STATISTICS.save(&preset_files::encode_match_statistics(stats));
            }
        }
        // The source's thumbnail (the target's with Source None)
        self.mc_thumbnail(ui, r(259.0, 429.0, 358.5, 528.5), source);

        // OK, Cancel and Preview
        let valid = self.effect().is_some();
        let ok = appkit::button(ui, r(411.5, 45.0, 471.0, 70.5), "OK", true, valid).clicked();
        let cancel =
            appkit::button(ui, r(411.5, 80.0, 471.0, 105.5), "Cancel", false, true).clicked();
        appkit::checkbox_with(
            ui,
            at(392.0, 125.0),
            (12.0, 10.5),
            "Preview",
            &mut self.preview,
            true,
        );
        self.legacy_outcome(ui, (ok, cancel))
    }

    /// A checkbox setting with its 12 pt box at `min`, the label 10 pt
    /// after; off, it is dimmed and unchecked.
    fn mc_check(
        &mut self,
        ui: &mut egui::Ui,
        frame: Rect,
        i: usize,
        min: (f32, f32),
        enabled: bool,
    ) {
        let at = frame.min + vec2(pt(min.0), pt(min.1));
        let label = self.kind.params()[i].label;
        let mut on = enabled && self.value(i) == Some(1.0);
        appkit::checkbox_with(ui, at, (12.0, 10.0), label, &mut on, enabled);
        if enabled {
            self.values[i] = (on as u8).to_string();
        }
    }

    /// The source document's thumbnail in `rect` (fitted, centered, a
    /// 1 pt black frame round it), or the target's with Source None.
    fn mc_thumbnail(&mut self, ui: &mut egui::Ui, rect: Rect, source: usize) {
        let thumb = match source {
            0 => self.extra.thumb.clone(),
            k => self.extra.sources.get(k - 1).and_then(|s| s.thumb.clone()),
        };
        let Some((w, h, px)) = thumb else {
            return;
        };
        let key = (
            op_core::color_match::ReplaceSamples::default(),
            source as u32,
            false,
        );
        if self
            .extra
            .preview_texture
            .as_ref()
            .is_none_or(|(k, _)| *k != key)
        {
            let pixels: Vec<Color32> = px
                .iter()
                .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
                .collect();
            let image = egui::ColorImage::new([w, h], pixels);
            let texture =
                ui.ctx()
                    .load_texture("match-color-thumb", image, egui::TextureOptions::LINEAR);
            self.extra.preview_texture = Some((key, texture));
        }
        if let Some((_, texture)) = &self.extra.preview_texture {
            let size = texture.size_vec2();
            let k = (rect.width() / size.x).min(rect.height() / size.y);
            let shown = Rect::from_center_size(rect.center(), size * k);
            ui.painter().image(
                texture.id(),
                shown,
                Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().rect_stroke(
                shown,
                0,
                Stroke::new(pt(1.0), Color32::BLACK),
                StrokeKind::Outside,
            );
        }
    }
}
