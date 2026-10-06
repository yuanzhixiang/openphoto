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

- 移出画布的像素保留在图层上（`with_canvas` 平移，或用 `set_pixel_at` 放下选中的像素），移回来时内容还在，与 Photoshop 一致；背景图层例外，它的像素只在画布内，移出去的部分被丢弃。
- 有选区时每一步都遍历整个图层。

## 多个选中图层

没有像素选区时，除活动图层外，其它选中且可以移动的图层（可见、不是背景、没有锁定位置或像素）一起移动同样的位移；背景等不能移动的选中图层留在原地。有像素选区时只移动活动图层的选中像素（与 Photoshop 一致）。

## 测试覆盖

- `selected_layers_move_together`：选中两个普通图层和背景图层，移动后两个普通图层都移动，背景不动。

- `pixels_moved_off_the_canvas_come_back`：把点移到画布左边外 5 px，画布内看不到它，图层上 (−5, 2) 处还在；再移回来恢复原位。
- `selected_pixels_moved_off_the_canvas_are_kept_except_on_the_background`：普通图层上把选中像素移到画布上方外仍保留；背景图层上移出去的部分被丢弃。

- `moves_whole_layer_from_its_start`：多次 `apply` 都相对开始位置计算。
- `moves_only_selected_pixels`：只移动选区内的像素，选区外不动，选区随之移动。
- `background_needs_a_selection_and_leaves_background_color`：背景图层没有选区时拒绝；有选区时原位置填背景色。
