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
| NewLayer | Layer › New › Layer | ⇧⌘N |
| DeleteLayer | Layer › Delete › Layer | 无 |
| ToggleLayerVisibility | Layer › Hide Layers / Show Layers | ⌘, |
| ZoomIn | View › Zoom In | ⌘+（同时接受 ⌘=） |
| ZoomOut | View › Zoom Out | ⌘- |
| FitOnScreen | View › Fit on Screen | ⌘0 |
| ActualPixels | View › 100% | ⌘1 |
| ToggleHistory | Window › History | 无 |

## 可用条件（`enabled`）

同时决定命令能否执行和菜单项是否置灰：

- 有模态对话框打开时，所有命令不可用。
- New、Open、ToggleHistory 始终可用。
- Undo、ToggleLastState：当前文档能撤销时可用；Redo：能重做时可用。
- DeleteLayer：当前文档的图层多于 1 个时可用。
- CloseOthers：打开的文档多于 1 个时可用。
- ToggleLayerVisibility：当前文档有选中图层时可用。
- 其余命令：有当前文档时可用。

`run()` 执行前会再检查一次 `enabled`，不可用的命令直接忽略。

## 快捷键识别

- **macOS**：带 ⌘ 的快捷键由原生菜单的 key equivalent 处理，egui 收不到这些按键。唯一的例外是 Zoom In：菜单项显示为 Photoshop 的 ⌘+，而 macOS 只在按住 Shift 时才匹配「+」，所以 `from_shortcuts` 额外在 egui 里捕获 ⌘=，转成 ZoomIn。
- **其它平台**：没有原生菜单，`from_shortcuts` 在 egui 里按 `SHORTCUT_ORDER` 依次匹配全部快捷键。egui 的 `consume_key` 会忽略多按的 Shift/Alt，所以列表必须把更具体的组合放在前面（例如 ⇧⌘Z 在 ⌘Z 之前、⌥⌘W 在 ⌘W 之前），否则会被误触发成另一个命令。ZoomIn 同时接受 ⌘= 和 ⌘+。

## 与 Photoshop 的差异

- Photoshop 的 Layer › New › Layer... 会弹出「New Layer」对话框；这里直接创建图层，所以菜单项文字不带省略号。
- Delete Layer 在 Photoshop 里还可以在 Layers 面板选中图层后按 Delete 键触发；这里目前只能通过菜单和 Layers 面板的垃圾桶按钮。
