//! Filter › Noise › Reduce Noise..., as Photoshop 2026 lays it out (807 ×
//! 658 pt, measured from 2x captures): the preview on the left; OK,
//! Cancel, Preview, Basic and Advanced at the top right; the Settings group
//! under them with its pop-up, Save and Delete; in Advanced, the Overall
//! and Per Channel tabs.

use std::sync::Mutex;

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, vec2};

use super::legacy::Row;
use super::{AdjustDialog, Outcome, ParamKind};
use crate::dialogs::{appkit, common};
use crate::theme::{self, pt};

pub const SIZE: egui::Vec2 = vec2(pt(807.0), pt(658.0));

/// The settings' order (`Kind::params`).
pub const MODE: usize = 0;
pub const STRENGTH: usize = 1;
pub const JPEG: usize = 5;
pub const CHANNEL: usize = 6;
/// Red's Strength; then red's Preserve Details, green's pair, blue's.
pub const CHANNELS: usize = 7;

/// Settings saved with the Save button this session (name, values).
static SAVED: Mutex<Vec<(String, Vec<String>)>> = Mutex::new(Vec::new());

/// The Overall rows: label center y, label, unit.
const OVERALL: [(f32, &str, Option<&str>); 4] = [
    (255.0, "Strength:", None),
    (311.0, "Preserve Details:", Some("%")),
    (367.0, "Reduce Color Noise:", Some("%")),
    (423.0, "Sharpen Details:", Some("%")),
];

fn overall_row(label: &'static str, y: f32, unit: Option<&'static str>) -> Row {
    Row {
        label,
        label_right: 653.5,
        field: [659.0, y - 9.5, 703.0, y + 8.5],
        unit: unit.map(|u| (u, 708.0)),
        track: (540.5, 758.5, y + 16.0),
        scale: super::filter_layout::Scale::Linear,
    }
}

impl AdjustDialog {
    pub(super) fn reduce_noise_ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Reduce Noise", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        self.legacy_preview(ui, frame, 637.5);
        let buttons = self.legacy_buttons(ui, frame, 616.0);
        self.legacy_preview_check(ui, frame, (607.0, 112.5));

        // Basic or Advanced
        let advanced = self.value(MODE) == Some(1.0);
        for (k, (x, label)) in [(569.5, "Basic"), (646.0, "Advanced")]
            .into_iter()
            .enumerate()
        {
            let font = theme::dialog(pt(13.0));
            if appkit::radio_with(ui, at(x, 155.0), label, advanced == (k == 1), (16.0, font)) {
                self.values[MODE] = k.to_string();
            }
        }

        // Settings: the group, its pop-up, Save and Delete
        appkit::group(
            &painter,
            r(499.0, 193.0, 796.0, 617.0),
            (at(512.0, 0.0).x, at(788.0, 0.0).x),
        );
        appkit::text(
            ui,
            at(518.0, 193.0),
            Align2::LEFT_CENTER,
            "Settings:",
            appkit::TEXT,
        );
        self.settings_popup(ui, r(574.0, 182.5, 740.5, 202.5));
        let save = r(746.0, 184.0, 765.0, 202.0);
        let delete = r(768.0, 184.0, 785.0, 202.0);
        let save_clicked = ui
            .interact(save, ui.id().with("rn-save"), Sense::click())
            .clicked();
        let delete_clicked = ui
            .interact(delete, ui.id().with("rn-delete"), Sense::click())
            .clicked();
        save_icon(&painter, save.center());
        trash_icon(&painter, delete.center());
        if save_clicked {
            let mut saved = SAVED.lock().expect("settings");
            let name = format!("Settings {}", saved.len() + 1);
            saved.push((name.clone(), self.values.clone()));
            self.extra.rn_setting = Some(name);
        }
        if delete_clicked && let Some(name) = self.extra.rn_setting.take() {
            SAVED.lock().expect("settings").retain(|(n, _)| *n != name);
        }

        // Advanced: the Overall and Per Channel tabs
        let per_channel = advanced && self.extra.rn_per_channel;
        if advanced {
            let bar = r(500.0, 213.5, 795.0, 229.5);
            painter.rect_filled(bar, 0, Color32::from_gray(0x47));
            for (k, (x0, x1, label)) in [(500.0, 549.5, "Overall"), (549.5, 622.5, "Per Channel")]
                .into_iter()
                .enumerate()
            {
                let tab = r(x0, 213.5, x1, 229.5);
                let chosen = per_channel == (k == 1);
                if chosen {
                    painter.rect_filled(tab, 0, Color32::from_gray(0x53));
                }
                painter.rect_stroke(
                    tab,
                    0,
                    Stroke::new(pt(1.0), Color32::from_gray(0x3e)),
                    egui::StrokeKind::Inside,
                );
                appkit::text(
                    ui,
                    tab.center(),
                    Align2::CENTER_CENTER,
                    label,
                    if chosen {
                        appkit::TEXT
                    } else {
                        Color32::from_gray(0xc0)
                    },
                );
                if ui
                    .interact(tab, ui.id().with(("rn-tab", k)), Sense::click())
                    .clicked()
                {
                    self.extra.rn_per_channel = k == 1;
                }
            }
        }
        if per_channel {
            self.per_channel_ui(ui, frame);
        } else {
            for (k, (y, label, unit)) in OVERALL.into_iter().enumerate() {
                self.legacy_row(
                    ui,
                    frame,
                    STRENGTH + k,
                    &overall_row(label, y, unit),
                    true,
                    k == 0,
                );
            }
            let mut jpeg = self.value(JPEG) == Some(1.0);
            appkit::checkbox_with(
                ui,
                at(530.0, 472.5),
                (12.0, 9.5),
                "Remove JPEG Artifact",
                &mut jpeg,
                true,
            );
            self.values[JPEG] = (jpeg as u8).to_string();
        }
        self.legacy_outcome(ui, buttons)
    }

    /// Per Channel: the channel's thumbnail, then its group with the
    /// Channel pop-up, Strength and Preserve Details (dimmed while the
    /// channel's Strength is 0).
    fn per_channel_ui(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let channel = (self.value(CHANNEL).unwrap_or(0.0) as usize).min(2);
        if let Some(texture) = &self.extra.rn_thumbs[channel] {
            let size = texture.size_vec2();
            let k = (pt(120.0) / size.x).min(pt(110.0) / size.y).min(pt(1.0));
            let rect = Rect::from_center_size(at(651.5, 334.5), size * k);
            ui.painter().image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        appkit::group(
            ui.painter(),
            r(520.5, 444.0, 778.5, 558.5),
            (at(535.0, 0.0).x, at(717.0, 0.0).x),
        );
        appkit::text(
            ui,
            at(539.5, 444.0),
            Align2::LEFT_CENTER,
            "Channel:",
            appkit::TEXT,
        );
        if let ParamKind::Choice(options) = self.kind.params()[CHANNEL].kind
            && let Some(k) = appkit::popup(
                ui,
                r(594.5, 434.0, 713.0, 453.5),
                "rn-channel",
                options[channel],
                &appkit::choices(options.iter().copied(), channel),
            )
        {
            self.values[CHANNEL] = k.to_string();
        }
        let strength = CHANNELS + channel * 2;
        let row = |label: &'static str, y: f32, unit| Row {
            label,
            label_right: 634.0,
            field: [639.0, y - 9.0, 683.5, y + 9.0],
            unit,
            track: (540.5, 758.5, y + 15.5),
            scale: super::filter_layout::Scale::Linear,
        };
        self.legacy_row(
            ui,
            frame,
            strength,
            &row("Strength:", 474.0, None),
            true,
            false,
        );
        let on = self.value(strength).unwrap_or(0.0) > 0.0;
        self.legacy_row(
            ui,
            frame,
            strength + 1,
            &row("Preserve Details:", 519.0, Some(("%", 688.5))),
            on,
            false,
        );
    }

    /// Settings: Default (Photoshop's values) and those saved this session.
    fn settings_popup(&mut self, ui: &mut Ui, rect: Rect) {
        let saved = SAVED.lock().expect("settings").clone();
        let names: Vec<&str> = std::iter::once("Default")
            .chain(saved.iter().map(|(n, _)| n.as_str()))
            .collect();
        let chosen = self
            .extra
            .rn_setting
            .as_ref()
            .and_then(|n| saved.iter().position(|(m, _)| m == n))
            .map_or(0, |k| k + 1);
        if let Some(k) = appkit::popup(
            ui,
            rect,
            "rn-settings",
            names[chosen],
            &appkit::choices(names.iter().copied(), chosen),
        ) {
            if k == 0 {
                self.extra.rn_setting = None;
                let params = self.kind.params();
                // Keep Basic or Advanced; everything else to Photoshop's
                for (i, p) in params.iter().enumerate().skip(1) {
                    self.values[i] = super::format(self.kind, p.default, p.decimals);
                }
            } else {
                let (name, values) = &saved[k - 1];
                self.values = values.clone();
                self.extra.rn_setting = Some(name.clone());
            }
        }
    }
}

/// Save Settings: a tray with an arrow down into it.
fn save_icon(painter: &egui::Painter, c: Pos2) {
    let s = Stroke::new(pt(1.0), appkit::TEXT);
    let p = |x: f32, y: f32| c + vec2(pt(x), pt(y));
    painter.line_segment([p(0.0, -7.0), p(0.0, 0.0)], s);
    painter.line_segment([p(-3.0, -3.0), p(0.0, 0.0)], s);
    painter.line_segment([p(3.0, -3.0), p(0.0, 0.0)], s);
    painter.add(egui::Shape::line(
        vec![p(-8.0, -1.0), p(-8.0, 4.0), p(8.0, 4.0), p(8.0, -1.0)],
        s,
    ));
    for k in 0..3 {
        painter.circle_filled(p(-2.0 + k as f32 * 2.0, 7.0), pt(0.6), appkit::TEXT);
    }
}

/// Delete Settings: a trash can.
fn trash_icon(painter: &egui::Painter, c: Pos2) {
    let s = Stroke::new(pt(1.0), Color32::from_gray(0x9a));
    let p = |x: f32, y: f32| c + vec2(pt(x), pt(y));
    painter.line_segment([p(-6.0, -5.0), p(6.0, -5.0)], s);
    painter.line_segment([p(-2.0, -7.0), p(2.0, -7.0)], s);
    painter.add(egui::Shape::line(
        vec![p(-5.0, -5.0), p(-4.0, 7.0), p(4.0, 7.0), p(5.0, -5.0)],
        s,
    ));
    for x in [-2.0, 0.0, 2.0] {
        painter.line_segment([p(x, -3.0), p(x, 5.0)], s);
    }
}
