# dialogs/appkit.rs：经典调整对话框的控件

## 组件职责

Photoshop 2026 用 AppKit 绘制的经典调整对话框（Levels、Curves）共用的控件，尺寸量自 Photoshop 的 2x 截图（Photoshop 点）。

## 控件

- `font()` / `text` / `label`：12 pt 系统字体（`theme::dialog`，按 macOS 的字距排版），`#f0f0f0`；不可用文字 `#8e8e8e`。`label` 的位置为大写字母的左中点。
- `field(ui, rect, value, id, range, step, decimals, focus)`：数值框，`#454545` 底、1 pt `#666666` 边，数字左边距 4 pt；有焦点时 ↑/↓ 按 `step` 步进（Shift 乘 10），限制在 `range` 内并按 `decimals` 位小数格式化。`focus` 时获得焦点并全选。
- `popup(ui, rect, id, value, menu)`：弹出菜单按钮，`#454545` 底、`#666666` 边、2 pt 圆角，文字左边距 8.5 pt，右侧 7.75 pt 处为 `Caret`；悬停时略亮。点击打开 `menu`。
- `button(ui, rect, label, default, enabled)`：26 pt 高的药丸按钮，默认按钮边框 `#e1e1e1`，其余 `#7d7d7d`，不可用 `#606060` 且文字变暗；文字比几何中心高 1 pt（与 Photoshop 吻合）。`button_with(ui, rect, label, (default, enabled), size, lift)` 可指定文字字号与上移量：插件式扭曲对话框的按钮用 13 pt、居中（`distort.md`）。
- `checkbox(ui, min, label, checked)`：13 pt 复选框；勾选时 `#d4d4d4` 底、深灰（`#323232`）勾，未勾选时 `#454545` 底加 `#a0a0a0` 边；标签在右侧 10.5 pt。
- `radio(ui, center, label, chosen)`：单选按钮，选中为 13 pt 浅灰圆盘加 5 pt 深色圆点，未选中为 `#474747` 圆盘加 `#848484` 圆环；标签在圆心右侧 16.5 pt。返回是否被点击。
- `group(painter, rect, gap)`：分组框，1 pt `#424242` 线框，顶边在 `gap` 两个 x 之间断开放标题。
- `pin(painter, tip, kind)`：直方图或渐变条下方的滑块「针」：12 × 10.5 pt 的房形（尖顶、下方圆角），1 pt 近黑轮廓；`Black` 为空心，`Gray` 填 `#a0a0a0`，`White` 填 `#e6e6e6`。
