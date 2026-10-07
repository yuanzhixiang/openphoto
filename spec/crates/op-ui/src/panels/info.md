# panels/info.rs: Info panel (F8)

## Component responsibilities

Shows the color under the pointer, the pointer position, the selection size, the color samplers, the status lines and the tool hint, in Photoshop 2026's sections.

## Layout

Measured from a 2x capture of Photoshop 2026's Info panel. Positions are in points from the top-left corner of the panel's body (below its tab bar); the body is 214 pt wide and draws over the floating frame's 10 pt margin (`floating.md`), since the dividers run edge to edge.

- **Text:** the panel's body font in `#d6d6d6`. Lines are 13.25 pt apart. A label's letter and its colon are drawn apart ("R :"): the colon ends at x 45, the letter 4.5 pt before it; values are right-aligned at x 83.5. The second column is the first moved 106 pt right.
- **Readouts (y 0–85):** an eyedropper with a menu triangle centered at (14.25, 33.75) in each column; the readout's lines from y 19.75 (cap-height centers); "8-bit" right-aligned at x 45.5 on the fifth line (y 72.75), whatever the readout's length.
- **Position (y 85–125):** a crosshair with a menu triangle at (14.5, 106.5), X and Y at y 99.75 and 113; a selection-size icon (two arrows round a dotted corner) in the second column, W and H beside it.
- **Color samplers:** one 66 pt section per two samplers (`SAMPLER_ROW`), odd numbers on the left, even on the right. "#1" at x 7 on the first line (11.25 pt below the section's top), an eyedropper under it (24.25 pt down), R, G, B in the column.
- **Status lines:** the section is 28 pt for one line, 13.25 pt more for each further one; text at x 10, the first line 16 pt below the divider.
- **Tool hint:** when Show Tool Hints is on and the tool has a hint (`tool_hint`; only the Color Sampler's "Click image to place new color sampler." / "Use Cmd for additional options." is known), a 46 pt section with its two lines at x 10, from 16.5 pt below the divider.
- **Dividers:** 1 pt. Under the readouts and under the position section `#424242`, inset 1 pt at each end; before the status and hint sections `#3e3e3e`, edge to edge. The column divider (`#424242`, x 106.5) runs from y 3 to 82 in the readouts and from y 83 to the end of the samplers.
- **Height** (`height`): 125 pt, the sampler rows, the status and hint sections, and 8 pt.

## Content

- Left column: the first color readout, right column: the second (Info Panel Options, `panel_options.md`; Actual Color and CMYK Color by default). Readouts (`Readout::lines`) of the composite's color under the pointer: Actual Color and RGB Color as R, G, B (0–255); Web Color as R, G, B in hexadecimal; HSB Color as H (degrees), S and B (percent); Grayscale as K (percent ink of the luminosity); CMYK Color as C, M, Y, K (a simple conversion via `op_color::Cmyk`, in percent); Lab Color as L, a, b; Opacity as Op, the active layer's opacity under the pointer (percent; the only readout that reads the layer rather than the composite).
- **Icon menus:** a click on a readout's eyedropper (an 18 pt square around it) opens its readout menu (Actual Color … Opacity, the current one checked), a native pop-up whose pick becomes that column's readout in Info Panel Options; a click on the position crosshair opens the units (Pixels … Percent), whose pick becomes Mouse Coordinates' unit.
- X, Y (the pointer's distance from the rulers' origin, see `rulers.md`); W, H (the width and height of the selection's bounding rectangle, empty when there is no selection). All four are in Info Panel Options' Mouse Coordinates unit, or the rulers' unit when it is "Rulers' Units" (the default) (`unit_value`): whole pixels (rounded down), inches to three decimals, centimeters and picas to two, millimeters, points and percent to one (Photoshop's decimal places per unit have not been measured).
- Color samplers (`DocState::color_samplers`): R, G, B of the merged image at each sampler, averaged over the Color Sampler's Sample Size.
- The status lines Info Panel Options turns on (Document Sizes by default, Document Dimensions, Current Tool), one a line, in the status bar's wording (`status_info::StatusInfo::text`, e.g. "Doc: flattened size/layered size").
- When the pointer is not over the document, color and position are empty. The pointer position comes from `DocState::pointer` (updated by the document view every frame), and color sampling uses `DocState::sample_average` (sample size 1, all layers).

## Known limitations

- CMYK is not converted using a color profile. Only three of Photoshop's status items are offered.
- The icons are drawn (the eyedropper is Phosphor's glyph), not Photoshop's artwork. The readout menus lack Photoshop's Proof Color and Total Ink, which need color management (P2 #19).
- Tool hints exist only for the Color Sampler; other tools show no hint section.
- The panel can't be resized; its height follows its sections.

## Test coverage

- `ui_tests::info_and_navigator_panel_options`: the panel menu's Panel Options... opens the dialog; HSB and Web readouts with Document Dimensions are kept on OK; Escape drops a Navigator change. Screenshots `info_panel_options.png`, `info_panel_hsb.png`.
- `ui_tests::info_panel_sections`: the height with one row of samplers and Document Sizes, and with the Color Sampler's hint. Screenshot `info_panel.png` (compared side by side with Photoshop's capture).
- `panel_options::tests::opacity_reads_the_alpha`: a half-transparent color reads Op 50%.
- `lengths_in_the_rulers_unit`: 36.6 px reads 36 in pixels; at 72 ppi 36 px are 0.500 in and 1.27 cm; 50 px are 25.0% of a 200 px width and 50.0% of a 100 px height. (The size format is covered by `status_info::tests::sizes_read_like_photoshops`.)
