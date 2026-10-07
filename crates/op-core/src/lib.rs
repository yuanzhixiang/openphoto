//! Document model: layer tree, tiled pixel storage and pixel formats.
//!
//! This crate has no GPU or UI dependencies; every edit ends up as a change to
//! the data structures defined here.

pub mod adjust;
pub mod align;
pub mod auto;
pub mod blend;
pub mod clipboard;
pub mod color;
pub mod color_match;
pub mod document;
pub mod fade;
pub mod fill;
pub mod filter;
pub mod gradient;
pub mod guide_layout;
pub mod heal;
pub mod history;
pub mod image_ops;
pub mod layer;
pub mod layer_ops;
pub mod link;
pub mod magnetic;
pub mod more_filters;
pub mod move_tool;
pub mod paint;
pub mod pixel;
pub mod selection;
pub mod selection_brush;
pub mod shape;
pub mod smart_select;
pub mod text;
pub mod tile;
pub mod tone;
pub mod transform;

pub use color::Color;
pub use document::{Anchor, DocId, Document, Guide, SampleScope, Snapshot};
pub use history::History;
pub use layer::{
    BlendMode, Layer, LayerColor, LayerId, LayerKind, LayerMask, Locks, neutral_color,
};
pub use pixel::{BitDepth, ColorMode};
pub use selection::{Selection, SelectionOp};
pub use tile::{TILE_SIZE, Tile, TiledImage};
