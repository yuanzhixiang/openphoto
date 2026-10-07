# dialogs/duplicate_layer.rs: Duplicate Layer dialog

## Component responsibilities

The dialog opened by Layer › Duplicate Layer...: names the copy of the active layer and chooses which document to put it in (the current document, another open document, or a new document), corresponding to Photoshop 2026's Duplicate Layer dialog.

## Layout (measured in Photoshop 2026, pt relative to the dialog's top-left)

The dialog is 447 × 208, displayed centered; the title "Duplicate Layer" is 13 pt bold (see `theme.md`). Text is AppKit's system font 12 pt (`theme::dialog`, opsz 17), labels `#d7d7d7`, values `#f1f1f1`, disabled `#888888`; labels are all right-aligned to x 81.5 (the position and width of each piece of text differ from Photoshop by no more than 1 px).

- "Duplicate:" centered at y 45.5, with the active layer's name to its right at x 90.5.
- "As:" centered at y 70.5; input field (86, 61)–(361, 80), focused with everything selected on open, defaulting to "original name copy" (`layer_ops::duplicate_name`).
- "Destination" group box (10.5, 96.5)–(361, 197.5): 1 pt `#424242` frame line, with the title breaking the frame line at x 31.
  - "Document:" centered at y 121.5; dropdown (86, 111)–(351.5, 132), `#454545` fill, 1 pt `#666666` border, value inset 8.5 from the left, a V-shaped arrow 7.5 from the right; a value that is too long is truncated with "..." (Photoshop truncates one or two characters later than here). The menu lists the current document, the other open documents and "New".
  - "Artboard:" and dropdown (86, 140)–(351.5, 161): always disabled, showing "Canvas" (this app has no artboards).
  - "Name:" and input field (86, 169)–(351.5, 188): enabled only when "New" is chosen, defaulting to the next "Untitled-N"; otherwise a disabled empty field.
- Buttons: OK (377.5, 39)–(437, 64) is the default button (bright border), Cancel (377.5, 74)–(437, 99); labels are AppKit system font 12 pt (unlike the New Layer dialog's bold Adobe Clean; this is a difference between Photoshop's two kinds of dialogs).

## Interaction

- Enter or OK: duplicates according to the choice (`actions::duplicate_layer`):
  - Current document: `layer_ops::duplicate_named`, records "Duplicate Layer".
  - Another open document: `layer_ops::duplicate_into`; the copy goes above the target document's active layer with its pixel position unchanged, and the target document records "Duplicate Layer"; the current document is unchanged and remains the active document.
  - New: `layer_ops::duplicate_to_new` creates a new document and opens it as the active document, with the initial history state "Duplicate Layer"; the Untitled counter increments by one.
  - When "As" is empty, the original name is used.
- Esc or Cancel: closes without doing anything.

## Known limitations

- There are no artboards, so Artboard is always disabled.
- When the "As" input field has focus it shows a 2 pt blue outer ring; Photoshop shows a 1 pt blue border here.

## Test coverage

- `destinations` (unit test): the default destination is the current document; after choosing "New" the destination is the new document's title.
- `duplicate_layer_and_layer_from_background_dialogs` (UI test): typing "Copy A" and pressing Enter duplicates into the current document; opening the dialog again, choosing "New" from the Document menu and clicking OK opens a new document containing only "Copy A copy"; back in the first document, Layer from Background... with "Base" entered and confirmed turns the background into the normal layer "Base" and records "Layer From Background".
- `screenshot_duplicate_layer_dialog` (screenshot, `#[ignore]`): for comparison with Photoshop screenshots.
