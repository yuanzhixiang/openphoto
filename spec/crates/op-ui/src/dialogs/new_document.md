# dialogs/new_document.rs: The New Document dialog for File › New

## Component responsibilities

File › New... (⌘N): Photoshop 2026's default new-style New Document dialog. On the left are blank document presets paged by category (plus recently used and saved presets); on the right are the details of the selected preset, which can be modified before creating. On confirm, `actions::create_document` creates the document (see "New" in `actions.md`), and `lib.rs` puts this set of settings into the Recent list. See `document_presets.md` for the preset data and card icons.

## Layout

Measured point by point from a 2x screenshot of Photoshop 2026; the dialog is 1080 × 718 pt, and coordinates are pt from the dialog's top-left corner:

- **Window**: 10 pt rounded corners, with a drop shadow. Title bar 0–28 (`#d3d4d5`, with a 0.5 pt `#0c0c0c` line along the bottom), centered system bold 13 pt "New Document", three traffic lights at the top left (centers x 14, 34, 54, y 14, radius 6: red, gray (disabled minimize), green). The traffic lights are appearance only.
- **Tab bar** 28.5–76 (`#323232`, with a `#3e3e3e` strip at the bottom 74–76): Recent (clock icon center 35, text from 49.5), Saved (112.5), Photo (172), Print (230), Art & Illustration (280), Web (398), Mobile (448.5), Film & Video (513), 14 pt, center y 51. The current page has white text with a 2 pt white line drawn below it (for Recent, starting at the clock's left edge 27); other pages are `#a8a8a8` and turn white on hover.
- **Left list** (0–770, from y 76, `#252525`): scrollable, with the template search bar below at 655–703.
  - Heading (bold 13 pt) "BLANK DOCUMENT PRESETS" / "YOUR RECENT ITEMS" / "YOUR SAVED PRESETS", followed by "(count)", left end 26.5, center y 103.
  - Cards are 168 pt square, 4 per row, 180.9 pt apart, the first one's top-left corner at (26, 124). The icon center is 62 below the card top; the name (12 pt white text, truncated with "..." when too long) is centered 121.5 below the top; the size line (e.g. "7 x 5 in @ 300 ppi", 12 pt `#c8c8c8`) is 140 below the top. The selected card has a 3 pt `#417ee4` border; a hovered card has background `#323232` and its name turns blue.
  - **Recent page**: welcome box (13, 89)–(741, 262), `#2e2e2e`, with a close X at the top right (720, 110); bold 28 pt "Let’s start something new." centered at (377, 147); below it two lines of 16 pt text, where "document presets" is an underlined blue link that switches to the Photo page when clicked. Once the welcome box is closed it does not reappear (within the current run), and the heading and cards move up to the same position as on the other pages; while it is not closed, the heading is at y 286 and the cards start at y 307.
  - **Saved page**: when there are no saved presets, shows the same kind of box, with the heading "You can always find it here." and description text.
  - **Search bar**: a magnifier, gray italic text "Search for an image or template", an underline, and an outlined "Search" pill (656.5–727, y 664–694.5). Photoshop searches Adobe Stock here; this application has no template service, so it is appearance only. The copy drops the trailing "from Adobe Stock" of Photoshop's original text: Adobe's brand name does not appear in the UI (`SEARCH_PLACEHOLDER`; the test `search_shows_no_adobe_brand` guards this).
- **Right details** (770–1080, `#323232`): labels 12 pt `#c3c3c3`, values 14–15 pt `#e3e3e3`.
  - "PRESET DETAILS" at (791, 100.75); name input box (underline 790–1015, y 149, `#909090`); on the right the save preset icon (a tray with a down arrow, 1034–1063 × 118–144, hover tooltip "Save Preset").
  - Width (791, 168.5): input box (790–862, 180.5–212.5, `#252525` background, 1 pt `#4a4a4a` border, 4 pt rounded corners, border turns blue when focused, value indented 15.5 from the left); unit dropdown (874–1060): Pixels, Inches, Centimeters, Millimeters, Points, Picas.
  - Height (791, 232), Orientation (874.5, 231), Artboards (944, 231): height input box (790–862, 243.5–275.5); two orientation icons, portrait and landscape (875–897 × 247–272, 905.5–930.5 × 250–269.5), with the current orientation on a blue background; clicking the other one swaps width and height. The Artboards checkbox (944–958 × 252–266) is disabled (no artboards).
  - Resolution (791, 292): input box (790–862, 304–336); Pixels/Inch / Pixels/Centimeter dropdown (874–1060).
  - Color Mode (791, 354.5): mode dropdown (790–957.5, 367–399, only RGB Color selectable) and bit depth dropdown (969.5–1060, only 8 bit selectable).
  - Background Contents (791, 415.5): dropdown (790–1016, 428–460): White, Black, Background Color, Transparent (Custom... grayed out); color swatch on the right (1028–1060): white, black, the current background color, or a checkerboard.
  - Advanced Options (chevron + text, center y 478.75, expanded by default, click to collapse): Color Profile (507.5) dropdown (519.5–551.5): Working RGB: sRGB IEC61966-2.1 (Don't Color Manage grayed out); Pixel Aspect Ratio (568) dropdown (580.5–612.5): Square Pixels.
  - Close (900–972, 658–690, a pill with a 2 pt `#e3e3e3` border) and Create (988–1060, a `#3671df` blue pill), both bold 15 pt white text. Create is grayed out when width or height is invalid.

Compared side by side with Photoshop screenshots, all element positions match; text differs slightly because Source Sans 3 approximates Adobe Clean.

## Interaction

- Opens on the Recent page. Initial details: if the clipboard holds an image, its size (pixels, 72 ppi); otherwise the first Recent item (with its card selected); if neither exists, 1920 × 1080 pixels, 72 ppi. The name is the next "Untitled-N", background White.
- Switching tabs selects and applies that page's first preset, as in Photoshop (an empty page leaves things unchanged). Single-clicking a card: selects it and fills its width, height, unit and resolution into the right side (the name is unchanged). Double-clicking a card: creates directly with that preset.
- Changing the unit converts width and height into the new unit while keeping the represented pixel count (pixels are rounded to integers, other units to at most 3 decimal places). Changing the resolution unit converts the value accordingly.
- Save icon: saves the current settings, named by the name field, into the Saved page (collected into `AppState::new_document_saved` by `lib.rs`, and kept across launches with Recent).
- Create or Enter: creates; width and height are converted to pixels and rounded, and must be between 1–30000, resolution 1–10000 ppi, otherwise nothing is created. When the name is empty, "Untitled" is used. Close or Esc: cancels.
- After creating, these settings (named "Custom") are put at the front of the Recent list; identical settings are not duplicated, up to 20 entries.

## Known limitations

- Recent and Saved are kept across launches in OpenPhoto's own file (`document_presets.md`), not Photoshop's; the welcome box comes back each launch.
- No Adobe Stock templates or search; card icons are approximate shapes drawn from the preset's shape, not Photoshop's icon assets.
- Color mode, bit depth, color profile and pixel aspect ratio each have only one selectable option; no artboards; Background Contents has no Custom.
- In Photoshop the Print page has 14 presets, Mobile 28 and Film & Video 25; here the ones visible in the screenshots are listed (12, 24, 24).

## Test coverage

- `defaults_units_and_validation`: default 1920 × 1080; sizing from the clipboard; after switching to inches the value is 8.889 while the pixels are unchanged; applying the 7 × 5 in @ 300 ppi preset gives 2100 × 1500; a width of 0 is invalid; when there is a recent item, starts from it with it selected.
- `search_shows_no_adobe_brand`: the search bar placeholder text contains neither "Adobe" nor "Stock".
- `new_document_dialog` in `ui_tests.rs` (⌘N then Enter creates Untitled-2, 1920 × 1080; a transparent background yields "Layer 1") and `new_document_dialog_presets_recent_and_saved`.
