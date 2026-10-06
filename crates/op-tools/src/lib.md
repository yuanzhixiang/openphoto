# op-tools：工具定义

## 职责

定义 Photoshop 2026 工具栏中的全部工具、它们在工具栏中的分组、显示名称和单键快捷键。工具的行为在 `op-ui` 中实现；这里只有静态定义和按键选择工具的规则。本 crate 没有依赖。

## 对外接口

### `Tool`

68 个工具，涵盖 Photoshop 工具栏及各组弹出菜单里的全部工具，例如 `Move`、`Artboard`、`RectangularMarquee`、`EllipticalMarquee`、`SingleRowMarquee`、`SingleColumnMarquee`、`Lasso`、`PolygonalLasso`……`Hand`、`RotateView`、`Zoom`。

### `TOOLBAR`

工具栏从上到下的各个「格」，每一格是共用一个按钮的一组工具，组内顺序与 Photoshop 弹出菜单一致：

| 格 | 工具组 |
|---|---|
| 1 | Move、Artboard |
| 2 | Rectangular / Elliptical / Single Row / Single Column Marquee |
| 3 | Lasso、Polygonal Lasso、Magnetic Lasso |
| 4 | Object Selection、Quick Selection、Magic Wand |
| 5 | Crop、Perspective Crop、Slice、Slice Select |
| 6 | Frame |
| 7 | Eyedropper、Color Sampler、Ruler、Note、Count |
| 8 | Spot Healing Brush、Remove、Healing Brush、Patch、Content-Aware Move、Red Eye |
| 9 | Brush、Pencil、Color Replacement、Mixer Brush |
| 10 | Clone Stamp、Pattern Stamp |
| 11 | History Brush、Art History Brush |
| 12 | Eraser、Background Eraser、Magic Eraser |
| 13 | Gradient、Paint Bucket |
| 14 | Blur、Sharpen、Smudge |
| 15 | Dodge、Burn、Sponge |
| 16 | Pen、Freeform Pen、Curvature Pen、Add / Delete Anchor Point、Convert Point |
| 17 | Horizontal / Vertical Type、Vertical / Horizontal Type Mask |
| 18 | Path Selection、Direct Selection |
| 19 | Rectangle、Ellipse、Triangle、Polygon、Line、Custom Shape |
| 20 | Hand、Rotate View |
| 21 | Zoom |

### `Tool::name()`、`Tool::shortcut()`、`Tool::slot()`

- `name()`：与 Photoshop 相同的英文全名，例如 `"Elliptical Marquee Tool"`。
- `shortcut()`：与 Photoshop 一致的单键快捷键。同组工具共用组的字母（V、M、L、W、C、K、I、J、B、S、Y、E、G、O、P、T、A、U、H、Z），Rotate View 单独用 R。Single Row/Column Marquee、Blur 组、Add/Delete Anchor Point、Convert Point 没有快捷键。
- `slot()`：工具所在的格。

### `tool_for_key(key, shift, active, current)`

按下字母键时选择哪个工具，规则与 Photoshop 一致：

- `current(slot)` 是每一格当前显示的工具（Photoshop 记住每组最近用过的那个）。
- 不按 Shift：选中该字母所在格当前显示的工具；如果它不使用这个字母，则选中组内第一个使用这个字母的工具。
- 按 Shift，且当前工具就在这一格：按组内顺序循环到下一个使用这个字母的工具，没有快捷键的工具被跳过。
- 按 Shift 但当前工具在别的格：与不按 Shift 相同。
- 一格内只有一个工具使用该字母时（例如 R、H）直接选中它。

## 测试覆盖

- `every_tool_appears_once`：68 个工具在工具栏中各出现一次。
- `keys_select_the_group_and_shift_cycles`：M、⇧M 循环（跳过单行/单列选框）、格记忆、从其它格按 ⇧M、R 与 H 的处理、未使用的字母。
