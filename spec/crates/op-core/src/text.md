# text.rs: text rasterization

## Responsibilities

Rasterizes text typed with the Type tool into pixels: uses `skrifa` to get glyph outlines from the font file, flattens curves into line segments, and fills them with anti-aliasing using the nonzero winding rule to produce coverage values; the coverage values are then used as alpha to create a text layer. Records no history.

## Public interface

- `coverage(font_data, text, size, origin, width, height)`: the text's coverage values on a `width` × `height` canvas (0–255 per pixel). `size` is the font size in pixels (em size), and `origin` is the start of the first line's baseline; `\n` breaks lines, the line spacing is the font's ascent − descent + line gap (1.2 times the font size when absent), and characters not in the font are skipped. Returns `FontError` when the font data cannot be parsed.
- `caret(font_data, text, size, origin)`: the position of the insertion point after the last character (on the last line's baseline) and the line height, for the interface to draw the cursor.
- `add_text_layer(doc, font_data, text, size, origin, color)`: creates a new layer inserted above the current layer and selects it: the color is `color` and the alpha is the coverage value; the layer name is the first line of the text (at most 30 characters), consistent with Photoshop naming type layers after their content. When the text has no visible pixels (for example only spaces), no layer is added and `Ok(None)` is returned.

## Rasterization rules

- Glyphs are laid out one after another by their advance widths, without kerning or complex text shaping.
- Quadratic curves are flattened into 8 segments and cubic curves into 12; font coordinates have y pointing up, canvas coordinates have y pointing down.
- Each pixel takes 4 sample rows; on each row the interior spans are found with the nonzero winding rule, and coverage is accumulated by the exact horizontal overlap of the spans with the pixel.

## Known limitations

- Text layers are raster layers, and the text can no longer be edited (Photoshop's type layers can be).
- There are no kerning, tracking, or leading settings, no paragraph text (text boxes), vertical text, or text warping, and no support for text that requires shaping (Arabic, Indic scripts, etc.) or fallback for characters missing from the font.

## Test coverage

- `letters_cover_pixels_above_the_baseline`: the vertical stroke of "I" is solid between the baseline and the cap height, and empty below the baseline and above the top of the letter.
- `holes_stay_open_and_lines_stack`: the middle of "O" is empty; the second line is one line height below the first.
- `text_layers_are_named_after_the_text`: the layer for two lines of text is named after the first line; no layer is added for only spaces; an invalid font returns an error.
