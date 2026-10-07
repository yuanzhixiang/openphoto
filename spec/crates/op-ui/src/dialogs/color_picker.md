# dialogs/color_picker.rs: Color Picker dialog

## Component responsibilities

Photoshop's Color Picker. Picks a color and returns it to whatever opened it: the foreground color, the background color, or Canvas Size's canvas extension color.

## How it opens and its title

| Entry point | Title | Written on OK to |
|---|---|---|
| The foreground color swatch in the toolbar | Color Picker (Foreground Color) | Foreground color |
| The background color swatch in the toolbar | Color Picker (Background Color) | Background color |
| Clicking the swatch currently being edited again in the Color panel | Same as above, distinguished by foreground/background | Same as above |
| Choosing "Other..." in Canvas Size or clicking the extension color swatch | Color Picker | Canvas Size's extension color (which also switches to Other...) |

When it opens, both "current" and "new" are the color from before it opened. The Color Picker can stack on top of Canvas Size, in which case the keyboard acts only on the Color Picker.

## Layout

All sizes are points (pt) measured 1:1 in Photoshop 2026, relative to the dialog's top-left corner (including the title bar):

- Dialog 534×374, title bar 28 tall (the same light gray title bar as other dialogs).
- **Color field**: from (10, 60), 256×256. Shows the two channels other than the one the slider controls.
- **Slider**: x 278–298, y 60–316. Each side has a small white triangle indicating the current value, both pointing right (matching Photoshop).
- **new / current swatches**: x 310–370, y 77–145; the top half is the new color and the bottom half the color at open; the words "new" above and "current" below.
- **Web-safe color warning**: when the color is not web-safe, a cube icon appears to the right of the swatches, with a 12-square swatch of the nearest web-safe color below it.
- **Buttons**: at x 408, 115×25 capsule buttons, in order OK (y 39), Cancel (y 73), Add to Swatches (y 116), Color Libraries (y 158).
- **Value area** (row center y):
  - Left column: H 209, S 232, B 255, R 281.5, G 304.5, B 327.5. Radio button center x 316, letter x 330, input field x 355, width 35; H is followed by "°", S and B by "%".
  - Right column: L 209, a 232, b 255. Radio button center x 429, letter x 442, input field x 466, width 43.
  - CMYK: C 281.5, M 304.5, Y 327.5, K 351. Letters right-aligned to x 462, input field x 466, width 34, followed by "%".
  - Hexadecimal: "#" centered at x 314, input field x 322, width 83, row center y 354.
  - Input fields are 20 tall with a `#484848` background and `#666666` border, and a blue outer ring `#1473e6` when focused. When opened, the hexadecimal field gets focus with its contents selected, as in Photoshop (you can type a color value directly and press Enter).
- **Only Web Colors**: checkbox (10, 330), 13 square, text starting at x 32.
- Photoshop's Color Picker is a classic AppKit dialog: text is the 12 pt system font (`theme::dialog`), button text 13 pt, and the title the 13 pt bold system font (`theme::dialog_bold`).

## Modes (radio buttons)

Nine radio buttons decide which channel the slider controls; the color field shows the other two channels (in the same arrangement as Photoshop):

| Slider | Color field horizontal axis | Color field vertical axis |
|---|---|---|
| H | S | B |
| S | H | B |
| B | H | S |
| R | B | G |
| G | B | R |
| B (blue) | R | G |
| L | a | b |
| a | b | L |
| b | a | L |

- On every axis the top/right is the maximum. The default mode is H.
- In H mode the slider shows pure hues (maximum saturation and brightness), independent of the current color; in the other modes the slider shows the gradient of the current color with only that channel changing.
- A hollow ring marks the current color in the color field; the ring is black when the color is light (L > 60), otherwise white. The ring is clipped to the color field, as in Photoshop: in a corner (such as the top-left for white) only a quarter of it shows.
- The color field and slider are generated on the CPU pixel by pixel as 256-level images from the current mode and color, and are regenerated only when the mode, the color or "Only Web Colors" changes.

## Values

- Display ranges: H 0–360, S/B 0–100, R/G/B 0–255, L 0–100, a/b −128–127, CMYK 0–100, all displayed as integers. Hexadecimal is 6 lowercase digits.
- Lab uses the D50 white point (sRGB via Bradford chromatic adaptation), matching the values Photoshop displays (e.g. #00afdc is L66 a−27 b−34).
- The color is stored internally as HSB, so when brightness is dragged to 0 (black) or saturation to 0 (gray), hue and saturation are not lost, and dragging back gives the original hue.

## Interaction

- Pressing or dragging in the color field sets the field's two channels; pressing or dragging on the slider (in an area widened by 10 on the left and right and 4 on the top and bottom) sets the slider's channel.
- Clicking a radio button or the letter after it switches the mode.
- Input fields: input takes effect live, and out-of-range values are clamped to the range; input that cannot be parsed or is incomplete (e.g. only two hex digits typed) does not change the color, and the typed text is kept. After an input field loses focus, it goes back to displaying the normalized value.
- Clicking the "current" swatch restores the color from when the dialog opened.
- Clicking the cube icon or the small swatch below it switches to the nearest web-safe color.
- Only Web Colors: when checked, the current color immediately becomes the nearest web-safe color, every later choice snaps to web-safe colors, and the color field and slider also show only web-safe colors.
- Add to Swatches: appends the current color to the end of the Swatches panel; the dialog stays open.
- OK or Enter: confirms and writes the color back to whatever opened the dialog.
- Cancel or Esc: closes without changing anything. When opened from Canvas Size's "Other...", canceling keeps the previous choice for the extension color.

## Differences from Photoshop and known limitations

- CMYK values use a device-independent formula (K = 1 − max(R,G,B)), whereas Photoshop converts through the ICC profile of the current CMYK working space (U.S. Web Coated SWOP by default), so the values differ (#00afdc is C72 M10 Y6 K0 in Photoshop and C100 M20 Y0 K14 here). There is no CMYK out-of-gamut warning icon.
- Add to Swatches adds directly, without Photoshop's "Color Swatch Name" naming dialog.
- The Color Libraries button is grayed out.
- While the dialog is open, moving the mouse over the document does not turn it into an eyedropper for sampling colors.
- Colors are not display color-managed, so the same color value looks more saturated than in Photoshop (which converts according to the display profile).
- Button text does not use macOS letter spacing and is slightly wider than in Photoshop (e.g. "Add to Swatches").
- The Color Libraries button is unavailable (there are no color libraries).
- The dialog cannot be dragged; its position is fixed in the center of the window.
