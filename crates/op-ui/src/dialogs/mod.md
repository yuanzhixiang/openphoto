# dialogs/mod.rs：模态对话框

只负责导出各对话框。目前只有 Canvas Size，见 `canvas_size.md`。

对话框打开期间，`AppState::modal_open()` 为真：所有菜单命令禁用，单键快捷键不触发，与 Photoshop 的模态对话框行为一致。
