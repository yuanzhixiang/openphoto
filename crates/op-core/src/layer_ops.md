# layer_ops.rs：图层菜单操作

## 职责

实现 Layer 菜单与 Layers 面板中改变图层结构的操作：复制图层、Layer Via Copy/Cut、背景图层转普通图层、调整顺序、改名、合并、拼合、删除隐藏图层。所有函数只改文档，不记录历史（由 `op-ui` 记录）。

## 对外接口与规则

### 复制

- `duplicate(doc)`：Layer › Duplicate Layer。在活动图层正上方插入一个完全相同的图层（像素、可见性、不透明度、Fill、混合模式、锁定都相同）并设为活动图层，返回新 ID；没有活动图层时返回 `None`。名称为「原名 copy」，已存在时依次为「原名 copy 2」「原名 copy 3」……，与 Photoshop 一致。背景图层的副本是普通图层。
- `can_merge_selected(doc)` / `merge_selected(doc)`：Layer › Merge Layers（多选时的 ⌘E）：选中图层中可见的那些合并成一个，放在最上面那个的位置并沿用它的名字（其中有背景图层时合并进背景图层）；结果的不透明度、填充、混合模式复位，蒙版并入像素；隐藏的选中图层保持不动。至少两个选中图层可见时可用。
- `delete_selected(doc)`：删除所有选中图层；会删光所有图层时拒绝。之后最低被删图层下面的那个（没有时最低的剩余图层）成为活动图层。
- `toggle_selected_visibility(doc)`：Layer › Hide Layers / Show Layers：隐藏所有选中图层；全部已隐藏时改为全部显示。返回现在是否可见。
- 图层组：
  - `can_group(doc)` / `group_selected(doc)`：Layer › Group Layers（⌘G）：选中的图层（连同其中的内容；已在另一个选中组里的不重复算）放进一个新的「Group N」，组放在最上面那个选中图层原来的位置和所在的组里；选中背景图层时不可用。新组被选中。
  - `ungroup(doc)`：Layer › Ungroup Layers（⇧⌘G）：当前组消失，它的直接子图层进入它原来所在的组，并全部被选中。
  - `new_group(doc)`：一个空的「Group N」，插入规则同 `Document::insert_above_active`，并被选中。
  - `can_move_block(doc, id, gap)` / `move_block(doc, id, gap)`：把图层连同其中的内容移到 `gap`（`layers` 中移动前的位置，介于 `gap - 1` 与 `gap` 之间）。新的父组：`gap` 正上方是一个组、正下方是它里面的图层时，就是这个组（放在组内最上面）；否则与正上方的图层同组；在最上面时不在任何组里。背景图层不能动，不能放到背景下面，组不能放进自己里面。Layers 面板拖动排序用它。
  - `delete_selected` 删除组时连同组里的内容一起删除。
- `duplicate_name(doc)`：Duplicate Layer 对话框「As」的默认值（与 `duplicate` 的命名规则相同）。
- `duplicate_named(doc, name)`：复制到同一文档，名称为 `name`。
- `duplicate_into(source, target, name)`：把 `source` 的活动图层复制到另一个文档 `target`：像素位置不变（`with_canvas` 平移 0，超出 `target` 画布的像素保留在画布外），图层蒙版扩展（显示）或裁到新画布，背景图层的副本是普通图层；插在 `target` 活动图层上方并设为活动图层。
- `duplicate_to_new(source, title, name)`：生成一个与 `source` 同尺寸、同分辨率、标题为 `title` 的新文档，里面只有活动图层的副本 `name`。
- `via_copy(doc)`：Layer › New › Layer Via Copy（⌘J）。
  - 没有选区：复制整个图层。背景图层的副本按 `next_layer_name` 命名为「Layer N」，普通图层的副本命名为「原名 copy」（规则同上）。
  - 有选区：把选区内的像素（规则同 Edit › Copy，见 `clipboard.md`）放到原位置的新图层「Layer N」上；选区保留（与 Paste 不同）。选区内只有透明像素时返回 `ClipError::Empty`。
  - 新图层沿用来源图层的不透明度、Fill 与混合模式（来源是背景图层时为默认值）。
- `via_cut(doc, background)`：Layer › New › Layer Via Cut（⇧⌘J）。像 Edit › Cut 一样剪下选区（背景图层上留下 `background` 颜色；隐藏或锁定像素的图层返回错误），再把像素放到原位置的新图层「Layer N」上；选区保留，属性规则同上。没有选区时作用于整个图层，但界面只在有选区时提供这个命令。

### 背景图层

- `layer_from_background(doc)`：把背景图层改为普通图层「Layer 0」，并解除三个锁定，之后可以有透明像素、可以移动和调整顺序。没有背景图层时返回 `false`。

### 顺序

- 图层列表自底向上，背景图层只能在最底层。
- `arrange_target(doc, arrange)`：Layer › Arrange 会把活动图层移到的索引；不会移动时为 `None`。Bring to Front 移到最上，Bring Forward 上移一层，Send Backward 下移一层，Send to Back 移到最下；有背景图层时「最下」是背景图层之上。背景图层本身不能移动。
- `arrange(doc, arrange)`：执行上述移动，返回是否移动。
- `move_layer(doc, from, to)`：把索引 `from` 的图层移到最终索引 `to`（Layers 面板拖动用）。`from == to`、越界、移动背景图层、或把图层放到背景图层下面时返回 `false` 且不改动。

### 改名

- `rename(doc, id, name)`：去掉首尾空白后改名。名字为空或与原名相同时返回 `false`。

### 合并

- `can_merge_down(doc)`：活动图层不是最底层，且它和下面一层都可见。
- `merge_down(doc)`：Layer › Merge Down（⌘E）。把活动图层按它的混合模式、不透明度和 Fill 合成到下面一层的像素上（下面一层的像素按 Normal、100% 参与合成），结果保留下面一层的名字和属性，活动图层被删除，下面一层成为活动图层。
- `can_merge_visible(doc)`：可见图层多于一个。
- `merge_visible(doc)`：Layer › Merge Visible（⇧⌘E）。把所有可见图层合成为一个图层：可见的背景图层存在时合并到背景图层，否则合并到活动图层（活动图层隐藏时为最上面的可见图层）。结果的混合模式重置为 Normal、不透明度和 Fill 为 100%（这些已经体现在合成结果里），成为活动图层；隐藏图层保持不变。
- `flatten(doc)`：Layer › Flatten Image。把可见图层合成后铺在白色上，得到唯一的不透明「Background」背景图层；隐藏图层被丢弃。
- `delete_hidden(doc)`：Layer › Delete › Hidden Layers。删除所有隐藏图层；没有隐藏图层或所有图层都隐藏时返回 `false` 且不改动。活动图层被删除时，最上面的图层成为活动图层。

合并都通过 `Document::composite_layers_rgba8` 计算，规则与文档合成一致（见 `document.md`）。

### 图层蒙版

- `NewMask`：`RevealAll`、`HideAll`、`RevealSelection`、`HideSelection`（Layer › Layer Mask 的前四项）。
- `can_add_mask(doc)`：当前图层存在、不是背景图层、还没有蒙版。
- `add_mask(doc, kind)`：给当前图层加蒙版——全白、全黑、选区（选择程度即蒙版值）或反选区；选区类需要有选区（选区保留）。之后编辑目标切换到蒙版（`mask_target = true`），与 Photoshop 一致。
- `delete_mask(doc)`：删除蒙版而不应用，编辑目标回到像素。
- `apply_mask(doc)`：启用的蒙版乘进图层的 alpha（四舍五入），然后删除蒙版；停用的蒙版直接删除。编辑目标回到像素。
- `toggle_mask(doc)`：停用/启用，返回新状态；没有蒙版时为 `None`。
- 合并：Merge Down 与 Merge Visible 的结果已经包含了各图层蒙版的效果，结果图层不再带蒙版；复制图层时蒙版随之复制。

## 已知限制

- 合并下面一层带非 Normal 混合模式或不透明度时，Photoshop 的结果与这里的近似（先按 Normal 100% 合成，再沿用下面一层的属性）可能不同。
- 没有图层组、多选，Merge Layers 只有 Merge Down 一种。
- 图层与画布同样大，合并结果也与画布同样大。

## 测试覆盖

- `grouping_ungrouping_and_moving_blocks`：⌘G 把不相邻的两层放进新组（位置、父子关系、选中）；组展开时新图层插在组内最上面；把组外的图层移到组内最上面；组不能移到自身里或背景下面；取消编组后结构复原并选中原子图层；删除组连同内容。
- `selected_layers_merge_delete_and_hide`：两个图层合并成上面那个，名字和位置正确、不透明像素叠加，未选的图层不动；隐藏/显示选中图层；删除选中图层后活动图层正确，最后一个图层不能删。
- `duplicate_layer_dialog_targets`：默认名「Background copy」；同文档改名复制；复制到更小的文档时位置不变、超出部分保留在画布外并成为活动图层；复制到新文档时尺寸、标题和唯一图层正确。
- `duplicates_are_named_like_photoshop`：「copy」「copy 2」命名；⌘J 作用于背景图层得到「Layer 2」这样的普通图层。
- `via_copy_and_cut_move_the_selection_in_place`：选区复制到「Layer N」并沿用不透明度、保留选区；剪切后原图层对应像素变透明。
- `arrange_keeps_the_background_at_the_bottom`：Send to Back 停在背景之上，背景不能上移。
- `dragging_and_renaming`：`move_layer` 的允许与拒绝情况；改名去空白、拒绝空名和同名。
- `masks_hide_reveal_and_apply`：背景图层不能加蒙版；Hide All 隐藏图层，停用后显示；在蒙版上填白重新显示、图层像素不变；应用 50% 灰的蒙版后 alpha 为 128；从选区建 Hide Selection 蒙版；删除蒙版。
- `merge_down_keeps_the_lower_layer`：50% 红色合并到白色背景得到 `[255, 128, 128, 255]`，结果仍是背景图层。
- `merge_visible_leaves_hidden_layers`：隐藏图层保留，可见图层合并到背景图层。
- `flatten_fills_transparency_with_white`：隐藏图层被丢弃，透明处变白。
- `delete_hidden_keeps_visible_layers`：删除隐藏图层并修正活动图层。
