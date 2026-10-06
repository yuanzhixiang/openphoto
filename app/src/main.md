# 应用入口（app/src/main.rs）

## 职责

`openphoto` 二进制的 `main` 函数：初始化日志，收集命令行传入的文件路径，配置原生窗口，然后用 eframe 启动 `op_ui::OpenPhotoApp`。除此之外不包含任何业务逻辑。

## 行为规则

### 日志

- 使用 `env_logger`，默认过滤级别为 `warn`；设置了 `RUST_LOG` 环境变量时以环境变量为准。

### 命令行参数

- 程序名之后的**所有**参数都被当作要打开的文件路径（`args_os().skip(1)`），按原样转换为 `PathBuf`，保留非 UTF-8 路径，不解析任何选项或标志。
- 路径列表交给 `OpenPhotoApp::new(cc, files)`：
  - 列表为空时，启动后新建一个默认文档（`Untitled-N`，1920×1080，白色背景）。
  - 列表非空时，按顺序逐个打开；某个文件打开失败会记录 `error` 日志并弹出提示，不影响其它文件，也不会退回到新建空白文档。

### 窗口

- 窗口标题和 eframe 应用名都是 `OpenPhoto`。
- 初始内容区尺寸 1350×800 逻辑点，最小 960×600 逻辑点，启动时在屏幕居中（`centered: true`）。
- 启用文件拖放（`with_drag_and_drop(true)`）。
- 渲染器固定为 `eframe::Renderer::Wgpu`；画布依赖 egui-wgpu 的 paint callback，`OpenPhotoApp::new` 在拿不到 wgpu 渲染状态时会 panic。
- 仅在 macOS 上：启用全尺寸内容视图（`with_fullsize_content_view(true)`），隐藏系统标题栏和标题文字。内容延伸到窗口顶部，标题栏区域由应用自己绘制，颜色与选项栏一致。

### 退出

- `main` 返回 `eframe::Result`；eframe 启动失败（例如无法创建 wgpu 设备或窗口）时以错误退出。

## 边界情况

- 传入的路径不存在或格式不支持：由 `op-ui` 的 `open_paths` 处理，记录日志并显示提示；窗口照常打开，可能没有任何文档。
- 形如 `--foo` 的参数也会被当作文件路径尝试打开。
- 非 macOS 平台使用系统默认标题栏。

## 与其它模块的关系

- 依赖 `op-ui`（`OpenPhotoApp`）、`eframe`、`env_logger`；所有界面、文档和渲染逻辑都在 `op-ui` 及其下游 crate 中。
- `OpenPhotoApp::new` 负责安装字体与样式、调用 `op_render::install`、安装 macOS 原生菜单，并根据 `files` 新建或打开文档。
- eframe 的 `default-features` 被关闭，只启用 `wgpu`、`default_fonts`、`accesskit`（见 `spec/Cargo.md`），因此没有启用 eframe 的持久化，窗口大小和位置不会在多次启动之间保存。

## 已知限制

- 不支持任何命令行选项（如版本号、帮助），所有参数一律视为文件路径。
- 窗口尺寸和位置每次启动都恢复为默认值。
