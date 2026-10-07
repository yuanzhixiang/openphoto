//! Image > Trim: "Based on" with three choices, "Trim away" with a
//! checkbox per side, OK and Cancel, laid out at the positions measured on
//! Photoshop 2026's dialog (258 × 248 pt, the New Layer dialog's family).
//! Sizes are in Photoshop points from the dialog's top-left corner.

use egui::{Align2, Color32, Key, Rect, Sense, Stroke, Ui, vec2};
use op_core::image_ops::{TrimBasis, TrimSides};

use super::common;
use crate::theme::{self, pt};

const SIZE: egui::Vec2 = vec2(pt(258.0), pt(248.0));
const FONT: f32 = pt(12.0);
const TEXT: Color32 = Color32::from_gray(0xf1);
const TEXT_OFF: Color32 = Color32::from_gray(0x8e);

pub enum Outcome {
    Open,
    Cancel,
    Apply { basis: TrimBasis, sides: TrimSides },
}

pub struct TrimDialog {
    basis: TrimBasis,
    sides: TrimSides,
    /// "Transparent Pixels" needs a document without a background layer.
    can_trim_transparent: bool,
}

impl TrimDialog {
    pub fn new(has_background: bool) -> Self {
        Self {
            basis: if has_background {
                TrimBasis::TopLeftColor
            } else {
                TrimBasis::Transparent
            },
            sides: TrimSides::default(),
            can_trim_transparent: !has_background,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("trim-dialog"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        common::frame(ui, frame, "Trim", theme::dialog_bold(pt(13.0)));
        let painter = ui.painter().clone();
        let heading = |cy: f32, text: &str| {
            painter.text(
                at(20.0, cy),
                Align2::LEFT_CENTER,
                text,
                theme::uxp_bold(FONT),
                TEXT,
            );
        };

        heading(57.0, "Based on");
        let choices = [
            (TrimBasis::Transparent, "Transparent pixels"),
            (TrimBasis::TopLeftColor, "Top left pixel color"),
            (TrimBasis::BottomRightColor, "Bottom right pixel color"),
        ];
        for (i, (basis, label)) in choices.into_iter().enumerate() {
            let enabled = basis != TrimBasis::Transparent || self.can_trim_transparent;
            if radio(
                ui,
                at(26.0, 84.0 + 24.0 * i as f32),
                label,
                self.basis == basis,
                enabled,
            ) {
                self.basis = basis;
            }
        }

        heading(166.0, "Trim away");
        let sides = [
            (&mut self.sides.top, "Top", 20.0, 186.0),
            (&mut self.sides.left, "Left", 97.0, 186.0),
            (&mut self.sides.bottom, "Bottom", 20.0, 210.0),
            (&mut self.sides.right, "Right", 97.0, 210.0),
        ];
        for (flag, label, x, y) in sides {
            common::ps_checkbox(ui, at(x, y), label, flag, true);
        }

        let ok = common::ps_button(ui, r(168.0, 48.0, 238.0, 72.0), "OK", true, true, true);
        let cancel = common::ps_button(
            ui,
            r(168.0, 84.0, 238.0, 108.0),
            "Cancel",
            false,
            true,
            true,
        );
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        if ok.clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
            return Outcome::Apply {
                basis: self.basis,
                sides: self.sides,
            };
        }
        Outcome::Open
    }
}

/// A 12 pt radio button centered on `center`, its label 15 pt to the
/// right, measured: chosen, a `#d6d6d6` disc with a dark dot; otherwise a
/// 1 pt `#a0a0a0` ring (dimmed when it can't be chosen). Returns whether
/// it was clicked.
fn radio(ui: &mut Ui, center: egui::Pos2, label: &str, chosen: bool, enabled: bool) -> bool {
    let ink = if enabled { TEXT } else { TEXT_OFF };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), theme::uxp(FONT), ink);
    let hit = Rect::from_min_max(
        center - vec2(pt(6.0), pt(8.0)),
        egui::pos2(center.x + pt(15.0) + galley.size().x, center.y + pt(8.0)),
    );
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let clicked = ui
        .interact(hit, ui.id().with(("trim-radio", label)), sense)
        .clicked();
    let painter = ui.painter();
    if chosen {
        painter.circle_filled(center, pt(6.0), Color32::from_gray(0xd6));
        painter.circle_filled(center, pt(2.25), Color32::from_gray(0x53));
    } else {
        let ring = if enabled {
            Color32::from_gray(0xa0)
        } else {
            Color32::from_gray(0x6e)
        };
        painter.circle_stroke(center, pt(5.5), Stroke::new(pt(1.0), ring));
    }
    painter.galley(
        egui::pos2(center.x + pt(15.0), center.y - galley.size().y / 2.0),
        galley,
        ink,
    );
    clicked
}
