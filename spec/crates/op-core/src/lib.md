# lib.rs

## Responsibilities

The crate root of `op-core`. Declares six public modules, `color`, `document`, `history`, `layer`, `pixel`, and `tile`, and re-exports commonly used types at the crate root so downstream crates can write `op_core::Document`, `op_core::TiledImage`, and so on directly.

The crate-level doc comment states the crate's role: the document model (layer tree, tiled pixel storage, pixel formats), independent of GPU or UI; every edit ultimately modifies the data structures here. For the overall model, see `README.md` in the same directory.

## Public interface

Types re-exported at the root:

- `color::Color`
- `document::{Anchor, DocId, Document, Snapshot}`
- `history::History`
- `layer::{BlendMode, Layer, LayerId, LayerKind}`
- `pixel::{BitDepth, ColorMode}`
- `tile::{TILE_SIZE, Tile, TiledImage}`

Public items not re-exported at the root, which must be reached through the module path: `history::HistoryState`, `history::DEFAULT_LIMIT`.

## Relationship to other modules

- Depended on by `op-color`, `op-io`, `op-tools`, and `op-ui`. `op-render` does not depend on `op-core`; it only receives RGBA pixels passed from `op-ui`.
- This crate does not depend on any external crate.

## Test coverage

This file has no tests; for each module's tests, see the corresponding spec.
