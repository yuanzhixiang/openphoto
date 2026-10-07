# theme.rs：视觉规范

## 职责

集中定义颜色、尺寸、字体和 egui 样式，数值取自 Photoshop 默认的「中灰」界面。尺寸体系与换算规则见目录的 `README.md`。

## 颜色（`color`）

| 名称 | 值 | 用途 |
|---|---|---|
| `PANEL` | `#535353` | 面板、工具栏、选项栏底色；激活标签 |
| `PASTEBOARD` | `#282828` | 画布外的粘贴板 |
| `TAB_BAR` | `#424242` | 标签栏 |
| `TAB_INACTIVE` | `#4c4c4c` | 文档标签悬停 |
| `FIELD` / `FIELD_BORDER` | `#454545` / `#3a3a3a` | 输入框 |
| `TOOL_ACTIVE` | `#383838` | 当前工具、按下状态 |
| `HOVER` | `#5e5e5e` | 悬停 |
| `LIST_BG` / `ROW_SELECTED` | `#4d4d4d` / `#6b6b6b` | 列表底色、选中行 |
| `SEPARATOR` / `SEPARATOR_LIGHT` | `#393939` / `#444444` | 深色、浅色分隔线 |
| `TEXT` / `TEXT_DIM` / `TEXT_DISABLED` | `#eaeaea` / `#bcbcbc` / `#7c7c7c` | 正文、次要文字、禁用文字 |
| `ICON` | `#d8d8d8` | 图标 |
| `ACCENT` | `#2c8be8` | 文字选区等强调色 |

## 尺寸

- `size`：窗口框架尺寸按 Photoshop 1:1 量取（pt）：标题栏 29、选项栏 33、工具栏 42、状态栏 16、图标列 43、面板列 322、面板标签栏 28（含下方 1 pt 线）。其余（输入框高 26、图层行高 60）仍是按参考截图像素量取的值。
- `UI_SCALE = 0.675`：通过 egui `zoom_factor` 把参考截图像素缩放到 Photoshop 的实际大小。该值由「参考截图 2000 px 宽对应 Photoshop 窗口 1349 pt」算出。
- `pt(x)`：把在 Photoshop 里按 1:1 量到的点数换算成 egui 单位（`x / UI_SCALE`），新界面一律用它。

## 字体

- 正文 11.5 pt、小字 10.5 pt，使大写字母高度与 Photoshop 面板文字一致（8 pt）。Source Sans 3 比 Photoshop 的 Adobe Clean 窄，同样的单词宽度约小 15%。图标 20（参考截图像素）。
- 字体文件随应用打包，字节数据以 `SOURCE_SANS_REGULAR`、`SOURCE_SANS_SEMIBOLD` 公开，文字工具也用它们栅格化文字。
- 比例字体族首选 Source Sans 3 Regular；另有名为 `semibold` 的字体族（Source Sans 3 Semibold），用于标签和标题。
- Phosphor 图标字体作为回退字体加在比例字体族末尾，所以图标可以直接写在文字里。
- 分隔与折叠条颜色（Photoshop 2026 实测）：3 pt 分隔条和折叠条的线 `DIVIDER_DARK` `#383838`、分隔条中间的线 `DIVIDER_LIGHT` `#474747`、折叠条底色 `COLLAPSE_BAR` `#424242`、折叠箭头 `COLLAPSE_CHEVRON` `#c8c8c8`；面板标签文字 `TAB_TEXT_ACTIVE` `#f0f0f0`、`TAB_TEXT` `#b0b0b0`。
- 选项栏颜色（Photoshop 2026 实测）：底色 `OPTIONS_BAR` `#535353`、分隔线 `OPTIONS_SEPARATOR` `#3e3e3e`、图标 `OPTIONS_ICON` `#dddddd`、铃铛 `OPTIONS_BELL` `#b9b9b9`、禁用图标 `OPTIONS_ICON_DISABLED` `#989898`、标签文字 `TEXT_BRIGHT` `#f0f0f0`；复选框 `CHECKBOX` `#d4d4d4` 与对勾 `CHECK_MARK` `#323232`；下拉框边框 `DROPDOWN_BORDER` `#666666`（悬停 `DROPDOWN_BORDER_HOVER` `#808080`）。
- 对话框字体：Photoshop 2026 的对话框文字有两类，必须按对话框选对（用文字宽度与字形在 Photoshop 截图上逐一核对过）：
  - **AppKit 绘制的**（经典对话框 Duplicate Layer、Image Size，macOS 提示框，所有窗口标题）：macOS 系统字体 SF。运行时从 `/System/Library/Fonts/SFNS.ttf` 读取（不随应用分发），按可变字体注册 `dialog`（wght 400）和 `dialog-bold`（700），两者都设 `opsz` = 17：AppKit 给 17 pt 以下的文字用 Text 光学尺寸，比字体默认的 Display（28）宽约 10%，不设就会窄。后面依次接界面字体作为回退；读不到时（非 macOS）只有 Source Sans 3。正文 12 pt（`dialog(pt(12.0))`），标题 13 pt 粗体，提示框 13 pt。
  - **UXP（Spectrum）对话框**（New Layer、New Group、Layer from Background）：Spectrum 的 Adobe Clean（t 的斜切顶、y 的直尾可以分辨），与面板同一种字体，所以用 Source Sans 3：`uxp(size)` 为 Regular、`uxp_bold(size)` 为 Semibold，都是 12 pt。
- 字距：AppKit 按 SF 的 `trak` 表随字号调整字距（12 pt 为 0，13 pt 为 −12/2048 em，14 pt 为 −22），egui 不会。`system_tracking(size)` 按表插值给出 egui 的额外字距，`tracked_galley(painter, text, font, color)` 用它排版；窗口标题和提示框的文字都这样画（12 pt 正文不需要）。
- 测试 `dialog_text_is_as_wide_as_photoshops`：九段文字（Image Size、Duplicate Layer、窗口标题、提示框、New Layer 的）在 2x 下的宽度与 Photoshop 实测相差不超过 3%；去掉 opsz 17、去掉字距或给 UXP 对话框用错字体都会失败（已验证）。
- Phosphor Regular 另外单独注册为字体族 `phosphor-regular`（前面没有文字字体），`tool_icon(size)` 返回它，只用于工具栏和工具列表的图标：Regular 在工具栏尺寸下笔画约 1 pt，与 Photoshop 的图标一致；之前用的 Bold 粗了约一半。图标字形本身仍与 Photoshop 不同（见 README 的 P0 差距）。

## egui 样式（`apply_style`）

- 固定使用深色主题，并设置 `zoom_factor = UI_SCALE`。
- 关闭 egui 自带的 ⌘+/⌘- 缩放整个界面的快捷键，这两个键留给画布缩放。
- 控件的底色、边框、圆角（3）、悬停和按下颜色都改成 Photoshop 的配色；悬停和按下时控件不放大。
- 关闭滚动区边缘的淡出（egui 0.36 默认在可继续滚动的一侧画 20 pt 的渐变，Photoshop 的列表没有）。自定义滚动样式的面板（History）也要单独关掉，因为 `ScrollStyle::solid()` 自带默认淡出。
