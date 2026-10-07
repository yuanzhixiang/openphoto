//! Photoshop 2026's small filter dialogs without a preview (Stylize ›
//! Tiles..., Pixelate › Color Halftone...): plug-in style OK and Cancel
//! (89 pt wide) at the top right and the settings as text, fields, radio
//! buttons and checkboxes, measured on Photoshop. Coordinates are
//! Photoshop points from the dialog's top-left corner.

/// One thing in the dialog; every item but `Text` sets the next setting.
pub enum Item {
    /// A label (left x, center y).
    Text { text: &'static str, at: (f32, f32) },
    /// A number field (x0, y0, x1, y1) with its label to the left and an
    /// optional unit to the right, both at the field's center line.
    Field {
        label: (&'static str, f32),
        rect: [f32; 4],
        unit: Option<(&'static str, f32)>,
    },
    /// Radio buttons for a choice: their centers, labels `gap` points
    /// after the center.
    Radios {
        centers: &'static [(f32, f32)],
        gap: f32,
    },
}

pub struct Layout {
    pub size: (f32, f32),
    /// OK's and Cancel's left edge.
    pub buttons_x: f32,
    pub items: &'static [Item],
}

/// Stylize › Tiles (302 × 220 pt).
pub const TILES: Layout = Layout {
    size: (302.0, 220.0),
    buttons_x: 195.5,
    items: &[
        Item::Field {
            label: ("Number Of Tiles:", 17.0),
            rect: [117.0, 44.0, 146.0, 65.0],
            unit: None,
        },
        Item::Field {
            label: ("Maximum Offset:", 17.0),
            rect: [118.0, 81.0, 147.0, 102.0],
            unit: Some(("%", 156.0)),
        },
        Item::Text {
            text: "Fill Empty Area With:",
            at: (23.0, 116.75),
        },
        Item::Radios {
            centers: &[(21.0, 134.0), (21.0, 154.0), (21.0, 174.0), (21.0, 194.0)],
            gap: 13.0,
        },
    ],
};

/// Pixelate › Color Halftone (326 × 243 pt).
pub const COLOR_HALFTONE: Layout = Layout {
    size: (326.0, 243.0),
    buttons_x: 219.5,
    items: &[
        Item::Field {
            label: ("Max. Radius:", 9.0),
            rect: [87.0, 36.5, 150.5, 56.5],
            unit: Some(("(Pixels)", 160.5)),
        },
        Item::Text {
            text: "Screen Angles (Degrees):",
            at: (13.0, 76.0),
        },
        Item::Field {
            label: ("Channel 1:", 17.0),
            rect: [83.5, 95.5, 146.5, 115.5],
            unit: None,
        },
        Item::Field {
            label: ("Channel 2:", 17.0),
            rect: [83.5, 132.5, 146.5, 152.5],
            unit: None,
        },
        Item::Field {
            label: ("Channel 3:", 17.0),
            rect: [83.5, 169.5, 146.5, 189.5],
            unit: None,
        },
        Item::Field {
            label: ("Channel 4:", 17.0),
            rect: [83.5, 206.5, 146.5, 226.5],
            unit: None,
        },
    ],
};
