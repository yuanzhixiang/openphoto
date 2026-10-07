# op-core: Document Model

## Role

`op-core` is OpenPhoto's document data layer: the layer list, tiled pixel storage, pixel format enums, color values and undo history are all defined here. Every edit ultimately becomes a modification of these data structures; the UI (`op-ui`) and rendering (`op-render`) only read or drive these structures.

This crate has no external dependencies (`[dependencies]` in `Cargo.toml` is empty) and does not depend on the GPU, the window system or a UI framework. This is a load-bearing constraint: it guarantees the document model can be unit tested without the interface, and lets `op-io`, `op-color` and `op-tools` work depending only on `op-core`.

## Module Breakdown

| File | Contents |
| --- | --- |
| `lib.rs` | Module declarations and re-exports of common types |
| `color.rs` | `Color`: straight (non-premultiplied) RGBA floating-point color |
| `pixel.rs` | `ColorMode` (document color mode), `BitDepth` (bit depth) |
| `tile.rs` | `Tile`, `TiledImage`: sparse, copy-on-write tiled pixel storage, `TILE_SIZE = 256` |
| `layer.rs` | `Layer`, `LayerId`, `LayerKind`, `BlendMode` |
| `document.rs` | `Document`, `DocId`, `Snapshot`, `Anchor`: the document itself, canvas size changes, CPU compositing |
| `history.rs` | `History`: linear undo history modeled on Photoshop's History panel |

`lib.rs` re-exports `Color`, `Anchor`, `DocId`, `Document`, `Snapshot`, `History`, `BlendMode`, `Layer`, `LayerId`, `LayerKind`, `BitDepth`, `ColorMode`, `TILE_SIZE`, `Tile` and `TiledImage` at the crate root. `HistoryState` and `history::DEFAULT_LIMIT` are not re-exported at the root and must be accessed through the `op_core::history::` path.

## Document Model

- A `Document` has a fixed pixel width and height, resolution (ppi), color mode and bit depth, plus a layer list `layers` ordered "bottom to top" and the current active layer `active_layer`.
- Currently the only layer type is the raster layer (`LayerKind::Raster(TiledImage)`). Each layer's `TiledImage` has the same width and height as the document; `Document::resize_canvas` rewrites all layers together, keeping this invariant.
- The "Background" layer (`is_background = true`) follows Photoshop's semantics: locked, opaque, always at the bottom. A new document gets a single background layer named `"Background"`. When opening a bitmap file, a fully opaque image likewise gets a background layer; an image with transparency gets a regular layer named `"Layer 0"`, matching Photoshop.
- `DocId` and `LayerId` share the same process-wide atomic counter (incrementing from 1), so across the whole process the two kinds of IDs never collide with each other and never repeat between documents.
- `Document` maintains a `revision` counter. Any change to pixels or layer properties must call `mark_dirty()` to increment it; the rendering layer uses it to decide whether to re-upload textures. Inside `op-core`, only `resize_canvas` and `restore` increment it automatically; callers that modify public fields directly (e.g. layer properties) must call `mark_dirty()` themselves.

## Why Tiled Storage Is Sparse and Copy-on-Write

- Pixels are stored in 256×256 tiles in a `HashMap<(tx, ty), Arc<Tile>>`. A missing tile is treated as fully transparent and takes no memory; when building from an RGBA buffer or resizing the canvas, fully transparent tiles are discarded. This lets sparse layers on a large canvas (e.g. a new layer with only a few strokes) take almost no memory.
- A solid-color fill (`TiledImage::filled`) makes all tiles share the same `Arc<Tile>`, so a solid background takes the memory of only one tile.
- Tiles are shared through `Arc`: cloning a `TiledImage` copies only pointers; on write, `tile_mut` uses `Arc::make_mut` to copy a tile first when it is shared. This way snapshots and the current document can share unchanged tiles.

## How History Snapshots Share Tiles

- `Snapshot` is the undoable part of the document: width, height, resolution, layer list, active layer. The cost of cloning it is cloning layer metadata and tile pointer tables, not pixels.
- Each state in `History` stores a complete snapshot. Because tiles are shared between snapshots, only edited tiles take extra memory.
- Snapshots are complete (not incremental), so when the history exceeds its limit, any intermediate state can be safely deleted.

## Why Compositing Is in Gamma Space

`Document::composite_rgba8` blends directly on sRGB-encoded values without converting to linear light first. This matches Photoshop's default behavior (Photoshop by default does not enable the linear blending of "Blend RGB Colors Using Gamma"), so the same layer stack produces the same values as Photoshop; for example, 50% opacity black over white gives 128 gray. `Color` components are likewise defined as sRGB-encoded values.

## Cross-File Rules

- Pixel format: all pixels are stored as 8-bit RGBA with straight (non-premultiplied) alpha. Other values of `BitDepth` and `ColorMode` are only selectable enum values and do not change the storage format.
- Color quantization: `Color::to_rgba8` and the composite output both use the rounding rule "clamp to 0..=1, multiply by 255, add 0.5, then truncate".
- Out-of-bounds reads: `TiledImage::pixel` returns transparent `[0, 0, 0, 0]` for out-of-bounds coordinates and does not panic.
- Photoshop alignment: the order and grouping of blending modes, Background layer semantics, the history limit of 50 entries, the round-trip semantics of Toggle Last State, and keeping the first state when trimming history all take Photoshop as the reference.

## Known Limitations

- Only raster layers; no layer groups, adjustment layers, type layers, smart objects, masks or layer styles.
- Only 8-bit RGB storage is supported; 16-bit, 32-bit and non-RGB color modes are not implemented.
- Compositing is done per pixel on the CPU (blending mode algorithms in `blend.md`).
- No selections and no color management (ICC).
