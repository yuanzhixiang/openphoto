# free_transform.rs：自由变换的画布交互

## 组件职责

Edit › Free Transform（⌘T）进行中的画布交互：显示变换框、拖动移动/缩放/旋转、确认或取消。像素变换见 `op-core` 的 `transform.md`。

## 状态

会话保存在 `DocState::free_transform`（`FreeTransform`）：开始时的文档快照 `before`、原始范围 `bounds`、平移 `offset`、缩放 `scale`、角度 `angle`、斜切 `skew`（水平与竖直，弧度）、参考点 `reference` 与是否显示 `show_reference`、相对定位 `relative`、宽高链接 `linked`、插值 `interpolation`、自由四角 `quad`（斜切、扭曲、透视之后才有：左上、右上、右下、左下，文档像素）、模式 `mode`（`TransformMode`：Free、Skew、Distort、Perspective）、进行中的拖动 `drag`，以及文档当前显示的变换 `applied`（`Projective`）。框的映射 `mapping()`：有自由四角时为范围到四角的投影变换（`Projective::rect_to_quad`），否则为「平移到范围中心 + offset · 旋转 angle · 斜切 skew · 缩放 scale · 平移回范围中心」。预览用所选插值（`transform_with`）。

## 流程

- `start(state)`：用 `transform::bounds` 检查并取得范围，保存快照，开始会话。`start_in(state, mode)`：Edit › Transform › Scale、Rotate（都是 Free）、Skew、Distort、Perspective；已在变换中时只切换模式。失败时由调用方弹出提示（例如背景图层无选区时「Could not complete the Free Transform command because the layer is locked.」）。
- 进行中，每帧 `preview`：框的映射与 `applied` 不同时，先恢复快照，再对文档应用新的映射，文档实时显示结果。预览不记录历史。
- 确认（`commit`）：Enter、在框内双击，或选项栏的 ✓ 按钮。映射不是恒等时记录「Free Transform」（无论从哪个模式开始，Photoshop 2026 都记这个名字，实测），并把映射记为 `AppState::last_transform`（供 Transform › Again 使用）；恒等时视为取消。
- 取消（`cancel`）：Esc 或选项栏的 ⦸ 按钮，恢复快照。
- 有输入框获得键盘焦点时，Enter 和 Esc 不作用于变换。
- 会话期间 `AppState::transforming()` 为真，`modal_open()` 也为真：菜单命令与单键工具快捷键都不生效，与 Photoshop 一致。

## Transform Selection

Select › Transform Selection（`start_selection`，有选区时可用）：同样的变换框围住选区，但只移动选区的轮廓（`transform::transform_selection`），像素不动；确认记录「Transform Selection」，不更新 Transform Again。测试 `ui_tests::transform_selection_moves_only_the_outline`。

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
- 斜切、扭曲、透视（Photoshop 的修饰键；在 Skew、Distort、Perspective 模式下不按修饰键也是这样）：
  - ⌘ 拖角点：扭曲，只移动这个角；⌘ 拖边：这条边的两个角一起移动。
  - ⌘⇧ 拖边：斜切，这条边沿自身方向滑动；Skew 模式下拖角点只沿水平或竖直中较大的方向移动。
  - ⌘⌥⇧ 拖角点：透视，角点沿较大的方向移动，同一条边上的相邻角反向移动同样的距离。
  - 有了自由四角后：普通拖控制点继续按扭曲处理；框内拖动平移四角；框外拖动让四角绕它们的中点转动（Shift 15°）。
- 右键菜单（`context_menu`）：Free Transform、Scale、Rotate、Skew、Distort、Perspective（切换模式），置灰的 Warp、Content-Aware Scale、Puppet Warp，Rotate 180°、Rotate 90° Clockwise、Rotate 90° Counter Clockwise、Flip Horizontal、Flip Vertical（`turn_box`：转动或翻转框本身——参数框改角度或缩放符号，自由四角绕中点转动或翻转）。
- 光标：框内为移动光标；框外为旋转（`Alias`）光标；控制点上为按框角度换算的双向箭头。

## 外观

- 框：1 pt 蓝色（`#2c8be8`）细线。
- 控制点：7 pt 白色方块，深灰描边。
- 中心参考点：半径 4 pt 的圆加十字线。
- 参考点（半径 4 pt 的圆加十字线）只在选项栏的参考点复选框打开时显示，位置为所选参考点（默认中心）变换后的位置；Photoshop 2026 默认不显示。
- 选项栏（见 `options_bar.md`）：参考点、X、Y、W、H、角度、斜切、插值、取消与确认。

## 移动工具的变换控件

`controls_handle_at` 与 `draw_controls` 供移动工具的 Show Transform Controls 使用（见 `document_view.md`）：框和控制点的画法与自由变换共用 `draw_box`，没有中心参考点；`controls_handle_at` 判断指针是否抓住了某个控制点（与自由变换相同的 8 pt 抓取半径）。

## 已知限制

- 选项栏的数值只读，不能输入；没有参考点位置选择、插值方式选择。
- 没有变形（Warp 与 Split Warp）、Content-Aware Scale、Puppet Warp。鼠标的缩放与旋转仍以中心为轴（参考点只影响选项栏的数值修改与 X/Y）；参考点不能在画布上拖动。

## 测试覆盖

- `corner_scales_proportionally_from_the_opposite_corner`：角点拖动等比放大 2 倍时左上角不动；Shift 时只放大宽度。
- `side_handles_move_and_rotate`：边控制点只缩放一个方向且对边不动；移动的偏移；Shift 旋转吸附到 90°。
- `hit_testing_the_quad`：点是否在框内。
- `distort_skew_and_perspective`：⌘ 拖右下角只移动它，之后普通拖动继续扭曲；透视模式下右上角外拉、左上角内收；⌘⇧ 拖右边沿自身滑动；映射跟随四角。
- `turning_and_flipping_the_box`：右键菜单的旋转与翻转改变参数框的角度与缩放；自由四角绕中点转 180°。
- `ui_tests::transform_distort_from_the_menu`：Edit › Transform › Distort 后拖右下角，确认记录「Free Transform」，远角变红而左上角不动，Transform Again 可用。
- `options_bar_numbers_pivot_on_the_reference_point`：参考点在左上时 W 50% 保持左上角不动；绕中心转 90° 中心不动；45° 水平斜切让左上角左移。
- `ui_tests::transform_bar_takes_typed_numbers`：在 W 框输入 50 回车，W、H 都变为 50% 且仍在变换中；角度输入 90 后中心不动。
