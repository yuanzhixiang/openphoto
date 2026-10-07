//! Modal dialogs.

mod adjust;
mod adjust_presets;
pub mod alert;
mod appkit;
pub(crate) mod auto_options;
pub(crate) mod auto_resolution;
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
pub(crate) mod document_presets;
mod duplicate_layer;
mod equalize;
mod exposure;
mod fade;
mod fill;
mod filter_layout;
pub(crate) mod gradient_editor;
mod gradient_map;
pub(crate) mod grid_size;
pub(crate) mod guide_layout;
mod hue_saturation;
pub(crate) mod image_size;
mod levels;
mod lock_layers;
mod modify_selection;
mod new_document;
mod new_guide;
mod new_layer;
pub(crate) mod new_preset;
mod photo_filter;
mod plain_filter;
mod rotate_canvas;
pub mod save_changes;
pub(crate) mod save_options;
mod selective_color;
pub(crate) mod size_presets;
mod threshold;
mod trim;
mod uxp;
mod vibrance;

#[cfg(test)]
pub use adjust::lens_image_rect as adjust_lens_image;
pub use adjust::{
    AdjustDialog, Effect, Kind as AdjustKind, Outcome as AdjustOutcome, Remembered,
    lens_blur_source, thumbnail as adjust_thumbnail,
};
pub use canvas_size::{CanvasSizeDialog, Outcome};
pub use color_picker::{ColorPicker, Outcome as ColorPickerOutcome};
pub use duplicate_layer::{Destination, DuplicateLayerDialog, Outcome as DuplicateOutcome};
pub use equalize::{EqualizeDialog, Outcome as EqualizeOutcome};
pub use fade::{FadeDialog, Outcome as FadeOutcome};
#[cfg(test)]
pub use fill::Contents as FillContents;
pub use fill::{FillDialog, FillWith, Outcome as FillOutcome};
pub use image_size::{ImageSizeDialog, Outcome as ImageSizeOutcome, Preview as ImageSizePreview};
pub use lock_layers::{LockLayersDialog, Outcome as LockOutcome};
pub use modify_selection::{ModifyDialog, ModifyKind, Outcome as ModifyOutcome};
pub use new_document::{
    Contents as NewContents, NewDocumentDialog, Outcome as NewDocumentOutcome,
    Preset as DocumentPreset,
};
pub use new_guide::{NewGuideDialog, Outcome as NewGuideOutcome, Remembered as NewGuideRemembered};
pub use new_layer::{Kind as NewLayerKind, NewLayer, NewLayerDialog, Outcome as NewLayerOutcome};
pub use rotate_canvas::{Outcome as RotateOutcome, RotateCanvasDialog};
pub use save_changes::SaveChoice;
pub use trim::{Outcome as TrimOutcome, TrimDialog};
