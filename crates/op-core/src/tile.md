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

- `new(width, height)`：空图像，不分配任何 tile，整体视为全透明。
- `filled(width, height, rgba)`：纯色图像。alpha 为 0 时等同于 `new`，不分配 tile；否则覆盖图像的每个 tile 位置都指向同一个共享的 `Arc<Tile>`。
- `from_rgba8(width, height, pixels)`：从紧密排列的 RGBA8 缓冲区构建，逐 tile 拷贝，全透明（所有像素 alpha 均为 0）的 tile 被丢弃。
- `width()`、`height()`：图像尺寸。
- `tiles_x()`、`tiles_y()`：横向/纵向 tile 数，按 `div_ceil(TILE_SIZE)` 计算。
- `tile(tx, ty)`：只读访问某个 tile；未分配时返回 `None`（语义上是全透明）。
- `tile_mut(tx, ty)`：可写访问。tile 缺失时分配一个全透明 tile；tile 被其他图像共享时先复制一份（写时复制），因此写入永远不会影响共享该 tile 的快照。
- `pixel(x, y)`：读取单个像素。
- `with_canvas(width, height, dx, dy, fill)`：生成一张新尺寸的图像，把当前图像放在 (`dx`, `dy`)（可为负），新图像中不被原图覆盖的区域填 `fill`。这是 Canvas Size 的底层实现。
- `to_rgba8()`：整幅图像转为紧密排列的 RGBA8 缓冲区（未分配的 tile 为全透明）。
- `remapped(width, height, source)`：生成新尺寸的图像，新图像的像素 (x, y) 取原图像素 `source(x, y)`。用于旋转和翻转；整幅图像先展开成缓冲区再重建，结果中全透明的 tile 不保留。`source` 必须返回原图范围内的坐标。
- `allocated_tiles()`：已分配的 tile 数，用于调试和内存统计。

## 行为规则与不变量

- 缺失的 tile 等价于全透明 `[0, 0, 0, 0]`，不占内存。
- `pixel` 对 `x >= width` 或 `y >= height` 返回 `[0, 0, 0, 0]`，不会 panic。
- 克隆 `TiledImage` 只复制 tile 指针表（`HashMap<(u32, u32), Arc<Tile>>`），不复制像素。
- `with_canvas` 逐个目标 tile 处理：对每个目标 tile 先算出它被原图覆盖的源区域，只有覆盖区域的像素会被逐行拷贝（`read_span` 跨源 tile 拼接一行，源 tile 缺失的部分填透明）。不被原图覆盖且 `fill` 的 alpha 为 0 的目标 tile 直接跳过不分配；生成后全透明的目标 tile 也不保留。因此内存占用与实际分配的 tile 数成正比，透明区域保持稀疏。
- `with_canvas` 总是生成新分配的 tile，不与原图共享 tile，即使偏移恰好是 tile 对齐的。
- 「全透明」的判定只看 alpha：所有像素 alpha 为 0 的 tile 即被视为可丢弃，即使其 RGB 分量非零。

## 边界情况

- 宽或高为 0：`tiles_x()`/`tiles_y()` 为 0，不分配任何 tile，所有读取都返回透明。
- `from_rgba8` 用 `assert_eq!` 校验缓冲区长度等于 `width * height * 4`，不匹配时 panic。该乘积按 `u32` 计算，像素数极大（超过约 10.7 亿像素）时会溢出。
- `tile_mut` 不检查 tile 坐标是否在图像范围内，越界坐标也会分配一个 tile，且该 tile 不会被 `pixel`、合成或 `with_canvas` 读取。
- `tile_mut` 分配或修改 tile 后不会再检查它是否变为全透明；被擦成全透明的 tile 仍保持分配状态，直到下一次 `with_canvas` 重建图像。
- `with_canvas` 中 `fill` 的 alpha 为 0 但 RGB 非零时：未被原图覆盖的目标 tile 被跳过（读出为 `[0,0,0,0]`），而与原图部分重叠的目标 tile 中未覆盖的像素保留 `fill` 的 RGB 与 0 alpha。两者在视觉上都是透明，只是存储值不同。
- 位于原图 tile 边缘、超出图像宽高的存储填充区不会被 `with_canvas` 拷贝到新图像。

## 与其它模块的关系

- `layer.rs`：`LayerKind::Raster` 持有一个 `TiledImage`。
- `document.rs`：新建/打开文档时用 `filled`/`from_rgba8` 建图；`resize_canvas` 调用 `with_canvas`；`composite_rgba8` 直接遍历 `tile()` 读取像素。
- `history.rs`：通过 `Snapshot` 克隆图层，依赖本模块的 `Arc` 共享来控制内存。
- `op-ui` 读取图层的 `TiledImage` 生成图层缩略图。

## 已知限制

- 只支持 8 位 RGBA 存储，没有 16 位或浮点 tile。
- 没有 mipmap 或多分辨率层级。
- 写入后不会自动回收变为全透明的 tile。

## 逐像素写入

`set_pixel(x, y, rgba)` 写入一个像素（超出图像时忽略，需要时写时复制）。向尚不存在的 tile 写入全透明像素时不分配 tile。绘画使用它（见 `paint.md`）。

## 测试覆盖

- `round_trip_and_sparse`：300×10 的图像中只有第二个 tile 列有一个不透明像素，构建后只分配 1 个 tile，该像素读回正确，其他位置读回透明。
- `with_canvas_grows_and_crops`：2×2 图像放大到 4×4 并偏移 (1,1)，四周为填充色、中间为原像素；以 (-1,-1) 裁剪到 1×1 得到原图右下角像素。
- `with_canvas_across_tiles`：宽度跨两个 tile 的图像平移 (300, 1) 后，每个像素都落在正确位置，未覆盖区域透明，且完全未覆盖的第一列 tile 保持未分配。
- `copy_on_write`：克隆后对副本 `tile_mut` 写入，不影响原图。
