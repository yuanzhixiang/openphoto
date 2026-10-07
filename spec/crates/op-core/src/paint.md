# paint.rs: Paint strokes

## Responsibilities

Stroke computation for the Brush, Pencil and Eraser: places dabs along a path and paints color onto the current layer, or erases pixels.

## Opacity and flow

Matches Photoshop: within one stroke, "flow" keeps accumulating but never exceeds "opacity".

- Each stroke keeps the layer's pixels from before it started (`base`; cheap to copy because tiles are shared), plus this stroke's coverage at each pixel (0–1, stored in tile chunks).
- Each dab increases coverage by `(1 − coverage) × dab alpha × flow`.
- Pixel result = starting from `base`, changed by `coverage × opacity × selection`. So scrubbing back and forth within one stroke reaches at most the opacity.

## Public interface

- `BrushTip`: diameter (pixels), hardness (0 soft – 1 hard), `aliased` (Pencil: no anti-aliasing), `square` (the Eraser's Block: a square with side length equal to the diameter, fully covered inside and 0 outside), and the Brush Tip Shape: `angle` (degrees, counterclockwise on screen), `roundness` (0.01–1: the tip's height over its width) and `spacing` (the distance between dabs as a fraction of the diameter; Photoshop's default 0.25). An elliptical tip measures each pixel's offset in the tip's own frame (turned by the angle, its short axis stretched by 1 / roundness) before the falloff below. Dab alpha: 1 from the center out to `radius × hardness`, then falls off smoothly to 0 at 0.5 pixels outside the radius; even at hardness 1 a 1-pixel anti-aliased edge is kept. For the Pencil, a pixel whose center is within the radius is 1, otherwise 0 (the diameter counts as at least 1 pixel).
- `StrokeKind`:
  - `Paint(rgb)`: paints color (Brush, Pencil). `Erase { background }`: Eraser.
  - `Dodge(range)`, `Burn(range)`: Dodge, Burn. `ToneRange` is Shadows / Midtones (default) / Highlights; `label()` is the menu text.
  - `Sponge { saturate }`: Sponge (saturate or desaturate).
  - `Blur`, `Sharpen`: Blur, Sharpen.
  - `Source { image, dx, dy }`: paints pixels from another image; target (x, y) takes (x − dx, y − dy) of `image`. Used by both the Clone Stamp (another spot on the same layer) and the History Brush (an earlier state of the layer, offset 0).
  `StrokeKind` can be cloned but is no longer `Copy` (`Source` carries an image).
- `Stroke::begin(doc, tip, kind, opacity, flow)`: starts a stroke on the current layer and records the current selection.
- `Stroke::add_point(doc, x, y)`: extends the stroke to (`x`, `y`) at full pressure. The first point places one dab. After that, dabs are placed along a straight line every `spacing` × the current diameter (at least 1 pixel), and the spacing stays continuous across calls.
- `Stroke::add_point_with_pressure(doc, x, y, pressure)`: the same with the pen's pressure there (0–1). Each dab's pressure is interpolated from the previous point's.
- `Stroke::with_pressure(Pressure { size, opacity, flow })`: chooses what pressure controls. By default it controls nothing.
  - **Size:** the dab's diameter is the tip's × pressure (at least 1 pixel), and spacing follows it.
  - **Opacity:** a pixel's coverage moves toward the dab's pressure instead of toward 1, and never goes down. Light pressure therefore can't build up past its own level, however often it passes.
  - **Flow:** the dab's flow is multiplied by the pressure.
- `last_point()`: where the stroke ended; the UI uses it for Shift+click straight lines.
- `StrokeError`: the reason a stroke cannot start; `message(tool)` gives Photoshop's message text:
  - No layer: "Could not use the {tool} because there is no layer to paint on."
  - `lock_pixels`: "Could not use the {tool} because the layer is locked."
  - Hidden layer: "Could not use the {tool} because the target layer is hidden."

## Paint modes (`PaintMode`)

The Mode of the Brush and Pencil, set with `Stroke::with_mode` (default Normal):

- `Blend(mode)`: uses the layer's existing pixels as the base color and composites the color on top with the blend formula of `blend::composite` at a strength of "coverage × opacity × selection" (Dissolve is random per pixel, as with the layer blend mode). Example: red in Multiply painted on `#808080` gives `#800000`.
- `Behind`: paints the color "under" the layer: existing pixels sit on top of the color, so only transparent areas get painted.
- `Clear`: lowers alpha by the strength (like the Eraser).
- On the background layer, with locked transparent pixels, or on a mask, the blend result keeps the original alpha; Behind and Clear do not change pixels there.

## Smudge, pattern, Background Eraser and Color Replacement

- `Pattern(image)` (Pattern Stamp): the target pixel takes the pattern's color at (x mod width, y mod height), i.e. tiled from the document origin (Aligned); mixed in by coverage, opacity and flow.
- `Smudge(strength)` (Smudge): does not use coverage but changes the current pixels directly: each dab pulls the pixels at the previous dab's position to the current position by "strength × tip coverage × selection" (reading everything before writing); the first dab only picks up color and changes nothing.
- `BackgroundErase(ColorMatch)` (Background Eraser) and `ReplaceColor { color, mode, matching }` (Color Replacement; `mode` is the Hue / Saturation / Color / Luminosity blend formula) only change "matching" pixels:
  - Sample color (`Sampling`): `Continuous` takes the center of each dab (pixels before the stroke started), `Once` takes the center of the first dab, `Swatch(c)` uses the given color (the background color).
  - Matching: the pixel's alpha is greater than 0, and the maximum per-channel difference from the sample color is ≤ the tolerance (0–1); when `protect` gives a color, pixels within tolerance of it stay unchanged (Protect Foreground Color).
  - With `contiguous`, only matching pixels within the dab that are 4-connected from the center pixel (if the center does not match, that dab changes no pixels); otherwise all matching pixels within the dab.
  - The Background Eraser lowers the alpha of matching pixels by the strength; Color Replacement moves the color toward the blend result by the strength, with alpha unchanged.

## Healing strokes

- `Heal { source, dx, dy }` (Healing Brush) and `SpotHeal(source)` (Spot Healing Brush) don't go through the coverage map. Each dab heals the pixels under the tip with `heal::heal_window` over the dab's box plus one pixel (the edge values come from the layer as it is at that moment, so dabs follow on from earlier ones), and mixes them into the layer by the tip's coverage × opacity × selection; alpha is kept (or follows `mix` when transparency isn't locked).
- The Healing Brush takes its texture at the fixed offset; the Spot Healing Brush picks an offset per dab with `heal::proximity_match` on its source image. A dab whose source would leave the image changes nothing.

## Pixel rules

- Painting color (regular layer): the color is composited source-over onto the original pixel with `amount` as alpha.
- Background layer or "lock transparent pixels": only mixes the color toward the target color; alpha stays the same.
- Eraser (regular layer): alpha is multiplied by `1 − amount`.
- Eraser (background layer or locked transparent pixels): like painting color, mixes the color toward the background color with alpha unchanged, matching Photoshop erasing to the background color on the background layer.
- Selection: scales the amount by selection degree (0–255 → 0–1); a feathered selection takes effect proportionally.
- `mark_dirty` is called after each dab.

### Painting on masks

The stroke target is decided by priority Quick Mask > layer mask > pixels. In Quick Mask mode, painting works even when the layer is hidden or locked (only the Quick Mask changes). When the editing target is a mask (Quick Mask or `Document::editing_mask`), the stroke's base image is the mask: Brush/Pencil colors are converted to gray (`adjust::mask_gray`, i.e. luminance), the Eraser paints the gray of the background color, and the retouching tools act directly on the grayscale image; the mask is opaque, and alpha stays the same.

### Retouching tools

Dodge, Burn, Sponge, Blur and Sharpen work dab by dab on the pixels as they are at that moment, so their effect builds up. Every overlapping dab, every pass back and forth within a stroke, and every airbrush dab (`build_up`) adds to the last, as Photoshop's do.

Each dab works like this:

- It reads the box it covers, plus one pixel of margin, before writing.
- For each pixel it computes the "full strength" target value with `retouch_target`.
- It mixes from the current pixel toward that target in premultiplied alpha by `amount`: the tip's coverage × flow (× pressure for flow) × opacity × selection degree. The UI passes Exposure or Strength as opacity, and the Sponge's Flow as flow.
- On the background layer or with locked transparent pixels, only color is mixed and alpha is kept.

The options (`Stroke::with_retouch(Retouch { protect_tones, vibrance, protect_detail })`) are described with the targets below. Target values (v is a channel value in 0–1):

- Dodge: `v + w(v) × (1 − v)`; Burn: `v − w(v) × v`. Weight w: Shadows `(1 − v)²`, Midtones `4v(1 − v)`, Highlights `v²`. Computed per channel; this approximates Photoshop's algorithm. With Protect Tones the same curve moves the pixel's luminosity (0.299 R + 0.587 G + 0.114 B) instead, the color is scaled with it (keeping its hue), and a color that would pass white is pulled toward the gray of its luminosity until it fits.
- Sponge: in HSL, sets saturation to 0 (desaturate) or doubles it (saturate, at most 1). With Vibrance, saturate gives `s + s(1 − s)` and desaturate `s²`, so colors near full or no saturation change least.
- Blur: the 3×3 premultiplied average of the current pixels; Sharpen: `v + (v − 3×3 average)`. With Protect Detail, Sharpen adds half that difference, and nothing where it is under one level (noise).
- Source: the pixel at the corresponding position in `image` (including alpha); outside the bounds of `image` the original pixel is kept.

## Known limitations

- Brush tips are computed (round or elliptical); there are no sampled tips. There is no airbrush build-up, and no scattering, texture, dual brush or color dynamics.
- Pressure only reaches the dabs of the coverage-based strokes (painting, erasing, retouching). Smudge, healing and the color-matching strokes use the tip's own size.
- The retouching tools' Protect Tones, Vibrance, Sample All Layers and Protect Detail options are not implemented.
- The only brush mode is Normal; the Eraser only has Brush mode (no Pencil or Block mode).
- Every dab triggers a full recomposite and upload of the document, so painting on large documents is slow.

## Test coverage (brush shape and dynamics)

- `retouching_builds_up_and_its_options`: a second Burn pass darkens further, `build_up` darkens in place, Protect Tones keeps a 2:1 red/green ratio that plain Dodge changes, Vibrance saturates a strong red less, Protect Detail sharpens an edge less.
- `elliptical_tips_and_spacing`: a 30% round tip is wide at 0° and tall at 90°; 100% spacing places dabs a diameter apart.
- `pressure_controls_size_and_opacity`: at 20% pressure a size-controlled line is thin; at 50% an opacity-controlled stroke stays at half coverage over four passes.

## Layer groups

When the current layer is a group and the target is pixels, starting a stroke returns `StrokeError::Group` ("Could not use the {tool} because the target layer is a group."); the group's mask can still be painted.

## Test coverage

- `pattern_smudge_background_eraser_and_color_replacement`: pattern tiling, smudge pulling red into an empty area, the Background Eraser erasing only the sampled gray, Color Replacement recoloring only the gray.
- `paint_modes_blend_with_the_layer`: Multiply, Behind, Clear; `a_square_tip_covers_a_block`: the corners of a square tip are also covered.
- `hard_brush_paints_full_color_in_its_core`, `pencil_is_aliased`: a hard brush paints full color at its center; the Pencil produces only fully transparent and fully opaque pixels.
- `dodge_burn_and_sponge`: midtone Dodge lightens and Burn darkens; Highlights barely affects dark pixels; after desaturation the three channels are equal.
- `quick_mask_round_trip`: in Quick Mask mode painting works even on a hidden layer; areas painted black are not in the selection after exiting; with nothing painted there is no selection after exiting.
- `blur_sharpen_and_clone`: Blur produces intermediate values at a black-white edge; Clone paints the red dot at (5, 5) to (20, 20) and leaves its surroundings unchanged.
- `opacity_caps_a_single_stroke`: scrubbing repeatedly within one stroke stops at 50% opacity.
- `flow_builds_up`: at low flow, passing over the same point repeatedly builds up.
- `selection_masks_paint`: areas outside the selection are not painted.
- `eraser_removes_alpha_and_paints_background_on_background_layer`: erases to transparency on a regular layer, and to the background color on the background layer.
- `locked_and_hidden_layers_refuse`: layers with locked pixels and hidden layers refuse painting.
- `spacing_places_dabs_along_a_line`: dabs along a straight line have no gaps.
