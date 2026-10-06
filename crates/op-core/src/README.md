# op-core：文档模型

## 定位

`op-core` 是 OpenPhoto 的文档数据层：图层列表、分块（tile）像素存储、像素格式枚举、颜色值和撤销历史都定义在这里。每一次编辑最终都落实为对这些数据结构的修改；UI（`op-ui`）和渲染（`op-render`）只读取或驱动这些结构。

本 crate 没有任何外部依赖（`Cargo.toml` 的 `[dependencies]` 为空），不依赖 GPU、窗口系统或 UI 框架。这是承重约束：它保证文档模型可以脱离界面做单元测试，也让 `op-io`、`op-color`、`op-tools` 只依赖 `op-core` 即可工作。

## 模块划分

| 文件 | 内容 |
| --- | --- |
| `lib.rs` | 模块声明与常用类型的重导出 |
| `color.rs` | `Color`：直通（非预乘）RGBA 浮点颜色 |
| `pixel.rs` | `ColorMode`（文档颜色模式）、`BitDepth`（位深） |
| `tile.rs` | `Tile`、`TiledImage`：稀疏、写时复制的分块像素存储，`TILE_SIZE = 256` |
| `layer.rs` | `Layer`、`LayerId`、`LayerKind`、`BlendMode` |
| `document.rs` | `Document`、`DocId`、`Snapshot`、`Anchor`：文档本体、画布尺寸调整、CPU 合成 |
| `history.rs` | `History`：仿 Photoshop 历史记录面板的线性撤销历史 |

`lib.rs` 在 crate 根重导出 `Color`、`Anchor`、`DocId`、`Document`、`Snapshot`、`History`、`BlendMode`、`Layer`、`LayerId`、`LayerKind`、`BitDepth`、`ColorMode`、`TILE_SIZE`、`Tile`、`TiledImage`。`HistoryState` 与 `history::DEFAULT_LIMIT` 不在根上重导出，需要通过 `op_core::history::` 路径访问。

## 文档模型

- 一个 `Document` 有固定的像素宽高、分辨率（ppi）、颜色模式、位深，以及按「自底向上」排列的图层列表 `layers` 和当前活动图层 `active_layer`。
- 目前唯一的图层类型是栅格图层（`LayerKind::Raster(TiledImage)`）。每个图层的 `TiledImage` 与文档同宽高；`Document::resize_canvas` 会同步改写所有图层，保持这一不变量。
- 「背景」图层（`is_background = true`）沿用 Photoshop 语义：锁定、不透明、总在最底层。新建文档得到单一的背景图层，名为 `"Background"`。打开位图文件时，完全不透明的图片同样得到背景图层；带透明的图片得到名为 `"Layer 0"` 的普通图层，与 Photoshop 一致。
- `DocId` 与 `LayerId` 共用同一个进程级原子计数器（从 1 开始递增），因此在整个进程内两类 ID 互不重复、也不会在文档之间重复。
- `Document` 维护一个 `revision` 计数器。任何像素或图层属性变化都需要调用 `mark_dirty()` 递增它；渲染层据此判断是否重新上传纹理。`op-core` 内部只有 `resize_canvas` 和 `restore` 会自动递增，直接修改公开字段（例如图层属性）的调用方必须自己调用 `mark_dirty()`。

## 为什么分块存储是稀疏且写时复制的

- 像素按 256×256 的 tile 存放在 `HashMap<(tx, ty), Arc<Tile>>` 中。缺失的 tile 视为全透明，不占内存；从 RGBA 缓冲区构建或调整画布时，全透明的 tile 会被丢弃。这让大画布上的稀疏图层（例如只画了几笔的新图层）几乎不占内存。
- 纯色填充（`TiledImage::filled`）让所有 tile 共享同一个 `Arc<Tile>`，一张纯色背景只占一个 tile 的内存。
- tile 通过 `Arc` 共享：克隆 `TiledImage` 只复制指针；写入时 `tile_mut` 用 `Arc::make_mut` 在 tile 被共享时先复制一份。这样快照与当前文档可以共享未改动的 tile。

## 历史快照如何共享 tile

- `Snapshot` 是文档中可撤销的部分：宽、高、分辨率、图层列表、活动图层。克隆它的代价是克隆图层元数据和 tile 指针表，而不是像素。
- `History` 的每个状态都保存一份完整快照。由于 tile 在快照之间共享，只有被编辑过的 tile 会额外占用内存。
- 快照是完整的（不是增量），所以历史超出上限时可以安全地删除中间任意状态。

## 为什么合成在 gamma 空间

`Document::composite_rgba8` 直接在 sRGB 编码值上做混合，不先转换到线性光。这与 Photoshop 的默认行为一致（Photoshop 默认不启用「用灰度系数混合 RGB 颜色」的线性混合），因此同样的图层叠加得到与 Photoshop 一致的数值，例如 50% 不透明度的黑色叠在白色上得到 128 灰。`Color` 的分量也约定为 sRGB 编码值。

## 跨文件规则

- 像素格式：所有像素存储为 8 位 RGBA、直通 alpha（非预乘）。`BitDepth` 和 `ColorMode` 的其他取值只是可选择的枚举值，不改变存储格式。
- 颜色量化：`Color::to_rgba8` 与合成输出都采用「clamp 到 0..=1，乘 255，加 0.5 后截断」的四舍五入规则。
- 越界读取：`TiledImage::pixel` 对越界坐标返回透明 `[0, 0, 0, 0]`，不会 panic。
- Photoshop 对齐：混合模式的顺序与分组、背景图层语义、历史记录条数上限 50、Toggle Last State 的往返语义、裁剪历史时保留第一条状态，均以 Photoshop 为参照。

## 已知限制

- 只有栅格图层，没有图层组、调整图层、文字图层、智能对象、蒙版或图层样式。
- 只支持 8 位 RGB 存储；16 位、32 位和非 RGB 颜色模式没有实现。
- 合成在 CPU 上逐像素完成（混合模式算法见 `blend.md`）。
- 没有选区、没有色彩管理（ICC）。
