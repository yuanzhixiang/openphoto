# pixel.rs

## 职责

定义文档级的像素格式描述：颜色模式 `ColorMode`（对应 Photoshop 的 Image > Mode）和每通道位深 `BitDepth`。它们是文档的元数据，供界面展示和选择；目前不影响实际的像素存储格式。

## 对外接口

### `ColorMode`

- 取值：`Bitmap`、`Grayscale`、`Indexed`、`Rgb`（默认）、`Cmyk`、`Lab`、`Multichannel`。
- `ALL`：按上述顺序列出全部 7 个取值，与 Photoshop Image > Mode 菜单的顺序一致，供菜单/下拉框遍历。
- `label()`：完整英文名，例如 `"RGB Color"`、`"Indexed Color"`、`"CMYK Color"`。
- `short()`：文档标签页上的短名，例如 `"RGB"`、`"Gray"`、`"Index"`，与位深一起组成 `RGB/8` 这类显示。

### `BitDepth`

- 取值：`U8`（默认）、`U16`、`F32`。
- `ALL`：按 8、16、32 的顺序列出。
- `bits()`：返回 8、16、32。
- `label()`：`"8 Bits/Channel"`、`"16 Bits/Channel"`、`"32 Bits/Channel"`。

## 行为规则

- 新建和打开的文档总是 `ColorMode::Rgb` + `BitDepth::U8`（见 `document.rs`）。
- `Document` 的 `color_mode`、`bit_depth` 是公开字段，可以被改写，但改写只改变元数据，不会转换任何像素。
- 这两个字段不属于 `Snapshot`，因此不参与撤销/重做。

## 与其它模块的关系

- `Document` 持有这两个字段。
- `op-ui` 的属性面板遍历 `ColorMode::ALL` 与 `BitDepth::ALL` 展示选项，文档标签页使用 `short()` 与 `bits()`。

## 已知限制

- 只实现了 RGB 颜色模式；其余 6 种颜色模式只是枚举值，没有对应的像素表示或转换。
- 像素存储只有 8 位；`U16` 与 `F32` 没有对应的存储实现。

## 测试覆盖

本文件没有单元测试。
