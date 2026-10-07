# clipboard.rs：剪切、复制与粘贴的像素规则

## 职责

实现 Edit › Cut、Copy、Copy Merged、Paste 与 Paste Special › Paste in Place 的像素部分：复制哪些像素、剪切后留下什么、粘贴到哪里。剪贴板本身（保存内容、与系统剪贴板交换）在 `op-ui` 的 `clipboard.rs`。

## 对外接口

### `Clip`

剪贴板上的像素：`width`、`height`、`pixels`（直通 alpha 的 RGBA8，逐行排列）、`origin`（在来源文档中的左上角位置；来自其它应用的图片为 `None`）。`Clip::from_rgba8(w, h, pixels)` 构造没有位置的图片，像素长度不对时 panic。

### `ClipError`

- `NoLayer`、`Hidden`、`Locked`：提示文字与 Fill 相同（见 `fill.md`），例如「Could not complete the Cut command because the target layer is hidden.」。
- `Empty`：复制范围内只有透明像素，提示「Could not complete the {命令} command because the selected area is empty.」，与 Photoshop 一致。
- `message(command)` 生成提示，`command` 为「Cut」「Copy」「Copy Merged」。

### 函数

- `copy(doc)`：复制活动图层。
- `copy_merged(doc)`：复制所有可见图层合成后的结果（`composite_rgba8`）。
- `cut(doc, background)`：先复制，再按 Edit › Clear 的规则清除（`fill::clear`）：普通图层上变透明，背景图层或锁定透明像素的图层上填背景色。
- `placement(clip, width, height, visible, in_place)`：计算粘贴位置（左上角）。
- `paste(doc, clip, at)`：把内容放到新图层上，返回新图层 ID。

## 行为规则

### 复制范围

- 有选区时取选区的外接矩形；没有选区时取整个画布。
- 每个像素的 alpha 乘以该点的选择程度（四舍五入），所以羽化、消除锯齿的选区边缘会被带上半透明。外接矩形内、选区外的像素 alpha 为 0。
- 范围内所有像素 alpha 都是 0 时返回 `Empty`，不改动剪贴板。选区为空（没有外接矩形）同样返回 `Empty`。
- `origin` 为外接矩形左上角。

### 可用条件

- Copy、Copy Merged 不检查图层是否隐藏或锁定，只读不写。
- Cut 要求活动图层可见（否则 `Hidden`）、像素未锁定（否则 `Locked`），检查在复制之前，失败时文档不变。

### 粘贴位置

- Paste in Place：始终使用 `origin`（没有 `origin` 时按下面的居中规则）。
- Paste：`origin` 存在、整块内容落在画布内、并且与可见区域有交集时，使用 `origin`——在同一文档里复制后粘贴，副本正好叠在原处，与 Photoshop 一致。否则把内容居中放在「可见区域与画布的交集」里；交集为空（画布被完全滚出窗口）时居中放在画布上。居中坐标四舍五入到整数像素。
- `visible` 由调用方给出（文档像素坐标的 `[x0, y0, x1, y1]`）。

### 粘贴

- 新建与画布同样大的透明图层，按 `Document::next_layer_name` 命名（「Layer N」），插到活动图层正上方并设为活动图层（`insert_above_active`）。
- 只写入 alpha 大于 0 的像素；落在画布外的部分保留在新图层上（`set_pixel_at`），与 Photoshop 一致，Image › Reveal All 可以让它们显示出来。
- 粘贴后取消选区（原选区记为可以 Reselect 的选区），与 Photoshop 一致。
- 历史记录由调用方负责（`op-ui` 记录「Paste」「Cut」）。

## 边界情况

- 没有活动图层时 Copy、Cut 返回 `NoLayer`；Copy Merged 不需要活动图层。
- 粘贴内容可以比画布大，超出部分被裁掉。

## 已知限制

- 没有 Paste Into / Paste Outside（需要图层蒙版）。
- 不支持粘贴文字（Photoshop 会创建文字图层）和矢量路径。

## 图层组

当前图层是组时，拷贝、剪切返回 `ClipError::Group`。

## 测试覆盖

- `copy_takes_the_selection_bounds`：复制选区外接矩形内的像素和位置；没有选区时复制整个图层。
- `copying_transparent_pixels_fails`：透明图层上复制返回 `Empty` 及其提示文字；Copy Merged 能取到下层的像素。
- `cut_clears_and_paste_stacks_on_the_original`：背景图层上剪切后留下背景色；粘贴回原位置，生成「Layer 1」、成为活动图层并取消选区。
- `paste_centers_when_the_origin_is_out_of_view`：原位置超出画布时居中；Paste in Place 保持原位置；外部图片居中到可见区域。
- `paste_keeps_pixels_outside_the_canvas`：画布内的部分正常显示，超出右边的像素保留在图层上，图层内容范围伸到画布外。
