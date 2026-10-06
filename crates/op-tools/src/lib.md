# op-tools：工具定义

## 职责

定义工具栏上的工具清单、显示名称、单键快捷键和分组标记。crate 注释把这里定位为工具行为（指针事件 → 文档编辑）的归属地，UI 只负责把画布坐标下的事件转发过来；目前本 crate 只包含工具的静态定义，不包含任何工具行为。

## 对外接口

### `Tool`

21 个工具，与 Photoshop 默认工具栏一致：`Move`、`RectangularMarquee`、`Lasso`、`ObjectSelection`、`Crop`、`Frame`、`Eyedropper`、`SpotHealingBrush`、`Brush`、`CloneStamp`、`HistoryBrush`、`Eraser`、`PaintBucket`、`Blur`、`Dodge`、`Pen`、`HorizontalType`、`PathSelection`、`Rectangle`、`Hand`、`Zoom`。

### `TOOLBAR`

工具栏顺序，与 Photoshop 默认工具栏一致，内容与上面的枚举顺序相同。它是一个扁平列表，本身不携带分组信息。

### `Tool::name()`

工具提示等处显示的英文全名，与 Photoshop 一致，例如 `"Rectangular Marquee Tool"`、`"Spot Healing Brush Tool"`。

### `Tool::shortcut()`

单键快捷键（大写字母），与 Photoshop 默认快捷键一致：

| 工具 | 键 | 工具 | 键 |
| --- | --- | --- | --- |
| Move | V | Eraser | E |
| RectangularMarquee | M | PaintBucket | G |
| Lasso | L | Blur | 无 |
| ObjectSelection | W | Dodge | O |
| Crop | C | Pen | P |
| Frame | K | HorizontalType | T |
| Eyedropper | I | PathSelection | A |
| SpotHealingBrush | J | Rectangle | U |
| Brush | B | Hand | H |
| CloneStamp | S | Zoom | Z |
| HistoryBrush | Y | | |

`Blur` 没有快捷键，与 Photoshop 一致。在 Photoshop 中用 Shift+同一键在同组工具间循环，这里不包含这种循环。

### `Tool::from_shortcut(c)`

把字符转为大写后，按 `TOOLBAR` 顺序找到第一个快捷键匹配的工具；没有匹配时返回 `None`。大小写不敏感。由于每个快捷键只对应一个工具，结果是唯一的。

### `Tool::has_group()`

是否在工具图标右下角画小三角，表示该位置在 Photoshop 中还收纳着同组的其他工具。`Hand`、`Zoom`、`Frame` 返回 `false`，其余工具返回 `true`。

## 与其它模块的关系

- 不依赖 `op-core` 中的任何类型（`Cargo.toml` 声明了依赖但代码未使用）。
- `op-ui` 的工具栏按 `TOOLBAR` 绘制并用 `has_group()` 决定是否画三角；键盘处理用 `from_shortcut` 切换当前工具；选项栏与文档视图根据当前 `Tool` 决定显示内容。

## 已知限制

- 所有工具都只有定义，没有实现编辑行为。
- 同组工具（例如 Lasso 组中的多边形套索）没有被定义，工具栏只显示每组的默认工具，`has_group()` 的小三角只是视觉提示。
- 不支持 Shift+快捷键在同组工具间循环，也不支持自定义快捷键。

## 测试覆盖

本 crate 没有单元测试。
