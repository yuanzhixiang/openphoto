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

- **四种选框工具、三种套索工具**：
  - 四个选区运算按钮（New / Add to / Subtract from / Intersect with selection），可切换，选中的有深色底。
  - Feather：数值框，0–1000 px，最多 1 位小数，显示为 `0 px`。
  - Anti-alias：复选框，矩形、单行、单列选框时置灰（它们没有曲线边缘），与 Photoshop 一致；默认勾选。
  - 仅选框工具：Style（Normal / Fixed Ratio / Fixed Size）；Width / Height 只有 Style 不是 Normal 时才可用，中间是交换宽高的图标。套索工具没有这两项，与 Photoshop 一致。
  - 「Select and Mask...」按钮。
  这些值保存在 `AppState::marquee`，选框与套索共用。组合方式、Feather、Anti-alias 在创建选区时生效（见 `document_view.md`）；Style、Width/Height 和「Select and Mask...」目前没有效果。
- **吸管**：
  - 「Sample Size:」下拉：Point Sample、3 by 3 Average、5 by 5 Average、11 by 11 Average、31 by 31 Average、51 by 51 Average、101 by 101 Average，默认 Point Sample。
  - 「Sample:」下拉：Current Layer / All Layers，默认 All Layers。
  - 「Show Sampling Ring」复选框：勾选但置灰（取样环尚未实现）。
  值保存在 `AppState::eyedropper`。
- **渐变工具**（经典渐变）：渐变色样（110 × 26，当前前景色到背景色，勾选 Reverse 时反过来）与下拉箭头（没有效果）；五个类型按钮（Linear、Radial、Angle、Reflected、Diamond，悬停提示为「Linear Gradient」等）；「Mode:」全部混合模式；「Opacity:」百分比；Reverse 复选框。值保存在 `AppState::gradient`。
- **魔棒**：四个选区运算按钮、Tolerance（0–255，默认 32）、Anti-alias（默认开）、Contiguous（默认开）、Sample All Layers（默认关）、「Select and Mask...」按钮（没有效果）。值保存在 `AppState::wand`。
- **画笔、铅笔、橡皮擦**：
  - 笔刷预设按钮：白色圆点，下方是当前大小数字，旁边一个下拉箭头。点击弹出 Size 滑块（1–5000 px，对数刻度）和 Hardness 滑块（铅笔没有）。
  - 分隔线后是「Mode:」下拉（画笔、铅笔显示 Normal，橡皮擦显示 Brush），目前置灰。
  - 「Opacity:」百分比；画笔和橡皮擦还有「Flow:」百分比。
- **油漆桶**：Fill（Foreground，置灰）、Mode（全部混合模式）、Opacity、Tolerance（0–255）、Anti-alias、Contiguous、All Layers，都会生效。
- **移动工具**：Auto-Select 复选框加「Layer」字段、Show Transform Controls 复选框。只有外观，状态不保存。
- **抓手、缩放工具**：「100%」和「Fit Screen」两个按钮，作用于当前文档。
- **裁剪工具**：「W:」「H:」两个只读字段（裁剪框的像素尺寸，中间是交换图标）、Clear 按钮（框恢复为整个画布）、分隔线后勾选且置灰的「Delete Cropped Pixels」；右侧原本的 Share、Search 等按钮换成取消（⦸，「Cancel current crop operation (Esc)」）和确认（✓，「Commit current crop operation (Return)」）。
- **自由变换进行中**（不论当前工具）：「W:」「H:」两个只读百分比字段、分隔线、角度图标与只读角度字段；右侧原本的 Share、Search 等按钮换成取消（⦸，「Cancel transform (Esc)」）和确认（✓，「Commit transform (Return)」）两个按钮。
- **其它工具**：不显示工具选项。

## 已知限制

除选框工具的组合方式、羽化、消除锯齿，绘画工具的大小、硬度、不透明度、流量，以及抓手和缩放工具的两个按钮外，选项栏的设置目前都没有实际效果。其余工具的选项栏内容尚未与 Photoshop 对齐。
