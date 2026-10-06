# fill.rs：填充、清除与油漆桶

## 职责

Edit › Fill、Edit › Clear 和油漆桶工具对当前图层像素的修改。

## 目标与错误

所有操作都作用于当前图层。不能操作时返回 `FillError`，`message(命令名)` 给出 Photoshop 的提示文字：

- 没有图层：「Could not complete the {命令} command because there is no layer.」
- 图层锁定像素：「Could not complete the {命令} command because the layer is locked.」
- 图层隐藏：「Could not complete the {命令} command because the target layer is hidden.」

## 填充（`fill`）

- 用纯色填充选区；没有选区时填充整个图层。选中程度（羽化）按比例生效。
- `FillOptions`：混合模式（全部 27 种，算法见 `blend.md`）、不透明度、保留透明区域。默认 Normal、100%、不保留。
- 背景图层、锁定透明像素的图层，或勾选了保留透明区域时：只给已有像素着色，alpha 不变，完全透明的像素不受影响。
- 其余情况：颜色按混合模式和 `不透明度 × 选中程度` 合成到像素上，可以让透明像素变为不透明。

## 清除（`clear`）

- 普通图层：选区内像素的 alpha 乘以 `1 − 选中程度`（没有选区时清空整个图层）。
- 背景图层或锁定透明像素的图层：改为用背景色填充，与 Photoshop 一致。

## 油漆桶（`bucket`）

- `BucketOptions`：填充设置、容差（0–255，默认 32）、消除锯齿（默认开）、连续（默认开）、所有图层（默认关），与 Photoshop 的默认值一致。
- 取样：默认取当前图层的像素；「所有图层」时取合成结果。
- 区域：RGBA 四个通道与单击点的差都不超过容差的像素。「连续」时只取与单击点四邻域相连的部分，否则取整个图像中所有相似像素。
- 消除锯齿：区域外紧邻区域的一圈像素按 50% 选中。
- 区域再与当前选区相乘，然后按填充设置填充前景色。单击点在文档外时不做任何事（返回 `Ok(false)`）。

## 蒙版

快速蒙版模式下填充不检查图层是否隐藏或锁定。编辑目标是蒙版（快速蒙版或图层蒙版）时，Fill（及 ⌥⌫、⌘⌫、油漆桶）把颜色换成灰度后写入蒙版；Clear（⌫）用背景色的灰度填充选区。

## 魔棒（`magic_wand`）

- `magic_wand(doc, x, y, options)`：魔棒单击得到的选区，区域规则与油漆桶完全相同（取样、容差、连续、消除锯齿），只是不填充。单击点在画布外、或不取所有图层时没有活动图层，返回 `None`。
- 测试 `magic_wand_selects_the_clicked_area`：点黑色方块得到方块的范围，点白色得到方块以外的区域，画布外为 `None`。

## 扩大选取与选取相似（`grow`）

- `grow(doc, options, contiguous)`：Select › Grow（`contiguous` 为真）与 Select › Similar。取当前选区中选择程度 ≥ 128 的像素在各通道（RGBA）上的最小、最大值，再向两边放宽魔棒的容差；Grow 从这些像素出发，按四邻域扩展到落在范围内的相连像素；Similar 选取整幅图像中所有落在范围内的像素。按魔棒的消除锯齿选项处理边缘，结果与原选区合并（相加）。取样与魔棒相同（当前图层或所有图层）。没有选区时返回 `None`。
- 测试 `grow_and_similar`：Grow 只扩到相连的同色方块，Similar 包括另一个方块但不包括中间的白色；没有选区时为 `None`。

## 已知限制

- 填充内容只有纯色；Content-Aware、Pattern、History 没有实现。
- 所有操作都遍历整个文档，没有按选区边界裁剪，大文档上较慢。
- 消除锯齿只是一圈 50% 的边缘，与 Photoshop 的抗锯齿效果不完全相同。

## 图层组

当前图层是组时，Fill、油漆桶、调整（`adjust::check`）和滤镜返回 `FillError::Group`，提示「Could not complete the {命令} command because the target layer is a group.」。Photoshop 在这种情况下直接把这些菜单项置灰（Invert 已核对），界面层应据此禁用命令。魔棒、Grow/Similar 在当前图层是组且不取样所有图层时没有结果。

## 测试覆盖

- `fill_respects_selection_opacity_and_mode`：选区内按 50% 不透明度填充，选区外不变；Multiply 模式的结果。
- `clear_erases_or_fills_background`：背景图层上清除得到背景色，普通图层上清除得到透明。
- `preserve_transparency_keeps_alpha`：保留透明区域时只改颜色。
- `bucket_fills_contiguous_region`：被一列黑色隔开时只填一侧；非连续时两侧相似像素都被填。
- `locked_layer_refuses`：锁定像素时拒绝，提示文字与 Photoshop 一致。
