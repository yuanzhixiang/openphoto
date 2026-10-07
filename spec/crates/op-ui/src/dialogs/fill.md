# dialogs/fill.rs: Fill dialog

## Component responsibilities

Edit › Fill... (⇧F5). Chooses the fill contents and blending; on OK, `lib.rs` calls `op_core::fill::fill` to fill the current layer's selection (the whole layer when there is no selection) and records a "Fill" history entry.

## Layout

Measured point by point from Photoshop 2026's Fill dialog (a classic AppKit dialog, 2x screenshots), the same kind as Levels and Curves, with controls from `appkit.rs`. The dialog is 348 × 293 pt; coordinates are pt from the dialog's top-left:

- Title bar: `common::frame`, system bold 13 pt "Fill". Text is 12 pt system font (`theme::dialog`, with macOS tracking), `#f0f0f0`.
- "Contents:" right-aligned at 112.5, centered at y 48.5; dropdown (118–263, y 38–59).
- **Options** group box (11.5–262.5, y 116–166): 1 pt `#424242` line, the title "Options" starting at 30.5 on top of the upper line, with the line leaving 5 pt on each side of the title. Inside is the disabled "Color Adaptation" checkbox (20, 141.5): it only matters for Content-Aware, and Content-Aware is not implemented yet, so it is always grayed out.
- **Blending** group box (11.5–262.5, y 184–282): "Mode:" (centered at y 208.5) and dropdown (118–253, y 198–219, all 27 blend modes, grouped as in Photoshop's menu); "Opacity:" (236.5) and input field (117–169, y 227–246, focused with everything selected on open, ↑↓ step 1, ⇧ step 10) plus "%" (177.5); "Preserve Transparency" checkbox (20, 257.5).
- Checkboxes are 12 pt, with text 11 pt to the right of the box; when disabled, a `#4d4d4d` box, `#5d5d5d` border and `#8e8e8e` text.
- OK (default button, bright border) (278.5–338.5, y 38.5–64.5), Cancel (y 73.5–99.5): 26 pt high capsule buttons.

Compared with Photoshop screenshots, each element differs by no more than 1 pt.

## Contents options

Foreground Color (default), Background Color, Color...; Content-Aware, Pattern, History (grayed out); Black, 50% Gray (128), White.

Choosing "Color..." opens the Color Picker titled "Color Picker (Fill Color)"; after OK there, Contents becomes Color... and uses the chosen color; Cancel keeps the previous choice.

## Keyboard

- Enter: OK (when the opacity is valid). Esc: Cancel.
- While the Color Picker is open, Enter/Esc act only on the Color Picker.

## Validation

The opacity must be a number from 0–100, otherwise OK is grayed out.

## Known limitations

- Content-Aware, Pattern and History are not implemented yet, so Color Adaptation is always grayed out; there is no pattern picker for Pattern and no script options.
- The last settings are not remembered.
