# selection.rs：选区

## 职责

像素选区。与 Photoshop 一样，选区是一张与文档同尺寸的 8 位蒙版：255 为完全选中，0 为未选中，中间值是部分选中（抗锯齿或羽化的边缘）。文档的选区保存在 `Document` 中（见 `document.md`）。

## 对外接口

- `SelectionOp`：新形状与现有选区的组合方式 —— `Replace`（新建）、`Add`（添加）、`Subtract`（减去）、`Intersect`（交叉）。
- `Rect`：文档像素坐标的矩形，构造时自动整理成左上到右下，右下边界不包含在内，可以超出文档。宽或高小于 1 像素时为空。
- `Selection::all(w, h)`：全选。
- `Selection::rect(w, h, rect)`：矩形选区，边界四舍五入到整像素，超出文档的部分被裁掉。
- `Selection::ellipse(w, h, rect, anti_alias)`：`rect` 的内切椭圆。开启抗锯齿时，边缘像素按 4×4 超采样的覆盖率部分选中；关闭时，像素中心在椭圆内才选中。
- `Selection::from_mask(w, h, mask, anti_alias)`：从 0/255 蒙版创建（油漆桶的填充区域）。开启抗锯齿时，紧邻选中区域外侧的像素按 50% 选中。
- `get(x, y)`：某像素的选中程度（文档外为 0）。
- `is_empty()`：没有任何像素被选中。
- `bounds()`：选中像素（值 > 0）的外接矩形。
- `inverse()`：反选（每个值取 255 − v）。
- `combine(current, shape, op)`：把新形状按 `op` 并入现有选区。添加取较大值，交叉取较小值，减去为 `a × (255 − b) / 255`。没有现有选区时：新建和添加得到形状本身，减去和交叉得到空选区。
- `feather(radius)`：羽化。用三次盒式模糊近似 sigma = radius / 2 的高斯模糊，边缘外侧按就近值延伸。
- `with_canvas(w, h, dx, dy)`：画布尺寸变化后的选区，原选区放在 (`dx`, `dy`)，新增区域未选中。
- `outline()`：蚂蚁线轮廓。以 128 为阈值区分选中与未选中，返回两者之间所有单位像素边，合并成水平和垂直的线段，每段为 `[x0, y0, x1, y1]`（文档像素）。

## 边界情况

- 选区尺寸始终等于文档尺寸；`combine` 假定两者尺寸相同。
- 羽化半径小于等于 0 时原样返回。
- 选中程度低于 128 的像素不在蚂蚁线以内（与 Photoshop 一致：羽化后很淡的边缘不显示蚂蚁线）。

## 已知限制

- 选区是密集存储的，每个像素 1 字节，与文档尺寸成正比。
- 没有 Photoshop 的 Select › Modify（扩展、收缩、平滑、边界）、Grow、Similar、Color Range 等运算。

## 测试覆盖

- `rect_and_bounds`：矩形边界取整与超出文档时的裁剪。
- `combine_ops`：四种组合方式，以及没有现有选区时的减去。
- `inverse_and_all`：反选与全选。
- `ellipse_anti_aliasing`：不抗锯齿时只有 0 和 255，抗锯齿时有中间值。
- `feather_softens_edges`：羽化后内部仍接近全选，边缘为中间值，远处接近 0。
- `outline_of_rect`：矩形选区的轮廓正好是四条边。
- `with_canvas_shifts`：画布扩展后选区随偏移移动。
