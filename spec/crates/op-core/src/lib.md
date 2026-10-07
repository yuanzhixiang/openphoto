# lib.rs

## 职责

`op-core` 的 crate 根。声明 `color`、`document`、`history`、`layer`、`pixel`、`tile` 六个公开模块，并把常用类型重导出到 crate 根，让下游 crate 可以直接写 `op_core::Document`、`op_core::TiledImage` 等。

crate 级文档注释说明了本 crate 的定位：文档模型（图层树、分块像素存储、像素格式），不依赖 GPU 或 UI，所有编辑最终都是对这里数据结构的修改。整体模型见同目录 `README.md`。

## 对外接口

根上重导出的类型：

- `color::Color`
- `document::{Anchor, DocId, Document, Snapshot}`
- `history::History`
- `layer::{BlendMode, Layer, LayerId, LayerKind}`
- `pixel::{BitDepth, ColorMode}`
- `tile::{TILE_SIZE, Tile, TiledImage}`

未在根上重导出、需走模块路径的公开项：`history::HistoryState`、`history::DEFAULT_LIMIT`。

## 与其它模块的关系

- 被 `op-color`、`op-io`、`op-tools`、`op-ui` 依赖。`op-render` 不依赖 `op-core`，它只接收 `op-ui` 传来的 RGBA 像素。
- 本 crate 不依赖任何外部 crate。

## 测试覆盖

本文件没有测试；各模块的测试见对应 spec。
