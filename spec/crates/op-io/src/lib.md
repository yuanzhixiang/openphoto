# op-io: file reading and writing

## Responsibilities

Converts between disk files and `op_core::Document`: opens Photoshop documents (PSD, see `psd.md`) and bitmap files as documents, saves documents as PSD, and exports a document's composite as a bitmap file. All bitmap encoding and decoding is delegated to the `image` crate (0.25, with only the six format features png, jpeg, webp, tiff, bmp, and gif enabled).

## Public interface

### `IoError`

| Variant | Meaning | Display text |
|---|---|---|
| `Read(ImageError)` | Decoding failed when opening (including errors reading the file) | `could not read image: …` |
| `Write(ImageError)` | Encoding failed when exporting | `could not write image: …` |
| `File(io::Error)` | Writing the file failed when exporting | `could not save file: …` |
| `Unsupported(String)` | Unsupported extension; carries the extension converted to lowercase | `unsupported file format: …` |
| `Psd(String)` | The PSD file is malformed or uses an unsupported feature | `could not read Photoshop document: …` |

`File(io::Error)` is also used for errors reading the file when opening a PSD.

### `OPEN_EXTENSIONS`

The extensions offered by the open dialog: `psd`, `png`, `jpg`, `jpeg`, `webp`, `tif`, `tiff`, `bmp`, `gif`.

### `SAVE_FORMATS`

The formats offered by the Save As dialog (name and extensions), in the same order as in the dialog, the first being the default format: Photoshop (`psd`), PNG (`png`), JPEG (`jpg`, `jpeg`).

### `open(path)`

1. Takes the path's extension and converts it to ASCII lowercase; when there is no extension, it is treated as an empty string.
2. When the extension is not in `OPEN_EXTENSIONS`, returns `Unsupported` without reading the file.
3. `psd` is handed to `psd::read` (keeping layers); everything else is decoded with `image::open` and uniformly converted to 8-bit RGBA (`into_rgba8`).
4. The document title is the file name (including the extension); when the file name cannot be obtained, it is `"Untitled"`.
5. Bitmaps build the document through `Document::from_rgba8`, at 72 ppi, RGB, 8-bit. The layer rules match Photoshop: when the image is fully opaque, it is a locked "Background" background layer; as soon as any pixel is not fully opaque, it is a regular layer named "Layer 0" (see `crates/op-core/src/document.md`).

### `save(doc, path)` and `save_with(doc, path, options)`

File › Save / Save As writes the file:

- When the extension is `psd` (case-insensitive), the result of `psd::write` is written, keeping the layers.
- Other extensions are handed to `export_with`, which writes the flattened composite image.
- `save` uses `ExportOptions::default()`.
- The PSD is likewise generated in memory first and then written in one go.

### `ExportOptions`

The JPEG Options and PNG Format Options of a save (`dialogs/save_options.md` in `op-ui`):

- `jpeg_quality`: Photoshop's 0–12. It defaults to 12.
- `matte`: what JPEG's transparent pixels are flattened onto. It defaults to white.
- `png`: `PngSize::Large` (the default), `Medium` or `Smallest`. These map to the PNG encoder's fast, default and best compression, with adaptive filtering. The pixels are the same at every size.
- `jpeg_quality(q)`: the encoder quality (1–100) for Photoshop's 0–12, from the table 30, 38, 46, 55, 62, 68, 75, 80, 85, 89, 93, 96, 98. This is an approximation of Photoshop's own quantization. Qualities above 12 count as 12.

### `export_composite(doc, path)` and `export_with(doc, path, options)`

`export_composite` is `export_with` with the default options.

1. Determines the format from the extension (`ImageFormat::from_path`). When it cannot be recognized, returns `Unsupported` without creating a file.
2. Takes the composite result of `Document::composite_rgba8`.
3. JPEG has no transparency channel: the composite is first laid onto the options' matte (white by default, matching Photoshop Export As's default matte for JPG) by alpha, in the same gamma-encoded space as compositing, converted to RGB, and encoded at `jpeg_quality(options.jpeg_quality)`. PNG is encoded as RGBA with the options' compression. Other formats (WebP, TIFF, BMP, GIF) are encoded directly as RGBA, keeping transparency.
4. Encoding is completed in memory first, and the file is written only after it succeeds. When encoding fails, the target file is not created or modified.

## Behavior rules and edge cases

- Extension matching is case-insensitive (`.PNG` can be opened).
- The actual decoding format is chosen by `image::open` from the extension rather than by sniffing the file contents; a file whose extension does not match its real format fails to decode and returns a `Read` error.
- Opening a bitmap discards the source file's bit depth, color space/ICC profile, and resolution metadata: 16-bit images are reduced to 8-bit, and the resolution is always 72 ppi. PSD reads the resolution.
- An image with an alpha channel whose pixels are all fully opaque (for example an opaque RGBA PNG) is treated as opaque and opens as a background layer.
- For multi-frame formats such as GIF, only the first frame is taken.
- `RgbaImage::from_raw` inside `export_composite` relies on the composite buffer length being equal to `width * height * 4`; this invariant is guaranteed by `Document`, and it panics when not met.

## Relationship to other modules

- Depends on `op-core` (`Document`), `image`, and `thiserror`.
- `op-ui`'s open flow uses `OPEN_EXTENSIONS` to set the file dialog filter and calls `open`, and on success uses `"Open"` as the first history state; the save flow uses `SAVE_FORMATS` to set the filter and calls `save`; the export flow (the dialog offers two filters, PNG and JPEG) calls `export_composite`, and on failure shows the error text in an alert.

## Known limitations

- For PSD limitations see `psd.md`; TIFF does not save layers.
- JPEG is always baseline (the encoder writes neither optimized nor progressive JPEGs), and the quality table is approximate.
- No metadata (EXIF, ICC, resolution) is read or written.

## Test coverage

- `jpeg_quality_and_matte_and_png_sizes`: a lower quality makes a smaller file, a black matte darkens transparent pixels, every PNG size reads back the same pixels, and the quality table's ends.
- `jpeg_export_flattens_onto_white`: a semi-transparent document exported as JPEG can be read back; opaque pixels keep their original color and transparent pixels become white (JPEG is lossy, so comparison uses a tolerance).
- `png_export_keeps_transparency`: exporting PNG keeps transparency.
- `unknown_extension_writes_nothing`: an unknown extension returns `Unsupported` and creates no file.
- `transparent_png_opens_as_regular_layer`: a PNG with semi-transparent pixels opens as a regular "Layer 0" layer.
- `flatten_blends_alpha`: 50% transparent black laid onto white gives `(127, 127, 127)`.
