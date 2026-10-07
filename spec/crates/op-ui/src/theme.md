# theme.rs: visual specification

## Responsibilities

Centrally defines colors, sizes, fonts, and egui styles, with values taken from Photoshop's default "medium gray" interface. For the sizing system and conversion rules, see the directory's `README.md`.

## Colors (`color`)

| Name | Value | Use |
|---|---|---|
| `PANEL` | `#535353` | Background of panels, toolbar, and options bar; active tab |
| `PASTEBOARD` | `#282828` | Pasteboard outside the canvas |
| `TAB_BAR` | `#424242` | Tab bar |
| `TAB_INACTIVE` | `#4c4c4c` | Document tab hover |
| `FIELD` / `FIELD_BORDER` | `#454545` / `#3a3a3a` | Input fields |
| `TOOL_ACTIVE` | `#383838` | Current tool, pressed state |
| `HOVER` | `#5e5e5e` | Hover |
| `LIST_BG` / `ROW_SELECTED` | `#4d4d4d` / `#6b6b6b` | List background, selected row |
| `SEPARATOR` / `SEPARATOR_LIGHT` | `#393939` / `#444444` | Dark and light dividers |
| `TEXT` / `TEXT_DIM` / `TEXT_DISABLED` | `#eaeaea` / `#bcbcbc` / `#7c7c7c` | Body text, secondary text, disabled text |
| `ICON` | `#d8d8d8` | Icons |
| `ACCENT` | `#2c8be8` | Accent color such as text selection |

## Sizes

- `size`: window frame sizes are measured 1:1 from Photoshop (pt): title bar 29, options bar 33, toolbar 42, status bar 16, icon column 43, panel column 322, panel tab bar 28 (including the 1 pt line below). The rest (input field height 26, layer row height 60) are still values measured in reference screenshot pixels.
- `UI_SCALE = 0.675`: scales reference screenshot pixels to Photoshop's actual size through egui's `zoom_factor`. The value is computed from "a reference screenshot 2000 px wide corresponds to a Photoshop window of 1349 pt".
- `pt(x)`: converts a point value measured 1:1 in Photoshop into egui units (`x / UI_SCALE`); all new interface code uses it.

## Fonts

- Body text 11.5 pt, small text 10.5 pt, so that the cap height matches Photoshop's panel text (8 pt). Source Sans 3 is narrower than Photoshop's Adobe Clean; the same word is about 15% narrower. Icons are 20 (reference screenshot pixels).
- The font files are bundled with the application, and their byte data is exposed as `SOURCE_SANS_REGULAR` and `SOURCE_SANS_SEMIBOLD`; the Type tool also uses them to rasterize text.
- The proportional font family prefers Source Sans 3 Regular; there is also a font family named `semibold` (Source Sans 3 Semibold), used for labels and titles.
- The Phosphor icon font is added as a fallback font at the end of the proportional font family, so icons can be written directly in text.
- Divider and collapse bar colors (measured in Photoshop 2026): the lines of 3 pt dividers and collapse bars `DIVIDER_DARK` `#383838`, the line in the middle of dividers `DIVIDER_LIGHT` `#474747`, collapse bar background `COLLAPSE_BAR` `#424242`, collapse chevron `COLLAPSE_CHEVRON` `#c8c8c8`; panel tab text `TAB_TEXT_ACTIVE` `#f0f0f0`, `TAB_TEXT` `#b0b0b0`.
- Options bar colors (measured in Photoshop 2026): background `OPTIONS_BAR` `#535353`, divider `OPTIONS_SEPARATOR` `#3e3e3e`, icon `OPTIONS_ICON` `#dddddd`, bell `OPTIONS_BELL` `#b9b9b9`, disabled icon `OPTIONS_ICON_DISABLED` `#989898`, label text `TEXT_BRIGHT` `#f0f0f0`; checkbox `CHECKBOX` `#d4d4d4` and check mark `CHECK_MARK` `#323232`; dropdown border `DROPDOWN_BORDER` `#666666` (hover `DROPDOWN_BORDER_HOVER` `#808080`).
- Dialog fonts: Photoshop 2026 dialog text comes in two kinds, and the right one must be chosen per dialog (checked one by one against Photoshop screenshots by text width and glyph shapes):
  - **Drawn by AppKit** (the classic dialogs Duplicate Layer and Image Size, macOS alerts, all window titles): the macOS system font SF. It is read at runtime from `/System/Library/Fonts/SFNS.ttf` (not distributed with the application) and registered as a variable font as `dialog` (wght 400) and `dialog-bold` (700), both with `opsz` = 17: AppKit uses the Text optical size for text below 17 pt, which is about 10% wider than the font's default Display (28); without it the text is too narrow. The interface fonts follow in order as fallbacks; when it cannot be read (not macOS) there is only Source Sans 3. Body text 12 pt (`dialog(pt(12.0))`), titles 13 pt bold, alerts 13 pt.
  - **UXP (Spectrum) dialogs** (New Layer, New Group, Layer from Background): Spectrum's Adobe Clean (distinguishable by the slanted top of t and the straight tail of y), the same font as the panels, so Source Sans 3 is used: `uxp(size)` is Regular and `uxp_bold(size)` is Semibold, both 12 pt.
- Tracking: AppKit adjusts tracking by font size according to SF's `trak` table (0 at 12 pt, −12/2048 em at 13 pt, −22 at 14 pt); egui does not. `system_tracking(size)` interpolates the table to give egui's extra letter spacing, and `tracked_galley(painter, text, font, color)` lays out text with it; window titles and alert text are all drawn this way (12 pt body text does not need it).
- Test `dialog_text_is_as_wide_as_photoshops`: the widths at 2x of nine text runs (from Image Size, Duplicate Layer, window titles, alerts, New Layer) differ from Photoshop measurements by no more than 3%; removing opsz 17, removing tracking, or using the wrong font for UXP dialogs all make it fail (verified).
- Phosphor Regular is additionally registered on its own as the font family `phosphor-regular` (with no text font in front of it); `tool_icon(size)` returns it, and it is used only for icons in the toolbar and tool lists: at toolbar size Regular has strokes of about 1 pt, matching Photoshop's icons; the Bold used previously was about half again as thick. The icon glyphs themselves still differ from Photoshop (see the P0 gaps in the README).

## egui style (`apply_style`)

- Always uses the dark theme and sets `zoom_factor = UI_SCALE`.
- Disables egui's built-in ⌘+/⌘- shortcuts for zooming the whole interface; these two keys are reserved for canvas zoom.
- The background, border, corner radius (3), and hover and pressed colors of widgets are all changed to Photoshop's color scheme; widgets do not grow on hover or press.
- Disables the fade at the edges of scroll areas (egui 0.36 by default draws a 20 pt gradient on the side that can keep scrolling; Photoshop's lists do not have one). Panels with a custom scroll style (History) must also turn it off separately, because `ScrollStyle::solid()` comes with the default fade.
