# heal.rs: healing

## Responsibilities

The healing algorithm shared by the Healing Brush, Spot Healing Brush, Patch tool and Content-Aware Move. As in Photoshop, a repair keeps the texture of a source and takes the shading of the place it repairs: the difference between target and source along the repaired area's edge is spread smoothly inside the area (a membrane, the solution of Laplace's equation with those edge values), and the result is the source plus that membrane. The repair therefore meets its surroundings at the edge without a seam.

## Interface

- `membrane(w, h, inside, values)`: solves Laplace's equation on a `w` × `h` window. Pixels outside `inside` keep their `values`; those inside become the smooth interpolation of their neighbors. Starts from the mean of the edge pixels touching the area, then successive over-relaxation (ω 1.9) until the largest change is below 0.01 of a level, at most `2 × max(w, h)` iterations (30–4000).
- `membrane_damped(w, h, inside, values, damping)`: the same with the screened equation ∇²u = damping · u, so the edge values fade toward 0 inside over about 1/√damping pixels. `membrane` is the undamped case.
- `heal_window(image, source, (x0, y0, w, h), inside, (dx, dy))`: healed colors (straight RGB 0–255) for the window: inside the area, the source's pixel at (x − dx, y − dy) plus the membrane of (image − source) on the edge; outside, `image`'s pixels. `None` when any source pixel of the window falls outside its image. It is `heal_window_with` in the default style.
- `heal_window_with(…, style)`: with a `HealStyle`: the membrane is damped by the style's Diffusion, and the healed colors are combined with the pixels under them by its mode.
- `HealStyle { mode, diffusion }`: the healing tools' Mode menu and Diffusion. `diffusion` is 1–7 (the options bars' Diffusion), or `None` for Legacy healing (the Healing Brush's Use Legacy, and the Spot Healing Brush), which spreads the edge's shading all the way across. Diffusion d damps the membrane so the shading reaches about 3 × 2^(d−1) pixels in (3 at 1, 48 at the default 5, 192 at 7): a low Diffusion keeps more of the source's own colors away from the edge, a high one blends the repair into its surroundings further in. These reaches were chosen to give the options bar's range a visible effect; they aren't measured against Photoshop.
- `HealMode`: Normal, Replace, Multiply, Screen, Darken, Lighten, Color, Luminosity (`ALL` in the menu's order). `apply(under, healed)` blends the healed color over the pixel under it with the blend mode of the same name (`blend::blend`); Normal and Replace take the healed color. Replace differs in how a stroke lays it down: `replace_takes(x, y, coverage)` is a fixed per-pixel dither (a hash of x and y below the coverage), so at a soft brush's edge each pixel either takes the healed color or keeps its own, keeping the grain instead of mixing it away.
- `mirrored_texture(image, window, (cx, cy), radius)`: the Spot Healing Brush's Create Texture: an image for the window whose pixels inside the dab (distance ρ ≤ radius + 1 from the center) come from `image` at distance 2·radius + 2 − ρ in the same direction (mirrored through the dab's edge, and reflected at the image's edges); pixels outside keep their own. Healing from it at offset (x0, y0) fills the dab with the texture just around it.
- `tiled_pattern(pattern, w, h)`: a pattern tiled over a `w` × `h` image from its top-left corner (opaque): the Healing Brush's Pattern source and the Patch tool's Use Pattern.
- `proximity_match(image, (cx, cy), radius)`: the Spot Healing Brush's source for a dab: of the spots three radii away in eight directions, the one whose ring just outside the dab (radius + 1 to radius + 4) differs least, on average, from the dab's own ring. Spots whose source area would leave the image are skipped. Returns the offset (target − source), or `None` when every direction is out of the image.
- `patch(doc, selection, (dx, dy))`: the Patch tool in Source mode: the selected area of the active layer is healed with the texture where the selection was dragged to (target (x, y) takes the source at (x + dx, y + dy)), blended by the selection's value. Alpha is kept. It is `patch_with` with the layer as the source, the default style and Transparent off.
- `patch_with(doc, selection, source, (dx, dy), style, transparent)`: heals the selection from `source` (target (x, y) takes the source at (x − dx, y − dy)) in `style`. The Patch tool passes its Diffusion; Use Pattern passes the tiled pattern at offset 0. With `transparent` only the source's texture goes over the layer: the healed result's difference from its own 5 × 5 average is added to the layer's colors, so the area keeps its colors and takes on the source's detail.
- `patch_to(doc, selection, (dx, dy))`: the Patch tool's Destination mode and Content-Aware Move's Extend: the selected pixels are copied by (dx, dy) and healed into their new place; the original stays.
- `content_aware_move(doc, selection, (dx, dy))`: the selected pixels move by (dx, dy), healed into their new place, and the area they leave is filled with the membrane of its surroundings (a smooth fill, without texture).

All three document operations return whether anything changed; they do nothing without an active pixel layer or an empty selection.

## Known limitations

- The hole Content-Aware Move leaves is filled smoothly, without synthesized texture (Photoshop's content-aware fill).
- The Spot Healing Brush's Content-Aware type uses Proximity Match (Photoshop's content-aware synthesis is P3 #26); Create Texture mirrors the surroundings, which is not Photoshop's own texture synthesis.
- The Patch tool's Content-Aware patch (its Structure and Color options) is P3 #26 as well; the Patch pop-up's Content-Aware works like Normal.

## Tests

- `membrane_interpolates_the_edge`: a row with ends 0 and 100 becomes a ramp (the middle ≈ 50).
- `healing_takes_texture_and_matches_shading`: a dark spot on a brightness ramp, healed from the ramp beside it, follows the ramp.
- `diffusion_limits_how_far_the_edge_reaches`: a flat 50 source healed into 100 with a dark left edge: Legacy reaches 100 far from the edge, Diffusion 1 keeps more of the source there; both match the dark edge next to it.
- `modes_combine_with_the_pixels_under`: Normal, Darken, Lighten and Multiply give the blend modes' colors; Replace's dither takes about 30% of pixels at coverage 0.3, all at 1 and none at 0.
- `create_texture_mirrors_the_surroundings`: outside the dab the texture keeps its own pixels; its center takes the pixel 2r + 2 away.
- `patch_transparent_and_pattern`: the tiled pattern repeats from the corner; a Transparent patch with a black-and-white checker keeps a red area reddish with the checker's alternation.
- `patch_and_content_aware_move`: patching a black square from white beside it turns it white; moving a black square fills its old place with white and leaves it black at the new place.
