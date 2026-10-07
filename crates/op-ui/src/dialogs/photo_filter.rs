//! Image > Adjustments > Photo Filter...: Photoshop 2026's UXP dialog,
//! 401 × 217 pt: a preset filter or a color of one's own, its density and
//! Preserve luminosity. Sizes are Photoshop points from the dialog's
//! top-left corner.

use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use op_core::adjust::Adjustment;

use super::{common, uxp};
use crate::theme::pt;

pub const SIZE: egui::Vec2 = vec2(pt(401.0), pt(217.0));
const DENSITY: (f32, f32) = (1.0, 100.0);

/// Photoshop's Photo Filter presets and their colors.
pub const FILTERS: [(&str, [u8; 3]); 20] = [
    ("Warming Filter (85)", [0xec, 0x8a, 0x00]),
    ("Warming Filter (LBA)", [0xfa, 0x96, 0x00]),
    ("Warming Filter (81)", [0xeb, 0xb1, 0x13]),
    ("Cooling Filter (80)", [0x00, 0x6d, 0xff]),
    ("Cooling Filter (LBB)", [0x00, 0x5d, 0xff]),
    ("Cooling Filter (82)", [0x00, 0xb5, 0xff]),
    ("Red", [0xea, 0x1a, 0x1a]),
    ("Orange", [0xf3, 0x84, 0x17]),
    ("Yellow", [0xf9, 0xe3, 0x1c]),
    ("Green", [0x19, 0xc9, 0x19]),
    ("Cyan", [0x1d, 0xcb, 0xea]),
    ("Blue", [0x1d, 0x35, 0xea]),
    ("Violet", [0x9b, 0x1d, 0xea]),
    ("Magenta", [0xe3, 0x18, 0xe3]),
    ("Sepia", [0xac, 0x7a, 0x33]),
    ("Deep Red", [0xff, 0x00, 0x00]),
    ("Deep Blue", [0x00, 0x22, 0xcd]),
    ("Deep Emerald", [0x00, 0x8c, 0x00]),
    ("Deep Yellow", [0xff, 0xd5, 0x00]),
    ("Underwater", [0x00, 0xc1, 0xb1]),
];

#[derive(Clone)]
pub struct Dialog {
    /// The preset chosen in the Filter menu.
    pub filter: usize,
    /// Color instead of Filter, and that color.
    pub use_color: bool,
    pub color: [u8; 3],
    pub density: String,
    pub preserve_luminosity: bool,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            filter: 0,
            use_color: false,
            color: FILTERS[0].1,
            density: "25".into(),
            preserve_luminosity: true,
        }
    }
}

fn parse(text: &str) -> Option<u8> {
    let v: f32 = text.trim().trim_end_matches('%').trim().parse().ok()?;
    (DENSITY.0..=DENSITY.1)
        .contains(&v)
        .then_some(v.round() as u8)
}

impl Dialog {
    pub fn adjustment(&self) -> Option<Adjustment> {
        Some(Adjustment::PhotoFilter {
            color: if self.use_color {
                self.color
            } else {
                FILTERS[self.filter].1
            },
            density: parse(&self.density)?,
            preserve_luminosity: self.preserve_luminosity,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        first_frame: bool,
        preview: &mut bool,
    ) -> Option<uxp::Button> {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));

        if uxp::radio(ui, at(26.0, 60.5), "Filter", !self.use_color) {
            self.use_color = false;
        }
        let name = FILTERS[self.filter].0;
        let mut chosen = None;
        common::ps_dropdown(
            ui,
            r(78.5, 48.5, 260.0, 72.5),
            "photo-filter-preset",
            |p, rect| {
                p.text(
                    rect.left_center() + vec2(pt(9.0), pt(0.75)),
                    Align2::LEFT_CENTER,
                    name,
                    uxp::font(),
                    uxp::TEXT,
                );
            },
            |ui| {
                for (i, (n, _)) in FILTERS.iter().enumerate() {
                    if ui.button(*n).clicked() {
                        chosen = Some(i);
                    }
                }
            },
        );
        if let Some(i) = chosen {
            self.filter = i;
            self.color = FILTERS[i].1;
            self.use_color = false;
        }

        if uxp::radio(ui, at(26.0, 94.5), "Color", self.use_color) {
            self.use_color = true;
        }
        // The color swatch; clicking it picks a color of one's own
        let swatch = r(79.5, 82.5, 127.0, 105.5);
        let [cr, cg, cb] = self.color;
        ui.painter().rect(
            swatch,
            CornerRadius::same(pt(4.0) as u8),
            Color32::from_rgb(cr, cg, cb),
            Stroke::new(pt(1.0), Color32::from_gray(0xb0)),
            StrokeKind::Inside,
        );
        let response = ui.interact(swatch, ui.id().with("photo-filter-color"), Sense::click());
        egui::Popup::menu(&response)
            .id(ui.id().with("photo-filter-picker"))
            .show(|ui| {
                let mut c = Color32::from_rgb(cr, cg, cb);
                if egui::color_picker::color_picker_color32(
                    ui,
                    &mut c,
                    egui::color_picker::Alpha::Opaque,
                ) {
                    self.color = [c.r(), c.g(), c.b()];
                    self.use_color = true;
                }
            });

        uxp::label(ui, at(20.0, 131.5), "Density");
        let field = r(213.0, 118.5, 260.0, 142.5);
        uxp::number_box(
            ui,
            field,
            &mut self.density,
            "photo-filter-density",
            DENSITY,
            false,
        );
        let _ = first_frame;
        if !self.density.contains('%') {
            let shown = ui
                .painter()
                .layout_no_wrap(self.density.clone(), uxp::font(), uxp::TEXT);
            ui.painter().text(
                field.left_center() + vec2(pt(11.0) + shown.size().x, pt(0.75)),
                Align2::LEFT_CENTER,
                "%",
                uxp::font(),
                uxp::TEXT,
            );
        }
        let value = parse(&self.density).unwrap_or(25) as f32;
        let track = |_| uxp::TRACK;
        // The track runs from 0% (values below 1% are not accepted)
        if let Some(v) = uxp::linear_slider(
            ui,
            "photo-filter",
            at(20.0, 0.0).x,
            at(260.0, 0.0).x,
            at(0.0, 154.0).y,
            value,
            DENSITY,
            &track,
            1.0,
        ) {
            self.density = format!("{}", v.max(1.0));
        }
        uxp::checkbox(
            ui,
            at(20.0, 174.0),
            "Preserve luminosity",
            &mut self.preserve_luminosity,
        );
        uxp::preview(ui, at(272.0, 127.0), preview);
        uxp::buttons(ui, frame, None, self.adjustment().is_some(), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_or_color() {
        let mut d = Dialog::default();
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::PhotoFilter {
                color: [0xec, 0x8a, 0x00],
                density: 25,
                preserve_luminosity: true
            })
        );
        d.filter = 3;
        d.color = [1, 2, 3];
        let Some(Adjustment::PhotoFilter { color, .. }) = d.adjustment() else {
            panic!()
        };
        assert_eq!(color, FILTERS[3].1);
        d.use_color = true;
        d.density = "60%".into();
        assert_eq!(
            d.adjustment(),
            Some(Adjustment::PhotoFilter {
                color: [1, 2, 3],
                density: 60,
                preserve_luminosity: true
            })
        );
        d.density = "0".into();
        assert_eq!(d.adjustment(), None);
    }
}
