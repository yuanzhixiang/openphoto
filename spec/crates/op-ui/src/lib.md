# lib.rs: Application entry point and overall layout

## Responsibilities

Defines `OpenPhotoApp` (implements `eframe::App`), which:

- At startup installs fonts and styles, registers the canvas renderer, reads the Crop tool's saved presets (`CropPresets::default_store`), on macOS installs the native menu bar (and, once the window's view is known, enables native dialog menus with `native_popup::set_available`) and marks the window as sRGB color space, then opens the files passed on the command line.
- Each frame, processes input, lays out the regions and shows overlays in a fixed order.
- Lays out the document tab bar (`doc_tabs.rs`) and the current document's view (`document_view.rs`).

## Startup

1. `theme::install_fonts`, `theme::apply_style`.
2. `op_render::install` registers the canvas pipeline. It depends on the wgpu rendering backend and panics if it is missing (the app supports only wgpu).
3. On macOS, `menu::NativeMenu::install`. `new_headless` skips this step, for windowless UI tests (see `ui_tests.md`).
4. On macOS, `color_management::use_srgb` marks the window's Metal layer as sRGB (see `color_management.md`). `new_headless` has no window and also skips this.
5. Connects to the system clipboard (`clipboard::Clipboard::new(true)`). `new_headless` skips this step, so tests do not change the user's clipboard.
6. If there are command-line files, opens them one by one; otherwise creates a new 1920×1080 "Untitled-1" document with a white background.

- The real app (`new`, not the headless tests) also loads File › Open Recent's list from its store and adds the files it was started with (see `recent.md`).

## Per-frame flow

1. Files dropped into the window are opened one by one.
2. Runs commands: first records whether an input field has focus (`AppState::typing`). On macOS with the native menu, takes the menu events and the ⌘= that egui additionally captures; without a native menu (other platforms, windowless tests), egui handles all shortcuts (see `commands.md`).
3. Handles single-key shortcuts (tools, D, X, F, Tab), then sets the window up for a changed screen mode (`apply_screen_mode`: Full Screen Mode With Menu Bar maximizes (zooms) the window, Full Screen Mode makes it full screen, Standard undoes it; a window that was already maximized stays so).
4. Lays out the regions in order (egui panels added first are on the outside), with sizes measured 1:1 from Photoshop (pt):
   - Title bar (macOS only, height 29, including a 1 pt separator at the bottom; Standard Screen Mode only)
   - Options bar (height 33) and left toolbar (width 42, including a 3 pt dark edge on the right); left out while `hide_tools` (Tab)
   - Right panel column (width 322) and the icon column to its left (width 43, including a 3 pt divider bar on each side); left out while `hide_panels` (Tab, Shift+Tab), together with the History popup and the floating panels
   - Center: with a document, the document tab bar on top (Standard Screen Mode only; the full screen modes show only the active document) and the current document's view below; without a document, only the pasteboard background (black in Full Screen Mode).
   None of these panels use egui's built-in separators; each region draws its own borders as in Photoshop.
5. Determines the current document: if the current document no longer exists, switches to the last tab (see `ensure_active` in `doc_tabs.rs`).
6. Overlays: the History popup panel (when `history_open`), floating panels (Info, Navigator, Histogram, see `panels/floating.md`), the Canvas Size dialog, the Image Size dialog, the Fill dialog, the Trim dialog, adjustment dialogs, the "Save changes?" confirmation, the Color Picker (drawn above the previous two), macOS menu state sync, the status bar's menu when its arrow was clicked (`status_menu`) and the units menu when a ruler was right-clicked (`ruler_menu`), and a dialog dropdown's native menu (`native_popup::take_request`, its pick handed back with `native_popup::deliver`) (macOS only; without a native menu the requests stay), the error alert dialog, and Full Screen Mode's warning (`full_screen_prompt`: Photoshop's text, the app icon, "Don’t show again", Cancel and "Full Screen"; Full Screen enters the mode and remembers the checkbox in `skip_full_screen_prompt`).

## Frame time log

When started with the environment variable `OPENPHOTO_FRAME_LOG` set (any value), each frame prints one line `frame <milliseconds> ms` to stderr, the interval since the previous frame (including rendering). Used during development to check in the real app whether operations such as dragging are smooth (moving a layer on a 3000 × 1080 document takes about 11 ms per frame).

## Closing the window and quitting

- At the start of each frame, checks for window close requests (clicking the window's close button, etc.): if quitting has not been approved yet (`quit_approved`) and a document has unsaved changes, the close is canceled (`ViewportCommand::CancelClose`) and, like Quit, asking one document at a time begins (if not already asking). Without unsaved changes it closes directly.
- The overlay stage shows the "Save changes?" confirmation (if the document for `save_prompt` no longer exists, it is skipped and the queue continues), and the answer is handed to `actions::answer_save_prompt`.
- When `quit_approved` is true, `ViewportCommand::Close` is sent and the app exits.

## Input injection

Before egui processes each frame's input, `raw_input_hook` appends the events in `AppState::forward_events` to the raw input and clears it. This is used when the menu's Cut/Copy/Paste is forwarded to an input field (see "Clipboard" in `commands.md`).

## Document tabs

Documents appear on the tab bar in the order of `AppState::doc_order`; the tabs' styling and interaction are described in `doc_tabs.md`.

## Positioning the History popup panel

Follows how Photoshop pops out a collapsed panel: the panel's right edge is flush with the icon column's left edge, its top is 13 pt above the History button, it is frontmost, and it has a shadow.

- Clicking anywhere other than the panel and the History button collapses the panel (corresponding to Photoshop's "Auto-Collapse Iconic Panels", on by default).
- The panel's ">>" button also collapses the panel.

## Wiring the Canvas Size dialog

When the dialog returns OK, if the new size differs from the current size, `resize_canvas` is applied to the current document and a "Canvas Size" history entry is recorded; if the size is the same, nothing happens (no history entry).

## Wiring the Fill dialog

- When the Fill dialog requests a color (Contents set to Color...), the Color Picker opens with the title "Color Picker (Fill Color)", and on OK the color is written back to the dialog.
- On OK, the fill is applied to the current document and "Fill" is recorded; on failure an alert is shown.

## Wiring adjustment and filter dialogs (preview)

- On open (`commands.md`), a snapshot of the document is saved to the dialog's `before`.
- Each frame: when the dialog's values are valid and Preview is checked, if the target adjustment differs from the current preview (`previewing`), the snapshot is restored first and then the new adjustment is applied to the document; when unchecked or the values are invalid, the snapshot is restored. Previews do not record history.
- Cancel / Esc: restores the snapshot if there is a preview.
- OK / Enter: first restores the snapshot, then applies for real (`Effect::apply`; the background color a filter needs is the current background color) and records the adjustment or filter name (e.g. "Levels", "Gaussian Blur"); when a filter is applied, it is recorded as `last_filter`. A filter dialog's settings (`settings()`) are stored in `filter_settings` by dialog kind and restored the next time it opens; an adjustment dialog's (`remembered()`) in `last_adjustments`. On failure an alert is shown.
- The preview pane of classic filter dialogs (`wants_pane()`): when there is no texture, `pane_texture` generates one from the central part of the document composite, at most 392 × 392 pixels, with nearest-neighbor sampling; each time the preview changes, the old texture is discarded and regenerated, so the preview pane shows the document with the preview effect.
- If the current document disappears while the dialog is open, the dialog is simply discarded.

## Crop preset dialogs

`crop_preset_dialogs` shows New Crop Preset... (`dialogs/new_preset.md`; OK adds the options' fields as a preset under that name and chooses it) and Delete Crop Preset... (`dialogs/size_presets.md`, centered; a confirmed delete removes the preset and keeps the chosen one pointing at the same preset). The Color Picker's `CropShield` target sets the crop shield's custom color.

## Adjustment dialogs: eyedroppers, the hand and the Color Picker

While an adjustment dialog samples (`AdjustDialog::sampling`/`targeting`, see `dialogs/adjust.md`), a primary press on the document view outside the dialog removes the preview, reads the merged pixel under it (`composite_rgba8`) and hands it to the dialog; the targeted adjustment hand's drag is followed until the button is released. Photo Filter's Color Picker request opens `PickerTarget::PhotoFilter`; the dialog is `blocked` while any Color Picker is open.

## Displace's map

When Displace's OK comes, the open panel asks for the map ("Choose a displacement map.", Photoshop documents or images); the chosen file is opened with `op_io::open`, its merged pixels registered (`more_filters::register_map`) and the filter applied with that map. Cancelling the panel cancels the filter.

## Wiring the Image Size dialog

Every frame the dialog's window size is copied to `AppState::image_size_extra`, so it reopens at that size. On OK: when Resample is on and the pixel size changed, resamples with the chosen method and Reduce Noise (`image_ops::resize_reducing_noise`); when the resolution changed, updates the document resolution; if anything changed, records "Image Size".

## Wiring the Modify dialogs

On OK, applies the corresponding operation to the current selection (`ModifyKind::apply`), replaces the selection and records the operation name. Does nothing when there is no selection.

## Wiring the New dialog

On OK, calls `actions::create_document`.

## Wiring the New Guide dialog

On OK, adds the guide to the current document and records "New Guide".

## Wiring the Fade dialog

`fade_dialog` keeps the layer previewing while the dialog is open: whenever the opacity, mode or Preview changes it sets the layer's pixels to `op_core::fade::fade` of the edit's before and after (or to the after with Preview off) and marks the document dirty; Cancel restores the after; OK sets the faded pixels and records "Fade <edit>". See `dialogs/fade.md`.

## Wiring the Trim dialog

On OK, applies `image_ops::trim` to the current document and records "Trim" if the canvas changed; if there is no edge to trim, nothing is recorded.

## Wiring the Color Picker

- When Canvas Size requests a color (choosing "Other..." or clicking the swatch), the Color Picker opens with the title "Color Picker" and the current extension color as the initial color.
- While the Color Picker is open, Canvas Size does not respond to Enter/Esc.
- Each frame, the Color Picker's "Add to Swatches" results are appended to `swatches`.
- On OK, writes back according to the target: foreground color, background color, or Canvas Size's extension color.

## Error alerts

When `AppState::alert` has content, it is shown in a macOS-style alert box (`dialogs/alert.md`: app icon, bold message, full-width OK), closed by Enter, Esc or OK. Failures opening files, failures exporting and the various "Could not ..." messages all use it. `AppState::flatten_prompt` is the Flatten Image question box, also shown with `dialogs::alert`.
