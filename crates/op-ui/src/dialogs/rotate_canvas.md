# dialogs/rotate_canvas.rs：Rotate Canvas 对话框

## 职责

Image › Image Rotation › Arbitrary... 的对话框：输入角度并选择方向，确定后由 `lib.rs` 调用 `op_core::image_ops::rotate_arbitrary`（背景色为当前背景色），记录「Rotate Canvas」历史（Photoshop 2026 实测的名字）。

## 布局（Photoshop 2026 实测，350 × 128 pt，相对左上角）

这是 UXP 对话框：文字用 `theme::uxp` 12 pt（Adobe Clean 的替代 Source Sans 3），`#f0f0f0`；标题「Rotate Canvas」为窗口标题（13 pt 粗体系统字体）。

- 「Angle」在 x 20、中心 y 60；输入框 (55, 47)–(126, 73)，文字左缩进 12，数字后面紧跟「°」。打开时值为 0 并全选、获得焦点。
- 单选按钮（12 pt，圆心 x 142.75）：Clockwise 中心 y 59.75（默认选中），Counter Clockwise 中心 y 83.75；文字在 x 158.5。选中时为浅色 `#d4d4d4` 圆盘、中间 4 pt 的深色圆点，未选中为 1 pt `#a0a0a0` 圆环。点击圆或文字选择。
- 按钮：OK (260, 48)–(330, 72) 为默认按钮，Cancel (260, 84)–(330, 108)，都是 `common::ps_button` 的粗体样式。

## 交互

- 角度可带「°」，绝对值必须小于 360；无法解析或超出范围时 OK 置灰。`degrees()` 返回有符号角度：顺时针为正，逆时针为负。
- Enter 等于 OK，Esc 等于 Cancel；打开期间是模态的。

## 已知限制

- 输入框获得焦点时是框外 2 pt 的蓝色外圈（`common::text_field`），Photoshop 是把框线换成蓝色。

## 测试

- `angle_and_direction`：0、30°、逆时针为负、360 与非数字无效。
- `ui_tests::rotate_canvas_dialog`：输入 90、选 Counter Clockwise、Enter 后 734 × 811 的文档变为 811 × 734，记录「Rotate Canvas」。
- `ui_tests::screenshot_rotate_canvas_dialog`（忽略）：与 Photoshop 截图对比用。
