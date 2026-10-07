# OpenPhoto 产品规格总览

## 产品定位

OpenPhoto 是用 Rust 编写的 Photoshop 复刻，目标是功能、交互、快捷键与 Photoshop 2026 一致，界面与 Photoshop 原版像素级一致（同一窗口尺寸下逐像素比对）。参照物是运行中的 Photoshop 2026（macOS，默认「中灰」主题、Essentials 工作区）。各模块的具体行为见对应源码路径的 spec；界面层总规范见 `crates/op-ui/src/README.md`。

除了图形界面，`op-mcp` 是一个 MCP 服务器：AI Agent 通过它调用与界面相同的编辑操作（调整、滤镜、图层、选区、画布、填充、历史记录），见 `crates/op-mcp/src/main.md`。

## 与 Photoshop 的功能差距

本节列出 Photoshop 2026 有、OpenPhoto 目前没有（或只有一部分）的功能，按优先级排列，数据来自 Photoshop 2026 的菜单、工具栏和面板。每补上一项就从这里删除或改写，所以本节始终描述当前的差距。

优先级：

- **P0**：已有界面与 Photoshop 不一致，或缺少最基本编辑流程（打开 → 选择 → 绘制/修改 → 撤销 → 导出）必需的功能。
- **P1**：常用编辑功能。
- **P2**：较常用、实现量较大的功能。
- **P3**：专业、低频或依赖 Adobe 云服务的功能。

### P0

1. **已有界面的像素级校准**：Image Size（Photoshop 左侧还有预览图）、Fill、Trim、各调整对话框与滤镜对话框已逐像素对齐，Layers 面板的改名输入框与拖动指示线已按 Photoshop 2026 实测；窗口框架（标题栏、选项栏、工具栏、图标列、面板列的外框尺寸，文档标签栏、滚动条、状态栏）、History 面板和 Color Picker 已经按 Photoshop 1:1 量取（Color Picker 的十六进制框打开时获得焦点，颜色区的标记裁在区域内）。所有工具的选项栏已按 Photoshop 2026 的截图逐个定位（分隔线与各种框的边缘相差不超过 1 pt）。还没有逐像素比对的是：工具栏图标的细节（已按 Photoshop 的样式逐个重画并对齐位置，线条类图标的外形仍是近似，见 `crates/op-ui/src/tool_icons.md`）。面板列的折叠条、组高度、标签栏、Color、Properties（含 Canvas 之后的 Rulers & Grids、Guides、Quick Actions 分区）、Layers 面板、图标列、工具栏底部已逐像素对齐；Canvas Size、Fill、Trim、New Guide、Select › Modify 各对话框与「Save changes」提示框已按 Photoshop 2026 逐点重做。
2. **界面字体**：Photoshop 面板和 UXP 对话框使用 Adobe Clean（Adobe 专有字体，不能内置，用 Source Sans 3 近似，长句会宽约 2.5%），经典对话框、提示框和窗口标题使用 AppKit 绘制的系统字体（`theme::dialog`，opsz 17 加 trak 字距，与 Photoshop 宽度一致）。New Layer、Duplicate Layer、Lock Layers、Canvas Size、Trim、New Guide、Select › Modify（UXP）与 Fill（AppKit）、提示框、调整与滤镜对话框已按这一区分校准。Color Picker（AppKit）也已改用系统字体；File › New 已换成 Photoshop 2026 新版的 New Document 对话框（UXP）并按截图定位。

### P1

3. **图层操作**：
    - 对话框：New Layer 对话框的「Use previous layer to create clipping mask」（依赖剪贴蒙版）；Duplicate Layer 的 Artboard（依赖画板）。New Layer（⇧⌘N）、Layer from Background...、Duplicate Layer... 对话框、Rename Layer...（就地改名）与 Flatten Image 的「Discard hidden layers?」询问已实现。
    - 图层组余项：组的图层蒙版的界面。（图层组的数据结构、合成与穿透模式、⌘G / ⇧⌘G、New Group 对话框、删除组的询问、复制组、面板显示与折叠、拖进拖出组、组整体移动/对齐/自由变换、PSD 读写组已实现。）
    - 对齐时带上链接的图层、按住 ⇧ 单击链接图标临时停用链接。（Layers 面板 ⌘/⇧ 多选、Merge Layers、Select › All Layers / Deselect Layers、多图层移动、删除、隐藏，Align / Distribute，Lock Layers... 对话框、⌘/ 全部锁定、五个锁定按钮与 PSD 的 `lspf` 读写，以及 Link Layers / Unlink Layers / Select Linked Layers、链接图层随移动工具和自由变换一起移动、多选图层一起自由变换、把图层拖到底部栏按钮上（复制、编组、加蒙版、删除）、PSD 的链接读写、拖动多行排序、组的锁定作用于组内图层（行上的实心/空心/暗色锁图标）、⌥ 单击箭头展开全部子组、列表自动滚动到活动图层已实现。）
4. **图像尺寸与方向**：Image Size 的 Auto Resolution... 子对话框、Fit To 的载入/存储预设、原生 macOS 样式的下拉菜单、Preserve Details 的真实算法（目前为 Lanczos 近似）、Scale Styles 的实际效果（依赖图层样式）、拖动改变对话框大小。（Image Size 对话框已按 Photoshop 2026 重做：100% 预览、单位、Fit To 预设、八种重采样方法与 ⌥ 数字键、Resample 与分辨率的联动；Image Rotation › Arbitrary... 的 Rotate Canvas 对话框与任意角度旋转也已实现。）
5. **裁剪工具**：Classic Mode 下转动框（目前总是图像在转）、Fill 的 Generative Expand / Content-Aware Fill、自定义遮挡颜色与 Auto Adjust Opacity、Front Image 与存储预设、Photoshop 的临时「Crop Preview」图层、切换工具时的「是否裁剪」询问、透视裁剪工具、下拉菜单的 macOS 原生外观。（选项栏已按 Photoshop 2026 逐像素重做；比例与尺寸预设、W x H x Resolution 的重采样、超出画布时扩展画布、Delete Cropped Pixels、六种叠加、线性光混合的遮挡层、非 Classic 模式下框居中而图像移动、Classic Mode、框外拖动转动图像（Shift 15°）与 Straighten 拉直已实现。）
6. **变形余项**：Split Warp、网格大小、变形样式预设（Arc、Flag、Wave 等）与 Bend；在画布上拖动参考点、以参考点为轴的鼠标缩放与旋转；变换框与控制点外观的逐像素比对。（选项栏的 X/Y/W/H/角度/斜切数值输入、参考点选择、相对定位、宽高链接、六种插值已按 Photoshop 2026 实现；Select › Transform Selection 与 Warp（Bézier 网格、拖动控制点与曲面）也已实现。）（斜切、扭曲、透视——Edit › Transform 的模式、⌘ / ⌘⇧ / ⌘⌥⇧ 拖动、投影变换——以及变换中的右键菜单已实现。）
7. **调整**：Image › Adjustments 的 Color Lookup、Shadows/Highlights、HDR Toning、Replace Color、Match Color；Levels 与 Curves 的预设、Photoshop 2026 默认的 Auto 算法与 Options...、吸管、Curves 的铅笔模式与 Smooth、Show Clipping、数值输入；Black & White 的 Auto；Photo Filter 的 Photoshop 拾色器；Vibrance 滑块的肤色保护；Gradient Map 的渐变编辑器与多色标、Dither；Auto Color 的中间调中和；Hue/Saturation 的吸管、Invert、目标调整与预设；Brightness/Contrast 的 Auto 与 Photoshop 一致；各调整对话框记住上次的值。（Levels、Curves 已按 Photoshop 2026 的经典对话框重做，支持单通道、显示选项与 Auto；Brightness/Contrast、Color Balance、Hue/Saturation 已按 UXP 对话框重做；五者的算法都按 Photoshop 的实测数据校准；Channel Mixer 与 Selective Color 已实现，算法与 Photoshop 相差不超过 1 级，对话框按 UXP 版重做，预设列表还没有；Vibrance、Posterize、Exposure、Photo Filter、Black & White 的对话框按 UXP 版重做，Exposure、Photo Filter、Black & White（含 Tint）、Vibrance 的 Saturation 已按 Photoshop 的数值校准。）
8. **滤镜**：Blur 的 Lens/Radial/Shape/Smart Blur；Blur Gallery；Distort 的 Displace、Ripple、Shear、Wave、ZigZag；Noise 的 Reduce Noise；Pixelate 的 Color Halftone、Crystallize、Facet、Mezzotint、Pointillize；Render 全部（Clouds 等）；Sharpen 的 Smart Sharpen；Stylize 的 Diffuse、Extrude、Oil Paint、Tiles；Wind 随机分布与 Photoshop 的进一步核对；斜向 Motion Blur、斜向 Emboss 与 Minimum/Maximum 圆形模式的精确算法；Video；Other 的 HSB/HSL；滤镜对话框预览框的缩放与拖动；扭曲示意图与 Photoshop 的细节差异；Filter Gallery；Edit › Fade（⇧⌘F）；已实现滤镜的取值细节（Add Noise 的强度、Offset 在背景图层上的「Set to Background」标签）与 Photoshop 核对。（Gaussian Blur、Box Blur、Unsharp Mask、Median、Minimum/Maximum 方形、High Pass、Mosaic、Solarize 已与 Photoshop 逐级核对；Blur、Blur More、Sharpen、Sharpen More、Find Edges、Motion Blur、Emboss 已实现；Fragment、Custom、Surface Blur、Dust & Scratches、Despeckle、Sharpen Edges、Trace Contour 已与 Photoshop 核对，Custom 对话框按 Photoshop 重做并支持 .acf 载入/存储；带设置的经典滤镜对话框已按 Photoshop 2026 逐点重做，带预览框，并记住上次的值；Twirl、Pinch、Spherize、Polar Coordinates 已按插件式对话框重做，带扭曲示意图。）
9. **文件**：PSD 的调整图层、16/32 位与 CMYK 等模式、ZIP 压缩数据的读写；Photoshop 保存时的选项对话框（PNG/JPEG 选项、多图层存为扁平格式时的提示）；Open Recent、New Document 对话框里的 Adobe Stock 模板与搜索、跨次启动保存最近使用与存储的预设、画板选项、颜色模式、位深与颜色配置文件的切换、Quick Export as PNG、File › Export 子菜单其余项；未保存修改确认框与 Photoshop 的逐像素比对；图层蒙版写入后用 Photoshop 打开的验证。
10. **其它工具**：画笔的笔刷预设、笔尖形状、压感与平滑，渐变工具的渐变编辑器与预设、Dither、Method（Perceptual/Linear/Classic）、透明度渐变以及 Photoshop 2024 起默认的「渐变填充图层」模式；颜色取样器、磁性套索、快速选择与对象选择、污点修复画笔/修复画笔/修补/内容感知移动/红眼、图案图章的图案选择与 Impressionist、艺术历史记录画笔、仿制源面板、历史记录画笔选择其它状态作为来源、修饰工具的 Protect Tones/Vibrance/Protect Detail 与按住拖动时连续累积的效果、魔术橡皮擦、混合器画笔、旋转视图（R）、选区画笔（Selection Brush）与调整画笔（Adjustment Brush，工具栏里已有按钮，但还没有行为）。
11. **视图**：对齐（Snap ⇧⌘;、Snap To）、智能参考线、画布参考线、参考线版面、Edit/Clear Selected Guides、标尺单位与原点、参考线与网格的颜色设置、参考线保存到 PSD；屏幕模式（F）、Actual Size、Flip Horizontal、Show › Selection Edges/Layer Edges/Pixel Grid 等其余项、Proof Setup/Colors 与 Gamut Warning；状态栏缩放框可输入。
12. **面板**：Navigator/Info/Histogram 的余项（面板选项、扩展视图、可输入的缩放）、Brushes 与 Brush Settings、Channels、Paths、Adjustments、Gradients、Patterns、Styles、Actions、Character、Paragraph、Tool Presets、Clone Source、Layer Comps、Notes、Timeline 等；面板拖动停靠、浮动、折叠为图标，Window › Workspace。

### P2

13. **文字**：可再次编辑的文字图层（目前文字确认后成为栅格图层）、插入点移动与文字选择、系统字体列表、字距/字符间距/行距、段落文字（文本框）、直排文字与文字蒙版工具、Character 与 Paragraph 面板、Type 菜单（抗锯齿、转为形状、变形文字、栅格化等）、复杂文字整形与字体回退。
14. **路径与形状**：钢笔/自由钢笔/弯度钢笔、锚点工具、路径选择/直接选择、Paths 面板；矢量形状图层（目前形状工具生成栅格图层）、实时形状属性、圆角、描边、自定形状工具、Path/Pixels 模式、Combine Shapes。
15. **蒙版**：从透明度建蒙版（From Transparency）、蒙版与图层的链接（移动或变换图层时蒙版跟随）、⌥单击查看蒙版、蒙版属性面板（密度、羽化）、在蒙版上应用滤镜与调整；矢量蒙版、剪贴蒙版（⌥⌘G）、快速蒙版选项（颜色、显示被选区域）、Select and Mask（⌥⌘R）、存储/载入选区。
16. **图层样式**：Blending Options、斜面和浮雕、描边、内阴影、内发光、光泽、颜色/渐变/图案叠加、外发光、投影。
17. **填充图层与调整图层**：Layer › New Fill Layer、New Adjustment Layer。
18. **选择菜单其余项**：Color Range、Focus Area、All Layers/Deselect Layers/Find Layers/Isolate Layers；Modify 的「Apply effect at canvas bounds」用于 Smooth 与 Feather；选框工具的 Fixed Ratio / Fixed Size 样式。
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
