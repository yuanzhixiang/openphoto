# adjust.rs：图像调整

## 职责

实现 Image › Adjustments 中逐像素的调整，作用于活动图层、限于选区。不记录历史（由 `op-ui` 按调整名称记录）。

## 对外接口

- `Adjustment`：
  - `Invert`：每个颜色通道取 `255 - v`。
  - `Desaturate`：三个通道都设为该像素的 HSL 亮度 `(max + min) / 2`（向上取整），与 Photoshop 的 Desaturate 一致（不是加权亮度）。
  - `Threshold(level)`：亮度（`luminosity`）大于等于 `level` 的像素变白，其余变黑。Photoshop 的取值范围 1–255，默认 128。
  - `Posterize(levels)`：每个通道量化为 `levels` 个色阶：`round(round(v × (L−1) / 255) × 255 / (L−1))`。范围 2–255，默认 4。
  - `Equalize`：直方图均衡。用活动图层选区内、alpha 不为 0 的像素的 R、G、B 三个通道值合在一起统计直方图（`channel_histogram`），每个值映射为它在累计直方图中的位置 × 255（四舍五入），同一张表作用于三个通道。
  - `Levels([Levels; 4])`：RGB 复合通道与红、绿、蓝通道各一组（`Levels { input_black, input_white, gamma, output_black, output_white }`），先应用各通道自己的，再应用复合通道的（Photoshop 的顺序）。`t = clamp((v − 输入黑场) / (输入白场 − 输入黑场), 0, 1)`，经 `levels_gamma(t, gamma)` 后映射到输出黑白场，半数进位取整。输入白场不大于黑场时按黑场 + 1 计算。`Levels::composite()` 只设复合通道。
  - `levels_gamma(t, gamma)`：Photoshop 的 gamma 曲线（从 Photoshop 2026 的 Levels 量得）：`t^(1/gamma)`，但 gamma 大于 1 时黑端斜率限制为 `2^gamma`：以二次曲线 `a·t + b·t²`（`a = 2^gamma`）从 0 起步，在与幂曲线斜率相同的点接上。gamma 8 以上幂曲线本身就不比它陡，没有这段。与 Photoshop 相差不超过 2 级（gamma 5–7 的最暗几级差 6 级以内）。
  - `HueSaturation(HueSaturation)`：Master 三个值、六个颜色范围（`HueRange { bounds, hue, saturation, lightness }`，默认边界 `HUE_RANGES`：Reds 315/345/15/45，其余每 60° 一组）和 Colorize。逐像素（拟合 Photoshop 2026，见测试）：
    - 每个范围按像素原色相的权重计入：边界之间线性渐入渐出，比边界晚半度（`HueRange::weight`）。
    - 色相转动 Master 的加上各范围加权的，保持最大与最小通道不变；
    - Master 的亮度：正值向白混合 `v + (255 − v)·k`，负值向黑 `v·(1 + k)`；
    - 各范围加权的亮度另一种算法：正值把各通道向最大通道移动，负值向最小通道移动；
    - 然后先各范围加权的饱和度，再 Master 的饱和度：正值时各通道离开 HSL 亮度 `L = (max+min)/2`，倍数 `1/a − 1`，`a` 为 `1 − 量`，当量与该色的 HSL 饱和度之和达到 1 时 `a` 为该饱和度（即推到全饱和）；负值时向 L 收缩 `(1 + 量)` 倍。
    - Colorize：取像素的 HSL 亮度，按 Master 亮度调整，再用 Master 的色相（0–360）与饱和度（0–100）组成颜色。
    - 结果与 Photoshop 相差：Master 不超过 4 级，Colorize 3 级，颜色范围 4 级（渐变边上）。
    - `HueSaturation::master(h, s, l)` 构造只有 Master 的设置；`HueSaturation::apply(px)` 作用于一个像素（对话框的 Before - After 色带也用它）。
  - `Exposure {  - `Exposure { exposure, offset, gamma }`：在 gamma 2.2 的线性光下计算（不是 sRGB 曲线，与 Photoshop 一致）：`v^2.2` 乘以 `2^exposure`、加 `offset`，负值截为 0，再取 `^(1/gamma)`，最后 `^(1/2.2)`。与 Photoshop 相差不超过 2 级。
  - `BrightnessContrast { brightness, contrast, legacy }`（−150–150、−50–100）：默认算法直接用 Photoshop 2026 的曲线：`data/brightness_contrast.bin` 存有 Photoshop 对每个亮度值（301 条）和每个对比度值（151 条）各自输出的 256 级表，按「先亮度、后对比度」复合（与 Photoshop 组合结果相差不超过 1 级）。这些曲线没有找到闭式（亮度的起始斜率为 `2^(b/110)`，正负亮度互为反函数）。`legacy`（Use Legacy）：亮度平移 `v + b`；对比度为正时以 128 为中心拉伸 `100/(100 − c)` 倍（100 时为阈值 128），为负时压缩 `(100 + c)/100` 倍，偏移量四舍五入（半数远离中心）；对比度为负时先对比度后亮度，为正时先亮度后对比度（与 Photoshop 的结果吻合）。
  - `ColorBalance { shadows, midtones, highlights, preserve_luminosity }`：每个通道一条曲线（Photoshop 的 Color Balance 是逐通道的查找表，包括 Preserve Luminosity），形如 Levels：阴影为负时输入黑场 = −值；高光为正时输入白场 = 255 − 值；gamma 为 `2^(G/100)`（经 `levels_gamma`）。不保持亮度时 `G = 中间调 + (阴影 + 高光)/2`；保持亮度时，三个轴的阴影先减去其最大值、高光减去其最小值、中间调减去 (最大 + 最小)/2，且只有中间调影响 gamma。与 Photoshop 2026 的 80 组随机设置相差不超过 3 级。
  - `BlackWhite { weights, tint }`：红、黄、绿、青、蓝、洋红六个权重（百分比）。灰度 = 最小通道 + (中间通道 − 最小) × 次色权重 + (最大 − 中间) × 主色权重；主色是最大的通道（红/绿/蓝），次色是最大两个通道合成的颜色（黄/青/洋红）。与 Photoshop 逐级相同（默认预设 40、60、40、60、20、80）。`tint` 为 Tint 颜色时，以「颜色」混合把它设到这个灰度上（`set_lum`：按 0.3 R + 0.59 G + 0.11 B 的亮度平移，再用 ClipColor 收回 0–1），与 Photoshop 相差不超过 2 级。
  - `Vibrance { vibrance, saturation }`：Vibrance 为近似：在 HSL 中饱和度乘 `1 + vibrance × (1 − s)`（对低饱和颜色作用更大；Photoshop 还会保护肤色，未还原）。Saturation 与 Photoshop 一致（相差不超过 2 级）：在 sRGB 线性光中各通道以灰 `0.2878 R + 0.7122 G`（蓝的权重为 0，与 Photoshop 的实测一致）为中心缩放 `1 + saturation/100` 倍。
  - `PhotoFilter { color, density, preserve_luminosity }`：在 D50 的 XYZ 中（线性 sRGB 经 `SRGB_TO_XYZ_D50`）把 X、Y、Z 分别乘以 `1 − 密度 + 密度 × 滤镜色的对应分量 / 白的分量`，再转回 sRGB——Photoshop 正是这样（拟合出的变换矩阵的本征向量即这组原色），与 Photoshop 相差不超过 1 级。Preserve Luminosity 时再用 `set_lum` 把结果的亮度设回原像素的亮度（相差不超过 6 级）。
  - `GradientMap { from, to, method }`：像素的亮度（0.299 R + 0.587 G + 0.114 B，与 Photoshop 一致）决定在渐变上的位置，颜色由 `gradient::blend_colors` 按方法插值（见 `gradient.md`）。
  - `AutoTone`、`AutoColor`：每个通道各自去掉最暗、最亮 0.1% 后拉伸到 0–255（Photoshop 的 Auto Color 还会中和中间调，这里没有）；`AutoContrast`：三个通道合并统计、按同一范围拉伸，颜色关系不变。直方图取当前图层选区内、alpha 不为 0 的像素。
  - `Curves { points, counts }`：RGB 复合、红、绿、蓝通道各最多 16 个（输入, 输出）点（`Adjustment::curves(points)` 只设复合通道，`curves_per_channel([..; 4])` 四个通道，空列表表示该通道不变）；先各通道自己的曲线，再复合曲线。`curve_table(points)` 生成查找表：按输入排序、去掉重复输入后做自然三次样条（两端二阶导为 0），第一个点之前、最后一个点之后保持平直，结果限制在 0–255、四舍五入；与 Photoshop 2026 的 12 组曲线逐级相同。没有点时为恒等，只有一个点时为常数。
  - `ChannelMixer { rows, monochrome }`：每个输出通道（Monochrome 时为三个通道相同的灰）= (R × 红% + G × 绿% + B × 蓝%) / 100 + 常数% × 2.55，半数进位并限制在 0–255。与 Photoshop 2026 相差不超过 1 级。
  - `SelectiveColor { colors, absolute }`：九个范围（Reds、Yellows、Greens、Cyans、Blues、Magentas、Whites、Neutrals、Blacks）各有 C、M、Y、K（−100–100%）。每个范围对像素有一个权重：Reds/Greens/Blues 为该通道单独最大时最大值与中间值之差；Cyans/Magentas/Yellows 为红/绿/蓝单独最小时中间值与最小值之差；Whites 为 `2·(min − 50%)`，Blacks 为 `2·(50% − max)`（不小于 0）；Neutrals 为 `1 − (|max − 50%| + |min − 50%|)`。每个通道的墨量 `ink = 1 − v`（C 对红、M 对绿、Y 对蓝）按每个范围变化 `权重 × clamp(d, −ink, 1 − ink)`，其中 `d = 量 + K × (1 + 量)`，Relative 时 d 再乘以 ink；各范围的变化相加（不是依次作用）。与 Photoshop 2026 的十余组设置（含多范围、Absolute）相差不超过 1 级。
  - Levels、Exposure、Brightness/Contrast、Color Balance、Curves、Equalize 与 Auto 系列先算出三个通道的 256 项查找表（`Adjustment::tables()`）再逐像素查表。
- `Adjustment::name()`：菜单与历史名称（「Invert」「Desaturate」「Threshold」「Posterize」「Equalize」「Levels」「Hue/Saturation」「Exposure」「Brightness/Contrast」「Color Balance」「Black & White」「Vibrance」「Photo Filter」「Gradient Map」「Auto Tone」「Auto Contrast」「Auto Color」「Curves」「Channel Mixer」「Selective Color」）。
- `rgb_histograms(doc)`：活动图层选区内、alpha 不为 0 的像素的红、绿、蓝三个直方图（Levels、Curves 对话框）。
- `channel_histogram(doc)`：活动图层选区内、alpha 不为 0 的像素的 R、G、B 值合并统计的直方图（Equalize 与 Levels 对话框使用）。
- `hsl_color(h, s, l)`、`hue_of(rgb)`：HSL 与 RGB 之间的换算（Colorize 的颜色、前景色的色相）。
- `mask_gray(rgb)`：颜色画在图层蒙版上的灰度（亮度，三个通道相同）。
- `luminosity(px)`：亮度 `(299 R + 587 G + 114 B) / 1000`，四舍五入到 0–255（Rec. 601 权重）。
- `luminosity_histogram(doc)`：活动图层选区内、alpha 不为 0 的像素的亮度直方图（Threshold 对话框显示它）。
- `check(doc)`：调整前的检查，失败时返回 `FillError`（没有图层、图层隐藏、像素锁定），提示文字与 Fill 相同格式，例如「Could not complete the Invert command because the target layer is hidden.」。
- `apply(doc, adjustment)`：先 `check`，再逐像素应用。

## 行为规则

- 只改颜色通道，alpha 不变；完全透明的像素跳过。
- 有选区时只处理选中的像素；部分选中（羽化、消除锯齿边缘）的像素按选择程度在原色与新颜色之间线性混合。
- 背景图层同样可以调整（背景图层只是不能移动、不能有透明，像素可以修改）。

## 已知限制

- Vibrance 滑块本身是近似（Photoshop 的 Vibrance 带有肤色保护，未还原）。
- Equalize 不提供 Photoshop 在有选区时弹出的选项，总是按选区内的直方图只处理选区。

## 数据来源

Brightness/Contrast 的表与 `fixtures/adjust/` 下的对照数据都由脚本在 Photoshop 2026 中生成：一张图每行一条灰阶，每行用选区套用不同的参数，存为 PNG 后读出。`probe.rgb` 为 64 × 72 的探测图（16 级 RGB 立方体、灰阶、随机颜色），其余 `.rgb` 为 Photoshop 对它的输出。

## 测试覆盖

- `invert_desaturate_threshold_posterize`：各调整对 (200, 100, 0) 等像素的结果，包括 Threshold 恰好在亮度 119 两侧的边界。
- `equalize_stretches_the_range`：只有 100 和 200 两个值时分别映射到 128 和 255。
- `levels_hue_saturation_and_exposure`：Levels 的黑白场拉伸与 gamma 2（128 → 181）；红色色相 +120° 变绿、饱和度 −100 加亮度 +50 得到 191 灰；曝光 +1 档把 128 变为 176。
- `color_adjustments`：Black & White 默认预设下纯红为 102、纯黄为 153；黑到红的渐变映射；Vibrance 提高低饱和颜色；蓝色滤镜 50% 把 200 灰变为 (100, 100, 200)。
- `curves_pass_through_their_points`：两点曲线为恒等；S 曲线经过各点且单调；端点外平直；(64→128) 的曲线把 64 灰变为 128。
- `auto_tone_stretches_each_channel`：两个像素时各通道拉伸到 0 与 255。
- `only_the_selection_changes`：选区外的像素不变。
- `hidden_layers_are_refused`：隐藏图层返回错误及 Photoshop 的提示文字。
- `photoshop::*`：与 Photoshop 2026 的输出对照（`fixtures/adjust/`）：
  - `hue_saturation_master_matches_photoshop`、`hue_saturation_colorize_and_ranges_match_photoshop`：七组 Master 设置、Colorize、五组颜色范围设置，限定最大误差与超过 1 级的通道数。
  - `levels_match_photoshop`：28 组 Levels（各种 gamma、黑白场、输出范围）。
  - `channel_levels_apply_before_the_composite`、`curves_per_channel`：单通道先于复合通道。
  - `brightness_contrast_matches_photoshop`：七组组合与十一组 Use Legacy。
  - `color_balance_matches_photoshop`：80 组随机三色调设置（含 Preserve Luminosity），每级误差不超过 3。
- `channel_histograms`：三个通道的直方图与合并直方图。
- `photoshop::channel_mixer_matches_photoshop`、`photoshop::selective_color_matches_photoshop`：三组通道混合（含单色）与七组可选颜色设置对照 Photoshop。
- `photoshop::black_white_photo_filter_and_exposure_match_photoshop`：Black & White（含两组 Tint）、四组 Photo Filter、四组 Exposure、三组 Vibrance 的 Saturation 对照 Photoshop。
- `photoshop::gradient_map_matches_photoshop`：红→蓝渐变在四种方法下对照 Photoshop。
