# panels/panel_options.rs: Info and Navigator Panel Options

## Responsibilities

The options the Info and Navigator panels' panel menus open with Panel Options... (`floating.md`). This module also holds the Info panel's color readouts.

## Info

- **`Readout`:** Actual Color, RGB Color, Web Color, HSB Color, Grayscale, CMYK Color, Lab Color, Opacity (given the active layer's color, it reads its alpha as a percentage). `lines(color)` gives the labels and values the Info panel shows (`info.md`), or the labels with empty values without a color.
- **`InfoOptions`** (`AppState::info_options`):
  - the first and second readout (Photoshop's defaults: Actual Color and CMYK Color);
  - the mouse coordinates' unit (None: the rulers');
  - three status lines, Document Sizes (on by default), Document Dimensions and Current Tool (`STATUS`);
  - Show Tool Hints (on by default, as in Photoshop): the Info panel's tool hint section (`info.md`).
- **`info_options_ui`:** a pop-up for each readout, one for the unit ("Rulers' Units", then the rulers' units), checkboxes for the status lines, and Show Tool Hints.

## Navigator

The view box's color (`AppState::navigator_box`), picked from `VIEW_BOX_COLORS`: Light Red (the default), Cyan, Green, Light Blue, Yellow and Magenta.

## The dialog (`floating::panel_options`)

- `AppState::panel_options` holds the panel and the options being edited.
- It is an egui modal titled "Info Panel Options" or "Navigator Panel Options", with the controls above and OK and Cancel.
- OK or Enter keeps the edited options; Cancel or Escape drops them.
- It counts as a modal dialog (`modal_open`).

## Known limitations

- The dialog uses egui's widgets and isn't laid out like Photoshop's.
- Of the status items, only the three above are offered.
- Show Tool Hints only has text for the tools whose hints were measured from Photoshop (`info::tool_hint`).

## Test coverage

- `readouts`.
- `ui_tests::info_and_navigator_panel_options` (see `info.md`).
