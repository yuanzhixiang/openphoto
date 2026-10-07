# state.rs: Application State and Document State

## Responsibilities

Holds all mutable state of the UI layer. `AppState` is the state of the whole application; `DocState` is the state of a single open document (the document itself, history, view, caches).

## AppState

- `docs` / `doc_order` / `active_doc`: the open documents, tab order, and current document. `add_document` appends a document at the end and makes it the current document; `close_document` closes a document, and when the closed one is the current document, the document to its left (or the first one if there is none) becomes the current document.
- `tool`: the current tool, defaulting to the Rectangular Marquee (matching the default of a fresh Photoshop install).
- `tool_slots`: the tool currently shown in each toolbar slot (the most recently used one in that group), initially the first in each group. `select_tool(t)` sets both the current tool and the tool shown in its slot.
- `foreground` / `background`: the foreground color defaults to `#14a5dc`, the background color to white.
- `recent` (`recent::RecentFiles`): File › Open Recent's list.
- `ruler_units` (`rulers::RulerUnit`) and `ruler_menu` (a ruler was right-clicked; its menu opens after the frame).
- `status_info` (what the status bar shows, `status_info.md`), `status_menu` (its arrow was clicked; the menu opens after the frame), `last_timing` (seconds the last document-changing command took, for Timing).
- `screen_mode` (`ScreenMode`: Standard, FullWithMenus, Full; `cycle` gives the next or previous for F / Shift+F), `hide_tools`, `hide_panels` (Tab, Shift+Tab), `full_screen_prompt` / `skip_full_screen_prompt` (Full Screen Mode's warning, `FULL_SCREEN_WARNING`, Photoshop's `$$$/FullScreenModeWarningWithShortcut` text with F). `set_screen_mode` asks first when entering Full Screen Mode; `enter_screen_mode` switches without asking: entering Full Screen Mode hides the tools and panels, leaving it shows them; `toggle_hidden` is Tab's rule (see `actions.md`).
- `view`: view toggles (`ViewOptions`, see `rulers.md`). Among them, `smart_guides` (Smart Guides, on by default) is toggled by View › Show › Smart Guides and the Properties panel's Guides section (see `smart_guides.md`). `DocState::smart_guides`: the Move drag's smart guides.
- `modify_dialog`: the Select › Modify dialog, `Some` while open.
- `new_document_dialog`: the New dialog, `Some` while open.
- `new_guide_dialog`: the New Guide dialog, `Some` while open.
- `marquee`: marquee tool options: combine mode, feather (pixels), anti-alias (on by default, matching Photoshop), style. The first three take effect when a selection is created; the Fixed Ratio / Fixed Size styles currently have no effect.
- `dodge`, `burn`, `sponge`, `blur`, `sharpen`, `clone_stamp`, `history_brush`: each retouching tool's own `PaintOptions` (default 30 px, hardness 0%; Dodge/Burn opacity is Exposure 50%, Blur/Sharpen opacity is Strength 50%, Sponge flow 50%). `retouch` (`RetouchOptions`): Dodge's and Burn's separate Range (default Midtones), whether Sponge uses Saturate (default no), Clone Stamp's Aligned (default yes). `paint_options(tool)` also returns these tools' settings.
- `smudge`, `pattern_stamp`, `background_eraser`, `color_replacement`: the `PaintOptions` of Smudge (opacity is Strength 50%), Pattern Stamp, Background Eraser, and Color Replacement (13 px, hardness 100%); `paint_options(tool)` returns them as well. `default_pattern()`: the Pattern Stamp's default pattern.
- `brush` / `pencil` / `eraser`: each painting tool's `PaintOptions` (size px, hardness, opacity, flow), saved separately per tool as in Photoshop. Defaults: Brush and Eraser 30 px, hardness 0%, opacity and flow 100%; Pencil 1 px. `paint_options(tool)` gets the current painting tool's settings. `size_step(size)` is the step for `[` and `]` (1 below 10, 5 for 10–50, 10 for 50–100, 25 for 100–200, 50 for 200–300, 100 above that); the size range is 1–5000.
- `eyedropper`: Eyedropper options (sample size `size`, 1 means Point Sample; `all_layers`). `EyedropperOptions::SIZES` is Photoshop's seven sample sizes.
- `type_options`: Type tool options (see `type_tool.md`).
- `typing_text()`: true when the current document has text being typed; `modal_open()` is also true then.
- `shape`: shape tool options (`ShapeOptions`: polygon sides 5, line weight 1 px, Photoshop's defaults).
- `move_options`: Move tool options (`MoveOptions`: `auto_select`, `show_transform_controls`, both off by default, matching Photoshop).
- `gradient`: Gradient tool options (`op_core::gradient::GradientOptions`: type, blend mode, opacity, reverse).
- `wand`: Magic Wand options: combine mode and region rules (tolerance, anti-alias, contiguous, all layers from `op_core::fill::BucketOptions`, with the same defaults as the Paint Bucket).
- `editing_background` and `picker_hsb`: whether the Color panel is editing the foreground or background, and the cached HSB. The HSB is cached so that the hue does not jump back to 0 on gray (saturation 0).
- `floating`: whether the floating panels (Info, Navigator, Histogram) are open (see `panels/floating.md`).
- `history_open`, `history_panel`: whether the History pop-out panel is open, and its tab and height.
- `fill_dialog`: the Fill dialog, `Some` while open.
- `bucket`: Paint Bucket options (defaults in `crates/op-core/src/fill.md`).
- `canvas_size_dialog`: the Canvas Size dialog, `Some` while open.
- `color_picker`: the Color Picker session, `Some` while open, containing the dialog and a `PickerTarget` (whether OK writes to the foreground color, the background color, Canvas Size's extension color, or the Fill dialog's Color...). `open_color_picker(target)` opens the foreground or background color picker with Photoshop's title.
- `swatches`: the Swatches panel's swatch list, initially a fixed set of 36 colors; the Color Picker's "Add to Swatches" appends to the end. Kept only for the current run.
- `alert`: an error message waiting to be shown.
- `delete_group_prompt`: the prompt shown when deleting a group that contains layers; `delete_layers()` deletes the selected layers and, on reaching a non-empty group, first shows this prompt ("Delete the group “name” and its contents or delete only the group?", Group and Contents / Group Only / Cancel). Both the DeleteLayer command and the Layers panel's delete button go through it.
- `flatten_prompt`: Flatten Image's "Discard hidden layers?" prompt (counts as a modal dialog while shown); `skip_flatten_prompt`: "Don’t show again" was checked in it, so it is not asked again during this run.
- Closing and quitting: `close_queue` is the documents waiting to be closed; `save_prompt` is the document currently being asked "Save changes?"; `quit_after_close` means quit once the queue is processed; `quit_approved` means the window may now close (see "Closing" in `actions.md`).
- `clipboard`: the clipboard used by Cut/Copy/Paste (see `clipboard.md`). By default it is not connected to the system clipboard (windowless tests must not alter the user's clipboard); `OpenPhotoApp::new` replaces it at startup with a version connected to the system clipboard.
- `typing`: whether a text field had keyboard focus in the previous frame, updated each frame before commands run. Decides whether the menu's Cut/Copy/Paste act on the text field or the document.
- `forward_events`: events to inject into egui's input for the next frame (used when the menu's Cut/Copy/Paste is handed to a text field; see `commands.md`).
- `image_size_dialog`: the Image Size dialog, `Some` while open. `image_size_extra`: how far its window was last enlarged (Photoshop points), for the next opening.
- `crop_presets`: the Crop tool's saved presets; `new_crop_preset` / `delete_crop_preset`: New Crop Preset... and Delete Crop Preset..., `Some` while open (both count in `modal_open`). `PickerTarget::CropShield`: the Color Picker for the crop shield's custom color.
- `trim_dialog`: the Trim dialog, `Some` while open.
- `adjust_dialog`: an adjustment or filter dialog (see `dialogs/adjust.md`), `Some` while open.
- `last_transform`: the most recently applied transform mapping, used by Edit › Transform › Again; kept only for the current run and shared by all documents.
- `transforming()`: true when the current document is in Free Transform; `modal_open()` is also true then.
- `last_filter`: the most recently successfully applied filter and its settings, used by Filter › Last Filter; kept only for the current run and shared by all documents.
- `filter_settings`: the settings in each filter dialog's fields the last time OK was pressed (stored by `AdjustKind`), restored the next time the same dialog opens; kept only for the current run and shared by all documents. `last_adjustments`: each adjustment dialog at its last OK, for opening with Option held.
- `modal_open()`: true when Canvas Size, Image Size, New, New Guide, Modify, Fill, Trim, an adjustment dialog, the Color Picker, the "Save changes?" confirmation, or an error alert is open; commands and single-key shortcuts do not run then.

## DocState

- `doc`: `op_core::Document`.
- `path`: the file the document was opened from or most recently saved to; `None` for a new, unsaved document.
- Unsaved changes: `saved_state` records the id of the history state that matches the file on disk (or, for a new document, the state at creation). `is_dirty()` is true when the current history state's id differs from it (so undoing back to the saved state counts as unmodified, matching Photoshop); `mark_saved()` records the current state as saved.
- `untagged`: the document has no embedded color profile (the tab title shows "#"). True for opened files, false for new documents.
- `history`: the document's `op_core::History`. The first state is named "Open" (opened file) or "New" (new document).
- `view`: view state, see below.
- Composite cache: `canvas_image()` returns the current composite, recompositing only when the document `revision` changes.
- Layer thumbnails: `layer_thumbnail()` caches per layer and regenerates after the document `revision` changes; downscaled with nearest neighbor.
- Move: `move_drag` is the move in progress and the drag start point (document pixels).
- `transform_controls_bounds()`: the box for the Move tool's Show Transform Controls (document pixels), i.e. the result of `op_core::transform::bounds`, `None` when transforming is not possible; cached by (revision, current layer, selection revision).
- Painting: `stroke` is the stroke in progress and its tool; `last_paint_point` is where the previous stroke ended, used for Shift+click straight lines.
- `text_edit`: the text being typed (see `type_tool.md`).
- `shape_drag`: the start point and current point of a shape tool drag.
- `gradient_drag`: the start point and current point of a Gradient tool drag (document pixels).
- `clone_source`, `clone_offset`, `picking_clone_source`: the Clone Stamp's source point, aligned offset, and the flag "this press is setting the source point" (see `document_view.md`).
- `pointer`: the pointer position on the document (document pixels), `None` when the pointer is not over the canvas; used by the Info panel.
- `movable_layers()`: the number of selected layers that Align / Distribute would move, cached by document revision, selection revision, and selected layers. The enabled state of the align buttons and menu items is queried a dozen or so times per frame, each time scanning the bounding box of layer pixels; without the cache, dragging in a large document stutters noticeably.
- Histogram and composite thumbnail: `composite_histogram()`, `composite_texture(ctx, max_px)`, both cached by document revision, used by the Histogram and Navigator panels. Both take the canvas's existing composite (`canvas_image()`) instead of compositing again; when Quick Mask is on (the canvas has a red overlay), the Navigator thumbnail is still composited separately.
- `guide_drag`: the guide being dragged (`index` is `None` when dragging out of a ruler, otherwise the index of the guide being moved).
- `crop`: the Crop tool's crop box (see `crop_tool.md`). `perspective_crop`: the Perspective Crop tool's box (`perspective_crop.md`).
- `free_transform`: the Free Transform session (see `free_transform.md`).
- `lasso`: the lasso path being drawn (points in document pixel coordinates, combine mode, whether it is the Polygonal Lasso).
- Canvas image (`canvas_image`): in Quick Mask mode, red is overlaid on the composite, 50% where unselected (Photoshop's default "Masked Areas" display), varying linearly with the gray value; affects display only.
- `sample_average(x, y, size, scope)`: Eyedropper sampling (`SampleScope`: current layer, current and below, all); see `document_view.md`.
- `renaming`: the layer being renamed in the Layers panel and the text being typed (see `panels/layers.md`).
- Marquee drag: `marquee_drag` holds the drag's start point, current point (document pixels), combine mode, and whether Shift/⌥ has already been used to choose the combine mode.
- Marching ants outline: `selection_outline()` caches the outline segments by selection version.
- Snapshot thumbnail: when a `DocState` is created, a thumbnail of the document's initial state is generated once (longest side 96 px, nearest neighbor) for the snapshot row at the top of the History panel, and is not updated afterwards (the snapshot represents the document as opened).

### History Integration Rules

- `record(name)`: records a history state immediately.
- Continuous edits (dragging opacity, Fill, etc.): call `mark_pending()` during the edit and `commit_pending(name)` when it ends; a state is recorded only if a change actually happened. This way one drag produces only one history state.
- `undo` / `redo` / `toggle_last_state` / `jump_to_state` / `delete_states_from` all clear the uncommitted continuous-edit flag.

### View

- `zoom`: the number of physical pixels per document pixel; 1.0 = 100%.
- `offset`: the offset of the document center relative to the viewport center (logical points). The center rather than the top-left corner is used as the reference so the document stays centered when the window size changes, matching Photoshop.
- `flip`: View › Flip Horizontal; the canvas is shown mirrored left to right about the document's center (see `document_view.md`); the pixels are unchanged. Not saved with the document.
- `rotation`: the view rotation in degrees, clockwise, within ±180° (Rotate View; see `document_view.md`). Not saved with the document.
- `initialized` / `viewport`: on first display, the initial zoom is decided only after the viewport size settles (see `document_view.md`); `viewport` is used by shortcut zooming.

## Known Limitations

- Undo restores layer visibility and the currently selected layer to how they were in the snapshot; Photoshop does not record these two by default, and undo does not change them.
- `new_document_recent`, `new_document_saved`, `new_document_welcome_closed`: the New Document dialog's Recent list (newest first, no duplicates, at most 20), presets saved with the save icon, and whether the Recent page's welcome box has been closed; kept only for the current run (see `dialogs/new_document.md`).
- `healing_brush`, `spot_healing`: the Healing Brush's and Spot Healing Brush's `PaintOptions` (13 px, 100% hardness), returned by `paint_options(tool)`.
- `DocState::patch_drag`: while the Patch tool or Content-Aware Move drags the selection, where the drag started and where it is (document pixels); the selection outline is drawn moved by the difference.
- `DocState::magnetic` (`MagneticPath`): the Magnetic Lasso under way: its `EdgeMap` (built from the merged image when it starts), the indices of the anchors in `DocState::lasso`'s points, and how many points are fixed (the rest is the live wire to the pointer).
- `DocState::quick` (`QuickStroke`): a Quick Selection stroke under way: the `QuickSelect` region, the selection it started from, how they combine, and where the last dab was. `DocState::object_drag`: the Object Selection tool's rectangle being dragged. `DocState::history_source` / `history_brush_snapshot`: the History Brush's source state (see `panels/history.md`). `DocState::fade` / `record_fadeable` / `can_fade` / `fade_source`: the last fadeable edit and what Edit › Fade would work on (`FadeSource`: name, layer, pixels before and after); `AppState::fade_dialog` (`FadeState`: the dialog, its source and what its preview last showed). See `dialogs/fade.md`. `DocState::selection_stroke` / `overlay`: a Selection Brush stroke under way and the cached overlay texture (see `selection_brush.md`). `AppState::selection_brush`: the Selection Brush's `PaintOptions` (200 px, hard). `DocState::color_samplers` / `sampler_drag`: the Color Sampler tool's points and the one being dragged (see `document_view.md`). `DocState::ruler_origin` / `origin_drag`: the rulers' zero point and the pointer while it is dragged out of the corner (see `rulers.md`). `DocState::snap`: what the drag in progress snaps to (`snap::Targets`, gathered when the drag starts; see `snap.md`).
