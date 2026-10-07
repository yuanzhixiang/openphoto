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
- `gradient(doc, a, b, colors, options)`: a two-color gradient interpolated linearly in RGB (Classic, Smoothness 0%), painted through `gradient_with`.
- `gradient_with(doc, a, b, gradient, options, dither)`: first performs the same checks as adjustments (`adjust::check`), then for each pixel takes t at the pixel center, reads the gradient's color and opacity there (from a 1024-entry table of `Gradient::sample`, interpolated between entries; Reverse uses `Gradient::reversed`), and composites onto the layer using the blend mode and opacity × the gradient's opacity × degree of selection. With `dither`, up to ±half a level of repeatable per-pixel noise (`adjust::dither_noise`) is added to the color before compositing. The background layer and layers with locked transparent pixels keep their original alpha (fully transparent pixels are not painted).

## Gradients with stops (`Gradient`)

- `Gradient { name, colors: Vec<ColorStop>, opacities: Vec<OpacityStop>, method, smoothness }`, like Photoshop's Solid gradient: color stops `{ location, color, midpoint }` and opacity stops `{ location, opacity, midpoint }`; locations are 0–1 and kept in order; a stop's midpoint (0–1, used clamped to 5–95%) is where between it and the next stop the blend is halfway.
- `Gradient::two(name, a, b)`: two color stops at the ends, both opacity stops 100%, midpoints 50%, the default Method (Smooth), Smoothness 100%.
- `sample(t) -> (rgb, opacity)`: finds the two stops around t; the midpoint remaps the position between them piecewise-linearly (the midpoint becomes 0.5); colors blend with `blend_colors` under the gradient's Method, mixed with a straight RGB blend by Smoothness (100%: only the method's easing, 0%: straight); opacity blends linearly. Before the first stop and after the last the end stop's value holds.
- `reversed()`: locations mirrored, stop order and midpoints flipped so the reversed gradient looks the mirror image.
- `table()`: 256 colors (for Gradient Map's `Adjustment::GradientTable`).

## Masks

In Quick Mask mode the layer state is not checked. When the editing target is a mask (Quick Mask or layer mask), the gradient runs between the gray levels of the two colors and is written to the mask.

## Known limitations

- Only Solid gradients (no Noise gradients).
- Does not create a gradient fill layer (since Photoshop 2024, the Gradient tool creates an editable gradient layer by default).
- Midpoints remap the position piecewise-linearly; Photoshop's exact midpoint curve and its Smoothness below 100% have not been measured.

## Test coverage

- `positions_of_each_kind`: t values of each kind at typical points.
- `paints_black_to_white`: 5 pixels black to white gives `[0, 64, 128, 191, 255]`, reversed when reversed.
- `half_opacity_and_selection`: 50% opacity affects only pixels inside the selection.
- `stops_midpoints_opacity_and_reverse`: two-stop sampling, a midpoint at 25%, a third red stop, an opacity stop, reversal and the table.
- `painting_stops_with_opacity_and_dither`: a black gradient fading to transparent over white, and Dither keeping colors within a level.

## Gradient interpolation methods (`Method`, `blend_colors`)

Measured from Photoshop 2026's Gradient Map (red→blue, black→white, compared level by level for each method):

- The position between two stops first gets "classic easing": the average of `t` and smoothstep `3t² − 2t³` (i.e. Photoshop gradient Smoothness 100%).
- `Classic`: interpolates in sRGB values at the eased position (identical to Photoshop level by level).
- `Linear`: interpolates in linear light (differs by no more than 2 levels).
- `Perceptual`: interpolates in OKLab (differs by no more than 2 levels).
- `Smooth` (dialog default): interpolates in OKLab, with easing taking only 40% of classic easing (the measurement is coarser; differs by no more than 6 levels).
- The Gradient tool and Gradient Map use these methods through `Gradient::sample` (the Gradient tool with the options bar's Method).
