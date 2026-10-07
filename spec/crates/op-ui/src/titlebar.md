# titlebar.rs：自绘标题栏（macOS）

## 组件职责

macOS 上窗口隐藏了系统标题栏（内容延伸到整个窗口），这里自绘一条与选项栏同色的标题栏，看起来和 Photoshop 的窗口一样。红绿灯按钮仍由系统绘制。其它平台使用系统标题栏，不显示这个组件。

## 视觉

- 高 40 参考像素，底色 `#535353`。
- 居中显示「OpenPhoto」（Photoshop 显示「Adobe Photoshop 2026」）。

## 交互

- 按住拖动：移动窗口（`ViewportCommand::StartDrag`）。
- 双击：在最大化与还原之间切换。
