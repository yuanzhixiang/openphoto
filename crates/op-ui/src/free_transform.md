# free_transform.rs：自由变换的画布交互

## 组件职责

Edit › Free Transform（⌘T）进行中的画布交互：显示变换框、拖动移动/缩放/旋转、确认或取消。像素变换见 `op-core` 的 `transform.md`。

## 状态

会话保存在 `DocState::free_transform`（`FreeTransform`）：开始时的文档快照 `before`、原始范围 `bounds`、平移 `offset`、缩放 `scale`、角度 `angle`、进行中的拖动 `drag`，以及文档当前显示的变换 `applied`。框的映射为 `Affine::around(范围中心, scale, angle, offset)`。

## 流程

- `start(state)`：用 `transform::bounds` 检查并取得范围，保存快照，开始会话。失败时由调用方弹出提示（例如背景图层无选区时「Could not complete the Free Transform command because the layer is locked.」）。
- 进行中，每帧 `preview`：框的映射与 `applied` 不同时，先恢复快照，再对文档应用新的映射，文档实时显示结果。预览不记录历史。
- 确认（`commit`）：Enter、在框内双击，或选项栏的 ✓ 按钮。映射不是恒等时记录「Free Transform」，并把映射记为 `AppState::last_transform`（供 Transform › Again 使用）；恒等时视为取消。
- 取消（`cancel`）：Esc 或选项栏的 ⦸ 按钮，恢复快照。
- 会话期间 `AppState::transforming()` 为真，`modal_open()` 也为真：菜单命令与单键工具快捷键都不生效，与 Photoshop 一致。

## 交互（与 Photoshop 一致）

- 按下时判定抓住的部位（屏幕坐标）：距 8 个控制点（四角与四边中点）8 pt 以内为缩放；在框内为移动；框外为旋转。
- 移动：框随指针平移。
- 缩放：
  - 默认以对面的控制点为固定点；按住 ⌥ 以中心为固定点。
  - 角控制点默认等比缩放（指针位移投影到框的对角线方向）；按住 Shift 自由缩放。
  - 边控制点只改变一个方向的大小，另一方向的中心不动。
  - 缩放比例的绝对值至少为 1 / 原始范围的较长边（框不会缩成 0）；越过固定点时翻转。
  - 框旋转后，缩放在框自身的坐标轴上计算。
- 旋转：绕当前中心，角度随指针相对中心的角度变化；按住 Shift 吸附到 15° 的倍数。
- 光标：框内为移动光标；框外为旋转（`Alias`）光标；控制点上为按框角度换算的双向箭头。

## 外观

- 框：1 pt 蓝色（`#2c8be8`）细线。
- 控制点：7 pt 白色方块，深灰描边。
- 中心参考点：半径 4 pt 的圆加十字线。
- 选项栏（见 `options_bar.md`）显示 W、H 百分比与角度，右侧是取消与确认按钮。

## 已知限制

- 选项栏的数值只读，不能输入；没有参考点位置选择、插值方式选择。
- 没有右键菜单与斜切、扭曲、透视、变形；没有 ⌘ 拖动角点的自由扭曲。

## 测试覆盖

- `corner_scales_proportionally_from_the_opposite_corner`：角点拖动等比放大 2 倍时左上角不动；Shift 时只放大宽度。
- `side_handles_move_and_rotate`：边控制点只缩放一个方向且对边不动；移动的偏移；Shift 旋转吸附到 90°。
- `hit_testing_the_quad`：点是否在框内。
