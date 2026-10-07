//! Photoshop 2026's small filter dialogs without a preview (Stylize ›
//! Tiles..., Pixelate › Color Halftone..., Blur › Radial Blur...): OK and Cancel
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
    /// A slider for the field before it: a 3 pt track (x0, x1, top) with
    /// the pin standing on it, its tip 3 pt above the track, its middle at
    /// `pins.0` for the setting's minimum and `pins.1` for its maximum.
    Track {
        x0: f32,
        x1: f32,
        top: f32,
        pins: (f32, f32),
    },
    /// A group box (x0, y0, x1, y1) with its title on the top edge, 19 pt
    /// in; sets nothing.
    Group { title: &'static str, rect: [f32; 4] },
    /// Radial Blur's Blur Center: a white box drawing the blur's pattern
    /// around the center, which a click or drag moves; sets nothing.
    CenterBox { rect: [f32; 4] },
}

pub struct Layout {
    pub size: (f32, f32),
    /// OK's and Cancel's left edge.
    pub buttons_x: f32,
    /// The buttons: width, OK's top, Cancel's top, height, label size.
    pub buttons: (f32, f32, f32, f32, f32),
    /// The labels' size (radio labels too).
    pub text_size: f32,
    pub items: &'static [Item],
}

/// The plug-in style dialogs' buttons: 89 × 26 at y 41 and 77, 13 pt.
const PLUGIN_BUTTONS: (f32, f32, f32, f32, f32) = (89.0, 41.0, 77.0, 26.0, 13.0);

/// Stylize › Tiles (302 × 220 pt).
pub const TILES: Layout = Layout {
    size: (302.0, 220.0),
    buttons_x: 195.5,
    buttons: PLUGIN_BUTTONS,
    text_size: 11.0,
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
    buttons: PLUGIN_BUTTONS,
    text_size: 11.0,
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

/// Other › HSB/HSL ("HSB/HSL Parameters", 270 × 163 pt): Input mode and
/// Row order radios side by side, 11 pt; 78 × 24 buttons with 11 pt labels.
pub const HSB_HSL: Layout = Layout {
    size: (270.0, 163.0),
    buttons_x: 172.0,
    buttons: (78.0, 48.0, 80.0, 24.0, 11.0),
    text_size: 11.0,
    items: &[
        Item::Text {
            text: "Input mode",
            at: (20.0, 60.5),
        },
        Item::Radios {
            centers: &[(26.0, 83.0), (26.0, 107.0), (26.0, 131.0)],
            gap: 15.0,
        },
        Item::Text {
            text: "Row order",
            at: (101.0, 60.5),
        },
        Item::Radios {
            centers: &[(107.0, 83.0), (107.0, 107.0), (107.0, 131.0)],
            gap: 15.0,
        },
    ],
};

/// Blur › Radial Blur (274 × 280 pt): Amount over a slider, the Blur
/// Method and Quality groups, and the Blur Center box; 108.5 × 26 buttons.
pub const RADIAL_BLUR: Layout = Layout {
    size: (274.0, 280.0),
    buttons_x: 148.0,
    buttons: (108.5, 39.0, 75.0, 26.0, 13.0),
    text_size: 11.0,
    items: &[
        Item::Field {
            label: ("Amount", 19.5),
            rect: [83.0, 48.5, 117.0, 67.0],
            unit: None,
        },
        Item::Track {
            x0: 14.5,
            x1: 121.5,
            top: 76.0,
            pins: (17.75, 116.75),
        },
        Item::Group {
            title: "Blur Method:",
            rect: [12.0, 113.0, 117.5, 175.5],
        },
        Item::Radios {
            centers: &[(26.0, 133.0), (26.0, 156.0)],
            gap: 13.5,
        },
        Item::Group {
            title: "Quality:",
            rect: [12.0, 184.0, 117.5, 266.5],
        },
        Item::Radios {
            centers: &[(26.0, 204.0), (26.0, 226.0), (26.0, 248.0)],
            gap: 13.5,
        },
        Item::Text {
            text: "Blur Center",
            at: (133.0, 125.0),
        },
        Item::CenterBox {
            rect: [132.0, 140.0, 262.0, 270.0],
        },
    ],
};
