# op-render: crate entry point

## Responsibilities

`op-render` is OpenPhoto's GPU rendering crate. Currently it does just one thing: draws the document's composite onto the canvas, including zoom, pan, the transparency checkerboard and the pixel grid at high zoom.

`lib.rs` itself contains no logic; it only declares the private module `canvas` and re-exports four symbols:

- `CanvasImage`: the composite to display (tightly packed, non-premultiplied RGBA8), with its texture slot key and content version number.
- `CanvasView`: the view transform (screen position of the document's top-left corner, zoom factor, whether the pixel grid is allowed, the crop shield, rotation, and horizontal mirroring).
- `install`: creates the render pipeline and registers it in egui-wgpu's callback resources; called once at startup.
- `paint_callback`: produces an egui paint command that draws the canvas within a given rectangle.

For detailed behavior see `spec/crates/op-render/src/canvas.md`.

## Behavior rules

- The canvas is not a separate window or surface; it is embedded in the UI through egui-wgpu's paint callback: it shares the same wgpu `Device`, `Queue` and render pass with egui, and inserts one draw at the corresponding position in egui's paint sequence.
- The crate depends directly on `egui`, `egui-wgpu` and `bytemuck`; wgpu types are used through the re-export in `egui_wgpu::wgpu`, without a separate dependency on the `wgpu` crate, to guarantee exactly the same wgpu version as egui.
- Only the four symbols above are exposed; the pipeline, texture slots, uniform layout, mip generation and so on are module-internal details.

## Edge cases

- `install` must be called before submitting commands produced by `paint_callback`; otherwise drawing panics because the callback resources cannot be found (see the canvas spec).

## Relation to other modules

- `op-ui` (`crates/op-ui/src/lib.rs`) gets eframe's `wgpu_render_state` in `OpenPhotoApp::new` and calls `op_render::install`.
- `state.rs` in `op-ui` builds a `CanvasImage` from the document composite, and `document_view.rs` builds a `CanvasView` and adds the canvas to the painter queue via `op_render::paint_callback`.
- `op-render` does not depend on `op-core`: it knows nothing about document structure such as layers and blend modes, and only receives already composited pixels.

## Known limitations

- Layer compositing, blend modes and filters are currently not in this crate; they are done upstream on the CPU (`Document::composite_rgba8`), and this crate only displays the final composite.
- `Cargo.toml` declares a `log` dependency, but the current source does not use it.
