# dialogs/save_options.rs: JPEG Options and PNG Format Options

## Component Responsibilities

Photoshop asks for a format's options after the save panel. This dialog does the same:

- **JPEG Options** for `.jpg` / `.jpeg`: Matte, Quality and the Format Options.
- **PNG Format Options** for `.png`: File Size.

The dialog edits an `op_io::ExportOptions` (see `crates/op-io/src/lib.md`) and returns it on OK. `actions.md` describes the waiting save (`PendingSave`) and what OK and Cancel do. The dialog opens with the options last used (`AppState::export_options`), so a new session starts from `ExportOptions::default()`.

## Layout

Both dialogs are centered. The arrangements follow Photoshop's but have **not been measured against Photoshop 2026**, so the positions below are provisional (pt from the dialog's top-left corner). OK and Cancel are 70 pt wide, 88 pt from the right edge, at y 38.5 and 73.5.

### JPEG Options (440 × 318)

- **Matte:** "Matte:" right-aligned at x 70, then a pop-up (76, 46)–(200, 67) offering None, Foreground, Background, White, Black and 50% Gray.
  - None flattens transparency onto white, as JPEG has to.
  - Foreground and Background use the current colors.
  - On open, the stored matte color picks the entry: white → None, black → Black, 128 gray → 50% Gray, then the foreground color, then the background color.
- **"Image Options" group** (18, 90)–(340, 176):
  - "Quality:" right-aligned at x 84.
  - A field (90, 107)–(130, 126) taking 0–12. It takes the focus with its text selected when the dialog opens. OK is disabled while the field is out of range.
  - A pop-up (140, 106)–(240, 127) naming the step the quality falls in: Low 0–4, Medium 5–7, High 8–9, Maximum 10–12. Choosing a step sets the quality to 3, 5, 8 or 10.
  - A slider track from x 34 to 324 at y 142, with a white pin at quality / 12. Clicking or dragging sets the quality.
  - "small file" and "large file" in 10 pt under the slider's ends.
- **"Format Options" group** (18, 196)–(340, 306):
  - Radio buttons at x 36 and y 219, 243, 267: Baseline ("Standard"), Baseline Optimized, Progressive.
  - "Scans:" with a pop-up (90, 281)–(150, 302) offering 3, 4 and 5. It is enabled only with Progressive and shown dimmed otherwise.

### PNG Format Options (400 × 150)

- **"File Size" group** (18, 45)–(330, 138), with radio buttons at x 36 and y 68, 92, 116:
  - Large file size (fastest saving)
  - Medium file size
  - Smallest file size (slowest saving)

## Interactions

- Enter (when no field is being typed in) or OK returns the options.
- Escape or Cancel returns nothing, and nothing is written.

## Known Limitations

- The layouts, the defaults (quality 12, None, Large file size) and the quality-to-encoder table are provisional (not measured).
- Baseline Optimized and Progressive are written as baseline JPEGs: the encoder can't write the others.
- Photoshop's live file-size readout and Preview checkbox are not shown.

## Test Coverage

- `quality_names_and_mattes`: steps by quality, and the stored matte picking Background.
- `ui_tests::jpeg_and_png_saves_ask_for_their_options`:
  - A JPEG save waits for its options. Choosing Low and pressing OK writes it with quality 3.
  - A PNG save of the one-layer document makes the PNG its file.
  - Closing it with changes and choosing Save asks for the PNG options, then closes.
  - Screenshots `jpeg_options.png`, `png_options.png`.
