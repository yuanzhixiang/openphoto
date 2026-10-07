# ps_icons.rs: Vector Icons Traced from Photoshop

## Responsibilities

The icons in the options bar are traced pixel by pixel from Photoshop 2026 2x screenshots into vector shapes (rectangles, convex polygons, polylines, rings) and drawn with egui drawing commands, so their shape, weight, and position match Photoshop rather than being approximated with an icon font. Adobe's icon resource files are not used.

## Public Interface

- `paint_scaled(painter, center, icon, color, background, scale)`: draws scaled (both coordinates and line widths are multiplied by `scale`); `paint` is `scale = 1`. The Lock Layers dialog uses the lock icons at about 0.45–0.5×.
- `Icon`: Home, Move (four-way arrow), Caret (V-shaped dropdown arrow), Share, Bell, Search, Lightbulb, Workspace, the eight align and distribute icons (AlignLeft, AlignHorizontalCenter, AlignRight, DistributeVertically, AlignTop, AlignVerticalCenter, AlignBottom, DistributeHorizontally), More (three dots), Gear (gear with a small white triangle at the bottom right), CollapseToolbar (the toolbar collapse bar's bold "»"), CollapseRight / CollapseLeft (the thin-line "»" "«" of the panel column and icon column collapse bars), History, Comments (the two buttons in the icon column), as well as the Layers panel's filter buttons (FilterPixel and four others), lock buttons (LockTransparent, LockPixels, LockPosition, LockArtboards, LockAll), the eye Eye, the hollow lock LayerLock (background layer and partially locked layers), the solid lock LayerLockFull (layers with Lock all), and the eight bottom-bar button icons.
- `paint(painter, center, icon, color, background)`: draws the icon centered on `center`. `background` is the color behind the icon, used for the gear's center hole.

## Coordinate Convention

Each icon's coordinates are in Photoshop 2x device pixels (i.e. half a pt), relative to the icon center. The icon center is the center of the ink extent when traced, and the options bar places it at the center position measured in Photoshop (see `options_bar.md`). Line widths are likewise given in 2x pixels.

## Visual States

Colors are supplied by the caller: normal `#dddddd`, bell `#b9b9b9`, disabled `#989898`; the gear's small triangle is always white.

## Known Limitations

- The toolbar's tool icons are drawn in `tool_icons.rs`; the graphics at the bottom of the toolbar are drawn directly in `toolbar.rs`.
- The avatar is a placeholder solid circle, not Photoshop's account avatar.
- Crop tool options bar icons (traced from Photoshop 2026 2x screenshots): Straighten (a level: a dot on top, squares on both sides, and a cup shape with a bubble), CropOverlay (a 3 × 3 grid plus crop marks and a small menu triangle), Info (an i in a circle), CropReset (a rotating arrow plus a baseline), CropCancel (⦸), CropCommit (✓), Swap (two opposing solid arrows, up and down).
- Hue/Saturation dialog icons (traced from Photoshop 2026 2x screenshots): `TargetedHand` (the targeted-adjustment hand, with small triangles on both sides), `Eyedropper`, `EyedropperPlus`, `EyedropperMinus` (hollow barrel, a diagonal band, and a ball at the top; the latter two add "+" "−" at the bottom right), `PresetMenu` (three horizontal lines and a small triangle at the bottom right).
- Levels / Curves icons: `EyedropperBlack` / `EyedropperGray` / `EyedropperWhite` (eyedroppers whose lower barrel is that color), `CurvePoints` (a wavy line through four points), `Pencil`, `TargetedHandVertical` (a hand with up and down arrows), `GridQuarters`, `GridTenths`.

## Options Bar Icons (approximately redrawn from Photoshop 2026 screenshots)

`PenPressure` (pressure), `Refresh`, `ObjectFinder`, `Feedback` (Object Selection), `QuickNew` / `QuickAdd` / `QuickSubtract` (Quick Selection modes), `Angle` (brush angle), `BrushPanel` / `CloneSourcePanel` (panel toggles: a solid square with a brush or stamp cut out), `OpacityPressure`, `Airbrush`, `Symmetry` (a butterfly with a small menu triangle), `IgnoreAdjustments`, `SampleContinuous` / `SampleOnce` / `SampleSwatch` (sampling modes), `MixerLoad` / `MixerClean`, the five gradient types (grayscale gradients in a 14 pt box: linear, radial, angle, reflected, diamond), `NotesPanel`.

Pen, type, shape, view, and other tools: `PathOperations` (two squares, the second hollowed out) and `PathCombine` (a solid square, when the shape and Path Selection tools have no paths that can be combined), `PathAlignment`, `PathArrangement`, `GearMenu` (gear), all four with a small menu triangle at the bottom right (`menu_mark`); `TextOrientation`, `FontSize`, `TextLeft` / `TextCenter` / `TextRight` (long and short horizontal lines for horizontal alignment) and `TextTop` / `TextMiddle` / `TextBottom` (vertical lines for vertical alignment), `WarpText`, `Text3d`, `CharacterPanels`; `Link` (chain link), `CornerRadius` (a quarter arc), `PolygonSides` (a # in a hexagon); `ZoomIn` / `ZoomOut`; `ArrangeFront` / `ArrangeForward` / `ArrangeBackward` / `ArrangeBack` (four stacked diamonds, the solid one marking the target position, with up and down arrows on the left); `ArtboardPortrait` / `ArtboardLandscape` (a page with a folded corner and tick marks at the corners), `AddArtboard` (a + in a page with a folded corner); the Frame tool's `FrameRect` / `FrameEllipse` / `FrameTriangle` / `FrameHexagon` / `FrameCustom` (gray fill `#6f6f6f`, light outline, and a cross); `StrokeCenter` (a box with control points at the four corners); `BrushAdd` / `BrushSubtract` (a brush plus "+" or "−": Selection Brush, Remove tool, Adjustment Brush).
