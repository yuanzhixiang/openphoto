# commands.rs：命令与快捷键

## 职责

把菜单项和快捷键统一成 `Command`，由 `run()` 在一处执行。菜单只负责发出命令，快捷键只负责识别按键。

## 命令与快捷键

快捷键与 Photoshop 2026 默认值一致（通过辅助功能接口读取 Photoshop 菜单得到）。⌘ 在其它平台对应 Ctrl。

| 命令 | 菜单位置 | 快捷键 |
|---|---|---|
| New | File › New... | ⌘N |
| Open | File › Open... | ⌘O |
| Close | File › Close | ⌘W |
| CloseAll | File › Close All | ⌥⌘W |
| CloseOthers | File › Close Others | ⌥⌘P |
| ExportAs | File › Export › Export As... | ⌥⇧⌘W |
| Undo | Edit › Undo | ⌘Z |
| Redo | Edit › Redo | ⇧⌘Z |
| ToggleLastState | Edit › Toggle Last State | ⌥⌘Z |
| CanvasSize | Image › Canvas Size... | ⌥⌘C |
| Rotate180 | Image › Image Rotation › 180° | 无 |
| Rotate90Clockwise | Image › Image Rotation › 90° Clockwise | 无 |
| Rotate90CounterClockwise | Image › Image Rotation › 90° Counter Clockwise | 无 |
| FlipCanvasHorizontal | Image › Image Rotation › Flip Canvas Horizontal | 无 |
| FlipCanvasVertical | Image › Image Rotation › Flip Canvas Vertical | 无 |
| Crop | Image › Crop | 无 |
| Trim | Image › Trim... | 无 |
| Invert | Image › Adjustments › Invert | ⌘I |
| Desaturate | Image › Adjustments › Desaturate | ⇧⌘U |
| Equalize | Image › Adjustments › Equalize | 无 |
| Threshold | Image › Adjustments › Threshold... | 无 |
| Posterize | Image › Adjustments › Posterize... | 无 |
| Levels | Image › Adjustments › Levels... | ⌘L |
| HueSaturation | Image › Adjustments › Hue/Saturation... | ⌘U |
| Exposure | Image › Adjustments › Exposure... | 无 |
| LastFilter | Filter › Last Filter | ⌃⌘F |
| Average | Filter › Blur › Average | 无 |
| BoxBlur | Filter › Blur › Box Blur... | 无 |
| GaussianBlur | Filter › Blur › Gaussian Blur... | 无 |
| AddNoise | Filter › Noise › Add Noise... | 无 |
| Median | Filter › Noise › Median... | 无 |
| Mosaic | Filter › Pixelate › Mosaic... | 无 |
| UnsharpMask | Filter › Sharpen › Unsharp Mask... | 无 |
| Solarize | Filter › Stylize › Solarize | 无 |
| HighPass | Filter › Other › High Pass... | 无 |
| Maximum | Filter › Other › Maximum... | 无 |
| Minimum | Filter › Other › Minimum... | 无 |
| Offset | Filter › Other › Offset... | 无 |
| NewLayer | Layer › New › Layer | ⇧⌘N |
| DeleteLayer | Layer › Delete › Layer | 无 |
| ToggleLayerVisibility | Layer › Hide Layers / Show Layers | ⌘, |
| LayerFromBackground | Layer › New › Layer from Background | 无 |
| LayerViaCopy | Layer › New › Layer Via Copy | ⌘J |
| LayerViaCut | Layer › New › Layer Via Cut | ⇧⌘J |
| DuplicateLayer | Layer › Duplicate Layer | 无 |
| DeleteHiddenLayers | Layer › Delete › Hidden Layers | 无 |
| BringToFront | Layer › Arrange › Bring to Front | ⇧⌘] |
| BringForward | Layer › Arrange › Bring Forward | ⌘] |
| SendBackward | Layer › Arrange › Send Backward | ⌘[ |
| SendToBack | Layer › Arrange › Send to Back | ⇧⌘[ |
| MergeDown | Layer › Merge Down | ⌘E |
| MergeVisible | Layer › Merge Visible | ⇧⌘E |
| FlattenImage | Layer › Flatten Image | 无 |
| Fill | Edit › Fill... | ⇧F5 |
| FillForeground | 无（Photoshop 的隐藏快捷键） | ⌥⌫ |
| FillBackground | 无（Photoshop 的隐藏快捷键） | ⌘⌫ |
| Clear | Edit › Clear | ⌫ / Delete |
| Cut | Edit › Cut | ⌘X |
| Copy | Edit › Copy | ⌘C |
| CopyMerged | Edit › Copy Merged | ⇧⌘C |
| Paste | Edit › Paste | ⌘V |
| PasteInPlace | Edit › Paste Special › Paste in Place | ⇧⌘V |
| SelectAll | Select › All | ⌘A |
| Deselect | Select › Deselect | ⌘D |
| Reselect | Select › Reselect | ⇧⌘D |
| SelectInverse | Select › Inverse | ⇧⌘I |
| ZoomIn | View › Zoom In | ⌘+（同时接受 ⌘=） |
| ZoomOut | View › Zoom Out | ⌘- |
| FitOnScreen | View › Fit on Screen | ⌘0 |
| ActualPixels | View › 100% | ⌘1 |
| ToggleHistory | Window › History | 无 |

## 可用条件（`enabled`）

同时决定命令能否执行和菜单项是否置灰：

- 有输入框获得键盘焦点时（`AppState::typing`），Cut、Copy、CopyMerged、Paste、PasteInPlace 始终可用（包括模态对话框里的输入框），执行时作用于输入框的文字（见下文「剪贴板」）。
- 除此之外，有模态对话框打开时所有命令不可用。
- New、Open、ToggleHistory 始终可用。
- Undo、ToggleLastState：当前文档能撤销时可用；Redo：能重做时可用。
- DeleteLayer：当前文档的图层多于 1 个时可用。
- CloseOthers：打开的文档多于 1 个时可用。
- ToggleLayerVisibility、DuplicateLayer、LayerViaCopy：当前文档有选中图层时可用；LayerViaCut 还要求有选区。
- Crop：当前文档有选区时可用。LastFilter：有当前文档且本次运行中用过滤镜时可用。
- LayerFromBackground：文档有背景图层时可用。DeleteHiddenLayers：既有隐藏图层也有可见图层时可用。
- Arrange 四项：当前图层按该命令能移动时可用（`layer_ops::arrange_target`）。MergeDown：当前图层和它下面的图层都可见时可用；MergeVisible：可见图层多于一个时可用；FlattenImage：文档不是只有一个背景图层时可用。
- Deselect、SelectInverse：当前文档有选区时可用；Reselect：没有选区且有可恢复的选区时可用；SelectAll：有当前文档时可用。
- 其余命令：有当前文档时可用。Paste 不检查剪贴板里有没有内容（读取系统剪贴板里的图片代价较高，不适合每帧检查），剪贴板为空时执行 Paste 什么也不做。

## 填充与清除

- Fill：打开 Fill 对话框（见 `dialogs/fill.md`）。
- FillForeground / FillBackground：立即用前景色 / 背景色填充当前图层的选区（没有选区时整个图层），Normal、100%，记录「Fill」。不能填充时弹出 Photoshop 的提示。
- Clear：有选区时清除选区内容（背景图层上填背景色），记录「Clear」；没有选区时删除当前图层（图层多于一个时），与 Photoshop 2026 一致。
- ⌥⌫、⌘⌫、⌫ 不是菜单快捷键：macOS 上由 `from_shortcuts_beside_menu`、其它平台由 `from_shortcuts` 在 egui 里识别。正在输入框里输入文字时不处理这些键，以免影响删除文字。

## 选区命令的历史记录

Select All、Deselect、Reselect、Inverse 各记录一条历史，名称分别为「Select All」「Deselect」「Reselect」「Select Inverse」，与 Photoshop 一致。

## 画布命令

- Image Rotation 五项：旋转或翻转整个文档（`image_ops::reorient`），旋转记录「Rotate Canvas」，翻转记录「Flip Canvas Horizontal」/「Flip Canvas Vertical」。
- Crop：裁剪到选区外接矩形（`image_ops::crop_to_selection`），记录「Crop」。
- Trim：打开 Trim 对话框（见 `dialogs/trim.md`）。

## 调整命令

- Invert、Desaturate、Equalize：立即对当前图层（选区内）应用（`op-core` 的 `adjust.md`），记录同名历史；图层隐藏或像素锁定时弹出 Photoshop 的提示（例如「Could not complete the Invert command because the target layer is hidden.」）。
- Threshold、Posterize、Levels、Hue/Saturation、Exposure：先做同样的检查，失败时弹出提示；通过后计算直方图（Levels 用 R/G/B 合并的通道直方图，其余用亮度直方图）、保存文档快照，打开对话框（见 `dialogs/adjust.md`，预览流程见 `lib.md`）。

## 滤镜命令

- Average、Solarize：立即对当前图层应用（`op-core` 的 `filter.md`），记录同名历史，并记为 `last_filter`。
- 带设置的滤镜（Box Blur、Gaussian Blur、Add Noise、Median、Mosaic、Unsharp Mask、High Pass、Maximum、Minimum、Offset）：检查同调整命令，通过后保存快照并打开对话框（`dialogs/adjust.md`）。
- LastFilter：用 `last_filter` 的设置立即再应用一次，记录该滤镜的名称，不弹出对话框。
- 失败时弹出 Photoshop 的提示，例如「Could not complete the Gaussian Blur command because the target layer is hidden.」。

## 图层命令

像素与顺序规则见 `op-core` 的 `layer_ops.md`。成功后记录的历史名称：Duplicate Layer →「Duplicate Layer」，Layer Via Copy / Cut →「Layer Via Copy」/「Layer Via Cut」，Layer from Background →「Layer From Background」，Delete › Hidden Layers →「Delete Hidden Layers」，Arrange 四项 →「Layer Order」，Merge Down / Merge Visible / Flatten Image → 同名。Layer Via Copy / Cut 失败时弹出 Photoshop 的提示（例如选区内只有透明像素时「Could not complete the Layer Via Copy command because the selected area is empty.」）。

菜单快捷键里的 `[`、`]` 在传给 muda 时写成符号本身（muda 不认识 egui 的 `OpenBracket` / `CloseBracket` 名称）。

## 剪贴板

Cut、Copy、CopyMerged、Paste、PasteInPlace 由 `actions::clipboard` 执行：

- 有输入框获得焦点时，把操作转交给输入框：Cut/Copy（CopyMerged 视同 Copy）转成 egui 的 `Event::Cut` / `Event::Copy`，Paste/PasteInPlace 读取系统剪贴板的文字转成 `Event::Paste`（没有文字时什么也不做）。这些事件放进 `AppState::forward_events`，由 `OpenPhotoApp::raw_input_hook` 注入 egui 的下一帧输入，并请求重绘。macOS 上这几个菜单项带 ⌘X/⌘C/⌘V 快捷键，按键被原生菜单接住、egui 收不到，所以需要这样转交，输入框里的剪切、复制、粘贴才能正常工作。
- 否则对当前文档执行（像素规则见 `op-core` 的 `clipboard.md`）：
  - Copy / Copy Merged：复制成功后放进剪贴板（见 `clipboard.md`），不记录历史。
  - Cut：复制并清除，记录「Cut」。
  - Paste / Paste in Place：从剪贴板取出内容，按当前视图的可见区域（`document_view::visible_rect`）计算位置，粘贴为新图层，记录「Paste」。剪贴板为空时什么也不做。
  - 失败时弹出 Photoshop 的提示，例如「Could not complete the Copy command because the selected area is empty.」。

`run()` 执行前会再检查一次 `enabled`，不可用的命令直接忽略。

## 快捷键识别

- **macOS**：带 ⌘ 的快捷键由原生菜单的 key equivalent 处理，egui 收不到这些按键。唯一的例外是 Zoom In：菜单项显示为 Photoshop 的 ⌘+，而 macOS 只在按住 Shift 时才匹配「+」，所以 `from_shortcuts_beside_menu` 额外在 egui 里捕获 ⌘=，转成 ZoomIn。
- `Shortcut` 有 `cmd`（macOS 上为 Command）、`shift`、`alt`、`ctrl`（macOS 上的 Control 键）四个修饰键。传给 muda 时 `ctrl` 写作 `Ctrl+`，在 macOS 上就是 Control 键。
- **没有原生菜单时**（其它平台，以及 macOS 上的无窗口测试）：`from_shortcuts` 在 egui 里按 `SHORTCUT_ORDER` 依次匹配全部快捷键。egui 的 `consume_key` 会忽略多按的 Shift/Alt，所以列表必须把更具体的组合放在前面（例如 ⇧⌘Z 在 ⌘Z 之前、⌥⌘W 在 ⌘W 之前、⇧⌘C 在 ⌘C 之前），否则会被误触发成另一个命令。ZoomIn 同时接受 ⌘= 和 ⌘+。
- egui-winit 不把 ⌘X/⌘C/⌘V 作为按键送出，而是转成 `Event::Cut`、`Event::Copy`、`Event::Paste`（而且只在系统剪贴板有文字时才送出 `Paste`）。`from_shortcuts` 把这些事件识别为 Cut、Copy（按住 Shift 时为 CopyMerged）、Paste（按住 Shift 时为 PasteInPlace）。正在输入框里输入时不识别剪贴板快捷键和事件，由输入框自己处理。

## 与 Photoshop 的差异

- 没有原生菜单的平台上，系统剪贴板只有图片时 ⌘V 收不到任何事件（见上文），只能通过菜单粘贴。

- Photoshop 的 Layer › New › Layer... 会弹出「New Layer」对话框；这里直接创建图层，所以菜单项文字不带省略号。
- Layer from Background、Duplicate Layer 在 Photoshop 中会先弹出对话框，这里直接执行；Flatten Image 在有隐藏图层时 Photoshop 会询问是否丢弃，这里直接丢弃。
