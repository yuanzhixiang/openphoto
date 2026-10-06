# options_bar.rs：顶部选项栏

## 组件职责

窗口顶部的工具选项栏，内容随当前工具变化，对应 Photoshop 的 Options Bar。

## 布局

高 33 pt，底色 `#535353`，垂直中心在选项栏顶部下方 17 pt。外框部分按 Photoshop 2026 实测（2x 截图逐像素量取）用绝对位置绘制，下列 x 坐标都以选项栏左端为原点，单位 pt：

1. 抓手：x 5.5 处一列 10 个 2 × 1 pt 的 `#454545` 小点，从顶部下方 8 pt 开始，间距 2 pt。
2. Home 按钮：图标中心 x 28，实心房屋图标。
3. 分隔线：x 53。
4. 工具预设：图标中心 x 68.5；移动工具画 Photoshop 的四向箭头，其它工具暂用 Phosphor Bold 图标。下拉箭头中心 x 90.25。
5. 分隔线：x 102。
6. 当前工具的选项，从 x 110 开始（见下文）。
7. 靠右，中心到窗口右边的距离：Share 200、Notifications 163.5（铃铛为较暗的 `#b9b9b9`）、Search 131、Discover 98.5、Workspace 69.5、下拉箭头 48、头像占位圆 21.5（直径 24）。

分隔线都是 1 pt 宽、22.5 pt 高的 `#3e3e3e` 竖线（顶部下方 6 pt 起）。图标颜色 `#dddddd`，按钮热区 24 pt 见方，悬停时有圆角底色。图标都是照 Photoshop 描出的矢量图形（见 `ps_icons.md`），不是图标字体。

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
- **横排文字工具**：字体下拉（只有 Source Sans 3）、样式下拉（Regular / Semibold）、字号图标与字号（0.5–1296 pt，默认 12 pt）、置灰的消除锯齿下拉（Sharp）、三个对齐按钮（只有左对齐可用且为按下状态）、颜色色块（当前前景色）。输入文字期间右侧显示取消（⦸）与确认（✓）按钮。值保存在 `AppState::type_options`。
- **形状工具**：置灰的模式下拉（Shape）、「Fill:」色块（当前前景色）、「Stroke:」色块（白底红斜线，表示无描边）；多边形还有多边形图标与边数（3–100，默认 5），直线还有「Weight:」粗细（1–1000 px，默认 1 px）。边数与粗细保存在 `AppState::shape`。
- **魔棒**：四个选区运算按钮、Tolerance（0–255，默认 32）、Anti-alias（默认开）、Contiguous（默认开）、Sample All Layers（默认关）、「Select and Mask...」按钮（没有效果）。值保存在 `AppState::wand`。
- **画笔、铅笔、橡皮擦**：
  - 笔刷预设按钮：白色圆点，下方是当前大小数字，旁边一个下拉箭头。点击弹出 Size 滑块（1–5000 px，对数刻度）和 Hardness 滑块（铅笔没有）。
  - 分隔线后是「Mode:」下拉（画笔、铅笔显示 Normal，橡皮擦显示 Brush），目前置灰。
  - 「Opacity:」百分比；画笔和橡皮擦还有「Flow:」百分比。
- **修饰工具**（笔刷选择器同画笔）：
  - 减淡、加深：「Range:」下拉（Shadows / Midtones / Highlights，默认 Midtones，两个工具各自保存）、「Exposure:」百分比（默认 50%）、勾选且置灰的「Protect Tones」。
  - 海绵：「Mode:」下拉（Desaturate / Saturate，默认 Desaturate）、「Flow:」百分比（默认 50%）、勾选且置灰的「Vibrance」。
  - 模糊、锐化：置灰的「Mode: Normal」、「Strength:」百分比（默认 50%）、置灰的「Sample All Layers」；锐化还有勾选且置灰的「Protect Detail」。
  - 仿制图章：置灰的「Mode: Normal」、Opacity、Flow、「Aligned」复选框（默认勾选）、置灰的「Sample: Current Layer」。
  - 历史记录画笔：置灰的「Mode: Normal」、Opacity、Flow。
  每个工具的笔刷（大小、硬度、不透明度/强度、流量）分开保存在 `AppState` 中；Range、海绵模式、Aligned 保存在 `AppState::retouch`。
- **油漆桶**：Fill（Foreground，置灰）、Mode（全部混合模式）、Opacity、Tolerance（0–255）、Anti-alias、Contiguous、All Layers，都会生效。
- **移动工具**（按 Photoshop 2026 逐像素对齐）：
  - 「Auto-Select:」复选框，后接「Layer」下拉（55 pt 宽，目前只有 Layer 一项，没有 Group）。
  - 分隔线后是「Show Transform Controls」复选框。
  - 分隔线后两组对齐与分布按钮：Align left edges、Align horizontal centers、Align right edges、Distribute vertically，分隔线，Align top edges、Align vertical centers、Align bottom edges、Distribute horizontally。它们画成 Photoshop 禁用时的 `#989898`，没有功能（Photoshop 里要选中两个以上图层才可用，本应用还不支持多选图层）。
  - 分隔线后是「•••」（More options，只有外观）、分隔线、齿轮（Set additional options，右下带白色小三角，只有外观）。
  两个复选框都默认关闭，值保存在 `AppState::move_options` 并生效，行为见 `document_view.md`「移动工具」。
- **抓手、缩放工具**：「100%」和「Fit Screen」两个按钮，作用于当前文档。
- **裁剪工具**：「W:」「H:」两个只读字段（裁剪框的像素尺寸，中间是交换图标）、Clear 按钮（框恢复为整个画布）、分隔线后勾选且置灰的「Delete Cropped Pixels」；右侧原本的 Share、Search 等按钮换成取消（⦸，「Cancel current crop operation (Esc)」）和确认（✓，「Commit current crop operation (Return)」）。
- **自由变换进行中**（不论当前工具）：「W:」「H:」两个只读百分比字段、分隔线、角度图标与只读角度字段；右侧原本的 Share、Search 等按钮换成取消（⦸，「Cancel transform (Esc)」）和确认（✓，「Commit transform (Return)」）两个按钮。
- **其它工具**：不显示工具选项。

## 已知限制

除选框工具的组合方式、羽化、消除锯齿，绘画工具的大小、硬度、不透明度、流量，移动工具的两个复选框，以及抓手和缩放工具的两个按钮外，选项栏的设置目前都没有实际效果。外框部分和移动工具的选项已与 Photoshop 逐像素对齐；其余工具的选项内容（控件尺寸、间距、下拉框样式）尚未对齐，只是复选框已统一为 Photoshop 样式（`widgets::checkbox`）。
