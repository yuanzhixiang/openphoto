# status_info.rs: What the status bar shows

## Responsibilities

The items of the menu behind the status bar's arrow, in Photoshop 2026's order, and the text the status bar shows for each. Labels and formats are Photoshop's own (its `$$$/Menu/Status/…` and `$$$/StatusBox/…` strings); the texts were compared one by one with Photoshop's status bar on the same 64 × 72 document.

## Items (`StatusInfo`)

| Item | Text |
| --- | --- |
| Document Sizes | `Doc: 13.5K/13.5K`: the flattened image's size (8-bit RGB, 3 bytes per pixel), then the layered document's estimate (the flattened size plus 4 bytes per non-transparent-bounds pixel of every layer after the first) |
| Document Profile | `Untagged RGB (8bpc)` (documents carry no color profile yet) |
| Document Dimensions (default) | `64 px x 72 px (72 ppi)` |
| GPU Mode | `Metal` |
| Compositing Mode | `Classic` |
| Measurement Scale | `1 pixel(s) = 1.0000 pixels` |
| Scratch Sizes | `Scratch: used/available`: the layered sizes of all open documents, and the free space on the disk holding the temporary directory (`statvfs`; 0 bytes elsewhere than Unix) |
| Efficiency | `Efficiency: 100%` (nothing is paged out) |
| Timing | `2.2s (100%)`: how long the last command that changed the active document took (`AppState::last_timing`, measured in `commands::run`) |
| Current Tool | the tool's name without " Tool" (`Move`) |
| 32-bit Exposure | `Exposure works in 32-bit only` |
| Save Progress, Download Progress | no text: an empty progress track (see `document_view.md`) |
| Smart Objects | `Missing: 0 / Changed: 0` |
| Layer Count | `1 Layer`, `3 Layers`, plus `, 1 Group` / `, 2 Groups` (groups are not counted as layers) |

`size` writes byte counts as Photoshop does: three significant digits with K, M or G (`13.5K`, `1.70M`, `82.9G`, `300K`), plain bytes below 1K (`0 bytes`).

## Known limitations

- The layered document size is an estimate; Photoshop's own figure includes its layer bookkeeping.
- Timing covers commands run from menus and shortcuts, not tool strokes.
- Save and Download Progress never show progress (saving is synchronous; nothing is downloaded).

## Test coverage

- `status_info::tests::sizes_read_like_photoshops`: the byte formats against Photoshop's readings.
- `ui_tests::status_bar_menu_picks_what_it_shows`: see `ui_tests.md`.
