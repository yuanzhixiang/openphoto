# options_tools.rs：按控件表布局的工具选项栏

## 组件职责

把工具的选项栏写成控件表（`Item`）：分隔线、标签、笔刷选择器、图标按钮、切换图标、下拉框、输入框、百分比（输入框加滑块箭头框）、复选框、不可用的复选框、色块，坐标都是在 Photoshop 2026 选项栏截图上量出的「栏坐标」（见 `options_kit.md`）。`layout(tool)` 返回工具的控件表，`show` 依次画出并处理交互。目前包括画笔、铅笔、颜色替换、混合器画笔、橡皮擦、背景橡皮擦、魔术橡皮擦、仿制图章、图案图章、历史记录画笔、历史记录艺术画笔、模糊、锐化、涂抹、减淡、加深、海绵。

## 设置的读写

每个控件带一个键：

- 绑定到真实状态的键：`paint.opacity`、`paint.flow`（当前工具的 `PaintOptions` 的不透明度、流量；模糊、锐化的 Strength 与减淡、加深的 Exposure 也是不透明度），笔刷选择器（`PaintOptions` 的大小与硬度），`dodge.range` / `burn.range`（`RetouchOptions` 的色调范围），`sponge.mode`（Desaturate / Saturate），`clone.aligned`。这些都按原来的方式生效。
- 其余键保存在 `AppState::tool_settings`（文字；复选与切换为 `1` / `0`；下拉为序号）：可以修改并在本次运行中保留，但还没有效果。没有 `PaintOptions` 的工具（图案图章、历史记录艺术画笔、颜色替换、混合器、背景橡皮擦、涂抹）的笔刷大小也存在这里。
- 默认值按 Photoshop：混合器画笔的「每次描边后载入 / 清理」与背景橡皮擦、颜色替换的「连续取样」默认按下；颜色替换的 Mode 默认 Color，Limits 默认 Contiguous；Protect Tones、Protect Detail、Vibrance、Aligned、Anti-alias 默认勾选。
- 百分比输入时限制在 0–100%（画笔的不透明度、流量为 1–100%）。输入框在输入结束时才写回；滑块框拖动时立即写回。

## 笔刷选择器

点击弹出 Size（1–5000 px，对数刻度）与 Hardness（铅笔没有）滑块，改的是当前工具的笔刷。

## 已知差异

- 笔刷大小、不透明度等的初始值是本程序的默认值，Photoshop 截图里是当时的状态（例如 13 px 的画笔）。
- 混合器的平滑图标、对称（蝴蝶）、喷枪等图标是按截图近似重画的。
- 各种面板开关（画笔设置、仿制源）与对称、平滑选项、忽略调整图层等按钮还没有效果。

## 测试覆盖

- `ui_tests.rs` 的 `options_bars_match_photoshops_layout`（这些工具的分隔线与框的边缘与 Photoshop 一致，误差 1 pt）。
- `brush_options_bars_edit_their_settings`：画笔 Opacity 输入 50 后不透明度为 0.5；喷枪切换写入设置；仿制图章的 Aligned 取消勾选后 `RetouchOptions::clone_aligned` 为假。
