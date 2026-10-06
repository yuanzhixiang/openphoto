# dialogs/common.rs：对话框公共部件

## 职责

各模态对话框共用的外观部件，保证所有对话框看起来一致。

## 部件

- `frame(ui, rect, title, font)`：画对话框外框：阴影、`#535353` 底色、圆角 10、深色外边框，以及顶部高 28 pt 的浅灰标题栏（`#d0d2d4`，深色居中标题，下方有分隔线）。对应 Photoshop 对话框的 macOS 窗口标题栏。
- `pill_button(ui, rect, label, font, enabled)`：胶囊形按钮。浅色边框（`#d0d0d0`，1.5），底色与对话框相同，悬停变浅，按下变深；禁用时边框和文字置灰且不响应点击。同一对话框里按钮文字必须唯一（用作交互 id）。

- `text_field(ui, rect, text, id, font, pad, select_all)`：同 `number_field`，但字体和左侧内边距由调用方给出，右侧内边距固定 2 pt（New Layer 对话框用它）。`number_field` 以比例字体、左侧内边距 = 框高 × 0.23 调用它。
- `number_field(ui, rect, text, id, font, select_all)`：对话框里的文字输入框：`#454545` 底、`#777777` 边框、圆角 3；获得焦点时外面一圈 2 像素的蓝色 `#1473e6`。`select_all` 时获得焦点并全选文字（对话框打开时的第一个输入框）。
- `dropdown(ui, rect, id, selected, font, enabled, menu)`：对话框里的下拉框，填满 `rect`：底色与对话框相同、`#6a6a6a` 边框、圆角 3，悬停变浅。

## 使用方

Canvas Size（`canvas_size.md`）、Fill（`fill.md`）和 Color Picker（`color_picker.md`）。
