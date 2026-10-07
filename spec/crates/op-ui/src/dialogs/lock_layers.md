# dialogs/lock_layers.rs：Lock Layers 对话框

## 职责

Layer › Lock Layers... 的对话框：设置所有选中图层（背景除外）的锁定。只能从菜单打开；⌘/ 在 Photoshop 中直接切换「全部锁定」，不打开对话框（见 `commands.md` 的 LockLayers / ToggleLockAll）。

## 输入与输出

- `LockLayersDialog::new(locks)`：`locks` 是 `layer_ops::selected_locks` 的结果，每一项只有所有选中图层都开启时才勾选。
- `show(ctx)` 返回 `Outcome::Open`（仍打开）、`Cancel` 或 `Apply(Locks)`。由 `lib.rs` 用 `layer_ops::set_selected_locks` 应用，有变化时记录「Lock Layers」历史（Photoshop 2026 实测的名字，和面板按钮的「Lock Layer」不同）。

## 布局

尺寸 269 × 206 pt，照 Photoshop 2026 的截图（2x）逐像素量取，坐标相对对话框左上角：

- 标题栏同其它对话框（`common::frame`），标题「Lock Layers」为系统字体粗体 14 pt。
- 五行，行中心 y 60、86、112、138、174：
  - 左侧小锁定图标（`ps_icons::paint_scaled`，约 0.45–0.5 倍）：透明 (27.75, 55.75)、像素 (27.75, 82.75)、位置 (27.75, 107.75)、防止自动嵌套 (25, 134.5)、全部 (28, 169.75)，颜色 `#f1f1f1`。
  - 12 pt 复选框，左边 x 52；标签在框右 9.5 pt：Transparency、Image、Position、Prevent auto-nest、All。
- 标签和按钮用面板字体（Adobe Clean，本应用用 Source Sans 3 近似）：标签为 `theme::body()`（常规 11.5 pt），比框的正中低 1 pt；按钮标签为半粗 12 pt。按字宽量出来的，这个对话框不用系统字体。
- OK (179, 48)–(249, 72)：带键盘焦点的默认按钮（`common::ps_focused_button`：`#737373` 底、白边、蓝色焦点环）。Cancel (179, 84)–(249, 108)：普通按钮。

## 交互

- 勾选 All 时，其余四项显示为勾选并置灰，不能修改；取消 All 后它们恢复成勾 All 之前的状态。确认时 All 作为独立标志保存（`lock_all`），其余四项按各自的勾选保存。
- Enter 等于 OK，Esc 等于 Cancel。
- 对话框打开时其它命令不可用（`AppState::modal_open`）。

## 测试

- `ui_tests::lock_layers_dialog`：只选背景时菜单项不可用；打开时勾选已有的锁定；点 Image、Prevent auto-nest 再点 OK 后锁定生效并记录「Lock Layers」；勾 All 后按 Enter 打开全部锁定；Cancel 不改动。
- `ui_tests::cmd_slash_toggles_lock_all`：⌘/ 在背景上弹出提示；在普通图层上不打开对话框，直接打开全部锁定（「Lock Layer」），再按一次清除所有锁定（「Unlock Layer」）。
- `ui_tests::screenshot_lock_layers_dialog`（忽略，手动运行）：截图与 Photoshop 对比用。
