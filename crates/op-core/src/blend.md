# blend.rs：混合模式算法

## 职责

实现 Photoshop 全部 27 种图层混合模式的像素计算，供 `Document::composite_rgba8` 使用。公式采用 W3C Compositing and Blending 规范，它与 Photoshop 对这些模式的定义一致。所有数值都是直通 alpha（非预乘）、gamma 编码的 0–1 浮点数，混合在 gamma 空间进行（与 Photoshop 默认一致）。

## 对外接口

- `blend(mode, cb, cs)`：两个不透明颜色的混合结果 B(cb, cs)，`cb` 为下层（背景），`cs` 为上层（源）。
- `composite(mode, dst, src, src_alpha, x, y)`：把一个源像素合成到下层像素上。`src_alpha` 已包含像素 alpha、图层不透明度和填充。按 W3C 的通用公式：下层透明的部分直接显示源颜色，两者都存在的部分使用混合结果，然后做 source-over；`x`、`y` 是像素在文档中的位置，只有 Dissolve 用到。

## 各模式

| 模式 | 计算 |
|---|---|
| Normal | 源颜色 |
| Dissolve | 每个像素按「伪随机数 < alpha」决定完全显示或完全隐藏，没有半透明；随机数由像素位置决定，所以同一位置结果固定 |
| Darken / Lighten | 逐通道取较小 / 较大值 |
| Multiply / Screen | `b·s` / `b + s − b·s` |
| Color Burn | `b = 1` 时为 1，`s = 0` 时为 0，否则 `1 − min(1, (1 − b) / s)` |
| Color Dodge | `b = 0` 时为 0，`s = 1` 时为 1，否则 `min(1, b / (1 − s))` |
| Linear Burn / Linear Dodge (Add) | `max(0, b + s − 1)` / `min(1, b + s)` |
| Darker Color / Lighter Color | 比较三个通道之和，整体取较暗 / 较亮的颜色（不逐通道） |
| Overlay | 以下层为条件的 Hard Light，即 `HardLight(s, b)` |
| Soft Light | W3C 公式（`s ≤ 0.5` 时 `b − (1 − 2s)·b·(1 − b)`，否则用 `D(b)`） |
| Hard Light | `s ≤ 0.5` 时 `Multiply(b, 2s)`，否则 `Screen(b, 2s − 1)` |
| Vivid Light | `s ≤ 0.5` 时 `ColorBurn(b, 2s)`，否则 `ColorDodge(b, 2s − 1)` |
| Linear Light | `b + 2s − 1`，截断到 0–1 |
| Pin Light | `s ≤ 0.5` 时 `min(b, 2s)`，否则 `max(b, 2s − 1)` |
| Hard Mix | `b + s ≥ 1` 时为 1，否则为 0 |
| Difference / Exclusion | `|b − s|` / `b + s − 2bs` |
| Subtract / Divide | `max(0, b − s)` / `min(1, b / s)`（`s = 0` 时下层为 0 则 0，否则 1） |
| Hue / Saturation / Color / Luminosity | W3C 不可分离模式：亮度 `Lum = 0.3R + 0.59G + 0.11B`，用 SetLum、SetSat、ClipColor 组合源与下层的色相、饱和度、亮度 |

## 已知限制

- Photoshop 对 Color Burn、Linear Burn、Color Dodge、Linear Dodge、Linear Light、Vivid Light、Hard Mix、Difference 这 8 种模式中「填充（Fill）」与「不透明度」的效果做了区别处理，这里两者都只是简单相乘作为源 alpha。
- Soft Light 使用 W3C 公式，与 Photoshop 的结果在部分取值上有细微差别。
- Dissolve 的随机图案与 Photoshop 不同。

## 测试覆盖

- `separable_modes`：Multiply、Screen、Difference、Darken、Lighten、Linear Dodge、Subtract 的具体数值（例如 128 与 128 相乘为 64、滤色为 192），白色相乘、黑色滤色不改变颜色，Overlay 保持黑白下层。
- `non_separable_modes`：Color 保留下层亮度、采用源的色相；Luminosity 采用源亮度；以灰色为源的 Saturation 得到无彩色。
- `composite_over_transparent_shows_source`：下层透明时显示源颜色。
- `dissolve_is_all_or_nothing`：Dissolve 的像素只有显示和隐藏两种结果，显示比例接近 alpha。
