//! Native macOS menu bar, laid out like Photoshop's. Items that aren't
//! implemented yet are shown disabled so the menus read like the real thing.

use std::str::FromStr;
use std::sync::mpsc::{Receiver, channel};

use muda::accelerator::{Accelerator, KeyAccelerator};
use muda::{
    AboutMetadata, CheckMenuItem, ContextMenu, IsMenuItem, Menu, MenuEvent, MenuItem,
    PredefinedMenuItem, Submenu,
};

use crate::commands::Command;
use crate::rulers::RulerUnit;
use crate::state::AppState;
use crate::status_info::StatusInfo;
use op_core::align::{Align, Distribute};

const ALL_COMMANDS: &[Command] = &[
    Command::New,
    Command::Open,
    Command::Close,
    Command::CloseAll,
    Command::CloseOthers,
    Command::Save,
    Command::SaveAs,
    Command::SaveACopy,
    Command::Revert,
    Command::Quit,
    Command::ExportAs,
    Command::Undo,
    Command::Redo,
    Command::ToggleLastState,
    Command::CanvasSize,
    Command::ImageSize,
    Command::Rotate180,
    Command::RotateArbitrary,
    Command::Rotate90Clockwise,
    Command::Rotate90CounterClockwise,
    Command::FlipCanvasHorizontal,
    Command::FlipCanvasVertical,
    Command::Crop,
    Command::Trim,
    Command::RevealAll,
    Command::Invert,
    Command::Desaturate,
    Command::Equalize,
    Command::Threshold,
    Command::Posterize,
    Command::Levels,
    Command::HueSaturation,
    Command::Exposure,
    Command::BrightnessContrast,
    Command::Curves,
    Command::ColorBalance,
    Command::BlackWhite,
    Command::Vibrance,
    Command::ChannelMixer,
    Command::SelectiveColor,
    Command::PhotoFilter,
    Command::GradientMap,
    Command::AutoTone,
    Command::AutoContrast,
    Command::AutoColor,
    Command::LastFilter,
    Command::Average,
    Command::Solarize,
    Command::GaussianBlur,
    Command::BoxBlur,
    Command::UnsharpMask,
    Command::AddNoise,
    Command::Median,
    Command::Minimum,
    Command::Maximum,
    Command::HighPass,
    Command::Offset,
    Command::Mosaic,
    Command::MotionBlur,
    Command::Emboss,
    Command::Twirl,
    Command::Pinch,
    Command::Spherize,
    Command::PolarCoordinates,
    Command::SurfaceBlur,
    Command::DustAndScratches,
    Command::Fragment,
    Command::CustomFilter,
    Command::Despeckle,
    Command::SharpenEdges,
    Command::TraceContour,
    Command::Wind,
    Command::Blur,
    Command::BlurMore,
    Command::Sharpen,
    Command::SharpenMore,
    Command::FindEdges,
    Command::NewLayer,
    Command::DeleteLayer,
    Command::ToggleLayerVisibility,
    Command::DuplicateLayer,
    Command::LayerViaCopy,
    Command::LayerViaCut,
    Command::LayerFromBackground,
    Command::DeleteHiddenLayers,
    Command::BringToFront,
    Command::BringForward,
    Command::SendBackward,
    Command::SendToBack,
    Command::MergeDown,
    Command::MergeVisible,
    Command::FlattenImage,
    Command::Fill,
    Command::Clear,
    Command::FreeTransform,
    Command::TransformSelection,
    TRANSFORM_MODES[0],
    TRANSFORM_MODES[1],
    TRANSFORM_MODES[2],
    TRANSFORM_MODES[3],
    TRANSFORM_MODES[4],
    WARP,
    Command::TransformAgain,
    Command::TransformRotate180,
    Command::TransformRotate90Clockwise,
    Command::TransformRotate90CounterClockwise,
    Command::TransformFlipHorizontal,
    Command::TransformFlipVertical,
    Command::Cut,
    Command::Copy,
    Command::CopyMerged,
    Command::Paste,
    Command::PasteInPlace,
    Command::PasteInto,
    Command::PasteOutside,
    Command::MaskRevealAll,
    Command::MaskHideAll,
    Command::MaskRevealSelection,
    Command::MaskHideSelection,
    Command::MaskDelete,
    Command::MaskApply,
    Command::MaskToggle,
    Command::SelectAll,
    Command::DeselectLayers,
    Command::LinkLayers,
    Command::SelectLinkedLayers,
    Command::SelectAllLayers,
    Command::RenameLayer,
    Command::LockLayers,
    Command::Deselect,
    Command::Reselect,
    Command::SelectInverse,
    Command::ModifyBorder,
    Command::ModifySmooth,
    Command::ModifyExpand,
    Command::ModifyContract,
    Command::ModifyFeather,
    Command::Grow,
    Command::Similar,
    Command::QuickMask,
    Command::ZoomIn,
    Command::ZoomOut,
    Command::FitOnScreen,
    Command::FitLayers,
    Command::ActualPixels,
    Command::Zoom200,
    Command::PrintSize,
    Command::ActualSize,
    Command::ToggleRulers,
    Command::ToggleExtras,
    Command::ToggleGuides,
    Command::ToggleGrid,
    Command::FlipView,
    Command::TogglePixelGrid,
    Command::ToggleSelectionEdges,
    Command::ToggleLayerEdges,
    Command::ShowAllExtras,
    Command::ShowNoExtras,
    Command::ToggleSnap,
    Command::RulerUnits(RulerUnit::Pixels),
    Command::RulerUnits(RulerUnit::Inches),
    Command::RulerUnits(RulerUnit::Centimeters),
    Command::RulerUnits(RulerUnit::Millimeters),
    Command::RulerUnits(RulerUnit::Points),
    Command::RulerUnits(RulerUnit::Picas),
    Command::RulerUnits(RulerUnit::Percent),
    Command::StatusInfo(StatusInfo::DocumentSizes),
    Command::StatusInfo(StatusInfo::DocumentProfile),
    Command::StatusInfo(StatusInfo::DocumentDimensions),
    Command::StatusInfo(StatusInfo::GpuMode),
    Command::StatusInfo(StatusInfo::CompositingMode),
    Command::StatusInfo(StatusInfo::MeasurementScale),
    Command::StatusInfo(StatusInfo::ScratchSizes),
    Command::StatusInfo(StatusInfo::Efficiency),
    Command::StatusInfo(StatusInfo::Timing),
    Command::StatusInfo(StatusInfo::CurrentTool),
    Command::StatusInfo(StatusInfo::Exposure32),
    Command::StatusInfo(StatusInfo::SaveProgress),
    Command::StatusInfo(StatusInfo::DownloadProgress),
    Command::StatusInfo(StatusInfo::SmartObjects),
    Command::StatusInfo(StatusInfo::LayerCount),
    Command::ScreenMode(crate::state::ScreenMode::Standard),
    Command::ScreenMode(crate::state::ScreenMode::FullWithMenus),
    Command::ScreenMode(crate::state::ScreenMode::Full),
    Command::SnapToGuides,
    Command::SnapToGrid,
    Command::SnapToLayers,
    Command::SnapToSlices,
    Command::SnapToBounds,
    Command::SnapToAll,
    Command::SnapToNone,
    Command::LockGuides,
    Command::ClearGuides,
    Command::NewGuide,
    Command::HideApp,
    Command::ToggleHistory,
    Command::ToggleInfo,
    Command::ToggleNavigator,
    Command::ToggleHistogram,
    Command::Align(Align::Top),
    Command::Align(Align::VerticalCenter),
    Command::Align(Align::Bottom),
    Command::Align(Align::Left),
    Command::Align(Align::HorizontalCenter),
    Command::Align(Align::Right),
    Command::Distribute(Distribute::Top),
    Command::Distribute(Distribute::VerticalCenter),
    Command::Distribute(Distribute::Bottom),
    Command::Distribute(Distribute::Left),
    Command::Distribute(Distribute::HorizontalCenter),
    Command::Distribute(Distribute::Right),
    Command::Distribute(Distribute::Horizontally),
    Command::Distribute(Distribute::Vertically),
    Command::GroupLayers,
    Command::UngroupLayers,
    Command::NewGroup,
    Command::NewGroupFromLayers,
    Command::ArrangeReverse,
];

/// Edit > Transform > Warp.
pub const WARP: Command = Command::TransformIn(crate::state::TransformMode::Warp, "Warp");

/// Edit > Transform's modes, as commands.
pub const TRANSFORM_MODES: [Command; 5] = {
    use crate::state::TransformMode::*;
    [
        Command::TransformIn(Free, "Scale"),
        Command::TransformIn(Free, "Rotate"),
        Command::TransformIn(Skew, "Skew"),
        Command::TransformIn(Distort, "Distort"),
        Command::TransformIn(Perspective, "Perspective"),
    ]
};

fn id(command: Command) -> String {
    format!("{command:?}")
}

fn accelerator(text: &str) -> Option<Accelerator> {
    Accelerator::from_str(text).ok()
}

pub struct NativeMenu {
    /// Keeps the native menu alive.
    _menu: Menu,
    items: Vec<(Command, MenuItem)>,
    /// View menu switches, shown with a check mark.
    checks: Vec<(Command, CheckMenuItem)>,
    events: Receiver<Command>,
    /// Last applied (enabled, dynamic label) per item, to avoid redundant
    /// native calls.
    applied: Vec<(bool, Option<String>)>,
    /// The window's NSView, for menus that pop up in it.
    view: Option<usize>,
}

impl NativeMenu {
    pub fn install(ctx: &egui::Context) -> Self {
        let items = std::cell::RefCell::new(Vec::new());
        let item = |label: &str, command: Command| {
            let accel = command
                .shortcut()
                .and_then(|s| accelerator(&s.accelerator()));
            let item = MenuItem::with_id(id(command), label, false, accel);
            if command == Command::ZoomIn {
                // Shown as Cmd++ like Photoshop; Cmd+= is handled in egui
                let plus = KeyAccelerator::from_str("CmdOrCtrl++").ok();
                let _ = item.set_key_accelerator(plus);
            }
            items.borrow_mut().push((command, item.clone()));
            item
        };
        let checks = std::cell::RefCell::new(Vec::new());
        // A switch with a check mark
        let check_item = |label: &str, command: Command| {
            let accel = command
                .shortcut()
                .and_then(|s| accelerator(&s.accelerator()));
            let item = CheckMenuItem::with_id(id(command), label, false, false, accel);
            checks.borrow_mut().push((command, item.clone()));
            item
        };
        // A command item without a key equivalent (its key is handled in egui)
        let plain_item = |label: &str, command: Command| {
            let item = MenuItem::with_id(id(command), label, false, None);
            items.borrow_mut().push((command, item.clone()));
            item
        };
        // Not implemented yet: shown disabled
        let todo = |label: &str, accel: Option<&str>| {
            MenuItem::new(label, false, accel.and_then(accelerator))
        };
        let todo_sub = |label: &str| Submenu::new(label, false);
        let sep = PredefinedMenuItem::separator;

        let app_menu = Submenu::with_items(
            "OpenPhoto",
            true,
            &[
                &PredefinedMenuItem::about(
                    Some("About OpenPhoto"),
                    Some(AboutMetadata {
                        name: Some("OpenPhoto".into()),
                        version: Some(env!("CARGO_PKG_VERSION").into()),
                        ..Default::default()
                    }),
                ),
                &sep(),
                &PredefinedMenuItem::services(None),
                &sep(),
                // Photoshop uses Ctrl+Cmd+H here and Cmd+H for View > Extras
                &item("Hide OpenPhoto", Command::HideApp),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &sep(),
                // Not the predefined item: quitting must ask about unsaved
                // changes first
                &item("Quit OpenPhoto", Command::Quit),
            ],
        );

        let file = Submenu::with_items(
            "File",
            true,
            &[
                &item("New...", Command::New) as &dyn IsMenuItem,
                &item("Open...", Command::Open),
                &todo("Browse in Bridge...", Some("CmdOrCtrl+Alt+O")),
                &todo("Open as Smart Object...", None),
                &todo_sub("Open Recent"),
                &sep(),
                &item("Close", Command::Close),
                &item("Close All", Command::CloseAll),
                &item("Close Others", Command::CloseOthers),
                &todo("Close and Go to Bridge...", Some("CmdOrCtrl+Shift+W")),
                &item("Save", Command::Save),
                &item("Save As...", Command::SaveAs),
                &item("Save a Copy...", Command::SaveACopy),
                &item("Revert", Command::Revert),
                &sep(),
                &todo("Invite to Edit...", None),
                &todo("Share for Review", None),
                &Submenu::with_items("Export", true, &[&item("Export As...", Command::ExportAs)])
                    .expect("static menu definition is valid"),
                &sep(),
                // Photoshop's Adobe Stock and Adobe Express items are left
                // out: the app shows no Adobe brand or service
                &todo("Place Embedded...", None),
                &todo("Place Linked...", None),
                &todo("Package...", None),
                &sep(),
                &todo_sub("Automate"),
                &todo_sub("Scripts"),
                &todo_sub("Import"),
                &todo("Import from iPhone or iPad", None),
                &sep(),
                &todo("File Info...", Some("CmdOrCtrl+Shift+Alt+I")),
                &todo("Version History", None),
                &sep(),
                &todo("Print...", Some("CmdOrCtrl+P")),
                &todo("Print One Copy", Some("CmdOrCtrl+Shift+Alt+P")),
            ],
        );

        let edit = Submenu::with_items(
            "Edit",
            true,
            &[
                &item("Undo", Command::Undo) as &dyn IsMenuItem,
                &item("Redo", Command::Redo),
                &item("Toggle Last State", Command::ToggleLastState),
                &sep(),
                &todo("Fade...", Some("CmdOrCtrl+Shift+F")),
                &sep(),
                // With a text field focused these pass the edit on to it
                &item("Cut", Command::Cut),
                &item("Copy", Command::Copy),
                &item("Copy Merged", Command::CopyMerged),
                &item("Paste", Command::Paste),
                &Submenu::with_items(
                    "Paste Special",
                    true,
                    &[
                        &item("Paste in Place", Command::PasteInPlace) as &dyn IsMenuItem,
                        &item("Paste Into", Command::PasteInto),
                        &item("Paste Outside", Command::PasteOutside),
                    ],
                )
                .expect("static menu definition is valid"),
                &plain_item("Clear", Command::Clear),
                &sep(),
                &todo("Search", None),
                &todo("Check Spelling...", None),
                &todo("Find and Replace Text...", None),
                &sep(),
                &item("Fill...", Command::Fill),
                &todo("Stroke...", None),
                &todo("Content-Aware Fill...", None),
                &sep(),
                &todo("Content-Aware Scale", None),
                &todo("Puppet Warp", None),
                &todo("Perspective Warp", None),
                &item("Free Transform", Command::FreeTransform),
                &Submenu::with_items(
                    "Transform",
                    true,
                    &[
                        &item("Again", Command::TransformAgain) as &dyn IsMenuItem,
                        &sep(),
                        &item("Scale", TRANSFORM_MODES[0]),
                        &item("Rotate", TRANSFORM_MODES[1]),
                        &item("Skew", TRANSFORM_MODES[2]),
                        &item("Distort", TRANSFORM_MODES[3]),
                        &item("Perspective", TRANSFORM_MODES[4]),
                        &sep(),
                        &item("Warp", WARP),
                        &todo("Split Warp Horizontally", None),
                        &todo("Split Warp Vertically", None),
                        &todo("Split Warp Crosswise", None),
                        &todo("Remove Warp Split", None),
                        &sep(),
                        &todo("Convert warp anchor point", None),
                        &sep(),
                        &todo("Toggle Guides", None),
                        &sep(),
                        &item("Rotate 180°", Command::TransformRotate180),
                        &item("Rotate 90° Clockwise", Command::TransformRotate90Clockwise),
                        &item(
                            "Rotate 90° Counter Clockwise",
                            Command::TransformRotate90CounterClockwise,
                        ),
                        &sep(),
                        &item("Flip Horizontal", Command::TransformFlipHorizontal),
                        &item("Flip Vertical", Command::TransformFlipVertical),
                    ],
                )
                .expect("static menu definition is valid"),
                &todo("Auto-Align Layers...", None),
                &todo("Auto-Blend Layers...", None),
                &sep(),
                &todo("Define Brush Preset...", None),
                &todo("Define Pattern...", None),
                &todo("Define Custom Shape...", None),
                &sep(),
                &todo_sub("Purge"),
                &sep(),
                &todo("Color Settings...", Some("CmdOrCtrl+Shift+K")),
                &todo("Assign Profile...", None),
                &todo("Convert to Profile...", None),
                &sep(),
                &todo("Keyboard Shortcuts...", Some("CmdOrCtrl+Shift+Alt+K")),
                &todo("Menus...", Some("CmdOrCtrl+Shift+Alt+M")),
                &todo("Toolbar...", None),
            ],
        );

        let image = Submenu::with_items(
            "Image",
            true,
            &[
                &todo_sub("Mode") as &dyn IsMenuItem,
                &sep(),
                &Submenu::with_items(
                    "Adjustments",
                    true,
                    &[
                        &item("Brightness/Contrast...", Command::BrightnessContrast)
                            as &dyn IsMenuItem,
                        &item("Levels...", Command::Levels),
                        &item("Curves...", Command::Curves),
                        &item("Exposure...", Command::Exposure),
                        &sep(),
                        &item("Vibrance...", Command::Vibrance),
                        &item("Hue/Saturation...", Command::HueSaturation),
                        &item("Color Balance...", Command::ColorBalance),
                        &item("Black & White...", Command::BlackWhite),
                        &item("Photo Filter...", Command::PhotoFilter),
                        &item("Channel Mixer...", Command::ChannelMixer),
                        &todo("Color Lookup...", None),
                        &sep(),
                        &item("Invert", Command::Invert),
                        &item("Posterize...", Command::Posterize),
                        &item("Threshold...", Command::Threshold),
                        &item("Gradient Map...", Command::GradientMap),
                        &item("Selective Color...", Command::SelectiveColor),
                        &sep(),
                        &todo("Shadows/Highlights...", None),
                        &todo("HDR Toning...", None),
                        &sep(),
                        &item("Desaturate", Command::Desaturate),
                        &todo("Match Color...", None),
                        &todo("Replace Color...", None),
                        &item("Equalize", Command::Equalize),
                    ],
                )
                .expect("static menu definition is valid"),
                &sep(),
                &item("Auto Tone", Command::AutoTone),
                &item("Auto Contrast", Command::AutoContrast),
                &item("Auto Color", Command::AutoColor),
                &sep(),
                &item("Image Size...", Command::ImageSize),
                &todo("Generative Upscale...", Some("CmdOrCtrl+Shift+Alt+U")),
                &item("Canvas Size...", Command::CanvasSize),
                &Submenu::with_items(
                    "Image Rotation",
                    true,
                    &[
                        &item("180°", Command::Rotate180) as &dyn IsMenuItem,
                        &item("90° Clockwise", Command::Rotate90Clockwise),
                        &item("90° Counter Clockwise", Command::Rotate90CounterClockwise),
                        &item("Arbitrary...", Command::RotateArbitrary),
                        &sep(),
                        &item("Flip Canvas Horizontal", Command::FlipCanvasHorizontal),
                        &item("Flip Canvas Vertical", Command::FlipCanvasVertical),
                    ],
                )
                .expect("static menu definition is valid"),
                &item("Crop", Command::Crop),
                &item("Trim...", Command::Trim),
                &item("Reveal All", Command::RevealAll),
                &sep(),
                &todo("Duplicate...", None),
                &todo("Apply Image...", None),
                &todo("Calculations...", None),
                &sep(),
                &todo_sub("Variables"),
                &todo("Apply Data Set...", None),
                &sep(),
                &todo("Trap...", None),
                &sep(),
                &todo_sub("Analysis"),
            ],
        );

        let layer = Submenu::with_items(
            "Layer",
            true,
            &[
                &Submenu::with_items(
                    "New",
                    true,
                    &[
                        &item("Layer...", Command::NewLayer) as &dyn IsMenuItem,
                        &item("Layer from Background...", Command::LayerFromBackground),
                        &item("Group...", Command::NewGroup),
                        &item("Group from Layers...", Command::NewGroupFromLayers),
                        &todo("Artboard...", None),
                        &todo("Artboard from Group", None),
                        &todo("Artboard from Layers...", None),
                        &todo("Frame from Layers...", None),
                        &todo("Convert to Frame", None),
                        &sep(),
                        &item("Layer Via Copy", Command::LayerViaCopy),
                        &item("Layer Via Cut", Command::LayerViaCut),
                    ],
                )
                .expect("static menu definition is valid") as &dyn IsMenuItem,
                &todo("Copy CSS", None),
                &todo("Copy SVG", None),
                &item("Duplicate Layer...", Command::DuplicateLayer),
                &Submenu::with_items(
                    "Delete",
                    true,
                    &[
                        &item("Layer", Command::DeleteLayer) as &dyn IsMenuItem,
                        &item("Hidden Layers", Command::DeleteHiddenLayers),
                    ],
                )
                .expect("static menu definition is valid"),
                &sep(),
                &todo("Quick Export as PNG", Some("CmdOrCtrl+Shift+'")),
                &todo("Export As...", Some("CmdOrCtrl+Shift+Alt+'")),
                &sep(),
                &item("Rename Layer...", Command::RenameLayer),
                &todo_sub("Layer Style"),
                &todo_sub("Smart Filter"),
                &sep(),
                &todo_sub("New Fill Layer"),
                &todo_sub("New Adjustment Layer"),
                &todo("Harmonize", None),
                &todo("Layer Content Options...", None),
                &sep(),
                &Submenu::with_items(
                    "Layer Mask",
                    true,
                    &[
                        &item("Reveal All", Command::MaskRevealAll) as &dyn IsMenuItem,
                        &item("Hide All", Command::MaskHideAll),
                        &item("Reveal Selection", Command::MaskRevealSelection),
                        &item("Hide Selection", Command::MaskHideSelection),
                        &todo("From Transparency", None),
                        &sep(),
                        &item("Delete", Command::MaskDelete),
                        &item("Apply", Command::MaskApply),
                        &sep(),
                        &item("Disable", Command::MaskToggle),
                        &todo("Link", None),
                    ],
                )
                .expect("static menu definition is valid"),
                &todo_sub("Vector Mask"),
                &todo("Create Clipping Mask", Some("CmdOrCtrl+Alt+G")),
                &todo("Mask All Objects", None),
                &sep(),
                &todo_sub("Smart Objects"),
                &todo_sub("Video Layers"),
                &todo_sub("Rasterize"),
                &sep(),
                &todo("New Layer Based Slice", None),
                &sep(),
                &item("Group Layers", Command::GroupLayers),
                &item("Ungroup Layers", Command::UngroupLayers),
                &item("Hide Layers", Command::ToggleLayerVisibility),
                &sep(),
                &Submenu::with_items(
                    "Arrange",
                    true,
                    &[
                        &item("Bring to Front", Command::BringToFront) as &dyn IsMenuItem,
                        &item("Bring Forward", Command::BringForward),
                        &item("Send Backward", Command::SendBackward),
                        &item("Send to Back", Command::SendToBack),
                        &item("Reverse", Command::ArrangeReverse),
                    ],
                )
                .expect("static menu definition is valid"),
                &todo_sub("Combine Shapes"),
                &sep(),
                &Submenu::with_items(
                    "Align",
                    true,
                    &[
                        &item("Top Edges", Command::Align(Align::Top)) as &dyn IsMenuItem,
                        &item("Vertical Centers", Command::Align(Align::VerticalCenter)),
                        &item("Bottom Edges", Command::Align(Align::Bottom)),
                        &sep(),
                        &item("Left Edges", Command::Align(Align::Left)),
                        &item(
                            "Horizontal Centers",
                            Command::Align(Align::HorizontalCenter),
                        ),
                        &item("Right Edges", Command::Align(Align::Right)),
                    ],
                )
                .expect("submenu"),
                &Submenu::with_items(
                    "Distribute",
                    true,
                    &[
                        &item("Top Edges", Command::Distribute(Distribute::Top)) as &dyn IsMenuItem,
                        &item(
                            "Vertical Centers",
                            Command::Distribute(Distribute::VerticalCenter),
                        ),
                        &item("Bottom Edges", Command::Distribute(Distribute::Bottom)),
                        &sep(),
                        &item("Left Edges", Command::Distribute(Distribute::Left)),
                        &item(
                            "Horizontal Centers",
                            Command::Distribute(Distribute::HorizontalCenter),
                        ),
                        &item("Right Edges", Command::Distribute(Distribute::Right)),
                        &sep(),
                        &item(
                            "Horizontally",
                            Command::Distribute(Distribute::Horizontally),
                        ),
                        &item("Vertically", Command::Distribute(Distribute::Vertically)),
                    ],
                )
                .expect("submenu"),
                &sep(),
                &item("Lock Layers...", Command::LockLayers),
                &sep(),
                &item("Link Layers", Command::LinkLayers),
                &item("Select Linked Layers", Command::SelectLinkedLayers),
                &sep(),
                &item("Merge Down", Command::MergeDown),
                &item("Merge Visible", Command::MergeVisible),
                &item("Flatten Image", Command::FlattenImage),
                &sep(),
                &todo_sub("Matting"),
            ],
        );

        let select = Submenu::with_items(
            "Select",
            true,
            &[
                &item("All", Command::SelectAll) as &dyn IsMenuItem,
                &item("Deselect", Command::Deselect),
                &item("Reselect", Command::Reselect),
                &item("Inverse", Command::SelectInverse),
                &sep(),
                &item("All Layers", Command::SelectAllLayers),
                &item("Deselect Layers", Command::DeselectLayers),
                &todo("Find Layers", Some("CmdOrCtrl+Shift+Alt+F")),
                &todo("Isolate Layers", None),
                &sep(),
                &todo("Color Range...", None),
                &todo("Focus Area...", None),
                &todo("Subject", None),
                &todo("Sky", None),
                &sep(),
                &todo("Select and Mask...", Some("CmdOrCtrl+Alt+R")),
                &Submenu::with_items(
                    "Modify",
                    true,
                    &[
                        &item("Border...", Command::ModifyBorder) as &dyn IsMenuItem,
                        &item("Smooth...", Command::ModifySmooth),
                        &item("Expand...", Command::ModifyExpand),
                        &item("Contract...", Command::ModifyContract),
                        &item("Feather...", Command::ModifyFeather),
                    ],
                )
                .expect("static menu definition is valid"),
                &sep(),
                &item("Grow", Command::Grow),
                &item("Similar", Command::Similar),
                &sep(),
                &item("Transform Selection", Command::TransformSelection),
                &sep(),
                &check_item("Edit in Quick Mask Mode", Command::QuickMask),
                &sep(),
                &todo("Load Selection...", None),
                &todo("Save Selection...", None),
            ],
        );

        let type_menu = Submenu::with_items(
            "Type",
            true,
            &[
                // Photoshop's first item, More from Adobe Fonts..., and its
                // separator are left out: the app shows no Adobe brand
                &todo_sub("Panels") as &dyn IsMenuItem,
                &sep(),
                &todo_sub("Anti-Alias"),
                &sep(),
                &todo_sub("Orientation"),
                &todo_sub("OpenType"),
                &sep(),
                &todo("Create Work Path", None),
                &todo("Convert to Shape", None),
                &sep(),
                &todo("Rasterize Type Layer", None),
                &todo("Convert to Paragraph Text", None),
                &todo_sub("Convert to Dynamic Text"),
                &todo("Warp Text...", None),
                &todo("Match Font...", None),
                &sep(),
                &todo_sub("Font Preview Size"),
                &sep(),
                &todo_sub("Language Options"),
                &sep(),
                &todo("Update All Text Layers", None),
                &todo("Manage Missing Fonts", None),
                &sep(),
                &todo("Paste Lorem Ipsum", None),
                &sep(),
                &todo("Load Default Type Styles", None),
                &todo("Save Default Type Styles", None),
            ],
        );

        // Items of a Filter submenu: implemented filters by command, the
        // rest disabled
        let filter_sub = |label: &str, entries: &[(&str, Option<Command>)]| {
            let built: Vec<MenuItem> = entries
                .iter()
                .map(|&(name, command)| match command {
                    Some(command) => item(name, command),
                    None => todo(name, None),
                })
                .collect();
            let refs: Vec<&dyn IsMenuItem> = built.iter().map(|i| i as &dyn IsMenuItem).collect();
            Submenu::with_items(label, true, &refs).expect("static menu definition is valid")
        };
        let filter = Submenu::with_items(
            "Filter",
            true,
            &[
                &item("Last Filter", Command::LastFilter) as &dyn IsMenuItem,
                &sep(),
                &todo("Convert for Smart Filters", None),
                &sep(),
                &todo("Neural Filters...", None),
                &sep(),
                &todo("Filter Gallery...", None),
                &todo("Adaptive Wide Angle...", Some("CmdOrCtrl+Shift+Alt+A")),
                &todo("Camera Raw Filter...", Some("CmdOrCtrl+Shift+A")),
                &todo("AI Denoise...", None),
                &todo("AI Sharpen...", None),
                &todo("Lens Correction...", Some("CmdOrCtrl+Shift+R")),
                &todo("Liquify...", Some("CmdOrCtrl+Shift+X")),
                &todo("Vanishing Point...", Some("CmdOrCtrl+Alt+V")),
                &sep(),
                &filter_sub(
                    "Blur",
                    &[
                        ("Average", Some(Command::Average)),
                        ("Blur", Some(Command::Blur)),
                        ("Blur More", Some(Command::BlurMore)),
                        ("Box Blur...", Some(Command::BoxBlur)),
                        ("Gaussian Blur...", Some(Command::GaussianBlur)),
                        ("Lens Blur...", None),
                        ("Motion Blur...", Some(Command::MotionBlur)),
                        ("Radial Blur...", None),
                        ("Shape Blur...", None),
                        ("Smart Blur...", None),
                        ("Surface Blur...", Some(Command::SurfaceBlur)),
                    ],
                ),
                &todo_sub("Blur Gallery"),
                &filter_sub(
                    "Distort",
                    &[
                        ("Displace...", None),
                        ("Pinch...", Some(Command::Pinch)),
                        ("Polar Coordinates...", Some(Command::PolarCoordinates)),
                        ("Ripple...", None),
                        ("Shear...", None),
                        ("Spherize...", Some(Command::Spherize)),
                        ("Twirl...", Some(Command::Twirl)),
                        ("Wave...", None),
                        ("ZigZag...", None),
                    ],
                ),
                &filter_sub(
                    "Noise",
                    &[
                        ("Add Noise...", Some(Command::AddNoise)),
                        ("Despeckle", Some(Command::Despeckle)),
                        ("Dust & Scratches...", Some(Command::DustAndScratches)),
                        ("Median...", Some(Command::Median)),
                        ("Reduce Noise...", None),
                    ],
                ),
                &filter_sub(
                    "Pixelate",
                    &[
                        ("Color Halftone...", None),
                        ("Crystallize...", None),
                        ("Facet", None),
                        ("Fragment", Some(Command::Fragment)),
                        ("Mezzotint...", None),
                        ("Mosaic...", Some(Command::Mosaic)),
                        ("Pointillize...", None),
                    ],
                ),
                &todo_sub("Render"),
                &filter_sub(
                    "Sharpen",
                    &[
                        ("Sharpen", Some(Command::Sharpen)),
                        ("Sharpen Edges", Some(Command::SharpenEdges)),
                        ("Sharpen More", Some(Command::SharpenMore)),
                        ("Smart Sharpen...", None),
                        ("Unsharp Mask...", Some(Command::UnsharpMask)),
                    ],
                ),
                &filter_sub(
                    "Stylize",
                    &[
                        ("Diffuse...", None),
                        ("Emboss...", Some(Command::Emboss)),
                        ("Extrude...", None),
                        ("Find Edges", Some(Command::FindEdges)),
                        ("Oil Paint...", None),
                        ("Solarize", Some(Command::Solarize)),
                        ("Tiles...", None),
                        ("Trace Contour...", Some(Command::TraceContour)),
                        ("Wind...", Some(Command::Wind)),
                    ],
                ),
                &todo_sub("Video"),
                &filter_sub(
                    "Other",
                    &[
                        ("Custom...", Some(Command::CustomFilter)),
                        ("High Pass...", Some(Command::HighPass)),
                        ("HSB/HSL", None),
                        ("Maximum...", Some(Command::Maximum)),
                        ("Minimum...", Some(Command::Minimum)),
                        ("Offset...", Some(Command::Offset)),
                    ],
                ),
            ],
        );

        let plugins = Submenu::with_items(
            "Plugins",
            true,
            &[
                &todo("Plugins Panel", None) as &dyn IsMenuItem,
                &todo("Manage Plugins...", None),
            ],
        );

        let view = Submenu::with_items(
            "View",
            true,
            &[
                &todo_sub("Proof Setup") as &dyn IsMenuItem,
                &todo("Proof Colors", Some("CmdOrCtrl+Y")),
                &todo("Gamut Warning", Some("CmdOrCtrl+Shift+Y")),
                &todo_sub("Pixel Aspect Ratio"),
                &todo("Pixel Aspect Ratio Correction", None),
                &todo("32-bit Preview Options...", None),
                &sep(),
                &item("Zoom In", Command::ZoomIn),
                &item("Zoom Out", Command::ZoomOut),
                &item("Fit on Screen", Command::FitOnScreen),
                &item("Fit Layer(s) on Screen", Command::FitLayers),
                &todo("Fit Artboard on Screen", None),
                &item("100%", Command::ActualPixels),
                &item("200%", Command::Zoom200),
                &item("Print Size", Command::PrintSize),
                &item("Actual Size", Command::ActualSize),
                &check_item("Flip Horizontal", Command::FlipView),
                &todo("Pattern Preview", None),
                &sep(),
                &Submenu::with_items(
                    "Screen Mode",
                    true,
                    &[
                        &check_item(
                            "Standard Screen Mode",
                            Command::ScreenMode(crate::state::ScreenMode::Standard),
                        ) as &dyn IsMenuItem,
                        &check_item(
                            "Full Screen Mode With Menu Bar",
                            Command::ScreenMode(crate::state::ScreenMode::FullWithMenus),
                        ),
                        &check_item(
                            "Full Screen Mode",
                            Command::ScreenMode(crate::state::ScreenMode::Full),
                        ),
                    ],
                )
                .expect("static menu definition is valid"),
                &sep(),
                &check_item("Extras", Command::ToggleExtras),
                &Submenu::with_items(
                    "Show",
                    true,
                    &[
                        &check_item("Layer Edges", Command::ToggleLayerEdges) as &dyn IsMenuItem,
                        &check_item("Selection Edges", Command::ToggleSelectionEdges),
                        &todo("Target Path", Some("CmdOrCtrl+Shift+H")),
                        &check_item("Grid", Command::ToggleGrid),
                        &check_item("Guides", Command::ToggleGuides),
                        &todo("Canvas Guides", None),
                        &todo("Artboard Guides", None),
                        &todo("Artboard Names", None),
                        &todo("Count", None),
                        &todo("Smart Guides", None),
                        &todo("Slices", None),
                        &todo("Notes", None),
                        &check_item("Pixel Grid", Command::TogglePixelGrid),
                        &todo("Pattern Preview Tile Bounds", None),
                        &sep(),
                        &todo("Mesh", None),
                        &todo("Edit Pins", None),
                        &sep(),
                        &item("All", Command::ShowAllExtras),
                        &item("None", Command::ShowNoExtras),
                        &sep(),
                        &todo("Show Extras Options...", None),
                    ],
                )
                .expect("static menu definition is valid"),
                &sep(),
                &check_item("Rulers", Command::ToggleRulers),
                &sep(),
                &check_item("Snap", Command::ToggleSnap),
                &Submenu::with_items(
                    "Snap To",
                    true,
                    &[
                        &check_item("Guides", Command::SnapToGuides) as &dyn IsMenuItem,
                        &check_item("Grid", Command::SnapToGrid),
                        &check_item("Layers", Command::SnapToLayers),
                        &check_item("Slices", Command::SnapToSlices),
                        &check_item("Document Bounds", Command::SnapToBounds),
                        &sep(),
                        &item("All", Command::SnapToAll),
                        &item("None", Command::SnapToNone),
                    ],
                )
                .expect("static menu definition is valid"),
                &sep(),
                &Submenu::with_items(
                    "Guides",
                    true,
                    &[
                        &todo("Edit Selected Guides", None) as &dyn IsMenuItem,
                        &check_item("Lock Guides", Command::LockGuides),
                        &item("Clear Guides", Command::ClearGuides),
                        &todo("Clear Selected Guides", None),
                        &todo("Clear Selected Artboard Guides", None),
                        &todo("Clear Canvas Guides", None),
                        &item("New Guide...", Command::NewGuide),
                        &todo("New Guide Layout...", None),
                        &todo("New Guides From Shape", None),
                    ],
                )
                .expect("static menu definition is valid"),
                &sep(),
                &todo("Lock Slices", None),
                &todo("Clear Slices", None),
            ],
        );

        let window = Submenu::with_items(
            "Window",
            true,
            &[
                &todo_sub("Arrange") as &dyn IsMenuItem,
                &todo_sub("Workspace"),
                &sep(),
                &todo("Actions", Some("Alt+F9")),
                &todo("Adjustments", None),
                &todo("Brush Settings", Some("F5")),
                &todo("Brushes", None),
                &todo("Channels", None),
                &todo("Character", None),
                &todo("Character Styles", None),
                &todo("Clone Source", None),
                &todo("Color", Some("F6")),
                &todo("Comments", None),
                &todo("Content Credentials (Beta)", None),
                &todo("Glyphs", None),
                &todo("Gradients", None),
                &check_item("Histogram", Command::ToggleHistogram),
                &check_item("History", Command::ToggleHistory),
                &check_item("Info", Command::ToggleInfo),
                &todo("Layer Comps", None),
                &todo("Layers", Some("F7")),
                &todo("Libraries", None),
                &todo("Materials", None),
                &todo("Measurement Log", None),
                &check_item("Navigator", Command::ToggleNavigator),
                &todo("Notes", None),
                &todo("Paragraph", None),
                &todo("Paragraph Styles", None),
                &todo("Paths", None),
                &todo("Patterns", None),
                &todo("Properties", None),
                &todo("Shapes", None),
                &todo("Styles", None),
                &todo("Swatches", None),
                &todo("Timeline", None),
                &todo("Tool Presets", None),
                &todo("Version History", None),
                &sep(),
                &todo("Application Frame", None),
                &todo("Options", None),
                &todo("Tools", None),
                &todo("Contextual Task Bar", None),
            ],
        );
        let help = Submenu::with_items("Help", true, &[&todo("OpenPhoto Help", None)]);

        let menu = Menu::new();
        let submenus: Vec<Submenu> = [
            app_menu, file, edit, image, layer, type_menu, select, filter, view, plugins, window,
            help,
        ]
        .into_iter()
        .map(|m| m.expect("static menu definition is valid"))
        .collect();
        for submenu in &submenus {
            menu.append(submenu).expect("append submenu");
        }
        menu.init_for_nsapp();
        // "Window" also gets the standard window list from macOS
        submenus[10].set_as_windows_menu_for_nsapp();

        // Menu events arrive on the main thread outside egui's frame; queue
        // them and wake egui up.
        let (tx, events) = channel();
        let ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(&command) = ALL_COMMANDS.iter().find(|c| event.id == id(**c).as_str()) {
                // Lock Layers... shows Cmd+/, but in Photoshop the key toggles
                // Lock all; only the menu opens the dialog
                let command =
                    if command == Command::LockLayers && crate::app_kit::handling_key_press() {
                        Command::ToggleLockAll
                    } else {
                        command
                    };
                let _ = tx.send(command);
                ctx.request_repaint();
            }
        }));

        let items = items.into_inner();
        let applied = vec![(false, None); items.len()];
        Self {
            _menu: menu,
            items,
            checks: checks.into_inner(),
            events,
            applied,
            view: None,
        }
    }

    /// The window's NSView (from its raw window handle), where pop-up menus
    /// open.
    pub fn set_view(&mut self, view: *mut std::ffi::c_void) {
        self.view = Some(view as usize);
    }

    /// The status bar's menu, as Photoshop's native one: what the bar
    /// shows, the current item checked.
    pub fn popup_status(&self, app: &AppState) {
        let items = StatusInfo::ALL.map(|info| {
            (
                Command::StatusInfo(info),
                info.label(),
                app.status_info == info,
            )
        });
        self.popup(&items);
    }

    /// A ruler's right-click menu: the units, the current one checked.
    pub fn popup_ruler_units(&self, app: &AppState) {
        let items = RulerUnit::ALL.map(|unit| {
            (
                Command::RulerUnits(unit),
                unit.label(),
                app.ruler_units == unit,
            )
        });
        self.popup(&items);
    }

    /// A native context menu of check items opening down from the pointer;
    /// a pick arrives as a command like the menu bar's.
    fn popup(&self, items: &[(Command, &str, bool)]) {
        let Some(view) = self.view else {
            return;
        };
        let menu = Menu::new();
        for &(command, label, checked) in items {
            let item = CheckMenuItem::with_id(id(command), label, true, checked, None);
            let _ = menu.append(&item);
        }
        // SAFETY: the window's live content view, on the main thread
        unsafe {
            menu.show_context_menu_for_nsview(view as *const std::ffi::c_void, None);
        }
    }

    /// Commands chosen from the menu (or via their shortcuts) since the last call.
    pub fn poll(&self) -> Vec<Command> {
        self.events.try_iter().collect()
    }

    /// Syncs enabled states and dynamic labels ("Undo Canvas Size") with the app.
    pub fn update(&mut self, app: &AppState) {
        for (command, item) in &self.checks {
            let enabled = command.enabled(app);
            if item.is_enabled() != enabled {
                item.set_enabled(enabled);
            }
            let checked = command.checked(app).unwrap_or(false);
            if item.is_checked() != checked {
                item.set_checked(checked);
            }
        }
        let doc = app.active_doc.and_then(|id| app.docs.get(&id));
        for (i, (command, item)) in self.items.iter().enumerate() {
            let enabled = command.enabled(app);
            let label = match command {
                Command::Undo => Some(match doc.and_then(|d| d.history.undo_name()) {
                    Some(name) => format!("Undo {name}"),
                    None => "Undo".into(),
                }),
                Command::Redo => Some(match doc.and_then(|d| d.history.redo_name()) {
                    Some(name) => format!("Redo {name}"),
                    None => "Redo".into(),
                }),
                // Disable or Enable, for the active layer's mask
                Command::MaskToggle => {
                    let mask = doc
                        .and_then(|d| d.doc.active_layer.and_then(|id| d.doc.layer(id)))
                        .and_then(|l| l.mask.as_ref());
                    Some(
                        if mask.is_some_and(|m| !m.enabled) {
                            "Enable"
                        } else {
                            "Disable"
                        }
                        .into(),
                    )
                }
                // Like Photoshop, the item names the last filter used
                Command::LastFilter => Some(
                    app.last_filter
                        .map_or("Last Filter", |f| f.name())
                        .to_string(),
                ),
                // Merge Layers with several layers selected
                Command::MergeDown => Some(
                    if doc.is_some_and(|d| d.doc.selected_layers().len() > 1) {
                        "Merge Layers"
                    } else if doc.is_some_and(|d| {
                        d.doc
                            .active_layer
                            .and_then(|id| d.doc.layer(id))
                            .is_some_and(|l| l.is_group())
                    }) {
                        "Merge Group"
                    } else {
                        "Merge Down"
                    }
                    .into(),
                ),
                // Equalize asks how to use a selection
                Command::Equalize => Some(
                    if doc.is_some_and(|d| d.doc.selection().is_some()) {
                        "Equalize..."
                    } else {
                        "Equalize"
                    }
                    .into(),
                ),
                // Unlink Layers when every selected layer is linked
                Command::LinkLayers => Some(
                    if doc.is_some_and(|d| op_core::link::can_unlink(&d.doc)) {
                        "Unlink Layers"
                    } else {
                        "Link Layers"
                    }
                    .into(),
                ),
                Command::ToggleLayerVisibility => {
                    // Show Layers when every selected layer is hidden
                    let hidden = doc.is_some_and(|d| {
                        let selected = d.doc.selected_layers();
                        !selected.is_empty()
                            && selected
                                .iter()
                                .all(|&id| d.doc.layer(id).is_some_and(|l| !l.visible))
                    });
                    Some(if hidden { "Show Layers" } else { "Hide Layers" }.into())
                }
                _ => None,
            };
            let applied = &mut self.applied[i];
            if applied.0 != enabled {
                item.set_enabled(enabled);
                applied.0 = enabled;
            }
            if let Some(label) = label
                && applied.1.as_ref() != Some(&label)
            {
                item.set_text(&label);
                applied.1 = Some(label);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menus_show_no_adobe_brand() {
        // Photoshop's items for Adobe services (Stock, Express, Fonts) are
        // left out; no menu label may name Adobe
        let source = include_str!("menu.rs");
        let labels: Vec<&str> = source
            .lines()
            .filter(|l| {
                ["todo(\"", "item(\"", "todo_sub(\"", "with_items(\""]
                    .iter()
                    .any(|k| l.contains(k))
            })
            .collect();
        assert!(labels.len() > 100, "found {} labels", labels.len());
        for label in labels {
            assert!(!label.contains("Adobe"), "{}", label.trim());
        }
    }

    /// Every shortcut shows in the native menu: a key muda can't parse
    /// would silently drop the key equivalent.
    #[test]
    fn every_shortcut_is_a_menu_accelerator() {
        let broken: Vec<String> = ALL_COMMANDS
            .iter()
            .filter_map(|c| Some((c, c.shortcut()?.accelerator())))
            .filter(|(_, a)| accelerator(a).is_none())
            .map(|(c, a)| format!("{c:?}: {a}"))
            .collect();
        assert!(broken.is_empty(), "{broken:?}");
    }
}
