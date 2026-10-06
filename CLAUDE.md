## 仓库布局

OpenPhoto 的源码和 spec 分属两个仓库：

- 源码仓库 `openphoto`（github.com/yuanzhixiang/openphoto）。
- spec 仓库 `openphoto-spec`（github.com/yuanzhixiang/openphoto-spec），在本地克隆到源码仓库根目录下的 `spec/`。

源码仓库通过 `.git/info/exclude` 在本地忽略 `spec/` 和 `CLAUDE.local.md`，所以两边互不干扰：

- spec 文档和本文件只提交到 `openphoto-spec`（在 `spec/` 目录里执行 git 命令），不要提交进 `openphoto`。
- 源码改动只提交到 `openphoto`。
- 一次改动同时涉及源码和 spec 时，两个仓库各提交一次。

源码仓库根目录的 `CLAUDE.local.md` 只有一行 `@spec/CLAUDE.md`，用来让 Claude Code 在源码仓库里自动加载本文件。在新机器上搭建时：

```bash
git clone git@github.com:yuanzhixiang/openphoto.git
cd openphoto
git clone git@github.com:yuanzhixiang/openphoto-spec.git spec
printf '/spec/\n/CLAUDE.local.md\n' >> .git/info/exclude
echo '@spec/CLAUDE.md' > CLAUDE.local.md
```

由于 spec 仓库挂载在 `spec/`，下文所说的 `spec/` 路径在本地可以直接使用：源码 `crates/op-core/src/tile.rs` 的 spec 是 `spec/crates/op-core/src/tile.md`。spec 仓库内部的路径则去掉 `spec/` 前缀。

## Spec 文档

项目文档统一写入 `spec/`，不再新增旧的编号式决策目录或旧的日志目录。`spec/` 是源码镜像的产品需求与行为规格文档，用于记录页面/组件 UI/UX、API 行为、业务规则、接口口径、权限约束、已知限制和非目标。

这里放的是规格，不是计划。它用现在时描述系统当前如何行为。打算做什么、路线图、还在权衡的方案都不属于这里——一份描述尚未实现之物的 spec，本身就是这份 spec 的 bug。

### 读源码时同步读 spec

**打开任何源码文件时，一并打开它对应的 spec。** 这条不限于修改场景：读代码回答问题、追查 bug、判断某处要不要改，同样适用。源码说的是代码做了什么，spec 说的是为什么这么做、什么被刻意省略、哪些约束是承重的。只读其一会得出自信而错误的结论——把 workaround 当冗余「简化」掉，或者去「修复」一个刻意为之的行为。

源码没有对应 spec 时，明确说出来，不要默认它就没有上下文。缺失的 spec 是待补的缺口，不是「这个文件不言自明」的证据。

### 目录结构

- 每个子项目维护自己的 `spec/`，路径镜像该项目内的源码相对路径。
- 文件级说明将源码扩展名替换成 `.md`：`src/foo/bar.ts` 对应 `spec/src/foo/bar.md`。
- 页面和 API route 对齐源码入口：`page.tsx` 对应 `page.md`，`route.ts` 对应 `route.md`。
- 组件级说明同样镜像组件源码路径，例如 `src/components/foo/bar.tsx` 对应 `spec/src/components/foo/bar.md`。
- 多个文件共同承担的能力、体验规范或系统规则，写到最近源码目录的 `README.md`。
- 全仓库或根目录文件的说明写到仓库根 `spec/`，同样镜像根目录路径，例如 `.prettierrc.mjs` 对应 `spec/.prettierrc.md`。

### 写作规则

- 文档必须使用中文编写，标题、正文、产品口径、交互说明、接口逻辑和限制说明都使用中文。
- 修改页面、组件、API、产品口径、业务规则或用户流程前，先阅读对应源码路径下的 spec。
- 与 AI 或团队讨论后确认的需求、交互、接口逻辑、业务规则和安全约束，更新到最相关的源码镜像 spec。
- 改动源码导致行为、接口口径或设计意图变化后，在同一次工作会话里更新对应 spec。源码与 spec 永远不允许互相矛盾；如果改动让某条既有表述失效，就地改正，不要留着过期内容。
- 移动或拆分源码文件时，它的 spec 在同一个提交里跟着移动或拆分。镜像路径失效的 spec 比没有 spec 更糟：agent 按路径约定去找、找不到，然后断定这里没有上下文。
- 前端 `page.md` 必须包含页面定位、用户流程、信息架构、主要交互、筛选/排序/分页口径、加载/空态/错误态、权限态、响应式行为、文案口径、可访问性和关键 UI/UX 约束。
- 前端组件 `.md` 必须包含组件职责、使用场景、props/数据输入、视觉状态、交互状态、键盘/hover/focus/disabled/loading 行为、边界状态，以及与页面或其它组件的组合关系。
- 后端 `route.md` 必须包含 API 功能定位、请求/响应、鉴权与租户边界、参数校验、核心处理流程、数据库读写、外部服务调用、副作用、错误码、过期/幂等/并发规则、安全约束和边界情况。
- 如果一个功能同时涉及前端和后端，前端交互与页面/组件 UI/UX 写到前端对应 spec，后端 API、数据流、详细逻辑和安全约束写到 API 或 service 对应 spec。
- 不再使用 `状态/决策/备选方案/影响` 作为 spec 主模板；历史内容统一放入「历史方案」「废弃口径」或「变更记录」章节，保留它对当前需求的解释价值。
