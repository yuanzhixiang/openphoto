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

- `size`：早期按参考截图像素量取的固定区域尺寸（标题栏 40、选项栏 52、工具栏 58、文档标签栏 42、状态栏 26、图标列 58、面板列 478、面板标签栏 42、输入框高 26、图层行高 60）。
- `UI_SCALE = 0.675`：通过 egui `zoom_factor` 把参考截图像素缩放到 Photoshop 的实际大小。该值由「参考截图 2000 px 宽对应 Photoshop 窗口 1349 pt」算出。
- `pt(x)`：把在 Photoshop 里按 1:1 量到的点数换算成 egui 单位（`x / UI_SCALE`），新界面一律用它。

## 字体

- 正文 15、小字 13.5、图标 20（参考截图像素）。
- 比例字体族首选 Source Sans 3 Regular；另有名为 `semibold` 的字体族（Source Sans 3 Semibold），用于标签和标题。
- Phosphor 图标字体作为回退字体加在比例字体族末尾，所以图标可以直接写在文字里。

## egui 样式（`apply_style`）

- 固定使用深色主题，并设置 `zoom_factor = UI_SCALE`。
- 关闭 egui 自带的 ⌘+/⌘- 缩放整个界面的快捷键，这两个键留给画布缩放。
- 控件的底色、边框、圆角（3）、悬停和按下颜色都改成 Photoshop 的配色；悬停和按下时控件不放大。
