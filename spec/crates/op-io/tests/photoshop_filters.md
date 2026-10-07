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

`motion_blur_and_emboss_angles` runs Motion Blur (distance 10) and Emboss (height 3, amount 100) at several angles on the rectangles pattern. `motion_blur_impulses` and `emboss_impulses` run them on one pixel, raised in red and lowered in green, on mid gray (`DOT`): the result is the filter's impulse response, read off level by level.

**What the impulses showed, and what the algorithms now do (`filter.md`):**

- **Motion Blur** is not one shift-invariant kernel. Its response changes with the pixel's column but not its row, which gave away a shear. Photoshop shears the image so the line lies along a row, averages along the rows, and shears back.
  - The line is `⌊d·cos a + 0.5⌋` pixels along and `⌊d·sin a + 0.5⌋` across, truncated toward zero. That is why negative angles come out shallower than positive ones: −5° at distance 10 is horizontal, and −45° rises 6 pixels in 7. Floating-point rounding makes −30° at distance 3 horizontal too.
  - The shear is −(x + 0.5)·across/along, interpolated with bicubic a = −0.75, the same for every image size.
  - The intermediates are clamped to 0–255, which shows near black.
  - Angles beyond ±90° act as ±90°.
  - Every case (36, including the image borders) is now within one level; before, the diagonals were off by up to 18.
- **Emboss** reads each of its two points from the pixels within one pixel of it, weighted 1 − d − 0.012·d³ and not normalized. Every case is within one level, and the axes are exact; with bilinear sampling 30° was off by 22 levels.

## Known limitations

Producing Photoshop's PNGs needs Photoshop and the measuring scripts, which are outside the repository.
