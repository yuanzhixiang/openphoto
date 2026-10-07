//! Panel Options for the floating panels: Info Panel Options (the two
//! color readouts, the mouse coordinates' unit, the status lines) and
//! Navigator Panel Options (the view box's color).

use egui::Ui;

use crate::rulers::RulerUnit;

/// A color readout mode of the Info panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readout {
    Actual,
    Rgb,
    Web,
    Hsb,
    Grayscale,
    Cmyk,
    Lab,
}

impl Readout {
    pub const ALL: [Readout; 7] = [
        Readout::Actual,
        Readout::Rgb,
        Readout::Web,
        Readout::Hsb,
        Readout::Grayscale,
        Readout::Cmyk,
        Readout::Lab,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Readout::Actual => "Actual Color",
            Readout::Rgb => "RGB Color",
            Readout::Web => "Web Color",
            Readout::Hsb => "HSB Color",
            Readout::Grayscale => "Grayscale",
            Readout::Cmyk => "CMYK Color",
            Readout::Lab => "Lab Color",
        }
    }

    /// The readout's lines for `color` (labels with empty values without
    /// one): R, G, B; hex R, G, B; H, S, B; K; C, M, Y, K; L, a, b.
    pub fn lines(self, color: Option<op_core::Color>) -> Vec<(&'static str, String)> {
        let empty = |labels: &[&'static str]| labels.iter().map(|l| (*l, String::new())).collect();
        let Some(c) = color else {
            return match self {
                Readout::Actual | Readout::Rgb | Readout::Web => empty(&["R:", "G:", "B:"]),
                Readout::Hsb => empty(&["H:", "S:", "B:"]),
                Readout::Grayscale => empty(&["K:"]),
                Readout::Cmyk => empty(&["C:", "M:", "Y:", "K:"]),
                Readout::Lab => empty(&["L:", "a:", "b:"]),
            };
        };
        let [r, g, b, _] = c.to_rgba8();
        match self {
            Readout::Actual | Readout::Rgb => vec![
                ("R:", r.to_string()),
                ("G:", g.to_string()),
                ("B:", b.to_string()),
            ],
            Readout::Web => vec![
                ("R:", format!("{r:02X}")),
                ("G:", format!("{g:02X}")),
                ("B:", format!("{b:02X}")),
            ],
            Readout::Hsb => {
                let h = op_color::Hsb::from_color(c);
                vec![
                    ("H:", format!("{:.0}°", h.h.round())),
                    ("S:", format!("{:.0}%", h.s * 100.0)),
                    ("B:", format!("{:.0}%", h.b * 100.0)),
                ]
            }
            Readout::Grayscale => {
                let l = op_core::adjust::luminosity([r, g, b, 255]) as f32;
                vec![("K:", format!("{:.0}%", (255.0 - l) / 2.55))]
            }
            Readout::Cmyk => {
                let k = op_color::Cmyk::from_color(c);
                [("C:", k.c), ("M:", k.m), ("Y:", k.y), ("K:", k.k)]
                    .into_iter()
                    .map(|(l, v)| (l, format!("{:.0}%", v * 100.0)))
                    .collect()
            }
            Readout::Lab => {
                let lab = op_color::Lab::from_color(c);
                vec![
                    ("L:", format!("{:.0}", lab.l)),
                    ("a:", format!("{:.0}", lab.a)),
                    ("b:", format!("{:.0}", lab.b)),
                ]
            }
        }
    }
}

/// Info Panel Options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InfoOptions {
    pub first: Readout,
    pub second: Readout,
    /// The mouse coordinates' unit; None follows the rulers'.
    pub units: Option<RulerUnit>,
    /// The status lines under the readouts (Document Sizes by default).
    pub status: [bool; 3],
    pub tool_hints: bool,
}

impl Default for InfoOptions {
    /// Photoshop's: Actual Color and CMYK, Document Sizes, Show Tool Hints.
    fn default() -> Self {
        Self {
            first: Readout::Actual,
            second: Readout::Cmyk,
            units: None,
            status: [true, false, false],
            tool_hints: true,
        }
    }
}

/// The status lines Info Panel Options offers, as the status bar names them.
pub const STATUS: [crate::status_info::StatusInfo; 3] = [
    crate::status_info::StatusInfo::DocumentSizes,
    crate::status_info::StatusInfo::DocumentDimensions,
    crate::status_info::StatusInfo::CurrentTool,
];

/// The Navigator's view box colors, Light Red (Photoshop's) first.
pub const VIEW_BOX_COLORS: [(&str, [u8; 3]); 6] = [
    ("Light Red", [0xff, 0x00, 0x00]),
    ("Cyan", [0x00, 0xff, 0xff]),
    ("Green", [0x4a, 0xff, 0x4a]),
    ("Light Blue", [0x4a, 0x9c, 0xff]),
    ("Yellow", [0xff, 0xff, 0x4a]),
    ("Magenta", [0xff, 0x4a, 0xff]),
];

/// The Info Panel Options' controls.
pub fn info_options_ui(ui: &mut Ui, o: &mut InfoOptions) {
    for (title, readout) in [
        ("First Color Readout", &mut o.first),
        ("Second Color Readout", &mut o.second),
    ] {
        ui.strong(title);
        egui::ComboBox::from_id_salt(title)
            .selected_text(readout.label())
            .show_ui(ui, |ui| {
                for r in Readout::ALL {
                    ui.selectable_value(readout, r, r.label());
                }
            });
    }
    ui.strong("Mouse Coordinates");
    egui::ComboBox::from_id_salt("info-units")
        .selected_text(o.units.map_or("Rulers' Units", |u| u.label()))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut o.units, None, "Rulers' Units");
            for u in RulerUnit::ALL {
                ui.selectable_value(&mut o.units, Some(u), u.label());
            }
        });
    ui.strong("Status Information");
    for (k, s) in STATUS.iter().enumerate() {
        ui.checkbox(&mut o.status[k], s.label());
    }
    ui.checkbox(&mut o.tool_hints, "Show Tool Hints");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readouts() {
        let c = op_core::Color::from_rgba8([255, 0, 0, 255]);
        assert_eq!(Readout::Web.lines(Some(c))[0], ("R:", "FF".to_string()));
        assert_eq!(Readout::Hsb.lines(Some(c))[1], ("S:", "100%".to_string()));
        assert_eq!(Readout::Grayscale.lines(None), vec![("K:", String::new())]);
        assert_eq!(Readout::Lab.lines(Some(c))[0].0, "L:");
    }
}
