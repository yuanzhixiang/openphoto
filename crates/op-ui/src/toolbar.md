# toolbar.rs：左侧工具栏

## 组件职责

窗口左侧的竖向工具栏，对应 Photoshop 的 Tools 面板（单列布局）。

## 布局（自上而下）

1. 顶部的折叠箭头「>>」和一条点状拖拽手柄，只有外观。
2. 工具按钮，顺序与 Photoshop 默认工具栏一致：Move、Rectangular Marquee、Lasso、Object Selection、Crop、Frame、Eyedropper、Spot Healing Brush、Brush、Clone Stamp、History Brush、Eraser、Paint Bucket、Blur、Dodge、Pen、Horizontal Type、Path Selection、Rectangle、Hand、Zoom。在 Crop、Eyedropper、Pen、Hand 前留出小间隔，表示分组。
3. Edit Toolbar（「…」）按钮，只有外观。
4. 前景色/背景色色块。
5. Quick Mask 和 Change Screen Mode 按钮，只有外观。

## 工具按钮

- 38 参考像素见方，当前工具有 `#383838` 深色底，悬停变浅。
- 有同组工具的按钮右下角画一个小三角（Hand、Zoom、Frame 没有）。目前点击小三角不会展开同组工具。
- 悬停提示为「工具名 (快捷键)」，例如「Move Tool (V)」。
- 图标使用 Phosphor 图标，对应关系见 `icons.md`。

## 前景色/背景色

- 前景色在左上，背景色在右下，相互错开叠放；色块有浅色内边框和深色外边框。
- 左上的小图标：恢复默认颜色（前景黑、背景白），等同于按 D。
- 右上的小图标：交换前景色与背景色，等同于按 X。
- 点击前景色或背景色色块：让 Color 面板转为编辑对应颜色。Photoshop 里点击会打开拾色器，这里目前没有拾色器对话框。

## 已知限制

- 不能展开同组工具（例如从 Rectangular Marquee 切到 Elliptical Marquee）。
- 工具栏不能折叠成双列，也不能拖动。
