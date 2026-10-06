# move_tool.rs：移动工具

## 职责

移动工具对像素的修改：移动当前图层的全部像素，或只移动选中的像素。

## 对外接口

- `Move::begin(doc, background)`：开始移动当前图层，保存图层开始时的像素和选区。每次 `apply` 都从这份原始像素计算，所以拖动过程中来回移动不会累积损失。
- `Move::apply(doc, dx, dy)`：显示相对开始位置移动 (`dx`, `dy`) 像素后的结果。
- `MoveError::message()`：不能移动时 Photoshop 的提示：
  - 没有图层：「Could not use the move tool because there is no layer.」
  - 锁定：「Could not use the move tool because the layer is locked.」
  - 隐藏：「Could not use the move tool because the target layer is hidden.」

## 规则

- 没有选区：整个图层的像素平移，空出来的地方透明。
- 有选区：选中的像素（按选中程度部分提起）被移走，原位置变透明（背景图层上是背景色），然后放到新位置，叠在留下的像素上面；选区也随之平移。
- 以下情况不能移动（`Locked`）：图层「锁定位置」或「锁定像素」，或者没有选区时的背景图层。背景图层有选区时可以移动选中的像素，与 Photoshop 一致。

## 已知限制

- 图层与画布同样大小，移出画布的像素会丢失；Photoshop 的图层可以超出画布，移回来时内容还在。
- 有选区时每一步都遍历整个图层。

## 测试覆盖

- `moves_whole_layer_from_its_start`：多次 `apply` 都相对开始位置计算。
- `moves_only_selected_pixels`：只移动选区内的像素，选区外不动，选区随之移动。
- `background_needs_a_selection_and_leaves_background_color`：背景图层没有选区时拒绝；有选区时原位置填背景色。
