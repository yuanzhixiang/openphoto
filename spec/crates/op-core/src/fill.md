# fill.rs: Fill, Clear and Paint Bucket

## Responsibilities

Modifications to the current layer's pixels by Edit › Fill, Edit › Clear and the Paint Bucket tool.

## Target and Errors

All operations act on the current layer. When an operation is not possible, it returns `FillError`, and `message(command name)` gives Photoshop's message text:

- No layer: "Could not complete the {command} command because there is no layer."
- Layer pixels locked: "Could not complete the {command} command because the layer is locked."
- Layer hidden: "Could not complete the {command} command because the target layer is hidden."

## Fill (`fill`)

- Fills the selection with a solid color; with no selection, fills the whole layer. The degree of selection (feathering) takes effect proportionally.
- `FillOptions`: blending mode (all 27, algorithms in `blend.md`), opacity, preserve transparency. Defaults are Normal, 100%, not preserved.
- `fill_pattern(doc, pattern, options)`: Fill's Pattern contents: each pixel takes the pattern's color at (x mod width, y mod height), tiled from the document's corner, with the same blending.
- `fill_history(doc, source, options)`: Fill's History contents: each pixel takes `source`'s pixel at the same place (the layer in the History panel's source state, which the UI looks up), with the same blending.
- For the Background layer, a layer with transparent pixels locked, or when preserve transparency is checked: only existing pixels are colored, alpha is unchanged, and fully transparent pixels are unaffected.
- In all other cases: the color is composited onto pixels by the blending mode and `opacity × degree of selection`, and can make transparent pixels opaque.

## Clear (`clear`)

- Regular layer: the alpha of pixels inside the selection is multiplied by `1 − degree of selection` (with no selection, the whole layer is cleared).
- Background layer or a layer with transparent pixels locked: fills with the background color instead, matching Photoshop.

## Paint Bucket (`bucket`)

- `BucketOptions`: fill settings, tolerance (0–255, default 32), anti-alias (default on), contiguous (default on), all layers (default off), matching Photoshop's defaults.
- Sampling: by default samples the current layer's pixels; with "all layers", samples the composite.
- Region: pixels whose difference from the clicked point in all four RGBA channels is within the tolerance. With "contiguous", only the part connected to the clicked point through 4-neighbors is taken; otherwise all similar pixels in the whole image.
- Anti-alias: the ring of pixels immediately outside the region is selected at 50%.
- The region is then multiplied by the current selection, and the foreground color (or, with Source: Pattern, the `pattern` passed in, tiled from the document's corner) is filled using the fill settings. When the clicked point is outside the document, nothing happens (returns `Ok(false)`).

## Masks

In Quick Mask mode, filling does not check whether the layer is hidden or locked. When the editing target is a mask (Quick Mask or layer mask), Fill (and ⌥⌫, ⌘⌫, the paint bucket) converts the color to gray and writes it into the mask; Clear (⌫) fills the selection with the gray of the background color.

## Magic Wand (`magic_wand`)

- `magic_wand(doc, x, y, options)`: the selection a Magic Wand click produces; the region rules are exactly the same as the paint bucket's (sampling, tolerance, contiguous, anti-alias), it just does not fill. When the clicked point is outside the canvas, or when not sampling all layers and there is no active layer, returns `None`.
- Test `magic_wand_selects_the_clicked_area`: clicking a black square gives the square's extent, clicking white gives the area outside the square, and outside the canvas gives `None`.

## Grow and Similar (`grow`)

- `grow(doc, options, contiguous)`: Select › Grow (`contiguous` true) and Select › Similar. Takes the minimum and maximum per channel (RGBA) of pixels in the current selection with selection degree ≥ 128, then widens that range on both sides by the Magic Wand's tolerance; Grow starts from these pixels and expands through 4-neighbors to connected pixels within the range; Similar selects all pixels in the whole image within the range. Edges are handled by the Magic Wand's anti-alias option, and the result is merged (added) with the original selection. Sampling is the same as the Magic Wand (current layer or all layers). With no selection, returns `None`.
- Test `grow_and_similar`: Grow expands only to the connected same-colored square; Similar includes the other square but not the white in between; with no selection, `None`.

## Known Limitations

- Fill's Content-Aware contents are not implemented (P3 #26).
- All operations traverse the whole document without clipping to the selection bounds, which is slow on large documents.
- Anti-aliasing is just a ring of 50% edge, not exactly the same as Photoshop's anti-aliasing.

## Layer Groups

When the current layer is a group, Fill, the paint bucket, adjustments (`adjust::check`) and filters return `FillError::Group`, with the message "Could not complete the {command} command because the target layer is a group.". In this case Photoshop simply grays out these menu items (verified for Invert), and the UI layer should disable the commands accordingly. The Magic Wand and Grow/Similar produce no result when the current layer is a group and not sampling all layers.

## Magic Eraser (`magic_erase`)

Erases on the current layer the region a Magic Wand click would select (same Tolerance, Contiguous, Anti-alias, Sample All Layers rules), with strength Opacity, and only inside the selection when there is one. If the current layer is the Background, it is first converted to the regular layer "Layer 0" (matching Photoshop); with transparent pixels locked, the region is filled with the background color instead. With pixels locked, hidden, a group, or no layer, returns the corresponding error. Returns whether any pixels changed.

## Red Eye (`red_eye`)

Near the clicked point (3% of the image's short side, at least 8 pixels), finds the pixel with the highest "redness" (R − max(G, B)) that exceeds the threshold (threshold = 40 + (1 − Pupil Size) × 60); starting from it, takes 4-connected pixels whose redness exceeds 60% of the threshold (no more than 4 times the search range). For these pixels, R becomes the average of G and B, then all three channels are darkened by Darken Amount (at most 60%); with a selection, blended by selection degree. When no red is found nearby, nothing happens.

## Test Coverage

- `magic_eraser_and_red_eye`: clicking Red Eye near a red dot removes the red, clicking in a corner changes nothing; clicking the Magic Eraser on white turns the background into a regular layer, erasing the white while keeping the dot.

- `fill_respects_selection_opacity_and_mode`: fills inside the selection at 50% opacity, outside the selection is unchanged; the result in Multiply mode.
- `clear_erases_or_fills_background`: clearing on the Background layer gives the background color; clearing on a regular layer gives transparency.
- `preserve_transparency_keeps_alpha`: with preserve transparency, only the color changes.
- `bucket_fills_with_a_pattern`: the bucket and `fill_pattern` lay a two-pixel pattern down tiled from the corner.
- `bucket_fills_contiguous_region`: when separated by a column of black, only one side is filled; when non-contiguous, similar pixels on both sides are filled.
- `locked_layer_refuses`: refuses when pixels are locked, with message text matching Photoshop.
