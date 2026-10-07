# document_view.rs: Document window (canvas and status bar)

## Component responsibility

Displays one document: the canvas area (zoom, pan, tool input) and the status bar at the bottom. Canvas pixels are drawn by `op-render` through an egui-wgpu paint callback; this file only computes the view transform and handles input.

## Usage

Displays the current document, below the document tab bar (`doc_tabs.rs`).

## Data input

`AppState` (reads the current tool and each tool's options; the Eyedropper tool writes the foreground or background color) and the document id. View state is stored in `DocState::view`.

## Layout and visuals

Sizes are measured 1:1 against Photoshop (pt):

- The canvas area is filled with the pasteboard color `#282828`.
- On the right is the vertical scrollbar column, 17 wide: on the left, a 1 pt `#2e2e2e` line followed by a 1 pt `#464646` line; on the right, a 1 pt `#454545` line; the track in between is `#4a4a4a`. The thumb is a 12-wide rounded bar, color `#696969`.
- At the bottom is the 16-high status bar (see below).
- The canvas is drawn on the pasteboard according to the view transform; transparent areas show a checkerboard, and a pixel grid appears above 800% when View › Show › Pixel Grid and Extras are on (implemented by `op-render`).
- The screen position of the document's top-left corner is snapped to physical pixels so pixels are crisp at 100%.

## Zoom

- The zoom value is "physical pixels per document pixel"; 1.0 displays as 100%, the same as Photoshop (on a Retina display, 100% also means one image pixel per physical pixel).
- Range 1%–12800%.
- Preset levels match Photoshop: 1, 2, 3, 4, 5, 6.25, 8.33, 12.5, 16.67, 25, 33.33, 50, 66.67, 100, 150, 200, 300, 400, 500, 600, 700, 800, 1200, 1600, 2400, 3200, 6400, 12800 (%). ⌘+ / ⌘- and the Zoom tool step through these levels.
- Percentage display: no decimals for whole numbers (`100%`), otherwise two decimals (`94.90%`).
- When zooming around a point, the document content under that point stays put; keyboard-shortcut zooms are centered on the viewport center.
- Fit on Screen: zooms so the whole document just fits, and centers it. Fill Screen (`fill_screen`, the button on the Hand and Zoom tool options bars): zooms so the document fills the entire viewport (the larger of the width and height ratios) and centers it. 100%: zooms to 1.0 and centers.
- `center_on(state, p)`: scrolls the view so document point `p` is at the center of the window (used by Navigator).
- 200% (`zoom_to`): zooms to 2.0 around the viewport center. Print Size (`print_size`): zooms to `72 / resolution × pixels per point`, so one inch displays as 72 pt. Actual Size (`actual_size`): zooms around the viewport center to the display's pixel density over the document's resolution, so an inch of the image is an inch on the screen; the density is the main display's native pixel width over its physical width (`app_kit::screen_ppi`, 255 ppi on a 14-inch MacBook Pro, giving 354.17% for a 72 ppi image, as Photoshop shows), or 72 per point elsewhere than macOS. Fit Layer(s) on Screen (`fit_layers`): the non-transparent pixels of the current layer fill the viewport and are centered; no change for an empty layer.

## View rotation

- `View::rotation` (degrees, clockwise, within ±180°) turns the canvas about the document's center, as Photoshop's Rotate View does; the pixels stay unchanged. The shader gets it as the canvas rotation (`CanvasView::rotation`, pivot at the document's center); while a crop is in progress the crop's own rotation is used and the view is drawn upright (`view_angle` returns 0).
- `to_doc` / `to_screen` include the rotation, so every tool works in document space in a turned view: the marquee, Object Selection rectangle, and shape previews are computed as document-space shapes and mapped corner by corner (they show turned on screen). `visible_rect` is the bounding box of the four viewport corners.
- View › Flip Horizontal (`View::flip`) mirrors the view left to right about the document's center, after the rotation; `to_doc` / `to_screen` include it, so tools keep working in document space. It is ignored while cropping, like the rotation. `center_on` places a document point at the window's center through the rotation and flip.
- Rotate View tool: dragging turns the view by the angle the pointer sweeps around the document center's screen position, starting from the rotation at the press; Shift snaps to multiples of 15°. While dragging, a compass is drawn at the center (a 60 pt translucent dark disc with a light ring; a needle whose red half points to the image's top). Esc returns the view to 0°. The options bar's angle field, dial, and Reset View act on the same value (see `options_tools.md`).

## Snapping

When a drag starts with the Rectangular or Elliptical Marquee, the Move tool, or a shape tool, `snap::begin` gathers what it can snap to (View › Snap; rules in `snap.md`). The marquee's start and dragged points and the shape's two points go through `snap::point`; the Move tool's offset goes through `snap::offset` before it is rounded to whole pixels. Control held: no snapping.

## Screen modes

The pasteboard is black in Full Screen Mode (Photoshop's default) and `PASTEBOARD` gray otherwise. The document tabs above the view and the hidden tools and panels are laid out by `lib.rs`; the view just gets a larger area.

## Initial view

- The layout may not be stable in the first few frames (the viewport size reported on the first frame is unreliable), so the initial zoom is decided only after the viewport size is the same for two consecutive frames; while waiting, a repaint is actively requested.
- If the document fits entirely in the viewport at 100%, 100% is used; otherwise Fit on Screen. In both cases the document is centered.

## Interaction

- Trackpad pinch, ⌘+scroll wheel: continuous zoom centered on the pointer.
- Plain scroll (trackpad two-finger swipe, scroll wheel): pan.
- Dragging with Space held, dragging with the Hand tool, or dragging with the middle mouse button: pan.
- Zoom tool: click zooms in one level, ⌥-click zooms out one level, centered on the click point. When Zoom Out is pressed in the options bar (sets `zoom.out`), the two are swapped: click zooms out, ⌥-click zooms in, and the cursor swaps accordingly.
- Magic Eraser, Red Eye tool: a click runs `fill::magic_erase` / `fill::red_eye` according to the options bar (Magic Eraser's Tolerance, Anti-alias, Contiguous, Sample All Layers, Opacity, `options_tools::magic_eraser_options`; Red Eye's Pupil Size, Darken Amount, `red_eye_options`), and records "Magic Eraser" / "Red Eye" when something changed; when the layer is locked, hidden, etc., an alert "Could not use the magic eraser / red eye tool because …" appears.
- Color Sampler: see "Color Sampler" below.
- Selection Brush: painting the selection, shown as an overlay instead of marching ants while it is the current tool (see `selection_brush.md`).
- Eyedropper tool: on press or drag, samples a color according to the eyedropper options (`DocState::sample_average`: a Sample Size square centered on the point, clipped to the canvas, alpha-weighted average; Sample option: Current Layer samples the current layer, Current & Below samples the composite of the current layer and the layers below it, All Layers samples the full composite; the two "No Adjustments" options are the same as the corresponding plain options (there are no adjustment layers yet)), and sets it as the foreground color; with ⌥ held it sets the background color. The sampled color is always opaque; nothing changes when the point is outside the document or the sampled area is fully transparent. No history is recorded.
- Sampling ring (options bar Show Sampling Ring, checked by default): while the Eyedropper is held down, a ring is drawn around the pointer (inner radius 38 pt, outer radius 58 pt, with a 2 pt gray line on both the inside and outside edge); the top half is the color being sampled, the bottom half is the original color at the time of the press (the background color with ⌥), drawn on the foreground layer, over the canvas.

## Type tool

When the current tool is the Horizontal Type tool, canvas input is handed to `type_tool::input` (see `type_tool.md`). Switching to another tool commits the text in progress. The text insertion line is drawn after the other overlays.

## Shape tools

- Rectangle, Ellipse, Triangle, Polygon, Line: drag from the press point; with Shift held, rectangle-type shapes become square (using the larger of the two directional lengths) and the line snaps to multiples of 45°; with ⌥ held, the shape expands outward in both directions with the press point as the center.
- While dragging, a 1 pt blue outline of the shape is drawn (in `draw_selection`, together with the selection preview).
- On release, a new shape layer is created with the foreground color (`op_core::shape::add_shape_layer`; polygon side count and line weight come from `AppState::shape`), recording "Rectangle Tool", "Ellipse Tool", "Triangle Tool", "Polygon Tool", "Line Tool"; nothing happens when the shape is empty.
- The drag in progress is stored in `DocState::shape_drag`.

## Gradient tool

- Dragging from the press point to the release point sets the gradient direction; with Shift held the direction snaps to multiples of 45° (length unchanged). While dragging, a thin black-and-white line with white endpoints shows the direction.
- On release, the tool's gradient (`AppState::tool_gradient`: Foreground to Background unless one was picked or edited, with the options bar's Method) is drawn on the current layer according to the gradient options and the options bar's Dither (`op_core::gradient::gradient_with`), recording "Gradient"; nothing happens when the start and end points are the same; when the layer is hidden or pixel-locked, Photoshop's alert appears.
- The drag in progress is stored in `DocState::gradient_drag`.

## Rulers and guides

- When rulers are on, the canvas area shrinks and the rulers are drawn along the top and left of the window; pressing and dragging on a ruler starts dragging out a guide (see `rulers.md`).
- While a guide is being dragged, canvas input goes only to the guide drag, not to the tool.
- When the current tool is the Move tool or ⌘ is held, pressing on a guide starts moving the guide (taking precedence over the Move tool moving the layer); hovering over a guide shows a resize cursor.
- Drawing order: image, grid, layer edges (Show › Layer Edges, with Extras on: a 1 pt blue `#2c8be8` outline around the non-transparent pixels of each selected layer, following the view's rotation and flip), selection marching ants (when Extras and Show › Selection Edges are on), guides, rulers, transform box, crop box.

## Crop tool

When the current tool is the Crop tool, canvas input is handed to `crop_tool::input`, the cursor is determined by `crop_tool::cursor`, and the crop box and shield are drawn after the transform box (see `crop_tool.md`). When the current tool is not the Crop tool, a changed crop box is committed first and the box is dropped. The Perspective Crop tool is wired the same way (`perspective_crop::input`, `cursor`, `draw`; its box is committed when another tool is picked, see `perspective_crop.md`). Neither tool takes input while a dialog is open (`AppState::modal_open`), so Enter and Escape reach the dialog.

## Free Transform

While the document is in Free Transform, presses, drags, Enter, and Esc on the canvas all go to `free_transform::input` (see `free_transform.md`), not to the current tool; the cursor is determined by `free_transform::cursor`; the transform box is drawn after the selection marching ants; the brush outline is not shown. On commit, the mapping is recorded as `AppState::last_transform`.

## Color Sampler

- A click inside the image places a color sampler on the pixel under it (`DocState::color_samplers`, document pixels; at most ten, `MAX_SAMPLERS`, after which clicks place nothing) and keeps following the pointer until the button is released. Pressing on a marker (within 7 pt of its center) drags that sampler; ⌥-clicking a marker removes it; a sampler let go outside the image is removed. Samplers are not recorded in the history, as in Photoshop. The options bar's Clear All removes them all.
- Markers show while the Eyedropper or the Color Sampler is the current tool and Extras is on, centered on their pixel (Photoshop puts them about a physical pixel from the pixel's corner at high zoom). Measured on Photoshop 2026: `#ececec`, a ring 6.75 pt in outer radius and 2 pt thick, a 2 pt dot in the middle, four 2 pt arms from the ring out to 11.75 pt, and the sampler's number (1–10, 10.5 pt) below right of the center.
- The Info panel reads each sampler's color (see `panels/info.md`).

## Magic Wand

- Click: gets a region according to the Magic Wand options (`op_core::fill::magic_wand`, the same region rule as the Paint Bucket) and combines it with the current selection using the combine mode (with an existing selection, Shift adds, ⌥ subtracts, both together intersect, same as the marquee), recording "Magic Wand". Nothing happens when the point is outside the document.

## Lasso tools

- Lasso: drag from the press point; a point is recorded every time the pointer moves at least 1 screen pixel, and the path is shown as marching ants. On release, the path is closed by joining its ends, and a polygon selection (`Selection::polygon`) is created according to the marquee options (Feather, Anti-alias; the combine mode is determined by modifier keys at press time), recording "Lasso". No selection is created when the area enclosed by the path is zero. A click without dragging deselects when there is a selection (same as the marquee).
- Holding ⌥ with the Lasso temporarily turns it into the Polygonal Lasso (matching Photoshop): releasing the mouse while ⌥ is held during a drag does not close the path (`LassoPath::held`), and a rubber-band edge follows from the last point to the pointer; a click adds a corner, and pressing and dragging continues freehand drawing; releasing ⌥ (with the mouse not pressed) closes the path, still recording "Lasso".
- Polygonal Lasso (Enter, Esc, and ⌫ are ignored while a text field has focus): a click places the first corner and starts; each subsequent click adds a corner, and a rubber-band edge follows from the last corner to the pointer. Double-clicking, pressing Enter, or clicking near the first corner (within 5 pt) when there are three or more corners closes the path and creates the selection, recording "Polygonal Lasso"; ⌫/Delete removes the last corner (removing all of them cancels); Esc cancels.
- The path in progress is stored in `DocState::lasso`.
- Magnetic Lasso (`magnetic_input`): a click starts it: the edge map (`op_core::magnetic::EdgeMap`) is built from the merged image with the bar's Contrast, the start point snaps to the strongest edge within Width, and the combine mode comes from modifiers as for the other lassos. Moving the pointer (no button) draws the live wire from the last fixed point to the pointer snapped within Width (`EdgeMap::trace` with a margin of Width). Anchors fall every 10 + (100 − Frequency) × 0.9 points of path. A click fixes the outline so far with an anchor. Double-click, Enter, or clicking within 5 screen points of the start closes it, recording "Magnetic Lasso"; ⌫/Delete removes the last anchor (removing all of them cancels); Esc cancels. The path in progress is `DocState::magnetic`.
- `finish_lasso`: the shared closing of lasso outlines (polygon selection, feather, combine, record).

## Quick Selection and Object Selection

- Quick Selection (`quick_input`): pressing starts a stroke (`op_core::smart_select::QuickSelect` over the merged image with Sample All Layers, else the active layer). Combine mode: ⌥ or the bar's Subtract → subtract, Shift or the bar's Add → add, otherwise replace. A dab at the press and then every half radius of pointer travel; the selection updates live (Enhance Edge feathers 1 px; the marquee Feather applies). Release records "Quick Selection". A stroke in New mode switches the bar's mode to Add (`quick.mode` = 1), as Photoshop does. The stroke in progress is `DocState::quick`.
- Object Selection (`object_input`): dragging draws a rectangle (marching ants, `DocState::object_drag`); on release `smart_select::object_in_rect` runs on it; with Hard Edge off the result is feathered 1 px; it is combined by the bar's mode with Shift/⌥ (`selection_op`), recording "Object Selection". When nothing is found, nothing changes. Its Lasso mode works like Rectangle.

## Healing tools

- Healing Brush: ⌥-click picks the source (shared with the Clone Stamp: `clone_source` / `clone_offset`); Aligned (`heal.aligned`, off by default); Sample scope (`heal.sample`); the stroke blends the source's texture into the target's surroundings (`paint.md`), recording "Healing Brush". Without a source the alert "Could not use the healing brush because the area to heal has not been defined (option-click to define a source point)." appears.
- Spot Healing Brush: the stroke kind `SpotHeal`, on the current layer or on all layers (`spotheal.all_layers`), recording "Spot Healing Brush".
- Patch and Content-Aware Move (`patch_input`): without a selection, or when pressing outside it, they draw a freehand selection like the Lasso (recording "Lasso"). Dragging inside the selection moves its outline (`DocState::patch_drag`, the outline drawn offset). On release: Patch with Source → `heal::patch`, Destination → `heal::patch_to` (the selection follows); Content-Aware Move with Move → `heal::content_aware_move`, Extend → `heal::patch_to`; the selection follows the moved pixels. Records "Patch Tool" / "Content-Aware Move". A zero-length drag does nothing.
- When the window size changes, the document stays centered (the view offset is stored relative to the viewport center).

## Marquee tools

Applies to the Rectangular, Elliptical, Single Row, and Single Column Marquee:

- **Drag**: draws a rectangle or ellipse from the press point; rectangle edges snap to whole pixels. Holding Shift while dragging constrains to a square/circle; holding ⌥ draws outward from the start point (the start point is the center). A marching-ants preview is shown live while dragging.
- **Combine mode**: if a selection already exists when the drag starts, Shift adds, ⌥ subtracts, Shift+⌥ intersects, and in this case those keys no longer act as constraints; otherwise the combine mode selected in the options bar is used.
- **Release**: creates the shape (the ellipse is anti-aliased or not according to "Anti-alias" in the options bar; when "Feather" is greater than 0 it is feathered first), merges it into the selection, and records the history "Rectangular Marquee" or "Elliptical Marquee". Nothing happens when the width or height is less than 1 pixel.
- **Click** (no drag): when there is a selection and neither Shift nor ⌥ is held, deselects and records "Deselect", matching Photoshop.
- **Single Row/Single Column Marquee**: a click selects the entire row or column at that point (1 pixel), combined by the same rules, recording "Single Row Marquee" or "Single Column Marquee".

## Move tool

- Smart guides: while dragging, the move lines up with the canvas's and other layers' edges and centers, shown by magenta lines (`smart_guides.md`).

- Drag: moves the pixels of the current layer; with a selection, only the selected pixels move (rules in `crates/op-core/src/move_tool.md`). The offset is the drag distance rounded to whole pixels, and each step is recomputed from the pixels at the start. On release, "Move" is recorded if anything actually moved.
- When moving is not possible (locked, hidden, background layer without a selection), Photoshop's alert appears when the drag starts.
- Auto-Select (options bar, off by default): on mouse press, makes the topmost visible layer showing pixels under the pointer the current layer (`Document::layer_at`), then drags it as usual. Holding ⌘ inverts this (temporarily on when off, temporarily off when on), matching Photoshop. When no layer has pixels under the pointer, the current layer stays the same. No auto-select happens when pressing on a transform control handle. Switching the current layer records no history.
- Show Transform Controls (options bar, off by default): draws the same blue box and 8 handles as Free Transform around the current layer's non-transparent pixels (around the selection when there is one), without the center reference point; not drawn for a background layer without a selection, or for hidden or locked layers (the same rule as whether Free Transform can start; see `bounds` in `crates/op-core/src/transform.md`). Starting a drag on a handle enters Free Transform, which takes over this drag; a drag elsewhere inside the box is still a normal move. The box extent is cached in `DocState` keyed by (revision, current layer, selection revision) and is not recomputed every frame. The box is not drawn when Extras is off (⌘H).
- Arrow-key nudging: see `actions.md`.

## Paint Bucket

Clicking the document: fills the region of similar color at the click point with the foreground color according to the Paint Bucket options (see `crates/op-core/src/fill.md`), recording "Paint Bucket". When filling is not possible, Photoshop's alert appears. The cursor is a crosshair.

## Painting tools

Applies to the Brush, Pencil, Eraser, and retouching tools (Dodge, Burn, Sponge, Blur, Sharpen, Clone Stamp, History Brush; stroke algorithms in `crates/op-core/src/paint.md`):

- Pressing on the canvas starts a stroke (using the current tool's size, hardness, opacity, flow; the Pencil's flow is fixed at 100% with no soft edge), dragging continues it, and on release one history entry is recorded: "Brush Tool", "Pencil", "Eraser", "Dodge Tool", "Burn Tool", "Sponge Tool", "Blur Tool", "Sharpen Tool", "Clone Stamp", "History Brush", "Smudge Tool", "Pattern Stamp", "Background Eraser", "Color Replacement Tool", "Healing Brush", "Spot Healing Brush". The Brush and Pencil paint with the foreground color using the options bar Mode (all blend modes plus Behind and Clear; see "Painting modes" in `paint.md`); on a background layer the Eraser uses the background color. Eraser Mode: Brush (uses the brush size, hardness, opacity, flow), Pencil (hard edge, no anti-aliasing, flow 100%), Block (fixed as a square 16 pixels on a side on screen, opacity and flow both 100%, matching Photoshop).
- The stroke kind is decided at press time (`stroke_kind`): Dodge/Burn by their respective Range, Sponge by Mode, Blur/Sharpen, and Clone Stamp and History Brush as below.
- Smudge: Strength (options bar, default 50%) is the stroke strength itself and is no longer multiplied by opacity. Pattern Stamp: tiles the default pattern (`state::default_pattern`: green dots on a dark green ground, 18 × 22 pixels) from the document origin; Impressionist and similar options have no effect yet. Background Eraser: sampling, Limits, Tolerance, and Protect Foreground Color come from the options bar (`options_tools::color_match`); when it acts on a background layer, the background is first converted to a normal layer "Layer 0" (matching Photoshop). Color Replacement: Mode (Hue / Saturation / Color (default) / Luminosity), sampling, Limits, Tolerance, using the foreground color. These four tools each have their own brush (`smudge`, `pattern_stamp`, `background_eraser`, `color_replacement` in `AppState`: 13 px hard edge).
- Clone Stamp: ⌥-click sets the source point (this press does not paint, even if ⌥ is released first). On later presses, the offset is "press position − source point"; with Aligned checked (default), the offset stays fixed after the first stroke, otherwise every stroke restarts from the source point. Sampling follows the options bar Sample: Current Layer (the current layer), Current & Below (the composite of the current layer and the layers below it, `Document::sample_source`), All Layers (the full composite), all using the pixels at press time. When there is no source point, the alert "Could not use the clone stamp because the area to clone has not been defined (option-click to define a source point)." appears. The source point is stored per document in `DocState::clone_source` / `clone_offset`.
- History Brush: paints with the pixels of the same layer taken from its source (`DocState::history_brush_snapshot`): the history state picked in the History panel's source column, or by default (and when that state is no longer kept) the state when the document was opened (or created), the first history entry; when that state does not contain this layer or its size differs, the alert "Could not use the history brush because the history state does not contain a corresponding layer." appears.
- Holding Shift on press: draws a straight line from where the previous stroke ended to the press point, then continues the stroke.
- When the layer cannot be painted (hidden, pixels locked), Photoshop's alert appears at the moment of the press and no stroke starts.
- Cursor: for all these tools, when the brush's on-screen diameter is at least 4 points, the system cursor is hidden and a circle the size of the brush is drawn (a translucent black outer ring plus a thin white ring); when smaller, a crosshair cursor is shown.

## Marching ants

- The selection outline (see `outline` in `crates/op-core/src/selection.md`) is converted from document pixels to screen, and only segments within the viewport are drawn.
- Each segment is first drawn as a white line 1 physical pixel wide, then overlaid with a black dashed line with 4-physical-pixel dashes. The dash phase is determined by the segment position, so the pattern stays continuous along curved edges made of many short segments.
- The dashes advance one step every 1/8 second; when there is a selection, the UI repaints every 120 milliseconds.

## Cursors

- Panning state (Space, Hand, middle button): hand; a grabbing hand while dragging.
- Zoom tool: magnifier; zoom-out with ⌥ held.
- Eyedropper, shape tools, Gradient, Magic Wand, Lasso, Polygonal Lasso, Paint Bucket, the four marquee tools, and painting tools when the brush is very small: crosshair.
- Move tool: move cursor. Type tool: text cursor. Others: default cursor.

## Scrollbars

- The scrollable range in each direction is "document size + one viewport", so a document edge can scroll to the center of the viewport. Thumb length and position are computed from this range.
- Dragging the thumb pans the document.
- After panning (scrolling, Hand, Space-drag, dragging a thumb) and zooming, the offset is clamped to this range.

## Status bar

Measured against Photoshop 2026's status bar (pt from the document area's left edge and the bar's top):

- Height 16, background in the panel color. A 1 pt top line: `#424242` over 0–230, `#3e3e3e` over the arrow's cell (230–243); none over the scrollbar.
- Zoom box 0–59: `#454545` with a 0.5 pt `#4a4a4a` top edge; the percentage centered (x 29.5, y 8) in Source Sans 3 13.5 pt, `#f0f0f0`. Then a 1 pt `#424242` divider (59–60).
- Info cell 60–230: the text of the item picked in the arrow's menu (`status_info.md`), centered (y 8.75) in 12.25 pt, `#d6d6d6`. Photoshop sets both texts in Adobe Clean; these sizes give the same widths. For Save and Download Progress the cell is an empty `#383838` track instead; Download Progress adds its cancel button (an 11 pt `#b7b7b7` disc at x 219, y 8.5, with a dark `#111111` ×).
- The arrow's cell 230–243: a thin chevron (1 pt `#e0e0e0` line from (234.5, 4.75) to (237.5, 8.75) to (234.5, 12.75)). Clicking the cell opens the menu (`AppState::status_menu`; `lib.rs` shows it after the frame through `NativeMenu::popup_status`): a native macOS menu as in Photoshop, listing the items with the current one checked, opening down from the pointer; a pick runs `Command::StatusInfo`.
- A 1 pt `#424242` divider (243–244) the full height, then from 244 up to the vertical scrollbar column the horizontal scrollbar: track `#4a4a4a` (also over the top line), thumb 10 high, color `#696969`. The corner below the vertical scrollbar column stays the panel color.
- The zoom box (`zoom_box`) is a text field: clicking it selects the whole percentage; a typed value (with or without "%", e.g. `200` or `50%`) zooms around the window's center when Enter is pressed or the box loses focus, clamped to 1%–12800%; Esc or text that is not a positive number leaves the zoom unchanged, and the box shows the current zoom again. While it has focus, single-key shortcuts are off (digits go into the box). Until it is edited, the percentage is painted as a label rather than by the text edit, so it stays exactly where it was (the text edit would place it a fraction of a pixel off). The selection highlight color while editing (`#2c5fb8`) has not been measured against Photoshop.
- The native menu's rows are taller than Photoshop's (24 pt against 18 pt): macOS lays out menus by the SDK an app is built with, and Photoshop's is older.

## Visible area

`visible_rect(state, ppp)` returns the viewport's extent in document pixel coordinates `[x0, y0, x1, y1]`, which may extend beyond the canvas. Paste uses it to decide where to paste (see `clipboard.md` in `op-core`).

## Known limitations

- The marquee tool's "Fixed Ratio" and "Fixed Size" styles have no effect.
- Scrollbar thumb length does not exactly match Photoshop (Photoshop computes the scrollable range differently), and clicking the track does not page.
