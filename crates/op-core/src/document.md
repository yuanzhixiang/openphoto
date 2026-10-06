# document.rs

## 职责

定义 `Document`：一个打开的图像文档，包括尺寸、分辨率、颜色模式、位深、图层列表和活动图层。负责新建/从像素构建文档、生成与恢复可撤销快照、调整画布尺寸（Image > Canvas Size），以及在 CPU 上把所有可见图层合成为一张 RGBA8 图像。

## 对外接口

### `DocId`

文档 ID，`u64` 包装。与 `LayerId` 共用同一个进程级原子计数器（从 1 开始，`Relaxed` 递增），所以进程内所有文档 ID 与图层 ID 互不重复。

### `Anchor`

Canvas Size 对话框中 3×3 锚点网格的位置，决定画布尺寸变化时原图像被钉在新画布的哪里。

- `x`：0 = 左，1 = 中，2 = 右；`y`：0 = 上，1 = 中，2 = 下。
- `Anchor::CENTER` 为 (1, 1)。
- 单轴偏移规则（原图左上角在新画布中的坐标，`diff = new - old`）：
  - 0：偏移 0；
  - 1：`diff.div_euclid(2)`，即向下取整；
  - 其他任何值（包括 2 以及大于 2 的非法值）：偏移 `diff`。
- 居中时尺寸差为奇数的取舍：放大时多出的那 1 像素加在右侧/底部（例如宽度 +3，左边加 1、右边加 2）；缩小时由于向负无穷取整，多裁掉的那 1 像素来自左侧/顶部（例如宽度 −3，左边裁 2、右边裁 1）。

### `Guide`

参考线：`vertical`（垂直还是水平）与 `position`（文档像素，可以在画布外）。

### `Snapshot`

文档中可撤销的部分：`width`、`height`、`resolution`、`layers`、`active_layer`、`selection`、`last_selection`、`guides`。字段私有，只能通过 `Document::snapshot()` 创建、`Document::restore()` 使用。克隆代价低，因为图层里的 tile 通过 `Arc` 共享。

`title`、`id`、`color_mode`、`bit_depth` 和 `revision` 不在快照中，撤销/重做不会改变它们。

### `Document`

公开字段：`id`、`title`、`width`、`height`、`resolution`（ppi）、`color_mode`、`bit_depth`、`layers`（自底向上）、`active_layer`、`guides`（参考线，与编辑一样可撤销，Photoshop 也是如此）。私有字段 `revision`。

- `new_with_background(title, width, height, background)`：File > New。生成单一背景图层（名为 `"Background"`、`is_background = true`），用 `background` 颜色填满，并设为活动图层。分辨率 72 ppi，RGB，8 位。
- `from_rgba8(title, width, height, pixels)`：从紧密排列的 RGBA8 缓冲区打开一个扁平位图，得到单一图层并设为活动图层，规则与 Photoshop 打开图片时一致：所有像素的 alpha 都是 255 时，是名为 `"Background"` 的背景图层（`is_background = true`）；只要有一个像素的 alpha 小于 255，就是名为 `"Layer 0"` 的普通图层，文档因此没有背景图层。其余元数据同上（72 ppi、RGB、8 位）。
- `new_layer_id()`：从全局计数器分配新的 `LayerId`；它不修改文档，也不把图层加入列表。
- `next_layer_name()`：新图层的名字「Layer N」，N 为现有「Layer 数字」名称中最大的数字加 1，没有时为 1，与 Photoshop 一致。
- `insert_above_active(layer)`：把图层插到活动图层正上方（没有活动图层时放在最上面），设为活动图层，并 `mark_dirty()`。
- `revision()` / `mark_dirty()`：读取 / 递增修订号。修订号在每次像素或图层属性变化时递增，渲染层与图层缩略图缓存据此判断是否需要刷新。
- `snapshot()` / `restore(snapshot)`：生成 / 恢复快照。`restore` 会覆盖宽高、分辨率、图层与活动图层，并调用 `mark_dirty()`。
- `resize_canvas(width, height, anchor, fill)`：Image > Canvas Size。
- `transform_canvas(width, height, image, selection)`：把画布换成 `width`×`height`：每个图层的图像经过 `image` 函数、当前选区和可 Reselect 的选区经过 `selection` 函数，然后更新尺寸、选区修订号并 `mark_dirty()`。Crop、Trim、Image Rotation 和画布翻转都通过它实现（见 `image_ops.md`）。不记录历史。
- `map_guides(f)`：用 `f` 变换每条参考线（裁剪、扩展画布、缩放、旋转、翻转时调用）。`resize_canvas` 自己按锚点偏移参考线。
- `has_background()`：是否存在背景图层。它决定 Canvas Size 中「画布扩展颜色」是否有意义：没有背景图层时，所有扩展区域都是透明的。
- `layer(id)` / `layer_mut(id)`：按 ID 线性查找图层。
- `composite_rgba8()`：合成为紧密排列的直通 RGBA8 缓冲区，长度为 `width * height * 4`。
- `composite_layers_rgba8(layers)`：按同样的规则只合成给定的图层列表（自底向上，尺寸与文档一致），供合并图层使用；`composite_rgba8` 就是对文档全部图层调用它。

## 行为规则

### 画布尺寸调整

- 按 `Anchor` 规则分别计算 x、y 偏移，然后对每个图层调用 `TiledImage::with_canvas`。
- 背景图层的扩展区域用 `fill` 的 RGB 填充，alpha 强制为 255（无论 `fill` 本身的 alpha 是多少），保持背景图层不透明。
- 非背景图层的扩展区域填充为全透明。
- 缩小画布时超出新画布的像素被直接裁掉，不会保留在画布外。
- 所有图层改写完成后更新文档宽高并调用 `mark_dirty()`。
- 本方法不记录历史；由调用方在之后调用 `History::record`。

### 合成

- 从底到顶遍历 `visible` 为真的图层；`opacity * fill` 小于等于 0 的图层整层跳过。
- 遍历图层已分配的 tile，未分配的 tile 视为透明直接跳过。
- 每个像素按图层的混合模式合成（`blend::composite`，算法见 `blend.md`）：源 alpha 为像素 alpha × `opacity` × `fill`，结果为直通 alpha。源 alpha 为 0 的像素跳过。
- 混合在 gamma 编码（sRGB）空间进行，与 Photoshop 的默认设置一致（理由见 `README.md`）。
- 累积使用 `f32` 缓冲区，最终每个分量 clamp 到 0..=1 后按 `×255 + 0.5` 截断量化为 `u8`。
- 结果中没有任何图层覆盖的像素为 `[0, 0, 0, 0]`。
- 例：白色背景上叠一层 50% 不透明度的黑色，得到 `[128, 128, 128, 255]`。

## 边界情况

- 宽或高为 0：允许创建和调整到 0 尺寸，不分配 tile，`composite_rgba8` 返回空缓冲区。本模块不校验尺寸下限或上限，这些限制由调用方（例如 Canvas Size 对话框）负责。
- `from_rgba8` 的缓冲区长度必须等于 `width * height * 4`，否则 panic（校验在 `TiledImage::from_rgba8` 中）。
- 用 alpha 为 0 的颜色调用 `new_with_background` 会得到一个不分配 tile、完全透明的背景图层。
- 没有任何图层或所有图层都不可见时，合成结果全透明。
- `composite_rgba8` 假设每个图层的图像尺寸与文档一致。若有图层比文档更大（只能通过直接改写公开字段造成），计算剩余宽高时会发生 `usize` 下溢。
- `layer` / `layer_mut` 找不到 ID 时返回 `None`；`active_layer` 指向的图层是否存在由调用方保证。

## 与其它模块的关系

- 依赖 `tile.rs`（像素存储、`with_canvas`）、`layer.rs`（图层）、`color.rs`（颜色量化）、`pixel.rs`（元数据）。
- `history.rs` 通过 `snapshot()` / `restore()` 实现撤销。
- `op-io` 用 `from_rgba8` 打开文件、用 `composite_rgba8` 导出。
- `op-ui` 新建文档、调用 Canvas Size、修改图层后调用 `mark_dirty()`，并把 `composite_rgba8` 的结果交给 `op-render` 显示。

## 已知限制

- 合成完全在 CPU 上进行，每次都整幅重算，不做脏区增量合成。
- 合成不区分 `opacity` 与 `fill`，两者相乘作为图层 alpha。
- Canvas Size 只支持像素尺寸变化，不做重采样（那属于 Image Size）。

## 选区

- 文档保存当前选区 `selection`（`None` 表示没有选区，此时编辑作用于整个文档）和上一次取消的选区 `last_selection`（用于 Select › Reselect）。两者都在快照里，所以选区变化可以撤销，与 Photoshop 一致。
- `set_selection(s)`：替换选区；空选区视为没有选区。取消选区（设为 `None`）时，原选区被记为 `last_selection`。
- `reselect()`：恢复上一次取消的选区；`can_reselect()` 在没有选区且有可恢复的选区时为真。
- `selection_revision()`：选区每次变化（包括撤销恢复和画布尺寸变化）都会递增，界面据此缓存蚂蚁线轮廓。选区变化不改变像素，不触发重新合成。
- `resize_canvas` 同时平移选区和 `last_selection`。

## 测试覆盖

- `selection_deselect_reselect_and_undo`：取消选区后可以重新选择，恢复快照会恢复快照里的选区，空选区视为没有选区。
- `opaque_bitmap_opens_as_background`：完全不透明的像素打开为背景图层。
- `transparent_bitmap_opens_as_regular_layer`：含半透明像素时打开为「Layer 0」普通图层，文档没有背景图层，该图层为活动图层。
- `resize_canvas_centered`：2×2 白色背景居中扩展到 4×5、扩展色黑色，验证新尺寸、四周为黑、原图位于 (1,1)–(2,2)，且高度差为奇数时多出的一行在底部（第 3 行为黑）。
- `resize_canvas_keeps_layers_transparent`：左上锚点扩展到 3×3 时，非背景图层的扩展区域保持透明，原像素位置不变。
- `composite_half_opacity_over_white`：50% 不透明度黑色图层叠在白色背景上得到 `[128, 128, 128, 255]`，验证 gamma 空间混合与量化规则。
