# panels/info.rs: Info panel (F8)

## Component responsibilities

Shows the color under the pointer, the pointer position, the selection size and the document size, arranged in Photoshop's two columns.

## Content

- Left column: R, G, B (the color of the composite image under the pointer, 0–255) and "8-bit"; right column: C, M, Y, K (a simple conversion via `op_color::Cmyk`, in percent) and "8-bit".
- Below the separator: X, Y (the document pixel under the pointer, rounded down); W, H (the width and height of the selection's bounding rectangle, empty when there is no selection).
- At the bottom, "Doc: flattened size/layered size": the flattened size is width × height × 3 bytes; the layered size is estimated here as width × height × 3 bytes per layer (so with a single layer the two numbers are the same, as in Photoshop). ≥ 1 MB is shown as M (two decimal places), otherwise as K (one decimal place).
- When the pointer is not over the document, color and position are empty. The pointer position comes from `DocState::pointer` (updated by the document view every frame), and color sampling uses `DocState::sample_average` (sample size 1, all layers).

## Known limitations

- CMYK is not converted using a color profile; there are no panel options (sample colors, ruler units, status information) and no tool hints.

## Test coverage

- `sizes_read_like_photoshop`: 1920 × 1080 is 5.93M, 100 × 100 is 29.3K.
