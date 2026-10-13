# Photoshop 2026 toolbar parity implementation plan

Reference: the user's third screenshot and installed Photoshop 2026 27.8.0.
Scope includes every flyout member, options bar, keyboard interaction, document
and layer effects, history, and persistence needed by the left toolbar. The
screenshot's first slot currently displays Artboard; Move precedes it in that
slot's flyout. Slot memory is respected rather than changing the default tool
to match a previously used screenshot state.

This is a work plan, not a specification of completed product behavior. Current
behavior belongs in the mirrored English documents under `spec/`. Existing
implementations are audited rather than assumed to meet full parity.

## Execution and acceptance

Proceed in table order, and within each row from left to right. Each individual
tool receives its own code/spec/test commit and push after acceptance. Environment
and planning have a separate initial commit. Do not skip a difficult tool and
mark later work as completing this ordered sequence.

For each tool:

1. Observe the real Photoshop tool and flyout, including names, shortcuts,
   defaults, option ranges, modifier keys, cursor, drag/click/release, Enter/Escape,
   error states, selection, locked/background/hidden/group layers and undo/redo.
2. Capture deterministic disposable reference documents and exported results.
   Record the reference version, input, settings and measurement method. Use
   focused REA analysis when source-free behavior requires it. Do not claim
   pseudocode or a visual approximation reproduces an unmeasured algorithm.
3. Audit existing source together with its mirrored specification; implement
   model, renderer, UI, options, keyboard and file format changes together.
4. Compare image output to Photoshop fixtures; test meaningful edge cases and
   interaction/history. Record measured error rather than assuming zero error.
   Compare native UI at the same display scale and document zoom.
5. Run formatting, relevant tests, workspace tests and clippy. Stop owned test
   app processes. Commit only this tool and its specs/fixtures; push normally,
   never force-push. Store validation and commit evidence below.

Exact proprietary AI results (Object Selection, Remove, content-aware algorithms)
need measured reference coverage and a reproducible implementation or a separately
authorized service/model. A fallback algorithm is not full 1:1 acceptance.

## Ordered tool inventory

| Slot | Tools in execution order | Main work and acceptance focus |
| --- | --- | --- |
| 1 | Move; Artboard | Move constraints, auto-select layer/group, multi-select, duplicate-drag, snapping and transform controls; artboard model, presets, resize, child layers, rendering, export and PSD round-trip |
| 2 | Rectangular Marquee; Elliptical Marquee; Single Row Marquee; Single Column Marquee | Selection operations, feather, anti-alias, fixed ratio/size, modifiers and 1-pixel rows/columns |
| 3 | Selection Brush; Lasso; Polygonal Lasso; Magnetic Lasso | Overlay/selection conversion, freehand and polygon completion, magnetic edge tracking and anchors |
| 4 | Object Selection; Quick Selection; Magic Wand | Rectangle/lasso object recognition, brush refinement, sample sources, contiguous/tolerance/anti-alias and subtract/intersect |
| 5 | Crop; Perspective Crop; Slice; Slice Select | Non-destructive crop and straighten, resampling and perspective mapping, slice metadata/selection/export |
| 6 | Frame | Frame layer/container model, rectangle/ellipse, placed content clipping and content transforms |
| 7 | Eyedropper; Color Sampler; Ruler; Note; Count | Sample radius/source, ring and colors; persistent samples, measurements, notes and count groups |
| 8 | Spot Healing Brush; Remove; Healing Brush; Patch; Content-Aware Move; Red Eye | Healing source and diffusion, model-backed removal parity, patch source/destination, content-aware filling/structure/color, pupil correction |
| 9 | Brush; Pencil; Color Replacement; Mixer Brush | Tip, spacing, pressure, flow, blend, smoothing, auto-erase, color matching and wet/load/mix state |
| 10 | Clone Stamp; Pattern Stamp | Source alignment/transforms/sample layers, pattern origin, impressionist and overlays |
| 11 | History Brush; Art History Brush | Source snapshots, incompatible states, modes, styles/area/tolerance and strokes |
| 12 | Eraser; Background Eraser; Magic Eraser | Brush/pencil/block, transparency locks, color sampling/limits/protection, tolerance and contiguous erase |
| 13 | Gradient; Paint Bucket | Classic/live gradients, interpolation/dither/styles, fill layers and editable controls; pattern/tolerance/contiguous/sample layers |
| 14 | Blur; Sharpen; Smudge | Strength, sampling, protect detail, finger painting, modes and stroke output |
| 15 | Adjustment Brush | Adjustment-layer/mask model, adjustment choice, add/subtract, subject selection and editability |
| 16 | Dodge; Burn; Sponge | Range/exposure/protect tones, saturation/vibrance and measured tone output |
| 17 | Pen; Freeform Pen; Curvature Pen; Add Anchor Point; Delete Anchor Point; Convert Point | Persistent paths, Bézier topology, handles, snapping, rubber band, magnetic freeform and path/shape operations |
| 18 | Horizontal Type; Vertical Type; Vertical Type Mask; Horizontal Type Mask | Editable typography, text metrics and font fallback, point/paragraph text, orientation, glyph masks and PSD preservation |
| 19 | Path Selection; Direct Selection | Whole-path versus node/handle picking, modifier multi-selection, transforms, alignment and arrangement |
| 20 | Rectangle; Ellipse; Triangle; Polygon; Line; Custom Shape | Editable vector geometry, fill/stroke, corners, sides, line arrows, custom presets, path versus shape versus pixels |
| 21 | Hand; Rotate View | Temporary hand, scrolling all documents, zoom buttons, rotation compass/snap/reset and coordinate mapping |
| 22 | Zoom | Scrubby/marquee zoom, click/option-click, presets, pointer anchoring, all windows and actual/print size |

There are 70 tools in these 22 groups. After Zoom, audit the remaining toolbar
controls in screenshot order: Edit Toolbar / extra tools, default colors, swap
colors, foreground/background picker, and (when visible) Quick Mask and Screen
Mode. The screenshot is vertically clipped, so its bottom edge does not establish
that those additional controls are absent.

## Dependencies and architectural work

- Artboard and Frame require semantic document containers and PSD metadata;
  flattening to an ordinary raster layer does not meet acceptance.
- Pen, paths, vector shapes, type and adjustment brushes require editable model
  data preserved by history and PSD, not just raster previews.
- AI/content-aware tools require independent algorithm/output evidence. Their UI
  alone does not establish implementation or acceptance.
- Every currently implemented tool still needs the per-tool audit. Existing
  tests provide a baseline, not a blanket Photoshop parity certificate.

## Progress and evidence

| Stage | State | Evidence |
| --- | --- | --- |
| Environment and plan | Verified | REA 6.4.0 setup reports `ready`; Hopper scoped doctor healthy; Rust 1.99.0; baseline workspace tests: 547 passed, 0 failed, 51 ignored; bounded runner verified for success and timeout |
| Move | Audit in progress | Existing core/UI implementation; checking missing modifier behavior against Photoshop |
| Artboard through Zoom and auxiliary controls | Pending | Tool definitions/options exist to varying degrees; per-tool acceptance has not run |
