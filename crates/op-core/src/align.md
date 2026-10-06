# align.rs：对齐与分布

## 职责

Layer › Align、Layer › Distribute 以及移动工具选项栏的对齐/分布按钮：按选中图层的像素范围移动它们，使其对齐或均匀分布，规则与 Photoshop 2026 一致（结果在 Photoshop 中实测核对过）。

## 对外接口

- `Align`：`Top`、`VerticalCenter`、`Bottom`、`Left`、`HorizontalCenter`、`Right`；`ALL` 为菜单顺序，`label()` 为菜单文字（「Top Edges」等），`name()` 为历史名称（「Align Left Edges」等，与 Photoshop 记录的一致）。
- `Distribute`：上面六种加 `Horizontally`、`Vertically`（等间距）；同样有 `ALL`、`label()`、`name()`（「Distribute Vertical Centers」等）。
- `can_align(doc)` / `align(doc, how)`：有像素选区时，选中的图层对齐到选区的外框（一个图层就可以）；没有选区时需要至少两个图层，对齐到它们的总外框。中心对齐的位移四舍五入到整像素。
- `can_distribute(doc)` / `distribute(doc, how)`：需要至少三个图层。按所选的边或中心排序，最外侧的两个不动，中间的按所选的边或中心等距排列；`Horizontally`/`Vertically` 则让相邻图层之间的空隙相等。

参与的图层：选中的、不是背景、没有锁定位置或像素、有不透明像素的图层；像素范围用 `TiledImage::content_bounds`（包括画布外的像素），移动用 `with_canvas` 平移，画布外的像素保留。

## 图层组

选中的组算作一个整体：范围是组里所有像素图层的并集，移动时组里的所有像素图层一起移动。

## 测试覆盖

- `matches_photoshop`：三个方块依次左对齐、垂直居中对齐、底对齐、按垂直中心分布，结果与 Photoshop 2026 中同样操作后的图层范围完全一致。
- `equal_gaps_and_selection_alignment`：等间距分布后中间一条两侧空隙相等；有像素选区时单个图层右对齐到选区。
