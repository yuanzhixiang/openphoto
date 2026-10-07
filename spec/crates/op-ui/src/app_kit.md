# app_kit.rs：少量 AppKit 调用（仅 macOS）

## 职责

winit 和 muda 没有提供、但要和 Photoshop 行为一致才需要的几个 AppKit 调用。

## 函数

- `hide_app()`：隐藏应用，等同于标准的「Hide」菜单项。OpenPhoto 自己的 Hide 菜单项用 Photoshop 的 ⌃⌘H，所以要自己调用。
- `handling_key_press()`：当前处理的 `NSApp.currentEvent` 是否是按键（`NSEventTypeKeyDown`）。菜单事件处理里用它区分「用快捷键触发」和「用鼠标选择」：Lock Layers... 的菜单项显示 ⌘/，但按 ⌘/ 应该切换全部锁定而不是打开对话框（见 `commands.md`）。

## 约束

- 都只能在主线程调用（菜单事件和事件循环都在主线程）。
- 找不到 `NSApplication` 或没有当前事件时安全返回（`handling_key_press` 返回 `false`，即按菜单选择处理）。
