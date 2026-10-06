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
| `BUTTON` | `#636363` | 普通按钮（如「Select and Mask...」） |
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
- 对话框字体：Photoshop 的对话框使用 macOS 系统字体 SF。运行时从 `/System/Library/Fonts/SFNS.ttf` 读取（不随应用分发），按可变字体的 `wght` 轴注册三个字体族：`dialog`（400）、`dialog-medium`（510，比 Regular 略重，Photoshop 新式对话框如 New Layer 中文字的样子）和 `dialog-bold`（700），后面依次接界面字体作为回退。读不到时（非 macOS）只有回退字体，即 Source Sans 3。`dialog(size)`、`dialog_medium(size)`、`dialog_bold(size)` 返回对应字体。目前 New Layer 与 Duplicate Layer 对话框使用它们，其它对话框仍用 Source Sans 3。
- Phosphor Bold 单独注册为字体族 `phosphor-bold`，`tool_icon(size)` 返回它，只用于工具栏和工具列表的图标。

## egui 样式（`apply_style`）

- 固定使用深色主题，并设置 `zoom_factor = UI_SCALE`。
- 关闭 egui 自带的 ⌘+/⌘- 缩放整个界面的快捷键，这两个键留给画布缩放。
- 控件的底色、边框、圆角（3）、悬停和按下颜色都改成 Photoshop 的配色；悬停和按下时控件不放大。
