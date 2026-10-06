# dialogs/new_layer.rs：New Layer 对话框

## 组件职责

Layer › New › Layer...（⇧⌘N）弹出的对话框，设置新图层的名称、颜色标签、混合模式、不透明度和是否用中性色填充，对应 Photoshop 2026 的 New Layer 对话框。⌥⇧⌘N 跳过对话框直接新建（见 `commands.md`），Layers 面板底部「新建图层」按钮按住 ⌥ 单击时也会弹出它。

## 布局（Photoshop 2026 实测，相对对话框左上角的 pt）

对话框 552 × 186，居中显示，标题栏与边框见 `common.md` 的 `frame`；标题「New Layer」用粗体系统字体 14 pt。文字用 `theme::dialog_medium`（macOS 系统字体，字重 510，12 pt），颜色 `#f1f1f1`。

- 「Name」右对齐到 x 49、中心 y 61；名称输入框 (59, 49)–(243, 71)，文字左侧留 10，打开时获得焦点并全选，默认值为下一个「Layer N」（`Document::next_layer_name`）。
- 「Color」右对齐到 x 280.5；颜色下拉 (290, 48)–(450, 72)：1 pt `#7a7a7a` 边、圆角 3，左边是 14 pt 的颜色方块（None 时为带叉的空心方块），x 321 起是颜色名，距右边 13.5 是 10 × 6 pt 的 V 形箭头。菜单列出 `LayerColor` 的全部八项。
- 「Use previous layer to create clipping mask」复选框 (58, 83)，12 pt 见方，文字在框右 9.5。本应用还不支持剪贴蒙版，所以它总是置灰且不勾选（Photoshop 中可用）。
- 「Mode」右对齐到 x 49、中心 y 125.5；模式下拉 (58, 113)–(218, 137)，值在左侧 9；菜单按 Photoshop 的分组列出全部混合模式，组间有分隔线。
- 「Opacity」右对齐到 x 280.5；数值框 (290, 113)–(340, 137)（文字左侧留 11.5，可直接输入，带不带 % 都可以）；右侧紧接带 V 形箭头的小框 (339, 113)–(358, 137)，点击弹出 0–100 的滑块。
- 「Fill with ‹mode›-neutral color」复选框 (58, 148) 与色块 (192, 142)–(216, 166)：只有当前模式有中性色（`op_core::neutral_color`）时可用，文字变为如「Fill with Multiply-neutral color」，色块显示中性色并跟在文字后面 12 pt；否则文字为「Fill with neutral color」并置灰，色块为灰色。
- 按钮：OK (462, 48)–(532, 72)，默认按钮，1 pt 亮边 `#f1f1f1`；Cancel (462, 84)–(532, 108)，1 pt `#727272` 边。都是胶囊形（`common::ps_button`），标签为粗体系统字体 13 pt，悬停或按下时底色变化。

## Layer from Background 版本

Layer › New › Layer from Background... 与双击 Layers 面板中的背景图层时弹出同一个对话框的变体（`NewLayerDialog::from_background`）：标题仍是「New Layer」，高 157（没有中性色那一行），名称默认为「Layer 0」，「Use previous layer...」置灰。确认后背景图层变为普通图层，并使用对话框中的名称、颜色标签、混合模式和不透明度（`panels::layer_from_background_with`），记录「Layer From Background」。

## New Group 与 New Group from Layers 版本

Layer › New › Group... 与 Group from Layers... 弹出同一布局的变体（`NewLayerDialog::group`，`Kind::Group` / `Kind::GroupFromLayers`）：标题分别为「New Group」「New Group from Layers」，高 128，没有剪贴蒙版和中性色两行，Mode 行上移 29 pt（中心 y 96.5），默认名称为下一个「Group N」，默认模式为 Pass Through，模式菜单最上面多一项「Pass Through」（与 Photoshop 2026 截图一致）。确认后新建空组或把选中的图层编组，并使用对话框中的名称、颜色标签、模式和不透明度（`panels::new_group_from`），分别记录「New Group」「Group Layers」。

## 交互

- Enter 或 OK：按当前值新建图层（`panels::new_layer_from`）：插在当前图层上方并选中，设名称（为空时用「Layer」）、颜色标签、混合模式和不透明度（0–100% 截断）；勾选中性色填充且模式有中性色时整幅画布填该颜色。记录一条「New Layer」历史。
- Esc 或 Cancel：关闭，不做任何事。
- 不透明度不是数字时 OK 不可用。
- 对话框打开期间菜单与快捷键不生效（`AppState::modal_open`）。

## 已知限制

- 「Use previous layer to create clipping mask」不可用（剪贴蒙版尚未实现）。
- Photoshop 中不透明度箭头弹出的是 Photoshop 样式的滑块；这里是 egui 的滑块。

## 测试覆盖

- `defaults_and_values`（单元测试）：默认值；不透明度的解析、截断和非法输入；中性色填充只在有中性色的模式下生效。
- `new_layer_dialog_names_colors_and_blends`（UI 测试）：⇧⌘N 打开对话框，输入「Shade」，从颜色菜单选 Violet、从模式菜单选 Multiply，勾选中性色填充后点 OK，得到名为 Shade、紫色标签、Multiply、整幅白色的图层并记录「New Layer」；Esc 取消不新建；⌥⇧⌘N 不弹对话框直接得到「Layer 1」。
- `screenshot_new_layer_dialog`（截图，`#[ignore]`）：用于与 Photoshop 的截图逐像素比对。
