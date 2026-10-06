# op-color：颜色模型转换

## 职责

在 `op_core::Color`（sRGB 编码的 RGBA 浮点颜色）与拾色器使用的其他表示之间转换：HSB、Lab、CMYK、6 位十六进制，以及网页安全色。全部是纯数学换算，没有 ICC 色彩管理。

## 对外接口

### `Hsb`

Photoshop 拾色器中的 HSB：`h` 为色相，取值 0..360 度；`s`（饱和度）与 `b`（亮度）取值 0..=1。字段公开。

- `Hsb::from_color(c)`：标准 RGB→HSV 换算。`b` 为三通道最大值；`s = (max − min) / max`，`max` 为 0 时 `s = 0`；无彩色（`max == min`）时 `h = 0`。忽略 `c.a`。
- `Hsb::to_color()`：标准 HSV→RGB 换算，输出 alpha 恒为 1。色相先对 360 取欧几里得余数，因此负值和大于等于 360 的值都会被折回 0..360。

### `Lab`

CIELAB，参考白点为 D50，与 Photoshop 拾色器显示的数值一致（例如 #00afdc 为 L66、a−27、b−34，四舍五入后完全相同）。`l` 为 0–100，`a`、`b` 大致为 −128–127。

- `Lab::from_color(c)`：sRGB → 线性 RGB → XYZ（D65）→ Bradford 色适应到 D50 → Lab。两步矩阵合并为一个常量矩阵。忽略 alpha。
- `Lab::to_color()`：反向换算。超出 sRGB 色域的结果在线性空间里截断到 0–1，所以返回的颜色总在 sRGB 内。alpha 恒为 1。

### `Cmyk`

C、M、Y、K 四个 0–1 的分量。

- `Cmyk::from_color(c)`：与设备无关的公式：K = 1 − max(R,G,B)，C = (1 − R − K) / (1 − K)，M、Y 同理；纯黑为 C=M=Y=0、K=1。
- `Cmyk::to_color()`：R = (1 − C)(1 − K)，G、B 同理。
- 这不是 Photoshop 的 CMYK：Photoshop 通过 CMYK 工作空间的 ICC 配置文件（默认 U.S. Web Coated SWOP）换算，数值不同（#00afdc 在 Photoshop 中为 C72 M10 Y6 K0，这里为 C100 M20 Y0 K14）。

### 网页安全色

- `web_safe(c)`：每个通道先按 8 位量化，再取最接近的 0x33 的倍数（0、51、102、153、204、255），保留 alpha。
- `is_web_safe(c)`：三个通道按 8 位量化后都是 51 的倍数。

### 十六进制

- `to_hex(c)`：返回不带 `#` 的 6 位小写十六进制字符串，例如 `"1fa3e0"`。先经 `Color::to_rgba8` 四舍五入量化，丢弃 alpha。
- `from_hex(s)`：解析 6 位十六进制。先去除首尾空白和开头的 `#`（可有可无），剩余部分必须正好 6 个字节，否则返回 `None`。大小写均可。结果 alpha 恒为 1。

## 边界情况

- `from_color` 在饱和度为 0 时总是给出 `h = 0`，所以灰色经过 HSB 往返后会丢失原来的色相；拾色器若需要在灰色上保留色相，需要自行保存 `Hsb` 值（`op-ui` 就是单独保存 `picker_hsb`）。
- `to_color` 不 clamp `s` 和 `b`；超出 0..=1 的输入会产生超出范围的分量，直到 `to_rgba8` 时才被截断。
- `from_hex` 只接受 6 位形式，不接受 3 位简写（如 `fff`）或带 alpha 的 8 位形式。
- `from_hex` 使用 `u32::from_str_radix` 解析，因此 6 字节中以 `+` 开头的字符串（例如 `"+12345"`）也会被接受。
- 开头的多个 `#` 都会被去掉。

## 与其它模块的关系

- 依赖 `op-core`（`Color`）。
- `op-ui` 的 Color 面板使用 `Hsb`；Color Picker 使用这里的全部表示（见 `crates/op-ui/src/dialogs/color_picker.md`）。

## 已知限制

- 没有 ICC 色彩管理，所有颜色都按 sRGB 编码值直接处理；CMYK 因此与 Photoshop 不一致。
- 没有灰度等其它颜色模式的换算。

## 测试覆盖

- `hsb_round_trip`：纯红、一个任意中间色、50% 灰、黑色经 HSB 往返后，8 位量化结果与原色一致。
- `hex`：`"#1fa3e0"` 解析后再输出为 `"1fa3e0"`。
- `lab_matches_photoshop`：#00afdc 的 Lab 四舍五入为 (66, −27, −34)，与 Photoshop 相同；白色为 (100, 0, 0)。
- `lab_round_trip`：若干颜色经 Lab 往返后十六进制不变。
- `cmyk_round_trip`：经 CMYK 往返不变，黑色的 K 为 1。
- `web_safe_snapping`：#00afdc 不是网页安全色，对齐后为 #0099cc。
