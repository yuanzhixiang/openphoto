# layer_comps.rs: layer comps

## Responsibilities

A layer comp records the layers' visibility, position and appearance under a name, to be put back later (Window › Layer Comps, `panels/layer_comps.md` in `op-ui`).

## Public interface

- **`LayerState`** is what a comp keeps of one layer:
  - the layer id;
  - whether it is visible;
  - the top-left of its pixels (`content_bounds`; None when empty);
  - opacity, fill and blend mode.
- **`LayerComp`** holds a name, a comment, its "Apply To Layers" switches (`visibility`, `position`, `appearance`) and the layers' states.
- **`record(doc)`** returns every layer's state now.
- **`apply(doc, comp)`** puts back, on each recorded layer that still exists:
  - with `visibility`, whether it is visible;
  - with `appearance`, its opacity, fill and blend mode;
  - with `position`, its place: when its pixels' top-left differs, the pixels are moved there as a whole, keeping pixels outside the canvas as layers do. The background layer doesn't move.

  The document is marked dirty.

## Known limitations

- Position is the place of the layer's pixels, since layers have no offset of their own. Moving a layer with a pixel edit since the comp counts as a move.
- Layer styles and "Selection for Layers" aren't recorded. Comps are not saved in PSD files.

## Test coverage

- `records_and_applies`: a hidden, half-opaque, moved layer gets its visibility, opacity and place back.
