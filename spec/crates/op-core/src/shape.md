# shape.rs：形状工具

## 职责

矩形、椭圆、三角形、多边形、直线工具：生成形状的覆盖范围，并作为一个新图层（用颜色填充）加入文档。不记录历史。

## 对外接口

- `ShapeKind`：`Rectangle`、`Ellipse`、`Triangle`、`Polygon(边数)`、`Line`。`layer_name()` 为新图层名的前缀（「Rectangle」「Ellipse」「Triangle」「Polygon」「Line」）。
- `coverage(kind, w, h, a, b, weight)`：形状在画布上的覆盖范围（`Selection`，全部消除锯齿）。
  - 矩形、椭圆、三角形、多边形填满 `a`、`b` 两点构成的矩形：矩形即该矩形；椭圆为内切椭圆；三角形顶点在上边中点、底边两角；多边形为内接于矩形的正多边形（少于 3 边按 3 边），第一个顶点在正上方。
  - 直线：从 `a` 到 `b`、宽 `weight` 像素的长方形；两点重合时为空。
- `add_shape_layer(doc, kind, a, b, weight, color)`：用覆盖值作为 alpha、`color` 作为颜色，生成一个新图层插在当前图层之上并选中它，返回其 ID。图层名为前缀加编号，编号为同名前缀已有最大编号 + 1（如「Rectangle 1」「Rectangle 2」）。覆盖为空时不添加图层，返回 `None`。

## 已知限制

- 形状是栅格图层，不是 Photoshop 的矢量形状图层：没有路径、实时形状属性、圆角、描边，也不能再编辑形状。
- 没有自定形状工具和路径/像素模式。

## 测试覆盖

- `shapes_cover_their_box`：矩形范围、椭圆中心与角、三角形、正六边形、2 像素粗的水平直线。
- `shape_layers_are_numbered`：连续两个矩形得到「Rectangle 1」「Rectangle 2」，像素为对应颜色；零尺寸形状不加图层。
