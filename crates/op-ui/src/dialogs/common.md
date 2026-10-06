# dialogs/common.rs：对话框公共部件

## 职责

各模态对话框共用的外观部件，保证所有对话框看起来一致。

## 部件

- `frame(ui, rect, title, font)`：画对话框外框：阴影、`#535353` 底色、圆角 10、深色外边框，以及顶部高 28 pt 的浅灰标题栏（`#d0d2d4`，深色居中标题，下方有分隔线）。对应 Photoshop 对话框的 macOS 窗口标题栏。
- `pill_button(ui, rect, label, font, enabled)`：胶囊形按钮。浅色边框（`#d0d0d0`，1.5），底色与对话框相同，悬停变浅，按下变深；禁用时边框和文字置灰且不响应点击。同一对话框里按钮文字必须唯一（用作交互 id）。

## 使用方

Canvas Size（`canvas_size.md`）和 Color Picker（`color_picker.md`）。
