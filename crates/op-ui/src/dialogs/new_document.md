# dialogs/new_document.rs：File › New 的 New Document 对话框

## 组件职责

File › New...（⌘N）：Photoshop 2026 默认的新版 New Document 对话框。左边是按类别分页的空白文档预设（以及最近使用和存储的预设），右边是所选预设的详情，可以修改后创建。确定后由 `actions::create_document` 创建（见 `actions.md`「新建」），并由 `lib.rs` 把这次的设置放进 Recent 列表。预设数据与卡片图标见 `document_presets.md`。

## 布局

按 Photoshop 2026 的 2x 截图逐点量取，对话框 1080 × 718 pt，坐标为距对话框左上角的 pt：

- **窗口**：10 pt 圆角，带阴影。标题栏 0–28（`#d3d4d5`，下边一条 0.5 pt `#0c0c0c` 线），居中系统粗体 13 pt「New Document」，左上角三个交通灯（中心 x 14、34、54，y 14，半径 6：红、灰（不可用的最小化）、绿）。交通灯只有外观。
- **标签栏** 28.5–76（`#323232`，底部 74–76 一条 `#3e3e3e`）：Recent（钟表图标中心 35，文字从 49.5）、Saved (112.5)、Photo (172)、Print (230)、Art & Illustration (280)、Web (398)、Mobile (448.5)、Film & Video (513)，14 pt，中心 y 51。当前页白字并在下面画 2 pt 白线（Recent 从钟表左缘 27 起），其它页 `#a8a8a8`，悬停变白。
- **左侧列表**（0–770，y 76 起，`#252525`）：可滚动，下方 655–703 是模板搜索栏。
  - 标题（粗体 13 pt）「BLANK DOCUMENT PRESETS」/「YOUR RECENT ITEMS」/「YOUR SAVED PRESETS」，后接「(数量)」，左端 26.5，中心 y 103。
  - 卡片 168 pt 见方，每行 4 张，相距 180.9 pt，第一张左上角 (26, 124)。图标中心在卡片顶下 62；名称（12 pt 白字，过长截断加「...」）中心在顶下 121.5；尺寸行（如「7 x 5 in @ 300 ppi」，12 pt `#c8c8c8`）在顶下 140。选中的卡片画 3 pt `#417ee4` 边框；悬停的卡片底色 `#323232`、名称变蓝。
  - **Recent 页**：欢迎框 (13, 89)–(741, 262)，`#2e2e2e`，右上 (720, 110) 有关闭的叉；粗体 28 pt「Let’s start something new.」中心 (377, 147)；下面两行 16 pt 文字，「document presets」是带下划线的蓝色链接，点击切到 Photo 页。关闭欢迎框后不再出现（本次运行内），标题与卡片上移到与其它页相同的位置；没关时标题在 y 286、卡片从 y 307 起。
  - **Saved 页**：没有存储的预设时显示同样的框，标题「You can always find it here.」与说明文字。
  - **搜索栏**：放大镜、斜体灰字「Search for an image or template」、下划线和描边的「Search」胶囊（656.5–727，y 664–694.5）。Photoshop 在这里搜索 Adobe Stock，本应用没有模板服务，所以只有外观。文案去掉了 Photoshop 原文末尾的「from Adobe Stock」：界面不出现 Adobe 的品牌名（`SEARCH_PLACEHOLDER`，测试 `search_shows_no_adobe_brand` 守住这一点）。
- **右侧详情**（770–1080，`#323232`）：标签 12 pt `#c3c3c3`，值 14–15 pt `#e3e3e3`。
  - 「PRESET DETAILS」(791, 100.75)；名称输入框（下划线 790–1015，y 149，`#909090`）；右侧存储预设图标（托盘加向下箭头，1034–1063 × 118–144，悬停提示「Save Preset」）。
  - Width (791, 168.5)：输入框 (790–862, 180.5–212.5，`#252525` 底、1 pt `#4a4a4a` 边、4 pt 圆角，聚焦时边框变蓝，值左缩进 15.5)；单位下拉 (874–1060)：Pixels、Inches、Centimeters、Millimeters、Points、Picas。
  - Height (791, 232)、Orientation (874.5, 231)、Artboards (944, 231)：高度输入框 (790–862, 243.5–275.5)；纵向、横向两个方向图标 (875–897 × 247–272、905.5–930.5 × 250–269.5)，当前方向的那个蓝底；点另一个时宽高互换。Artboards 复选框 (944–958 × 252–266) 不可用（没有画板）。
  - Resolution (791, 292)：输入框 (790–862, 304–336)；Pixels/Inch / Pixels/Centimeter 下拉 (874–1060)。
  - Color Mode (791, 354.5)：模式下拉 (790–957.5, 367–399，只有 RGB Color 可选) 与位深下拉 (969.5–1060，只有 8 bit 可选)。
  - Background Contents (791, 415.5)：下拉 (790–1016, 428–460)：White、Black、Background Color、Transparent（Custom... 置灰）；右侧色块 (1028–1060)：白、黑、当前背景色或棋盘格。
  - Advanced Options（V 形箭头 + 文字，中心 y 478.75，默认展开，点击折叠）：Color Profile (507.5) 下拉 (519.5–551.5)：Working RGB: sRGB IEC61966-2.1（Don't Color Manage 置灰）；Pixel Aspect Ratio (568) 下拉 (580.5–612.5)：Square Pixels。
  - Close (900–972, 658–690，2 pt `#e3e3e3` 边的胶囊) 与 Create (988–1060，`#3671df` 蓝色胶囊)，都是粗体 15 pt 白字。宽高无效时 Create 置灰。

与 Photoshop 截图并排比对，各元素位置一致；文字因 Source Sans 3 近似 Adobe Clean 略有差异。

## 交互

- 打开时在 Recent 页。初始详情：剪贴板里有图像时为它的尺寸（像素、72 ppi）；否则为 Recent 的第一项（并选中它的卡片）；都没有时 1920 × 1080 像素、72 ppi。名称为下一个「Untitled-N」，背景 White。
- 切换标签页时与 Photoshop 一样选中并应用该页的第一个预设（空页不变）。单击卡片：选中并把它的宽、高、单位、分辨率填入右侧（名称不变）。双击卡片：直接以该预设创建。
- 改变单位时，宽高换算成新单位，表示的像素数不变（像素取整，其它单位最多 3 位小数）。改变分辨率单位时数值随之换算。
- 存储图标：把当前设置以名称为名存进 Saved 页（本次运行内保留，由 `lib.rs` 收进 `AppState::new_document_saved`）。
- Create 或 Enter：创建；宽高换算成像素后取整，必须在 1–30000 之间，分辨率 1–10000 ppi，否则不创建。名称为空时用「Untitled」。Close 或 Esc：取消。
- 创建后这次的设置（名称为「Custom」）放进 Recent 列表最前面，相同的设置不重复，最多 20 个。

## 已知限制

- Recent、Saved 只在本次运行内保留，Photoshop 会跨次启动保存。
- 没有 Adobe Stock 模板与搜索；卡片图标是按预设形状画的近似图形，不是 Photoshop 的图标资源。
- 颜色模式、位深、颜色配置文件、像素长宽比只有一个可选项；没有画板；Background Contents 没有 Custom。
- Print 页 Photoshop 有 14 个预设、Mobile 28 个、Film & Video 25 个，这里列出截图里能看到的（12、24、24 个）。

## 测试覆盖

- `defaults_units_and_validation`：默认 1920 × 1080、按剪贴板尺寸、改为英寸后数值为 8.889 而像素不变、应用 7 × 5 in @ 300 ppi 预设为 2100 × 1500、宽为 0 无效、有最近使用时从它开始并选中。
- `search_shows_no_adobe_brand`：搜索栏占位文案不含「Adobe」与「Stock」。
- `ui_tests.rs` 的 `new_document_dialog`（⌘N 后 Enter 创建 Untitled-2，1920 × 1080；透明背景得到「Layer 1」）与 `new_document_dialog_presets_recent_and_saved`。
