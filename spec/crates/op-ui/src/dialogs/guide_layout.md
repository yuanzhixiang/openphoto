# dialogs/guide_layout.rs: New Guide Layout

## Component Responsibilities

The dialog for View › Guides › New Guide Layout.... It sets columns, rows and margins, which `op_core::guide_layout` turns into guides (`guide_layout.md` in `op-core`). The guides show on the canvas while the dialog is open.

## Data

- **On open:** Photoshop's defaults. Columns are on with Number 8, Width empty (shared evenly) and Gutter 20 px. Rows and Margin are off. Center Columns and Clear Existing Guides are off. Preview is on.
- **`layout()`** reads the fields in use:
  - Number takes whole numbers 1–1000.
  - Lengths take 0–30000, optionally followed by "px".
  - An empty Width or Height shares the space evenly. An empty Gutter or margin counts as 0.
  - If any field in use doesn't read, there is no layout and OK is disabled.

## Layout

The dialog is 420 × 330 pt, centered, and titled "New Guide Layout". The arrangement follows Photoshop's, but it has **not been measured against Photoshop 2026**, so the positions below are provisional (pt from the top-left corner).

- **"Preset:"** pop-up at (70, 46)–(250, 67), offering Custom, 8 Column, 12 Column and 4 Column.
  - Picking one of the column presets turns Columns on with that number, an empty width and a 20 px gutter.
  - The pop-up shows the preset matching the fields, or Custom.
- **"Target:"** pop-up at (70, 76)–(250, 97), offering only "Canvas".
- **Columns** checkbox at (20, 112), with Number, Width and Gutter fields at x 90–150, y 145, 172 and 199.
- **Rows** checkbox at (200, 112), with Number, Height and Gutter at x 270–330.
- **Margin** checkbox at (20, 222), with Top, Left, Bottom and Right fields in a row at y 255.
- Fields of a group that is off are drawn as empty frames.
- **Center Columns** checkbox at (20, 282).
- **Clear Existing Guides** checkbox at (20, 306).
- **Buttons:** OK (330, 38.5)–(400, 64.5), which is the default; Cancel (330, 73.5)–(400, 99.5). The **Preview** checkbox is at (330, 112).

## Interactions

`lib.rs` (`guide_layout_dialog`) keeps the document's guides from before the dialog (`AppState::guide_layout`).

- **Each frame:** with Preview on and a valid layout, the guides show as the result. That is the earlier guides (none with Clear Existing Guides) plus the layout's. Otherwise the earlier guides show.
- **OK** (or Enter while no field is being typed in) keeps the result and records "New Guide Layout".
- **Cancel** (or Escape) puts the earlier guides back.

## Known Limitations

- The layout is provisional (not measured).
- There are no artboard targets, no saved presets (Save Preset... / Delete Preset...) and no guide colors.

## Test Coverage

- `fields_make_the_layout`.
- `ui_tests::new_guide_layout_previews_and_applies`:
  - the defaults preview 16 guides on a 734 px canvas;
  - Escape removes them;
  - OK keeps them, records the step, and the first guide sits at 0;
  - screenshot `guide_layout.png`.
