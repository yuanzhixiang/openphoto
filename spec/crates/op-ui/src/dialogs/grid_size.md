# dialogs/grid_size.rs: Custom Grid Size

## Component Responsibilities

The dialog that Warp's Grid › Custom... opens (options bar, `options_bar.md`). It asks for the number of rows and columns of the warp grid. OK regrids the warp (`WarpMesh::regrid(columns, rows)`), keeping the current surface; Cancel changes nothing.

## Data and Layout

- It opens with the current grid's rows and columns.
- The dialog is 300 × 112 pt, centered, and titled "Custom Grid Size". The layout follows the AppKit-style dialogs; it has **not been measured against Photoshop 2026**, so the positions below are provisional (pt from the top-left corner):
  - "Rows:" is right-aligned at x 90, with its field at x 96–150, centered on y 47.5. The field takes the focus with its text selected.
  - "Columns:" sits the same way at y 81.5.
  - Both fields take whole numbers from 1 to 50; the arrow keys step by 1.
- OK (220, 38.5)–(288, 64.5) is the default button. It is disabled while a field is out of range. Cancel is at (220, 73.5)–(288, 99.5).

## Interactions

- Enter or OK accepts; Escape or Cancel closes the dialog.
- The dialog lives in `FreeTransform::grid_dialog`. While it is open, the transform ignores the canvas, so Enter doesn't commit the warp. `lib.rs` shows it (`warp_grid_dialog`).

## Known Limitations

- The layout and the 1–50 range are provisional (not measured).

## Test Coverage

- `rows_and_columns_in_range`.
- `ui_tests::warp_splits_grids_and_styles`: Custom... with 2 rows and 6 columns gives a 6 × 2 grid, and the transform stays open. Screenshot `warp_custom_grid.png`.
