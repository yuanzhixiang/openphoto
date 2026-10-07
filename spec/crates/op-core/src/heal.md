# heal.rs: healing

## Responsibilities

The healing algorithm shared by the Healing Brush, Spot Healing Brush, Patch tool and Content-Aware Move. As in Photoshop, a repair keeps the texture of a source and takes the shading of the place it repairs: the difference between target and source along the repaired area's edge is spread smoothly inside the area (a membrane, the solution of Laplace's equation with those edge values), and the result is the source plus that membrane. The repair therefore meets its surroundings at the edge without a seam.

## Interface

- `membrane(w, h, inside, values)`: solves Laplace's equation on a `w` × `h` window. Pixels outside `inside` keep their `values`; those inside become the smooth interpolation of their neighbors. Starts from the mean of the edge pixels touching the area, then successive over-relaxation (ω 1.9) until the largest change is below 0.01 of a level, at most `2 × max(w, h)` iterations (30–4000).
- `heal_window(image, source, (x0, y0, w, h), inside, (dx, dy))`: healed colors (straight RGB 0–255) for the window: inside the area, the source's pixel at (x − dx, y − dy) plus the membrane of (image − source) on the edge; outside, `image`'s pixels. `None` when any source pixel of the window falls outside its image.
- `proximity_match(image, (cx, cy), radius)`: the Spot Healing Brush's source for a dab: of the spots three radii away in eight directions, the one whose ring just outside the dab (radius + 1 to radius + 4) differs least, on average, from the dab's own ring. Spots whose source area would leave the image are skipped. Returns the offset (target − source), or `None` when every direction is out of the image.
- `patch(doc, selection, (dx, dy))`: the Patch tool in Source mode: the selected area of the active layer is healed with the texture where the selection was dragged to (target (x, y) takes the source at (x + dx, y + dy)), blended by the selection's value. Alpha is kept.
- `patch_to(doc, selection, (dx, dy))`: the Patch tool's Destination mode and Content-Aware Move's Extend: the selected pixels are copied by (dx, dy) and healed into their new place; the original stays.
- `content_aware_move(doc, selection, (dx, dy))`: the selected pixels move by (dx, dy), healed into their new place, and the area they leave is filled with the membrane of its surroundings (a smooth fill, without texture).

All three document operations return whether anything changed; they do nothing without an active pixel layer or an empty selection.

## Known limitations

- The hole Content-Aware Move leaves is filled smoothly, without synthesized texture (Photoshop's content-aware fill).
- Spot Healing's three types (Content-Aware, Create Texture, Proximity Match) all use Proximity Match.
- The tools' Mode menus (Replace, Multiply …) and the Patch tool's Diffusion and Transparent options have no effect; healing always works in Normal mode.

## Tests

- `membrane_interpolates_the_edge`: a row with ends 0 and 100 becomes a ramp (the middle ≈ 50).
- `healing_takes_texture_and_matches_shading`: a dark spot on a brightness ramp, healed from the ramp beside it, follows the ramp.
- `patch_and_content_aware_move`: patching a black square from white beside it turns it white; moving a black square fills its old place with white and leaves it black at the new place.
