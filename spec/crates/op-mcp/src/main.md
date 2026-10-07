# crates/op-mcp/src/main.rs: OpenPhoto's MCP server

## Responsibilities

Lets AI agents edit images through MCP (Model Context Protocol). `op-mcp` is a standalone executable (binary name `op-mcp`) that speaks JSON-RPC 2.0 over standard input and output, one message per line. It has no interface and does not connect to a running OpenPhoto window: documents exist only in this process's memory and disappear when the process exits (`save_image` / `export_composite` must be called first).

All edits call the same `op-core` / `op-io` functions the application itself uses (adjustments, filters, layers, selection, canvas, fill, history) rather than reimplementing them, so the results match operating in OpenPhoto.

Startup: the macOS release ships the binary inside the app as `OpenPhoto.app/Contents/MacOS/op-mcp` (see `spec/scripts/package-macos.md`), so MCP clients run it from the installed app with no Rust toolchain; from a source checkout, `cargo run -p op-mcp`. For client configuration examples, see the "MCP server" section of the repository's `README.md`.

## Public interface: only two MCP tools

Rather than registering more than a hundred operations one by one as MCP tools (which would fill the client's context), only two are exposed:

- `search_tools`: parameters `query` (required, keywords) and `limit` (1–50, default 10). Scores the internal tool catalog by keyword and returns the matching tool names, descriptions, and full parameter JSON Schema (`inputSchema`), along with `count` and `total_tools`. When `query` is empty, `*`, `all`, `list`, or `list all`, it lists the first `limit` tools in catalog order.
- `call_tool`: parameters `name` (required, internal tool name) and `arguments` (object, optional). Executes that internal tool.

Usage is search first, then call: `search_tools {"query": "gaussian blur"}` returns `filter_gaussian_blur` and its parameter descriptions, then `call_tool {"name": "filter_gaussian_blur", "arguments": {"radius": 2}}`.

### Search scoring

The query is compared with each tool's "name + keywords + description", sorted by score from high to low, ties sorted by name, and only tools with a score greater than 0 are returned:

- After removing non-alphanumeric characters, a tool name identical to the query scores +200, a tool name containing the query +100; a description containing the whole query +20.
- The query is split into words at non-alphanumeric characters, and words of length < 3 are ignored. Each word takes its best match in the tool's word list: identical +15, tool word contains the query word +7, query word contains a tool word of length ≥ 5 +5 (e.g. "blurring" matches "blur"; the length limit keeps short words like per and ion from matching wrongly); an additional +10 when the query word is exactly one of the tool's keywords.

## Protocol handling

- `initialize`: returns the `protocolVersion` requested by the client (`2024-11-05` when absent), `capabilities.tools` (`listChanged: false`), `serverInfo` (`openphoto-mcp` and the package version), and an `instructions` text explaining usage.
- `ping`: returns an empty object. `tools/list`: returns the definitions of the two tools above.
- `tools/call`: names other than `search_tools` / `call_tool` return JSON-RPC error -32601. When an internal tool fails, it is not a JSON-RPC error but a normal result with `isError: true`, and `content` is `{"error": "reason"}`; on success `content` is formatted JSON text. `call_tool` missing `name` is likewise returned with `isError`.
- Methods starting with `notifications/` and messages without an `id` get no reply. Other unknown methods with an `id` return -32601.
- A line that cannot be parsed returns a -32700 error with `id: null`; empty lines are skipped. The process exits on an error reading stdin or writing stdout.

## Documents and state

- Multiple documents can be open in the process at once, each with its own `History`. After being created or opened, a document becomes the "current document"; every internal tool accepts an optional `doc` (document id, i.e. the string of the `Document` id number), which defaults to the current document when omitted, and reports an error when there is no document.
- After `close_document`, if the closed one was the current document, the current document changes to any one of the remaining documents (`HashMap` order).
- Layers are specified with `layer` (layer id number), defaulting to the active layer when omitted.

## Internal tool catalog (107 tools)

- **Files and documents**: `new_document` (width and height default to 800 × 600, background color `fill` defaults to white), `open_image`, `save_image` (`.psd` keeps layers, other extensions save the composite), `export_composite`, `list_documents`, `get_document_info`, `close_document`, `set_active_document`, `get_pixel` (one pixel of the composite or of a single layer), `get_histogram` (RGB histogram of the active layer within the selection).
- **Adjustments** (`adjust_*`, recording the same history names as the application): Invert, Desaturate, Threshold, Posterize, Equalize (including Entire Image), Levels (composite and each of the R/G/B channels), Hue/Saturation (master and six color ranges, Colorize), Exposure, Brightness/Contrast (including Legacy), Color Balance, Black & White, Vibrance, Photo Filter, Gradient Map, Auto Tone / Contrast / Color, Curves (at most 16 points per channel), Channel Mixer, Selective Color. Each parameter's value range matches the corresponding dialog, and values outside it report an error.
- **Filters** (`filter_*`): Gaussian Blur, Box Blur, Average, Unsharp Mask, Add Noise, Median, Minimum, Maximum, Blur, Blur More, Sharpen, Sharpen More, Sharpen Edges, Despeckle, Find Edges, Motion Blur, Emboss, Fragment, Custom (5 × 5 convolution kernel), Surface Blur, Dust & Scratches, Trace Contour, Wind, Twirl, Pinch, Spherize, Polar Coordinates, High Pass, Offset, Mosaic, Solarize. The optional `background` ([r, g, b]) is the background color used by filters that need one, default white.
- **Layers**: `list_layers`, `set_active_layer`, `new_raster_layer` (transparent or solid color), `new_group`, `duplicate_layer`, `delete_selected_layers`, `rename_layer`, `set_layer_visibility`, `set_layer_opacity`, `set_layer_fill`, `set_layer_blend_mode`, `arrange_layer`, `merge_down`, `merge_visible`, `flatten_image`, `group_selected_layers`, `ungroup_layers`, `add_layer_mask`, `convert_background_to_layer`, `align_layers`, `distribute_layers`, `layer_via_copy`, `layer_via_cut`.
- **Selection**: `select_all`, `clear_selection`, `reselect_selection`, `select_rect`, `select_ellipse` (anti-aliasing can be turned off), `feather_selection` (radius 0.1–1000).
- **Canvas**: `rotate_flip_canvas`, `rotate_canvas_arbitrary`, `resize_canvas` (anchor `anchor_x` / `anchor_y` 0–2, centered by default), `crop_image`, `crop_to_selection`, `resize_image` (resampling method defaults to Automatic), `trim_image`, `reveal_all`.
- **Edit**: `fill_solid` (color required, blend mode, opacity, preserve transparency), `clear_layer`, `paint_bucket` (click position and color required, tolerance, anti-alias, contiguous, all layers), `gradient_fill` (two points, two colors, `kind` linear/radial/angle/reflected/diamond, opacity, reverse), `move_active_layer` (Move tool: moves the layer or the selected pixels).
- **History**: `history_undo`, `history_redo`, `list_history`.

Each tool returns JSON: edit tools return the document id, width and height, the active layer, and `revision`, plus fields specific to the operation; query tools return what was queried.

## Validation and limits

- When a parameter has the wrong type (e.g. a string passed for a number) or a required parameter is missing, an error is reported and nothing is executed. The `required` of each tool schema matches the parameters the handler actually requires (for example `color` of `fill_solid` and `paint_bucket`).
- Width and height limits match the application: the width and height of `new_document`, `resize_image`, and `resize_canvas` must be 1–30000 pixels, to keep a single call from requesting tens of GB of memory and crashing the process.
- Filter parameters are checked against the ranges of the application's filter dialogs (for example Gaussian Blur, High Pass, Unsharp Mask radius 0.1–1000, Box Blur 1–2000, Median 1–500, Minimum / Maximum 0.2–500, Motion Blur angle −360–360 and distance 1–2000, Unsharp Mask amount 1–500, Add Noise amount 0.1–400, Offset displacement ±30000, Mosaic cell 2–200). The server handles requests sequentially on one thread, and an overly large radius would leave it unresponsive for a long time, so values out of range report an error directly.
- `gradient_fill` has no interpolation method (op-core's gradient tool has no such option, only the gradient map does); passing `method` reports an error suggesting `adjust_gradient_map` instead.

## History

- Creating or opening a document starts the history with "New" or "Open".
- Each successful operation that changes the document records one step, with the same name as in the application (e.g. "Gaussian Blur", "Merge Down", "Canvas Size"). Failed operations are not recorded; `arrange_layer`, `trim_image`, and `reveal_all` are also not recorded when they change nothing.
- Selection changes are also history steps, matching the application and Photoshop: "Select All", "Deselect" (deselecting with no selection is not recorded), "Reselect", "Rectangular Marquee", "Elliptical Marquee", "Feather". So undoing after a marquee selection only undoes the selection and does not also undo the earlier edit.
- A history snapshot is the whole document (including the selection), and `history_undo` / `history_redo` switch between snapshots.

## Known limitations

- Documents live only in process memory and do not communicate with a running OpenPhoto window; unsaved changes are lost when the process exits.
- `set_layer_visibility` records "Show Layer" / "Hide Layer" history, whereas toggling the eye in the application does not record history.
- `get_pixel` recomposites the whole image every time, which is slow for large images.
- `initialize` returns the protocol version requested by the client as is, without negotiation; JSON-RPC batch requests are not supported (messages in array form are ignored without a reply).
- There are no tools that require dragging or typing, such as brush, type, shapes, or mask painting, and there are no resources or prompts.

## Test coverage

`cargo test -p op-mcp`:

- `search_finds_blur_and_returns_definitions`, `search_empty_query_lists_tools`, `search_unknown_query_returns_empty`: search scoring and the returned definitions.
- `new_invert_and_pixel_roundtrip`, `gaussian_blur_and_undo_redo`, `canvas_and_layer_ops_roundtrip`: editing, undo/redo, canvas and layer operations, export.
- `mcp_initialize_list_call_roundtrip`: protocol-level initialize, tools/list, search_tools, and call_tool (an unknown tool is returned with `isError`).
- `selections_are_history_states`: blurring, then making a marquee selection and undoing, undoes only the selection and keeps the blur; deselecting with no selection adds no history.
- `sizes_and_filter_parameters_are_limited`: oversized dimensions and out-of-range filter parameters report errors and record no history, while those within range still take effect.
- `schemas_list_what_handlers_require`: the schemas of `fill_solid` and `paint_bucket` list `color` as required; passing `method` to `gradient_fill` reports an error.
