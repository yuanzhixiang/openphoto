# menu.rs：macOS 原生菜单栏

## 职责

在 macOS 上用 muda 创建原生菜单栏，结构照 Photoshop 2026。仅在 macOS 编译。

## 菜单结构

开关类命令（View › Extras、Rulers、Show › Grid、Guides、Guides › Lock Guides）用带勾选标记的菜单项（`CheckMenuItem`），每帧与 `Command::checked` 同步。


已实现的项发出 `Command`；尚未实现的项显示为置灰，并带上 Photoshop 的快捷键文字，让菜单读起来和 Photoshop 一致。菜单栏的顺序与 Photoshop 相同：OpenPhoto、File、Edit、Image、Layer、Type、Select、Filter、View、Plugins、Window、Help。

- **OpenPhoto**：About OpenPhoto、Services、Hide Others（⌥⌘H）、Show All（系统预置项）；Hide OpenPhoto 用自定义项（⌃⌘H，发出 `Command::HideApp`），因为 ⌘H 属于 View › Extras，与 Photoshop 一致；以及 Quit OpenPhoto（⌘Q）。Quit 不用系统预置项（它会直接结束进程），而是发出 `Command::Quit`，先询问未保存的修改。
- **File**：按 Photoshop 2026 的完整顺序列出。可用的有 New...、Open...；Close、Close All、Close Others；Save（⌘S）、Save As...（⇧⌘S）、Save a Copy...（⌥⌘S）、Revert（F12）；Export › Export As...。其余置灰并带 Photoshop 的快捷键：Browse in Bridge...（⌥⌘O）、Open as Smart Object...、Open Recent ›、Close and Go to Bridge...（⇧⌘W）、Invite to Edit...、Share for Review、Search Adobe Stock...、Search Adobe Express Templates...、Place Embedded...、Place Linked...、Place Free Adobe Stock Images...、Package...、Automate ›、Scripts ›、Import ›、Import from iPhone or iPad、File Info...（⌥⇧⌘I）、Version History、Print...（⌘P）、Print One Copy（⌥⇧⌘P）。
- **Edit**：Undo、Redo、Toggle Last State；Cut、Copy、Copy Merged、Paste，Paste Special 子菜单里的 Paste in Place；Clear；Fill...；Free Transform（⌘T）；Transform › Again（⇧⌘T）、Rotate 180°、Rotate 90° Clockwise、Rotate 90° Counter Clockwise、Flip Horizontal、Flip Vertical（Transform 子菜单按 Photoshop 列出全部项，Scale、Rotate、Skew、Distort、Perspective、Warp 与三个 Split Warp、Remove Warp Split、Convert warp anchor point、Toggle Guides 置灰）。其余项按 Photoshop 的顺序列出并置灰（Fade...、Paste Special 里的 Paste Into（⌥⇧⌘V）与 Paste Outside、Search、Check Spelling...、Find and Replace Text...、Stroke...、Content-Aware Fill...、Content-Aware Scale、Puppet Warp、Perspective Warp、Auto-Align/Blend Layers...、Define Brush Preset/Pattern/Custom Shape...、Purge、Color Settings...、Assign/Convert to Profile...、Keyboard Shortcuts...、Menus...、Toolbar...）。Cut、Copy、Copy Merged、Paste、Paste in Place 带 Photoshop 的快捷键（⌘X、⌘C、⇧⌘C、⌘V、⇧⌘V）；输入框获得焦点时这几项保持可用，并把操作转交给输入框（见 `commands.md`「剪贴板」），所以输入框里的 ⌘C/⌘V 照常工作。Clear 不带快捷键，⌫ 由 egui 处理。macOS 会自动往名为「Edit」的菜单里追加 Writing Tools、AutoFill、Start Dictation、Emoji & Symbols，Photoshop 里也有这些项。
- **Image**：条目、分组和顺序与 Photoshop 一致（Mode ›、Adjustments ›、Auto Tone、Auto Contrast、Auto Color、Image Size...、Canvas Size...、Image Rotation ›、Crop、Trim...、Reveal All、Duplicate...、Apply Image...、Calculations...、Variables ›、Apply Data Set...、Trap...、Analysis ›，Image Size... 与 Canvas Size... 之间还有 Generative Upscale...）。Adjustments › 子菜单按 Photoshop 的顺序列出全部 24 项，其中 Levels...（⌘L）、Exposure...、Hue/Saturation...（⌘U）、Invert（⌘I）、Posterize...、Threshold...、Desaturate（⇧⌘U）、Equalize 可用，其余置灰并显示 Photoshop 的快捷键（Curves ⌘M、Color Balance ⌘B、Black & White ⌥⇧⌘B）。可用的还有 Image Size...（⌥⌘I）、Canvas Size...；Image Rotation › 180°、90° Clockwise、90° Counter Clockwise、Flip Canvas Horizontal、Flip Canvas Vertical（Arbitrary... 置灰）；Crop；Trim...。其余全部置灰。
- **Layer**：按 Photoshop 2026 的完整顺序列出。可用的有：New › Layer、Layer from Background、Layer Via Copy、Layer Via Cut；Duplicate Layer；Delete › Layer、Hidden Layers；Hide Layers；Arrange › Bring to Front、Bring Forward、Send Backward、Send to Back；Merge Down；Merge Visible；Flatten Image。其余项置灰（New 里的 Group/Artboard/Frame 各项，Copy CSS、Copy SVG、Quick Export as PNG、Export As...、Rename Layer...、Layer Style、Smart Filter、New Fill Layer、New Adjustment Layer、Harmonize、Layer Content Options...、Layer Mask、Vector Mask、Create Clipping Mask、Mask All Objects、Smart Objects、Video Layers、Rasterize、New Layer Based Slice、Group Layers、Ungroup Layers、Arrange › Reverse、Combine Shapes、Align、Distribute、Lock Layers...、Link Layers、Select Linked Layers、Matting）。Photoshop 中 Layer from Background...、Duplicate Layer... 会先弹出对话框，这里直接执行，所以标签不带省略号；Photoshop 在只选中一个图层时把 ⌘E 项显示为「Merge Down」，这里只能单选，所以固定为 Merge Down。
- **Select**：All、Deselect、Reselect、Inverse；其余项与 Photoshop 相同但置灰（All Layers、Deselect Layers、Find Layers、Isolate Layers、Color Range...、Focus Area...、Subject、Sky、Select and Mask...、Modify ›、Grow、Similar、Transform Selection、Edit in Quick Mask Mode、Load Selection...、Save Selection...）。
- **Type**：按 Photoshop 的顺序列出全部项（More from Adobe Fonts...、Panels ›、Anti-Alias ›、Orientation ›、OpenType ›、Create Work Path、Convert to Shape、Rasterize Type Layer、Convert to Paragraph Text、Convert to Dynamic Text ›、Warp Text...、Match Font...、Font Preview Size ›、Language Options ›、Update All Text Layers、Manage Missing Fonts、Paste Lorem Ipsum、Load/Save Default Type Styles），全部置灰。
- **Filter**：按 Photoshop 的顺序列出全部项。可用的有：Last Filter（⌃⌘F，标签显示上次使用的滤镜名称，如「Gaussian Blur」，从未使用时为「Last Filter」）；Blur › Average、Box Blur...、Gaussian Blur...；Noise › Add Noise...、Median...；Pixelate › Mosaic...；Sharpen › Unsharp Mask...；Stylize › Solarize；Other › High Pass...、Maximum...、Minimum...、Offset...。其余置灰并带 Photoshop 的快捷键（Adaptive Wide Angle ⌥⇧⌘A、Camera Raw Filter ⇧⌘A、Lens Correction ⇧⌘R、Liquify ⇧⌘X、Vanishing Point ⌥⌘V），Blur Gallery、Distort、Render、Video 子菜单整体置灰。
- **Plugins**：Plugins Panel、Manage Plugins...，置灰。
- **View**：按 Photoshop 2026 的完整顺序列出。可用的有 Zoom In、Zoom Out、Fit on Screen、Fit Layer(s) on Screen、100%、200%、Print Size；Extras（⌘H，带勾选）；Show › Grid（⌘'）、Guides（⌘;）（带勾选）；Rulers（⌘R，带勾选）；Guides › Lock Guides（⌥⌘;，带勾选）、Clear Guides、New Guide...。其余置灰并带 Photoshop 的快捷键：Proof Setup ›、Proof Colors（⌘Y）、Gamut Warning（⇧⌘Y）、Pixel Aspect Ratio ›、Pixel Aspect Ratio Correction、32-bit Preview Options...、Fit Artboard on Screen、Actual Size、Flip Horizontal、Pattern Preview、Screen Mode ›、Show 里的其余项（Target Path ⇧⌘H 等）、Snap（⇧⌘;）、Snap To ›、Guides 里的其余项、Lock Slices、Clear Slices。
- **Window**：History。同时被设为 macOS 的窗口菜单，系统会在里面列出窗口。
- **Help**：OpenPhoto Help（置灰）。

## 动态内容

每帧调用 `update()` 同步菜单状态，只在值真正变化时才调用原生接口：

- 每个命令项的启用状态取自 `Command::enabled`。
- Undo / Redo 的文字跟随历史：「Undo New Layer」「Redo Canvas Size」，没有可撤销/重做的步骤时显示「Undo」「Redo」。
- Hide Layers 的文字在当前图层隐藏时变为「Show Layers」。

## 快捷键

- 快捷键文字由 `Command::shortcut()` 生成 muda 的 accelerator。
- Zoom In 使用逻辑键 accelerator「⌘+」，以便菜单显示与 Photoshop 一致；实际按下的 ⌘= 由 `commands::from_shortcuts` 补充处理。

## 事件传递

菜单事件发生在 egui 帧之外。事件处理函数把命令放进通道并调用 `request_repaint`，下一帧由 `poll()` 取出执行。

## 已知限制

- Image 菜单中 Generative Upscale... 没有列出。
- Edit、Window 菜单，以及 File › Export 子菜单只列出了部分项。
