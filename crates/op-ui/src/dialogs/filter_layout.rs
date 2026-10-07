//! The classic Photoshop 2026 filter dialogs' layouts (Gaussian Blur, Box
//! Blur, Unsharp Mask, Add Noise, Median, Minimum, Maximum, High Pass,
//! Offset, Mosaic, Motion Blur, Emboss, Surface Blur, Dust & Scratches,
//! Trace Contour),
//! measured on Photoshop: a preview pane at the top left with zoom
//! controls under it, OK / Cancel / Preview at the top right, and the
//! settings below: "Label: [field] unit" rows over 3 pt slider tracks with
//! a pin, pop-up menus, radio groups and checkboxes. Coordinates are
//! Photoshop points from the dialog's top-left corner.

/// One setting's place; the order matches the dialog's parameters.
pub enum Row {
    Number {
        label: &'static str,
        /// The label's right end (labels are right-aligned to the field).
        label_right: f32,
        /// Field (x0, y0, x1, y1).
        field: [f32; 4],
        unit: &'static str,
        /// Where the unit starts.
        unit_x: f32,
        /// Track (x0, x1, top); the pin's tip is half a point under it.
        track: Option<(f32, f32, f32)>,
        /// How values spread along the track.
        scale: Scale,
        /// An angle dial (center x, y, radius) next to the field.
        dial: Option<(f32, f32, f32)>,
    },
    Popup {
        label: &'static str,
        label_right: f32,
        rect: [f32; 4],
    },
    Radios {
        title: &'static str,
        /// Group box (x0, y0, x1, y1); its title sits on the top edge.
        group: [f32; 4],
        /// The radio buttons' centers (y) at x 27; labels at 43.5.
        ys: &'static [f32],
    },
    Check {
        label: &'static str,
        /// The box's top-left corner.
        min: (f32, f32),
    },
    /// A setting this layout doesn't show (Shadows/Highlights without
    /// Show More Options).
    Hidden,
}

/// Where a value sits along a slider track, 0–1. Photoshop's sliders are
/// mostly not even: measured from Photoshop 2026 by typing values and
/// reading the pin's place.
#[derive(Clone, Copy)]
pub enum Scale {
    /// Evenly from the parameter's minimum to its maximum.
    Linear,
    /// (value, place) points, interpolated between.
    Table(&'static [(f32, f32)]),
}

impl Scale {
    pub fn place(self, v: f32, min: f32, max: f32) -> f32 {
        match self {
            Scale::Linear => ((v - min) / (max - min)).clamp(0.0, 1.0),
            Scale::Table(t) => {
                if v <= t[0].0 {
                    return 0.0;
                }
                for w in t.windows(2) {
                    let ((a, pa), (b, pb)) = (w[0], w[1]);
                    if v <= b {
                        return pa + (pb - pa) * (v - a) / (b - a);
                    }
                }
                1.0
            }
        }
    }

    /// The value at `place` (0–1), the inverse of [`place`](Self::place).
    pub fn value(self, place: f32, min: f32, max: f32) -> f32 {
        match self {
            Scale::Linear => min + (max - min) * place.clamp(0.0, 1.0),
            Scale::Table(t) => {
                for w in t.windows(2) {
                    let ((a, pa), (b, pb)) = (w[0], w[1]);
                    if place <= pb {
                        return a + (b - a) * ((place - pa) / (pb - pa)).max(0.0);
                    }
                }
                t[t.len() - 1].0
            }
        }
    }
}

/// Radius 0.1–1000 (Gaussian Blur, High Pass, Unsharp Mask's radius).
pub const RADIUS_1000: Scale = Scale::Table(&[
    (0.1, 0.0),
    (1.0, 0.048),
    (5.0, 0.27),
    (10.0, 0.402),
    (25.0, 0.455),
    (50.0, 0.55),
    (100.0, 0.64),
    (250.0, 0.831),
    (500.0, 0.889),
    (1000.0, 1.0),
]);
/// 1–2000 (Box Blur's radius, Motion Blur's distance).
pub const RANGE_2000: Scale = Scale::Table(&[
    (1.0, 0.0),
    (10.0, 0.048),
    (50.0, 0.27),
    (100.0, 0.354),
    (500.0, 0.72),
    (1000.0, 0.81),
    (2000.0, 1.0),
]);
/// 1–500 (Median, Minimum, Maximum, Dust & Scratches radius; Unsharp Mask
/// and Emboss amount).
pub const RANGE_500: Scale = Scale::Table(&[
    (1.0, 0.0),
    (5.0, 0.021),
    (10.0, 0.048),
    (50.0, 0.27),
    (100.0, 0.54),
    (200.0, 0.814),
    (250.0, 0.868),
    (300.0, 0.912),
    (400.0, 0.968),
    (500.0, 1.0),
]);
/// Thresholds 0–255 (Unsharp Mask, Dust & Scratches, Trace Contour's
/// level).
pub const LEVELS_255: Scale = Scale::Table(&[
    (0.0, 0.0),
    (10.0, 0.056),
    (50.0, 0.274),
    (100.0, 0.547),
    (150.0, 0.82),
    (200.0, 0.958),
    (255.0, 1.0),
]);

pub struct Layout {
    pub size: (f32, f32),
    /// OK and Cancel are 59.5 pt wide, or 80 in a few dialogs.
    pub button_width: f32,
    /// The Preview checkbox's top.
    pub preview_y: f32,
    /// Whether the dialog shows a preview pane (Offset doesn't).
    pub pane: bool,
    pub rows: &'static [Row],
}

const fn number(
    label: &'static str,
    label_right: f32,
    field: [f32; 4],
    unit: &'static str,
    unit_x: f32,
    track: Option<(f32, f32, f32)>,
    scale: Scale,
) -> Row {
    Row::Number {
        label,
        label_right,
        field,
        unit,
        unit_x,
        track,
        scale,
        dial: None,
    }
}

/// The preview pane and its zoom controls.
pub const PANE: [f32; 4] = [16.0, 43.0, 212.0, 239.0];
pub const ZOOM_Y: f32 = 261.75;

const RADIUS_TALL: Layout = Layout {
    size: (324.0, 335.0),
    button_width: 80.0,
    preview_y: 111.5,
    pane: true,
    rows: &[number(
        "Radius:",
        60.0,
        [65.0, 280.0, 125.5, 299.0],
        "Pixels",
        134.5,
        Some((21.0, 207.0, 310.0)),
        RADIUS_1000,
    )],
};

const RADIUS: Layout = Layout {
    size: (324.0, 342.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[number(
        "Radius:",
        60.0,
        [65.0, 287.0, 125.5, 306.0],
        "Pixels",
        134.5,
        Some((21.0, 207.0, 317.0)),
        RANGE_500,
    )],
};

const BOX: Layout = Layout {
    size: (324.0, 342.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[number(
        "Radius:",
        60.0,
        [65.0, 287.0, 125.5, 306.0],
        "Pixels",
        134.5,
        Some((21.0, 207.0, 317.0)),
        RANGE_2000,
    )],
};

const RANK: Layout = Layout {
    size: (324.0, 378.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Radius:",
            60.0,
            [65.0, 287.0, 125.5, 306.0],
            "Pixels",
            134.5,
            Some((21.0, 207.0, 317.0)),
            RANGE_500,
        ),
        Row::Popup {
            label: "Preserve:",
            label_right: 62.5,
            rect: [68.5, 347.0, 159.0, 368.0],
        },
    ],
};

pub const GAUSSIAN_BLUR: &Layout = &RADIUS_TALL;
pub const HIGH_PASS: &Layout = &RADIUS_TALL;
pub const BOX_BLUR: &Layout = &BOX;
pub const MEDIAN: &Layout = &RADIUS;
pub const MINIMUM: &Layout = &RANK;
pub const MAXIMUM: &Layout = &RANK;

pub const UNSHARP_MASK: &Layout = &Layout {
    size: (324.0, 436.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Amount:",
            78.0,
            [83.0, 287.0, 143.5, 306.0],
            "%",
            148.0,
            Some((21.0, 303.0, 313.0)),
            RANGE_500,
        ),
        number(
            "Radius:",
            78.0,
            [83.0, 336.0, 143.5, 355.0],
            "Pixels",
            148.5,
            Some((21.0, 303.0, 362.0)),
            RADIUS_1000,
        ),
        number(
            "Threshold:",
            78.0,
            [83.0, 385.0, 143.5, 404.0],
            "levels",
            148.5,
            Some((21.0, 303.0, 411.0)),
            LEVELS_255,
        ),
    ],
};

pub const ADD_NOISE: &Layout = &Layout {
    size: (324.0, 452.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Amount:",
            66.0,
            [71.0, 287.0, 131.5, 306.0],
            "%",
            136.0,
            Some((21.0, 188.0, 313.0)),
            Scale::Linear,
        ),
        Row::Radios {
            title: "Distribution",
            group: [10.5, 344.5, 314.0, 413.5],
            ys: &[369.5, 396.5],
        },
        Row::Check {
            label: "Monochromatic",
            min: (10.0, 426.0),
        },
    ],
};

pub const MOSAIC: &Layout = &Layout {
    size: (324.0, 338.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[number(
        "Cell Size:",
        145.5,
        [150.5, 287.0, 196.0, 306.0],
        "square",
        200.5,
        Some((21.0, 188.0, 313.0)),
        Scale::Linear,
    )],
};

pub const MOTION_BLUR: &Layout = &Layout {
    size: (324.0, 389.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        Row::Number {
            label: "Angle:",
            label_right: 71.5,
            field: [76.5, 295.5, 137.0, 314.5],
            unit: "°",
            unit_x: 0.0,
            track: None,
            scale: Scale::Linear,
            dial: Some((178.25, 304.75, 17.25)),
        },
        number(
            "Distance:",
            141.0,
            [146.5, 338.0, 207.0, 357.0],
            "Pixels",
            212.0,
            Some((21.0, 188.0, 364.0)),
            RANGE_2000,
        ),
    ],
};

pub const EMBOSS: &Layout = &Layout {
    size: (324.0, 431.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        Row::Number {
            label: "Angle:",
            label_right: 66.0,
            field: [71.0, 295.5, 116.5, 314.5],
            unit: "°",
            unit_x: 0.0,
            track: None,
            scale: Scale::Linear,
            dial: Some((152.0, 304.75, 17.25)),
        },
        number(
            "Height:",
            66.0,
            [71.0, 331.0, 116.5, 350.0],
            "Pixels",
            121.5,
            Some((21.0, 303.0, 357.0)),
            Scale::Linear,
        ),
        number(
            "Amount:",
            66.0,
            [71.0, 380.0, 116.5, 399.0],
            "%",
            121.0,
            Some((21.0, 303.0, 406.0)),
            RANGE_500,
        ),
    ],
};

pub const SURFACE_BLUR: &Layout = &Layout {
    size: (324.0, 395.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Radius:",
            78.0,
            [83.0, 287.0, 143.5, 306.0],
            "Pixels",
            152.5,
            Some((21.0, 207.0, 317.0)),
            Scale::Linear,
        ),
        number(
            "Threshold:",
            78.0,
            [83.0, 340.0, 143.5, 359.0],
            "levels",
            152.5,
            Some((21.0, 207.0, 370.0)),
            Scale::Linear,
        ),
    ],
};

pub const DUST_AND_SCRATCHES: &Layout = &Layout {
    size: (324.0, 387.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Radius:",
            78.0,
            [83.0, 287.0, 128.5, 306.0],
            "Pixels",
            133.5,
            Some((21.0, 303.0, 313.0)),
            RANGE_500,
        ),
        number(
            "Threshold:",
            78.0,
            [83.0, 336.0, 128.5, 355.0],
            "levels",
            133.5,
            Some((21.0, 303.0, 362.0)),
            LEVELS_255,
        ),
    ],
};

pub const TRACE_CONTOUR: &Layout = &Layout {
    size: (324.0, 425.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Level:",
            52.0,
            [57.0, 287.0, 102.5, 306.0],
            "",
            0.0,
            Some((21.0, 198.0, 313.0)),
            LEVELS_255,
        ),
        Row::Radios {
            title: "Edge",
            group: [10.5, 344.5, 314.0, 413.5],
            ys: &[369.5, 396.5],
        },
    ],
};

/// Stylize > Diffuse: the Mode radios under the preview (324 × 430 pt).
pub const DIFFUSE: &Layout = &Layout {
    size: (324.0, 430.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[Row::Radios {
        title: "Mode",
        group: [10.5, 295.5, 314.0, 419.0],
        ys: &[320.0, 347.0, 374.0, 401.0],
    }],
};

pub const OFFSET: &Layout = &Layout {
    size: (323.0, 256.0),
    button_width: 60.0,
    preview_y: 118.5,
    pane: false,
    rows: &[
        number(
            "Horizontal:",
            80.0,
            [85.0, 38.0, 146.0, 57.0],
            "pixels right",
            152.0,
            Some((21.0, 206.0, 64.0)),
            Scale::Linear,
        ),
        number(
            "Vertical:",
            80.0,
            [85.0, 87.0, 146.0, 106.0],
            "pixels down",
            152.0,
            Some((21.0, 206.0, 113.0)),
            Scale::Linear,
        ),
        Row::Radios {
            title: "Undefined Areas",
            group: [10.5, 146.5, 314.0, 243.5],
            ys: &[172.0, 199.0, 225.5],
        },
    ],
};

pub const SMART_BLUR: &Layout = &Layout {
    size: (324.0, 454.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Radius:",
            135.0,
            [140.0, 287.0, 200.5, 306.0],
            "",
            205.5,
            Some((21.0, 303.0, 313.0)),
            Scale::Linear,
        ),
        number(
            "Threshold:",
            135.0,
            [140.0, 336.0, 200.5, 355.0],
            "",
            205.5,
            Some((21.0, 303.0, 362.0)),
            Scale::Linear,
        ),
        Row::Popup {
            label: "Quality:",
            label_right: 135.0,
            rect: [140.0, 385.0, 300.0, 406.0],
        },
        Row::Popup {
            label: "Mode:",
            label_right: 135.0,
            rect: [140.0, 418.0, 300.0, 439.0],
        },
    ],
};

pub const SHAPE_BLUR: &Layout = &Layout {
    size: (324.0, 372.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Radius:",
            135.0,
            [140.0, 287.0, 200.5, 306.0],
            "Pixels",
            205.5,
            Some((21.0, 303.0, 313.0)),
            Scale::Linear,
        ),
        Row::Popup {
            label: "Shape:",
            label_right: 135.0,
            rect: [140.0, 336.0, 300.0, 357.0],
        },
    ],
};

pub const REDUCE_NOISE: &Layout = &Layout {
    size: (324.0, 513.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Strength:",
            135.0,
            [140.0, 287.0, 200.5, 306.0],
            "",
            205.5,
            Some((21.0, 303.0, 313.0)),
            Scale::Linear,
        ),
        number(
            "Preserve Details:",
            135.0,
            [140.0, 336.0, 200.5, 355.0],
            "%",
            205.5,
            Some((21.0, 303.0, 362.0)),
            Scale::Linear,
        ),
        number(
            "Reduce Color Noise:",
            135.0,
            [140.0, 385.0, 200.5, 404.0],
            "%",
            205.5,
            Some((21.0, 303.0, 411.0)),
            Scale::Linear,
        ),
        number(
            "Sharpen Details:",
            135.0,
            [140.0, 434.0, 200.5, 453.0],
            "%",
            205.5,
            Some((21.0, 303.0, 460.0)),
            Scale::Linear,
        ),
        Row::Check {
            label: "Remove JPEG Artifact",
            min: (10.0, 483.0),
        },
    ],
};

pub const SMART_SHARPEN: &Layout = &Layout {
    size: (324.0, 519.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Amount:",
            135.0,
            [140.0, 287.0, 200.5, 306.0],
            "%",
            205.5,
            Some((21.0, 303.0, 313.0)),
            Scale::Linear,
        ),
        number(
            "Radius:",
            135.0,
            [140.0, 336.0, 200.5, 355.0],
            "px",
            205.5,
            Some((21.0, 303.0, 362.0)),
            Scale::Linear,
        ),
        number(
            "Reduce Noise:",
            135.0,
            [140.0, 385.0, 200.5, 404.0],
            "%",
            205.5,
            Some((21.0, 303.0, 411.0)),
            Scale::Linear,
        ),
        Row::Popup {
            label: "Remove:",
            label_right: 135.0,
            rect: [140.0, 434.0, 300.0, 455.0],
        },
        number(
            "Angle:",
            135.0,
            [140.0, 467.0, 200.5, 486.0],
            "°",
            205.5,
            Some((21.0, 303.0, 493.0)),
            Scale::Linear,
        ),
    ],
};

pub const OIL_PAINT: &Layout = &Layout {
    size: (324.0, 584.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: true,
    rows: &[
        number(
            "Stylization:",
            135.0,
            [140.0, 287.0, 200.5, 306.0],
            "",
            205.5,
            Some((21.0, 303.0, 313.0)),
            Scale::Linear,
        ),
        number(
            "Cleanliness:",
            135.0,
            [140.0, 336.0, 200.5, 355.0],
            "",
            205.5,
            Some((21.0, 303.0, 362.0)),
            Scale::Linear,
        ),
        number(
            "Scale:",
            135.0,
            [140.0, 385.0, 200.5, 404.0],
            "",
            205.5,
            Some((21.0, 303.0, 411.0)),
            Scale::Linear,
        ),
        number(
            "Bristle Detail:",
            135.0,
            [140.0, 434.0, 200.5, 453.0],
            "",
            205.5,
            Some((21.0, 303.0, 460.0)),
            Scale::Linear,
        ),
        number(
            "Angle:",
            135.0,
            [140.0, 483.0, 200.5, 502.0],
            "°",
            205.5,
            Some((21.0, 303.0, 509.0)),
            Scale::Linear,
        ),
        number(
            "Shine:",
            135.0,
            [140.0, 532.0, 200.5, 551.0],
            "",
            205.5,
            Some((21.0, 303.0, 558.0)),
            Scale::Linear,
        ),
    ],
};

pub const SHADOWS_HIGHLIGHTS: &Layout = &Layout {
    size: (324.0, 560.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: false,
    rows: &[
        number(
            "Shadows Amount:",
            135.0,
            [140.0, 38.0, 200.5, 57.0],
            "%",
            205.5,
            Some((21.0, 206.0, 64.0)),
            Scale::Linear,
        ),
        number(
            "Shadows Tone:",
            135.0,
            [140.0, 87.0, 200.5, 106.0],
            "%",
            205.5,
            Some((21.0, 206.0, 113.0)),
            Scale::Linear,
        ),
        number(
            "Shadows Radius:",
            135.0,
            [140.0, 136.0, 200.5, 155.0],
            "px",
            205.5,
            Some((21.0, 206.0, 162.0)),
            Scale::Linear,
        ),
        number(
            "Highlights Amount:",
            135.0,
            [140.0, 185.0, 200.5, 204.0],
            "%",
            205.5,
            Some((21.0, 206.0, 211.0)),
            Scale::Linear,
        ),
        number(
            "Highlights Tone:",
            135.0,
            [140.0, 234.0, 200.5, 253.0],
            "%",
            205.5,
            Some((21.0, 206.0, 260.0)),
            Scale::Linear,
        ),
        number(
            "Highlights Radius:",
            135.0,
            [140.0, 283.0, 200.5, 302.0],
            "px",
            205.5,
            Some((21.0, 206.0, 309.0)),
            Scale::Linear,
        ),
        number(
            "Color:",
            135.0,
            [140.0, 332.0, 200.5, 351.0],
            "",
            205.5,
            Some((21.0, 206.0, 358.0)),
            Scale::Linear,
        ),
        number(
            "Midtone:",
            135.0,
            [140.0, 381.0, 200.5, 400.0],
            "",
            205.5,
            Some((21.0, 206.0, 407.0)),
            Scale::Linear,
        ),
        number(
            "Black Clip:",
            135.0,
            [140.0, 430.0, 200.5, 449.0],
            "%",
            205.5,
            Some((21.0, 206.0, 456.0)),
            Scale::Linear,
        ),
        number(
            "White Clip:",
            135.0,
            [140.0, 479.0, 200.5, 498.0],
            "%",
            205.5,
            Some((21.0, 206.0, 505.0)),
            Scale::Linear,
        ),
        Row::Check {
            label: "Show More Options",
            min: (21.0, 530.0),
        },
    ],
};

/// Shadows/Highlights without Show More Options: the two amounts.
pub const SHADOWS_HIGHLIGHTS_SIMPLE: &Layout = &Layout {
    size: (324.0, 170.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: false,
    rows: &[
        number(
            "Shadows Amount:",
            135.0,
            [140.0, 38.0, 200.5, 57.0],
            "%",
            205.5,
            Some((21.0, 206.0, 64.0)),
            Scale::Linear,
        ),
        Row::Hidden,
        Row::Hidden,
        number(
            "Highlights Amount:",
            135.0,
            [140.0, 87.0, 200.5, 106.0],
            "%",
            205.5,
            Some((21.0, 206.0, 113.0)),
            Scale::Linear,
        ),
        Row::Hidden,
        Row::Hidden,
        Row::Hidden,
        Row::Hidden,
        Row::Hidden,
        Row::Hidden,
        Row::Check {
            label: "Show More Options",
            min: (21.0, 140.0),
        },
    ],
};

pub const HDR_TONING: &Layout = &Layout {
    size: (324.0, 515.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: false,
    rows: &[
        Row::Popup {
            label: "Method:",
            label_right: 135.0,
            rect: [140.0, 38.0, 230.0, 59.0],
        },
        number(
            "Radius:",
            135.0,
            [140.0, 71.0, 200.5, 90.0],
            "px",
            205.5,
            Some((21.0, 206.0, 97.0)),
            Scale::Linear,
        ),
        number(
            "Strength:",
            135.0,
            [140.0, 120.0, 200.5, 139.0],
            "",
            205.5,
            Some((21.0, 206.0, 146.0)),
            Scale::Linear,
        ),
        number(
            "Gamma:",
            135.0,
            [140.0, 169.0, 200.5, 188.0],
            "",
            205.5,
            Some((21.0, 206.0, 195.0)),
            Scale::Linear,
        ),
        number(
            "Exposure:",
            135.0,
            [140.0, 218.0, 200.5, 237.0],
            "",
            205.5,
            Some((21.0, 206.0, 244.0)),
            Scale::Linear,
        ),
        number(
            "Detail:",
            135.0,
            [140.0, 267.0, 200.5, 286.0],
            "%",
            205.5,
            Some((21.0, 206.0, 293.0)),
            Scale::Linear,
        ),
        number(
            "Shadow:",
            135.0,
            [140.0, 316.0, 200.5, 335.0],
            "%",
            205.5,
            Some((21.0, 206.0, 342.0)),
            Scale::Linear,
        ),
        number(
            "Highlight:",
            135.0,
            [140.0, 365.0, 200.5, 384.0],
            "%",
            205.5,
            Some((21.0, 206.0, 391.0)),
            Scale::Linear,
        ),
        number(
            "Vibrance:",
            135.0,
            [140.0, 414.0, 200.5, 433.0],
            "%",
            205.5,
            Some((21.0, 206.0, 440.0)),
            Scale::Linear,
        ),
        number(
            "Saturation:",
            135.0,
            [140.0, 463.0, 200.5, 482.0],
            "%",
            205.5,
            Some((21.0, 206.0, 489.0)),
            Scale::Linear,
        ),
    ],
};

/// Replace Color: Fuzziness, then the selection preview (drawn by the
/// dialog itself in `REPLACE_PREVIEW`), then the replacement.
pub const REPLACE_COLOR: &Layout = &Layout {
    size: (324.0, 410.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: false,
    rows: &[
        number(
            "Fuzziness:",
            135.0,
            [140.0, 38.0, 200.5, 57.0],
            "",
            205.5,
            Some((21.0, 206.0, 64.0)),
            Scale::Linear,
        ),
        number(
            "Hue:",
            135.0,
            [140.0, 260.0, 200.5, 279.0],
            "",
            205.5,
            Some((21.0, 206.0, 286.0)),
            Scale::Linear,
        ),
        number(
            "Saturation:",
            135.0,
            [140.0, 309.0, 200.5, 328.0],
            "",
            205.5,
            Some((21.0, 206.0, 335.0)),
            Scale::Linear,
        ),
        number(
            "Lightness:",
            135.0,
            [140.0, 358.0, 200.5, 377.0],
            "",
            205.5,
            Some((21.0, 206.0, 384.0)),
            Scale::Linear,
        ),
    ],
};

/// Replace Color's preview box, and the centers of its Selection and
/// Image radio buttons below it.
pub const REPLACE_PREVIEW: [f32; 4] = [21.0, 82.0, 206.0, 222.0];
pub const REPLACE_RADIOS: [(f32, f32); 2] = [(27.0, 238.0), (117.0, 238.0)];

pub const MATCH_COLOR: &Layout = &Layout {
    size: (324.0, 248.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: false,
    rows: &[
        number(
            "Luminance:",
            135.0,
            [140.0, 38.0, 200.5, 57.0],
            "",
            205.5,
            Some((21.0, 206.0, 64.0)),
            Scale::Linear,
        ),
        number(
            "Color Intensity:",
            135.0,
            [140.0, 87.0, 200.5, 106.0],
            "",
            205.5,
            Some((21.0, 206.0, 113.0)),
            Scale::Linear,
        ),
        number(
            "Fade:",
            135.0,
            [140.0, 136.0, 200.5, 155.0],
            "",
            205.5,
            Some((21.0, 206.0, 162.0)),
            Scale::Linear,
        ),
        Row::Check {
            label: "Neutralize",
            min: (10.0, 185.0),
        },
        Row::Popup {
            label: "Source:",
            label_right: 135.0,
            rect: [140.0, 212.0, 230.0, 233.0],
        },
    ],
};

pub const COLOR_LOOKUP: &Layout = &Layout {
    size: (324.0, 160.0),
    button_width: 59.5,
    preview_y: 118.5,
    pane: false,
    rows: &[Row::Popup {
        label: "3DLUT File:",
        label_right: 135.0,
        rect: [140.0, 38.0, 230.0, 59.0],
    }],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_match_photoshops_pins() {
        // Gaussian Blur's radius 1 sits 9 of 189 points along, 250 at 157
        assert!((RADIUS_1000.place(1.0, 0.1, 1000.0) * 189.0 - 9.0).abs() < 0.5);
        assert!((RADIUS_1000.place(250.0, 0.1, 1000.0) * 189.0 - 157.0).abs() < 0.5);
        // Add Noise is even from 0: 100% at 42.5 of 170
        assert!((Scale::Linear.place(100.0, 0.0, 400.0) * 170.0 - 42.5).abs() < 0.01);
        // place and value undo each other
        for scale in [RADIUS_1000, RANGE_2000, RANGE_500, LEVELS_255] {
            for t in [0.0, 0.1, 0.33, 0.5, 0.9, 1.0] {
                let v = scale.value(t, 0.0, 1.0);
                assert!((scale.place(v, 0.0, 1.0) - t).abs() < 1e-4, "{t}");
            }
        }
    }
}
