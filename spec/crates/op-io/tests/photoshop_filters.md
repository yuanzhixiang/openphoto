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

`add_noise_strength` compares the standard deviation of Add Noise on flat 50% gray (256 × 256) with Photoshop's at 5–100%, uniform and Gaussian. Photoshop's noise is twice what OpenPhoto had: Gaussian has a standard deviation of amount% × 255 and is bounded at about 3.5σ (a sum of four uniforms), and uniform noise spans ±(amount% × 255 + 0.75). All are now within about 1%.

`wind_patterns` writes ours for Wind, Blast and Stagger on a white line at x 40 and on gray steps (60–255 against 40), so the streaks can be counted beside Photoshop's runs of the same patterns:

- **Wind:** a streak's first pixel is ½, ¾, ⅞ or 15⁄16 of the step, in 58/32/8/1% of streaked rows. That is Poisson-many (mean 1.1) passes of "average with the pixel before, keep the lighter", which halves a streak at each pixel (107, 53, 26, 13 … above the ground). Two passes give 161, 107, 66, 39 …. Most rows also get passes starting inside the streak, which lengthen it. Photoshop streaks two thirds of the rows at every contrast; lengths from white are 8–21 in 80% of rows (mean 14).
- **Blast:** streaks of the full color, exactly 10, 20, 30 or 40 pixels, in the same proportions as Wind's passes.
- **Stagger:** every row moves. A line moves 0.636 of the way to the downwind edge (the same at widths 128, 256 and 512 and from both sides), give or take about 0.36·√distance. A block moves whole; a gradient of 2-pixel steps moves 0–1 pixels.

## Known limitations

Producing Photoshop's PNGs needs Photoshop and the measuring scripts, which are outside the repository.
