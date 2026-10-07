# dialogs/adjust/legacy.rs: parts of the legacy filter dialogs

## Responsibilities

The parts Photoshop 2026's larger legacy filter dialogs share: Reduce Noise (`reduce_noise.md`), Smart Sharpen, Oil Paint and Shape Blur. They are methods on `AdjustDialog` (`adjust.md`) that the dialogs call.

## Parts

- **Preview** (`legacy_preview`, `preview_rect`): the document as filtered at the pane's zoom (`pane_ui`; dragging pans it), unframed, in each dialog's area:

  | Dialog | Preview area |
  | --- | --- |
  | Reduce Noise | 8–491 × 36–619 |
  | Smart Sharpen | 8–271 × 36–457 open, 8–271 × 36–235.5 folded (`legacy_area`) |
  | Oil Paint | 8–220 × 36–248 |
  | Shape Blur | 8–219 × 36–248 |

  Under the area are the magnifier zoom controls of the classic pane (`zoom_controls`), centered under it on the dialog's zoom line. `pane_px` is the area at two pixels a point.
- **OK and Cancel** (`legacy_buttons`): 63 × 29 pt pills at (x, 37) and (x, 72), 13 pt labels; OK is the default and is off while the settings don't make an effect.
- **Preview** checkbox (`legacy_preview_check`): a 12 pt box, the label 9.5 pt after.
- **Rows** (`legacy_row`, `Row`): the label right-aligned to `label_right` on the field's center line, the field (the setting's decimals; arrows step by its last place), an optional unit, and a 3 pt `#757575` track with the white pin hanging under it (`classic_track`), spreading values by the row's `scale` (linear in most; Smart Sharpen's Radius measured). Disabled, the label, field and unit dim and nothing changes; `focus` gives the field the keyboard focus when the dialog opens.
- **Outcome** (`legacy_outcome`): Cancel closes; OK or Enter (unless a field is being typed in) applies.
