# adjust.rs：图像调整

## 职责

实现 Image › Adjustments 中逐像素的调整，作用于活动图层、限于选区。不记录历史（由 `op-ui` 按调整名称记录）。

## 对外接口

- `Adjustment`：
  - `Invert`：每个颜色通道取 `255 - v`。
  - `Desaturate`：三个通道都设为该像素的 HSL 亮度 `(max + min) / 2`（向上取整），与 Photoshop 的 Desaturate 一致（不是加权亮度）。
  - `Threshold(level)`：亮度（`luminosity`）大于等于 `level` 的像素变白，其余变黑。Photoshop 的取值范围 1–255，默认 128。
  - `Posterize(levels)`：每个通道量化为 `levels` 个色阶：`round(round(v × (L−1) / 255) × 255 / (L−1))`。范围 2–255，默认 4。
  - `Equalize`：直方图均衡。用活动图层选区内、alpha 不为 0 的像素的 R、G、B 三个通道值合在一起统计直方图，每个值映射为它在累计直方图中的位置 × 255（四舍五入），同一张表作用于三个通道。
- `Adjustment::name()`：菜单与历史名称（「Invert」「Desaturate」「Threshold」「Posterize」「Equalize」）。
- `luminosity(px)`：亮度 `(299 R + 587 G + 114 B) / 1000`，四舍五入到 0–255（Rec. 601 权重）。
- `luminosity_histogram(doc)`：活动图层选区内、alpha 不为 0 的像素的亮度直方图（Threshold 对话框显示它）。
- `check(doc)`：调整前的检查，失败时返回 `FillError`（没有图层、图层隐藏、像素锁定），提示文字与 Fill 相同格式，例如「Could not complete the Invert command because the target layer is hidden.」。
- `apply(doc, adjustment)`：先 `check`，再逐像素应用。

## 行为规则

- 只改颜色通道，alpha 不变；完全透明的像素跳过。
- 有选区时只处理选中的像素；部分选中（羽化、消除锯齿边缘）的像素按选择程度在原色与新颜色之间线性混合。
- 背景图层同样可以调整（背景图层只是不能移动、不能有透明，像素可以修改）。

## 已知限制

- 只实现了上面五种调整；其余调整见 `README.md` 的差距列表。
- Equalize 不提供 Photoshop 在有选区时弹出的选项，总是按选区内的直方图只处理选区。

## 测试覆盖

- `invert_desaturate_threshold_posterize`：各调整对 (200, 100, 0) 等像素的结果，包括 Threshold 恰好在亮度 119 两侧的边界。
- `equalize_stretches_the_range`：只有 100 和 200 两个值时分别映射到 128 和 255。
- `only_the_selection_changes`：选区外的像素不变。
- `hidden_layers_are_refused`：隐藏图层返回错误及 Photoshop 的提示文字。
