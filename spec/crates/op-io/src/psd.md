# psd.rs: Photoshop Document Reading and Writing

## Responsibilities

Reads and writes Photoshop documents (.psd), implemented according to Adobe's "Photoshop File Formats Specification", supporting 8-bit RGB pixel layers. No external dependencies.

## Writing (`write(doc)`)

Produces the bytes of the whole file:

1. File header: `8BPS`, version 1, channel count (3 when the composite is fully opaque, otherwise 4), height, width, bit depth 8, color mode 3 (RGB).
2. Color mode data: empty.
3. Image resources: only ResolutionInfo (ID 1005); horizontal and vertical resolution both take `doc.resolution` (16.16 fixed point), in pixels/inch.
4. Layer and mask information:
   - When the document has only a background layer, no layers are written, only the composite—the same way Photoshop saves a document with a single background.
   - Otherwise all layers are written (bottom to top). When the composite has transparency, the layer count is written as a negative number (meaning the composite's alpha channel is the merged transparency).
   - Layer groups: going from bottom to top, a divider record named "</Layer group>" (`lsct` type 3) is written before each group's bottommost layer (or the group itself for an empty group); the group itself is written as a record without pixels, with `lsct` 1 (expanded) or 2 (collapsed) plus "8BIM" and the group's blend mode key (`pass` for Pass Through); when nested, the outer group's divider record comes first. Verified by opening in Photoshop 2026: the nested structure, group opacity, and blend mode; files with nested groups saved by Photoshop are also read back correctly.
   - Each layer record: the layer bounds are the bounding rectangle of the non-transparent pixels (including those outside the canvas), which may be negative or extend past the canvas (a fully transparent layer has empty bounds); channels (−1 = alpha, 0/1/2 = R/G/B; as in Photoshop, the background layer has no alpha channel and only 0/1/2 are written); the blend mode key (`norm`, `mul `, `scrn`, etc., 27 in all, mapping one-to-one to `BlendMode`); opacity (0–255); flag bit 0 "transparency protected" (set for the background layer or when transparent pixels are locked), bit 1 "hidden", bit 3 (indicates bit 4 is valid, always set since Photoshop 5); the Pascal-format layer name (non-ASCII characters written as "?", padded to a multiple of 4); the `luni` additional info (the full name in UTF-16); the `iOpa` additional info (fill opacity); `lclr` when there is a color label (a 2-byte number plus 6 bytes of 0); layer links in image resource 1026 (Layer Group Information: one u16 per layer record, including group end records, bottom to top; layers with the same number are linked together, 0 is unlinked; renumbered 1, 2… in order of first appearance; not written when there are no links); `lspf` when a non-background layer has locks (4 bytes: bit 0 transparency, bit 1 pixels, bit 2 position, bit 3 prevent auto-nesting, bit 31 lock all; checked against files saved by Photoshop 2026); the background layer also has `lnsr` = `bgnd` (name source). All of these match files saved by Photoshop 2026 (checked field by field against PSDs re-saved by Photoshop).
   - When there is a layer mask, an extra channel −2 is written (mask values covering the whole canvas), and 20 bytes are written to the "layer mask data" of the extra data: the mask rectangle (the whole canvas), the default value 255 outside the rectangle, flags (bit 1 means the mask is disabled), and 2 bytes of padding.
   - Channel data is written after the records, layer by layer and channel by channel, all compressed with PackBits (RLE): first the compressed byte count of each row (u16), then each row's data.
5. Composite image data: the result of `Document::composite_rgba8`, RLE-compressed, in R, G, B (, A) plane order.

PackBits encoding: runs of 3 or more identical bytes are encoded as repeat segments (at most 128); the rest are literal segments (at most 128).

## Reading (`read(data, title)`)

1. Verifies the `8BPS` signature and version 1 (version 2 PSB is not supported); only 8-bit RGB is accepted, otherwise `IoError::Psd` is returned.
2. Skips the color mode data; reads ResolutionInfo's horizontal resolution from the image resources as the document resolution (72 when absent).
3. Reads layers: the name prefers `luni`, then the Pascal name; `iOpa` is fill opacity; `lclr` is the color label; records with `lsct`/`lsdk` are layer groups: type 3 is the divider record at the bottom of a group (its position is noted when read), types 1/2 are the group itself (expanded/collapsed), and reading one creates a group layer into which all layers since the corresponding divider record that do not yet have a parent group are placed; nested groups are restored with a stack. Channel data supports uncompressed (0), RLE (1), ZIP (2: a zlib stream of the plane) and ZIP with prediction (3: after inflating, each row holds each byte's difference from the one before, added back up); other compressions, or a damaged or short zlib stream, are an error. When layer mask data is present, channel −2 is read and restored to a full-canvas mask using the mask rectangle and default value, and the enabled state is read; −3 (real user mask) and other channels are skipped. Layer pixels are placed on the layer according to the bounds in the record, and parts outside the canvas are kept outside the canvas (matching Photoshop); the background layer is clipped to the canvas.
4. If the bottommost record is "transparency protected" and either has no alpha channel (Photoshop's way of writing it) or is named "Background", it is the background layer; other records take their locks from `lspf`; without `lspf`, this flag bit maps to locking transparent pixels. When `lspf` is present this flag bit is ignored: Photoshop also sets it when pixels are locked, and looking only at it would misread a transparency lock. Visibility, opacity, fill opacity, and blend mode (unknown keys become Normal) are set as recorded. The topmost layer becomes the active layer.
5. When there are no layers, the composite is read (uncompressed, RLE, or ZIP with or without prediction, where one zlib stream holds all the channels' planes in order) and opened according to the rules of `Document::from_rgba8` (fully opaque becomes a background layer).

## Known Limitations

- Not supported: vector masks, adjustment layers, type layers, smart objects, layer styles, clipping masks, channels and paths, and 16/32-bit, grayscale, CMYK, and other modes.
- ICC profiles and other image resources (guides, slices, thumbnails, etc.) are not read or written.
- No thumbnail is written on save; PSD previews in Finder may rely on the system rendering the composite itself.
- Verified: when Photoshop 2026 opens a PSD written by OpenPhoto, layer names, opacity, blend modes, visibility, and the background layer are all correct; when OpenPhoto reads a PSD saved by Photoshop, both the background layer and normal layers are correct. Layer masks have not yet been verified with Photoshop.

## Test Coverage

- `groups_round_trip`: nested groups Outer { Inner { a }, b } and top-level layers are written and read back with order, parent-child relationships, collapsed state, Pass Through mode, and group opacity unchanged. `write_group_sample` (`#[ignore]`) writes a sample to open and check in Photoshop.

- `color_labels_round_trip`: a layer with a Violet label is written and read back with the label unchanged; the background layer has no label. For a file in which Photoshop 2026 set the seven labels Red…Gray in turn and saved, the labels read by this module correspond one-to-one.

- `pixels_outside_the_canvas_round_trip`: a layer has pixels outside the canvas to the left, outside to the bottom right, and inside the canvas; after writing and reading back, the content bounds and the three pixels are unchanged. The written file was verified by opening it in Photoshop 2026: the layer bounds are (20, −5)–(90, 30), matching what was written.

- `zip_channels_with_and_without_prediction`: a 4 × 3 plane stored with compression 2 and with 3 (row deltas) reads back unchanged; a damaged stream is an error.
- `packbits_round_trip`: a row mixing a long repeat run, 256 distinct bytes, and short repeats becomes shorter when compressed and is unchanged after decompression.
- `layers_round_trip`: a background plus one layer with a Unicode name, 50% opacity, 25% fill, Multiply, and hidden; after writing and reading back, all properties and pixels (including semi-transparent pixels and blank areas) match, resolution 300 is kept, and the topmost layer is the active layer.
- `masks_round_trip`: a disabled mask is written and read back with matching values and enabled state.
- `a_lone_background_is_stored_as_the_merged_image`: with only a background, it is written as the composite and reads back as a background layer with matching pixels.
- `rejects_other_files`: non-PSD data returns an error.
- `photoshop_check::write_sample` (`#[ignore]`): writes `target/psd-check/ours.psd` for inspection in Photoshop or other software.
- `photoshop_check::read_photoshop_file` (`#[ignore]`): reads the Photoshop-saved PSD specified by the environment variable `OPENPHOTO_PSD`; the bottommost layer should be the background layer.

### Lock Tests

- `locks_round_trip`: the five lock flags are written and read back unchanged.
- `reads_photoshop_locks`: reads `fixtures/photoshop_locks.psd` (saved by Photoshop 2026, one lock per layer: tp, px, pos, nest, all); each layer reads back only its corresponding lock.
- `reads_photoshop_links`: reads `fixtures/photoshop_links.psd` (saved by Photoshop 2026: A linked with B, C linked with the background, D unlinked); the link relationships are correct; after writing and reading back, the layers linked together are unchanged (the numbers may differ).
