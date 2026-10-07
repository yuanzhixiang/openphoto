//! The adjustment presets Photoshop 2026 ships (Presets/Levels, Curves,
//! Hue and Saturation, Black and White, Channel Mixer, Exposure), as the
//! numbers its preset files hold. The dialogs list them in their Preset
//! menus, in this (Photoshop's) order.

/// Levels: per channel (RGB, Red, Green, Blue) input black, gamma × 100,
/// input white, output black, output white.
pub const LEVELS: &[(&str, [[i32; 5]; 4])] = &[
    (
        "Darker",
        [
            [15, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
    (
        "Increase Contrast 1",
        [
            [10, 100, 245, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
    (
        "Increase Contrast 2",
        [
            [20, 100, 235, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
    (
        "Increase Contrast 3",
        [
            [30, 100, 225, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
    (
        "Lighten Shadows",
        [
            [0, 160, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
    (
        "Lighter",
        [
            [0, 100, 230, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
    (
        "Midtones Brighter",
        [
            [0, 125, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
    (
        "Midtones Darker",
        [
            [0, 75, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
            [0, 100, 255, 0, 255],
        ],
    ),
];

/// Curves: per channel (RGB, Red, Green, Blue) the points (input, output).
/// A name and four channels' points.
pub type CurvesPreset = (&'static str, [&'static [(u8, u8)]; 4]);

pub const CURVES: &[CurvesPreset] = &[
    (
        "Color Negative",
        [
            &[(0, 0), (255, 255)],
            &[(33, 255), (119, 127), (185, 0)],
            &[(28, 255), (77, 127), (132, 0)],
            &[(25, 255), (60, 127), (108, 0)],
        ],
    ),
    (
        "Cross Process",
        [
            &[(0, 0), (255, 255)],
            &[(0, 0), (64, 40), (128, 125), (175, 190), (255, 255)],
            &[(0, 0), (64, 48), (97, 128), (190, 208), (255, 208)],
            &[(0, 0), (59, 24), (181, 223), (255, 255)],
        ],
    ),
    (
        "Darker",
        [
            &[(0, 0), (130, 101), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
        ],
    ),
    (
        "Increase Contrast",
        [
            &[(0, 0), (38, 17), (212, 231), (231, 250), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
        ],
    ),
    (
        "Lighter",
        [
            &[(0, 0), (103, 125), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
        ],
    ),
    (
        "Linear Contrast",
        [
            &[(0, 0), (78, 73), (177, 182), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
        ],
    ),
    (
        "Medium Contrast",
        [
            &[(0, 0), (73, 56), (163, 164), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
        ],
    ),
    (
        "Negative",
        [
            &[(0, 255), (255, 0)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
        ],
    ),
    (
        "Strong Contrast",
        [
            &[(0, 0), (77, 50), (151, 153), (175, 188), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
            &[(0, 0), (255, 255)],
        ],
    ),
];

/// Hue/Saturation: the Master hue, saturation and lightness, and the
/// Colorize values when the preset colorizes (its six ranges are all at
/// their defaults and untouched).
/// A name, Master's values and Colorize's.
pub type HueSaturationPreset = (&'static str, [i32; 3], Option<[i32; 3]>);

pub const HUE_SATURATION: &[HueSaturationPreset] = &[
    ("Cyanotype", [0, 0, 0], Some([215, 25, 0])),
    ("Increase Saturation", [0, 10, 0], None),
    ("Increase Saturation More", [0, 30, 0], None),
    ("Old Style", [0, -40, 5], None),
    ("Red Boost", [-5, 20, 0], None),
    ("Sepia", [0, 0, 0], Some([35, 25, 0])),
    ("Strong Saturation", [0, 50, 0], None),
    ("Yellow Boost", [5, 20, 0], None),
];

/// Black & White: Reds, Yellows, Greens, Cyans, Blues, Magentas (no
/// preset tints).
pub const BLACK_WHITE: &[(&str, [i32; 6])] = &[
    ("Blue Filter", [0, 0, 0, 110, 110, 110]),
    ("Darker", [30, 50, 30, 50, 10, 70]),
    ("Green Filter", [50, 120, 90, 50, 0, 0]),
    ("High Contrast Blue Filter", [-50, -50, -50, 150, 150, 150]),
    ("High Contrast Red Filter", [120, 120, -10, -50, -50, 120]),
    ("Infrared", [-40, 235, 144, -68, -3, -107]),
    ("Lighter", [50, 70, 50, 70, 30, 90]),
    ("Maximum Black", [0, 0, 0, 0, 0, 0]),
    ("Maximum White", [100, 100, 100, 100, 100, 100]),
    ("Neutral Density", [128, 128, 100, 100, 128, 100]),
    ("Red Filter", [120, 110, -10, -50, 0, 120]),
    ("Yellow Filter", [120, 110, 40, -30, 0, 70]),
];

/// Channel Mixer: monochrome presets, the gray output's Red, Green and Blue
/// percentages.
pub const CHANNEL_MIXER: &[(&str, [i32; 3])] = &[
    ("Black & White Infrared", [-70, 200, -30]),
    ("Black & White with Blue Filter", [0, 0, 100]),
    ("Black & White with Green Filter", [0, 100, 0]),
    ("Black & White with Orange Filter", [50, 50, 0]),
    ("Black & White with Red Filter", [100, 0, 0]),
    ("Black & White with Yellow Filter", [34, 66, 0]),
];

/// Exposure: exposure, offset, gamma correction.
pub const EXPOSURE: &[(&str, [f32; 3])] = &[
    ("Minus 1.0", [-1e+00, 0e+00, 1e+00]),
    ("Minus 2.0", [-2e+00, 0e+00, 1e+00]),
    ("Plus 1.0", [1e+00, 0e+00, 1e+00]),
    ("Plus 2.0", [2e+00, 0e+00, 1e+00]),
];
