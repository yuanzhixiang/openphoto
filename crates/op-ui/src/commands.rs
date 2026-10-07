//! Application commands. Menu items and keyboard shortcuts both resolve to a
//! [`Command`], which is executed in one place.

use egui::{Key, Modifiers};

use op_core::adjust::{self, Adjustment};
use op_core::filter::Filter;
use op_core::image_ops::{self, Orientation};
use op_core::layer_ops::{self, Arrange};
use op_core::transform::{self, FixedTransform};

use crate::actions;
use crate::dialogs::{AdjustDialog, AdjustKind, ModifyKind};
use crate::document_view;
use crate::state::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    New,
    Open,
    Close,
    CloseAll,
    CloseOthers,
    Save,
    SaveAs,
    SaveACopy,
    /// File > Revert (F12).
    Revert,
    /// Quit OpenPhoto (asks about unsaved changes first).
    Quit,
    ExportAs,
    Undo,
    Redo,
    ToggleLastState,
    CanvasSize,
    ImageSize,
    Rotate180,
    /// Image > Image Rotation > Arbitrary... (opens Rotate Canvas).
    RotateArbitrary,
    Rotate90Clockwise,
    Rotate90CounterClockwise,
    FlipCanvasHorizontal,
    FlipCanvasVertical,
    /// Image > Crop (to the selection).
    Crop,
    /// Image > Trim... (opens the dialog).
    Trim,
    /// Image > Reveal All: grows the canvas over the pixels outside it.
    RevealAll,
    Invert,
    Desaturate,
    Equalize,
    /// Image > Adjustments > Threshold... (opens the dialog).
    Threshold,
    /// Image > Adjustments > Posterize... (opens the dialog).
    Posterize,
    Levels,
    Curves,
    HueSaturation,
    Exposure,
    BrightnessContrast,
    ColorBalance,
    BlackWhite,
    Vibrance,
    ChannelMixer,
    SelectiveColor,
    PhotoFilter,
    GradientMap,
    AutoTone,
    AutoContrast,
    AutoColor,
    /// Filter > Last Filter: the last filter again, with its settings.
    LastFilter,
    Average,
    Solarize,
    /// Filter > Render > Clouds, Difference Clouds.
    Clouds,
    DifferenceClouds,
    Blur,
    BlurMore,
    Sharpen,
    SharpenMore,
    FindEdges,
    GaussianBlur,
    BoxBlur,
    UnsharpMask,
    AddNoise,
    Median,
    Minimum,
    Maximum,
    HighPass,
    Offset,
    Mosaic,
    MotionBlur,
    Emboss,
    Twirl,
    /// Filter > Pixelate > Crystallize..., Pointillize...; Stylize > Diffuse...
    Crystallize,
    Pointillize,
    Diffuse,
    Ripple,
    Mezzotint,
    Pinch,
    Spherize,
    PolarCoordinates,
    SurfaceBlur,
    DustAndScratches,
    Fragment,
    /// Filter > Other > Custom...
    CustomFilter,
    Despeckle,
    SharpenEdges,
    TraceContour,
    Wind,
    /// Layer > New > Layer... (Shift+Cmd+N): opens the New Layer dialog.
    NewLayer,
    /// Alt+Shift+Cmd+N: a new layer without the dialog.
    NewLayerNoDialog,
    /// Select > All Layers (Alt+Cmd+A): every layer but the background.
    SelectAllLayers,
    /// Layer > Group Layers (Cmd+G).
    GroupLayers,
    /// Layer > Arrange > Reverse.
    ArrangeReverse,
    /// Layer > Ungroup Layers (Shift+Cmd+G).
    UngroupLayers,
    /// Layer > New > Group... (opens the New Group dialog).
    NewGroup,
    /// Layer > New > Group from Layers... (opens its dialog).
    NewGroupFromLayers,
    /// Layer > Align (and the Move tool's align buttons).
    Align(op_core::align::Align),
    /// Layer > Distribute.
    Distribute(op_core::align::Distribute),
    /// Select > Deselect Layers.
    DeselectLayers,
    /// Layer > Link Layers, or Unlink Layers when every selected layer is
    /// linked (also the Layers panel's link button).
    LinkLayers,
    /// Layer > Select Linked Layers.
    SelectLinkedLayers,
    /// Layer > Rename Layer...: starts renaming the active layer in the
    /// Layers panel.
    RenameLayer,
    /// Layer > Lock Layers...: the locks of the selected layers. Its menu
    /// item shows Cmd+/, but the key itself runs [`Self::ToggleLockAll`].
    LockLayers,
    /// Cmd+/: Lock all on the selected layers, or every lock off.
    ToggleLockAll,
    DeleteLayer,
    /// Layer > Hide Layers / Show Layers for the active layer.
    ToggleLayerVisibility,
    DuplicateLayer,
    LayerViaCopy,
    LayerViaCut,
    LayerFromBackground,
    DeleteHiddenLayers,
    BringToFront,
    BringForward,
    SendBackward,
    SendToBack,
    MergeDown,
    MergeVisible,
    FlattenImage,
    /// Edit > Fill... (opens the dialog).
    Fill,
    /// Option+Delete: fill with the foreground color.
    FillForeground,
    /// Command+Delete: fill with the background color.
    FillBackground,
    /// Edit > Clear (Delete).
    Clear,
    FreeTransform,
    /// Select > Transform Selection.
    TransformSelection,
    /// Edit > Transform > Scale, Rotate, Skew, Distort, Perspective: a
    /// transform whose handles do that.
    TransformIn(crate::state::TransformMode, &'static str),
    /// Edit > Transform > Again: the last transform once more.
    TransformAgain,
    TransformRotate180,
    TransformRotate90Clockwise,
    TransformRotate90CounterClockwise,
    TransformFlipHorizontal,
    TransformFlipVertical,
    Cut,
    Copy,
    CopyMerged,
    Paste,
    /// Edit > Paste Special > Paste in Place.
    PasteInPlace,
    /// Edit > Paste Special > Paste Into: a new layer masked by the selection.
    PasteInto,
    /// Edit > Paste Special > Paste Outside: masked by the inverse.
    PasteOutside,
    MaskRevealAll,
    MaskHideAll,
    MaskRevealSelection,
    MaskHideSelection,
    MaskDelete,
    MaskApply,
    /// Layer > Layer Mask > Disable / Enable.
    MaskToggle,
    SelectAll,
    Deselect,
    Reselect,
    SelectInverse,
    /// Select > Modify > Border... (and the next four: dialogs).
    ModifyBorder,
    ModifySmooth,
    ModifyExpand,
    ModifyContract,
    /// Select > Modify > Feather... (Shift+F6).
    ModifyFeather,
    Grow,
    Similar,
    /// Select > Edit in Quick Mask Mode (Q in egui).
    QuickMask,
    ZoomIn,
    ZoomOut,
    FitOnScreen,
    FitLayers,
    ActualPixels,
    Zoom200,
    PrintSize,
    /// View > Actual Size.
    ActualSize,
    /// Edit > Fade (opens the dialog for the last fadeable edit).
    Fade,
    /// View > Rulers.
    ToggleRulers,
    /// View > Extras.
    ToggleExtras,
    /// View > Show > Guides.
    ToggleGuides,
    /// View > Show > Grid.
    ToggleGrid,
    /// View > Flip Horizontal (the view only).
    FlipView,
    /// View > Show > Pixel Grid, Selection Edges, Layer Edges, All, None.
    TogglePixelGrid,
    ToggleSelectionEdges,
    ToggleLayerEdges,
    ShowAllExtras,
    ShowNoExtras,
    /// View > Screen Mode.
    ScreenMode(crate::state::ScreenMode),
    /// A ruler's right-click menu: the rulers' unit.
    RulerUnits(crate::rulers::RulerUnit),
    /// The status bar's menu: what it shows.
    StatusInfo(crate::status_info::StatusInfo),
    /// View > Snap and View > Snap To.
    ToggleSnap,
    SnapToGuides,
    SnapToGrid,
    SnapToLayers,
    SnapToSlices,
    SnapToBounds,
    SnapToAll,
    SnapToNone,
    /// View > Guides > Lock Guides.
    LockGuides,
    ClearGuides,
    /// View > Guides > New Guide... (opens the dialog).
    NewGuide,
    /// Hide OpenPhoto (Ctrl+Cmd+H, as Photoshop: Cmd+H is Extras).
    HideApp,
    ToggleHistory,
    /// Window > Info (F8).
    ToggleInfo,
    ToggleNavigator,
    ToggleHistogram,
}

/// A keyboard shortcut. `cmd` is Command on macOS and Ctrl elsewhere.
#[derive(Clone, Copy, Debug)]
pub struct Shortcut {
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    /// The Control key on macOS (where `cmd` is Command).
    pub ctrl: bool,
    pub key: Key,
}

const fn cmd(key: Key) -> Shortcut {
    Shortcut {
        cmd: true,
        shift: false,
        alt: false,
        ctrl: false,
        key,
    }
}

const fn shift_cmd(key: Key) -> Shortcut {
    Shortcut {
        shift: true,
        ..cmd(key)
    }
}

const fn alt_cmd(key: Key) -> Shortcut {
    Shortcut {
        alt: true,
        ..cmd(key)
    }
}

impl Shortcut {
    fn modifiers(self) -> Modifiers {
        let mut m = Modifiers::NONE;
        m.command = self.cmd;
        m.shift = self.shift;
        m.alt = self.alt;
        m.ctrl = self.ctrl;
        m
    }

    /// Accelerator string for native menus, e.g. `CmdOrCtrl+Alt+C`.
    pub fn accelerator(self) -> String {
        let mut s = String::new();
        if self.cmd {
            s.push_str("CmdOrCtrl+");
        }
        if self.shift {
            s.push_str("Shift+");
        }
        if self.alt {
            s.push_str("Alt+");
        }
        if self.ctrl {
            s.push_str("Ctrl+");
        }
        // muda knows the bracket and equals keys by their symbols
        s.push_str(match self.key {
            Key::OpenBracket => "[",
            Key::CloseBracket => "]",
            Key::Equals => "=",
            key => key.name(),
        });
        s
    }
}

impl Command {
    /// Shortcuts match Photoshop's defaults (read from Photoshop 2026's menus).
    pub fn shortcut(self) -> Option<Shortcut> {
        Some(match self {
            Self::New => cmd(Key::N),
            Self::Open => cmd(Key::O),
            Self::Close => cmd(Key::W),
            Self::CloseAll => alt_cmd(Key::W),
            Self::CloseOthers => alt_cmd(Key::P),
            Self::Save => cmd(Key::S),
            Self::SaveAs => shift_cmd(Key::S),
            Self::SaveACopy => alt_cmd(Key::S),
            Self::Revert => Shortcut {
                cmd: false,
                shift: false,
                alt: false,
                ctrl: false,
                key: Key::F12,
            },
            Self::Quit => cmd(Key::Q),
            Self::ExportAs => Shortcut {
                shift: true,
                ..alt_cmd(Key::W)
            },
            Self::Undo => cmd(Key::Z),
            Self::Redo => shift_cmd(Key::Z),
            Self::ToggleLastState => alt_cmd(Key::Z),
            Self::CanvasSize => alt_cmd(Key::C),
            Self::ImageSize => alt_cmd(Key::I),
            Self::NewLayer => shift_cmd(Key::N),
            Self::NewLayerNoDialog => Shortcut {
                alt: true,
                ..shift_cmd(Key::N)
            },
            Self::ToggleLayerVisibility => cmd(Key::Comma),
            Self::LayerViaCopy => cmd(Key::J),
            Self::LayerViaCut => shift_cmd(Key::J),
            Self::BringToFront => shift_cmd(Key::CloseBracket),
            Self::BringForward => cmd(Key::CloseBracket),
            Self::SendBackward => cmd(Key::OpenBracket),
            Self::SendToBack => shift_cmd(Key::OpenBracket),
            Self::MergeDown => cmd(Key::E),
            Self::MergeVisible => shift_cmd(Key::E),
            Self::Fill => Shortcut {
                cmd: false,
                shift: true,
                alt: false,
                ctrl: false,
                key: Key::F5,
            },
            Self::FillForeground => Shortcut {
                cmd: false,
                shift: false,
                alt: true,
                ctrl: false,
                key: Key::Backspace,
            },
            Self::FillBackground => cmd(Key::Backspace),
            Self::Clear => Shortcut {
                cmd: false,
                shift: false,
                alt: false,
                ctrl: false,
                key: Key::Backspace,
            },
            Self::FreeTransform => cmd(Key::T),
            Self::TransformAgain => shift_cmd(Key::T),
            Self::Cut => cmd(Key::X),
            Self::Copy => cmd(Key::C),
            Self::CopyMerged => shift_cmd(Key::C),
            Self::Paste => cmd(Key::V),
            Self::PasteInPlace => shift_cmd(Key::V),
            Self::PasteInto => Shortcut {
                alt: true,
                ..shift_cmd(Key::V)
            },
            Self::PasteOutside
            | Self::MaskRevealAll
            | Self::MaskHideAll
            | Self::MaskRevealSelection
            | Self::MaskHideSelection
            | Self::MaskDelete
            | Self::MaskApply
            | Self::MaskToggle => return None,
            Self::SelectAll => cmd(Key::A),
            Self::Deselect => cmd(Key::D),
            Self::Reselect => shift_cmd(Key::D),
            Self::SelectInverse => shift_cmd(Key::I),
            Self::ModifyFeather => Shortcut {
                cmd: false,
                shift: true,
                alt: false,
                ctrl: false,
                key: Key::F6,
            },
            Self::ModifyBorder
            | Self::ModifySmooth
            | Self::ModifyExpand
            | Self::ModifyContract
            | Self::Grow
            | Self::Similar
            | Self::QuickMask => return None,
            // Photoshop shows Cmd++ and also accepts Cmd+=
            Self::ZoomIn => cmd(Key::Equals),
            Self::ZoomOut => cmd(Key::Minus),
            Self::FitOnScreen => cmd(Key::Num0),
            Self::ActualPixels => cmd(Key::Num1),
            Self::ToggleInfo => Shortcut {
                cmd: false,
                shift: false,
                alt: false,
                ctrl: false,
                key: Key::F8,
            },
            Self::ToggleNavigator
            | Self::ToggleHistogram
            | Self::FlipView
            | Self::TogglePixelGrid
            | Self::ToggleSelectionEdges
            | Self::ToggleLayerEdges
            | Self::ShowAllExtras
            | Self::ShowNoExtras
            | Self::SnapToGuides
            | Self::SnapToGrid
            | Self::SnapToLayers
            | Self::SnapToSlices
            | Self::SnapToBounds
            | Self::SnapToAll
            | Self::SnapToNone
            | Self::ScreenMode(_)
            | Self::StatusInfo(_)
            | Self::RulerUnits(_) => return None,
            Self::ToggleRulers => cmd(Key::R),
            Self::ToggleExtras => cmd(Key::H),
            Self::ToggleGuides => cmd(Key::Semicolon),
            Self::ToggleGrid => cmd(Key::Quote),
            Self::LockGuides => alt_cmd(Key::Semicolon),
            Self::ToggleSnap => shift_cmd(Key::Semicolon),
            Self::HideApp => Shortcut {
                ctrl: true,
                ..cmd(Key::H)
            },
            Self::TransformRotate180
            | Self::TransformRotate90Clockwise
            | Self::TransformRotate90CounterClockwise
            | Self::TransformFlipHorizontal
            | Self::TransformFlipVertical
            | Self::TransformIn(..)
            | Self::TransformSelection
            | Self::RenameLayer
            | Self::RotateArbitrary
            | Self::DeselectLayers
            | Self::LinkLayers
            | Self::SelectLinkedLayers
            | Self::Align(_)
            | Self::Distribute(_)
            | Self::NewGroup
            | Self::NewGroupFromLayers
            | Self::ArrangeReverse => return None,
            Self::LockLayers | Self::ToggleLockAll => cmd(Key::Slash),
            Self::GroupLayers => cmd(Key::G),
            Self::UngroupLayers => shift_cmd(Key::G),
            Self::SelectAllLayers => Shortcut {
                alt: true,
                ..cmd(Key::A)
            },
            Self::Rotate180
            | Self::Rotate90Clockwise
            | Self::Rotate90CounterClockwise
            | Self::FlipCanvasHorizontal
            | Self::FlipCanvasVertical
            | Self::Crop
            | Self::Trim
            | Self::RevealAll
            | Self::Equalize
            | Self::Threshold
            | Self::Posterize
            | Self::Exposure
            | Self::Average
            | Self::Solarize
            | Self::Clouds
            | Self::DifferenceClouds
            | Self::Blur
            | Self::BlurMore
            | Self::Sharpen
            | Self::SharpenMore
            | Self::FindEdges
            | Self::MotionBlur
            | Self::Twirl
            | Self::Crystallize
            | Self::Pointillize
            | Self::Diffuse
            | Self::Ripple
            | Self::Mezzotint
            | Self::Pinch
            | Self::Spherize
            | Self::PolarCoordinates
            | Self::Emboss
            | Self::GaussianBlur
            | Self::BoxBlur
            | Self::UnsharpMask
            | Self::AddNoise
            | Self::Median
            | Self::Minimum
            | Self::Maximum
            | Self::HighPass
            | Self::Offset
            | Self::Mosaic
            | Self::SurfaceBlur
            | Self::DustAndScratches
            | Self::Fragment
            | Self::CustomFilter
            | Self::Despeckle
            | Self::SharpenEdges
            | Self::TraceContour
            | Self::Wind => return None,
            Self::Invert => cmd(Key::I),
            Self::Levels => cmd(Key::L),
            Self::Curves => cmd(Key::M),
            Self::HueSaturation => cmd(Key::U),
            Self::ColorBalance => cmd(Key::B),
            Self::BlackWhite => Shortcut {
                shift: true,
                ..alt_cmd(Key::B)
            },
            Self::AutoTone => shift_cmd(Key::L),
            Self::AutoContrast => Shortcut {
                shift: true,
                ..alt_cmd(Key::L)
            },
            Self::AutoColor => shift_cmd(Key::B),
            Self::BrightnessContrast
            | Self::Vibrance
            | Self::PhotoFilter
            | Self::GradientMap
            | Self::ChannelMixer
            | Self::SelectiveColor => {
                return None;
            }
            Self::LastFilter => Shortcut {
                cmd: true,
                shift: false,
                alt: false,
                ctrl: true,
                key: Key::F,
            },
            Self::Desaturate => shift_cmd(Key::U),
            Self::Fade => shift_cmd(Key::F),
            Self::FitLayers
            | Self::Zoom200
            | Self::PrintSize
            | Self::ActualSize
            | Self::ClearGuides
            | Self::NewGuide => {
                return None;
            }
            Self::DeleteLayer
            | Self::ToggleHistory
            | Self::DuplicateLayer
            | Self::LayerFromBackground
            | Self::DeleteHiddenLayers
            | Self::FlattenImage => return None,
        })
    }

    fn orientation(self) -> Option<Orientation> {
        Some(match self {
            Self::Rotate180 => Orientation::Rotate180,
            Self::Rotate90Clockwise => Orientation::Rotate90Clockwise,
            Self::Rotate90CounterClockwise => Orientation::Rotate90CounterClockwise,
            Self::FlipCanvasHorizontal => Orientation::FlipHorizontal,
            Self::FlipCanvasVertical => Orientation::FlipVertical,
            _ => return None,
        })
    }

    fn arrange(self) -> Option<Arrange> {
        Some(match self {
            Self::BringToFront => Arrange::BringToFront,
            Self::BringForward => Arrange::BringForward,
            Self::SendBackward => Arrange::SendBackward,
            Self::SendToBack => Arrange::SendToBack,
            _ => return None,
        })
    }

    /// The check mark of View menu switches.
    pub fn checked(self, app: &AppState) -> Option<bool> {
        let v = &app.view;
        Some(match self {
            Self::ToggleRulers => v.rulers,
            Self::ToggleExtras => v.extras,
            Self::ToggleGuides => v.guides,
            Self::ToggleGrid => v.grid,
            Self::TogglePixelGrid => v.pixel_grid,
            Self::ToggleSelectionEdges => v.selection_edges,
            Self::ToggleLayerEdges => v.layer_edges,
            Self::ToggleSnap => v.snap,
            Self::ScreenMode(mode) => app.screen_mode == mode,
            Self::StatusInfo(info) => app.status_info == info,
            Self::RulerUnits(unit) => app.ruler_units == unit,
            Self::SnapToGuides => v.snap_guides,
            Self::SnapToGrid => v.snap_grid,
            Self::SnapToLayers => v.snap_layers,
            Self::SnapToSlices => v.snap_slices,
            Self::SnapToBounds => v.snap_bounds,
            Self::FlipView => app
                .active_doc
                .and_then(|id| app.docs.get(&id))
                .is_some_and(|d| d.view.flip),
            Self::LockGuides => v.lock_guides,
            Self::ToggleHistory => app.history_open,
            Self::ToggleInfo => app.floating.info,
            Self::ToggleNavigator => app.floating.navigator,
            Self::ToggleHistogram => app.floating.histogram,
            Self::QuickMask => app
                .active_doc
                .and_then(|id| app.docs.get(&id))
                .is_some_and(|d| d.doc.quick_mask.is_some()),
            _ => return None,
        })
    }

    fn is_clipboard(self) -> bool {
        matches!(
            self,
            Self::Cut | Self::Copy | Self::CopyMerged | Self::Paste | Self::PasteInPlace
        )
    }

    /// Whether the command can run in the current state (also drives menu
    /// item enabled states).
    pub fn enabled(self, app: &AppState) -> bool {
        // With a text field focused, Cut/Copy/Paste edit its text (even in
        // a dialog)
        if app.typing && (self.is_clipboard() || self == Self::SelectAll) {
            return true;
        }
        // Menus are disabled while a modal dialog is open, as in Photoshop
        if app.modal_open() {
            return false;
        }
        let doc = app.active_doc.and_then(|id| app.docs.get(&id));
        match self {
            Self::New
            | Self::Open
            | Self::ToggleHistory
            | Self::Quit
            | Self::HideApp
            | Self::ToggleInfo
            | Self::ToggleNavigator
            | Self::ToggleHistogram
            | Self::ToggleRulers
            | Self::ToggleExtras
            | Self::ToggleGuides
            | Self::ToggleGrid
            | Self::TogglePixelGrid
            | Self::ToggleSelectionEdges
            | Self::ToggleLayerEdges
            | Self::ShowAllExtras
            | Self::ShowNoExtras
            | Self::ToggleSnap
            | Self::SnapToGuides
            | Self::SnapToGrid
            | Self::SnapToLayers
            | Self::SnapToSlices
            | Self::SnapToBounds
            | Self::SnapToAll
            | Self::SnapToNone
            | Self::ScreenMode(_)
            | Self::StatusInfo(_)
            | Self::RulerUnits(_)
            | Self::LockGuides => true,
            Self::FlipView => doc.is_some(),
            Self::ClearGuides => doc.is_some_and(|d| !d.doc.guides.is_empty()),
            Self::Revert => doc.is_some_and(|d| d.path.is_some() && d.is_dirty()),
            Self::Undo | Self::ToggleLastState => doc.is_some_and(|d| d.history.can_undo()),
            Self::Redo => doc.is_some_and(|d| d.history.can_redo()),
            Self::DeleteLayer => doc.is_some_and(|d| {
                let selected = d.doc.selected_layers();
                let n = selected.len();
                n > 0
                    && n < d.doc.layers.len()
                    && !selected.iter().any(|&id| d.doc.in_locked_group(id))
            }),
            Self::SelectAllLayers => {
                doc.is_some_and(|d| d.doc.layers.iter().any(|l| !l.is_background))
            }
            Self::DeselectLayers => doc.is_some_and(|d| d.doc.active_layer.is_some()),
            Self::LinkLayers => doc.is_some_and(|d| {
                op_core::link::can_link(&d.doc) || op_core::link::can_unlink(&d.doc)
            }),
            Self::SelectLinkedLayers => {
                doc.is_some_and(|d| op_core::link::can_select_linked(&d.doc))
            }
            Self::Align(_) => {
                doc.is_some_and(|d| op_core::align::can_align_count(&d.doc, d.movable_layers()))
            }
            Self::GroupLayers | Self::NewGroupFromLayers => {
                doc.is_some_and(|d| layer_ops::can_group(&d.doc))
            }
            Self::UngroupLayers => doc.is_some_and(|d| {
                d.doc
                    .active_layer
                    .and_then(|id| d.doc.layer(id))
                    .is_some_and(|l| l.is_group())
            }),
            Self::NewGroup => doc.is_some(),
            Self::Distribute(_) => doc.is_some_and(|d| d.movable_layers() >= 3),
            Self::CloseOthers => app.docs.len() > 1,
            Self::Deselect
            | Self::SelectInverse
            | Self::ModifyBorder
            | Self::ModifySmooth
            | Self::ModifyExpand
            | Self::ModifyContract
            | Self::ModifyFeather
            | Self::Grow
            | Self::Similar => doc.is_some_and(|d| d.doc.selection().is_some()),
            Self::Reselect => doc.is_some_and(|d| d.doc.can_reselect()),
            Self::ToggleLayerVisibility | Self::DuplicateLayer | Self::LayerViaCopy => doc
                .and_then(|d| d.doc.active_layer.and_then(|id| d.doc.layer(id)))
                .is_some(),
            Self::LayerViaCut => {
                doc.is_some_and(|d| d.doc.selection().is_some() && d.doc.active_layer.is_some())
            }
            Self::Crop | Self::TransformSelection => {
                doc.is_some_and(|d| d.doc.selection().is_some())
            }
            Self::PasteInto | Self::PasteOutside => {
                doc.is_some_and(|d| d.doc.selection().is_some())
            }
            Self::MaskRevealAll | Self::MaskHideAll => {
                doc.is_some_and(|d| layer_ops::can_add_mask(&d.doc))
            }
            Self::MaskRevealSelection | Self::MaskHideSelection => {
                doc.is_some_and(|d| layer_ops::can_add_mask(&d.doc) && d.doc.selection().is_some())
            }
            Self::MaskDelete | Self::MaskApply | Self::MaskToggle => doc.is_some_and(|d| {
                d.doc
                    .active_layer
                    .and_then(|id| d.doc.layer(id))
                    .is_some_and(|l| l.mask.is_some())
            }),
            Self::TransformAgain => doc.is_some() && app.last_transform.is_some(),
            Self::LastFilter => doc.is_some() && app.last_filter.is_some(),
            Self::LayerFromBackground => doc.is_some_and(|d| d.doc.has_background()),
            // Photoshop renames the background through Layer from Background
            Self::RenameLayer => doc.is_some_and(|d| {
                d.doc
                    .active_layer
                    .and_then(|id| d.doc.layer(id))
                    .is_some_and(|l| !l.is_background)
            }),
            // Not for the background alone
            Self::LockLayers => doc.is_some_and(|d| layer_ops::selected_locks(&d.doc).is_some()),
            // On the background it says it can't, like Photoshop
            Self::ToggleLockAll => doc.is_some(),
            Self::DeleteHiddenLayers => doc.is_some_and(|d| {
                let layers = &d.doc.layers;
                layers.iter().any(|l| !l.visible) && layers.iter().any(|l| l.visible)
            }),
            Self::BringToFront | Self::BringForward | Self::SendBackward | Self::SendToBack => doc
                .is_some_and(|d| {
                    let arrange = self.arrange().expect("arrange command");
                    layer_ops::arrange_target(&d.doc, arrange).is_some()
                }),
            Self::MergeDown => doc.is_some_and(|d| {
                if d.doc.selected_layers().len() > 1 {
                    layer_ops::can_merge_selected(&d.doc)
                } else if d
                    .doc
                    .active_layer
                    .and_then(|id| d.doc.layer(id))
                    .is_some_and(|l| l.is_group())
                {
                    layer_ops::can_merge_group(&d.doc)
                } else {
                    layer_ops::can_merge_down(&d.doc)
                }
            }),
            Self::ArrangeReverse => doc.is_some_and(|d| layer_ops::can_reverse(&d.doc)),
            Self::MergeVisible => doc.is_some_and(|d| layer_ops::can_merge_visible(&d.doc)),
            Self::FlattenImage => doc.is_some_and(|d| {
                let layers = &d.doc.layers;
                !(layers.len() == 1 && layers[0].is_background)
            }),
            Self::Close
            | Self::Save
            | Self::SaveAs
            | Self::SaveACopy
            | Self::Fill
            | Self::FillForeground
            | Self::FillBackground
            | Self::Clear
            | Self::FreeTransform
            | Self::TransformIn(..)
            | Self::TransformRotate180
            | Self::TransformRotate90Clockwise
            | Self::TransformRotate90CounterClockwise
            | Self::TransformFlipHorizontal
            | Self::TransformFlipVertical
            | Self::Cut
            | Self::Copy
            | Self::CopyMerged
            | Self::Paste
            | Self::PasteInPlace
            | Self::SelectAll
            | Self::CloseAll
            | Self::ExportAs
            | Self::CanvasSize
            | Self::ImageSize
            | Self::Rotate180
            | Self::Rotate90Clockwise
            | Self::RotateArbitrary
            | Self::Rotate90CounterClockwise
            | Self::FlipCanvasHorizontal
            | Self::FlipCanvasVertical
            | Self::Trim
            | Self::RevealAll
            | Self::Invert
            | Self::Desaturate
            | Self::Equalize
            | Self::Threshold
            | Self::Posterize
            | Self::Levels
            | Self::HueSaturation
            | Self::Curves
            | Self::Exposure
            | Self::BrightnessContrast
            | Self::ColorBalance
            | Self::BlackWhite
            | Self::Vibrance
            | Self::ChannelMixer
            | Self::SelectiveColor
            | Self::PhotoFilter
            | Self::GradientMap
            | Self::AutoTone
            | Self::AutoContrast
            | Self::AutoColor
            | Self::Average
            | Self::Solarize
            | Self::Clouds
            | Self::DifferenceClouds
            | Self::Blur
            | Self::BlurMore
            | Self::Sharpen
            | Self::SharpenMore
            | Self::FindEdges
            | Self::SurfaceBlur
            | Self::DustAndScratches
            | Self::Fragment
            | Self::CustomFilter
            | Self::Despeckle
            | Self::SharpenEdges
            | Self::TraceContour
            | Self::Wind
            | Self::Twirl
            | Self::Crystallize
            | Self::Pointillize
            | Self::Diffuse
            | Self::Ripple
            | Self::Mezzotint
            | Self::Pinch
            | Self::Spherize
            | Self::PolarCoordinates
            | Self::MotionBlur
            | Self::Emboss
            | Self::GaussianBlur
            | Self::BoxBlur
            | Self::UnsharpMask
            | Self::AddNoise
            | Self::Median
            | Self::Minimum
            | Self::Maximum
            | Self::HighPass
            | Self::Offset
            | Self::Mosaic
            | Self::NewLayer
            | Self::NewLayerNoDialog
            | Self::ZoomIn
            | Self::ZoomOut
            | Self::FitOnScreen
            | Self::FitLayers
            | Self::ActualPixels
            | Self::Zoom200
            | Self::PrintSize
            | Self::ActualSize
            | Self::NewGuide
            | Self::QuickMask => doc.is_some(),
            Self::Fade => doc.is_some_and(|d| d.can_fade()),
        }
    }
}

/// Commands that have keyboard shortcuts, most specific first: egui ignores
/// extra Shift/Alt when matching, so Shift+Cmd+Z must be checked before Cmd+Z.
const SHORTCUT_ORDER: &[Command] = &[
    Command::ExportAs,
    Command::BlackWhite,
    Command::AutoContrast,
    Command::ModifyFeather,
    Command::HideApp,
    Command::ToggleSnap,
    Command::LockGuides,
    Command::SaveAs,
    Command::SaveACopy,
    Command::Revert,
    Command::LastFilter,
    Command::PasteInto,
    Command::CopyMerged,
    Command::PasteInPlace,
    Command::LayerViaCut,
    Command::TransformAgain,
    Command::BringToFront,
    Command::SendToBack,
    Command::MergeVisible,
    Command::Fill,
    Command::FillForeground,
    Command::FillBackground,
    Command::Reselect,
    Command::SelectInverse,
    Command::Fade,
    Command::Desaturate,
    Command::AutoTone,
    Command::AutoColor,
    Command::Redo,
    Command::ToggleLastState,
    Command::NewLayerNoDialog,
    Command::NewLayer,
    Command::ToggleLockAll,
    Command::CanvasSize,
    Command::ImageSize,
    Command::CloseAll,
    Command::CloseOthers,
    Command::Undo,
    Command::FreeTransform,
    Command::Save,
    Command::Quit,
    Command::Invert,
    Command::Levels,
    Command::Curves,
    Command::HueSaturation,
    Command::ColorBalance,
    Command::LayerViaCopy,
    Command::BringForward,
    Command::SendBackward,
    Command::MergeDown,
    Command::Cut,
    Command::Copy,
    Command::Paste,
    Command::New,
    Command::Open,
    Command::Close,
    Command::ToggleLayerVisibility,
    Command::SelectAllLayers,
    Command::SelectAll,
    Command::UngroupLayers,
    Command::GroupLayers,
    Command::Deselect,
    Command::ZoomIn,
    Command::ZoomOut,
    Command::FitOnScreen,
    Command::ActualPixels,
    Command::ToggleRulers,
    Command::ToggleExtras,
    Command::ToggleGuides,
    Command::ToggleGrid,
    Command::ToggleInfo,
];

/// Shortcuts the macOS menu bar can't catch. Zoom In's menu item shows Cmd++
/// like Photoshop, which macOS only matches with Shift held, so the plain
/// Cmd+= key is handled here.
#[cfg(target_os = "macos")]
pub fn from_shortcuts_beside_menu(ctx: &egui::Context) -> Vec<Command> {
    ctx.input_mut(|i| {
        let mut out = Vec::new();
        if i.consume_key(Modifiers::COMMAND, Key::Equals) {
            out.push(Command::ZoomIn);
        }
        // Cmd+/ reaches egui when Lock Layers... is disabled (only the
        // background selected); Photoshop still answers it with an alert
        if i.consume_key(Modifiers::COMMAND, Key::Slash) {
            out.push(Command::ToggleLockAll);
        }
        out
    })
}

/// Command shortcuts typed into egui, used where there is no native menu bar
/// (other platforms, and headless tests on macOS). On macOS the native menu bar handles
/// these instead, so this is only used on other platforms.
pub fn from_shortcuts(ctx: &egui::Context, app: &AppState) -> Vec<Command> {
    let mut out = Vec::new();
    // Every command is disabled under a modal dialog; leave its keys (such
    // as Cmd+D for "Don't Save") to the dialog
    if app.modal_open() {
        return out;
    }
    let typing = ctx.egui_wants_keyboard_input();
    ctx.input_mut(|i| {
        for &command in SHORTCUT_ORDER {
            // Delete-key fills would eat text editing keys, and text fields
            // handle their own Cut/Copy/Paste
            if typing
                && (command.is_clipboard()
                    || matches!(
                        command,
                        Command::FillForeground | Command::FillBackground | Command::SelectAll
                    ))
            {
                continue;
            }
            let s = command.shortcut().expect("listed commands have shortcuts");
            let mut hit = i.consume_key(s.modifiers(), s.key);
            // Cmd+Shift+= arrives as Cmd+Plus on some layouts
            if command == Command::ZoomIn {
                hit |= i.consume_key(s.modifiers(), Key::Plus);
            }
            if hit {
                out.push(command);
            }
        }
        // egui-winit turns Cmd+X/C/V into these events instead of key presses
        if !typing {
            let shift = i.modifiers.shift;
            for event in &i.events {
                out.push(match event {
                    egui::Event::Cut => Command::Cut,
                    egui::Event::Copy if shift => Command::CopyMerged,
                    egui::Event::Copy => Command::Copy,
                    egui::Event::Paste(_) if shift && i.modifiers.alt => Command::PasteInto,
                    egui::Event::Paste(_) if shift => Command::PasteInPlace,
                    egui::Event::Paste(_) => Command::Paste,
                    _ => continue,
                });
            }
        }
        if !typing
            && (i.consume_key(Modifiers::NONE, Key::Backspace)
                || i.consume_key(Modifiers::NONE, Key::Delete))
        {
            out.push(Command::Clear);
        }
    });
    out
}

pub fn run(command: Command, ctx: &egui::Context, app: &mut AppState) {
    if !command.enabled(app) {
        return;
    }
    // The status bar's Timing: how long a command that changed the
    // document took
    let revision = |app: &mut AppState| app.active().map(|d| d.doc.revision());
    let (start, before) = (std::time::Instant::now(), revision(app));
    run_command(command, ctx, app);
    if revision(app) != before {
        app.last_timing = start.elapsed().as_secs_f32();
    }
}

fn run_command(command: Command, ctx: &egui::Context, app: &mut AppState) {
    let ppp = ctx.pixels_per_point();
    // With a text field focused, Select All selects its text (the native
    // menu takes Cmd+A before the field sees it)
    if command == Command::SelectAll && app.typing {
        app.forward_events.push(egui::Event::Key {
            key: Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        });
        return;
    }
    match command {
        Command::New => actions::new_dialog(app),
        Command::Open => actions::open_dialog(app),
        Command::Close => actions::close_active(app),
        Command::Save | Command::SaveAs | Command::SaveACopy => {
            if let Some(id) = app.active_doc {
                match command {
                    Command::Save => actions::save(app, id),
                    Command::SaveAs => actions::save_as(app, id, false),
                    _ => actions::save_as(app, id, true),
                };
            }
        }
        Command::Revert => actions::revert(app),
        Command::Quit => actions::quit(app),
        Command::CloseAll => actions::close_all(app),
        Command::CloseOthers => actions::close_others(app),
        Command::ExportAs => actions::export_dialog(app),
        Command::ToggleHistory => app.history_open = !app.history_open,
        Command::Fill => app.fill_dialog = Some(Default::default()),
        Command::FillForeground | Command::FillBackground => {
            let color = if command == Command::FillForeground {
                app.foreground
            } else {
                app.background
            };
            let [r, g, b, _] = color.to_rgba8();
            if let Some(state) = app.active() {
                let result = op_core::fill::fill(&mut state.doc, [r, g, b], Default::default());
                match result {
                    Ok(()) => state.record_fadeable("Fill"),
                    Err(e) => app.alert = Some(e.message("Fill")),
                }
            }
        }
        Command::Clear => {
            let [r, g, b, _] = app.background.to_rgba8();
            if let Some(state) = app.active() {
                if state.doc.selection().is_none() {
                    // Without a pixel selection, Delete removes the layer
                    if state.doc.layers.len() > 1 {
                        crate::panels::delete_active_layer(state);
                    }
                } else {
                    match op_core::fill::clear(&mut state.doc, [r, g, b]) {
                        Ok(()) => state.record("Clear"),
                        Err(e) => app.alert = Some(e.message("Clear")),
                    }
                }
            }
        }
        Command::Cut
        | Command::Copy
        | Command::CopyMerged
        | Command::Paste
        | Command::PasteInPlace
        | Command::PasteInto
        | Command::PasteOutside => actions::clipboard(command, app, ppp),
        Command::LayerViaCopy | Command::LayerViaCut => {
            let [r, g, b, _] = app.background.to_rgba8();
            let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id)) else {
                return;
            };
            let (name, result) = if command == Command::LayerViaCopy {
                ("Layer Via Copy", layer_ops::via_copy(&mut state.doc))
            } else {
                (
                    "Layer Via Cut",
                    layer_ops::via_cut(&mut state.doc, [r, g, b]),
                )
            };
            match result {
                Ok(_) => state.record(name),
                Err(e) => app.alert = Some(e.message(name)),
            }
        }
        Command::Invert
        | Command::Desaturate
        | Command::Equalize
        | Command::AutoTone
        | Command::AutoContrast
        | Command::AutoColor => {
            let adjustment = match command {
                Command::Invert => Adjustment::Invert,
                Command::Desaturate => Adjustment::Desaturate,
                Command::AutoTone => Adjustment::AutoTone,
                Command::AutoContrast => Adjustment::AutoContrast,
                Command::AutoColor => Adjustment::AutoColor,
                _ => Adjustment::Equalize,
            };
            if let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id)) {
                // With a selection, Equalize asks how to use it
                if adjustment == Adjustment::Equalize && state.doc.selection().is_some() {
                    match adjust::check(&state.doc) {
                        Ok(()) => app.equalize_dialog = Some(Default::default()),
                        Err(e) => app.alert = Some(e.message("Equalize")),
                    }
                    return;
                }
                match adjust::apply(&mut state.doc, adjustment) {
                    Ok(()) => state.record_fadeable(adjustment.name()),
                    Err(e) => app.alert = Some(e.message(adjustment.name())),
                }
            }
        }
        Command::Average
        | Command::Solarize
        | Command::Clouds
        | Command::DifferenceClouds
        | Command::Blur
        | Command::BlurMore
        | Command::Sharpen
        | Command::SharpenMore
        | Command::FindEdges
        | Command::Fragment
        | Command::Despeckle
        | Command::SharpenEdges
        | Command::LastFilter => {
            let filter = match command {
                Command::Average => Some(Filter::Average),
                Command::Solarize => Some(Filter::Solarize),
                Command::Blur => Some(Filter::Blur),
                Command::BlurMore => Some(Filter::BlurMore),
                Command::Sharpen => Some(Filter::Sharpen),
                Command::SharpenMore => Some(Filter::SharpenMore),
                Command::FindEdges => Some(Filter::FindEdges),
                Command::Fragment => Some(Filter::Fragment),
                Command::Despeckle => Some(Filter::Despeckle),
                Command::SharpenEdges => Some(Filter::SharpenEdges),
                Command::Clouds => Some(Filter::Clouds {
                    foreground: [0; 3],
                    background: [0; 3],
                    seed: 0,
                }),
                Command::DifferenceClouds => Some(Filter::DifferenceClouds {
                    foreground: [0; 3],
                    background: [0; 3],
                    seed: 0,
                }),
                _ => app.last_filter,
            };
            // The clouds take the current colors and a new pattern each
            // time, Last Filter included
            let filter = filter.map(|f| with_cloud_colors(f, app));
            let [r, g, b, _] = app.background.to_rgba8();
            if let Some(filter) = filter
                && let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id))
            {
                match op_core::filter::apply(&mut state.doc, filter, [r, g, b]) {
                    Ok(()) => {
                        state.record_fadeable(filter.name());
                        app.last_filter = Some(filter);
                    }
                    Err(e) => app.alert = Some(e.message(filter.name())),
                }
            }
        }
        Command::Threshold
        | Command::Posterize
        | Command::Levels
        | Command::HueSaturation
        | Command::Exposure
        | Command::Curves
        | Command::BrightnessContrast
        | Command::ColorBalance
        | Command::BlackWhite
        | Command::Vibrance
        | Command::ChannelMixer
        | Command::SelectiveColor
        | Command::PhotoFilter
        | Command::GradientMap
        | Command::GaussianBlur
        | Command::BoxBlur
        | Command::UnsharpMask
        | Command::AddNoise
        | Command::Median
        | Command::Minimum
        | Command::Maximum
        | Command::HighPass
        | Command::Offset
        | Command::Mosaic
        | Command::MotionBlur
        | Command::Emboss
        | Command::Twirl
        | Command::Crystallize
        | Command::Pointillize
        | Command::Diffuse
        | Command::Ripple
        | Command::Mezzotint
        | Command::Pinch
        | Command::Spherize
        | Command::PolarCoordinates
        | Command::SurfaceBlur
        | Command::DustAndScratches
        | Command::CustomFilter
        | Command::TraceContour
        | Command::Wind => {
            let (kind, name) = match command {
                Command::Threshold => (AdjustKind::Threshold, "Threshold"),
                Command::Posterize => (AdjustKind::Posterize, "Posterize"),
                Command::Levels => (AdjustKind::Levels, "Levels"),
                Command::HueSaturation => (AdjustKind::HueSaturation, "Hue/Saturation"),
                Command::Exposure => (AdjustKind::Exposure, "Exposure"),
                Command::Curves => (AdjustKind::Curves, "Curves"),
                Command::BrightnessContrast => {
                    (AdjustKind::BrightnessContrast, "Brightness/Contrast")
                }
                Command::ColorBalance => (AdjustKind::ColorBalance, "Color Balance"),
                Command::BlackWhite => (AdjustKind::BlackWhite, "Black & White"),
                Command::Vibrance => (AdjustKind::Vibrance, "Vibrance"),
                Command::ChannelMixer => (AdjustKind::ChannelMixer, "Channel Mixer"),
                Command::SelectiveColor => (AdjustKind::SelectiveColor, "Selective Color"),
                Command::PhotoFilter => (AdjustKind::PhotoFilter, "Photo Filter"),
                Command::GradientMap => (AdjustKind::GradientMap, "Gradient Map"),
                Command::GaussianBlur => (AdjustKind::GaussianBlur, "Gaussian Blur"),
                Command::BoxBlur => (AdjustKind::BoxBlur, "Box Blur"),
                Command::UnsharpMask => (AdjustKind::UnsharpMask, "Unsharp Mask"),
                Command::AddNoise => (AdjustKind::AddNoise, "Add Noise"),
                Command::Median => (AdjustKind::Median, "Median"),
                Command::Minimum => (AdjustKind::Minimum, "Minimum"),
                Command::Maximum => (AdjustKind::Maximum, "Maximum"),
                Command::HighPass => (AdjustKind::HighPass, "High Pass"),
                Command::Offset => (AdjustKind::Offset, "Offset"),
                Command::MotionBlur => (AdjustKind::MotionBlur, "Motion Blur"),
                Command::Emboss => (AdjustKind::Emboss, "Emboss"),
                Command::Twirl => (AdjustKind::Twirl, "Twirl"),
                Command::Crystallize => (AdjustKind::Crystallize, "Crystallize"),
                Command::Pointillize => (AdjustKind::Pointillize, "Pointillize"),
                Command::Diffuse => (AdjustKind::Diffuse, "Diffuse"),
                Command::Ripple => (AdjustKind::Ripple, "Ripple"),
                Command::Mezzotint => (AdjustKind::Mezzotint, "Mezzotint"),
                Command::Pinch => (AdjustKind::Pinch, "Pinch"),
                Command::Spherize => (AdjustKind::Spherize, "Spherize"),
                Command::PolarCoordinates => (AdjustKind::PolarCoordinates, "Polar Coordinates"),
                Command::SurfaceBlur => (AdjustKind::SurfaceBlur, "Surface Blur"),
                Command::DustAndScratches => (AdjustKind::DustAndScratches, "Dust & Scratches"),
                Command::CustomFilter => (AdjustKind::Custom, "Custom"),
                Command::TraceContour => (AdjustKind::TraceContour, "Trace Contour"),
                Command::Wind => (AdjustKind::Wind, "Wind"),
                _ => (AdjustKind::Mosaic, "Mosaic"),
            };
            let rgb = |c: op_core::Color| {
                let [r, g, b, _] = c.to_rgba8();
                [r, g, b]
            };
            let colors = (rgb(app.foreground), rgb(app.background));
            if let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id)) {
                match adjust::check(&state.doc) {
                    Ok(()) => {
                        let histogram = if kind == AdjustKind::Levels {
                            adjust::channel_histogram(&state.doc)
                        } else {
                            adjust::luminosity_histogram(&state.doc)
                        };
                        let before = state.doc.snapshot();
                        let mut dialog = AdjustDialog::new(kind, histogram, before);
                        if let Some(values) = app.filter_settings.get(&kind) {
                            dialog.restore(values);
                        }
                        // Gradient Map runs from the foreground to the background color
                        dialog.set_gradient_colors(colors);
                        // Colorize starts from the foreground color's hue
                        dialog.set_colorize_hue(adjust::hue_of(colors.0).round() as i32);
                        dialog.set_channel_histograms(adjust::rgb_histograms(&state.doc));
                        app.adjust_dialog = Some(dialog);
                    }
                    Err(e) => app.alert = Some(e.message(name)),
                }
            }
        }
        Command::TransformSelection => {
            if let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id)) {
                crate::free_transform::start_selection(state);
            }
        }
        Command::TransformIn(mode, name) => {
            if let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id))
                && let Err(e) = crate::free_transform::start_in(state, mode)
            {
                app.alert = Some(e.message(name));
            }
        }
        Command::FreeTransform
        | Command::TransformAgain
        | Command::TransformRotate180
        | Command::TransformRotate90Clockwise
        | Command::TransformRotate90CounterClockwise
        | Command::TransformFlipHorizontal
        | Command::TransformFlipVertical => {
            let [r, g, b, _] = app.background.to_rgba8();
            let last = app.last_transform;
            let Some(state) = app.active_doc.and_then(|id| app.docs.get_mut(&id)) else {
                return;
            };
            if command == Command::FreeTransform {
                if let Err(e) = crate::free_transform::start(state) {
                    app.alert = Some(e.message("Free Transform"));
                }
                return;
            }
            let fixed = match command {
                Command::TransformRotate180 => Some(FixedTransform::Rotate180),
                Command::TransformRotate90Clockwise => Some(FixedTransform::Rotate90Clockwise),
                Command::TransformRotate90CounterClockwise => {
                    Some(FixedTransform::Rotate90CounterClockwise)
                }
                Command::TransformFlipHorizontal => Some(FixedTransform::FlipHorizontal),
                Command::TransformFlipVertical => Some(FixedTransform::FlipVertical),
                _ => None,
            };
            let (name, affine) = match (fixed, last) {
                (Some(f), _) => match transform::bounds(&state.doc) {
                    Ok(b) => (f.name(), transform::Projective::from_affine(f.affine(b))),
                    Err(e) => {
                        app.alert = Some(e.message(f.name()));
                        return;
                    }
                },
                (None, Some(m)) => ("Transform Again", m),
                (None, None) => return,
            };
            match transform::transform(&mut state.doc, affine, [r, g, b]) {
                Ok(()) => {
                    state.record(name);
                    app.last_transform = Some(affine);
                }
                Err(e) => app.alert = Some(e.message(name)),
            }
        }
        Command::ImageSize => {
            if let Some(state) = app.active() {
                let d = &state.doc;
                let preview = crate::dialogs::ImageSizePreview {
                    rgba: std::sync::Arc::new(d.composite_rgba8()),
                    width: d.width,
                    height: d.height,
                };
                let dialog = crate::dialogs::ImageSizeDialog::new(d.width, d.height, d.resolution)
                    .with_preview(preview);
                app.image_size_dialog = Some(dialog);
            }
        }
        Command::ToggleInfo => app.floating.toggle(crate::panels::floating::Floating::Info),
        Command::ToggleNavigator => app
            .floating
            .toggle(crate::panels::floating::Floating::Navigator),
        Command::ToggleHistogram => app
            .floating
            .toggle(crate::panels::floating::Floating::Histogram),
        Command::ToggleRulers => app.view.rulers = !app.view.rulers,
        Command::ToggleExtras => app.view.extras = !app.view.extras,
        Command::ToggleGuides => app.view.guides = !app.view.guides,
        Command::ToggleGrid => app.view.grid = !app.view.grid,
        Command::TogglePixelGrid => app.view.pixel_grid = !app.view.pixel_grid,
        Command::ToggleSelectionEdges => app.view.selection_edges = !app.view.selection_edges,
        Command::ToggleLayerEdges => app.view.layer_edges = !app.view.layer_edges,
        Command::ShowAllExtras | Command::ShowNoExtras => {
            // Every Show item OpenPhoto has, on or off together
            let on = command == Command::ShowAllExtras;
            let v = &mut app.view;
            (v.layer_edges, v.selection_edges, v.grid, v.guides) = (on, on, on, on);
            (v.smart_guides, v.pixel_grid) = (on, on);
        }
        Command::FlipView => {
            if let Some(doc) = app.active() {
                doc.view.flip = !doc.view.flip;
            }
        }
        Command::LockGuides => app.view.lock_guides = !app.view.lock_guides,
        Command::ToggleSnap => app.view.snap = !app.view.snap,
        Command::ScreenMode(mode) => app.set_screen_mode(mode),
        Command::Fade => {
            if let Some(source) = app.active().and_then(|d| d.fade_source()) {
                app.fade_dialog = Some(crate::state::FadeState {
                    dialog: Default::default(),
                    source,
                    shown: None,
                });
            }
        }
        Command::StatusInfo(info) => app.status_info = info,
        Command::RulerUnits(unit) => app.ruler_units = unit,
        Command::SnapToGuides => app.view.snap_guides = !app.view.snap_guides,
        Command::SnapToGrid => app.view.snap_grid = !app.view.snap_grid,
        Command::SnapToLayers => app.view.snap_layers = !app.view.snap_layers,
        Command::SnapToSlices => app.view.snap_slices = !app.view.snap_slices,
        Command::SnapToBounds => app.view.snap_bounds = !app.view.snap_bounds,
        Command::SnapToAll | Command::SnapToNone => {
            let on = command == Command::SnapToAll;
            let v = &mut app.view;
            (v.snap_guides, v.snap_grid, v.snap_layers) = (on, on, on);
            (v.snap_slices, v.snap_bounds) = (on, on);
        }
        Command::NewGuide => app.new_guide_dialog = Some(Default::default()),
        Command::HideApp => {
            #[cfg(target_os = "macos")]
            crate::app_kit::hide_app();
        }
        Command::ModifyBorder
        | Command::ModifySmooth
        | Command::ModifyExpand
        | Command::ModifyContract
        | Command::ModifyFeather => {
            let kind = match command {
                Command::ModifyBorder => ModifyKind::Border,
                Command::ModifySmooth => ModifyKind::Smooth,
                Command::ModifyExpand => ModifyKind::Expand,
                Command::ModifyContract => ModifyKind::Contract,
                _ => ModifyKind::Feather,
            };
            app.modify_dialog = Some(crate::dialogs::ModifyDialog::new(kind));
        }
        Command::QuickMask => {
            if let Some(state) = app.active() {
                crate::toolbar::toggle_quick_mask(state);
            }
        }
        Command::Grow | Command::Similar => {
            let options = app.wand.region;
            if let Some(state) = app.active()
                && let Some(s) = op_core::fill::grow(&state.doc, &options, command == Command::Grow)
            {
                state.doc.set_selection(Some(s));
                state.record(if command == Command::Grow {
                    "Grow"
                } else {
                    "Similar"
                });
            }
        }
        Command::RevealAll => {
            let background = app.background;
            if let Some(state) = app.active()
                && op_core::image_ops::reveal_all(&mut state.doc, background)
            {
                state.record("Reveal All");
            }
        }
        Command::Trim => {
            if let Some(state) = app.active() {
                let dialog = crate::dialogs::TrimDialog::new(state.doc.has_background());
                app.trim_dialog = Some(dialog);
            }
        }
        Command::CanvasSize => {
            if let Some(state) = app.active() {
                let dialog = crate::dialogs::CanvasSizeDialog::new(&state.doc);
                app.canvas_size_dialog = Some(dialog);
            }
        }
        _ => {
            let skip_flatten_prompt = app.skip_flatten_prompt;
            let Some(state) = app.active() else {
                return;
            };
            match command {
                Command::Undo => {
                    state.undo();
                }
                Command::Redo => {
                    state.redo();
                }
                Command::ToggleLastState => {
                    state.toggle_last_state();
                }
                Command::ToggleLayerVisibility => crate::panels::toggle_active_visibility(state),
                Command::SelectAll => {
                    let (w, h) = (state.doc.width, state.doc.height);
                    state.doc.set_selection(Some(op_core::Selection::all(w, h)));
                    state.record("Select All");
                }
                Command::Deselect => {
                    state.doc.set_selection(None);
                    state.record("Deselect");
                }
                Command::Reselect => {
                    state.doc.reselect();
                    state.record("Reselect");
                }
                Command::SelectInverse => {
                    let inverse = state.doc.selection().map(|s| s.inverse());
                    state.doc.set_selection(inverse);
                    state.record("Select Inverse");
                }
                Command::Rotate180
                | Command::Rotate90Clockwise
                | Command::Rotate90CounterClockwise
                | Command::FlipCanvasHorizontal
                | Command::FlipCanvasVertical => {
                    let orientation = command.orientation().expect("rotation command");
                    image_ops::reorient(&mut state.doc, orientation);
                    state.record(orientation.history_name());
                }
                Command::RotateArbitrary => {
                    app.rotate_dialog = Some(crate::dialogs::RotateCanvasDialog::default());
                }
                Command::Crop => {
                    if image_ops::crop_to_selection(&mut state.doc) {
                        state.record("Crop");
                    }
                }
                Command::MaskRevealAll
                | Command::MaskHideAll
                | Command::MaskRevealSelection
                | Command::MaskHideSelection => {
                    let kind = match command {
                        Command::MaskRevealAll => layer_ops::NewMask::RevealAll,
                        Command::MaskHideAll => layer_ops::NewMask::HideAll,
                        Command::MaskRevealSelection => layer_ops::NewMask::RevealSelection,
                        _ => layer_ops::NewMask::HideSelection,
                    };
                    if layer_ops::add_mask(&mut state.doc, kind) {
                        state.record("Add Layer Mask");
                    }
                }
                Command::MaskDelete => {
                    if layer_ops::delete_mask(&mut state.doc) {
                        state.record("Delete Layer Mask");
                    }
                }
                Command::MaskApply => {
                    if layer_ops::apply_mask(&mut state.doc) {
                        state.record("Apply Layer Mask");
                    }
                }
                Command::MaskToggle => match layer_ops::toggle_mask(&mut state.doc) {
                    Some(true) => state.record("Enable Layer Mask"),
                    Some(false) => state.record("Disable Layer Mask"),
                    None => {}
                },
                Command::NewLayer => {
                    let name = state.doc.next_layer_name();
                    app.new_layer_dialog = Some(crate::dialogs::NewLayerDialog::new(name));
                }
                Command::NewLayerNoDialog => crate::panels::new_layer(state),
                Command::RenameLayer => {
                    if let Some(layer) = state.doc.active_layer.and_then(|id| state.doc.layer(id))
                        && !layer.is_background
                    {
                        state.renaming = Some((layer.id, layer.name.clone()));
                    }
                }
                Command::LockLayers => {
                    if let Some(locks) = layer_ops::selected_locks(&state.doc) {
                        app.lock_dialog = Some(crate::dialogs::LockLayersDialog::new(locks));
                    }
                }
                Command::ToggleLockAll => match layer_ops::toggle_lock_all(&mut state.doc) {
                    Some(true) => state.record("Lock Layer"),
                    Some(false) => state.record("Unlock Layer"),
                    None => {
                        app.alert = Some(
                            "The command \u{201c}Set\u{201d} is not currently available.".into(),
                        );
                    }
                },
                Command::DuplicateLayer => {
                    let current = state.doc.id;
                    let source = state
                        .doc
                        .active_layer
                        .and_then(|id| state.doc.layer(id))
                        .map(|l| l.name.clone());
                    let name = layer_ops::duplicate_name(&state.doc);
                    if let (Some(source), Some(name)) = (source, name) {
                        // The current document first, then the others
                        let mut documents: Vec<(op_core::DocId, String)> = Vec::new();
                        for id in std::iter::once(current).chain(app.doc_order.iter().copied()) {
                            if documents.iter().any(|(d, _)| *d == id) {
                                continue;
                            }
                            if let Some(d) = app.docs.get(&id) {
                                documents.push((id, d.doc.title.clone()));
                            }
                        }
                        let title = format!("Untitled-{}", app.untitled_counter + 1);
                        app.duplicate_dialog = Some(crate::dialogs::DuplicateLayerDialog::new(
                            source, name, documents, title,
                        ));
                    }
                }
                Command::LayerFromBackground => {
                    if state.doc.has_background() {
                        app.new_layer_dialog =
                            Some(crate::dialogs::NewLayerDialog::from_background());
                    }
                }
                Command::DeleteHiddenLayers => {
                    if layer_ops::delete_hidden(&mut state.doc) {
                        state.record("Delete Hidden Layers");
                    }
                }
                Command::BringToFront
                | Command::BringForward
                | Command::SendBackward
                | Command::SendToBack => {
                    let arrange = command.arrange().expect("arrange command");
                    if layer_ops::arrange(&mut state.doc, arrange) {
                        state.record("Layer Order");
                    }
                }
                Command::MergeDown => {
                    // With several layers selected, Cmd+E is Merge Layers
                    if state.doc.selected_layers().len() > 1 {
                        if layer_ops::merge_selected(&mut state.doc) {
                            state.record("Merge Layers");
                        }
                    } else if layer_ops::can_merge_group(&state.doc) {
                        if layer_ops::merge_group(&mut state.doc) {
                            state.record("Merge Group");
                        }
                    } else if layer_ops::merge_down(&mut state.doc) {
                        state.record("Merge Down");
                    }
                }
                Command::ArrangeReverse => {
                    if layer_ops::reverse_selected(&mut state.doc) {
                        state.record("Reverse");
                    }
                }
                Command::SelectAllLayers => {
                    state.doc.select_all_layers();
                }
                Command::DeselectLayers => state.doc.deselect_layers(),
                Command::LinkLayers => {
                    if let Some(name) = op_core::link::toggle(&mut state.doc) {
                        state.record(name);
                    }
                }
                Command::SelectLinkedLayers => {
                    op_core::link::select_linked(&mut state.doc);
                }
                Command::GroupLayers => {
                    if layer_ops::group_selected(&mut state.doc).is_some() {
                        state.record("Group Layers");
                    }
                }
                Command::UngroupLayers => {
                    if layer_ops::ungroup(&mut state.doc) {
                        state.record("Ungroup Layers");
                    }
                }
                Command::NewGroup | Command::NewGroupFromLayers => {
                    let kind = if command == Command::NewGroup {
                        crate::dialogs::NewLayerKind::Group
                    } else {
                        crate::dialogs::NewLayerKind::GroupFromLayers
                    };
                    let name = state.doc.next_group_name();
                    app.new_layer_dialog = Some(crate::dialogs::NewLayerDialog::group(name, kind));
                }
                Command::Align(how) => {
                    if op_core::align::align(&mut state.doc, how) {
                        state.record(how.name());
                    }
                }
                Command::Distribute(how) => {
                    if op_core::align::distribute(&mut state.doc, how) {
                        state.record(how.name());
                    }
                }
                Command::MergeVisible => {
                    if layer_ops::merge_visible(&mut state.doc) {
                        state.record("Merge Visible");
                    }
                }
                Command::FlattenImage => {
                    // Photoshop asks before throwing hidden layers away
                    let hidden = state.doc.layers.iter().any(|l| !l.visible);
                    if hidden && !skip_flatten_prompt {
                        app.flatten_prompt = Some(crate::dialogs::alert::Alert::caution(
                            "Discard hidden layers?",
                        ));
                    } else {
                        layer_ops::flatten(&mut state.doc);
                        state.record("Flatten Image");
                    }
                }
                Command::DeleteLayer => app.delete_layers(),
                Command::ZoomIn => document_view::zoom_step(state, true, ppp),
                Command::ZoomOut => document_view::zoom_step(state, false, ppp),
                Command::FitOnScreen => document_view::fit_on_screen(state, ppp),
                Command::ActualPixels => document_view::actual_pixels(state, ppp),
                Command::Zoom200 => document_view::zoom_to(state, 2.0, ppp),
                Command::PrintSize => document_view::print_size(state, ppp),
                Command::ActualSize => document_view::actual_size(state, ppp),
                Command::FitLayers => document_view::fit_layers(state, ppp),
                Command::ClearGuides => {
                    state.doc.guides.clear();
                    state.record("Clear Guides");
                }
                _ => unreachable!("handled above"),
            }
        }
    }
}

/// Clouds and Difference Clouds with the current foreground and background
/// colors and a fresh pattern; other filters as they are.
fn with_cloud_colors(filter: op_core::filter::Filter, app: &AppState) -> op_core::filter::Filter {
    use op_core::filter::Filter;
    let rgb = |c: op_core::Color| {
        let [r, g, b, _] = c.to_rgba8();
        [r, g, b]
    };
    let (foreground, background) = (rgb(app.foreground), rgb(app.background));
    let seed = app.next_seed();
    match filter {
        Filter::Clouds { .. } => Filter::Clouds {
            foreground,
            background,
            seed,
        },
        Filter::DifferenceClouds { .. } => Filter::DifferenceClouds {
            foreground,
            background,
            seed,
        },
        other => other,
    }
}
