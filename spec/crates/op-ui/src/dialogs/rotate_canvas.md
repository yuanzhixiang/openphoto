# dialogs/rotate_canvas.rs: Rotate Canvas dialog

## Responsibilities

The dialog for Image › Image Rotation › Arbitrary...: enter an angle and choose a direction; on confirm, `lib.rs` calls `op_core::image_ops::rotate_arbitrary` (with the current background color as the background) and records a "Rotate Canvas" history entry (the name measured in Photoshop 2026).

## Layout (measured in Photoshop 2026, 350 × 128 pt, relative to the top-left corner)

This is a UXP dialog: text uses `theme::uxp` 12 pt (Source Sans 3 as a substitute for Adobe Clean), `#f0f0f0`; the title "Rotate Canvas" is the window title (13 pt bold system font).

- "Angle" at x 20, center y 60; input field (55, 47)–(126, 73), text indented 12 from the left, with "°" directly after the number. On open the value is 0, selected, and the field has focus.
- Radio buttons (12 pt, center x 142.75): Clockwise at center y 59.75 (selected by default), Counter Clockwise at center y 83.75; text at x 158.5. Selected is a light `#d4d4d4` disc with a 4 pt dark dot in the middle; unselected is a 1 pt `#a0a0a0` ring. Clicking the circle or the text selects it.
- Buttons: OK (260, 48)–(330, 72) is the default button, Cancel (260, 84)–(330, 108), both in the bold style of `common::ps_button`.

## Interaction

- The angle may include "°", and its absolute value must be less than 360; OK is grayed out when it cannot be parsed or is out of range. `degrees()` returns a signed angle: clockwise is positive, counterclockwise negative.
- Enter equals OK, Esc equals Cancel; the dialog is modal while open.

## Known limitations

- When the input field has focus it gets a blue ring 2 pt outside the box (`common::text_field`); Photoshop instead turns the box border blue.

## Tests

- `angle_and_direction`: 0, 30°, counterclockwise is negative, 360 and non-numbers are invalid.
- `ui_tests::rotate_canvas_dialog`: entering 90, choosing Counter Clockwise and pressing Enter turns a 734 × 811 document into 811 × 734 and records "Rotate Canvas".
- `ui_tests::screenshot_rotate_canvas_dialog` (ignored): for comparison with Photoshop screenshots.
