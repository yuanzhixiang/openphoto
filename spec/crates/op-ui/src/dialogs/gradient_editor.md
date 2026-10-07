# dialogs/gradient_editor.rs: Gradient Editor

## Component Responsibilities

Photoshop's Gradient Editor: picks a preset, names the gradient, sets Smoothness, and edits the color and opacity stops and their midpoints. It edits an `op_core::gradient::Gradient` (see `gradient.md` in `op-core`) and hands it back on OK. It opens from two places:

- The Gradient tool's options bar: clicking the gradient swatch (`options_tools.md`). OK makes the gradient the tool's gradient (`AppState::gradient_preset`).
- Gradient Map's gradient bar (`gradient_map.md`). OK replaces the dialog's gradient.

`AppState::gradient_editor` holds the open editor and its `EditorTarget` (`Tool` / `GradientMap`). `lib.rs` shows it above everything else, Gradient Map included. While it is open, the adjustment dialog underneath is blocked and the document is modal.

## Layout

The dialog is 516 × 470 pt and centered (an `egui::Modal` with no backdrop). Coordinates are in pt from its top-left corner. The arrangement follows Photoshop's, but it has **not been measured against Photoshop 2026**, so every position below is provisional.

- **Title:** "Gradient Editor".
- **Presets:** the label sits at (20, 48). Swatches are 40 × 40 in 8 columns, 7 apart, starting at (20, 62). The swatch matching the current stops, and the hovered one, get a white edge. Each swatch's tooltip is the preset's name. Clicking a swatch loads that preset (its name, Smoothness and stops) and selects the first color stop.
- **Name:**
  - The "Name:" label is right-aligned at x 65 and the field spans (70, 190)–(330, 209). Typing renames the gradient.
  - "New" (340, 186.5)–(400, 212.5) adds the current gradient to the presets, named "Custom" if the name is blank. These presets last for the session (`AppState::gradient_made`) and appear in the options bar's preset menu too.
- **Gradient Type:** a pop-up (110, 225)–(200, 246) that only offers "Solid".
- **Smoothness:** a field (300, 226)–(345, 245) taking 0–100, followed by "%".
- **Bar:** spans (30, 286)–(486, 314) and shows the gradient over a checkerboard.
  - Opacity stops are house shapes above the bar, pointing down, filled with gray (black = opaque). Color stops sit below, pointing up, filled with their color. The selected stop has a light edge.
  - The selected stop's row shows its midpoints as small diamonds 4 pt off the bar.
- **Stops group:** titled "Stops" at (35, 350.5).
  - Selecting an opacity stop enables "Opacity:" (field x 90–140, y 372, with "%") and "Location:" (field x 270–320, with "%").
  - Selecting a color stop enables "Color:" (a swatch x 90–140, y 414) and "Location:".
  - The other row's labels are dimmed.
  - "Delete" spans x 400–480.
- **Buttons:**
  - OK (420, 38.5)–(500, 64.5), which is the default.
  - Cancel (420, 73.5)–(500, 99.5).
  - Import... (420, 115)–(500, 141).
  - Export... (420, 150)–(500, 176).

## Interactions

- **Click above or below the bar:** zones extend 8 pt past each end and run 18 pt deep.
  - A stop within 6 pt is selected.
  - Otherwise, on the selected stop's row, a midpoint within 4 pt is grabbed.
  - Otherwise a new stop is added there, taking the gradient's own color or opacity at that point (`add_stop`), and it is selected.
- **Drag:**
  - Dragging a stop moves it, keeping the stops in order. The selection follows it, and the Location field shows the new position.
  - Dragging a stop more than 30 pt away from its zone deletes it, but two stops of each kind always remain.
  - Dragging a midpoint sets it between its two stops, clamped to 5–95%.
- **Double-clicking a color stop, or clicking the Color swatch,** opens the Color Picker titled "Color Picker (Stop Color)" (`PickerTarget::GradientStop`). While it is open the editor ignores Escape and Enter. The picker's OK sets the stop's color.
- **Fields:** Opacity and Location accept 0–100 and apply as you type.
- **Delete** removes the selected stop when more than two of its kind remain. The previous stop becomes selected.
- **Enter** (when no field is being typed in) is OK; **Escape** is Cancel. OK returns the gradient; Cancel changes nothing.
- **Import... / Export...** read and write a `.opgrd` text file (`serialize` / `parse`) holding:
  - the name on the first line;
  - `method smoothness` on the second;
  - then `c location r g b midpoint` for each color stop and `o location opacity midpoint` for each opacity stop.

  A file without at least two stops of each kind is rejected (`parse` returns None).

## Presets (`presets(fg, bg)`)

The presets are, in order:

1. Foreground to Background
2. Foreground to Transparent (foreground at both ends, with the right end's opacity at 0%)
3. Black, White
4. Blue, Red
5. Violet, Orange
6. Yellow, Violet, Orange, Blue
7. Copper
8. Chrome
9. Spectrum (seven stops)

The two Foreground presets follow the colors given. The three-stop presets put their middle stop at 50%. The colors approximate Photoshop's Basics and Legacy gradients and have not been measured.

## Known Limitations

- The layout and preset colors are provisional (not measured).
- Noise gradients are not supported, and neither are Photoshop's preset folders, the preset gear menu, or renaming and deleting presets.
- Import and Export use OpenPhoto's own text format, not Photoshop's `.grd`.
- Opacity stops have no Color-Picker-style slider.

## Test Coverage

- `stops_are_added_moved_and_deleted`: adding, moving (order kept) and deleting stops, with two of each kind kept.
- `export_and_import`: round trip through the text format.
- `ui_tests::gradient_editor_for_the_tool_and_gradient_map`:
  - from the options bar, a click below the bar adds a stop;
  - the swatch opens the Color Picker, which makes the stop red;
  - OK makes the tool paint black, red, white;
  - Gradient Map's bar opens the editor for Gradient Map, and Escape cancels it;
  - screenshot `gradient_editor.png`.
