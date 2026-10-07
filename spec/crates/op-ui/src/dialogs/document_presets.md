# dialogs/document_presets.rs: New Document presets

## Component responsibility

The data and drawing used by the New Document dialog (`new_document.md`): length units, presets, the preset list for each category, and the icons on the cards.

## Units (`Unit`)

Pixels, Inches, Centimeters, Millimeters, Points, Picas. `pixels(ppi)` gives how many pixels one unit equals (1 inch = 2.54 centimeters = 25.4 millimeters = 72 points = 6 picas); `short` is the abbreviation on the card (px, in, cm, mm, pt, pica); `format` displays a value: pixels are rounded to integers, others have at most 3 decimals with trailing zeros removed.

## Presets (`Preset`)

Name, width, height, unit, resolution (ppi), and the kind of card icon (`Kind`). `size_label` is the size line on the card, e.g. "7 x 5 in @ 300 ppi"; `pixels` converts to pixels.

`category(index)` gives each page's blank document presets in Photoshop 2026's order:

- **Photo** (9, inches, 300 ppi): Default Photoshop Size 7 × 5, Landscape 3 × 2, 6 × 4, 7 × 5, 10 × 8, Portrait 2 × 3, 4 × 6, 5 × 7, 8 × 10.
- **Print** (300 ppi): Letter, Legal, Tabloid (inches); A4, A6, A5, A3, B5, B4, B3, C4, C5 (millimeters).
- **Art & Illustration** (300 ppi): 1000 / 2000 pixel grid, Poster 18 × 24 in, Postcard 4 × 6 in, 1080p, 720p.
- **Web** (pixels, 72 ppi): Web Most Common 1366 × 768, Web Large, Web Medium, Web Minimum, Web Small, MacBook Pro 13 / 15 (Retina), iMac 27, Desktop HD Design.
- **Mobile** (pixels, 72 ppi): iPhone, iPad, Android, Surface, Apple Watch, Mobile Design, iOS 7 and Mac icon sizes, 24 in total.
- **Film & Video** (pixels, 72 ppi): HDTV, HDV, DVCPRO HD, DCI 2K/4K/8K, UHDTV, FUHDTV, NTSC, PAL, Cineon, Film (2K)/(4K), 24 in total.

## Card icons (`paint_icon`)

1 pt `#b9b9b9` lines; a frame is drawn with the preset's aspect ratio: the longest side runs from 36 pt to 82 pt according to the square root of its size within the page (`scales`: the position of the square root of the pixel area within the page, 0–1), and the other side is at most 62 pt (measured from the cards on Photoshop's Photo page). Depending on the kind, the frame contains: photo (mountains and sun), page (folded corner; Custom draws a crosshair outside the top-left corner), dot grid, brush, browser (top bar), phone/tablet (rounded corners and Home button), Surface (stylus on the right), watch (watch band), app icon (rounded dot grid), thin strip, video (play triangle and 2K/4K/8K corner badge).

## Known limitations

- The icons are approximate drawings made from shapes, not Photoshop's icon assets; Print, Mobile, and Film & Video list only the presets visible in the screenshots (Photoshop has 14, 28, and 25 respectively).

## Test coverage

- `presets_convert_to_pixels`: Photo has 9 and the first is 2100 × 1500 pixels; A4 is 2480 × 3508; the Web size line; three decimals for inches.
