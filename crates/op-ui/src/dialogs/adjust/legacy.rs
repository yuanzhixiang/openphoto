//! The parts Photoshop 2026's larger legacy filter dialogs share (Reduce
//! Noise, Smart Sharpen, Oil Paint, Shape Blur): an unframed preview with
//! the magnifier zoom controls under it, 63 × 29 pt OK and Cancel pills
//! and a Preview checkbox at the top right, and rows of a right-aligned
//! label, a field, a unit and a slider whose pin hangs under its track.

use egui::{Align2, Color32, Key, Pos2, Rect, Ui, vec2};

use super::{AdjustDialog, Kind, Outcome, zoom_controls};
use crate::dialogs::appkit;
use crate::theme::pt;

/// The unframed preview area of each legacy dialog (x0, y0, x1, y1).
pub fn preview_rect(kind: Kind) -> Option<[f32; 4]> {
    Some(match kind {
        Kind::ReduceNoise => [8.0, 36.0, 491.0, 619.0],
        Kind::SmartSharpen => [8.0, 36.0, 271.0, 457.0],
        Kind::OilPaint => [8.0, 36.0, 220.0, 248.0],
        Kind::ShapeBlur => [8.0, 36.0, 219.0, 248.0],
        _ => return None,
    })
}

/// A slider row: the label right-aligned to `label_right` on the field's
/// center line, the field, an optional unit, and the track (x0, x1, top).
pub struct Row {
    pub label: &'static str,
    pub label_right: f32,
    pub field: [f32; 4],
    pub unit: Option<(&'static str, f32)>,
    pub track: (f32, f32, f32),
    /// How values spread along the track (linear in most).
    pub scale: super::filter_layout::Scale,
}

impl AdjustDialog {
    /// This dialog's preview area: `preview_rect`, Smart Sharpen's shorter
    /// with Shadows / Highlights folded.
    pub(super) fn legacy_area(&self) -> Option<[f32; 4]> {
        let area = preview_rect(self.kind)?;
        Some(if self.kind == Kind::SmartSharpen && !self.extra.ss_open {
            [8.0, 36.0, 271.0, 235.5]
        } else {
            area
        })
    }

    /// The preview (the document as filtered at the pane's zoom; dragging
    /// pans it) and its zoom controls centered under it at `zoom_y`.
    pub(super) fn legacy_preview(&mut self, ui: &mut Ui, frame: Rect, zoom_y: f32) {
        let Some(p) = self.legacy_area() else {
            return;
        };
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let rect = Rect::from_min_max(at(p[0], p[1]), at(p[2], p[3]));
        self.pane_ui(ui, rect);
        // The controls' middle (the zoom) sits 113.75 pt from their left
        let left = rect.center().x - pt(113.75);
        let zoom = zoom_controls(ui, at(0.0, zoom_y).y, left, self.pane_zoom);
        self.zoom_pane(zoom);
    }

    /// OK and Cancel at (x, 37) and (x, 72), 63 × 29 pt, 13 pt labels.
    pub(super) fn legacy_buttons(&self, ui: &mut Ui, frame: Rect, x: f32) -> (bool, bool) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let valid = self.effect().is_some();
        let ok = appkit::button_with(
            ui,
            Rect::from_min_max(at(x, 37.0), at(x + 63.0, 66.0)),
            "OK",
            (true, valid),
            13.0,
            0.0,
        );
        let cancel = appkit::button_with(
            ui,
            Rect::from_min_max(at(x, 72.0), at(x + 63.0, 101.0)),
            "Cancel",
            (false, true),
            13.0,
            0.0,
        );
        (ok.clicked(), cancel.clicked())
    }

    /// Cancel closes; OK or Enter (unless a field is being typed in)
    /// applies.
    pub(super) fn legacy_outcome(&self, ui: &Ui, (ok, cancel): (bool, bool)) -> Outcome {
        if cancel {
            return Outcome::Cancel;
        }
        let typing = ui.ctx().egui_wants_keyboard_input();
        let enter = !typing && ui.input(|i| i.key_pressed(Key::Enter));
        if (ok || enter)
            && let Some(effect) = self.effect()
        {
            return Outcome::Apply(effect);
        }
        Outcome::Open
    }

    /// The Preview checkbox, its 12 pt box at `min`, the label 9.5 pt after.
    pub(super) fn legacy_preview_check(&mut self, ui: &mut Ui, frame: Rect, min: (f32, f32)) {
        let at = frame.min + vec2(pt(min.0), pt(min.1));
        appkit::checkbox_with(ui, at, (12.0, 9.5), "Preview", &mut self.preview, true);
    }

    /// Setting `i`'s row (`Row`); `enabled` false dims it and keeps it from
    /// changing. The first row drawn with `focus` takes the keyboard focus
    /// when the dialog opens.
    pub(super) fn legacy_row(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        i: usize,
        row: &Row,
        enabled: bool,
        focus: bool,
    ) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let f = row.field;
        let field = Rect::from_min_max(at(f[0], f[1]), at(f[2], f[3]));
        let ink = if enabled {
            appkit::TEXT
        } else {
            appkit::TEXT_OFF
        };
        appkit::text(
            ui,
            Pos2::new(at(row.label_right, 0.0).x, field.center().y),
            Align2::RIGHT_CENTER,
            row.label,
            ink,
        );
        let p = &self.kind.params()[i];
        if enabled {
            appkit::field(
                ui,
                field,
                &mut self.values[i],
                ("legacy-field", i),
                (p.min, p.max),
                10f32.powi(-(p.decimals as i32)),
                p.decimals,
                focus && self.first_frame,
            );
        } else {
            ui.painter().rect(
                field,
                0,
                Color32::from_gray(0x4d),
                egui::Stroke::new(pt(1.0), Color32::from_gray(0x5d)),
                egui::StrokeKind::Inside,
            );
            appkit::text(
                ui,
                field.left_center() + vec2(pt(4.0), 0.0),
                Align2::LEFT_CENTER,
                &self.values[i],
                appkit::TEXT_OFF,
            );
        }
        if let Some((unit, x)) = row.unit {
            appkit::text(
                ui,
                Pos2::new(at(x, 0.0).x, field.center().y),
                Align2::LEFT_CENTER,
                unit,
                ink,
            );
        }
        let (x0, x1, top) = row.track;
        if enabled {
            self.classic_track(ui, i, at(x0, top), at(x1, 0.0).x, row.scale);
        } else {
            let line = Rect::from_min_max(at(x0, top), at(x1, top + 3.0));
            ui.painter().rect_filled(line, 0, Color32::from_gray(0x75));
            let v = self.value(i).unwrap_or(p.default);
            let (a, b) = (line.left() - pt(2.25), line.right() + pt(0.75));
            let x = a + (b - a) * row.scale.place(v, p.min, p.max);
            appkit::pin(
                ui.painter(),
                Pos2::new(x, line.bottom() + pt(0.5)),
                appkit::Pin::White,
            );
        }
    }
}
