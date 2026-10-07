# tile.rs

## 职责

提供图层像素的底层存储：把一张 RGBA8 图像切成 256×256 的 tile，稀疏地存放，并通过 `Arc` 在多个图像（尤其是历史快照）之间共享未修改的 tile。设计动机见同目录 `README.md`。

## 对外接口

### `TILE_SIZE`

tile 边长，固定为 256 像素。每个 tile 占 256×256×4 = 262144 字节。

### `Tile`

- 一个 256×256 的 RGBA8 tile，直通 alpha，像素按行优先、每像素 4 字节存于公开字段 `data: Box<[u8]>`。
- `Tile::filled(rgba)`：整块填充同一颜色。
- 边缘 tile 同样是完整的 256×256；超出图像宽高的部分是存储上的填充，不属于图像内容。

### `TiledImage`

`width` × `height` 是画布。与 Photoshop 的图层一样，像素也可以在画布外：tile 坐标是有符号的（`HashMap<(i32, i32), Arc<Tile>>`），画布左边、上边的 tile 用负坐标，右边、下边的 tile 在画布的 tile 范围之外；边缘 tile 中超出画布宽高的那一部分同样是「画布外」的像素。因此除非确实要把像素放到画布外，任何代码都不能往边缘 tile 的画布外部分写东西（这是本模块最重要的不变量）。

- `new(width, height)`：空图像，不分配任何 tile，整体视为全透明。
- `filled(width, height, rgba)`：画布填满纯色，画布外没有像素。alpha 为 0 时等同于 `new`；否则内部 tile 共享同一个 `Arc<Tile>`，边缘 tile 单独复制并把画布外部分清成透明（`clear_edge_padding`）。
- `from_rgba8(width, height, pixels)`：从画布大小的紧密 RGBA8 缓冲区构建，逐 tile 拷贝，全透明的 tile 被丢弃；边缘 tile 的画布外部分为透明。
- `width()`、`height()`：画布尺寸。`tiles_x()`、`tiles_y()`：覆盖画布的 tile 数，按 `div_ceil(TILE_SIZE)` 计算。
- `tile(tx, ty)` / `tile_mut(tx, ty)`：画布范围内（`u32` 坐标）的 tile；`tile_mut` 在缺失时分配全透明 tile，被共享时先复制（写时复制）。
- `tile_at(tx, ty)` / `tile_at_mut(tx, ty)`：任意 tile（`i32` 坐标，可在画布外）。
- `pixel(x, y)` / `set_pixel(x, y, rgba)`：画布内的像素；坐标在画布外时读出透明、写入被忽略。
- `pixel_at(x, y)` / `set_pixel_at(x, y, rgba)`：任意位置（`i64`，可为负）的像素。向缺失的 tile 写透明像素不分配 tile。
- `content_bounds()`：所有不透明像素（alpha > 0，包括画布外的）的外框 `(x0, y0, x1, y1)`，没有时为 `None`。先处理 tile 网格边缘的 tile，之后完全落在已得外框内的 tile 直接跳过；每个 tile 内从上下向里找第一行、再在其间从左右向里找第一列，遇到像素即停。它每帧会被调用多次（移动工具的变换控件、对齐按钮的可用状态），所以不能逐像素扫描整个图层。
- `has_pixels_outside()`：是否有像素在画布外。
- `clipped()`：删掉画布外像素后的图像（背景图层、开启「删除裁剪的像素」的裁剪用它）。只保留画布范围内的 tile 并清掉边缘 tile 的画布外部分。
- `with_canvas(width, height, dx, dy, fill)`：生成新画布尺寸的图像，所有像素（包括画布外的）平移 (`dx`, `dy`)，平移后落在新画布外的像素继续保留。`fill` 不透明时，新画布中原画布（平移后）没有覆盖的区域填 `fill`（背景图层的扩展色）；原画布内本来透明的像素保持透明。这是 Canvas Size、移动工具和 Reveal All 的底层实现。
- `to_rgba8()`：画布部分转为紧密排列的 RGBA8 缓冲区。
- `region_rgba8(x0, y0, w, h)`：任意区域（可伸到画布外）的像素，按 tile 逐行段拷贝。
- `from_region(width, height, x0, y0, w, h, pixels)`：画布尺寸为 `width`×`height`、内容只有给定区域像素的图像，用 `write_span_at` 逐行写入，全透明 tile 丢弃。自由变换用这两个方法在画布外也能处理像素。
- `remapped(width, height, source)`：新尺寸的图像，像素 (x, y) 取原图画布内像素 `source(x, y)`。用于旋转和翻转；画布外的像素不参与（见已知限制）。
- `allocated_tiles()`：已分配的 tile 数，用于调试和内存统计。

## 行为规则与不变量

- 缺失的 tile 等价于全透明 `[0, 0, 0, 0]`，不占内存。
- 克隆 `TiledImage` 只复制 tile 指针表，不复制像素；历史快照因此也完整保留画布外的像素。
- `with_canvas` 逐个已分配的源 tile、逐行把像素写到目标位置（`write_span_at` 跨目标 tile 拆分一行；整段透明且目标 tile 缺失时不分配），然后按行段填 `fill`，最后丢掉全透明的 tile。内存占用与已分配的 tile 数成正比。
- 「全透明」的判定只看 alpha：所有像素 alpha 为 0 的 tile 被视为可丢弃，即使其 RGB 分量非零。
- 合成、缩略图等只遍历画布范围内的 tile、只读画布内的像素，画布外的像素不显示。

## 边界情况

- 宽或高为 0：`tiles_x()`/`tiles_y()` 为 0，读取画布内像素都返回透明。
- `from_rgba8` 用 `assert_eq!` 校验缓冲区长度等于 `width * height * 4`，不匹配时 panic。
- `tile_mut` 分配或修改 tile 后不会再检查它是否变为全透明；被擦成全透明的 tile 仍保持分配状态，直到下一次 `with_canvas` 或 `clipped` 重建图像。

## 与其它模块的关系

- `layer.rs`：`LayerKind::Raster` 持有一个 `TiledImage`。
- `document.rs`：新建/打开文档时用 `filled`/`from_rgba8` 建图；`place_canvas`（Canvas Size、Reveal All）调用 `with_canvas`，背景图层再 `clipped`；`content_bounds` 汇总各图层的 `content_bounds`；`composite_rgba8` 直接遍历 `tile()` 读取像素。
- `move_tool.rs`、`clipboard.rs`（粘贴）、`op-io` 的 PSD 读写用 `pixel_at`/`set_pixel_at`/`content_bounds` 处理画布外的像素。
- `history.rs`：通过 `Snapshot` 克隆图层，依赖本模块的 `Arc` 共享来控制内存。
- `op-ui` 读取图层的 `TiledImage` 生成图层缩略图。

## 已知限制

- 只支持 8 位 RGBA 存储，没有 16 位或浮点 tile。
- 没有 mipmap 或多分辨率层级。
- 写入后不会自动回收变为全透明的 tile。
- 以画布大小缓冲区重建图层的操作会丢掉画布外的像素：`remapped`（图像旋转、画布翻转）、Image Size 的重采样、合并图层（Merge Down/Visible/Layers）。Photoshop 中这些操作会连同画布外的像素一起处理。

## 测试覆盖

- `round_trip_and_sparse`：300×10 的图像中只有第二个 tile 列有一个不透明像素，构建后只分配 1 个 tile，该像素读回正确，其他位置读回透明。
- `with_canvas_grows_and_crops`：2×2 图像放大到 4×4 并偏移 (1,1)，四周为填充色、中间为原像素；以 (-1,-1) 裁剪到 1×1 得到原图右下角像素。
- `with_canvas_across_tiles`：宽度跨两个 tile 的图像平移 (300, 1) 后，每个像素都落在正确位置，未覆盖区域透明，且完全未覆盖的第一列 tile 保持未分配。
- `copy_on_write`：克隆后对副本 `tile_mut` 写入，不影响原图。
- `filled_leaves_nothing_past_the_canvas`：10×10 纯色图像在 (10, 0) 读出透明，内容范围正好是画布，没有画布外像素。
- `moving_keeps_pixels_outside_the_canvas`：平移 (−300, 5) 后像素落到画布左边很远处并被保留，内容范围正确；再平移回来两个像素都复原。
- `clipped_drops_what_is_outside`：左边、右边（边缘 tile 内）和下方 tile 中的画布外像素都被删掉，只剩画布内的一个 tile。
- `regions_reach_outside_the_canvas`：读出伸到画布左上外和右边很远处的区域，再用 `from_region` 写回，内容范围和像素都不变。
- `opaque_fill_only_covers_new_canvas_area`：用不透明色扩展画布时只填新增区域，原画布内的透明像素保持透明，结果没有画布外像素。
- `content_bounds_match_a_full_scan`：随机稀疏点（含画布外）与逐点计算一致；3000 × 1080 的大块内容外框准确，10 次调用少于 20 ms。
