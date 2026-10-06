# options_bar.rs：顶部选项栏

## 组件职责

窗口顶部的工具选项栏，内容随当前工具变化，对应 Photoshop 的 Options Bar。

## 布局

高 52 参考像素，底色 `#535353`，内容垂直居中。从左到右：

1. Home 按钮。
2. 分隔线。
3. 工具预设：当前工具的图标加一个下拉箭头。
4. 分隔线。
5. 当前工具的选项（见下文）。
6. 靠右：Share、Notifications、Search、Discover、Workspace（带下拉箭头）、头像占位圆。

Home、工具预设、右侧图标都只有外观和悬停提示，点击没有功能。

## 各工具的选项

- **矩形选框、套索**：
  - 四个选区运算按钮（New / Add to / Subtract from / Intersect with selection），可切换，选中的有深色底。
  - Feather：数值框，0–1000 px，最多 1 位小数，显示为 `0 px`。
  - Anti-alias：复选框，矩形选框时置灰，与 Photoshop 一致。
  - Style：Normal / Fixed Ratio / Fixed Size。
  - Width / Height：只有 Style 不是 Normal 时才可用，中间是交换宽高的图标。
  - 「Select and Mask...」按钮。
  这些值保存在 `AppState::marquee`，目前还没有接到选区功能上；「Select and Mask...」点击没有功能。
- **移动工具**：Auto-Select 复选框加「Layer」字段、Show Transform Controls 复选框。只有外观，状态不保存。
- **抓手、缩放工具**：「100%」和「Fit Screen」两个按钮，作用于当前文档。
- **其它工具**：不显示工具选项。

## 已知限制

除抓手和缩放工具的两个按钮外，选项栏的设置目前都没有实际效果。其余工具的选项栏内容尚未与 Photoshop 对齐。
