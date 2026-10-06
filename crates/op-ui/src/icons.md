# icons.rs：图标映射

## 职责

重新导出 Phosphor（Regular）全部图标常量，并提供 `tool()`：工具到图标的映射。不使用 Adobe 的图标资源。

## 工具图标

`tool()` 为 `op-tools` 中的全部 68 个工具各指定一个 Phosphor 图标，挑选形状最接近 Photoshop 图标的，例如 Rectangular Marquee 为 `SELECTION`、Elliptical Marquee 为 `CIRCLE_DASHED`、Magic Wand 为 `MAGIC_WAND`、Pencil 为 `PENCIL`、Gradient 为 `GRADIENT`、Rotate View 为 `ARROW_CLOCKWISE`。部分同组工具共用一个图标（例如三种橡皮擦都是 `ERASER`）。

图标外观与 Photoshop 不同：Photoshop 的工具图标更大、多为实心风格。
