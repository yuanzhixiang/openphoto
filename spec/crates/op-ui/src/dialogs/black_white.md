# dialogs/black_white.rs：Black and White 对话框

## 组件职责

Image › Adjustments › Black & White...（⌥⇧⌘B）的对话框，按 Photoshop 2026 的 UXP 对话框重做（401 × 568 pt）。

## 数据与布局

- 六个权重 Reds、Yellows、Greens、Cyans、Blues、Magentas（−200–300%，Photoshop 默认 40 / 60 / 40 / 60 / 20 / 80）。标签 x 20，输入框 x 209–260、y 从 84 起每 55 pt，数字后画「%」；轨道在框下 36 pt，从 −200 到 300 线性（不以 0 居中），颜色为黑 → 该色（范围中点）→ 白。打开时 Reds 框获得焦点并全选。
- 「Preset:」(20, 61)，下拉 (59, 48.5)–(231, 73.5)，预设菜单图标 (248, 61)。
- Tint：复选框 (20, 420.5) 与色块 (72.5, 414)–(120, 437.5)；Hue（0–360°，默认 42）与 Saturation（0–100%，默认 20），输入框 y 450 / 505，轨道 y 486 / 541。未勾选 Tint 时这两行变暗且不可用。Tint 颜色为 HSB（色相、饱和度、亮度 88.2%），`tint_color(42, 20)` = (225, 211, 180)。
- 右侧 OK、Cancel、Auto（不可用：Photoshop 按图像自动设权重的算法未实现）；「Preview (Opt+P)」(272, 163)。

## 测试覆盖

- `weights_and_tint`；`ui_tests::small_uxp_adjustment_dialogs_apply`；截图 `black_white_dialog.png`。
