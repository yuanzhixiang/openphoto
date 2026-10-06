# link.rs：链接图层

## 职责

Layer › Link Layers / Unlink Layers / Select Linked Layers 的规则。链接在一起的图层跟随移动工具一起移动（`move_tool.md`），在 Layers 面板上显示链接图标（`op-ui` 的 `panels/layers.md`），存入 PSD 的图像资源 1026（`op-io` 的 `psd.md`）。

## 数据

`Layer::link: Option<u32>` 是链接编号，编号相同的图层链接在一起。没有别的图层同号的编号等于没有链接，所以删除图层、取消链接后不需要整理残留的编号。

## 规则（Photoshop 2026 实测）

- `linked_with(doc, id)`：与 `id` 链接的其它图层。`is_linked`：有没有。
- `with_linked(doc)`：选中的图层，加上与任何一个选中图层链接的图层（自下而上）。面板在其中已链接的图层上显示链接图标；移动工具移动的就是它们。
- `can_unlink(doc)`：每个选中的图层都已链接（链接到谁都可以，不必是同一组）。此时菜单项和面板按钮的作用是「Unlink Layers」。
- `can_link(doc)`：选中两个以上图层，且不满足 `can_unlink`。
- `link_selected(doc)`：选中的图层与已经和它们链接的图层合成一组（用一个新编号）。例如 A、C 已链接，再选中 B、A 链接，则 A、B、C 三者链接。
- `unlink_selected(doc)`：只把选中的图层移出各自的链接组；组里剩下的图层仍然相互链接（例如 A、B、C 链接，只选 C 取消链接，A 与 B 仍然链接）。
- `toggle(doc)`：可以取消链接时取消，否则链接；返回历史名「Unlink Layers」或「Link Layers」，都不行时为 `None`。
- `can_select_linked` / `select_linked`：Select Linked Layers 只在会多选中图层时可用（选中的已包括全部链接图层时置灰）；执行后活动图层不变。不记录历史。
- 背景图层也可以参与链接。
- 复制图层时副本保留编号（与原图层的链接组链接）；复制到其它文档时清除（`layer_ops.md`）。

## 测试

- `matches_photoshop`：按在 Photoshop 2026 中走过的顺序（单层不能链接；链接 A、C；B 单独选中时不显示链接；B、A 链接把 C 带入；C 单独取消链接；C 与背景链接；A 已链接 D 未链接时为 Link；分属两组的 A、C 为 Unlink；Select Linked Layers 的可用条件与结果）逐步核对。
- `deleting_a_linked_layer_leaves_no_stale_link`：删掉两个链接图层中的一个，另一个不再算链接。

## 已知限制

- 自由变换、对齐/分布还不带上链接的图层；⇧ 单击链接图标临时停用链接未实现。
