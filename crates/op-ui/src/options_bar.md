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
7. 靠右，中心到窗口右边的距离：Share 200、Notifications 163.5（铃铛为较暗的 `#b9b9b9`）、Search 131、Discover 98.5、Workspace 69.5、下拉箭头 48、头像占位圆 21.5（直径 24）。输入文字期间这一组换成取消（⦸）与确认（✓）按钮。

分隔线都是 1 pt 宽、22.5 pt 高的 `#3e3e3e` 竖线（顶部下方 6 pt 起）。图标颜色 `#dddddd`，按钮热区 24 pt 见方，悬停时有圆角底色。图标都是照 Photoshop 描出的矢量图形（见 `ps_icons.md`），不是图标字体。

Home、工具预设、右侧图标都只有外观和悬停提示，点击没有功能。

## 各工具的选项

- **按实测布局的工具**（`measured_bar`，用 `options_kit` 按 Photoshop 2026 截图量出的「栏坐标」绝对定位，见 `options_kit.md`；坐标为 pt）：
  - **四种选框工具**：选区运算四按钮（110 起，每个 26）；分隔线 222；「Feather:」(233) 与输入框 (276–330，如 `0 px`，0–1000)；Anti-alias 复选框 (338，只在椭圆选框可用，不可用时显示为未勾选)；分隔线 412；「Style:」(421.5) 与下拉 (453.5–531，Normal / Fixed Ratio / Fixed Size)；「Width:」(541) 与输入框 (574–615.5)、交换图标 (中心 638.5)、「Height:」(663.5) 与输入框 (699.5–741)，Style 为 Normal 时都不可用；分隔线 749；「Select and Mask...」(760.5–870.5，有选区时可用)。单行、单列选框的 Style 不可用，Feather 输入框起的元素左移 2 pt（与 Photoshop 一致）。
  - **套索、多边形套索**：同上到分隔线 412，之后是「Select and Mask...」(423.5–533.5)。
  - **磁性套索**：分隔线 412 后是「Width:」(422.5) 与输入框 (457.5–500.5，10 px)、「Contrast:」(509) 与输入框 (557.5–600.5，10%)、「Frequency:」(609.5) 与输入框 (666–693，57)；分隔线 701；压感按钮 (712–738，可切换)；分隔线 748；「Select and Mask...」(759.5–869.5)。
  - **魔棒**：运算按钮 (110)；分隔线 222；「Sample Size:」(231.5) 与下拉 (298–413.5，Point Sample 到 101 by 101 Average)；「Tolerance:」(422.5) 与输入框 (476.5–524.5，0–255)；Anti-alias (532.5)、Contiguous (606.5)、Sample All Layers (690)；分隔线 803.5；Select Subject (815–906，不可用) 与其菜单按钮 (907–926)；「Select and Mask...」(945–1055)。
  - **对象选择**：运算按钮 (104 起)；分隔线 212；「Select people」菜单按钮 (217.5–305.5，右下角小三角)；刷新 (中心 323)、显示全部对象开关 (353)、齿轮 (383)；分隔线 400；模式下拉 (405–483，Rectangle / Lasso)；分隔线 487；Sample All Layers (492)、Hard Edge (602，默认勾选)；分隔线 677；反馈 (696)；分隔线 712；Select Subject (717.5–808.5) 与菜单按钮 (813.5–832.5)；「Select and Mask...」(837.5–947.5)。
  - **快速选择**：三个模式按钮（新选区 / 添加 / 减去，中心 129、157、185）；分隔线 202；笔刷选择器 (中心 221.5，大小 30)；分隔线 253；角度图标 (266) 与输入框 (277–318.5，0°)；分隔线 322.5；Sample All Layers (327.5)、Enhance Edge (437)；分隔线 530；Select Subject (543.5–634.5) 与菜单按钮；「Select and Mask...」(669.5–779.5)。
  - **其余工具**（`options_tools.rs` 里的控件表，见 `options_tools.md`）：绘画、修饰、修复、填充、取样与测量工具，钢笔与锚点工具，文字工具，路径选择，形状工具，抓手、旋转视图、缩放，画板、透视裁剪、切片、切片选择、画框；各控件的坐标与绑定见该文件。
  - 选区运算、Feather、Anti-alias（`AppState::marquee`，选框与套索共用；对象选择也用它的运算方式）与魔棒的 Tolerance、Anti-alias、Contiguous、Sample All Layers（`AppState::wand`）生效；其余设置（选框的固定宽高、磁性套索的参数与压感、魔棒的 Sample Size、对象选择与快速选择的各项）保存在 `AppState::tool_settings`，可以修改并保留，但还没有效果。Select Subject 不可用；Select and Mask 有选区时可点但还没有效果。
- **移动工具**（按 Photoshop 2026 逐像素对齐）：
  - 「Auto-Select:」复选框，后接「Layer」下拉（55 pt 宽，目前只有 Layer 一项，没有 Group）。
  - 分隔线后是「Show Transform Controls」复选框。
  - 分隔线后两组对齐与分布按钮：Align left edges、Align horizontal centers、Align right edges、Distribute vertically，分隔线，Align top edges、Align vertical centers、Align bottom edges、Distribute horizontally。它们与 Photoshop 一样在条件满足时可用：对齐需要两个以上可移动的选中图层（或有像素选区），分布需要三个以上；不可用时画成 Photoshop 禁用时的 `#989898`。点击执行对应的 `Command::Align` / `Command::Distribute`（第一组第四个是 Distribute Vertically 等间距，第二组第四个是 Distribute Horizontally）。
  - 分隔线后是「•••」（More options，只有外观）、分隔线、齿轮（Set additional options，右下带白色小三角，只有外观）。
  两个复选框都默认关闭，值保存在 `AppState::move_options` 并生效，行为见 `document_view.md`「移动工具」。
- **裁剪工具**（`crop_bar`，绝对定位，坐标为距选项栏左边的 pt，Photoshop 2026 实测，与截图相差 1–2 px）：比例菜单 (110–201，显示「W x H x Reso...」或预设名)；W 输入框 (205–272.5)；交换按钮（实心双箭头，中心 290.25）；H 输入框 (310–376)；W x H x Resolution 类时分隔线 380、分辨率框 (385–439.5)、单位下拉「px/in / px/cm」(442.5–496.5)、分隔线 500.5；Clear (506–552.5，`#454545` 底 `#666666` 边)；Straighten 图标（中心 574）与文字（x 592）；分隔线 649.5；叠加菜单（网格图标，中心 670.75：六种叠加、Auto/Always/Never Show Overlay、Cycle Overlay、置灰的 Cycle Orientation）；齿轮菜单（中心 705：Use Classic Mode、Show Cropped Area、Auto Center Preview、Enable Crop Shield、Opacity、Auto Adjust Opacity）；分隔线 726.5；「Delete Cropped Pixels」复选框 (735)；「Fill:」(870.5) 与下拉 (892–1023.5，Background (default)，Generative Expand 与 Content-Aware Fill 置灰)；ⓘ (1042)；复位 (1070.5，框没变时置灰)；框改变后出现取消 ⦸ (1103.75) 与确认 ✓ (1138.5)。Ratio 类没有分辨率框，Clear 及其后的元素左移 121 pt。这一栏比 1350 pt 的窗口宽，所以右侧的 Share 等应用按钮整体右移（Share 在 1166.5，头像被窗口裁掉一半），与 Photoshop 一致。
- **自由变换进行中**（不论当前工具，`transform_bar`，绝对定位，Photoshop 2026 实测，坐标为距选项栏左边的 pt）：
  - Home、工具预设（换成暗色的变换图标：带控制点的框与箭头）、右侧的 Share 和 Workspace 都变暗、不可点，Bell、Search、Discover 照常。
  - 参考点复选框（13.5 pt 圆角框，中心 120.25，默认不勾选）与 3 × 3 参考点格（中心 146，未勾选时暗色，勾选后可点选参考点，所选为实心）。
  - 「X:」(163) 输入框 (174.5–235.5)、相对定位 △（中心 250.75，打开时有底色，X/Y 显示相对开始位置的位移）、「Y:」(268.5) 输入框 (279.5–340)，显示参考点当前位置，如「60.00 px」。
  - 分隔线 343.5；「W:」(349) 输入框 (362.5–417)；链接按钮 (420–446，打开时 `#383838` 底 `#636363` 边，默认打开)；「H:」(450.5) 输入框 (463–517)，如「100.00%」。
  - 分隔线 520.5；角度图标 (532.25) 与输入框 (542.5–597) 和「°」；分隔线 609.5；斜切「H:」(617) 输入框 (629.5–677.5)「°」，「V:」(691.5) 输入框 (702.5–750.5)「°」。
  - 分隔线 763.5；「Interpolation:」(770) 与下拉 (839.5–901.5)：Nearest Neighbor、Bilinear、Bicubic（默认）、Bicubic Smoother、Bicubic Sharper、Bicubic Automatic。
  - 变形切换按钮（919，点击进入 Warp 模式）；分隔线 935；取消 ⦸ (983) 与确认 ✓ (1011.5)。
- **变形进行中**（`warp_bar`，Photoshop 2026 的位置）：暗色的参考点开关；分隔线 161.5；「Split:」(171.5) 与三个拆分按钮（208.5、234、260，暗色，还没有拆分）；分隔线 282；「Grid:」(290.5) 与暗色的「Default」下拉 (319.5–378.5)；分隔线 386.5；「Warp:」(395) 与样式下拉 (428.5–518.5，只有 Custom 可选，Arc、Flag 等 15 种置灰)；暗色的方向与 Bend/H/V 框（Bend 631–678、H 713–760、V 793–840，后面是「%」）；打开状态的变形切换按钮 (907–931，`#383838` 底，点击回到自由变换)；分隔线 935；复位 (951.5，网格回到平整)；取消 (983)；确认 (1013)。
  - 输入框（`value_box`）：获得焦点时保留输入的文字，按 Enter 或失去焦点时提交（`typed_number` 取开头的数字，后面可带任意单位，如「px」「pt」「%」「°」），提交用的 Enter 不再传给画布（否则会确认变换）。W、H、角度、斜切的修改以参考点为轴（`FreeTransform::pivoting`）；链接时 W 与 H 按比例一起变。有自由四角（扭曲后）时输入框不可用。

## 已知限制

选项栏的外框、所有工具的选项、裁剪、自由变换与变形的栏都已按 Photoshop 2026 实测定位。哪些设置真正生效、哪些只保存在 `tool_settings` 里，见 `options_tools.md` 与上文各工具的说明。
