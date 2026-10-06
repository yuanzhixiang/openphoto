# layer.rs

## 职责

定义图层及其属性：图层 ID、混合模式、图层内容类型，以及可见性、不透明度、锁定等 Photoshop 图层属性。

## 对外接口

### `LayerId`

不透明的 `u64` 包装，可比较、可哈希。ID 由 `Document::new_layer_id()` 从进程级计数器分配（与 `DocId` 共用计数器），本模块不负责分配。

### `BlendMode`

- 收录 Photoshop 全部 27 种图层混合模式，枚举顺序与 Photoshop 混合模式菜单一致，默认值为 `Normal`。
- `GROUPS`：按 Photoshop 菜单分成 6 组，组与组之间画分隔线：
  1. `Normal`、`Dissolve`
  2. `Darken`、`Multiply`、`ColorBurn`、`LinearBurn`、`DarkerColor`
  3. `Lighten`、`Screen`、`ColorDodge`、`LinearDodge`、`LighterColor`
  4. `Overlay`、`SoftLight`、`HardLight`、`VividLight`、`LinearLight`、`PinLight`、`HardMix`
  5. `Difference`、`Exclusion`、`Subtract`、`Divide`
  6. `Hue`、`Saturation`、`Color`、`Luminosity`
- `label()`：菜单显示的英文名，与 Photoshop 措辞一致，例如 `LinearDodge` 显示为 `"Linear Dodge (Add)"`。

### `LayerKind`

图层内容类型。目前只有 `Raster(TiledImage)`：栅格图层，图像尺寸与文档一致。

### `Layer`

公开字段：

- `id`、`name`
- `visible`：是否参与合成。
- `opacity`：图层不透明度，语义上作用于整个图层（包括图层样式）。
- `fill`：填充不透明度，语义上只作用于像素本身，不作用于图层样式。
- `blend_mode`
- `is_background`：「背景」图层标记。按 Photoshop 语义，背景图层锁定、不透明、总在最底层。
- `lock_transparency`、`lock_pixels`、`lock_position`：三种锁定。
- `kind`：图层内容。

`Layer::raster(id, name, image)` 构造一个普通栅格图层：可见、`opacity` 与 `fill` 均为 1.0、Normal 混合、非背景、无任何锁定。

`is_locked()`：当图层是背景图层，或开启了 `lock_pixels` 或 `lock_position` 时返回 `true`。`lock_transparency` 不计入；它在 Photoshop 中属于部分锁定。

## 行为规则

- `opacity` 与 `fill` 是 0..=1 的浮点数，本模块不做范围校验。
- 背景图层的「锁定、不透明、在最底层」是语义约定，本模块不强制：字段都是公开的，由调用方（文档构造函数和 `op-ui` 图层面板）维护。`op-ui` 图层面板对背景图层禁用混合模式、不透明度和锁定开关的编辑。
- 合成时 `opacity * fill` 作为图层整体 alpha 系数（见 `document.md`）；由于目前没有图层样式，两者在效果上没有区别。
- `blend_mode` 在合成时生效，算法见 `blend.md`。

## 与其它模块的关系

- `document.rs` 持有 `Vec<Layer>`，并在构造时设置背景图层。
- `op-ui` 图层面板用 `BlendMode::GROUPS` 和 `label()` 构建混合模式菜单，用 `is_locked()` 决定锁图标状态，直接修改 `Layer` 字段后调用 `Document::mark_dirty()`。

## 已知限制

- 只有栅格图层，没有图层组、调整图层、文字图层、形状图层或智能对象。
- 没有图层蒙版、剪贴蒙版和图层样式（因此 `fill` 与 `opacity` 在效果上相同）。
- 锁定标志只是属性，本 crate 内没有编辑操作去检查它们。

## 测试覆盖

本文件没有单元测试。
