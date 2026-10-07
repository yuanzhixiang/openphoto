# smart_select.rs: Quick Selection and Object Selection

## Responsibilities

The region-finding behind the Quick Selection and Object Selection tools. Photoshop uses learned models; these are color-and-edge approximations that select uniform objects against contrasting surroundings.

## Interface

- `Sampler::new(width, height, rgba)`: the pixels to look at (the merged image or the active layer, straight RGBA8) with their edge strength (the `magnetic::EdgeMap` gradient, 0–1).
- `QuickSelect::new(sampler)`, `dab((cx, cy), radius)`, `selection()`: the Quick Selection tool's region, grown dab by dab. A dab takes the pixels under the brush (a disc of `radius`) as seeds; their mean color and mean spread (largest channel difference from the mean) set a threshold of 24 levels + 1.5 × spread, at most 80. The region grows breadth-first through 4-connected neighbors whose largest channel difference from the mean is within the threshold, never across a strong edge (gradient 0.35 or more), and no farther than eight radii from the dab's center. Dabs add to the region; `selection()` is a hard-edged selection of it.
- `object_in_rect(sampler, (x0, y0, x1, y1))`: the Object Selection tool's object inside a rectangle. Colors sampled along the rectangle's edge (about every 1% of its perimeter) are the background; pixels farther than 40 levels (largest channel difference) from every one of them are the object. Its largest 4-connected part is kept, with every part at least a tenth of its size; holes that don't reach the rectangle's edge are filled. `None` when nothing stands out or the rectangle is under 3 pixels across.

## Known limitations

- No subject detection: objects that don't differ in color from their surroundings aren't found, and textured backgrounds can leak into the Quick Selection or count as object.

## Tests

- `quick_selection_stays_within_edges`: a dab inside a red disc on gray grows over the disc and stops at its edge.
- `object_selection_finds_the_disc`: a rectangle around the disc selects it (bounds within a pixel), the white dot inside it filled in; a plain area gives `None`.
