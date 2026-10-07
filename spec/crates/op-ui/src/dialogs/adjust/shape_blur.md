# dialogs/adjust/shape_blur.rs: the Shape Blur dialog

## Responsibilities

Filter › Blur › Shape Blur...'s dialog, laid out as Photoshop 2026's (359 × 485 pt, measured from a 2x capture). Part of `AdjustDialog` (`adjust.md`), with the legacy parts (`legacy.md`).

Its settings are Radius (`RADIUS`, 5–1000 pixels, 10) and Shape (`SHAPE`, one of `op_core::more_filters::BlurShape::ALL`). The effect is `Filter::ShapeBlur`.

## Layout

Positions in points from the dialog's top-left corner.

- **Preview:** 8–219 × 36–248, its zoom controls on y 261.
- **Top right:** OK and Cancel at x 267. The Preview checkbox's box is at (268, 118.5).
- **Shape:** "Shape:" at (268, 172) over a black swatch 268.5–330 × 196.5–231.5 showing the chosen shape in white.
- **Radius:** "Radius:" left-aligned at (19.5, 296.75); the field 65.5–125 × 288–305.5, which has the focus on open; "Pixels" at 129.5; the track 20.5–207 from y 312.5 with its pin under it.
- **The shapes:** a `#3f3f3f` list 10.5–316.5 × 344–473 with a 16 pt scroll gutter on the right. Each shape is a 29 pt row:
  - its white-on-black thumbnail, 18 pt square, at x 32.5;
  - its name at 54;
  - a `#464646` fill, `#4c4c4c` while hovered, `#5c5c5c` when chosen.

  A click chooses the shape. The thumbnails are made once (96 pixels, from `BlurShape::contains`) into `Extra::shape_thumbs`.

## Known limitations

- Photoshop lists its custom shape library (folders such as Wild Animals, Leaf Trees, Boats, Flowers) with a gear menu to manage it. OpenPhoto lists its five built-in shapes (Circle, Square, Star, Heart, Diamond) and has no gear: custom shapes come with the Custom Shape tool (P2 #14).

## Test coverage

- `ui_tests::shape_blur_dialog_picks_a_shape`: a click on the fourth row chooses Heart for the filter, and Enter applies "Shape Blur". Screenshots `shape_blur.png` and `shape_blur_heart.png`, compared with Photoshop's capture.
