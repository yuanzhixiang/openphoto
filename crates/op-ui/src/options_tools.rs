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
    /// An icon centered at x that can't be used.
    IconOff(f32, Icon),
    /// An icon centered at x that only marks the next control.
    Glyph(f32, Icon),
    /// Text starting at x, dimmed.
    LabelOff(f32, &'static str),
    /// An empty field from x0 to x1 that can't be used.
    FieldOff(f32, f32),
    /// An unusable field from x0 to x1 joined to its chevron box (to x2).
    ComboOff(f32, f32, f32),
    /// A field from x0 to x1 joined to a chevron box ending at x2 that
    /// offers the options: its key and default.
    Combo(
        f32,
        f32,
        f32,
        &'static str,
        &'static str,
        &'static [&'static str],
    ),
    /// A note centered in the bar ("No options for the … Tool.").
    Notice(&'static str),
    /// The foreground color in a frame from x0 to x1 (a shape's Fill).
    FillSwatch(f32, f32),
    /// No color in a frame from x0 to x1 (the shapes draw no stroke).
    NoStroke(f32, f32),
    /// The stroke's line style: a pop-up from x0 to x1.
    LineStyle(f32, f32, &'static str),
    /// A color box from x0 to x1, y0 to y1: the foreground ("fg") or an
    /// artboard's background.
    ColorFrame(f32, f32, f32, f32, &'static str),
    /// A push button from x0 to x1 that does what its label says (100%,
    /// Fit Screen, Fill Screen, Reset View, Front Image, Clear).
    Action(f32, f32, &'static str),
    /// The view rotation dial centered at x, showing a setting's angle.
    Dial(f32, &'static str),
    /// The Selection Brush's gear: a popup with its overlay color
    /// (`selbrush.overlay`).
    OverlayGear(f32),
    /// The custom shape: its box from x0 to x1, the chevron box to x2.
    ShapePicker(f32, f32, f32),
    /// A pop-up from x0 to x1 with an icon before its value.
    IconPopup(f32, f32, Icon, &'static str, &'static [&'static str]),
    /// A Slice tool size: its label at lx and field from x0 to x1, usable
    /// once a style other than Normal is chosen.
    SliceSize(f32, &'static str, f32, f32, &'static str),
    /// A choice drawn as a box from x0 to x1 with an icon (centered at the
    /// third x) and a label (from the fourth); the chosen one is pressed,
    /// the others dimmed: the setting's key and this choice's value.
    LabeledRadio(f32, f32, f32, f32, Icon, &'static str, &'static str, usize),
    /// A button from x0 to x1 with a menu (a small triangle at its corner).
    MenuButton(f32, f32, &'static str),
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

const FONTS: &[&str] = &["Source Sans 3"];
const FONT_STYLES: &[&str] = &["Regular", "Semibold"];
const FONT_SIZES: &[&str] = &[
    "6 pt", "8 pt", "9 pt", "10 pt", "11 pt", "12 pt", "14 pt", "18 pt", "24 pt", "30 pt", "36 pt",
    "48 pt", "60 pt", "72 pt",
];
const STROKE_WIDTHS: &[&str] = &["0.5 px", "1 px", "2 px", "3 px", "5 px", "10 px"];
const LINE_STYLES: &[&str] = &["Solid", "Dashed", "Dotted"];
const DRAW_MODES: &[&str] = &["Shape", "Path", "Pixels"];
const ARTBOARD_SIZES: &[&str] = &[
    "Custom",
    "iPhone 8/7/6",
    "iPhone 8/7/6 Plus",
    "iPhone X",
    "iPad Pro",
    "Web 1280",
    "Web 1920",
];

/// The pen tools' shared start: the mode, Make, path operations,
/// alignment and arrangement, the gear.
macro_rules! pen_bar {
    ($($rest:expr),* $(,)?) => {
        &[
            Popup(110.0, 180.0, "pen.mode", DRAW_MODES),
            Sep(183.0),
            Label(188.0, "Make:"),
            Button(220.5, 295.5, "Selection...", false),
            Button(299.5, 346.0, "Mask", false),
            Button(350.0, 401.0, "Shape", false),
            Sep(404.5),
            Icon(421.5, Icon::PathOperations, "Path operations"),
            Sep(437.5),
            Icon(454.0, Icon::PathAlignment, "Path alignment"),
            Sep(470.5),
            Icon(487.5, Icon::PathArrangement, "Path arrangement"),
            Sep(503.5),
            Icon(520.5, Icon::GearMenu, "Set additional pen and path options"),
            $($rest),*
        ]
    };
}

/// The type tools: orientation, font, style, size, alignment (the
/// vertical tools' icons given), color, warp, 3D and the panels.
macro_rules! type_bar {
    ($a:expr, $b:expr, $c:expr) => {
        &[
            Icon(125.75, Icon::TextOrientation, "Toggle text orientation"),
            Sep(146.0),
            Combo(152.0, 296.5, 311.0, "type.font", "Source Sans 3", FONTS),
            Combo(320.0, 462.0, 476.5, "type.style", "Regular", FONT_STYLES),
            Sep(482.5),
            Glyph(498.5, Icon::FontSize),
            Combo(516.5, 580.0, 594.5, "type.size", "12 pt", FONT_SIZES),
            Sep(600.5),
            Radio(620.5, $a, "Align text left or top", "type.align", 0),
            Radio(646.5, $b, "Center text", "type.align", 1),
            Radio(672.5, $c, "Align text right or bottom", "type.align", 2),
            Sep(691.5),
            ColorFrame(697.5, 725.5, 8.5, 26.5, "fg"),
            Sep(730.5),
            Icon(750.0, Icon::WarpText, "Create warped text"),
            IconOff(784.5, Icon::Text3d),
            Sep(807.5),
            Icon(
                831.5,
                Icon::CharacterPanels,
                "Toggle the Character and Paragraph panels",
            ),
            Sep(851.5),
        ]
    };
}

/// The shape tools' shared start: mode, Fill, Stroke, W and H, path
/// operations, alignment and arrangement, the gear.
macro_rules! shape_bar {
    ($($rest:expr),* $(,)?) => {
        &[
            Popup(110.0, 180.0, "shape.mode", DRAW_MODES),
            Sep(183.0),
            Label(190.0, "Fill:"),
            FillSwatch(207.0, 237.0),
            Label(241.5, "Stroke:"),
            NoStroke(276.0, 306.0),
            Combo(310.0, 361.5, 376.0, "shape.stroke_width", "1 px", STROKE_WIDTHS),
            LineStyle(381.5, 430.5, "shape.stroke_type"),
            Sep(435.0),
            Label(437.5, "W:"),
            Field(453.5, 499.0, "shape.w", "0 px"),
            Toggle(515.5, Icon::Link, "Link width and height", "shape.link"),
            Label(535.0, "H:"),
            Field(548.5, 594.0, "shape.h", "0 px"),
            Sep(603.0),
            Icon(620.0, Icon::PathCombine, "Path operations"),
            Sep(636.0),
            Icon(652.75, Icon::PathAlignment, "Path alignment"),
            Sep(669.0),
            Icon(686.0, Icon::PathArrangement, "Path arrangement"),
            Sep(702.0),
            Icon(719.0, Icon::GearMenu, "Set additional shape and path options"),
            $($rest),*
        ]
    };
}

/// The path selection tools.
const PATH_SELECTION: &[Item] = &[
    Label(110.0, "Select:"),
    Popup(
        148.0,
        247.5,
        "pathselect.select",
        &["Active Layers", "All Layers"],
    ),
    Sep(255.5),
    Label(267.5, "Fill:"),
    FillSwatch(284.0, 314.0),
    Label(318.5, "Stroke:"),
    NoStroke(353.5, 383.5),
    ComboOff(387.5, 439.0, 453.5),
    PopupOff(459.0, 508.0, ""),
    Sep(512.5),
    LabelOff(515.0, "W:"),
    FieldOff(531.0, 576.5),
    IconOff(593.25, Icon::Link),
    LabelOff(612.5, "H:"),
    FieldOff(626.0, 671.5),
    Sep(685.0),
    Icon(707.0, Icon::PathCombine, "Path operations"),
    Sep(728.0),
    Icon(749.75, Icon::PathAlignment, "Path alignment"),
    Sep(771.0),
    Icon(793.0, Icon::PathArrangement, "Path arrangement"),
    Sep(814.0),
    CheckOff(823.5, "Align Edges"),
    Sep(908.5),
    Icon(930.5, Icon::GearMenu, "Set additional path options"),
    Check(
        951.5,
        "Constrain Path Dragging",
        "pathselect.constrain",
        false,
    ),
];

/// The type tools (the masks share them).
const HORIZONTAL_TYPE: &[Item] = type_bar!(Icon::TextLeft, Icon::TextCenter, Icon::TextRight);
const VERTICAL_TYPE: &[Item] = type_bar!(Icon::TextTop, Icon::TextMiddle, Icon::TextBottom);

const ADJUSTMENTS: &[&str] = &[
    "Color and vibrance",
    "Brightness and contrast",
    "Exposure",
    "Hue and saturation",
    "Color balance",
    "Black and white",
];
const BRUSH_SIZES: &[&str] = &["10", "25", "50", "100", "200", "400"];

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
            Radio(
                182.0,
                Icon::SampleContinuous,
                "Sampling: Continuous",
                "bgeraser.sampling",
                0,
            ),
            Radio(
                208.0,
                Icon::SampleOnce,
                "Sampling: Once",
                "bgeraser.sampling",
                1,
            ),
            Radio(
                234.5,
                Icon::SampleSwatch,
                "Sampling: Background Swatch",
                "bgeraser.sampling",
                2,
            ),
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
            Radio(
                295.5,
                Icon::SampleContinuous,
                "Sampling: Continuous",
                "colorreplace.sampling",
                0,
            ),
            Radio(
                321.5,
                Icon::SampleOnce,
                "Sampling: Once",
                "colorreplace.sampling",
                1,
            ),
            Radio(
                348.0,
                Icon::SampleSwatch,
                "Sampling: Background Swatch",
                "colorreplace.sampling",
                2,
            ),
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
            Action(310.0, 371.5, "Clear All"),
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
        Pen => pen_bar!(
            Check(536.5, "Auto Add/Delete", "pen.auto_add", true),
            CheckOff(639.0, "Align Edges"),
        ),
        FreeformPen => pen_bar!(
            Check(536.5, "Magnetic", "freeform.magnetic", false),
            CheckOff(605.0, "Align Edges"),
        ),
        CurvaturePen => pen_bar!(CheckOff(537.0, "Align Edges")),
        AddAnchorPoint => &[Notice("No options for the Add Anchor Point Tool.")],
        DeleteAnchorPoint => &[Notice("No options for the Delete Anchor Point Tool.")],
        ConvertPoint => &[Notice("No options for the Convert Point Tool.")],
        HorizontalType | HorizontalTypeMask => HORIZONTAL_TYPE,
        VerticalType | VerticalTypeMask => VERTICAL_TYPE,
        PathSelection | DirectSelection => PATH_SELECTION,
        Rectangle | Triangle => shape_bar!(
            Sep(735.0),
            Glyph(749.0, Icon::CornerRadius),
            Field(759.0, 804.5, "shape.radius", "0 px"),
            Sep(807.0),
            Check(811.0, "Align Edges", "shape.align_edges", true),
        ),
        Ellipse => shape_bar!(Check(735.0, "Align Edges", "shape.align_edges", true)),
        Polygon => shape_bar!(
            Sep(735.0),
            Glyph(749.0, Icon::PolygonSides),
            Field(761.0, 806.5, "shape.sides", "5"),
            Sep(809.0),
            Glyph(823.0, Icon::CornerRadius),
            Field(833.0, 878.5, "shape.radius", "0 px"),
            Sep(881.5),
            Check(885.5, "Align Edges", "shape.align_edges", true),
        ),
        Line => shape_bar!(
            Label(736.5, "Weight:"),
            Field(776.5, 822.0, "shape.weight", "1 px"),
            Check(825.0, "Align Edges", "shape.align_edges", true),
        ),
        CustomShape => shape_bar!(
            Label(736.5, "Shape:"),
            ShapePicker(772.0, 802.0, 814.0),
            Check(817.0, "Align Edges", "shape.align_edges", true),
        ),
        Hand => &[
            Check(110.0, "Scroll All Windows", "hand.scroll_all", false),
            Sep(227.0),
            Action(239.0, 287.0, "100%"),
            Action(296.0, 365.5, "Fit Screen"),
            Action(374.5, 445.5, "Fill Screen"),
            Sep(454.0),
        ],
        RotateView => &[
            Label(112.0, "Rotation Angle:"),
            Field(188.5, 234.0, "rotate.angle", "0°"),
            Dial(252.0, "rotate.angle"),
            Action(272.0, 346.0, "Reset View"),
            Check(354.5, "Rotate All Windows", "rotate.all", false),
        ],
        Zoom => &[
            Radio(123.0, Icon::ZoomIn, "Zoom in", "zoom.out", 0),
            Radio(152.0, Icon::ZoomOut, "Zoom out", "zoom.out", 1),
            Sep(169.0),
            Check(178.0, "Resize Windows to Fit", "zoom.resize", true),
            Check(312.0, "Zoom All Windows", "zoom.all", false),
            Check(429.5, "Scrubby Zoom", "zoom.scrubby", true),
            Action(524.5, 572.5, "100%"),
            Action(581.5, 651.5, "Fit Screen"),
            Action(660.0, 731.5, "Fill Screen"),
        ],
        Artboard => &[
            Label(110.5, "Size:"),
            Popup(138.5, 379.0, "artboard.size", ARTBOARD_SIZES),
            Label(388.5, "Width:"),
            Field(423.5, 499.0, "artboard.w", "750 px"),
            Label(509.0, "Height:"),
            Field(547.0, 622.5, "artboard.h", "1334 px"),
            ColorFrame(630.5, 647.5, 9.0, 26.0, "artboard.bg"),
            Popup(
                652.0,
                734.5,
                "artboard.bg",
                &["White", "Black", "Transparent"],
            ),
            Sep(742.5),
            IconOff(764.5, Icon::ArtboardPortrait),
            IconOff(798.5, Icon::ArtboardLandscape),
            Sep(819.5),
            Icon(841.0, Icon::AddArtboard, "Add new artboard"),
            Sep(862.5),
            Icon(884.25, Icon::PathAlignment, "Align artboards"),
            Sep(905.5),
            Icon(927.5, Icon::GearMenu, "Set additional artboard options"),
        ],
        PerspectiveCrop => &[
            Label(111.5, "W:"),
            Field(126.5, 200.5, "pcrop.w", ""),
            Icon(215.5, Icon::Swap, "Swap height and width"),
            Label(232.5, "H:"),
            Field(245.0, 319.0, "pcrop.h", ""),
            Sep(323.0),
            Label(330.0, "Resolution:"),
            Field(385.5, 459.5, "pcrop.res", ""),
            Popup(462.5, 533.5, "pcrop.unit", &["Pixels/in", "Pixels/cm"]),
            Sep(537.0),
            Action(544.5, 623.5, "Front Image"),
            Action(632.5, 678.5, "Clear"),
            Sep(685.0),
            Check(690.0, "Show Grid", "pcrop.grid", true),
        ],
        Slice => &[
            Label(110.5, "Style:"),
            Popup(
                142.5,
                257.0,
                "slice.style",
                &["Normal", "Fixed Aspect Ratio", "Fixed Size"],
            ),
            SliceSize(266.5, "Width:", 301.5, 362.0, "slice.w"),
            SliceSize(372.0, "Height:", 410.0, 470.5, "slice.h"),
            Sep(478.5),
            Button(488.0, 601.5, "Slices From Guides", false),
        ],
        SliceSelect => &[
            Icon(123.0, Icon::ArrangeFront, "Bring to front"),
            Icon(149.0, Icon::ArrangeForward, "Bring forward"),
            Icon(174.75, Icon::ArrangeBackward, "Send backward"),
            Icon(200.75, Icon::ArrangeBack, "Send to back"),
            Button(222.5, 284.5, "Promote", false),
            Button(293.5, 353.5, "Divide...", false),
            Sep(362.0),
            Icon(383.75, Icon::AlignLeft, "Align left edges"),
            Icon(
                409.75,
                Icon::AlignHorizontalCenter,
                "Align horizontal centers",
            ),
            Icon(435.75, Icon::AlignRight, "Align right edges"),
            Icon(
                470.0,
                Icon::DistributeVertically,
                "Distribute vertical centers",
            ),
            Sep(491.0),
            Icon(513.0, Icon::AlignTop, "Align top edges"),
            Icon(539.0, Icon::AlignVerticalCenter, "Align vertical centers"),
            Icon(565.0, Icon::AlignBottom, "Align bottom edges"),
            Icon(
                599.0,
                Icon::DistributeHorizontally,
                "Distribute horizontal centers",
            ),
            Sep(620.0),
            Icon(642.0, Icon::More, "More distribute options"),
            Sep(663.0),
            Button(672.5, 772.5, "Hide Auto Slices", true),
            Sep(781.0),
            Icon(
                803.0,
                Icon::CharacterPanels,
                "Set options for current slice",
            ),
        ],
        Frame => &[
            Radio(
                123.0,
                Icon::FrameRect,
                "Create rectangular frame",
                "frame.shape",
                0,
            ),
            Radio(
                149.0,
                Icon::FrameEllipse,
                "Create elliptical frame",
                "frame.shape",
                1,
            ),
            Radio(
                175.0,
                Icon::FrameTriangle,
                "Create triangular frame",
                "frame.shape",
                2,
            ),
            Radio(
                201.0,
                Icon::FrameHexagon,
                "Create polygonal frame",
                "frame.shape",
                3,
            ),
            Radio(
                227.0,
                Icon::FrameCustom,
                "Create custom frame",
                "frame.shape",
                4,
            ),
            Sep(250.0),
            Label(255.5, "Stroke:"),
            NoStroke(290.5, 320.5),
            Combo(
                324.5,
                376.0,
                390.5,
                "frame.stroke_width",
                "1 px",
                STROKE_WIDTHS,
            ),
            IconPopup(
                395.5,
                490.5,
                Icon::StrokeCenter,
                "frame.stroke_align",
                &["Inside", "Center", "Outside"],
            ),
            Glyph(508.5, Icon::CornerRadius),
            Field(518.5, 564.0, "frame.radius", "0 px"),
        ],
        SelectionBrush => &[
            LabeledRadio(
                108.0,
                157.0,
                121.0,
                131.0,
                Icon::BrushAdd,
                "Add",
                "selbrush.mode",
                0,
            ),
            LabeledRadio(
                160.0,
                233.0,
                174.0,
                184.5,
                Icon::BrushSubtract,
                "Subtract",
                "selbrush.mode",
                1,
            ),
            Label(243.5, "Opacity:"),
            Percent(288.0, 332.5, 347.0, "paint.opacity", "100%"),
            Picker(372.0),
            OverlayGear(423.0),
        ],
        Remove => &[
            Radio(
                128.5,
                Icon::BrushAdd,
                "Add to the area to remove",
                "remove.mode",
                0,
            ),
            Radio(
                157.0,
                Icon::BrushSubtract,
                "Subtract from the area to remove",
                "remove.mode",
                1,
            ),
            Label(175.5, "Size"),
            Combo(198.5, 235.0, 249.5, "remove.size", "50", BRUSH_SIZES),
            Toggle(
                275.0,
                Icon::PenPressure,
                "Always use pressure for size",
                "remove.pressure",
            ),
            Sep(294.5),
            Icon(312.5, Icon::GearMenu, "Set additional options"),
            Sep(329.5),
            MenuButton(339.0, 439.5, "Find distractions"),
            Sep(444.0),
            MenuButton(453.5, 557.0, "Auto (May use g..."),
            Sep(561.0),
            Check(566.0, "Sample all layers", "remove.all_layers", false),
            Check(
                671.5,
                "Remove after each stroke",
                "remove.after_stroke",
                true,
            ),
            Check(818.5, "Create new layer", "remove.new_layer", false),
            Sep(923.5),
            IconOff(941.5, Icon::Feedback),
            Sep(958.5),
            IconOff(976.0, Icon::CropReset),
            IconOff(1009.5, Icon::CropCommit),
        ],
        AdjustmentBrush => &[
            Label(110.5, "Adjustment:"),
            Popup(173.0, 322.0, "adjbrush.adjustment", ADJUSTMENTS),
            Sep(326.0),
            Radio(
                346.0,
                Icon::BrushSubtract,
                "Subtract from the adjustment",
                "adjbrush.mode",
                0,
            ),
            Radio(
                371.5,
                Icon::BrushAdd,
                "Add to the adjustment",
                "adjbrush.mode",
                1,
            ),
            Sep(391.0),
            Picker(410.0),
            Toggle(
                457.0,
                Icon::PenPressure,
                "Always use pressure for size",
                "adjbrush.pressure",
            ),
            Sep(476.0),
            Icon(494.75, Icon::ObjectFinder, "Select a subject to adjust"),
            Sep(511.0),
            Check(517.0, "Overlay", "adjbrush.overlay", false),
            Sep(579.0),
            Label(585.5, "Opacity:"),
            Percent(629.5, 667.0, 681.5, "adjbrush.opacity", "100%"),
            Toggle(
                701.75,
                Icon::OpacityPressure,
                "Always use pressure for opacity",
                "adjbrush.opacity_pressure",
            ),
            Sep(720.5),
            Label(727.5, "Flow:"),
            Percent(754.5, 792.0, 806.5, "adjbrush.flow", "100%"),
            Toggle(
                826.0,
                Icon::Airbrush,
                "Enable airbrush-style build-up effects",
                "adjbrush.airbrush",
            ),
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
        "paint.opacity" | "smudge.strength" | "pattern.opacity" => {
            app.paint_options(tool).map(|o| percent(o.opacity))
        }
        "pattern.flow" => app.paint_options(tool).map(|o| percent(o.flow)),
        "paint.flow" => app.paint_options(tool).map(|o| percent(o.flow)),
        "bucket.opacity" => Some(percent(app.bucket.fill.opacity)),
        "bucket.tolerance" => Some(app.bucket.tolerance.to_string()),
        // The active document's view rotation
        "rotate.angle" => app
            .active()
            .map(|d| format!("{}°", d.view.rotation.round())),
        "type.style" => Some(FONT_STYLES[app.type_options.semibold as usize].to_owned()),
        "type.size" => Some(format!(
            "{} pt",
            (app.type_options.size_pt * 10.0).round() / 10.0
        )),
        "shape.sides" => Some(app.shape.sides.to_string()),
        "shape.weight" => Some(format!("{} px", (app.shape.weight * 10.0).round() / 10.0)),
        _ => None,
    }
    .unwrap_or_else(|| app.setting(key, default).clone())
}

/// Stores typed text: percentages and numbers are kept in range.
fn set_text(app: &mut AppState, key: &'static str, default: &str, typed: String) {
    let tool = app.tool;
    let number = crate::options_bar::typed_number(&typed);
    match key {
        "paint.opacity" | "paint.flow" | "smudge.strength" | "pattern.opacity" | "pattern.flow" => {
            if let (Some(o), Some(v)) = (app.paint_options(tool), number) {
                let v = (v / 100.0).clamp(0.01, 1.0);
                if key != "paint.flow" && key != "pattern.flow" {
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
        // Only the bundled family can be set
        "type.font" => {}
        "type.style" => {
            if let Some(i) = FONT_STYLES
                .iter()
                .position(|s| s.eq_ignore_ascii_case(typed.trim()))
            {
                app.type_options.semibold = i == 1;
            }
        }
        "type.size" => {
            if let Some(v) = number {
                app.type_options.size_pt = v.clamp(0.5, 1296.0);
            }
        }
        "shape.sides" => {
            if let Some(v) = number {
                app.shape.sides = (v.round() as u32).clamp(3, 100);
            }
        }
        "shape.weight" => {
            if let Some(v) = number {
                app.shape.weight = v.clamp(1.0, 1000.0);
            }
        }
        "rotate.angle" => {
            if let Some(v) = number
                && let Some(doc) = app.active()
            {
                doc.view.rotation = normalize_angle(v);
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
            let scope = app.eyedropper.sample;
            let shown: usize = app.setting(key, "2").parse().unwrap_or(2);
            // The setting's own choice, as long as it agrees with the state
            // (the "No Adjustments" ones sample like their plain versions:
            // there are no adjustment layers)
            if eyedropper_scope(shown) == scope {
                shown
            } else {
                match scope {
                    op_core::SampleScope::Current => 0,
                    op_core::SampleScope::CurrentAndBelow => 1,
                    op_core::SampleScope::All => 2,
                }
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
        "pen.mode" | "frame.stroke_align" | "artboard.size" | "adjbrush.mode" => "1",
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
            app.eyedropper.sample = eyedropper_scope(i);
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
            OverlayGear(x) => {
                let response = b.icon(x, Icon::GearMenu, "Set additional options", false, true);
                let mut index = overlay_color(app);
                egui::Popup::from_response(&response)
                    .open_memory(response.clicked().then_some(egui::SetOpenCommand::Toggle))
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        ui.label("Overlay Option");
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let (name, _) = crate::selection_brush::OVERLAY_COLORS[index];
                            egui::ComboBox::from_id_salt("selbrush-overlay")
                                .selected_text(name)
                                .show_ui(ui, |ui| {
                                    for (i, (name, c)) in
                                        crate::selection_brush::OVERLAY_COLORS.iter().enumerate()
                                    {
                                        let swatch = egui::RichText::new("■ ")
                                            .color(egui::Color32::from_rgb(c[0], c[1], c[2]));
                                        let mut job = egui::text::LayoutJob::default();
                                        swatch.append_to(
                                            &mut job,
                                            ui.style(),
                                            egui::FontSelection::Default,
                                            egui::Align::Center,
                                        );
                                        egui::RichText::new(*name).append_to(
                                            &mut job,
                                            ui.style(),
                                            egui::FontSelection::Default,
                                            egui::Align::Center,
                                        );
                                        ui.selectable_value(&mut index, i, job);
                                    }
                                });
                        });
                    });
                *app.setting("selbrush.overlay", "6") = index.to_string();
            }
            Toggle(x, icon, tip, key) => {
                // The Mixer Brush loads and cleans after each stroke by default
                // The Mixer Brush loads and cleans after each stroke by
                // default; the sampling tools sample continuously
                let on = flag(
                    app,
                    key,
                    matches!(key, "mixer.load" | "mixer.clean" | "remove.pressure"),
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
                let g = app.tool_gradient();
                let shown = if app.gradient.reverse {
                    g.reversed()
                } else {
                    g.clone()
                };
                let (swatch, menu) = b.gradient_swatch(x0, x1, x2, &shown);
                // The swatch opens the Gradient Editor, the chevron the presets
                if swatch.clicked() {
                    let presets = app.gradient_presets();
                    app.gradient_editor = Some((
                        crate::dialogs::gradient_editor::GradientEditor::new(g, presets),
                        crate::state::EditorTarget::Tool,
                    ));
                }
                let presets = app.gradient_presets();
                let entries: Vec<_> = presets
                    .iter()
                    .map(|p| crate::native_popup::Entry::item(p.name.clone(), false))
                    .collect();
                if let Some(k) = crate::native_popup::dropdown(
                    b.ui,
                    &menu,
                    b.ui.id().with("gradient-presets"),
                    &entries,
                ) {
                    app.gradient_preset = presets.get(k).cloned();
                }
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
            IconOff(x, icon) => {
                b.icon(x, icon, "", false, false);
            }
            Glyph(x, icon) => crate::ps_icons::paint(
                b.ui.painter(),
                b.at(x, 17.5),
                icon,
                crate::theme::color::OPTIONS_ICON,
                crate::theme::color::OPTIONS_BAR,
            ),
            LabelOff(x, t) => b.label(x, t, false),
            FieldOff(x0, x1) => b.field_off(x0, x1),
            ComboOff(x0, x1, x2) => {
                b.combo(x0, x1, x2, "off", String::new(), &[], false);
            }
            Combo(x0, x1, x2, key, default, options) => {
                let shown = text(app, key, default);
                if let Some(t) = b.combo(x0, x1, x2, key, shown, options, true) {
                    set_text(app, key, default, t);
                }
            }
            Notice(t) => {
                // Centered in what's left of the bar after the presets
                let cx = (b.width() + 124.5) / 2.0;
                b.centered_label(cx, t);
            }
            FillSwatch(x0, x1) => {
                b.framed_swatch(x0, x1, Some(to32(app.foreground)))
                    .on_hover_text("Set shape fill type");
            }
            NoStroke(x0, x1) => {
                b.framed_swatch(x0, x1, None)
                    .on_hover_text("Set shape stroke type");
            }
            LineStyle(x0, x1, key) => {
                let mut i = choice(app, key).min(LINE_STYLES.len() - 1);
                let before = i;
                b.line_popup(x0, x1, key, LINE_STYLES, &mut i);
                if i != before {
                    set_choice(app, key, i);
                }
            }
            ColorFrame(x0, x1, y0, y1, key) => {
                let fill = if key == "fg" {
                    to32(app.foreground)
                } else {
                    match choice(app, key) {
                        1 => egui::Color32::BLACK,
                        2 => egui::Color32::from_gray(0xcc),
                        _ => egui::Color32::WHITE,
                    }
                };
                b.framed_color(x0, x1, (y0, y1), fill);
            }
            Action(x0, x1, label) => {
                if b.button(x0, x1, label, true).clicked() {
                    let ppp = b.ui.ctx().pixels_per_point();
                    action(app, label, ppp);
                }
            }
            Dial(cx, key) => {
                let c = b.at(cx, 17.5);
                // Dragging on the dial points its tick at the pointer
                let r = crate::theme::pt(11.0);
                let response = b.ui.interact(
                    egui::Rect::from_center_size(c, egui::Vec2::splat(2.0 * r)),
                    b.ui.id().with(("dial", key)),
                    egui::Sense::click_and_drag(),
                );
                if (response.dragged() || response.clicked())
                    && let Some(p) = response.interact_pointer_pos()
                    && p != c
                {
                    let a = (p.y - c.y).atan2(p.x - c.x).to_degrees() + 90.0;
                    set_text(app, key, "0°", format!("{}", normalize_angle(a).round()));
                }
                let angle = crate::options_bar::typed_number(&text(app, key, "0°")).unwrap_or(0.0);
                let ink = crate::theme::color::OPTIONS_ICON;
                let p = b.ui.painter();
                p.circle_stroke(
                    c,
                    crate::theme::pt(10.5),
                    egui::Stroke::new(crate::theme::pt(1.0), ink),
                );
                // A tick from the rim toward the center, and a dot there
                let a = (angle - 90.0).to_radians();
                let dir = egui::vec2(a.cos(), a.sin());
                p.line_segment(
                    [
                        c + dir * crate::theme::pt(10.0),
                        c + dir * crate::theme::pt(4.0),
                    ],
                    egui::Stroke::new(crate::theme::pt(1.0), ink),
                );
                p.circle_filled(c, crate::theme::pt(1.25), ink);
            }
            ShapePicker(x0, x1, x2) => {
                // The current shape in white on black, then its menu
                let frame =
                    |x0: f32, x1: f32| egui::Rect::from_min_max(b.at(x0, 5.0), b.at(x1, 30.0));
                let border =
                    egui::Stroke::new(crate::theme::pt(1.0), egui::Color32::from_gray(0x66));
                let round = egui::CornerRadius::same(crate::theme::pt(2.0) as u8);
                let p = b.ui.painter();
                p.rect(
                    frame(x0, x1),
                    round,
                    egui::Color32::BLACK,
                    border,
                    egui::StrokeKind::Inside,
                );
                crate::ps_icons::paint(
                    p,
                    b.at((x0 + x1) / 2.0, 17.5),
                    Icon::FrameCustom,
                    egui::Color32::WHITE,
                    egui::Color32::BLACK,
                );
                p.rect(
                    frame(x1 - 1.0, x2),
                    round,
                    crate::theme::color::FIELD,
                    border,
                    egui::StrokeKind::Inside,
                );
                crate::ps_icons::paint(
                    p,
                    b.at((x1 - 1.0 + x2) / 2.0, 18.0),
                    Icon::Caret,
                    crate::theme::color::OPTIONS_ICON,
                    crate::theme::color::FIELD,
                );
            }
            IconPopup(x0, x1, icon, key, options) => {
                let mut i = choice(app, key).min(options.len() - 1);
                let mut chosen = i;
                b.popup(x0, x1, key, "", true, |ui| {
                    for (k, o) in options.iter().enumerate() {
                        ui.selectable_value(&mut chosen, k, *o);
                    }
                });
                crate::ps_icons::paint(
                    b.ui.painter(),
                    b.at(x0 + 13.0, 17.5),
                    icon,
                    crate::theme::color::OPTIONS_ICON,
                    crate::theme::color::FIELD,
                );
                b.label(x0 + 25.5, options[i], true);
                if chosen != i {
                    i = chosen;
                    set_choice(app, key, i);
                }
            }
            SliceSize(lx, label, x0, x1, key) => {
                let on = choice(app, "slice.style") != 0;
                b.label(lx, label, on);
                if on {
                    let default = if choice(app, "slice.style") == 1 {
                        "1"
                    } else {
                        "64 px"
                    };
                    let shown = text(app, key, default);
                    if let Some(t) = b.value(x0, x1, key, shown, true) {
                        set_text(app, key, default, t);
                    }
                } else {
                    b.field_off(x0, x1);
                }
            }
            LabeledRadio(x0, x1, cx, lx, icon, label, key, value) => {
                let on = choice(app, key) == value;
                let rect = b.toggle_frame(x0, x1, on);
                let r = b.ui.interact(
                    rect,
                    b.ui.id().with(("labeled-radio", key, value)),
                    egui::Sense::click(),
                );
                // The choice not taken is dimmed
                let ink = if on {
                    crate::theme::color::OPTIONS_ICON
                } else {
                    egui::Color32::from_gray(0x9a)
                };
                let bg = if on {
                    crate::theme::color::TOOL_ACTIVE
                } else {
                    crate::theme::color::OPTIONS_BAR
                };
                crate::ps_icons::paint(b.ui.painter(), b.at(cx, 17.5), icon, ink, bg);
                b.ui.painter().text(
                    b.at(lx - 0.75, 17.25),
                    egui::Align2::LEFT_CENTER,
                    label,
                    crate::theme::body(),
                    if on { crate::options_kit::LABEL } else { ink },
                );
                if r.clicked() {
                    set_choice(app, key, value);
                }
            }
            MenuButton(x0, x1, label) => {
                b.menu_button(x0, x1, label);
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

fn to32(c: op_core::Color) -> egui::Color32 {
    let [r, g, b, _] = c.to_rgba8();
    egui::Color32::from_rgb(r, g, b)
}

/// What the bars' push buttons do.
fn action(app: &mut AppState, label: &str, ppp: f32) {
    use crate::document_view as view;
    match label {
        "Reset View" => {
            if let Some(doc) = app.active() {
                doc.view.rotation = 0.0;
            }
        }
        // The Color Sampler's Clear All
        "Clear All" => {
            if let Some(doc) = app.active() {
                doc.color_samplers.clear();
            }
        }
        "Clear" => {
            for key in ["pcrop.w", "pcrop.h", "pcrop.res"] {
                app.setting(key, "").clear();
            }
        }
        "Front Image" => {
            let Some((w, h, res)) = app
                .active()
                .map(|d| (d.doc.width, d.doc.height, d.doc.resolution))
            else {
                return;
            };
            *app.setting("pcrop.w", "") = format!("{w} px");
            *app.setting("pcrop.h", "") = format!("{h} px");
            *app.setting("pcrop.res", "") = format!("{}", res.round());
        }
        _ => {
            let Some(doc) = app.active() else {
                return;
            };
            match label {
                "100%" => view::actual_pixels(doc, ppp),
                "Fit Screen" => view::fit_on_screen(doc, ppp),
                "Fill Screen" => view::fill_screen(doc, ppp),
                _ => {}
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
            let s = app
                .setting(picker_key(tool), picker_default(tool))
                .parse()
                .unwrap_or(13.0);
            // The Adjustment Brush's tip is soft
            let hardness = if tool == Tool::AdjustmentBrush {
                0.0
            } else {
                1.0
            };
            (s, hardness)
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
        None => *app.setting(picker_key(tool), picker_default(tool)) = format!("{size:.0}"),
    }
}

/// A brush size's starting value (Photoshop's).
fn picker_default(tool: Tool) -> &'static str {
    match tool {
        Tool::SelectionBrush => "200",
        Tool::AdjustmentBrush => "100",
        _ => "13",
    }
}

/// Where a tool without paint options keeps its brush size.
fn picker_key(tool: Tool) -> &'static str {
    match tool {
        Tool::SelectionBrush => "selbrush.size",
        Tool::AdjustmentBrush => "adjbrush.size",
        Tool::ArtHistoryBrush => "arthistory.size",
        Tool::MixerBrush => "mixer.size",
        _ => "brush.size",
    }
}

/// The Eraser's Mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EraserMode {
    Brush,
    Pencil,
    Block,
}

pub fn eraser_mode(app: &mut AppState) -> EraserMode {
    match choice(app, "eraser.mode") {
        1 => EraserMode::Pencil,
        2 => EraserMode::Block,
        _ => EraserMode::Brush,
    }
}

/// The Brush's or Pencil's Mode as chosen in its bar.
pub fn paint_mode(app: &mut AppState, tool: Tool) -> op_core::paint::PaintMode {
    use op_core::paint::PaintMode;
    let key = match tool {
        Tool::Brush => "brush.mode",
        Tool::Pencil => "pencil.mode",
        _ => return PaintMode::Normal,
    };
    match BLEND.get(choice(app, key)).copied() {
        Some("Behind") => PaintMode::Behind,
        Some("Clear") => PaintMode::Clear,
        Some("Normal") | None => PaintMode::Normal,
        Some(label) => op_core::BlendMode::GROUPS
            .iter()
            .flat_map(|g| g.iter())
            .find(|m| m.label() == label)
            .map_or(PaintMode::Normal, |m| PaintMode::Blend(*m)),
    }
}

/// The Eyedropper's Sample choice (Current Layer, Current & Below, All
/// Layers, All Layers No Adjustments, Current & Below No Adjustments).
fn eyedropper_scope(i: usize) -> op_core::SampleScope {
    match i {
        0 => op_core::SampleScope::Current,
        1 | 4 => op_core::SampleScope::CurrentAndBelow,
        _ => op_core::SampleScope::All,
    }
}

/// The Clone Stamp's Sample (Current Layer, Current & Below, All Layers).
pub fn clone_scope(app: &mut AppState) -> op_core::SampleScope {
    match choice(app, "clone.sample") {
        1 => op_core::SampleScope::CurrentAndBelow,
        2 => op_core::SampleScope::All,
        _ => op_core::SampleScope::Current,
    }
}

/// What the Background Eraser and Color Replacement change: their
/// Sampling, Limits (Contiguous and Find Edges flood from the brush's
/// center), Tolerance and the Background Eraser's Protect Foreground
/// Color; `None` for the other tools.
pub fn color_match(
    app: &mut AppState,
    tool: Tool,
    (foreground, background): ([u8; 3], [u8; 3]),
) -> Option<op_core::paint::ColorMatch> {
    use op_core::paint::{ColorMatch, Sampling};
    let (sampling, limits, tolerance, protect) = match tool {
        Tool::BackgroundEraser => (
            "bgeraser.sampling",
            "bgeraser.limits",
            text(app, "bgeraser.tolerance", "50%"),
            flag(app, "bgeraser.protect", false),
        ),
        Tool::ColorReplacement => (
            "colorreplace.sampling",
            "colorreplace.limits",
            text(app, "colorreplace.tolerance", "30%"),
            false,
        ),
        _ => return None,
    };
    let sampling = match choice(app, sampling) {
        1 => Sampling::Once,
        2 => Sampling::Swatch(background),
        _ => Sampling::Continuous,
    };
    let tolerance = crate::options_bar::typed_number(&tolerance).unwrap_or(50.0) / 100.0;
    Some(ColorMatch {
        sampling,
        tolerance: tolerance.clamp(0.0, 1.0),
        contiguous: choice(app, limits) != 0,
        protect: protect.then_some(foreground),
    })
}

/// The Color Replacement tool's Mode.
pub fn replace_mode(app: &mut AppState) -> op_core::BlendMode {
    use op_core::BlendMode;
    match choice(app, "colorreplace.mode") {
        0 => BlendMode::Hue,
        1 => BlendMode::Saturation,
        3 => BlendMode::Luminosity,
        _ => BlendMode::Color,
    }
}

/// The Magic Eraser's options: its wand rule and opacity.
pub fn magic_eraser_options(app: &mut AppState) -> (op_core::fill::BucketOptions, f32) {
    let number = |app: &mut AppState, key: &'static str, default: &str| {
        crate::options_bar::typed_number(&text(app, key, default))
    };
    let tolerance = number(app, "magiceraser.tolerance", "32").unwrap_or(32.0);
    let opacity = number(app, "magiceraser.opacity", "100%").unwrap_or(100.0) / 100.0;
    let options = op_core::fill::BucketOptions {
        tolerance: tolerance.clamp(0.0, 255.0) as u8,
        anti_alias: flag(app, "magiceraser.anti_alias", true),
        contiguous: flag(app, "magiceraser.contiguous", true),
        all_layers: flag(app, "magiceraser.all_layers", false),
        ..Default::default()
    };
    (options, opacity.clamp(0.0, 1.0))
}

/// The Red Eye tool's Pupil Size and Darken Amount (0–1).
pub fn red_eye_options(app: &mut AppState) -> (f32, f32) {
    let pct = |app: &mut AppState, key: &'static str| {
        crate::options_bar::typed_number(&text(app, key, "50%")).unwrap_or(50.0) / 100.0
    };
    (pct(app, "redeye.pupil"), pct(app, "redeye.darken"))
}

/// The healing tools' options: the Healing Brush's Aligned and Sample, the
/// Spot Healing Brush's Sample All Layers, the Patch tool's Destination
/// mode and Content-Aware Move's Extend.
#[derive(Clone, Copy, Debug)]
pub struct HealOptions {
    pub aligned: bool,
    pub scope: op_core::SampleScope,
    pub spot_all_layers: bool,
    pub patch_destination: bool,
    pub extend: bool,
}

pub fn heal_options(app: &mut AppState) -> HealOptions {
    HealOptions {
        aligned: flag(app, "heal.aligned", false),
        scope: match choice(app, "heal.sample") {
            1 => op_core::SampleScope::CurrentAndBelow,
            2 => op_core::SampleScope::All,
            _ => op_core::SampleScope::Current,
        },
        spot_all_layers: flag(app, "spotheal.all_layers", false),
        patch_destination: choice(app, "patch.source") == 1,
        extend: choice(app, "cam.move") == 1,
    }
}

/// The Magnetic Lasso's Width (pixels), Contrast (0–1) and Frequency
/// (0–100).
pub fn magnetic_options(app: &mut AppState) -> (u32, f32, f32) {
    let number = |app: &mut AppState, key: &'static str, default: &str| {
        crate::options_bar::typed_number(&text(app, key, default))
    };
    let width = number(app, "lasso.width", "10 px")
        .unwrap_or(10.0)
        .clamp(1.0, 256.0);
    let contrast = number(app, "lasso.contrast", "10%")
        .unwrap_or(10.0)
        .clamp(1.0, 100.0)
        / 100.0;
    let frequency = number(app, "lasso.frequency", "57")
        .unwrap_or(57.0)
        .clamp(0.0, 100.0);
    (width as u32, contrast, frequency)
}

/// The Quick Selection tool's mode (0 New, 1 Add, 2 Subtract), brush
/// diameter, Sample All Layers and Enhance Edge.
pub fn quick_options(app: &mut AppState) -> (usize, f32, bool, bool) {
    let mode = app.setting("quick.mode", "0").parse().unwrap_or(0);
    let size = app.setting("quick.size", "30").parse().unwrap_or(30.0);
    (
        mode,
        size,
        flag(app, "quick.all_layers", false),
        flag(app, "quick.enhance_edge", false),
    )
}

/// The Object Selection tool's Sample All Layers and Hard Edge.
pub fn object_options(app: &mut AppState) -> (bool, bool) {
    (
        flag(app, "object.all_layers", false),
        flag(app, "object.hard_edge", true),
    )
}

/// An angle in degrees within Photoshop's ±180°.
pub fn normalize_angle(v: f32) -> f32 {
    let a = (v + 180.0).rem_euclid(360.0) - 180.0;
    if a == -180.0 { 180.0 } else { a }
}

/// The Selection Brush's overlay color, an index into
/// `selection_brush::OVERLAY_COLORS` (Magenta by default).
pub fn overlay_color(app: &mut AppState) -> usize {
    app.setting("selbrush.overlay", "6")
        .parse::<usize>()
        .ok()
        .filter(|&i| i < crate::selection_brush::OVERLAY_COLORS.len())
        .unwrap_or(crate::selection_brush::DEFAULT_OVERLAY)
}

/// Whether the Selection Brush is set to Subtract.
pub fn selection_brush_subtracts(app: &mut AppState) -> bool {
    app.setting("selbrush.mode", "0") == "1"
}
