# magnetic.rs: Magnetic Lasso edge snapping

## Responsibilities

The edge model behind the Magnetic Lasso: a per-pixel cost that is low along strong edges, the cheapest path between two points through it (the "live wire" the outline follows), and snapping the pointer to the nearest strong edge.

## Interface

- `EdgeMap::new(width, height, rgba, contrast)`: from straight RGBA8 pixels (the merged image). Luminance (0.299 R + 0.587 G + 0.114 B, times alpha) gets a Sobel gradient, scaled to 0–1 (magnitude / 4, capped at 1). `contrast` (the bar's Contrast, 0–1) is the weakest edge that counts: a pixel's edge strength is `max(0, (gradient − contrast) / (1 − contrast))`, and its cost `1 − √strength + 0.02` (flat areas cost about 1, strong edges almost nothing).
- `snap((x, y), radius)`: the pixel within `radius` (the bar's Width) with the strongest edge; the point itself when nothing there beats it by more than 0.05.
- `trace(from, to, margin)`: Dijkstra over 8-connected pixels (diagonal steps cost √2 × the pixel's cost) within the two points' bounding box widened by `margin`; the path includes both ends.
- `gradient(x, y)`: the gradient magnitude there.

## Tests

- `the_path_follows_an_edge`: on a black/white image split at x = 20, a path between two points left of the edge runs mostly along x 19–20, and the pointer snaps to the edge.
