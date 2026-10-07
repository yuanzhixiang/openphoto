# tile.rs

## Responsibilities

Provides the low-level storage for layer pixels: splits an RGBA8 image into 256×256 tiles, stores them sparsely, and shares unmodified tiles between multiple images (especially history snapshots) through `Arc`. See `README.md` in the same directory for the design motivation.

## Public Interface

### `TILE_SIZE`

The tile edge length, fixed at 256 pixels. Each tile takes 256×256×4 = 262144 bytes.

### `Tile`

- A 256×256 RGBA8 tile with straight alpha; pixels are stored row-major, 4 bytes per pixel, in the public field `data: Box<[u8]>`.
- `Tile::filled(rgba)`: fills the whole tile with one color.
- Edge tiles are also full 256×256; the part beyond the image's width and height is storage padding and not part of the image content.

### `TiledImage`

`width` × `height` is the canvas. As with Photoshop layers, pixels can also lie outside the canvas: tile coordinates are signed (`HashMap<(i32, i32), Arc<Tile>>`); tiles to the left of and above the canvas use negative coordinates, and tiles to the right and below lie outside the canvas's tile range; the part of an edge tile beyond the canvas width and height also holds "outside the canvas" pixels. Therefore, unless pixels are really meant to go outside the canvas, no code may write anything into the outside-the-canvas part of an edge tile (this is the most important invariant of this module).

- `new(width, height)`: an empty image; no tiles are allocated, and the whole image is treated as fully transparent.
- `filled(width, height, rgba)`: the canvas is filled with a solid color, with no pixels outside the canvas. With alpha 0 it is equivalent to `new`; otherwise interior tiles share the same `Arc<Tile>`, and edge tiles are copied individually with their outside-the-canvas part cleared to transparent (`clear_edge_padding`).
- `from_rgba8(width, height, pixels)`: builds from a tightly packed canvas-sized RGBA8 buffer, copying tile by tile; fully transparent tiles are discarded; the outside-the-canvas part of edge tiles is transparent.
- `width()`, `height()`: the canvas size. `tiles_x()`, `tiles_y()`: the number of tiles covering the canvas, computed with `div_ceil(TILE_SIZE)`.
- `tile(tx, ty)` / `tile_mut(tx, ty)`: a tile within the canvas range (`u32` coordinates); `tile_mut` allocates a fully transparent tile when missing and copies it first when shared (copy-on-write).
- `tile_at(tx, ty)` / `tile_at_mut(tx, ty)`: any tile (`i32` coordinates, may be outside the canvas).
- `pixel(x, y)` / `set_pixel(x, y, rgba)`: a pixel within the canvas; with coordinates outside the canvas, reads return transparent and writes are ignored.
- `pixel_at(x, y)` / `set_pixel_at(x, y, rgba)`: a pixel at any position (`i64`, may be negative). Writing a transparent pixel into a missing tile does not allocate a tile.
- `content_bounds()`: the bounding box `(x0, y0, x1, y1)` of all non-transparent pixels (alpha > 0, including those outside the canvas), or `None` if there are none. Tiles at the edge of the tile grid are processed first; after that, tiles lying entirely within the bounds found so far are skipped; within each tile it searches inward from the top and bottom for the first row, then inward from the left and right between them for the first column, stopping at the first pixel. It is called several times per frame (the Move tool's transform controls, the enabled state of the align buttons), so it must not scan the whole layer pixel by pixel.
- `has_pixels_outside()`: whether any pixels lie outside the canvas.
- `clipped()`: the image with pixels outside the canvas deleted (used by the Background layer and by cropping with "Delete Cropped Pixels" on). Keeps only tiles within the canvas range and clears the outside-the-canvas part of edge tiles.
- `with_canvas(width, height, dx, dy, fill)`: produces an image with the new canvas size; all pixels (including those outside the canvas) are offset by (`dx`, `dy`), and pixels that land outside the new canvas after the offset are kept. When `fill` is opaque, the area of the new canvas not covered by the (offset) original canvas is filled with `fill` (the Background layer's extension color); pixels inside the original canvas that were transparent stay transparent. This is the underlying implementation of Canvas Size, the Move tool and Reveal All.
- `to_rgba8()`: converts the canvas part into a tightly packed RGBA8 buffer.
- `region_rgba8(x0, y0, w, h)`: the pixels of any region (may extend outside the canvas), copied tile by tile in row segments.
- `from_region(width, height, x0, y0, w, h, pixels)`: an image with a `width`×`height` canvas whose content is only the given region's pixels, written row by row with `write_span_at`; fully transparent tiles are discarded. Free Transform uses these two methods to handle pixels outside the canvas as well.
- `remapped(width, height, source)`: an image of the new size where pixel (x, y) takes the original image's in-canvas pixel `source(x, y)`. Used for rotation and flipping; pixels outside the canvas do not take part (see Known Limitations).
- `allocated_tiles()`: the number of allocated tiles, for debugging and memory statistics.

## Behavior Rules and Invariants

- A missing tile is equivalent to fully transparent `[0, 0, 0, 0]` and takes no memory.
- Cloning a `TiledImage` copies only the tile pointer table, not the pixels; history snapshots therefore also fully keep pixels outside the canvas.
- `with_canvas` writes pixels row by row to their target positions, one allocated source tile at a time (`write_span_at` splits a row across target tiles; when a whole segment is transparent and the target tile is missing, nothing is allocated), then fills `fill` in row segments, and finally discards fully transparent tiles. Memory use is proportional to the number of allocated tiles.
- "Fully transparent" is determined by alpha only: a tile whose pixels all have alpha 0 is considered discardable, even if its RGB components are nonzero.
- Compositing, thumbnails, etc. iterate only over tiles within the canvas range and read only in-canvas pixels; pixels outside the canvas are not displayed.

## Edge Cases

- Width or height 0: `tiles_x()`/`tiles_y()` are 0, and reading in-canvas pixels always returns transparent.
- `from_rgba8` uses `assert_eq!` to check that the buffer length equals `width * height * 4`, and panics on mismatch.
- After `tile_mut` allocates or modifies a tile, it does not check again whether the tile became fully transparent; a tile erased to full transparency stays allocated until the next `with_canvas` or `clipped` rebuilds the image.

## Relationship to Other Modules

- `layer.rs`: `LayerKind::Raster` holds a `TiledImage`.
- `document.rs`: new/opened documents build images with `filled`/`from_rgba8`; `place_canvas` (Canvas Size, Reveal All) calls `with_canvas`, and the Background layer then calls `clipped`; `content_bounds` aggregates each layer's `content_bounds`; `composite_rgba8` iterates `tile()` directly to read pixels.
- `move_tool.rs`, `clipboard.rs` (paste), and PSD reading/writing in `op-io` use `pixel_at`/`set_pixel_at`/`content_bounds` to handle pixels outside the canvas.
- `history.rs`: clones layers through `Snapshot`, relying on this module's `Arc` sharing to control memory.
- `op-ui` reads a layer's `TiledImage` to generate layer thumbnails.

## Known Limitations

- Only 8-bit RGBA storage is supported; there are no 16-bit or floating-point tiles.
- No mipmaps or multi-resolution levels.
- Tiles that become fully transparent after writes are not reclaimed automatically.
- Operations that rebuild a layer from a canvas-sized buffer drop pixels outside the canvas: `remapped` (image rotation, canvas flip), Image Size resampling, merging layers (Merge Down/Visible/Layers). In Photoshop these operations process the pixels outside the canvas along with the rest.

## Test Coverage

- `round_trip_and_sparse`: in a 300×10 image, only the second tile column has one opaque pixel; after building, only 1 tile is allocated, that pixel reads back correctly, and other positions read back transparent.
- `with_canvas_grows_and_crops`: a 2×2 image enlarged to 4×4 and offset by (1,1) has the fill color around it and the original pixels in the middle; cropping to 1×1 at (-1,-1) gives the original image's bottom-right pixel.
- `with_canvas_across_tiles`: after offsetting an image whose width spans two tiles by (300, 1), every pixel lands in the correct position, uncovered areas are transparent, and the completely uncovered first column of tiles stays unallocated.
- `copy_on_write`: after cloning, writing to the copy through `tile_mut` does not affect the original.
- `filled_leaves_nothing_past_the_canvas`: a 10×10 solid-color image reads transparent at (10, 0); the content bounds are exactly the canvas, with no pixels outside the canvas.
- `moving_keeps_pixels_outside_the_canvas`: after offsetting by (−300, 5), pixels land far to the left of the canvas and are kept, with correct content bounds; offsetting back restores both pixels.
- `clipped_drops_what_is_outside`: pixels outside the canvas on the left, on the right (inside an edge tile) and in tiles below are all deleted, leaving only one tile inside the canvas.
- `regions_reach_outside_the_canvas`: reads a region extending outside the canvas to the top left and far to the right, writes it back with `from_region`, and the content bounds and pixels are unchanged.
- `opaque_fill_only_covers_new_canvas_area`: when extending the canvas with an opaque color, only the new area is filled; transparent pixels inside the original canvas stay transparent, and the result has no pixels outside the canvas.
- `content_bounds_match_a_full_scan`: random sparse points (including outside the canvas) match point-by-point computation; the bounds of a large 3000 × 1080 block of content are exact, and 10 calls take less than 20 ms.
