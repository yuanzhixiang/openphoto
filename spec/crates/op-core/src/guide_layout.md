# guide_layout.rs: guide layouts

## Responsibilities

The geometry of View › Guides › New Guide Layout...: turns columns, rows and margins into guides. The dialog is in `op-ui` (`dialogs/guide_layout.md`).

## Public interface

- `GuideLayout` has these fields:
  - `columns: Option<(number, width, gutter)>`
  - `rows: Option<(number, height, gutter)>`
  - `margin: Option<[top, left, bottom, right]>`
  - `center_columns`

  Lengths are in pixels. A width or height of `None` shares the space evenly. `Default` is Photoshop's: 8 columns with 20 px gutters, no rows or margins.
- `guides(width, height)` returns the guides for a canvas of that size:
  - **Space:** the area left inside the margins (the whole canvas without them).
  - **Margins:** when on, a vertical guide at each side and a horizontal one at the top and the bottom of that space.
  - **Columns and rows:** n spans of the given or shared size with gutters between them. A guide goes at each span's start and end. Columns start at the space's left edge, or sit centered in it with Center Columns. Rows always start at the top.
  - **Rounding:** positions are rounded to 1/32 px, as Photoshop stores them, and a position already used in the same direction isn't repeated (so spans with no gutter share their edge).
  - **Order:** vertical guides come first, then horizontal ones.

## Test coverage

- `columns_rows_and_margins`: 4 columns with 10 px gutters across 430 px are 100 px each; 2 columns and 2 rows without gutters inside margins share their middle edges; two 50 px columns with a 10 px gutter centered in 210 px start at 50.
