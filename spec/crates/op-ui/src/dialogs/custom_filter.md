# dialogs/custom_filter.rs：Custom 滤镜对话框

## 组件职责

Filter › Other › Custom... 的对话框，按 Photoshop 2026 的经典对话框逐点重做：编辑 5 × 5 卷积核、Scale、Offset，载入/存储 Photoshop 的 .acf 文件。卷积算法见 `op-core` 的 `filter.md`（`Filter::Custom`）。窗口框、预览框与缩放控件由宿主 `adjust.rs` 画（`Custom::Kernel`），本模块只画网格、字段与按钮。

## 数据输入

- `Dialog { cells: [String; 25], scale, offset }`：字段里的文字，从上到下逐行。
- `Default`：Photoshop 第一次打开时的核——十字锐化（中心 5，上下左右 −1，其余为空），Scale 1，Offset 空。
- `filter()`：所有字段有效时得到 `Filter::Custom`，否则 `None`（OK、Save... 置灰，也不预览）。
  - 核：整数 −999–999，空字段按 0。
  - Scale：整数 1–9999，不能为空。
  - Offset：整数 −9999–9999，空按 0。
  - 小数、非数字都无效。
- `settings()` / `restore(values)`：27 个字符串（25 格、Scale、Offset），供宿主记住上次按 OK 时的设置（与其它滤镜对话框一样）；个数不符时忽略。

## 布局与视觉

尺寸 634 × 282 pt，坐标为从对话框左上角起算的 Photoshop 点：

- 左侧预览框与缩放控件同其它经典滤镜对话框（`filter_layout.md` 的 `PANE`、`ZOOM_Y`）。
- 网格：25 个 44 × 19 的输入框，列左缘 277、329、380.5、432.5、484.5，行上缘 78、105、132、159、186。
- 网格下方 y 213–232：「Scale:」右缘 325.5，输入框 330.5–374.5；「Offset:」右缘 429.5，输入框 434.5–478.5。
- 右侧按钮 x 543.5–623.5，高 26：OK（默认按钮）y 38.5、Cancel 73.5、Load... 143.5、Save... 178.5；Preview 复选框左上角 (543, 216.5)。

## 交互

- 打开时左上角的格子获得焦点并全选；Tab 按创建顺序（逐行、再 Scale、Offset）移动焦点；输入框支持 ↑↓（⇧ 时 ±10）。
- OK 或 Enter（有效时）应用；Cancel 或 Esc 取消。
- Load...：系统打开对话框选 .acf 文件，读入前 54 字节：27 个大端 16 位整数（25 格、Scale、Offset）。为 0 的格子与 Offset 显示为空，Scale 至少为 1。不足 54 字节的文件忽略。
- Save...：系统存储对话框（默认名 Untitled.acf），按同样格式写出。写入失败时静默忽略。

## 已知限制

- Load/Save 失败时没有提示（Photoshop 会弹出错误）。

## 测试覆盖

- `opens_with_photoshops_sharpening_cross`：默认核与 Scale/Offset。
- `fields_must_be_integers_in_range`：越界、小数、Scale 为空或 0 时无效，边界值有效。
- `acf_files_round_trip`：写出 54 字节、大端，读回得到相同的滤镜，0 读回为空，过短的文件被拒绝。
- `settings_round_trip`：记住的设置放回后得到相同的滤镜。
- `ui_tests.rs` 的 `custom_filter_types_a_kernel_and_remembers_it`：从菜单打开，在左上角格子输入 2，Enter 应用并记录「Custom」，再次打开时保留该核；`screenshot_filter_dialogs` 截取此对话框用于与 Photoshop 比对。
