# dialogs/brightness_contrast.rs：Brightness/Contrast 对话框

## 组件职责

Image › Adjustments › Brightness/Contrast... 的对话框，按 Photoshop 2026 的 UXP 对话框逐像素重做（401 × 212 pt，与 Photoshop 截图的差别在 1 像素内）。只管理设置；预览与应用由 `adjust.rs` 的宿主和 `lib.rs` 完成。

## 数据与默认值

- `brightness`（−150–150）、`contrast`（−50–100）为输入框文字，默认 0；`legacy`（Use Legacy）默认不勾选。每次打开都恢复默认值。
- `adjustment()`：两个值都有效时返回 `Adjustment::BrightnessContrast`，否则 `None`（OK 置灰、不预览）。小数四舍五入。

## 布局（Photoshop 点，自对话框左上角）

- 「Brightness」标签 x 20，输入框 (203, 48)–(260, 72)，滑块轨道 y 84、x 20–260。
- 「Contrast」标签，输入框 (203, 103)–(260, 127)，滑块 y 139。滑块的 0 在轨道中点（Contrast 的负半段只有 −50）。轨道为 `#737373`。
- 「Use Legacy」复选框 (20, 164)；「Preview (Opt+P)」复选框 (272, 174)。
- 右侧按钮列：OK、Cancel、Auto（`uxp::buttons`）。
- 打开时 Brightness 输入框获得焦点并全选。

## 交互

- 输入框可输入，↑/↓ 步进 1（Shift 10）；拖动或点击滑块改变数值。
- Auto：按图层直方图选出亮度与对比度。Photoshop 的 Auto 规则未能还原（例如 0–255 的均匀灰阶它给出 14 / −19，128–255 给出 0 / 0），这里在 Photoshop 的亮度/对比度曲线中选出与 Auto Contrast（两端各裁去 0.1%）拉伸最接近的一组（按直方图加权的平方误差，步长 2）。
- Enter / OK 应用，记录历史「Brightness/Contrast」；Esc / Cancel 取消；⌥P 切换预览。

## 测试覆盖

- `ui_tests::brightness_contrast_dialog_types_and_applies`：输入 50、勾选 Use Legacy 后 100 灰变 150；不勾选时结果等于 Photoshop 的亮度曲线。
- 截图对比：`ui_tests::screenshot_adjustment_dialogs`（`brightness_contrast_dialog.png`）。
