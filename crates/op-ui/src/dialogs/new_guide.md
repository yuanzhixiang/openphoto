# dialogs/new_guide.rs：View › Guides › New Guide... 对话框

## 组件职责

按数值新建一条参考线。确定后由 `lib.rs` 加入当前文档并记录「New Guide」。

## 布局

按 Photoshop 2026 的「New guide」对话框（UXP 样式，2x 截图）逐点量取，对话框 390 × 188 pt，坐标为距对话框左上角的 pt：

- 标题栏：`common::frame`，系统粗体 13 pt「New guide」（g 小写，与 Photoshop 2026 一致）。文字为 12 pt 面板字体（`uxp::font`）。
- 「Orientation」粗体，左端 20，中心 y 57；两个单选项并排：Horizontal（圆心 26, 84）、Vertical（圆心 108, 84），`uxp::radio`。
- 「Position」右对齐于 59.5，中心 y 120.5；输入框 (68–161，y 108–132)，默认「0 px」，文字左缩进 11.5 pt。
- 「Color」右对齐于 59.5，中心 y 156.5；下拉 (68–228，y 144–168)：16 pt 的颜色方块（`#a9a9a9` 边）加颜色名，选项为 Photoshop 的参考线颜色 Cyan（默认）、Light Blue、Light Red、Green、Medium Blue、Yellow、Magenta、Gray、Black。右侧 (240–288) 是自定颜色的白色色块。
- OK (300–370，y 48–72) 是持有键盘焦点的默认按钮（`common::ps_focused_button`：`#737373` 底、白边、蓝色焦点环），Cancel (y 84–108)。打开时输入框没有焦点，与 Photoshop 一致。

与 Photoshop 截图相比各元素相差不超过 1.5 pt。

## 交互

- 单击单选项切换方向，默认 Horizontal。
- Position 可以带「px」后缀输入。范围 −30000–30000，超出或不是数字时按 OK / Enter 不做任何事。
- Enter 确定，Esc 取消。打开期间是模态的。

## 已知限制

- Color 的选择还没有效果：所有参考线用同一种颜色绘制；自定颜色色块不能点。
- 只能用像素单位。
- Photoshop 记住上次的方向（截图里是 Vertical）；这里每次恢复 Horizontal。

## 测试覆盖

- `position_accepts_a_px_suffix`：「120 px」解析为 120 的垂直参考线；非数字无效。
- `ui_tests.rs` 的 `new_guide_dialog_adds_a_guide`：选 Vertical、Position 输入「100 px」回车，文档多一条位置 100 的垂直参考线。`screenshot_new_guide_dialog`（`#[ignore]`）截出对话框用于与 Photoshop 比对。
