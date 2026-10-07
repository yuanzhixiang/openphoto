# Canvas rendering (canvas.rs + canvas.wgsl)

> This document covers both `crates/op-render/src/canvas.rs` (Rust side: resource management, texture upload, mip generation, uniform writes) and `crates/op-render/src/canvas.wgsl` (shader: checkerboard, image sampling, pixel grid). The two source files have the same name without their extensions, so they share this one spec.

## Responsibilities

Within a rectangular region of the egui interface, draws the document composite at the given zoom and pan:

- A checkerboard shows beneath transparent areas;
- At zoom ≥ 100% it displays with nearest-neighbor sampling and sharp pixel edges; at zoom < 100% it uses the mip chain for smooth downscaling;
- At zoom ≥ 600% a pixel grid is overlaid;
- Nothing is drawn outside the document bounds, exposing the pasteboard background that egui draws underneath.

## Behavior rules

### Embedding

- `install(render_state)` is called once at startup: it compiles `canvas.wgsl`, creates the bind group layout (binding 0 is the uniform buffer, 1 is a filterable 2D float texture, 2 is a filtering sampler, all visible only in the fragment stage), the render pipeline and the sampler, and inserts `CanvasResources` into the egui-wgpu renderer's `callback_resources`.
- The pipeline's color target uses egui's `render_state.target_format` directly, with no blend state (fragment results overwrite the target directly), no depth/stencil and no multisampling.
- `paint_callback(rect, image, view)` returns an `egui::PaintCallback` produced by `egui_wgpu::Callback::new_paint_callback`. The caller adds it to a painter (`op-ui` also adds a clip rectangle). The canvas therefore shares the `Device`, `Queue` and render pass with egui and is inserted in egui's draw order.
- In each frame's `prepare` stage: creates or rebuilds the texture slot as needed and writes this frame's uniforms; submits no extra command buffers.
- In the `paint` stage: binds the pipeline and the document slot's bind group and draws, with 3 vertices, one large triangle covering the whole viewport (i.e. the callback rectangle), with no vertex buffer.

### View transform

- The unit of `CanvasView::zoom` is **physical pixels / document pixel**. As in Photoshop, 100% means one image pixel maps to one physical screen pixel; on a 2× Retina screen a document at 100% occupies only half the size in logical points.
- `CanvasView::origin` is the screen position of the document's top-left corner, in egui logical points. In `prepare` it is multiplied by `ScreenDescriptor::pixels_per_point` to convert to physical pixels before being written to the uniform.
- The shader uses the framebuffer coordinates of `@builtin(position)` (physical pixels), independent of the viewport egui sets for the callback. A fragment's document coordinate is `(frag.xy - origin_px) / zoom`.
- Aligning the origin to physical pixels is the caller's responsibility; `op-ui` snaps the origin to the physical pixel grid before passing it in, so the image is sharp at 100%.

### Uniform layout

Two `vec4<f32>` (Rust side `Uniforms`, `#[repr(C)]` + `Pod`):

- `origin_zoom`: xy is the physical pixel coordinate of the document's top-left corner, z is zoom, w is the checkerboard cell size (physical pixels).
- `doc`: xy is the document width and height (document pixels), z is the maximum mip level (`mip_count - 1`), w is the pixel grid switch (0 or 1).

### Texture slots

- `CanvasResources::slots` is a `HashMap<u64, Slot>` keyed by `CanvasImage::key`, one slot per document (the key `op-ui` passes is the document id).
- Each slot holds: the currently uploaded `revision`, the mip level count, and the slot's own uniform buffer and bind group (texture view + shared sampler + uniform).
- In `prepare`, if the slot does not exist, or the slot's `revision` differs from `CanvasImage::revision`, the mip chain is generated first: when both the size and the level count match the slot's texture, each level is written directly into the existing texture (this happens every frame while dragging a layer; rebuilding the texture is too slow); otherwise `create_slot` rebuilds everything: a new texture, uploading all mip levels, a new uniform buffer and bind group, replacing the old slot (whose GPU resources are released with it). When the revision is the same, nothing is uploaded; only the uniform is written.
- Whether to re-upload depends only on whether `revision` is equal; neither pixel content nor size is compared. The meaning of revision is guaranteed by the caller.

### Texture format

- The texture format is `Rgba8Unorm`, deliberately **not** an sRGB format: Photoshop blends and displays in gamma-encoded space, so sampling, mip averaging and compositing with the checkerboard all operate directly on gamma-encoded values, without linearization.
- Fragment output likewise performs no color space conversion and is written as-is to egui's target format.

### Premultiplication and the mip chain (CPU)

- The input `CanvasImage::pixels` is tightly packed **non-premultiplied** RGBA8. `build_mips` first premultiplies on the CPU: each color channel `c' = (c * a + 127) / 255` (rounded), alpha unchanged.
- Premultiplication and each downscale level are split into row bands across CPU threads (single-threaded when there are fewer than 64 rows), with results identical to per-pixel computation; a 3000 × 1080 image takes about 1.7 ms (about 12 ms single-threaded).
- Then, starting from the original size, it downscales level by level with a 2×2 box filter down to 1×1: the next level's size is `max(w/2, 1) × max(h/2, 1)`, and each output pixel is the four source pixels `(2x, 2y)`, `(2x+1, 2y)`, `(2x, 2y+1)`, `(2x+1, 2y+1)` (coordinates out of bounds are clamped to the last row/column) summed per channel and then `(sum + 2) / 4`. Averaging is done on premultiplied data, so transparent pixels do not "bleed" color in.
- After each level is generated, only levels whose width and height are both no greater than `device.limits().max_texture_dimension_2d` are kept. Large levels exceeding the limit are skipped, and level 0 of the texture is the first level that fits on the GPU.

### Sampling

- The shader first normalizes the document coordinate to `uv = d / doc_size`, then multiplies by the actual size of texture level 0 to fetch texels. This proportional mapping still covers the whole document area after large levels are skipped (texture smaller than the document).
- `zoom >= 1.0`: reads the `floor(uv * dims)` texel of level 0 with `textureLoad`, i.e. nearest-neighbor sampling, bypassing the sampler.
- `zoom < 1.0`: samples with an explicit LOD using `textureSampleLevel`, `lod = clamp(log2(dims.x / doc.x / zoom), 0, max_level)`; the sampler's magnification, minification and mip filters are all linear (trilinear), and the address mode is the default clamp-to-edge. LOD is computed from the x direction only.

### Checkerboard

- The cell size is 8 logical points, converted to physical pixels and rounded (`round(8 * pixels_per_point)`); it does not change with zoom.
- Cells are anchored at the document's top-left corner (`floor((frag - origin) / cell)`) and move with panning; the two alternating colors are white `1.0` and gray `0.8`.
- Because the texture is premultiplied, the compositing formula is `rgb = c.rgb + checker * (1 - c.a)`. Output alpha is always 1.

### Pixel grid

- Drawn when `pixel_grid` is true and `zoom >= 6.0` (600%).
- Takes the fractional part of the fragment's document coordinate and computes the distance to the nearest pixel edge (multiplied by zoom to convert to physical pixels); when the distance in either direction is < 0.5 physical pixels, the color is mixed 35% toward gray `0.55`. The effect is a semi-transparent gray line about 1 physical pixel wide on every pixel boundary.

### Outside the document

- When a fragment's document coordinate `d` satisfies `d.x < 0`, `d.y < 0`, `d.x >= width` or `d.y >= height`, it is `discard`ed and no color is written. Areas inside the callback rectangle but outside the document keep what egui drew earlier (the pasteboard).

## Edge cases

- Running the paint callback without calling `install`: `expect("canvas renderer installed")` in `prepare` / `paint` panics.
- An image with zero width or height: `build_mips` returns an empty list and `create_slot` falls back to a 1×1 fully transparent texture; in the shader the document size is 0, every fragment is discarded and nothing is drawn in the canvas area.
- An image exceeding the GPU's maximum texture size: the first downscaled level that fits is displayed; at `zoom >= 1` it is scaled up proportionally and displayed with nearest-neighbor sampling, so it looks coarser than the original and the actual resolution is lower than the document's.
- No matching slot found during `paint` (does not happen in the normal flow, since `prepare` always creates the slot first): returns immediately without drawing.
- Downscaling odd sizes: `w/2` rounds down, so for levels with an odd width (height) the last column (row) does not take part in the next level's average; clamping only matters when the source size is 1.
- When the same key is submitted multiple times in one frame (e.g. two views of the same document), they share one uniform buffer; the uniform written later in `prepare` overwrites the earlier one, and all draws use the last view parameters.
- The pixel grid also shows a half line just inside the document's outer edges (the distance to a pixel boundary at the edge is likewise < 0.5 physical pixels).
- The length of `pixels` must be exactly `width * height * 4`. A trailing remainder of fewer than 4 extra bytes is ignored by the premultiply step; a shorter length makes the data size of the later texture write mismatch.

## Relationship with other modules

- `crates/op-render/src/lib.rs` re-exports `CanvasImage`, `CanvasView`, `install` and `paint_callback`.
- `op-ui/src/lib.rs`: calls `install` in `OpenPhotoApp::new`, which requires eframe to use the wgpu renderer.
- `op-ui/src/state.rs`: `DocState::canvas_image` uses `doc.id.0` as the key and `doc.revision()` as the revision, calls `Document::composite_rgba8()` on the CPU to produce the pixels, and caches the `Arc<CanvasImage>` by revision.
- `op-ui/src/document_view.rs`: computes the origin snapped to physical pixels, builds the `CanvasView` (`pixel_grid` is always `true`, zoom is limited to 0.01–128), and adds it through `paint_callback` to a painter with a clip rectangle.

## Known limitations

- Compositing is done upstream on the CPU; this module only displays the final composite and does no layer compositing, blend modes or filters on the GPU.
- Any content change (revision change) triggers re-premultiplying the whole image, regenerating the complete mip chain and re-uploading all levels (reusing the texture when the size is unchanged); there are no partial dirty-region updates.
- Texture slots are not released after a document is closed: `slots` only has insertion and replacement by key, no removal path, so textures of closed documents keep occupying video memory until the program exits.
- Levels exceeding the GPU texture size limit are still fully generated on the CPU before being discarded, so large images incur the corresponding CPU time and memory cost; the display resolution is also reduced, and there is no tiled display.
- Mip selection is computed from the x-direction scale only, with no anisotropic filtering.
- The checkerboard colors, cell size and pixel grid color/threshold are constants hard-coded in the code and not configurable.

## Crop shield (`Shield`)

When `CanvasView::shield` is `Some`, the fragment shader transforms the document coordinate into the shield box's own coordinate system (center, half width/height, clockwise angle, reserved for rotated crop boxes); outside the box, the color and the shield color are mixed by opacity in **linear light** and then converted back to sRGB: both are first converted from sRGB to linear values, `mix`ed, then converted back. This matches Photoshop 2026's crop shield (75% `#282828` over white gives 141). The uniform gains three vec4s: `shield_box` (center xy, half width/height zw), `shield_color` (rgb and angle), `shield_params` (opacity, enabled).

## Image rotation (`rotation`)

When `CanvasView::rotation` is `Some((pivot, angle))`, a screen point is first converted by the view into a "box space" point b, then rotated clockwise by `angle` about `pivot` to get the image point d to sample (`d = pivot + R(angle)(b − pivot)`), so the image appears rotated by −angle. The shield box is evaluated in box space. Used by the Crop tool when rotating the image.

## Test coverage

- `mips_built_in_bands_match_the_reference`: the mip chain generated with multiple threads is byte-for-byte identical to a reference implementation computed per pixel as described here (including odd sizes and 1-pixel width/height).
- `levels_over_the_texture_limit_are_skipped`: levels exceeding the texture size limit are skipped.
- Performance regressions are covered by `op-ui`'s `dragging_on_a_large_document_keeps_up`.
