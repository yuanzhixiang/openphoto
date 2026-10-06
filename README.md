# OpenPhoto 产品规格总览

## 产品定位

OpenPhoto 是用 Rust 编写的 Photoshop 复刻，目标是功能、交互、快捷键与 Photoshop 2026 一致，界面与 Photoshop 原版像素级一致（同一窗口尺寸下逐像素比对）。参照物是运行中的 Photoshop 2026（macOS，默认「中灰」主题、Essentials 工作区）。各模块的具体行为见对应源码路径的 spec；界面层总规范见 `crates/op-ui/src/README.md`。

## 与 Photoshop 的功能差距

本节列出 Photoshop 2026 有、OpenPhoto 目前没有（或只有一部分）的功能，按优先级排列，数据来自 Photoshop 2026 的菜单、工具栏和面板。每补上一项就从这里删除或改写，所以本节始终描述当前的差距。

优先级：

- **P0**：已有界面与 Photoshop 不一致，或缺少最基本编辑流程（打开 → 选择 → 绘制/修改 → 撤销 → 导出）必需的功能。
- **P1**：常用编辑功能。
- **P2**：较常用、实现量较大的功能。
- **P3**：专业、低频或依赖 Adobe 云服务的功能。

### P0

1. **已有界面的像素级校准**：Fill、Trim 对话框和 Layers 面板的改名输入框、拖动指示线是按 Photoshop 的结构排列的，尚未逐像素比对；窗口框架（标题栏、选项栏、工具栏、图标列、面板列的外框尺寸，文档标签栏、滚动条、状态栏）、History 面板和 Color Picker 已经按 Photoshop 1:1 量取。还没有逐像素比对的是：选项栏内容、工具栏的工具按钮与图标（Photoshop 的图标更大、为实心风格）、右侧各面板（Color、Properties、Layers）内部的元素位置和各面板组的高度、Canvas Size 对话框。
2. **界面字体**：Photoshop 面板使用 Adobe Clean，对话框使用 macOS 系统字体；OpenPhoto 全部使用 Source Sans 3，字形和字宽不同。

### P1

3. **图层操作**：
    - 图层可以超出画布：目前图层与画布同样大，移动到画布外、以及粘贴时超出画布的像素会丢失。
    - 对话框：「New Layer」（⇧⌘N、Layer from Background...、双击背景图层时弹出）、「Duplicate Layer」（含目标文档）、Rename Layer...；Flatten Image 时询问是否丢弃隐藏图层。
    - Group Layers（⌘G）、Ungroup Layers（⇧⌘G）与图层组。
    - Layers 面板多选（以及多选后的 Merge Layers）、Select › All Layers（⌥⌘A）、Arrange › Reverse、Lock Layers...（⌘/）、Link Layers。
    - Layers 面板中把图层拖到「新建」按钮上复制、拖到垃圾桶上删除。
4. **图像尺寸与方向**：Image › Image Size...（⌥⌘I，含重采样）、Image Rotation › Arbitrary...（任意角度）、Reveal All。
5. **裁剪工具**：裁剪框的拖动、比例约束、确认/取消。
6. **变换**：Edit › Free Transform（⌘T）、Transform 子菜单（缩放、旋转、斜切、扭曲、透视、翻转、旋转 90°/180°）。
7. **调整**：Image › Adjustments 的 Invert（⌘I）、Desaturate（⇧⌘U）、Brightness/Contrast、Levels（⌘L）、Curves（⌘M）、Exposure、Hue/Saturation（⌘U）、Color Balance（⌘B）、Black & White、Threshold、Posterize、Vibrance、Photo Filter、Channel Mixer、Gradient Map、Selective Color、Shadows/Highlights、Equalize、Replace Color、Match Color；Auto Tone/Contrast/Color。
8. **滤镜**：Blur（Gaussian Blur、Box Blur、Motion Blur 等）、Sharpen（Unsharp Mask、Smart Sharpen 等）、Noise（Add Noise、Median 等）、Pixelate、Stylize、Distort、Render、Other（High Pass、Offset、Minimum/Maximum）、Last Filter（⌃⌘F）。
9. **文件**：保存与打开 PSD（File › Save ⌘S、Save As... ⇧⌘S、Save a Copy...）、Revert（F12）、Open Recent、File › New 对话框（尺寸、分辨率、背景内容，以及按剪贴板内容尺寸新建）、Quick Export as PNG、关闭有未保存更改的文档时的确认。
10. **其它工具**：画笔的笔刷预设、笔尖形状、压感与平滑，画笔模式（Multiply 等）和橡皮擦的 Pencil/Block 模式；渐变工具、吸管的取样大小选项、颜色取样器、套索/多边形套索/磁性套索、魔棒与快速选择、仿制图章、污点修复画笔、模糊/锐化/涂抹、减淡/加深/海绵、历史记录画笔、背景橡皮擦与魔术橡皮擦、旋转视图（R）。工具栏按住按钮弹出同组工具列表（目前只支持右键）。
11. **视图**：标尺（⌘R）、参考线与 View › Guides、网格、对齐（⇧⌘;）、Extras（⌘H）、屏幕模式（F）、Fit Layer(s) on Screen、200%、Print Size、Flip Horizontal，状态栏缩放框可输入。
12. **面板**：Navigator、Info、Histogram、Brushes 与 Brush Settings、Channels、Paths、Adjustments、Gradients、Patterns、Styles、Actions、Character、Paragraph、Tool Presets、Clone Source、Layer Comps、Notes、Timeline 等；面板拖动停靠、浮动、折叠为图标，Window › Workspace。

### P2

13. **文字**：横排/直排文字工具、Character 与 Paragraph 面板、Type 菜单（抗锯齿、转为形状、变形文字、栅格化等）。
14. **路径与形状**：钢笔/自由钢笔/弯度钢笔、锚点工具、路径选择/直接选择、Paths 面板；矩形、椭圆、三角形、多边形、直线、自定形状工具与形状图层、Combine Shapes。
15. **蒙版**：图层蒙版（包括 Edit › Paste Special › Paste Into ⌥⇧⌘V 与 Paste Outside）、矢量蒙版、剪贴蒙版（⌥⌘G）、快速蒙版（Q）、Select and Mask（⌥⌘R）、存储/载入选区。
16. **图层样式**：Blending Options、斜面和浮雕、描边、内阴影、内发光、光泽、颜色/渐变/图案叠加、外发光、投影。
17. **填充图层与调整图层**：Layer › New Fill Layer、New Adjustment Layer。
18. **选择菜单其余项**：Color Range、Focus Area、Modify（Border、Smooth、Expand、Contract、Feather）、Grow、Similar、Transform Selection；选框工具的 Fixed Ratio / Fixed Size 样式。
19. **颜色模式与位深**：Image › Mode 的灰度、位图、双色调、索引颜色、CMYK、Lab、多通道，16/32 位每通道；Color Settings、Assign/Convert to Profile；CMYK 与 Photoshop 一致的数值（需要 ICC 色彩管理）。
20. **Color Picker 余项**：「Add to Swatches」的命名对话框、Color Libraries、CMYK 色域外警告、打开拾色器期间在文档上吸取颜色。
21. **History 面板余项**：快照、从状态新建文档、历史记录画笔源、按操作类型显示不同图标、History Options。

### P3

22. **智能对象与智能滤镜**、Place Embedded/Linked、Package。
23. **画板与 Frame 工具**、切片工具与 Save for Web、Export 子菜单其余项（Artboards/Layer Comps/Layers to Files 等）。
24. **视频与动画**：Timeline、Video Layers、Render Video。
25. **自动化**：Actions、Batch、Scripts、Image Processor、Droplet、Data Sets 与 Variables。
26. **专业滤镜与工具**：Liquify、Camera Raw Filter、Lens Correction、Adaptive Wide Angle、Vanishing Point、Puppet Warp、Perspective Warp、Content-Aware Scale/Fill/Move、Auto-Align/Blend Layers、Photomerge、Merge to HDR Pro、Apply Image、Calculations、Trap、Analysis（测量、计数）、3D/Materials。
27. **依赖 Adobe 云与 AI 的功能**：Generative Fill/Expand/Upscale、Generate Image、Neural Filters、AI Denoise/Sharpen、Select Subject/Sky、Sky Replacement、Remove Background、Reflection Removal、Harmonize、Adobe Stock、Libraries、Bridge、Firefly Boards、Invite to Edit、Share for Review、Version History、Content Credentials、上下文任务栏（Contextual Task Bar）。
28. **其它**：打印、File Info、Plugins 菜单、Keyboard Shortcuts 与 Menus 自定义、Preset Manager、Purge、Help 菜单内容。
