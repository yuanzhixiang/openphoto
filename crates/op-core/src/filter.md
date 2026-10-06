# filter.rs：滤镜

## 职责

实现 Filter 菜单中的滤镜，作用于活动图层、限于选区。不记录历史（由 `op-ui` 按滤镜名称记录）。

## 对外接口

- `Filter`（括号内为参数）：
  - `GaussianBlur { radius }`：高斯模糊，`radius` 作为标准差（像素），核半径取 `ceil(3σ)`，行列分离卷积。
  - `BoxBlur { radius }`：(2r + 1)² 方框内的平均，行列分离。
  - `Average`：选区（没有选区时整个图层）内所有像素的平均颜色（按选择程度加权）填满选区。
  - `UnsharpMask { amount, radius, threshold }`：先以 `radius` 做高斯模糊，原值与模糊值之差的绝对值大于 `threshold` 时，结果为 `原值 + 差 × amount%`。只处理颜色通道。
  - `AddNoise { amount, gaussian, monochromatic }`：每个通道加 `n × amount% × 127.5`，`n` 为 −1–1 的均匀分布，或单位方差的近似正态分布（4 个均匀随机数之和）。单色时三个通道使用同一个随机数。随机数由像素坐标和通道决定，同样的参数总是得到同样的结果，预览与最终结果一致。
  - `Median { radius }`、`Minimum { radius }`、`Maximum { radius }`：每个通道分别取 (2r + 1)² 方框内的中位数、最小值、最大值。
  - `HighPass { radius }`：`原值 − 高斯模糊值 + 128`。
  - `Offset { dx, dy, fill }`：图层整体右移 `dx`、下移 `dy`。空出的区域按 `OffsetFill`：`Background`（普通图层为透明，背景图层为背景色）、`RepeatEdges`（重复边缘像素）、`Wrap`（从另一侧绕回）。
  - `Mosaic { cell }`：从左上角起把图层划成 `cell` × `cell` 的格子，每格填其平均颜色（右、下边缘的格子可能不完整）。
  - `Solarize`：每个颜色通道大于 127 的值取反（`255 − v`）。
- `Filter::name()`：菜单与历史名称（「Gaussian Blur」「Box Blur」「Average」「Unsharp Mask」「Add Noise」「Median」「Minimum」「Maximum」「High Pass」「Offset」「Mosaic」「Solarize」）。
- `apply(doc, filter, background)`：先做与调整相同的检查（`adjust::check`：没有图层、图层隐藏、像素锁定时返回 `FillError`），再应用。`background` 是 Offset 在背景图层上使用的背景色。

## 行为规则

- 滤镜读取整个图层计算（选区边缘的模糊会用到选区外的像素），只写入选中的像素；部分选中的像素按选择程度在原值与新值之间混合（包括 alpha）。
- 模糊、排序类滤镜在预乘 alpha 下计算，透明像素的颜色不会渗入相邻像素；结果再转回直通 alpha。
- 图层边界外按最近的边缘像素延伸（clamp）。
- 背景图层和锁定透明像素的图层保持原 alpha。

## 已知限制

- 全部在 CPU 上单线程计算，大图大半径时较慢；Gaussian Blur 没有用近似加速。
- Median/Minimum/Maximum 使用方形范围、整数半径（Photoshop 是圆形范围，Minimum/Maximum 还支持小数半径和「保持方形/圆度」选项）。
- 参数与 Photoshop 结果的数值对应（例如 Gaussian Blur 的半径、Add Noise 的强度）尚未与 Photoshop 核对。

## 测试覆盖

- `blurs_spread_the_dark_pixel`：5×1 白底中间一个黑点，Box Blur 半径 1 得到 `[255, 170, 170, 170, 255]`；Gaussian Blur 对称且由中心向外变亮；Average 得到平均值 204。
- `rank_filters`：Median 去掉孤立黑点，Minimum 扩大黑点，Maximum 去掉黑点。
- `sharpen_high_pass_and_solarize`：Unsharp Mask 保持边缘的黑白；High Pass 中心低于 128、边缘高于 128；Solarize 把白变黑。
- `offset_wraps_or_fills_with_the_background`：绕回与用背景色填充。
- `mosaic_and_noise`：Mosaic 一格取平均；Add Noise 重复执行结果相同，单色噪点保持灰色。
- `transparent_pixels_do_not_bleed_color`：透明图层上的红点模糊后仍是红色，alpha 变为 85。
