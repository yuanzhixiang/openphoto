# gradient.rs: Gradient tool (classic gradient)

## Responsibilities

Draws a two-color gradient on the active layer, limited to the selection. Corresponds to the "classic gradient" behavior of Photoshop's Gradient tool (changing pixels directly rather than creating a gradient fill layer). Records no history.

## Public interface

- `GradientKind`: `Linear`, `Radial`, `Angle`, `Reflected`, `Diamond` (`ALL` is the options bar order, `label()` gives "Linear Gradient" and so on).
- `GradientKind::position(a, b, x, y)`: the position t of point (x, y) in a gradient dragged from `a` to `b` (0 is the start color, 1 is the end color). Let the drag vector be d with length L:
  - Linear: the ratio of the point's projection onto the d direction, clamped to 0–1.
  - Radial: distance to the start point / L, at most 1.
  - Reflected: the absolute value of the projection ratio, at most 1 (symmetric on both sides of the start point).
  - Diamond: `(|u| + |v|) / L` in the coordinate system with d as its axis, at most 1.
  - Angle: one full turn around the start point beginning in the d direction; t is the angle turned / 360° (increasing counterclockwise on screen).
  - 0 when the start and end points coincide.
- `GradientOptions`: `kind`, `mode` (blend mode), `opacity` (0–1), `reverse` (swaps the start and end colors). Defaults: Linear, Normal, 100%, not reversed.
- `gradient(doc, a, b, colors, options)`: first performs the same checks as adjustments (`adjust::check`), then for each pixel takes t at the pixel center, linearly interpolates the two colors in RGB, and composites onto the layer using the blend mode and opacity × degree of selection. The background layer and layers with locked transparent pixels keep their original alpha (fully transparent pixels are not painted).

## Masks

In Quick Mask mode the layer state is not checked. When the editing target is a mask (Quick Mask or layer mask), the gradient runs between the gray levels of the two colors and is written to the mask.

## Known limitations

- Only two-color (foreground to background) gradients; there is no gradient editor, presets, opacity stops, Dither, or interpolation method option.
- Does not create a gradient fill layer (since Photoshop 2024, the Gradient tool creates an editable gradient layer by default).

## Test coverage

- `positions_of_each_kind`: t values of each kind at typical points.
- `paints_black_to_white`: 5 pixels black to white gives `[0, 64, 128, 191, 255]`, reversed when reversed.
- `half_opacity_and_selection`: 50% opacity affects only pixels inside the selection.

## Gradient interpolation methods (`Method`, `blend_colors`)

Measured from Photoshop 2026's Gradient Map (red→blue, black→white, compared level by level for each method):

- The position between two stops first gets "classic easing": the average of `t` and smoothstep `3t² − 2t³` (i.e. Photoshop gradient Smoothness 100%).
- `Classic`: interpolates in sRGB values at the eased position (identical to Photoshop level by level).
- `Linear`: interpolates in linear light (differs by no more than 2 levels).
- `Perceptual`: interpolates in OKLab (differs by no more than 2 levels).
- `Smooth` (dialog default): interpolates in OKLab, with easing taking only 40% of classic easing (the measurement is coarser; differs by no more than 6 levels).
- The Gradient tool itself still interpolates linearly in RGB and does not yet use these methods.
