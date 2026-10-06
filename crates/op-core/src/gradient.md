# gradient.rs：渐变工具（经典渐变）

## 职责

在活动图层上画两色渐变，限于选区。对应 Photoshop 渐变工具的「经典渐变」行为（直接改像素，而不是创建渐变填充图层）。不记录历史。

## 对外接口

- `GradientKind`：`Linear`、`Radial`、`Angle`、`Reflected`、`Diamond`（`ALL` 为选项栏的顺序，`label()` 为「Linear Gradient」等）。
- `GradientKind::position(a, b, x, y)`：点 (x, y) 在从 `a` 拖到 `b` 的渐变中的位置 t（0 为起点颜色，1 为终点颜色）。设拖动向量为 d、长度为 L：
  - Linear：点在 d 方向上的投影比例，限制在 0–1。
  - Radial：到起点的距离 / L，最大 1。
  - Reflected：投影比例的绝对值，最大 1（起点两侧对称）。
  - Diamond：在以 d 为轴的坐标系中 `(|u| + |v|) / L`，最大 1。
  - Angle：绕起点从 d 方向开始转一圈，t 为转过的角度 / 360°（屏幕上逆时针方向递增）。
  - 起止点重合时为 0。
- `GradientOptions`：`kind`、`mode`（混合模式）、`opacity`（0–1）、`reverse`（交换起止颜色）。默认 Linear、Normal、100%、不反向。
- `gradient(doc, a, b, colors, options)`：先做与调整相同的检查（`adjust::check`），然后对每个像素取像素中心的 t，在 RGB 中线性插值两种颜色，按混合模式、不透明度 × 选择程度合成到图层上。背景图层和锁定透明像素的图层保持原 alpha（完全透明的像素不画）。

## 已知限制

- 只有两色（前景到背景）渐变，没有渐变编辑器、预设、透明度色标、Dither 和插值方式选项。
- 不创建渐变填充图层（Photoshop 2024 起渐变工具默认创建可编辑的渐变图层）。

## 测试覆盖

- `positions_of_each_kind`：各类型在典型点的 t 值。
- `paints_black_to_white`：5 像素黑到白得到 `[0, 64, 128, 191, 255]`，反向时颠倒。
- `half_opacity_and_selection`：50% 不透明度只作用于选区内的像素。
