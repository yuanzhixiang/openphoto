//! Modal dialogs.

mod adjust;
pub mod alert;
mod appkit;
mod black_white;
mod brightness_contrast;
mod canvas_size;
mod channel_mixer;
mod color_balance;
mod color_picker;
mod common;
mod curves;
mod custom_filter;
mod distort;
mod document_presets;
mod duplicate_layer;
mod equalize;
mod exposure;
mod fade;
mod fill;
mod filter_layout;
mod gradient_map;
mod hue_saturation;
mod image_size;
mod levels;
mod lock_layers;
mod modify_selection;
mod new_document;
mod new_guide;
mod new_layer;
mod photo_filter;
mod plain_filter;
mod rotate_canvas;
pub mod save_changes;
mod selective_color;
mod threshold;
mod trim;
mod uxp;
mod vibrance;

pub use adjust::{AdjustDialog, Effect, Kind as AdjustKind, Outcome as AdjustOutcome};
pub use canvas_size::{CanvasSizeDialog, Outcome};
pub use color_picker::{ColorPicker, Outcome as ColorPickerOutcome};
pub use duplicate_layer::{Destination, DuplicateLayerDialog, Outcome as DuplicateOutcome};
pub use equalize::{EqualizeDialog, Outcome as EqualizeOutcome};
pub use fade::{FadeDialog, Outcome as FadeOutcome};
pub use fill::{FillDialog, Outcome as FillOutcome};
pub use image_size::{ImageSizeDialog, Outcome as ImageSizeOutcome, Preview as ImageSizePreview};
pub use lock_layers::{LockLayersDialog, Outcome as LockOutcome};
pub use modify_selection::{ModifyDialog, ModifyKind, Outcome as ModifyOutcome};
pub use new_document::{
    Contents as NewContents, NewDocumentDialog, Outcome as NewDocumentOutcome,
    Preset as DocumentPreset,
};
pub use new_guide::{NewGuideDialog, Outcome as NewGuideOutcome};
pub use new_layer::{Kind as NewLayerKind, NewLayer, NewLayerDialog, Outcome as NewLayerOutcome};
pub use rotate_canvas::{Outcome as RotateOutcome, RotateCanvasDialog};
pub use save_changes::SaveChoice;
pub use trim::{Outcome as TrimOutcome, TrimDialog};
