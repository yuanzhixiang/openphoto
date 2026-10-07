# panels/info.rs: Info panel (F8)

## Component responsibilities

Shows the color under the pointer, the pointer position, the selection size and the document size, arranged in Photoshop's two columns.

## Content

- Left column: R, G, B (the color of the composite image under the pointer, 0–255) and "8-bit"; right column: C, M, Y, K (a simple conversion via `op_color::Cmyk`, in percent) and "8-bit".
- Below the separator: X, Y (the pointer's distance from the rulers' origin, see `rulers.md`); W, H (the width and height of the selection's bounding rectangle, empty when there is no selection). All four are in the rulers' unit (`unit_value`): whole pixels (rounded down), inches to three decimals, centimeters and picas to two, millimeters, points and percent to one (Photoshop's decimal places per unit have not been measured).
- Color samplers (`DocState::color_samplers`): "#1", "#2", … each with R, G, B of the merged image at the sampler (averaged over the Color Sampler's Sample Size), two to a row as in Photoshop (odd numbers on the left, even on the right), each row 3 lines plus an 8 pt gap (`SAMPLER_ROW`), below X/Y/W/H. The panel grows by a row's height for every two samplers (`sampler_rows`, see `floating.md`).
- At the bottom, "Doc: flattened size/layered size", the same figures and format as the status bar's Document Sizes (`status_info::document_sizes` and `size`, see `status_info.md`).
- When the pointer is not over the document, color and position are empty. The pointer position comes from `DocState::pointer` (updated by the document view every frame), and color sampling uses `DocState::sample_average` (sample size 1, all layers).

## Known limitations

- CMYK is not converted using a color profile; there are no panel options (sample colors, ruler units, status information) and no tool hints. The panel keeps OpenPhoto's simpler layout: Photoshop 2026 draws section icons and dividers between the readouts, which are not reproduced yet.

## Test coverage

- `lengths_in_the_rulers_unit`: 36.6 px reads 36 in pixels; at 72 ppi 36 px are 0.500 in and 1.27 cm; 50 px are 25.0% of a 200 px width and 50.0% of a 100 px height. (The size format is covered by `status_info::tests::sizes_read_like_photoshops`.)
