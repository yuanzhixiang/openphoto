# doc_tabs.rs: Document Tab Bar

## Component Responsibilities

The document tab bar above the canvas, with one tab per open document. Dimensions are measured 1:1 in Photoshop 2026 (pt).

## Visuals

- Bar height 30 (including a 1 pt `#363636` divider at the bottom), background `#424242`.
- Tabs are laid out in sequence starting from the far left of the bar. Each tab:
  - In Quick Mask mode, the color mode in parentheses shows as "Quick Mask" (e.g. "(Quick Mask/8)"), matching Photoshop.
- When there are unsaved changes, " *" is appended to the end of the title, matching Photoshop (`title()`).
- The close button "×" is centered 11 from the tab's left edge; it is a 6 pt cross made of two 1 pt thin lines, color `#8c8c8c`, brightening on hover.
  - The title starts 24 from the left edge, semibold 12 pt; a right margin of 12 is left after the title.
  - There is a 1 pt `#404040` vertical line on the right.
  - The active tab has background `#535353` and title `#eaeaea`; inactive tabs have the same color as the bar and title `#bcbcbc`, with background `#4c4c4c` on hover.
- The title format matches Photoshop: `{file name} @ {zoom} ({mode}/{bit depth})`, for example `1.webp @ 100% (RGB/8#)`. The "#" after the bit depth means the document has no embedded color profile: opened files never have one (profiles are not read yet), and new documents are treated as sRGB, without "#".

## Interaction

- Clicking a tab: makes it the current document.
- Clicking "×": closes that document (confirming first when there are unsaved changes; see "Closing" in `actions.md`). When the closed one is the current document, the tab to its left (or the first one if there is none) becomes the current document.

## Known Limitations

- Tabs cannot be reordered by dragging or dragged out into floating windows; with too many tabs, they do not scroll or collapse.
