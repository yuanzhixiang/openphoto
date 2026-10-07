# panels/layer_comps.rs: Layer Comps panel

## Responsibilities

The Layer Comps panel (Window › Layer Comps) is a floating panel (`floating.md`) holding the active document's layer comps (`op_core::layer_comps`, `DocState::layer_comps`).

## Layout and interactions

- **List:** "Last Document State" first, then the comps by name. A comp's comment is its tooltip. The comp applied last (`DocState::comp_applied`), or Last Document State, is highlighted.
- **Applying** (`apply`): clicking a comp applies it and records "Apply Layer Comp".
  - When the first comp is applied, the layers as they were are kept as Last Document State (`DocState::last_document_state`).
  - Clicking Last Document State puts those layers back (all of visibility, position and appearance) and records "Apply Layer Comp".
- **Buttons below the list:**
  - ◀ and ▶ apply the previous or next comp, wrapping around.
  - Update records the layers again into the comp applied last.
  - New... opens New Layer Comp.
  - Delete removes the comp applied last.
- **New Layer Comp** (`new_comp_dialog`, `AppState::new_layer_comp`, a modal) has these fields:
  - Name ("Layer Comp N");
  - Apply To Layers: Visibility, Position and Appearance (Layer Style), all on;
  - Comment.

  OK (or Enter while no field is being typed in) records the layers into a new comp, which becomes the current one. Cancel or Escape drops it.

## Known limitations

- The panel and dialog use egui's widgets.
- The comps last for the session and are not part of the History: undo doesn't remove one.
- There is no "Selection for Layers" switch and no warning icon for comps that no longer match the layers.

## Test coverage

`ui_tests::layer_comps_record_and_apply`:

- New... records "Layer Comp 1";
- applying it shows a layer hidden since, recorded as "Apply Layer Comp";
- Last Document State hides it again;
- screenshot `layer_comps_panel.png`.
