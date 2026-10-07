# options_tools.rs：按控件表布局的工具选项栏

## 组件职责

把工具的选项栏写成控件表（`Item`）：分隔线、标签、笔刷选择器、图标按钮、切换图标、下拉框、输入框、百分比（输入框加滑块箭头框）、复选框、不可用的复选框、色块，坐标都是在 Photoshop 2026 选项栏截图上量出的「栏坐标」（见 `options_kit.md`）。`layout(tool)` 返回工具的控件表，`show` 依次画出并处理交互。目前包括画笔、铅笔、颜色替换、混合器画笔、橡皮擦、背景橡皮擦、魔术橡皮擦、仿制图章、图案图章、历史记录画笔、历史记录艺术画笔、模糊、锐化、涂抹、减淡、加深、海绵；污点修复画笔、修复画笔、修补、内容感知移动、红眼；渐变、油漆桶；吸管、颜色取样器、标尺、注释、计数；钢笔、自由钢笔、弯度钢笔、添加/删除锚点、转换点；横排/直排文字与两个文字蒙版；路径选择、直接选择；矩形、椭圆、三角形、多边形、直线、自定形状；抓手、旋转视图、缩放；画板、透视裁剪、切片、切片选择、画框；选区画笔、移除工具、调整画笔。另有带文字的选择框（`LabeledRadio`）、菜单按钮（`MenuButton`）、分段按钮（`Segments`）、单选图标（`Radio`，如五个渐变类型）、按钮、不可用的下拉框、空的图案框、颜色块、渐变色块（前景色到背景色，勾选 Reverse 时反过来）与选区运算四按钮（`Modes`）。

后加入的控件：不可用的图标（`IconOff`）、只作标记的图标（`Glyph`，如字号、圆角半径、多边形边数图标，不可点）、暗色标签（`LabelOff`）、不可用的空输入框（`FieldOff`）与组合框（`ComboOff`）、组合框（`Combo`：输入框加箭头菜单）、居中提示（`Notice`）、前景色填充色块（`FillSwatch`）、无描边色块（`NoStroke`）、线型下拉（`LineStyle`）、带深色边的颜色块（`ColorFrame`，`fg` 为前景色，`artboard.bg` 随画板背景下拉为白、黑或透明的灰）、执行动作的按钮（`Action`）、视图旋转刻度盘（`Dial`）、自定形状选择框（`ShapePicker`）、带图标的下拉（`IconPopup`，画框的描边位置）、切片的宽高（`SliceSize`，Style 为 Normal 时不可用，选了固定比例或固定大小后可输入）。

## 各工具（栏坐标，pt）

- **钢笔类**（共用开头）：模式下拉 (110–180，Shape / Path / Pixels，默认 Path)；分隔线 183；「Make:」(188) 与三个不可用按钮 Selection... (220.5–295.5)、Mask (299.5–346)、Shape (350–401)；分隔线 404.5；路径运算 (421.5)、分隔线 437.5、路径对齐 (454)、分隔线 470.5、路径排列 (487.5)、分隔线 503.5、齿轮 (520.5，三者与齿轮右下都有菜单小三角)。钢笔：Auto Add/Delete (536.5，默认勾选) 与不可用的 Align Edges (639)；自由钢笔：Magnetic (536.5) 与 Align Edges (605)；弯度钢笔：Align Edges (537)。
- **添加锚点、删除锚点、转换点**：只有一行居中的「No options for the … Tool.」，中心在 (栏宽 + 124.5) / 2（1350 pt 宽的窗口里是 737.25，与 Photoshop 一致）。
- **文字工具**：切换方向 (125.75)；分隔线 146；字体组合框 (152–296.5，箭头框到 311，只有 Source Sans 3)；样式组合框 (320–462 / 476.5，Regular / Semibold)；分隔线 482.5；字号图标 (498.5) 与字号组合框 (516.5–580 / 594.5，6–72 pt 常用字号，可输入 0.5–1296)；分隔线 600.5；三个对齐单选 (620.5、646.5、672.5；直排工具为顶/中/底对齐图标)；分隔线 691.5；文字颜色 (697.5–725.5，前景色)；分隔线 730.5；变形文字 (750)、不可用的 3D (784.5)；分隔线 807.5；字符和段落面板 (831.5)；分隔线 851.5。
- **路径选择、直接选择**：「Select:」与下拉 (148–247.5)；分隔线 255.5；「Fill:」色块 (284–314)、「Stroke:」无描边 (353.5–383.5)；不可用的描边宽度与线型；分隔线 512.5；不可用的 W (531–576.5)、链接、H (626–671.5)；分隔线 685；路径运算 (707)、对齐 (749.75)、排列 (793)，各隔分隔线；不可用的 Align Edges (823.5)；分隔线 908.5；齿轮 (930.5)；Constrain Path Dragging (951.5)。
- **形状工具**（共用开头）：模式下拉 (110–180，默认 Shape)；分隔线 183；「Fill:」(190) 色块 (207–237，当前前景色)；「Stroke:」(241.5) 色块 (276–306，无描边)；描边宽度组合框 (310–361.5 / 376)；线型下拉 (381.5–430.5)；分隔线 435；「W:」(437.5) 输入框 (453.5–499)、链接切换 (515.5)、「H:」(535) 输入框 (548.5–594)；分隔线 603；路径运算 (620，实心方块)、对齐 (652.75)、排列 (686)、齿轮 (719)，之间分隔线 636、669、702。之后：矩形、三角形为分隔线 735、圆角图标 (749) 与半径 (759–804.5)、分隔线 807、Align Edges (811)；椭圆直接 Align Edges (735)；多边形为分隔线 735、边数图标 (749) 与边数 (761–806.5)、分隔线 809、圆角 (823) 与半径 (833–878.5)、分隔线 881.5、Align Edges (885.5)；直线为「Weight:」(736.5) 与粗细 (776.5–822)、Align Edges (825)；自定形状为「Shape:」(736.5)、形状框 (772–802，黑底白色形状) 与箭头框 (到 814)、Align Edges (817)。
- **抓手**：Scroll All Windows (110)；分隔线 227；100% (239–287)、Fit Screen (296–365.5)、Fill Screen (374.5–445.5)；分隔线 454。
- **旋转视图**：「Rotation Angle:」(112) 与输入框 (188.5–234，输入后归入 ±180°)；刻度盘 (中心 252，半径 10.5，按角度画刻线与中心点)；Reset View (272–346，角度归零)；Rotate All Windows (354.5)。
- **缩放**：放大、缩小两个单选 (123、152，设置 `zoom.out`)；分隔线 169；Resize Windows to Fit (178，默认勾选)、Zoom All Windows (312)、Scrubby Zoom (429.5，默认勾选)；100% (524.5–572.5)、Fit Screen (581.5–651.5)、Fill Screen (660–731.5)。
- **画板**：「Size:」与下拉 (138.5–379，默认 iPhone 8/7/6)；「Width:」(388.5) 输入框 (423.5–499，750 px)；「Height:」(509) 输入框 (547–622.5，1334 px)；背景色块 (630.5–647.5) 与下拉 (652–734.5，White / Black / Transparent)；分隔线 742.5；不可用的竖版、横版 (764.5、798.5)；分隔线 819.5；添加画板 (841)；分隔线 862.5；对齐 (884.25)；分隔线 905.5；齿轮 (927.5)。
- **透视裁剪**：「W:」输入框 (126.5–200.5)、交换 (215.5)、「H:」输入框 (245–319)；分隔线 323；「Resolution:」输入框 (385.5–459.5) 与单位下拉 (462.5–533.5)；分隔线 537；Front Image (544.5–623.5，填入当前文档的宽、高与分辨率)、Clear (632.5–678.5，清空三个输入框)；分隔线 685；Show Grid (690，默认勾选)。
- **切片**：「Style:」与下拉 (142.5–257，Normal / Fixed Aspect Ratio / Fixed Size)；Width (301.5–362)、Height (410–470.5)；分隔线 478.5；不可用的 Slices From Guides (488–601.5)。
- **切片选择**：四个排列图标 (123、149、174.75、200.75)；不可用的 Promote (222.5–284.5)、Divide... (293.5–353.5)；分隔线 362；对齐左/水平居中/右与垂直分布 (383.75、409.75、435.75、470)；分隔线 491；对齐顶/垂直居中/底与水平分布 (513、539、565、599)；分隔线 620；••• (642)；分隔线 663；Hide Auto Slices (672.5–772.5)；分隔线 781；切片选项 (803)。
- **选区画笔**：Add (108–157，图标中心 121，文字从 131) 与 Subtract (160–233) 两个带文字的选择框（`LabeledRadio`，选中的为按下框，另一个变暗）；「Opacity:」(243.5) 与百分比 (288–332.5 / 347)；笔刷选择器 (372，默认 200)；齿轮 (423)。
- **移除工具**：加/减画笔单选 (128.5、157)；「Size」(175.5) 与组合框 (198.5–235 / 249.5，默认 50)；压感切换 (275，默认按下)；分隔线 294.5；齿轮 (312.5)；分隔线 329.5；「Find distractions」菜单按钮 (339–439.5)；分隔线 444；「Auto (May use g...」菜单按钮 (453.5–557)；分隔线 561；Sample all layers (566)、Remove after each stroke (671.5，默认勾选)、Create new layer (818.5)；分隔线 923.5；不可用的反馈 (941.5)；分隔线 958.5；不可用的复位 (976) 与确认 (1009.5)。
- **调整画笔**：「Adjustment:」与下拉 (173–322，Color and vibrance 等)；分隔线 326；减/加画笔单选 (346、371.5，默认加)；分隔线 391；笔刷选择器 (410，默认 100，软边)；压感 (457)；分隔线 476；选择主体 (494.75)；分隔线 511；Overlay (517)；分隔线 579；「Opacity:」(585.5) 与百分比 (629.5–667 / 681.5)；不透明度压感 (701.75)；分隔线 720.5；「Flow:」(727.5) 与百分比 (754.5–792 / 806.5)；喷枪 (826)。
- **画框**：五个形状单选 (123、149、175、201、227：矩形、椭圆、三角形、六边形、自定，各带叉)；分隔线 250；「Stroke:」无描边 (290.5–320.5)、宽度组合框 (324.5–376 / 390.5)、描边位置下拉 (395.5–490.5，带图标，Inside / Center / Outside，默认 Center)；圆角图标 (508.5) 与半径 (518.5–564)。

## 设置的读写

每个控件带一个键：

- 绑定到真实状态的键：`paint.opacity`、`paint.flow`（当前工具的 `PaintOptions` 的不透明度、流量；模糊、锐化的 Strength 与减淡、加深的 Exposure 也是不透明度），笔刷选择器（`PaintOptions` 的大小与硬度），`dodge.range` / `burn.range`（`RetouchOptions` 的色调范围），`sponge.mode`（Desaturate / Saturate），`clone.aligned`，`gradient.kind` 与 `gradient.reverse`（`AppState::gradient`），`bucket.mode`、`bucket.opacity`、`bucket.tolerance`、`bucket.anti_alias`、`bucket.contiguous`、`bucket.all_layers`（`AppState::bucket`），`eyedropper.size` 与 `eyedropper.sample`（`AppState::eyedropper.sample`：Current Layer、Current & Below、All Layers，两个 No Adjustments 选项分别同 All Layers 与 Current & Below，所选项另存在设置里以便显示），仿制图章的 `clone.sample`（`clone_scope`）。这些都按原来的方式生效。
- 文字、形状、视图工具绑定到真实状态的键：`type.style`（`TypeOptions::semibold`）、`type.size`（`TypeOptions::size_pt`，0.5–1296）、`shape.sides`（`ShapeOptions::sides`，3–100）、`shape.weight`（`ShapeOptions::weight`，1–1000 px）；`zoom.out`（缩放工具单击时缩小，见 `document_view.md`）；`Action` 按钮里 100%、Fit Screen、Fill Screen 作用于当前文档，Front Image / Clear 写透视裁剪的输入框，Reset View 把 `rotate.angle` 归零。文字颜色、形状 Fill 显示当前前景色（文字与形状都用前景色）；形状不画描边，所以 Stroke 显示为无颜色。字体只能是内置的 Source Sans 3，输入别的字体名不生效。
- 画笔的 `brush.mode`、铅笔的 `pencil.mode`（`paint_mode`：转成 `PaintMode`）与橡皮擦的 `eraser.mode`（`eraser_mode`：Brush / Pencil / Block）保存在 `tool_settings`，由 `document_view.rs` 在下笔时读取并生效。
- 其余键保存在 `AppState::tool_settings`（文字；复选与切换为 `1` / `0`；下拉为序号）：可以修改并在本次运行中保留，但还没有效果。没有 `PaintOptions` 的工具（图案图章、历史记录艺术画笔、颜色替换、混合器、背景橡皮擦、涂抹）的笔刷大小也存在这里。
- 默认值按 Photoshop：混合器画笔的「每次描边后载入 / 清理」与背景橡皮擦、颜色替换的「连续取样」默认按下；颜色替换的 Mode 默认 Color，Limits 默认 Contiguous；Protect Tones、Protect Detail、Vibrance、Aligned、Anti-alias、Dither、Show Sampling Ring、Use Measurement Scale、Transform On Drop 默认勾选；渐变的 Method 默认 Smooth；修复与修补的 Diffusion 默认 5。
- 百分比输入时限制在 0–100%（画笔的不透明度、流量为 1–100%）。输入框在输入结束时才写回；滑块框拖动时立即写回。

## 笔刷选择器

点击弹出 Size（1–5000 px，对数刻度）与 Hardness（铅笔没有）滑块，改的是当前工具的笔刷。

## 已知差异

- 笔刷大小、不透明度等的初始值是本程序的默认值，Photoshop 截图里是当时的状态（例如 13 px 的画笔）。
- 混合器的平滑图标、对称（蝴蝶）、喷枪、角度渐变等图标是按截图近似重画的；标尺的读数目前固定为 0。
- 各种面板开关（画笔设置、仿制源）与对称、平滑选项、忽略调整图层等按钮还没有效果。
- 钢笔、路径选择、旋转视图、画板、透视裁剪、切片、切片选择、画框这些工具本身还没有实现，它们的选项只保存在 `tool_settings` 里；路径运算/对齐/排列、齿轮、变形文字、面板开关等图标只有外观。
- 选区画笔、移除工具、调整画笔本身还没有实现，选项只保存在 `tool_settings` 里；移除工具的反馈图标、调整画笔的选择主体图标是近似画法。移动工具的栏由 `options_bar.rs` 单独布局（分隔线与边缘同样与 Photoshop 一致）。

## 测试覆盖

- `ui_tests.rs` 的 `options_bars_match_photoshops_layout`（这些工具的分隔线与框的边缘与 Photoshop 一致，误差 1 pt）。
- `fill_and_sample_options_bars_edit_their_settings`：渐变的第二个类型按钮选中 Radial、Reverse 勾选生效；油漆桶 Tolerance 输入 10 生效；污点修复的 Type 点 Create Texture 写入设置。
- `selection_brush_remove_and_adjustment_brush_bars_edit_their_settings`：选区画笔点 Subtract；移除工具勾选 Create new layer、关掉压感；调整画笔勾选 Overlay，都写入设置。
- `type_shape_and_view_options_bars_edit_their_settings`：文字样式输入 Semibold、字号输入「24 pt」生效、居中对齐写入设置；多边形边数输入 7、直线粗细输入「3 px」生效；钢笔 Auto Add/Delete 取消勾选；抓手的 100% / Fit Screen / Fill Screen 改变缩放（Fill 大于 Fit）；缩放工具按下 Zoom Out 后单击画布缩小；旋转角度输入 200 变为 -160°、Reset View 归零；透视裁剪 Front Image 填入 734 px / 811 px、Clear 清空；切片在 Normal 时宽度输入无效，改为 Fixed Size 后可输入。
- `brush_options_bars_edit_their_settings`：画笔 Opacity 输入 50 后不透明度为 0.5；喷枪切换写入设置；仿制图章的 Aligned 取消勾选后 `RetouchOptions::clone_aligned` 为假。
