# dialogs/equalize.rs：Equalize 询问框

## 组件职责

有选区时执行 Image › Adjustments › Equalize... 弹出的经典对话框（409 × 127 pt，按 Photoshop 2026 重做）：询问只均衡选区，还是按选区的直方图均衡整幅图像。

## 布局与交互

- 分组框「Options」(11, 46.5)–(322, 116.5)；单选「Equalize selected area only」(27, 70.5)、「Equalize entire image based on selected area」(27, 97)，默认选中后者（Photoshop 的默认）。
- 按钮 x 339.5–398.5：OK (45)、Cancel (80)。Enter = OK，Esc = Cancel。
- OK 返回 `Outcome::Apply { entire_image }`：`lib.rs` 执行 `Adjustment::Equalize`（只处理选区）或 `Adjustment::EqualizeEntireImage`（按选区的直方图处理整个图层），历史名称都是「Equalize」。

## 测试覆盖

- `ui_tests::equalize_asks_about_the_selection`：没有选区时直接执行；有选区时弹出询问、默认整幅图像、Enter 应用。截图 `equalize_dialog.png`。
