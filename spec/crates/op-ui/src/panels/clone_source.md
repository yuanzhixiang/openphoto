# panels/clone_source.rs: Clone Source panel

## Responsibilities

The Clone Source panel (Window › Clone Source, or the Clone Stamp's options bar toggle). It holds:

- the active document's five clone sources;
- the active source's offset;
- the scale (W, H) and angle the Clone Stamp clones with;
- the overlay options.

It is a floating panel (`floating.md`).

## Data

- **`ClonePanel`** (`AppState::clone_panel`, shared by the documents):
  - `scale`: W and H in percent, 100% by default, linked by default (`linked`).
  - `angle`: degrees, counterclockwise.
  - The overlay options: `show_overlay` (on), `opacity` (100%), `clipped` (on), `auto_hide` (off), `invert` (off).
  - `transform(origin)` gives the stroke's `op_core::paint::SourceTransform` about the source point, or None when W and H are 100% and the angle is 0.
- **Source slots** (`DocState::clone_slots`, `clone_slot`): five (source point, offset) pairs. The pair in use is `clone_source` / `clone_offset`. `choose_clone_slot(k)` puts the current pair back in its slot and takes slot k's.

## Layout and interactions

The panel is built from egui's own widgets, not rebuilt from Photoshop's. From top to bottom:

- **The five source buttons**, "◉n" when set and "○n" when empty. The one in use is highlighted, and clicking another switches to it.
- **Offset:** X and Y of the active source. They are enabled once a source is set and edit `clone_offset`.
- **W and H** (1–10000%) with the link toggle between them. While linked, editing one sets the other.
- **The angle**, −360° to 360°, and **Reset Transform**, which returns W and H to 100% and the angle to 0.
- **Show Overlay**, and under it Opacity, Clipped, Auto Hide and Invert.

The overlay is drawn by `document_view.md` (`clone_overlay`).

## Known limitations

- The panel is not laid out like Photoshop's: its icons, fields and the Frame Offset / Lock Frame rows (video) are missing.
- Sources belong to the document they were set in; Photoshop's can come from another open document.
- The overlay's blend mode is not offered.
- The scale and angle apply to the Clone Stamp but not to the Healing Brush.

## Test coverage

- `transform_only_when_scaled_or_turned`.
- `ui_tests::clone_source_panel_scales_and_keeps_sources`:
  - the options bar toggle opens the panel;
  - at 200% a one-pixel column clones two pixels wide;
  - a second slot starts empty and switching back restores the first source;
  - screenshot `clone_source_panel.png`.
