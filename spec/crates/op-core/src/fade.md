# fade.rs: Edit › Fade's pixels

## Responsibilities

Blends an edit's result back toward the pixels before it, for Edit › Fade (dialog and availability in `crates/op-ui/src/dialogs/fade.md`).

## `fade(before, after, opacity, mode)` and `fade_pixel`

- Works over the union of both images' non-transparent bounds; pixels the edit didn't change are kept as they are.
- Per pixel (straight RGBA, 0–1): the edit's pixel in the mode, M: in Normal (and Dissolve, Pass Through) the edit's color itself; in other modes, where the earlier pixel has alpha, `blend(mode, before, after)` mixed by the earlier alpha (the W3C general formula, as layers composite). M keeps the edit's alpha.
- The result mixes the earlier pixel and M at `opacity`, premultiplied: alpha = before.a × (1 − o) + after.a × o, color = (before × before.a × (1 − o) + M × after.a × o) / alpha. So 100% Normal gives the edit's result exactly, 0% the earlier pixels, and erased pixels come back partly.

## Test coverage

- `opacity_mixes_the_before_and_after`: 100% and 0% Normal give each side; 50% the average; Multiply at 100% multiplies; an erased pixel at 50% comes back half opaque.
- `whole_images_fade`: black to white at 25% gives 64.
