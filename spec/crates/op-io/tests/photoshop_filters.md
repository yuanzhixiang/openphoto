# tests/photoshop_filters.rs: filters against Photoshop

## Responsibilities

An ignored integration test that compares filters with Photoshop 2026 on the same 64 × 64 test patterns.

- **On the Photoshop side,** a script makes each pattern on Photoshop's probe document, runs the filter there and saves a PNG copy.
- **On ours,** the test makes the same pattern (`pattern`: a background color with filled rectangles) and runs the filter.
- **Each case reports** the largest channel difference, and how many channel values differ by more than one level. Ours is also written as `ours_<case>.png` next to Photoshop's file, for a closer look.

Run it with `OPENPHOTO_FILTER_DIR` set to the folder of Photoshop's PNGs:

`cargo test -p op-io --test photoshop_filters -- --ignored --nocapture`

When the folder or a file is missing, the case says so and nothing fails: it is a measuring tool, not a regression test.

## Cases and what they showed

`minimum_and_maximum_round` runs Minimum and Maximum with Roundness on two patterns. Results with the current algorithm (`filter.md`):

- **A single white (or black) pixel, radius 1–10:** the largest difference is 8–14 levels.
- **Rectangles of several colors on gray, radius 3 and 7:** 28 levels at radius 3, 63 at radius 7.

**What Photoshop does:**

- Around a single pixel, its Roundness window is an anti-aliased disc of radius r + 0.5, with pixel-area coverage.
- At a rectangle's corner, the edge follows a circle of about radius r around the rectangle's outer corner, not around the corner pixel's middle.

**Window shapes tried,** none of which fits both patterns:

- A disc around each pixel's middle, with coverage averaged over 8 × 8 points: single pixels within 10 levels, but rectangles up to 80.
- Each pixel square grown by the radius: single pixels up to 71 levels.

So the simpler coverage model stays.

## Known limitations

Producing Photoshop's PNGs needs Photoshop and the measuring scripts, which are outside the repository.
