# more_filters.rs: Wave, Shear, Displace, blurs, Reduce Noise, Smart Sharpen, renders, Extrude, Oil Paint

## Responsibility

The pixel work of more of the Filter menu, on a layer's straight RGBA pixels, reached through `Filter` variants (`filter.md`, `more`). Their dialogs are in `op-ui` (`dialogs/adjust.md`); Displace's map is picked after OK (`lib.md`).

## Sampling

`sample`: bilinear, premultiplied while mixing; outside the image `Undefined::WrapAround` or `RepeatEdge`. `hash`: a repeatable random value per position, purpose and seed.

## Filters

- Wave (`Wave`): Generators (1–999) waves, each with a wavelength and amplitude drawn between the Min and Max from the seed (Randomize = another seed), Sine / Triangle / Square, the horizontal and vertical Scale (%), and Undefined Areas; x moves by the waves along y and y by those along x. Defaults are Photoshop's (5, 10–120, 5–35, 100 %, Sine, Repeat Edge Pixels).
- Shear: each row moves sideways by the curve's offset at its height (`shear_offsets` turns up to eight (height 0–1, offset −0.5–0.5 of the width) points into one offset per row along a natural cubic spline through them, the same as Curves' (`adjust::Spline`), so the curve bends smoothly as Photoshop's does).
- Displace: a registered map (`register_map`, `map`; tiled, or stretched to fit) moves pixels by `(value − 128) / 128 × 128 px × scale %` (red horizontally, green vertically).
- Radial Blur: Spin (along circles, the amount as degrees) or Zoom (along rays, the amount as percent of the distance), 8 / 16 / 32 samples for Draft / Good / Best, about a center in 0–1 of the image.
- Smart Blur: the mean of the neighbours within the radius whose channels are within the threshold; Edge Only (white edges on black) and Overlay Edge (white edges on the blur). Quality reads every third (Low), every second (Medium) or every (High) neighbour across and down; the nearest four always count, so edges are found the same way.
- Shape Blur: the mean over a shape's footprint (Circle, Square, Star, Heart, Diamond — standing in for Photoshop's custom-shape library) of the radius.
- Lens Blur (`LensBlur`, `lens_blur`): each pixel averages an iris-shaped neighborhood. The iris (`LensBlur::inside`) is a polygon of 3–8 blades turned by Rotation, rounded toward the circle by Blade Curvature (100 is a circle). Pixels at or above Threshold count up to 9× by Brightness, blooming like specular highlights. Noise is Add Noise's (`filter.md`): uniform or Gaussian, per channel or Monochromatic. With a depth map (`DepthSource`: Transparency reads the layer's alpha, Layer Mask its mask, passed in by `filter::apply`), a pixel's radius is Radius × |depth − Blur Focal Distance| / 255 (Invert flips the depths), so the focal depth stays sharp; each radius's taps are made once.
- Reduce Noise (`ReduceNoise`, `reduce_noise`): Remove JPEG Artifact first (`deblock`: across each 8 × 8 block edge, the pixels on either side move halfway toward each other where every channel differs by less than 24 levels); then each channel's own Strength and Preserve Details (Advanced › Per Channel; `smooth_channel`, the overall smoothing on that channel alone); then the overall pass: an edge-keeping 5 × 5 average (differences above 12–52 levels by Preserve Details are kept out) by Strength / 10; Reduce Color Noise pulls each pixel's chroma toward its neighbourhood's; Sharpen Details adds an unsharp mask back.
- Smart Sharpen (`SmartSharpen`, `smart_sharpen`): an unsharp mask of Radius and Amount with the blur it undoes (Gaussian; Lens: a disc, finer edges; Motion: along the angle), Reduce Noise keeping small differences out (not in Use Legacy). Shadows and Highlights (`ToneFade`) fade it by their Fade Amount over the darkest or brightest Tonal Width of the tones, judged by each pixel's neighbourhood luminance over their Radius: fully at black or white, not at all past the width. Use Legacy's More Accurate runs two passes of half the amount.
- Fibers: each column a random walk between the background and foreground colors, Variance its step, Strength the length of its smoothing along the column.
- Lens Flare: a glow and ring at the center, ghosts along the line through the image's middle, a streak for Movie Prime, screened over the image; Brightness scales it; the lens sets the size.
- Extrude: squares of the size, Blocks (rims on the lower and right edges in shadow; Solid Front Faces paints the square's average) or Pyramids (four faces lit from the top left), Depth scaling the shading, random or Level-based heights. Mask Incomplete Blocks leaves the squares cut off by the image's edge as they are.
- Oil Paint: a Kuwahara smoothing whose reach grows with Stylization and Scale, mixed by Cleanliness, then lighting from the angle with Shine.

## Not included

Blur Gallery, Render › Flame (needs paths), Picture Frame and Tree (Photoshop's procedural artwork libraries) are not here (see `spec/README.md`).

## Known limitations

These follow Photoshop's controls and pictures; Adobe's exact algorithms are its own, so results differ.

## Test coverage

`wave_and_shear_move_pixels_sideways`, `displace_by_a_gray_map`, `blurs_soften_and_keep_flat_areas` (each blur changes a checker; flat fields stay; Smart Blur keeps the checker's edges and Edge Only finds them; Low and High quality blur noise differently), `reduce_noise_jpeg_and_channels` (Remove JPEG Artifact softens a block step; red's own Strength changes red alone), `reduce_noise_and_smart_sharpen` (fully faded shadows keep an edge's dark side; Legacy's More Accurate differs from one pass; speckle's deviation falls below 60 %; edges gain contrast), `renders` (fibers mix the colors, a flare is bright at its center and dark far away, pyramids and oil paint change the image).
