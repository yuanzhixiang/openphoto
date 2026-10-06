# dialogs/mod.rs：模态对话框

只负责导出各对话框：Canvas Size（`canvas_size.md`）、Fill（`fill.md`）、Trim（`trim.md`）、调整对话框 Threshold、Posterize、Levels、Hue/Saturation、Exposure（`adjust.md`）和 Color Picker（`color_picker.md`）。外框和按钮等公共部件见 `common.md`。

任一对话框打开期间，`AppState::modal_open()` 为真：所有菜单命令禁用，单键快捷键不触发，与 Photoshop 的模态对话框行为一致。
