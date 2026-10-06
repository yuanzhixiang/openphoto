# menu.rs：macOS 原生菜单栏

## 职责

在 macOS 上用 muda 创建原生菜单栏，结构照 Photoshop 2026。仅在 macOS 编译。

## 菜单结构

已实现的项发出 `Command`；尚未实现的项显示为置灰，并带上 Photoshop 的快捷键文字，让菜单读起来和 Photoshop 一致。

- **OpenPhoto**：About OpenPhoto、Services、Hide OpenPhoto、Hide Others、Show All、Quit OpenPhoto（系统预置项）。
- **File**：New...、Open...；Close、Close All、Close Others；Save（置灰，⌘S）、Save As...（置灰，⇧⌘S）；Export › Export As...。
- **Edit**：Undo、Redo、Toggle Last State；Cut、Copy、Copy Merged、Paste，Paste Special 子菜单里的 Paste in Place；Clear；Fill...。其余项按 Photoshop 的顺序列出并置灰（Fade...、Paste Special 里的 Paste Into（⌥⇧⌘V）与 Paste Outside、Search、Check Spelling...、Find and Replace Text...、Stroke...、Content-Aware Fill...、Content-Aware Scale、Puppet Warp、Perspective Warp、Free Transform、Transform、Auto-Align/Blend Layers...、Define Brush Preset/Pattern/Custom Shape...、Purge、Color Settings...、Assign/Convert to Profile...、Keyboard Shortcuts...、Menus...、Toolbar...）。Cut、Copy、Copy Merged、Paste、Paste in Place 带 Photoshop 的快捷键（⌘X、⌘C、⇧⌘C、⌘V、⇧⌘V）；输入框获得焦点时这几项保持可用，并把操作转交给输入框（见 `commands.md`「剪贴板」），所以输入框里的 ⌘C/⌘V 照常工作。Clear 不带快捷键，⌫ 由 egui 处理。macOS 会自动往名为「Edit」的菜单里追加 Writing Tools、AutoFill、Start Dictation、Emoji & Symbols，Photoshop 里也有这些项。
- **Image**：条目、分组和顺序与 Photoshop 一致（Mode ›、Adjustments ›、Auto Tone、Auto Contrast、Auto Color、Image Size...、Canvas Size...、Image Rotation ›、Crop、Trim...、Reveal All、Duplicate...、Apply Image...、Calculations...、Variables ›、Apply Data Set...、Trap...、Analysis ›），只少了 Photoshop 的 Generative Upscale...。除 Canvas Size... 外全部置灰。
- **Layer**：按 Photoshop 2026 的完整顺序列出。可用的有：New › Layer、Layer from Background、Layer Via Copy、Layer Via Cut；Duplicate Layer；Delete › Layer、Hidden Layers；Hide Layers；Arrange › Bring to Front、Bring Forward、Send Backward、Send to Back；Merge Down；Merge Visible；Flatten Image。其余项置灰（New 里的 Group/Artboard/Frame 各项，Copy CSS、Copy SVG、Quick Export as PNG、Export As...、Rename Layer...、Layer Style、Smart Filter、New Fill Layer、New Adjustment Layer、Harmonize、Layer Content Options...、Layer Mask、Vector Mask、Create Clipping Mask、Mask All Objects、Smart Objects、Video Layers、Rasterize、New Layer Based Slice、Group Layers、Ungroup Layers、Arrange › Reverse、Combine Shapes、Align、Distribute、Lock Layers...、Link Layers、Select Linked Layers、Matting）。Photoshop 中 Layer from Background...、Duplicate Layer... 会先弹出对话框，这里直接执行，所以标签不带省略号；Photoshop 在只选中一个图层时把 ⌘E 项显示为「Merge Down」，这里只能单选，所以固定为 Merge Down。
- **Select**：All、Deselect、Reselect、Inverse；其余项与 Photoshop 相同但置灰（All Layers、Deselect Layers、Find Layers、Isolate Layers、Color Range...、Focus Area...、Subject、Sky、Select and Mask...、Modify ›、Grow、Similar、Transform Selection、Edit in Quick Mask Mode、Load Selection...、Save Selection...）。
- **View**：Zoom In、Zoom Out、Fit on Screen、100%。
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
- Type、Filter、Plugins 菜单没有列出；File、Edit、Layer、View、Window 菜单只列出了部分项。
