# more_filters.rs: Wave, Shear, Displace, blurs, Reduce Noise, Smart Sharpen, renders, Extrude, Oil Paint

## Responsibility

The pixel work of more of the Filter menu, on a layer's straight RGBA pixels, reached through `Filter` variants (`filter.md`, `more`). Their dialogs are not built yet: the Filter menu items are still placeholders.

## Sampling

`sample`: bilinear, premultiplied while mixing; outside the image `Undefined::WrapAround` or `RepeatEdge`. `hash`: a repeatable random value per position, purpose and seed.

## Filters

- Wave (`Wave`): Generators (1–999) waves, each with a wavelength and amplitude drawn between the Min and Max from the seed (Randomize = another seed), Sine / Triangle / Square, the horizontal and vertical Scale (%), and Undefined Areas; x moves by the waves along y and y by those along x. Defaults are Photoshop's (5, 10–120, 5–35, 100 %, Sine, Repeat Edge Pixels).
- Shear: each row moves sideways by the curve's offset at its height (`shear_offsets` turns up to eight (height, offset) points into one offset per row, linear between them).
- Displace: a registered map (`register_map`, `map`; tiled, or stretched to fit) moves pixels by `(value − 128) / 128 × 128 px × scale %` (red horizontally, green vertically).
- Radial Blur: Spin (along circles, the amount as degrees) or Zoom (along rays, the amount as percent of the distance), 8 / 16 / 32 samples for Draft / Good / Best, about a center in 0–1 of the image.
- Smart Blur: the mean of the neighbours within the radius whose channels are within the threshold; Edge Only (white edges on black) and Overlay Edge (white edges on the blur).
- Shape Blur: the mean over a shape's footprint (Circle, Square, Star, Heart, Diamond — standing in for Photoshop's custom-shape library) of the radius.
- Lens Blur (no depth map): a disc or polygonal iris (blades) of the radius; pixels at or above the threshold count up to 9× by Brightness, blooming like specular highlights; Noise adds uniform noise.
- Reduce Noise: an edge-keeping 5 × 5 average (differences above 12–52 levels by Preserve Details are kept out) by Strength / 10; Reduce Color Noise pulls each pixel's chroma toward its neighbourhood's; Sharpen Details adds an unsharp mask back.
- Smart Sharpen: an unsharp mask whose blur matches Remove (Gaussian of the radius, Lens: a tighter Gaussian, Motion: a line at the angle); differences under Reduce Noise × 10 % levels are left alone; the shadows' and highlights' Fade Amounts weaken it in the darkest and brightest 30 %.
- Fibers: each column a random walk between the background and foreground colors, Variance its step, Strength the length of its smoothing along the column.
- Lens Flare: a glow and ring at the center, ghosts along the line through the image's middle, a streak for Movie Prime, screened over the image; Brightness scales it; the lens sets the size.
- Extrude: squares of the size, Blocks (rims on the lower and right edges in shadow; Solid Front Faces paints the square's average) or Pyramids (four faces lit from the top left), Depth scaling the shading, random or Level-based heights.
- Oil Paint: a Kuwahara smoothing whose reach grows with Stylization and Scale, mixed by Cleanliness, then lighting from the angle with Shine.

## Not included

Blur Gallery, Render › Flame (needs paths), Picture Frame and Tree (Photoshop's procedural artwork libraries) are not here (see `spec/README.md`).

## Known limitations

These follow Photoshop's controls and pictures; Adobe's exact algorithms are its own, so results differ.

## Test coverage

`wave_and_shear_move_pixels_sideways`, `displace_by_a_gray_map`, `blurs_soften_and_keep_flat_areas` (each blur changes a checker; flat fields stay; Smart Blur keeps the checker's edges and Edge Only finds them), `reduce_noise_and_smart_sharpen` (speckle's deviation falls below 60 %; edges gain contrast), `renders` (fibers mix the colors, a flare is bright at its center and dark far away, pyramids and oil paint change the image).
