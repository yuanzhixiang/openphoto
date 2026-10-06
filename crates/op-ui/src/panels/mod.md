# panels/mod.rs：右侧面板列与图标列

## 组件职责

- 右侧面板列：三组带标签的面板上下排列，对应 Photoshop 默认工作区右侧的面板。
- 图标列：面板列左边一列竖向图标，对应 Photoshop 收起成图标的面板（History、Comments）。

## 面板列

### 布局

- 顶部一行只有一个「>>」折叠箭头（只有外观）。
- 三组面板，组之间是 3 参考像素宽的深色分隔条 `#393939`：
  1. Color、Swatches、Gradients、Patterns，默认高 228。
  2. Properties、Adjustments、Libraries，占用剩余空间（伸缩组）。
  3. Layers、Channels、Paths，默认高 430。
- 每组顶部是标签栏（高 42，底色 `#424242`），右侧有面板菜单图标（只有外观）；下面是面板内容，底色 `#535353`。

### 标签

- 标签宽度为文字宽度加 30，文字半粗体。
- 激活标签底色 `#535353`、文字 `#eaeaea`；未激活标签与标签栏同色，右侧有分隔线，文字 `#bcbcbc`，悬停时变亮。
- 点击切换该组显示的面板。

### 调整高度

拖动组间分隔条可以改变高度（光标变为上下箭头）。拖动伸缩组旁边的分隔条时，改变的是另一侧的固定高度组，伸缩组自动吸收差值。每组最小高度为标签栏加 40。

### 各面板

- Color、Swatches：见 `color_panel.md`。
- Properties：见 `properties.md`。
- Layers：见 `layers.md`。
- Gradients、Patterns、Adjustments、Libraries、Channels、Paths：显示「{名称} — not implemented yet」占位文字。

## 图标列

- 宽 44 pt：左侧 3 pt 的深色边 `#3f3f3f`（紧挨文档的垂直滚动条），右侧 5 pt 的分隔带（外沿 `#4d4d4d`、中间 `#414141`），中间是图标区域。
- 顶部一行只有一个「<<」箭头（只有外观）。
- History 按钮：点击打开或收起 History 弹出面板（见 `history.md`），打开时按钮显示为选中状态。
- Comments 按钮：只有外观。
- 按钮下方有一条分隔线。
- 返回 History 按钮的位置，供弹出面板定位。

## 已知限制

- 面板不能拖动重新停靠、不能拖出成浮动窗口、不能收起成图标。
- 面板菜单（右上角三横线）没有内容。
