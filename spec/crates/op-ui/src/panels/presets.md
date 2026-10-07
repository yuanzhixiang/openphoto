# panels/presets.rs: Gradients and Patterns panels

## Responsibilities

The Gradients and Patterns tabs of the first panel group (`mod.md`). Each shows its presets as swatches. A click makes one the Gradient tool's gradient or the Pattern Stamp's pattern.

## Layout

- Swatches are 36 pt squares with 6 pt gaps, in as many columns as fit, starting 10 pt from the panel's top-left corner.
- A swatch's edge is a 1 pt line: white when it is the current one or hovered, the divider color otherwise.
- Each swatch's tooltip is its name.

## Gradients (`gradients`)

- **Swatches:** the Gradient Editor's presets for the current colors and the gradients made with its New (`AppState::gradient_presets`). Each is drawn over a checkerboard where it is see-through (`gradient_editor::paint_gradient`).
- **Current:** the swatch whose stops match the tool's gradient (`tool_gradient`).
- **Click:** the gradient becomes the Gradient tool's (`gradient_preset`).

## Patterns (`patterns`)

- **Swatches:** `AppState::patterns`, each tiled a pixel a point.
- **Current:** the Pattern Stamp's (`AppState::pattern`).
- **Click:** the pattern becomes the Pattern Stamp's.

## Known limitations

- Photoshop's preset folders (Basics, Blues, ... and the Trees, Grass, Water pattern sets) and the panel menus (New Gradient/Pattern, view as list, import) aren't there.
- Photoshop 2026 applies a clicked gradient or pattern to the selected layer as a fill layer. Fill layers are in P2 #17, so here the click sets the tool's choice instead.
- Swatches past the panel's bottom are cut off (no scrolling).

## Test coverage

- `ui_tests::gradients_and_patterns_panels_pick`:
  - the second gradient swatch makes Foreground to Transparent the tool's;
  - on the Patterns tab, a second pattern and then the first are picked;
  - screenshots `gradients_panel.png` and `patterns_panel.png`.
