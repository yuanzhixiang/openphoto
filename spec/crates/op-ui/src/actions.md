# actions.rs: File operations and single-key shortcuts

## Responsibility

Opening, creating, saving, reverting, exporting, and closing documents (including confirmation for unsaved changes) and quitting, running clipboard commands, and single-key shortcuts (tool switching, default colors, swapping colors). Shortcuts with modifier keys are commands; see `commands.md`.

## Open

- After a successful open, the document's file path is recorded (`DocState::path`).

- `open_dialog`: the system file dialog, multi-select, filtered to the formats listed in `op_io::OPEN_EXTENSIONS` (PNG, JPEG, WebP, TIFF, BMP, GIF).
- `open_paths`: opens each in turn; each success is added as a new document and becomes the current document, with "Open" as the first history entry. On failure, a log entry is written and `alert` shows "Could not open “path”: reason".
- When an ordinary bitmap is opened, the whole image is one layer, matching Photoshop: a fully opaque image is a locked "Background" layer, and an image with transparency is a normal "Layer 0" layer.

## New

- `new_dialog`: File › New... (⌘N) opens the New Document dialog (`dialogs/new_document.md`), with the next "Untitled-N" as the name; when the clipboard holds an image, the width and height are taken from the image's size (matching Photoshop), otherwise from the first recently used preset, and when there is neither, 1920×1080 pixels at 72 ppi. It also passes in this run's Recent and Saved presets, whether the welcome box has been closed, and the current background color (the Background Color swatch).
- `create_document(name, size, resolution, contents)`: creates the document from the dialog and makes it the current document, with "New" as the first history entry, and increments the untitled counter. Background contents of White, Black, or Background Color (the current background color) produce a background layer of the corresponding color; Transparent produces a transparent normal layer "Layer 1" (no background layer), matching Photoshop.
- `new_document`: the document used when the app starts with no files opened: 1920×1080, white background, 72 ppi, RGB/8, titled "Untitled-1", without going through the dialog.

## Export

`export_dialog` (File › Export › Export As...): the system save dialog; the default file name is the document title without its extension plus `.png`, and PNG or JPEG can be chosen. What is exported is the composite of all visible layers; PNG keeps transparency, and JPEG fills transparent areas with white (details in `crates/op-io/src/lib.md`). On failure, `alert` shows "Could not export: reason", and no incomplete file is left behind.

## Close

- Close: closes the current document; Close All: closes all documents; Close Others: closes all documents except the current one; the "×" on a tab closes that document.
- All closes go through `request_close(ids)`: the documents to close are put into `close_queue`, and `continue_closing` processes them in order: documents without unsaved changes are closed directly (`AppState::close_document`, which also removes the tab and document state); a document with unsaved changes is made the current document, a "Save changes?" confirmation is shown (`save_prompt`, see `dialogs/save_changes.md`), and processing pauses.
- `answer_save_prompt`: Save → runs Save (which may show the Save As dialog), and on success closes the document and continues processing the queue; Don't Save → closes directly and continues; Cancel, or the Save As dialog being canceled, or the save failing → clears the queue and stops closing (also stops quitting).
- `quit`: Quit OpenPhoto and closing the window: puts all documents into the queue and sets `quit_after_close`; when the queue is fully processed, `quit_approved` is set, and `lib.rs` then closes the window and quits the app.

## Save

- `save(id)`: File › Save. When the document has a file and the file can hold it (PSD always can; flat formats such as PNG/JPEG only when there is a single layer), it writes straight back to that file and marks the document as saved; otherwise it goes to Save As. Returns whether the save succeeded.
- `save_as(id, copy)`: File › Save As... (`copy` false) and Save a Copy... (true). The system save dialog; the default file name is the title without its extension plus `.psd`, the default directory is the document's directory, and the format filter is `op_io::SAVE_FORMATS`. Returns false when canceled.
- `save_to(id, path, copy)`: writes to `path` (on failure shows "Could not save “path”: reason"). Afterwards: if it is a copy, or the chosen format cannot hold the document's layers (for example a multi-layer document saved as PNG, equivalent to Photoshop's "Save a Copy"), the document's file, title, and saved state are unchanged; otherwise the document switches to the new file, its title becomes the new file name, and it is marked as saved.
- `revert()`: File › Revert (F12). Re-reads the document's file and replaces the document with its contents (size, layers, selection, etc., restored via a snapshot), records a "Revert" history entry, and marks the document as saved. Shows an alert when reading fails.

## Clipboard (`clipboard`)

Runs Cut, Copy, Copy Merged, Paste, Paste in Place; when a text field has focus, these are handed to the text field. Full rules under "Clipboard" in `commands.md`.

## Single-key shortcuts (`handle_tool_keys`)

- Q: enters/exits Quick Mask (same as the toolbar button).

- Not handled while a modal dialog is open or a text field has focus.
- Only responds to press events without ⌘/Ctrl/Alt (Shift is allowed), ignoring key repeat.
- When the current tool is the Brush, Pencil, or Eraser, painting keys are handled first (matching Photoshop):
  - `[`, `]`: decrease or increase the brush size by the step from `PaintOptions::size_step`.
  - Shift+`[`, Shift+`]`: decrease or increase hardness by 25%.
  - Number keys 1–9, 0: set opacity to 10%–90%, 100%; Shift+number sets flow (the Pencil has no flow).
- When the current tool is the Move tool, arrow keys move the current layer (or the selected pixels) by 1 pixel and Shift+arrow by 10 pixels, recording one "Nudge" each time, matching Photoshop. When moving is not possible, an alert appears.
- D: restores the default colors (black foreground, white background); X: swaps the foreground and background colors.
- Other letters: select a tool via `op_tools::tool_for_key` (rules in `crates/op-tools/src/lib.md`) and make that toolbar slot show the selected tool. Shift+letter cycles through the tools in the same group, matching Photoshop.

## Duplicating a layer to another document (`duplicate_layer`)

Called after the Duplicate Layer dialog is confirmed; handled in three cases depending on the destination, see `dialogs/duplicate_layer.md`. When both documents need to be mutable at the same time, `HashMap::get_disjoint_mut` is used to get the source and destination documents.

## Known limitations

- Shortcuts for features not yet implemented, such as Q (Quick Mask), F (screen mode), and R (Rotate View), are not handled.
