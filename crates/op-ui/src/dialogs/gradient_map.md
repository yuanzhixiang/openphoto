# dialogs/gradient_map.rs：Gradient Map 对话框

## 组件职责

Image › Adjustments › Gradient Map... 的对话框，按 Photoshop 2026 的经典对话框重做（416 × 230 pt）。算法见 `op-core` 的 `adjust.md` 与 `gradient.md`。

## 数据与布局

- 渐变为两种颜色，打开时为前景色到背景色（`AdjustDialog::set_gradient_colors`）；Dither、Reverse 默认不勾选；Method 默认 Smooth（Perceptual / Linear / Classic / Smooth）。
- 分组框「Gradient Used for Grayscale Mapping」(11, 46.5)–(309, 97)：渐变框 (20, 67.5)–(300, 87.5)，渐变条 (21, 68.5)–(286, 86.5) 按当前方法与 Reverse 画出；右侧的 ⌄ 打开几个两色渐变预设（Foreground to Background、Black, White、White, Black）。
- 分组框「Gradient Options」(11, 121)–(309, 219)：「Dither」(20, 139)、「Reverse」(20, 166) 复选框，「Method:」(20.5, 200) 与弹出菜单 (72, 189.5)–(157, 210.5)。
- 按钮 x 325.5–385.5：OK (45)、Cancel (80)；「Preview」(325, 124.5)。

## 已知限制

- 渐变只有两种颜色；点击渐变条不会打开渐变编辑器；Dither 没有效果。

## 测试覆盖

- `reverse_and_method`；`ui_tests::more_adjustments`（Smooth 映射与核心一致）；截图 `gradient_map_dialog.png`。
