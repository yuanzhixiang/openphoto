# filter.rs：滤镜

## 职责

实现 Filter 菜单中的滤镜，作用于活动图层、限于选区。不记录历史（由 `op-ui` 按滤镜名称记录）。

## 对外接口

- `Filter`（括号内为参数）：
  - `GaussianBlur { radius }`：按 Photoshop 2026 实测：半径（Photoshop 以 0.1 为单位）≤ 2 时用 Photoshop 自己的 8 位整数核（`SMALL_KERNELS`，中心与一侧的权重，总和 256；小半径的核明显比 σ = r 的高斯宽）；2.1–2.9 用实测的浮点核（`MID_KERNELS`，2.2–2.4、2.5–2.6 共用同一个）；≥ 3 时用五个方差各为 r²/5 的扩展方框（Gwosdek 等人的 extended box，整数半径 l 加两端小数权重 α）卷积成一个核，一次行列分离卷积、边缘重复。与 Photoshop 相差不超过 1 级（探测图 0.3、1、2.5、10 四个半径）。
  - `BoxBlur { radius }`：(2r + 1)² 方框内的平均，行列分离。
  - `Average`：选区（没有选区时整个图层）内所有像素的平均颜色（按选择程度加权）填满选区。
  - `UnsharpMask { amount, radius, threshold }`：以 `radius` 做上述高斯模糊，锐化量 `Δ = (原值 − 模糊值) × amount%`；阈值不是开关，而是从锐化量里减去：结果 = `原值 + sign(Δ) × max(0, |Δ| − threshold)`（Photoshop 实测，相差不超过 2 级）。只处理颜色通道。
  - `AddNoise { amount, gaussian, monochromatic }`：每个通道加 `n × amount% × 127.5`，`n` 为 −1–1 的均匀分布，或单位方差的近似正态分布（4 个均匀随机数之和）。单色时三个通道使用同一个随机数。随机数由像素坐标和通道决定，同样的参数总是得到同样的结果，预览与最终结果一致。
  - `Median { radius }`：每个通道取 (2r + 1)² 方框内的中位数（与 Photoshop 一致）。
  - `Minimum { radius, round }`、`Maximum { radius, round }`：Preserve 为 Squareness 时取半径四舍五入后的方框内的最小/最大值（与 Photoshop 一致，2.5 即 7 × 7）；Roundness 时用圆形窗口，边缘像素按覆盖率 `clamp(1 − (距离 − 半径), 0, 1)` 参与（结果为 `原值 + (邻值 − 原值) × 覆盖率` 的最小/最大），是近似，个别像素与 Photoshop 相差较大。
  - `HighPass { radius }`：`原值 − 高斯模糊值 + 128`。
  - `Offset { dx, dy, fill }`：图层整体右移 `dx`、下移 `dy`。空出的区域按 `OffsetFill`：`Background`（普通图层为透明，背景图层为背景色）、`RepeatEdges`（重复边缘像素）、`Wrap`（从另一侧绕回）。
  - `Mosaic { cell }`：从左上角起把图层划成 `cell` × `cell` 的格子，每格填其平均颜色（右、下边缘的格子可能不完整）。
  - `Solarize`：每个颜色通道大于 127 的值取反（`255 − v`）。
  - `Blur`：3 × 3 核 [[0,1,0],[1,4,1],[0,1,0]] / 8；`BlurMore`：[[1,2,1],[2,2,2],[1,2,1]] / 14；`Sharpen`：中心 2、上下左右 −¼；`SharpenMore`：中心 3、八邻 −¼。均由脉冲实测，与 Photoshop 逐级相同。
  - `FindEdges`：每个通道 `255 − √(gx² + gy²)`，gx、gy 为不归一化的 Sobel 核（与 Photoshop 相差不超过 1 级）。
  - `MotionBlur { angle, distance }`：沿角度方向每隔 1 像素取 `distance + 1` 个点（从 `−ceil(distance/2)` 起），各以双线性分摊，平均后作为卷积核（脉冲响应即这些点，读取方向相反）。水平时与 Photoshop 逐级相同；斜向是近似（内部相差约 12 级，图像边缘处 Photoshop 的边界处理不同，差得更多）。
  - `Emboss { angle, height, amount }`：每个通道 `128 + (I(p + s) − I(p − s)) × amount%`，`s = (height/2)·(cos a, −sin a)`，双线性取样。水平、竖直时与 Photoshop 一致；斜向时 Photoshop 的取样更集中，存在误差（探测图平均 0.2–0.4 级，最大 31 级）。
- `Filter::name()`：菜单与历史名称（「Gaussian Blur」「Box Blur」「Average」「Unsharp Mask」「Add Noise」「Median」「Minimum」「Maximum」「High Pass」「Offset」「Mosaic」「Solarize」「Blur」「Blur More」「Sharpen」「Sharpen More」「Find Edges」「Motion Blur」「Emboss」）。
- `apply(doc, filter, background)`：先做与调整相同的检查（`adjust::check`：没有图层、图层隐藏、像素锁定时返回 `FillError`），再应用。`background` 是 Offset 在背景图层上使用的背景色。

## 行为规则

- 滤镜读取整个图层计算（选区边缘的模糊会用到选区外的像素），只写入选中的像素；部分选中的像素按选择程度在原值与新值之间混合（包括 alpha）。
- 模糊、排序类滤镜在预乘 alpha 下计算，透明像素的颜色不会渗入相邻像素；结果再转回直通 alpha。
- 图层边界外按最近的边缘像素延伸（clamp）。
- 背景图层和锁定透明像素的图层保持原 alpha。

## 已知限制

- 全部在 CPU 上单线程计算，大图大半径时较慢。
- Add Noise 的强度、Mosaic 以外的 Pixelate 滤镜等尚未与 Photoshop 核对；Sharpen Edges、Despeckle 的算法（带阈值的有条件锐化/模糊）尚未还原。

## 测试覆盖

- `blurs_spread_the_dark_pixel`：5×1 白底中间一个黑点，Box Blur 半径 1 得到 `[255, 170, 170, 170, 255]`；Gaussian Blur 对称且由中心向外变亮；Average 得到平均值 204。
- `rank_filters`：Median 去掉孤立黑点，Minimum 扩大黑点，Maximum 去掉黑点。
- `sharpen_high_pass_and_solarize`：Unsharp Mask 保持边缘的黑白；High Pass 中心低于 128、边缘高于 128；Solarize 把白变黑。
- `offset_wraps_or_fills_with_the_background`：绕回与用背景色填充。
- `mosaic_and_noise`：Mosaic 一格取平均；Add Noise 重复执行结果相同，单色噪点保持灰色。
- `transparent_pixels_do_not_bleed_color`：透明图层上的红点模糊后仍是红色，alpha 变为 85。
- `photoshop::filters_match_photoshop`：26 组滤镜对照 Photoshop 的输出（`fixtures/filter`），每组限定最大误差。
