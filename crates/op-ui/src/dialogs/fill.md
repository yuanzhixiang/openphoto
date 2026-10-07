# dialogs/fill.rs：Fill 对话框

## 组件职责

Edit › Fill...（⇧F5）。选择填充内容和混合方式，确定后由 `lib.rs` 调用 `op_core::fill::fill` 填充当前图层的选区（没有选区时整个图层），并记录一条「Fill」历史。

## 布局

按 Photoshop 2026 的 Fill 对话框（经典 AppKit 对话框，2x 截图）逐点量取，与 Levels、Curves 同一类，控件来自 `appkit.rs`。对话框 348 × 293 pt，坐标为距对话框左上角的 pt：

- 标题栏：`common::frame`，系统粗体 13 pt「Fill」。文字为 12 pt 系统字体（`theme::dialog`，带 macOS 字距），`#f0f0f0`。
- 「Contents:」右对齐于 112.5，中心 y 48.5；下拉 (118–263，y 38–59)。
- **Options** 分组框 (11.5–262.5，y 116–166)：1 pt `#424242` 线，标题「Options」从 30.5 起压在上边线上，线在标题两侧各留 5 pt。里面是不可用的「Color Adaptation」复选框 (20, 141.5)：只对 Content-Aware 有用，而 Content-Aware 还没有实现，所以总是置灰。
- **Blending** 分组框 (11.5–262.5，y 184–282)：「Mode:」(中心 y 208.5) 与下拉 (118–253，y 198–219，全部 27 种混合模式，按 Photoshop 菜单分组)；「Opacity:」(236.5) 与输入框 (117–169，y 227–246，打开时获得焦点并全选，↑↓ 步进 1、⇧ 步进 10) 加「%」(177.5)；「Preserve Transparency」复选框 (20, 257.5)。
- 复选框 12 pt，文字在方框右 11 pt；不可用时为 `#4d4d4d` 方框、`#5d5d5d` 边、`#8e8e8e` 文字。
- OK（默认按钮，亮边）(278.5–338.5，y 38.5–64.5)、Cancel (y 73.5–99.5)：26 pt 高的胶囊按钮。

与 Photoshop 截图相比各元素相差不超过 1 pt。

## Contents 选项

Foreground Color（默认）、Background Color、Color...；Content-Aware、Pattern、History（置灰）；Black、50% Gray（128）、White。

选择「Color...」打开标题为「Color Picker (Fill Color)」的 Color Picker；在那里确定后，Contents 变为 Color... 并使用所选颜色，取消则保持原来的选择。

## 键盘

- Enter：确定（不透明度有效时）。Esc：取消。
- Color Picker 打开期间，Enter/Esc 只作用于 Color Picker。

## 校验

不透明度必须是 0–100 的数字，否则 OK 置灰。

## 已知限制

- Content-Aware、Pattern、History 还没有实现，所以 Color Adaptation 总是置灰；没有 Pattern 的图案选择和脚本选项。
- 不记住上次的设置。
