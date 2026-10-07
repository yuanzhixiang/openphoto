# panels/histogram.rs: Histogram panel

## Component responsibilities

The Histogram panel (Window › Histogram) shows the merged image's histogram, in a compact view or an expanded one with a channel menu and statistics. The histograms come from `DocState::channel_histograms`: luminosity, red, green, blue, and the three together. They count only pixels with an alpha above 0, are taken from what the canvas shows (`canvas_image`), and are cached by revision.

## Views

- **Toggle:** a "+" (Expanded View) or "−" (Compact View) at the panel's top right switches between the views (`AppState::histogram_expanded`). The floating panel is 150 pt high in the compact view and 260 pt in the expanded one (`floating.md`).
- **Compact view:** the luminosity histogram as light vertical lines on `#3c3c3c`, normalized to the highest count.
- **Expanded view:**
  - **Channel menu:** a pop-up offering Luminosity, Red, Green, Blue, RGB and Colors (`AppState::histogram_channel`).
  - **Graph:** 120 pt high, below the menu. Red, green and blue are drawn in their colors, Luminosity and RGB light, and Colors as the three histograms drawn over each other.
  - **Statistics** of the channel (Colors uses RGB's): Mean and Std Dev (two decimals), Median and Pixels (`statistics`). With the pointer over the graph, Level, Count (the pixels at that level) and Percentile (the share at or below it) are shown too. Cache Level is always 1.

## Known limitations

- There is no All Channels view and no panel menu. The cache warning and Uncached Refresh don't exist, since the histogram is always current.
- The Source pop-up (Entire Image, Selected Layer, Adjustment Composite) is missing.

## Test coverage

- `statistics_of_a_histogram`.
- `ui_tests::histogram_expands_and_navigator_zooms`: the expanded view's red statistics of a flat gray image (every pixel, no spread). Screenshot `histogram_expanded.png`.
