# dialogs/modify_selection.rs: Select › Modify dialogs

## Component responsibilities

The single-value dialog shared by Select › Modify's Border..., Smooth..., Expand..., Contract... and Feather... (⇧F6). On OK, `lib.rs` modifies the current selection and records "Border", "Smooth", "Expand", "Contract" or "Feather". For the selection operations see `op-core`'s `selection.md`.

## Layout and visuals

Measured point by point from Photoshop 2026's five dialogs (UXP style, 2x screenshots); all dialogs are 295 × 128 pt, with coordinates in pt from the dialog's top-left:

- Title bar: `common::frame`, system bold 13 pt, with the titles "Border Selection", "Smooth Selection", "Expand Selection", "Contract Selection" and "Feather Selection". Text is the 12 pt panel font (`uxp::label`).
- One row: the label (Photoshop 2026 uses no colon: "Width", "Sample Radius", "Expand By", "Contract By", "Feather Radius") with its left end at 20.5, centered at y 60; the input field follows the label, 9 pt to its right, 36 × 24 (y 48–72, text indented 11.5 pt from the left, focused with everything selected on open, default 1); 5.5 pt after it is "pixels".
- All five dialogs have the "Apply effect at canvas bounds" checkbox (20, 90), unchecked by default (matching Photoshop 2026).
- OK (default button) (205–275, y 48–72), Cancel (y 84–108): `common::ps_button`. Enter confirms, Esc cancels. Modal while open.

Labels use Source Sans 3 to approximate Adobe Clean and are slightly wider ("Feather Radius" is 2.5 pt wider), so the input field shifts right accordingly; the other elements differ from the Photoshop screenshots by no more than 1 pt.

## Value ranges

Border 1–200; Smooth, Expand, Contract 1–500; Feather 0.1–1000. When out of range or not a number, OK is grayed out. Values for Smooth, Expand and Contract are rounded to integers.

## Known limitations

- "Apply effect at canvas bounds" only takes effect for Contract; Border, Expand, Smooth and Feather show the option but ignore it, and their handling at the canvas edges may differ from Photoshop.
- Each time the dialog opens, the default value 1 is restored; the last value is not remembered.

## Test coverage

- `ranges`: Border above 200 is invalid; Feather accepts 0.5.
- `modify_selection_and_grow` in `ui_tests.rs` (Expand 5 px); `screenshot_feather_dialog` (`#[ignore]`) captures the Feather dialog for comparison with Photoshop.
