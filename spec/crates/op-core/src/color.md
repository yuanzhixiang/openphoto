# color.rs

## 职责

定义 `Color`：文档颜色空间中的直通（非预乘）RGBA 颜色，四个分量都是 `f32`，约定取值 0..=1。目前文档颜色空间总是 sRGB，所以分量是 sRGB 编码值而不是线性光值。前景色、背景色、画布扩展色等都用它表示。

## 对外接口

- 字段 `r`、`g`、`b`、`a` 全部公开，可直接读写。
- 常量：`BLACK`（0,0,0,1）、`WHITE`（1,1,1,1）、`TRANSPARENT`（0,0,0,0）。
- `rgb(r, g, b)`：alpha 固定为 1；`rgba(r, g, b, a)`：四个分量原样存入。二者都是 `const fn`，不做范围检查。
- `from_rgba8([u8; 4])`：每个分量除以 255。
- `to_rgba8()`：每个分量先 clamp 到 0..=1，再乘 255、加 0.5 后截断为 `u8`，即四舍五入量化。

## 行为规则与边界情况

- 构造函数不 clamp：超出 0..=1 的值会被保存下来，只在 `to_rgba8` 时才被截到 0 或 255。
- `from_rgba8` 后再 `to_rgba8` 对任意 8 位输入都能精确还原。
- `NaN` 分量经 `clamp` 后仍为 `NaN`，按 Rust 浮点转整数的饱和规则变为 0。
- `Color` 实现 `PartialEq`，按浮点逐分量比较。

## 与其它模块的关系

- `Document::new_with_background` 与 `Document::resize_canvas` 用 `to_rgba8` 把颜色写入像素。
- `op-color` 在它之上实现 HSB 与十六进制转换。
- `op-ui` 用它保存前景色/背景色等。

## 已知限制

- 只表示 RGB 颜色，不携带颜色空间或 ICC 信息。

## 测试覆盖

本文件没有单元测试；量化规则间接由 `document.rs` 与 `op-color` 的测试覆盖。
