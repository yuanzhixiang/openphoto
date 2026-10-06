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

1. **已有界面的像素级校准**：Image Size（Photoshop 左侧还有预览图）、Fill、Trim、各调整对话框（Threshold、Posterize、Levels、Hue/Saturation、Exposure）与滤镜对话框（Photoshop 的滤镜对话框带预览缩略图和缩放按钮，这里没有）和 Layers 面板的改名输入框、拖动指示线是按 Photoshop 的结构排列的，尚未逐像素比对；窗口框架（标题栏、选项栏、工具栏、图标列、面板列的外框尺寸，文档标签栏、滚动条、状态栏）、History 面板和 Color Picker 已经按 Photoshop 1:1 量取。还没有逐像素比对的是：选项栏内容、工具栏的工具按钮与图标（Photoshop 的图标更大、为实心风格）、右侧各面板（Color、Properties、Layers）内部的元素位置和各面板组的高度、Canvas Size 对话框。
2. **界面字体**：Photoshop 面板使用 Adobe Clean，对话框使用 macOS 系统字体；OpenPhoto 全部使用 Source Sans 3，字形和字宽不同。

### P1

3. **图层操作**：
    - 图层可以超出画布：目前图层与画布同样大，移动到画布外、以及粘贴时超出画布的像素会丢失。
    - 对话框：「New Layer」（⇧⌘N、Layer from Background...、双击背景图层时弹出）、「Duplicate Layer」（含目标文档）、Rename Layer...；Flatten Image 时询问是否丢弃隐藏图层。
    - Group Layers（⌘G）、Ungroup Layers（⇧⌘G）与图层组。
    - Layers 面板多选（以及多选后的 Merge Layers）、Select › All Layers（⌥⌘A）、Arrange › Reverse、Lock Layers...（⌘/）、Link Layers。
    - Layers 面板中把图层拖到「新建」按钮上复制、拖到垃圾桶上删除。
4. **图像尺寸与方向**：Image Size 对话框的预览图、Fit To 预设、百分比/英寸等单位、Automatic/Preserve Details/Bicubic Smoother/Sharper 等重采样方式、打开 Resample 时改分辨率联动像素尺寸、Scale Styles；Image Rotation › Arbitrary...（任意角度）、Reveal All。
5. **裁剪工具**：裁剪框超出画布时扩展画布、旋转与拉直、比例预设与叠加方式、关闭「删除裁剪的像素」、拖动框内移动图像（Photoshop 的默认方式）、切换工具时的「是否裁剪」询问、透视裁剪工具。
6. **变换**：斜切、扭曲、透视、变形（Skew、Distort、Perspective、Warp 及 Split Warp）；自由变换选项栏的数值输入、参考点、插值方式；右键菜单与 ⌘ 拖动角点的自由扭曲；Transform Selection；变换框外观与 Photoshop 的逐像素比对。
7. **调整**：Image › Adjustments 的 Brightness/Contrast、Curves（⌘M）、Color Balance（⌘B）、Black & White（⌥⇧⌘B）、Vibrance、Photo Filter、Channel Mixer、Color Lookup、Gradient Map、Selective Color、Shadows/Highlights、HDR Toning、Replace Color、Match Color；Auto Tone/Contrast/Color；Levels 的单通道（R/G/B）、预设、自动与吸管；Hue/Saturation 的分颜色范围编辑、Colorize 与预设；Exposure 的预设与吸管；各调整对话框记住上次的值；Equalize 在有选区时的「只均化选区 / 按选区均化整幅图像」询问。
8. **滤镜**：Blur 的 Blur、Blur More、Lens/Motion/Radial/Shape/Smart/Surface Blur；Blur Gallery；Distort 全部；Noise 的 Despeckle、Dust & Scratches、Reduce Noise；Pixelate 的 Color Halftone、Crystallize、Facet、Fragment、Mezzotint、Pointillize；Render 全部（Clouds 等）；Sharpen 的 Sharpen、Sharpen Edges、Sharpen More、Smart Sharpen；Stylize 的 Diffuse、Emboss、Extrude、Find Edges、Oil Paint、Tiles、Trace Contour、Wind；Video；Other 的 Custom、HSB/HSL；Filter Gallery；Edit › Fade（⇧⌘F）；Photoshop 滤镜对话框的预览缩略图；已实现滤镜的取值细节（Gaussian Blur 半径与标准差的换算、Add Noise 的强度、Minimum/Maximum 的圆形范围与小数半径、Offset 在背景图层上的「Set to Background」标签）与 Photoshop 核对。
9. **文件**：PSD 的图层组、图层蒙版、调整图层、16/32 位与 CMYK 等模式、ZIP 压缩数据的读写；Photoshop 保存时的选项对话框（PNG/JPEG 选项、多图层存为扁平格式时的提示）；Open Recent、File › New 对话框（尺寸、分辨率、背景内容，以及按剪贴板内容尺寸新建）、Quick Export as PNG、File › Export 子菜单其余项；未保存修改确认框与 Photoshop 的逐像素比对；用 Photoshop 打开 OpenPhoto 写出的 PSD 的实际验证。
10. **其它工具**：画笔的笔刷预设、笔尖形状、压感与平滑，画笔模式（Multiply 等）和橡皮擦的 Pencil/Block 模式；渐变工具的渐变编辑器与预设、Dither、Method（Perceptual/Linear/Classic）、透明度渐变以及 Photoshop 2024 起默认的「渐变填充图层」模式；吸管的取样环与其余取样方式（Current & Below 等）、颜色取样器、磁性套索、套索拖动中按 ⌥ 临时切换多边形、快速选择与对象选择、污点修复画笔/修复画笔/修补/内容感知移动/红眼、涂抹、图案图章、艺术历史记录画笔、仿制图章的取样范围（Current & Below、All Layers）与仿制源面板、历史记录画笔选择其它状态作为来源、修饰工具的 Protect Tones/Vibrance/Protect Detail 与按住拖动时连续累积的效果、背景橡皮擦与魔术橡皮擦、颜色替换与混合器画笔、旋转视图（R）。工具栏按住按钮弹出同组工具列表（目前只支持右键）。
11. **视图**：对齐（Snap ⇧⌘;、Snap To）、智能参考线、画布参考线、参考线版面、Edit/Clear Selected Guides、标尺单位与原点、参考线与网格的颜色设置、参考线保存到 PSD；屏幕模式（F）、Actual Size、Flip Horizontal、Show › Selection Edges/Layer Edges/Pixel Grid 等其余项、Proof Setup/Colors 与 Gamut Warning；状态栏缩放框可输入。
12. **面板**：Navigator、Info、Histogram、Brushes 与 Brush Settings、Channels、Paths、Adjustments、Gradients、Patterns、Styles、Actions、Character、Paragraph、Tool Presets、Clone Source、Layer Comps、Notes、Timeline 等；面板拖动停靠、浮动、折叠为图标，Window › Workspace。

### P2

13. **文字**：横排/直排文字工具、Character 与 Paragraph 面板、Type 菜单（抗锯齿、转为形状、变形文字、栅格化等）。
14. **路径与形状**：钢笔/自由钢笔/弯度钢笔、锚点工具、路径选择/直接选择、Paths 面板；矢量形状图层（目前形状工具生成栅格图层）、实时形状属性、圆角、描边、自定形状工具、Path/Pixels 模式、Combine Shapes。
15. **蒙版**：从透明度建蒙版（From Transparency）、蒙版与图层的链接（移动或变换图层时蒙版跟随）、⌥单击查看蒙版、蒙版属性面板（密度、羽化）、在蒙版上应用滤镜与调整；矢量蒙版、剪贴蒙版（⌥⌘G）、快速蒙版选项（颜色、显示被选区域）、Select and Mask（⌥⌘R）、存储/载入选区。
16. **图层样式**：Blending Options、斜面和浮雕、描边、内阴影、内发光、光泽、颜色/渐变/图案叠加、外发光、投影。
17. **填充图层与调整图层**：Layer › New Fill Layer、New Adjustment Layer。
18. **选择菜单其余项**：Color Range、Focus Area、Transform Selection、All Layers/Deselect Layers/Find Layers/Isolate Layers；Modify 的「Apply effect at canvas bounds」用于 Smooth 与 Feather；选框工具的 Fixed Ratio / Fixed Size 样式。
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
