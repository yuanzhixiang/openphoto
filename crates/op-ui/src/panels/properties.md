# panels/properties.rs：Properties 面板

## 组件职责

显示当前文档的属性，对应 Photoshop 在没有选中特定对象时 Properties 面板显示的「Document」内容。

## 布局

1. 顶部：带边框的文档图标和「Document」字样，下面一条分隔线。
2. 可折叠的「Canvas」分区（默认展开，点击标题切换，折叠状态保存在 egui 临时内存里）：
   - 宽高链条图标。
   - W、H：文档宽高，例如 `734 px`，只读外观的输入框。
   - X、Y：置灰的 `0 px`。
   - 纵向、横向两个方向按钮，与文档方向一致的那个显示为选中（高 ≥ 宽时为纵向）。点击没有功能。
   - 「Resolution: N pixels/inch」。
   - Mode：颜色模式下拉，列出 Photoshop 的全部模式，只有 RGB Color 可选。
   - 位深下拉：8/16/32 Bits/Channel，只有 8 Bits/Channel 可选。
3. 内容超出面板高度时可以滚动。

没有打开的文档时，面板中央显示「No Properties」。

## 已知限制

- 宽高、分辨率不能在这里编辑（Photoshop 里可以直接改）。改画布尺寸请用 Image › Canvas Size。
- 颜色模式与位深不能转换。
