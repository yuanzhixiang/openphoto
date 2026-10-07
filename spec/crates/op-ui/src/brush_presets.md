# brush_presets.rs: Brush presets and the brush picker popup

## Responsibilities

This module holds:

- Photoshop 2026's **General Brushes** presets.
- The **brush picker popup** that the options bar's brush picker opens (`options_tools.md`, `picker`). It edits the current tool's `PaintOptions` (`state.md`).

## Presets (`GENERAL`, `apply`)

There are eight presets, all 30 px. Each sets a size, a hardness, and what the pen's pressure controls (`op_core::paint::Pressure`):

| Preset | Hardness | Pressure controls |
| --- | --- | --- |
| Soft Round | 0% | none |
| Hard Round | 100% | none |
| Soft Round Pressure Size | 0% | size |
| Hard Round Pressure Size | 100% | size |
| Soft Round Pressure Opacity | 0% | opacity |
| Hard Round Pressure Opacity | 100% | opacity |
| Soft Round Pressure Opacity and Flow | 0% | opacity and flow |
| Hard Round Pressure Opacity and Flow | 100% | opacity and flow |

`apply(options, k)`:

- sets the preset's size, hardness and pressure dynamics;
- resets the tip to round: angle 0°, roundness 100%, spacing 25%;
- records the pick in `PaintOptions::preset`.

## The popup (`picker_ui`)

The popup is an egui popup that closes on a click outside. From top to bottom:

- **Size:** 1–5000 px on a logarithmic slider.
- **Hardness:** 0–100%. The Pencil has no Hardness.
- **The tip's dial** (`tip_dial`):
  - A 64 pt square holding a circle and the tip drawn as an ellipse, turned by the angle and squashed by the roundness, with a line along its long axis.
  - Dragging near the short axis inside the circle sets the roundness. Dragging anywhere else turns the angle toward the pointer.
  - Beside the dial, Angle (−180–180°) and Roundness (1–100%) fields do the same.
- **"General Brushes":** one row per preset, with a thumbnail (a dot showing the preset's hardness) and its name. The current preset is highlighted. Clicking a row applies it.

Changing size, hardness, angle or roundness by hand keeps the preset's dynamics, as Photoshop does until another preset is chosen.

For tools without `PaintOptions` (Selection Brush, Adjustment Brush, Art History Brush, Mixer Brush), only the size is kept, in their settings.

## Known limitations

- The popup is drawn with egui's widgets and is not a rebuild of Photoshop's brush preset picker. It has no search field and no preset folders other than General Brushes. Photoshop's Dry Media, Wet Media and Special Effects brushes are sampled tips that aren't included.
- Spacing has no control yet; Photoshop sets it in the Brush Settings panel.
- New Brush Preset and preset management are not implemented.

## Test coverage

- `presets_set_size_hardness_and_dynamics`.
- `ui_tests::brush_presets_pressure_and_smoothing`:
  - picking Hard Round Pressure Size from the picker;
  - a pen at 20% pressure paints a thin line;
  - with a mouse the stroke is full size;
  - full Smoothing still ends the stroke at the release point.
