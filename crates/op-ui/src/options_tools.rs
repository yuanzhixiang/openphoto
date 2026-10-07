//! Tools' options bars as tables of controls at Photoshop 2026's measured
//! places (bar points, see `options_kit`). Each control names a setting:
//! the ones the app uses are bound to its state (the brushes' size,
//! hardness, opacity and flow, the toning ranges, Clone Stamp's Aligned,
//! …); the rest live in `AppState::tool_settings`.

use op_tools::Tool;

use crate::options_kit::Bar;
use crate::ps_icons::Icon;
use crate::state::AppState;

/// One control of a bar.
#[derive(Clone, Copy)]
pub enum Item {
    Sep(f32),
    Label(f32, &'static str),
    /// The brush preset picker centered at x.
    Picker(f32),
    /// An icon button centered at x (a command or panel nothing opens yet).
    Icon(f32, Icon, &'static str),
    /// A toggle icon centered at x, its state under the key.
    Toggle(f32, Icon, &'static str, &'static str),
    /// A pop-up menu from x0 to x1.
    Popup(f32, f32, &'static str, &'static [&'static str]),
    /// A text field from x0 to x1 with its default text.
    Field(f32, f32, &'static str, &'static str),
    /// A percentage: the field from x0 to x1, its slider's chevron box to
    /// x2.
    Percent(f32, f32, f32, &'static str, &'static str),
    /// A checkbox at x and whether it starts on.
    Check(f32, &'static str, &'static str, bool),
    /// A checkbox at x that can't be used (shown off).
    CheckOff(f32, &'static str),
    /// A swatch from x0 to x1 with a chevron at x2 (white: the Mixer
    /// Brush's load color; gray: the Pattern Stamp's pattern).
    Swatch(f32, f32, f32, bool),
    /// Segmented buttons split at the edges, choosing a setting.
    Segments(&'static [f32], &'static [&'static str], &'static str),
    /// An icon among several choosing a setting: centered at x, the
    /// setting's value when it's pressed.
    Radio(f32, Icon, &'static str, &'static str, usize),
    /// A push button from x0 to x1, usable or not.
    Button(f32, f32, &'static str, bool),
    /// An unusable pop-up showing its text.
    PopupOff(f32, f32, &'static str),
    /// An empty, unusable pattern box with its chevron box.
    NoPattern(f32, f32, f32),
    /// A color box from x0 to x1.
    ColorBox(f32, f32, [u8; 3]),
    /// The current gradient (foreground to background) from x0 to x1, its
    /// chevron box to x2.
    GradientSwatch(f32, f32, f32),
    /// The four selection combine modes from x, chosen by a setting.
    Modes(f32, &'static str),
}

use Item::*;

const BLEND: &[&str] = &[
    "Normal",
    "Dissolve",
    "Behind",
    "Clear",
    "Darken",
    "Multiply",
    "Color Burn",
    "Linear Burn",
    "Darker Color",
    "Lighten",
    "Screen",
    "Color Dodge",
    "Linear Dodge (Add)",
    "Lighter Color",
    "Overlay",
    "Soft Light",
    "Hard Light",
    "Vivid Light",
    "Linear Light",
    "Pin Light",
    "Hard Mix",
    "Difference",
    "Exclusion",
    "Subtract",
    "Divide",
    "Hue",
    "Saturation",
    "Color",
    "Luminosity",
];
const RANGES: &[&str] = &["Shadows", "Midtones", "Highlights"];
const SAMPLE: &[&str] = &["Current Layer", "Current & Below", "All Layers"];
const LIMITS: &[&str] = &["Discontiguous", "Contiguous", "Find Edges"];
const SAMPLE_SIZES: &[&str] = &[
    "Point Sample",
    "3 by 3 Average",
    "5 by 5 Average",
    "11 by 11 Average",
    "31 by 31 Average",
    "51 by 51 Average",
    "101 by 101 Average",
];
const EYEDROPPER_SAMPLE: &[&str] = &[
    "Current Layer",
    "Current & Below",
    "All Layers",
    "All Layers No Adjustments",
    "Current & Below No Adjustments",
];

/// The tools' bars, measured on Photoshop 2026.
pub fn layout(tool: Tool) -> Option<&'static [Item]> {
    use Tool::*;
    Some(match tool {
        Brush => &[
            Picker(124.0),
            Icon(171.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(190.0),
            Label(196.0, "Mode:"),
            Popup(230.5, 360.0, "brush.mode", BLEND),
            Sep(368.0),
            Label(378.5, "Opacity:"),
            Percent(422.5, 460.0, 474.5, "paint.opacity", "100%"),
            Toggle(
                494.75,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "brush.opacity_pressure",
            ),
            Sep(513.5),
            Label(520.5, "Flow:"),
            Percent(547.5, 585.0, 599.5, "paint.flow", "100%"),
            Toggle(
                619.25,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "brush.airbrush",
            ),
            Sep(638.5),
            Label(645.0, "Smoothing:"),
            Percent(702.5, 740.0, 754.5, "brush.smoothing", "10%"),
            Icon(772.5, Icon::Gear, "Set additional options for smoothing"),
            Sep(789.5),
            Icon(802.5, Icon::Angle, "Set the brush angle"),
            Field(813.5, 855.0, "brush.angle", "0°"),
            Sep(859.0),
            Toggle(
                879.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "brush.size_pressure",
            ),
            Icon(911.5, Icon::Symmetry, "Set paint symmetry options"),
        ],
        Pencil => &[
            Picker(124.0),
            Icon(171.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(190.0),
            Label(196.0, "Mode:"),
            Popup(230.5, 330.0, "pencil.mode", BLEND),
            Label(354.0, "Opacity:"),
            Percent(398.0, 435.5, 450.0, "paint.opacity", "100%"),
            Toggle(
                474.25,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "pencil.opacity_pressure",
            ),
            Sep(493.0),
            Label(499.5, "Smoothing:"),
            Percent(557.0, 594.5, 609.0, "pencil.smoothing", "10%"),
            Icon(627.0, Icon::Gear, "Set additional options for smoothing"),
            Sep(644.0),
            Icon(657.0, Icon::Angle, "Set the brush angle"),
            Field(668.0, 709.5, "pencil.angle", "0°"),
            Sep(713.5),
            Check(718.5, "Auto Erase", "pencil.auto_erase", false),
            Toggle(
                810.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "pencil.size_pressure",
            ),
            Icon(842.5, Icon::Symmetry, "Set paint symmetry options"),
        ],
        Eraser => &[
            Picker(124.0),
            Icon(171.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(190.0),
            Label(196.0, "Mode:"),
            Popup(230.5, 284.5, "eraser.mode", &["Brush", "Pencil", "Block"]),
            Label(289.5, "Opacity:"),
            Percent(332.5, 370.0, 384.5, "paint.opacity", "100%"),
            Toggle(
                404.75,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "eraser.opacity_pressure",
            ),
            Sep(423.5),
            Label(430.5, "Flow:"),
            Percent(457.5, 495.0, 509.5, "paint.flow", "100%"),
            Toggle(
                529.25,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "eraser.airbrush",
            ),
            Sep(548.5),
            Label(555.0, "Smoothing:"),
            Percent(612.5, 650.0, 664.5, "eraser.smoothing", "0%"),
            Icon(682.5, Icon::Gear, "Set additional options for smoothing"),
            Sep(699.5),
            Icon(712.5, Icon::Angle, "Set the brush angle"),
            Field(723.5, 765.0, "eraser.angle", "0°"),
            Sep(769.5),
            CheckOff(775.5, "Erase to History"),
            Toggle(
                890.75,
                Icon::PenPressure,
                "Always use pressure for size",
                "eraser.size_pressure",
            ),
            Icon(923.0, Icon::Symmetry, "Set paint symmetry options"),
        ],
        BackgroundEraser => &[
            Picker(124.0),
            Sep(160.0),
            Toggle(
                182.0,
                Icon::SampleContinuous,
                "Sampling: Continuous",
                "bgeraser.continuous",
            ),
            Icon(208.0, Icon::SampleOnce, "Sampling: Once"),
            Icon(234.5, Icon::SampleSwatch, "Sampling: Background Swatch"),
            Sep(255.0),
            Label(261.0, "Limits:"),
            Popup(297.0, 390.5, "bgeraser.limits", LIMITS),
            Sep(394.0),
            Label(400.5, "Tolerance:"),
            Percent(453.0, 490.5, 505.0, "bgeraser.tolerance", "50%"),
            Sep(514.0),
            Icon(531.0, Icon::Angle, "Set the brush angle"),
            Field(542.0, 583.5, "bgeraser.angle", "0°"),
            Sep(591.5),
            Check(600.5, "Protect Foreground Color", "bgeraser.protect", false),
            Toggle(
                766.75,
                Icon::PenPressure,
                "Always use pressure for size",
                "bgeraser.size_pressure",
            ),
        ],
        MagicEraser => &[
            Label(111.5, "Tolerance:"),
            Field(165.0, 200.5, "magiceraser.tolerance", "32"),
            Check(208.5, "Anti-alias", "magiceraser.anti_alias", true),
            Sep(282.5),
            Check(291.5, "Contiguous", "magiceraser.contiguous", true),
            Check(375.0, "Sample All Layers", "magiceraser.all_layers", false),
            Sep(488.5),
            Label(499.0, "Opacity:"),
            Percent(543.0, 580.5, 595.0, "magiceraser.opacity", "100%"),
            Sep(604.0),
        ],
        ColorReplacement => &[
            Picker(124.0),
            Sep(156.0),
            Label(162.0, "Mode:"),
            Popup(
                196.5,
                273.5,
                "colorreplace.mode",
                &["Hue", "Saturation", "Color", "Luminosity"],
            ),
            Sep(277.5),
            Toggle(
                295.5,
                Icon::SampleContinuous,
                "Sampling: Continuous",
                "colorreplace.continuous",
            ),
            Icon(321.5, Icon::SampleOnce, "Sampling: Once"),
            Icon(348.0, Icon::SampleSwatch, "Sampling: Background Swatch"),
            Label(365.5, "Limits:"),
            Popup(401.5, 494.5, "colorreplace.limits", LIMITS),
            Label(499.5, "Tolerance:"),
            Percent(552.5, 590.0, 604.5, "colorreplace.tolerance", "30%"),
            Check(609.5, "Anti-alias", "colorreplace.anti_alias", true),
            Sep(679.5),
            Icon(692.5, Icon::Angle, "Set the brush angle"),
            Field(703.5, 745.0, "colorreplace.angle", "0°"),
            Sep(749.0),
            Toggle(
                769.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "colorreplace.size_pressure",
            ),
        ],
        MixerBrush => &[
            Picker(124.0),
            Icon(169.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(186.0),
            Swatch(189.0, 227.0, 0.0, true),
            Toggle(
                245.5,
                Icon::MixerLoad,
                "Load the brush after each stroke",
                "mixer.load",
            ),
            Toggle(
                274.0,
                Icon::MixerClean,
                "Clean the brush after each stroke",
                "mixer.clean",
            ),
            Popup(
                289.0,
                419.0,
                "mixer.combo",
                &["Custom", "Dry", "Moist", "Wet", "Very Wet"],
            ),
            Label(422.5, "Wet:"),
            Percent(446.0, 483.5, 498.0, "mixer.wet", "80%"),
            Label(503.0, "Load:"),
            Percent(530.5, 568.0, 582.5, "mixer.load_amount", "75%"),
            Label(587.5, "Mix:"),
            Percent(608.5, 646.0, 660.5, "mixer.mix", "90%"),
            Label(665.5, "Flow:"),
            Percent(691.5, 729.0, 743.5, "mixer.flow", "100%"),
            Toggle(
                762.0,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "mixer.airbrush",
            ),
            Icon(788.0, Icon::Refresh, "Smoothing"),
            Percent(802.5, 840.0, 854.5, "mixer.smoothing", "10%"),
            Icon(870.5, Icon::Gear, "Set additional options for smoothing"),
            Icon(893.5, Icon::Angle, "Set the brush angle"),
            Field(904.5, 946.0, "mixer.angle", "0°"),
            Check(948.5, "Sample All Layers", "mixer.all_layers", false),
            Toggle(
                1071.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "mixer.size_pressure",
            ),
        ],
        CloneStamp => &[
            Picker(124.0),
            Icon(170.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Icon(
                203.0,
                Icon::CloneSourcePanel,
                "Toggle the Clone Source panel",
            ),
            Sep(221.0),
            Label(226.0, "Mode:"),
            Popup(260.5, 364.0, "clone.mode", BLEND),
            Label(368.5, "Opacity:"),
            Percent(411.5, 449.0, 463.5, "paint.opacity", "100%"),
            Toggle(
                482.75,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "clone.opacity_pressure",
            ),
            Sep(500.5),
            Label(506.5, "Flow:"),
            Percent(533.5, 571.0, 585.5, "paint.flow", "100%"),
            Toggle(
                604.25,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "clone.airbrush",
            ),
            Sep(622.5),
            Icon(634.5, Icon::Angle, "Set the brush angle"),
            Field(645.5, 687.0, "clone.angle", "0°"),
            Sep(690.5),
            Check(694.5, "Aligned", "clone.aligned", true),
            Sep(754.0),
            Label(758.5, "Sample:"),
            Popup(802.0, 904.5, "clone.sample", SAMPLE),
            Icon(
                922.5,
                Icon::IgnoreAdjustments,
                "Ignore adjustment layers when cloning",
            ),
            Sep(940.5),
            Toggle(
                959.75,
                Icon::PenPressure,
                "Always use pressure for size",
                "clone.size_pressure",
            ),
        ],
        PatternStamp => &[
            Picker(124.0),
            Icon(170.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(188.0),
            Label(193.0, "Mode:"),
            Popup(227.5, 346.5, "pattern.mode", BLEND),
            Label(350.5, "Opacity:"),
            Percent(393.5, 431.0, 445.5, "pattern.opacity", "100%"),
            Toggle(
                464.75,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "pattern.opacity_pressure",
            ),
            Sep(482.5),
            Label(488.5, "Flow:"),
            Percent(516.0, 553.5, 568.0, "pattern.flow", "100%"),
            Toggle(
                587.0,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "pattern.airbrush",
            ),
            Sep(605.0),
            Icon(617.0, Icon::Angle, "Set the brush angle"),
            Field(628.0, 669.5, "pattern.angle", "0°"),
            Sep(672.5),
            Swatch(676.5, 706.5, 718.5, false),
            Check(721.5, "Aligned", "pattern.aligned", true),
            Check(781.0, "Impressionist", "pattern.impressionist", false),
            Sep(869.0),
            Toggle(
                888.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "pattern.size_pressure",
            ),
        ],
        HistoryBrush => &[
            Picker(124.0),
            Icon(171.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(190.0),
            Label(196.0, "Mode:"),
            Popup(230.5, 330.0, "historybrush.mode", BLEND),
            Label(334.5, "Opacity:"),
            Percent(377.5, 415.0, 429.5, "paint.opacity", "100%"),
            Toggle(
                448.75,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "historybrush.opacity_pressure",
            ),
            Sep(467.5),
            Label(474.5, "Flow:"),
            Percent(501.5, 539.0, 553.5, "paint.flow", "100%"),
            Toggle(
                573.25,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "historybrush.airbrush",
            ),
            Sep(592.5),
            Icon(605.5, Icon::Angle, "Set the brush angle"),
            Field(616.5, 658.0, "historybrush.angle", "0°"),
            Sep(662.5),
            Toggle(
                682.75,
                Icon::PenPressure,
                "Always use pressure for size",
                "historybrush.size_pressure",
            ),
        ],
        ArtHistoryBrush => &[
            Picker(124.0),
            Icon(171.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(190.0),
            Label(196.0, "Mode:"),
            Popup(230.5, 330.0, "arthistory.mode", BLEND),
            Label(334.5, "Opacity:"),
            Percent(377.5, 415.0, 429.5, "arthistory.opacity", "100%"),
            Toggle(
                448.75,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "arthistory.opacity_pressure",
            ),
            Sep(467.5),
            Label(473.0, "Style:"),
            Popup(
                505.0,
                608.0,
                "arthistory.style",
                &[
                    "Tight Short",
                    "Tight Medium",
                    "Tight Long",
                    "Loose Medium",
                    "Loose Long",
                    "Dab",
                    "Tight Curl",
                    "Tight Curl Long",
                    "Loose Curl",
                    "Loose Curl Long",
                ],
            ),
            Label(613.0, "Area:"),
            Field(640.5, 686.0, "arthistory.area", "50 px"),
            Label(691.5, "Tolerance:"),
            Percent(744.0, 781.5, 796.0, "arthistory.tolerance", "0%"),
            Sep(801.0),
            Icon(814.0, Icon::Angle, "Set the brush angle"),
            Field(825.0, 866.5, "arthistory.angle", "0°"),
            Sep(871.0),
            Toggle(
                891.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "arthistory.size_pressure",
            ),
        ],
        Blur | Sharpen | Smudge => match tool {
            Blur => &[
                Picker(124.0),
                Icon(175.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
                Sep(198.0),
                Label(208.0, "Mode:"),
                Popup(
                    242.5,
                    342.0,
                    "blur.mode",
                    &[
                        "Normal",
                        "Darken",
                        "Lighten",
                        "Hue",
                        "Saturation",
                        "Color",
                        "Luminosity",
                    ],
                ),
                Label(361.5, "Strength:"),
                Percent(410.0, 447.5, 462.0, "paint.opacity", "50%"),
                Sep(471.0),
                Icon(488.0, Icon::Angle, "Set the brush angle"),
                Field(499.0, 540.5, "blur.angle", "0°"),
                Sep(548.5),
                Check(557.5, "Sample All Layers", "blur.all_layers", false),
                Toggle(
                    686.25,
                    Icon::PenPressure,
                    "Always use pressure for size",
                    "blur.size_pressure",
                ),
            ],
            Sharpen => &[
                Picker(124.0),
                Icon(175.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
                Sep(198.0),
                Label(208.0, "Mode:"),
                Popup(
                    242.5,
                    342.0,
                    "sharpen.mode",
                    &[
                        "Normal",
                        "Darken",
                        "Lighten",
                        "Hue",
                        "Saturation",
                        "Color",
                        "Luminosity",
                    ],
                ),
                Label(361.5, "Strength:"),
                Percent(410.0, 447.5, 462.0, "paint.opacity", "50%"),
                Sep(471.0),
                Icon(488.0, Icon::Angle, "Set the brush angle"),
                Field(499.0, 540.5, "sharpen.angle", "0°"),
                Sep(548.5),
                Check(557.5, "Sample All Layers", "sharpen.all_layers", false),
                Check(671.0, "Protect Detail", "sharpen.protect_detail", true),
                Sep(765.5),
                Toggle(
                    789.75,
                    Icon::PenPressure,
                    "Always use pressure for size",
                    "sharpen.size_pressure",
                ),
            ],
            _ => &[
                Picker(124.0),
                Icon(175.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
                Sep(198.0),
                Label(208.0, "Mode:"),
                Popup(
                    242.5,
                    342.0,
                    "smudge.mode",
                    &[
                        "Normal",
                        "Darken",
                        "Lighten",
                        "Hue",
                        "Saturation",
                        "Color",
                        "Luminosity",
                    ],
                ),
                Label(361.5, "Strength:"),
                Percent(410.0, 447.5, 462.0, "smudge.strength", "50%"),
                Sep(471.0),
                Icon(488.0, Icon::Angle, "Set the brush angle"),
                Field(499.0, 540.5, "smudge.angle", "0°"),
                Sep(548.5),
                Check(557.5, "Sample All Layers", "smudge.all_layers", false),
                Check(671.0, "Finger Painting", "smudge.finger", false),
                Toggle(
                    787.25,
                    Icon::PenPressure,
                    "Always use pressure for size",
                    "smudge.size_pressure",
                ),
            ],
        },
        Dodge => &[
            Picker(124.0),
            Icon(175.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(198.0),
            Label(208.0, "Range:"),
            Popup(245.5, 319.0, "dodge.range", RANGES),
            Label(329.0, "Exposure:"),
            Percent(379.5, 417.0, 431.5, "paint.opacity", "50%"),
            Toggle(
                455.5,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "dodge.airbrush",
            ),
            Sep(478.5),
            Icon(495.5, Icon::Angle, "Set the brush angle"),
            Field(506.5, 548.0, "dodge.angle", "0°"),
            Sep(556.0),
            Check(565.0, "Protect Tones", "dodge.protect", true),
            Sep(660.5),
            Toggle(
                684.75,
                Icon::PenPressure,
                "Always use pressure for size",
                "dodge.size_pressure",
            ),
        ],
        Burn => &[
            Picker(124.0),
            Icon(175.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(198.0),
            Label(208.0, "Range:"),
            Popup(245.5, 319.0, "burn.range", RANGES),
            Label(336.0, "Exposure:"),
            Percent(387.0, 424.5, 439.0, "paint.opacity", "50%"),
            Toggle(
                465.5,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "burn.airbrush",
            ),
            Sep(486.0),
            Icon(503.0, Icon::Angle, "Set the brush angle"),
            Field(514.0, 555.5, "burn.angle", "0°"),
            Sep(563.5),
            Check(572.5, "Protect Tones", "burn.protect", true),
            Sep(667.5),
            Toggle(
                691.75,
                Icon::PenPressure,
                "Always use pressure for size",
                "burn.size_pressure",
            ),
        ],
        Sponge => &[
            Picker(124.0),
            Icon(175.0, Icon::BrushPanel, "Toggle the Brush Settings panel"),
            Sep(198.0),
            Label(208.0, "Mode:"),
            Popup(242.5, 320.0, "sponge.mode", &["Desaturate", "Saturate"]),
            Label(330.0, "Flow:"),
            Percent(358.0, 395.5, 410.0, "paint.flow", "50%"),
            Toggle(
                436.5,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "sponge.airbrush",
            ),
            Sep(457.0),
            Icon(474.0, Icon::Angle, "Set the brush angle"),
            Field(485.0, 526.5, "sponge.angle", "0°"),
            Sep(535.0),
            Check(544.0, "Vibrance", "sponge.vibrance", true),
            Sep(615.0),
            Toggle(
                639.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "sponge.size_pressure",
            ),
        ],
        SpotHealingBrush => &[
            Picker(124.0),
            Sep(156.0),
            Label(162.0, "Mode:"),
            Popup(
                196.5,
                296.0,
                "spotheal.mode",
                &[
                    "Normal",
                    "Replace",
                    "Multiply",
                    "Screen",
                    "Darken",
                    "Lighten",
                    "Color",
                    "Luminosity",
                ],
            ),
            Sep(300.0),
            Label(306.5, "Type:"),
            Segments(
                &[336.5, 425.5, 513.5, 609.0],
                &["Content-Aware", "Create Texture", "Proximity Match"],
                "spotheal.type",
            ),
            Sep(613.0),
            Check(618.0, "Sample All Layers", "spotheal.all_layers", false),
            Sep(727.5),
            Icon(740.5, Icon::Angle, "Set the brush angle"),
            Field(751.5, 793.0, "spotheal.angle", "0°"),
            Sep(797.0),
            Toggle(
                817.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "spotheal.size_pressure",
            ),
        ],
        HealingBrush => &[
            Picker(124.0),
            Icon(
                171.0,
                Icon::CloneSourcePanel,
                "Toggle the Clone Source panel",
            ),
            Sep(190.0),
            Label(196.0, "Mode:"),
            Popup(
                230.5,
                330.0,
                "heal.mode",
                &[
                    "Normal",
                    "Replace",
                    "Multiply",
                    "Screen",
                    "Darken",
                    "Lighten",
                    "Color",
                    "Luminosity",
                ],
            ),
            Sep(334.0),
            Label(340.5, "Source:"),
            Segments(
                &[381.0, 440.0, 493.0],
                &["Sampled", "Pattern"],
                "heal.source",
            ),
            Sep(497.0),
            NoPattern(502.0, 532.0, 544.0),
            Sep(548.0),
            Check(553.0, "Aligned", "heal.aligned", false),
            Check(613.5, "Use Legacy", "heal.legacy", false),
            Label(694.5, "Sample:"),
            Popup(738.0, 840.5, "heal.sample", SAMPLE),
            Icon(
                859.5,
                Icon::IgnoreAdjustments,
                "Ignore adjustment layers when healing",
            ),
            Sep(878.5),
            Icon(891.5, Icon::Angle, "Set the brush angle"),
            Field(902.5, 944.0, "heal.angle", "0°"),
            Sep(948.0),
            Toggle(
                968.25,
                Icon::PenPressure,
                "Always use pressure for size",
                "heal.size_pressure",
            ),
            Sep(987.0),
            Label(998.0, "Diffusion:"),
            Percent(1051.5, 1080.0, 1094.5, "heal.diffusion", "5"),
        ],
        Patch => &[
            Modes(110.0, "patch.mode"),
            Sep(222.0),
            Label(232.0, "Patch:"),
            Popup(266.5, 363.5, "patch.patch", &["Normal", "Content-Aware"]),
            Sep(371.0),
            Segments(
                &[380.0, 431.0, 503.5],
                &["Source", "Destination"],
                "patch.source",
            ),
            Check(511.5, "Transparent", "patch.transparent", false),
            Sep(597.5),
            Button(607.5, 685.0, "Use Pattern", false),
            NoPattern(688.5, 718.5, 730.5),
            Sep(738.5),
            Label(749.5, "Diffusion:"),
            Percent(803.0, 831.5, 846.0, "patch.diffusion", "5"),
            Sep(855.0),
        ],
        ContentAwareMove => &[
            Modes(110.0, "cam.mode"),
            Sep(222.0),
            Label(232.0, "Mode:"),
            Popup(266.5, 324.5, "cam.move", &["Move", "Extend"]),
            Sep(332.0),
            Label(345.0, "Structure:"),
            Percent(401.5, 425.0, 439.5, "cam.structure", "4"),
            Label(445.0, "Color:"),
            Percent(481.5, 505.0, 519.5, "cam.color", "0"),
            Sep(528.5),
            Check(537.5, "Sample All Layers", "cam.all_layers", false),
            Check(651.0, "Transform On Drop", "cam.transform", true),
        ],
        RedEye => &[
            Label(112.0, "Pupil Size:"),
            Percent(164.0, 201.5, 216.0, "redeye.pupil", "50%"),
            Label(223.0, "Darken Amount:"),
            Percent(302.5, 340.0, 354.5, "redeye.darken", "50%"),
            Sep(359.5),
        ],
        Gradient => &[
            Popup(
                115.0,
                222.0,
                "gradient.style",
                &["Gradient", "Classic gradient"],
            ),
            Sep(230.0),
            GradientSwatch(241.0, 314.0, 328.0),
            Sep(338.0),
            Radio(
                362.0,
                Icon::GradientLinear,
                "Linear Gradient",
                "gradient.kind",
                0,
            ),
            Radio(
                388.0,
                Icon::GradientRadial,
                "Radial Gradient",
                "gradient.kind",
                1,
            ),
            Radio(
                414.0,
                Icon::GradientAngle,
                "Angle Gradient",
                "gradient.kind",
                2,
            ),
            Radio(
                440.0,
                Icon::GradientReflected,
                "Reflected Gradient",
                "gradient.kind",
                3,
            ),
            Radio(
                466.0,
                Icon::GradientDiamond,
                "Diamond Gradient",
                "gradient.kind",
                4,
            ),
            Sep(489.0),
            Check(500.0, "Reverse", "gradient.reverse", false),
            Check(569.0, "Dither", "gradient.dither", true),
            Sep(629.0),
            Label(641.0, "Method:"),
            Popup(
                685.0,
                761.0,
                "gradient.method",
                &["Perceptual", "Linear", "Classic", "Smooth"],
            ),
        ],
        PaintBucket => &[
            Popup(110.0, 191.0, "bucket.source", &["Foreground", "Pattern"]),
            NoPattern(193.5, 223.5, 235.5),
            Label(239.5, "Mode:"),
            Popup(274.0, 373.0, "bucket.mode", BLEND),
            Label(377.0, "Opacity:"),
            Percent(419.0, 456.5, 471.0, "bucket.opacity", "100%"),
            Label(476.5, "Tolerance:"),
            Field(530.0, 565.0, "bucket.tolerance", "32"),
            Sep(568.5),
            Check(572.5, "Anti-alias", "bucket.anti_alias", true),
            Check(641.5, "Contiguous", "bucket.contiguous", true),
            Check(720.0, "All Layers", "bucket.all_layers", false),
        ],
        Eyedropper => &[
            Label(110.5, "Sample Size:"),
            Popup(177.0, 292.5, "eyedropper.size", SAMPLE_SIZES),
            Label(300.5, "Sample:"),
            Popup(344.0, 525.0, "eyedropper.sample", EYEDROPPER_SAMPLE),
            Sep(532.5),
            Check(541.5, "Show Sampling Ring", "eyedropper.ring", true),
        ],
        ColorSampler => &[
            Label(110.5, "Sample Size:"),
            Popup(177.0, 292.5, "sampler.size", SAMPLE_SIZES),
            Sep(300.0),
            Button(310.0, 371.5, "Clear All", false),
            Sep(379.5),
        ],
        Ruler => &[
            Label(111.5, "X: 0.00"),
            Label(172.0, "Y: 0.00"),
            Sep(230.0),
            Label(234.5, "W: 0.00"),
            Label(297.5, "H: 0.00"),
            Sep(356.5),
            Label(361.0, "A: 0.0°"),
            Label(422.0, "L1: 0.00"),
            Label(485.5, "L2:"),
            Sep(549.0),
            Check(554.0, "Use Measurement Scale", "ruler.scale", true),
            Sep(694.5),
            Button(702.0, 802.5, "Straighten Layer", false),
            Button(811.5, 858.0, "Clear", false),
        ],
        Note => &[
            Label(111.5, "Author:"),
            Field(150.5, 325.0, "note.author", ""),
            Sep(333.0),
            Label(343.5, "Color:"),
            ColorBox(376.5, 403.0, [255, 255, 255]),
            Sep(411.5),
            Button(421.5, 483.0, "Clear All", false),
            Sep(491.0),
            Icon(515.0, Icon::NotesPanel, "Toggle the Notes panel"),
        ],
        Count => &[
            Label(110.5, "Count:"),
            Label(151.5, "0"),
            Sep(223.0),
            PopupOff(232.0, 382.0, "Count Group"),
            Icon(403.0, Icon::Eye, "Toggle count group visibility"),
            Icon(431.0, Icon::Folder, "Create a new count group"),
            Icon(459.0, Icon::DeleteLayer, "Delete the current count group"),
            Button(480.5, 533.5, "Clear", false),
            Sep(542.0),
            ColorBox(552.0, 578.0, [0x9c, 0xf2, 0xf4]),
            Label(589.0, "Marker Size:"),
            Field(652.0, 691.5, "count.marker", "2"),
            Label(701.5, "Label Size:"),
            Field(757.0, 796.5, "count.label", "8"),
        ],
        _ => return None,
    })
}

/// A percentage's value as shown ("50%").
fn percent(v: f32) -> String {
    format!("{}%", (v * 100.0).round())
}

/// The text a setting shows.
fn text(app: &mut AppState, key: &'static str, default: &str) -> String {
    let tool = app.tool;
    match key {
        "paint.opacity" => app.paint_options(tool).map(|o| percent(o.opacity)),
        "paint.flow" => app.paint_options(tool).map(|o| percent(o.flow)),
        "bucket.opacity" => Some(percent(app.bucket.fill.opacity)),
        "bucket.tolerance" => Some(app.bucket.tolerance.to_string()),
        _ => None,
    }
    .unwrap_or_else(|| app.setting(key, default).clone())
}

/// Stores typed text: percentages and numbers are kept in range.
fn set_text(app: &mut AppState, key: &'static str, default: &str, typed: String) {
    let tool = app.tool;
    let number = crate::options_bar::typed_number(&typed);
    match key {
        "paint.opacity" | "paint.flow" => {
            if let (Some(o), Some(v)) = (app.paint_options(tool), number) {
                let v = (v / 100.0).clamp(0.01, 1.0);
                if key == "paint.opacity" {
                    o.opacity = v;
                } else {
                    o.flow = v;
                }
            }
        }
        "bucket.opacity" => {
            if let Some(v) = number {
                app.bucket.fill.opacity = (v / 100.0).clamp(0.01, 1.0);
            }
        }
        "bucket.tolerance" => {
            if let Some(v) = number {
                app.bucket.tolerance = v.clamp(0.0, 255.0) as u8;
            }
        }
        _ => {
            if default.ends_with('%') {
                if let Some(v) = number {
                    *app.setting(key, default) = format!("{}%", v.clamp(0.0, 100.0).round());
                }
            } else {
                *app.setting(key, default) = typed;
            }
        }
    }
}

fn choice(app: &mut AppState, key: &'static str) -> usize {
    use op_core::paint::ToneRange;
    let range = |r: ToneRange| ToneRange::ALL.iter().position(|x| *x == r).unwrap_or(1);
    match key {
        "dodge.range" => range(app.retouch.dodge_range),
        "burn.range" => range(app.retouch.burn_range),
        "sponge.mode" => app.retouch.sponge_saturate as usize,
        "gradient.kind" => op_core::gradient::GradientKind::ALL
            .iter()
            .position(|k| *k == app.gradient.kind)
            .unwrap_or(0),
        "bucket.mode" => BLEND
            .iter()
            .position(|l| *l == app.bucket.fill.mode.label())
            .unwrap_or(0),
        "eyedropper.size" => crate::state::EyedropperOptions::SIZES
            .iter()
            .position(|(s, _)| *s == app.eyedropper.size)
            .unwrap_or(0),
        "eyedropper.sample" => {
            let all = app.eyedropper.all_layers;
            let shown: usize = app.setting(key, "2").parse().unwrap_or(2);
            // The setting's own choice, as long as it agrees with the state
            if (shown >= 2 && shown != 4) == all {
                shown
            } else if all {
                2
            } else {
                0
            }
        }
        _ => app.setting(key, default_choice(key)).parse().unwrap_or(0),
    }
}

/// Settings that don't start at their first option (Photoshop's
/// defaults).
fn default_choice(key: &str) -> &'static str {
    match key {
        "colorreplace.mode" => "2",
        "colorreplace.limits" | "bgeraser.limits" => "1",
        "gradient.method" => "3",
        _ => "0",
    }
}

fn set_choice(app: &mut AppState, key: &'static str, i: usize) {
    use op_core::paint::ToneRange;
    match key {
        "dodge.range" => app.retouch.dodge_range = ToneRange::ALL[i],
        "burn.range" => app.retouch.burn_range = ToneRange::ALL[i],
        "sponge.mode" => app.retouch.sponge_saturate = i == 1,
        "gradient.kind" => app.gradient.kind = op_core::gradient::GradientKind::ALL[i],
        "bucket.mode" => {
            if let Some(m) = op_core::BlendMode::GROUPS
                .iter()
                .flat_map(|g| g.iter())
                .find(|m| m.label() == BLEND[i])
            {
                app.bucket.fill.mode = *m;
            }
        }
        "eyedropper.size" => app.eyedropper.size = crate::state::EyedropperOptions::SIZES[i].0,
        "eyedropper.sample" => {
            app.eyedropper.all_layers = i >= 2 && i != 4;
            *app.setting(key, "2") = i.to_string();
        }
        _ => *app.setting(key, default_choice(key)) = i.to_string(),
    }
}

fn flag(app: &mut AppState, key: &'static str, default: bool) -> bool {
    match key {
        "clone.aligned" => app.retouch.clone_aligned,
        "gradient.reverse" => app.gradient.reverse,
        "bucket.anti_alias" => app.bucket.anti_alias,
        "bucket.contiguous" => app.bucket.contiguous,
        "bucket.all_layers" => app.bucket.all_layers,
        _ => app.flag(key, default),
    }
}

fn set_flag(app: &mut AppState, key: &'static str, on: bool) {
    match key {
        "clone.aligned" => app.retouch.clone_aligned = on,
        "gradient.reverse" => app.gradient.reverse = on,
        "bucket.anti_alias" => app.bucket.anti_alias = on,
        "bucket.contiguous" => app.bucket.contiguous = on,
        "bucket.all_layers" => app.bucket.all_layers = on,
        _ => app.set_flag(key, on),
    }
}

/// Lays out `items` for the current tool.
pub fn show(b: &mut Bar, app: &mut AppState, items: &[Item]) {
    for &item in items {
        match item {
            Sep(x) => b.sep(x),
            Label(x, t) => b.label(x, t, true),
            Picker(x) => picker(b, app, x),
            Icon(x, icon, tip) => {
                b.icon(x, icon, tip, false, true);
            }
            Toggle(x, icon, tip, key) => {
                // The Mixer Brush loads and cleans after each stroke by default
                // The Mixer Brush loads and cleans after each stroke by
                // default; the sampling tools sample continuously
                let on = flag(
                    app,
                    key,
                    matches!(
                        key,
                        "mixer.load"
                            | "mixer.clean"
                            | "bgeraser.continuous"
                            | "colorreplace.continuous"
                    ),
                );
                if b.icon(x, icon, tip, on, true).clicked() {
                    set_flag(app, key, !on);
                }
            }
            Popup(x0, x1, key, options) => {
                let mut i = choice(app, key).min(options.len() - 1);
                let before = i;
                b.choice(x0, x1, key, options, &mut i, true);
                if i != before {
                    set_choice(app, key, i);
                }
            }
            Field(x0, x1, key, default) => {
                let shown = text(app, key, default);
                if let Some(t) = b.value(x0, x1, key, shown, true) {
                    set_text(app, key, default, t);
                }
            }
            Percent(x0, x1, x2, key, default) => {
                let shown = text(app, key, default);
                if let Some(t) = b.value(x0, x1, key, shown.clone(), true) {
                    set_text(app, key, default, t);
                }
                let current = crate::options_bar::typed_number(&shown).unwrap_or(0.0);
                if let Some(v) = b.slider_box(x1 - 1.0, x2, key, current) {
                    set_text(app, key, default, format!("{v}%"));
                }
            }
            Check(x, label, key, default) => {
                let mut on = flag(app, key, default);
                if b.check(x, label, &mut on, true).changed() {
                    set_flag(app, key, on);
                }
            }
            CheckOff(x, label) => {
                let mut off = false;
                b.check(x, label, &mut off, false);
            }
            Segments(edges, labels, key) => {
                let chosen = choice(app, key);
                if let Some(k) = b.segmented(edges, labels, chosen, true) {
                    set_choice(app, key, k);
                }
            }
            Radio(x, icon, tip, key, value) => {
                let on = choice(app, key) == value;
                if b.icon(x, icon, tip, on, true).clicked() {
                    set_choice(app, key, value);
                }
            }
            Button(x0, x1, label, enabled) => {
                b.button(x0, x1, label, enabled);
            }
            PopupOff(x0, x1, label) => {
                b.popup(x0, x1, label, label, false, |_| {});
            }
            NoPattern(x0, x1, x2) => b.empty_pattern(x0, x1, x2),
            ColorBox(x0, x1, [r, g, bl]) => {
                b.color_box(x0, x1, egui::Color32::from_rgb(r, g, bl));
            }
            GradientSwatch(x0, x1, x2) => {
                let to32 = |c: op_core::Color| {
                    let [r, g, b, _] = c.to_rgba8();
                    egui::Color32::from_rgb(r, g, b)
                };
                let (a, z) = if app.gradient.reverse {
                    (app.background, app.foreground)
                } else {
                    (app.foreground, app.background)
                };
                b.gradient_swatch(x0, x1, x2, to32(a), to32(z));
            }
            Modes(x, key) => {
                use crate::state::SelectionMode;
                const MODES: [SelectionMode; 4] = [
                    SelectionMode::New,
                    SelectionMode::Add,
                    SelectionMode::Subtract,
                    SelectionMode::Intersect,
                ];
                let i = choice(app, key).min(3);
                let mut mode = MODES[i];
                b.modes(x, &mut mode);
                let j = MODES.iter().position(|m| *m == mode).unwrap_or(0);
                if j != i {
                    set_choice(app, key, j);
                }
            }
            Swatch(x0, x1, x2, white) => {
                let fill = if white {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgb(0x16, 0x34, 0x18)
                };
                // The pattern's chevron sits in a box after it
                let chevron_box = (!white).then_some((x1 - 1.0, x2));
                b.swatch(x0, x1, fill, chevron_box);
            }
        }
    }
}

/// The brush preset picker: the current tool's size and hardness, which a
/// click lets you change.
fn picker(b: &mut Bar, app: &mut AppState, x: f32) {
    let tool = app.tool;
    let (size, hardness) = match app.paint_options(tool) {
        Some(o) => (o.size, o.hardness),
        None => {
            let s = app.setting(picker_key(tool), "13").parse().unwrap_or(13.0);
            (s, 1.0)
        }
    };
    let response = b.brush_picker(x, &format!("{size:.0}"), size, hardness);
    let mut size = size;
    let mut hardness = hardness;
    egui::Popup::from_response(&response)
        .open_memory(response.clicked().then_some(egui::SetOpenCommand::Toggle))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(240.0);
            ui.label("Size:");
            ui.add(
                egui::Slider::new(&mut size, 1.0..=crate::state::PaintOptions::MAX_SIZE)
                    .logarithmic(true)
                    .max_decimals(0)
                    .suffix(" px"),
            );
            if tool != Tool::Pencil {
                ui.label("Hardness:");
                let mut pct = hardness * 100.0;
                if ui
                    .add(
                        egui::Slider::new(&mut pct, 0.0..=100.0)
                            .max_decimals(0)
                            .suffix("%"),
                    )
                    .changed()
                {
                    hardness = pct / 100.0;
                }
            }
        });
    match app.paint_options(tool) {
        Some(o) => {
            o.size = size;
            o.hardness = hardness;
        }
        None => *app.setting(picker_key(tool), "13") = format!("{size:.0}"),
    }
}

/// Where a tool without paint options keeps its brush size.
fn picker_key(tool: Tool) -> &'static str {
    match tool {
        Tool::PatternStamp => "pattern.size",
        Tool::ArtHistoryBrush => "arthistory.size",
        Tool::ColorReplacement => "colorreplace.size",
        Tool::MixerBrush => "mixer.size",
        Tool::BackgroundEraser => "bgeraser.size",
        Tool::Smudge => "smudge.size",
        _ => "brush.size",
    }
}
