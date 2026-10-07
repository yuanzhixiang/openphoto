# icons.rs：图标映射

## 职责

重新导出 Phosphor（Regular）全部图标常量，并提供 `tool()`：工具到图标的映射。不使用 Adobe 的图标资源。工具栏与工具弹出列表现在用 `tool_icons.rs` 绘制的图标，`tool()` 只在没有绘制的工具（Remove Tool）上作为后备。

## 工具图标

`tool()` 为 `op-tools` 中的全部 70 个工具各指定一个 Phosphor 图标，挑选形状最接近 Photoshop 图标的，例如 Rectangular Marquee 为 `SELECTION`、Elliptical Marquee 为 `CIRCLE_DASHED`、Magic Wand 为 `MAGIC_WAND`、Pencil 为 `PENCIL`、Gradient 为 `GRADIENT`、Rotate View 为 `ARROW_CLOCKWISE`、Selection Brush 为 `PAINT_BRUSH_BROAD`、Adjustment Brush 为 `PAINT_BRUSH_HOUSEHOLD`。部分同组工具共用一个图标（例如三种橡皮擦都是 `ERASER`）。

图标外观与 Photoshop 不同：Photoshop 的工具图标更大、多为实心风格。
