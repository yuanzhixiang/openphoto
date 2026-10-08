# dialogs/adjust/match_color.rs: Match Color

## Responsibility

Image › Adjustments › Match Color..., drawn as Photoshop 2026's dialog as part of `AdjustDialog` (`adjust.md`). Its settings (`Kind::params`): 0 Luminance and 1 Color Intensity (1–200, 100), 2 Fade (0–100), 3 Neutralize, 4 Source (None, then the other open documents and loaded statistics), 5 Layer (the source's layers, Merged first), 6 Ignore Selection when Applying Adjustment, 7 Use Selection in Source to Calculate Colors, 8 Use Selection in Target to Calculate Adjustment.

## Data

`commands.rs` fills `Extra` when the dialog opens (`set_match_sources`):

- the active layer's Lab statistics, and those of its pixels at least half selected when the document has a selection (`target`, `target_selected`), the document's title and the layer's name;
- for every other open document a `MatchSource`: its title, the statistics of its merged image and of each layer (all pixels, and the selected ones when it has a selection), and its merged image made small (at most 200 × 200);
- the active document made small (`Extra::thumb`).

Statistics are sampled from at most about 20 000 pixels.

## Layout

Measured from a 2x capture of Photoshop 2026's dialog: 481 × 559 pt, points from the dialog's top-left corner.

- **Destination Image** group (10.75, 46.75)–(381.25, 318.75), titled at x 31: "Target:" at x 24.5 on y 69.25 and "<title> (RGB/8)" at 71.5; Ignore Selection when Applying Adjustment, its 12 pt box at (68, 88), the label 10 pt after.
- **Image Options** group (20.75, 119.75)–(371.75, 308.75), titled at x 40.5: Luminance, Color Intensity and Fade labels at x 68.5 on y 143.5, 192.5, 241.5; fields 242.5–286 (Fade's focused on open); classic tracks 78.5–275, 16.5 pt below each line; Neutralize, its box at (68, 285).
- **Image Statistics** group (10.75, 336.75)–(381.25, 548), titled at x 31: the two selection checkboxes with boxes at (68, 355) and (68, 389); "Source:" and "Layer:" right-aligned at 61.5 on y 431 and 460 with pop-ups 68–213.5, 18 pt tall; Load Statistics... and Save Statistics... flat buttons 67.5–191.5 at y 477.5–503 and 512.5–538; the thumbnail fitted in (259, 429)–(358.5, 528.5) with a 1 pt black frame.
- Group titles break the top edge 6 pt before and 5 pt after them.
- **OK** (411.5, 45)–(471, 70.5), the default (off while the settings don't make an effect), **Cancel** (411.5, 80)–(471, 105.5), **Preview** with its box at (392, 125).

Compared side by side with the capture (`more_filters_and_adjustments_apply`'s `match_color.png`), the groups, rows, pop-ups and buttons land within about a point.

## Behavior

- Ignore Selection and Use Selection in Target are usable only when the document has a selection; Use Selection in Source only when the chosen source has one; Layer only when a source is chosen. Off, they are dimmed (and count as unchecked).
- The effect is `Adjustment::MatchColor` from the target statistics (inside the selection with Use Selection in Target) to the source's (the chosen layer's, inside its selection with Use Selection in Source; with Source None, the target's own); Neutralize zeroes the source's a and b means; Ignore Selection changes the whole layer.
- Choosing a source starts its Layer at Merged. The thumbnail shows the chosen source's merged image (the target's with None).
- Load Statistics... reads a statistics file and adds it as a source, chosen; Save Statistics... writes the chosen source's statistics (the target's with None). The files are OpenPhoto's own (`preset_files::MATCH_STATISTICS`, `.sta`, `preset_files.md`).

## Known limitations

- Statistics files don't follow Photoshop's own format.
- Source thumbnails are of the merged image, whatever layer is chosen.
