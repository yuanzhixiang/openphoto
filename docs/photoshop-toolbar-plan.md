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
| Move | Offline implementation validated; live acceptance pending | Cross-document tab/drop, mask linkage, groups, constraints, copy/cancel, hover and alignment validated; pushed commit 0467f1d |
| Artboard | Offline implementation validated; live acceptance pending | Semantic containers, canvas gestures, options, clipping, nesting, export and standard PSD descriptors implemented; offline evidence below |
| Rectangular Marquee through Zoom and auxiliary controls | Pending | Tool definitions/options exist to varying degrees; per-tool acceptance has not run |

### Move audit boundary

The repaired Group/Layer behavior follows Adobe's
[Move, stack, and lock layers](https://helpx.adobe.com/photoshop/using/moving-stacking-locking-layers.html)
and the dropdown observed in installed Photoshop 27.8.0. Nested-group selection
uses the outermost parent; it has a deterministic model regression test, but
that nested case has not yet been compared by clicking the running Photoshop.

Full Move acceptance still needs: Shift-constrained 45-degree dragging,
Option-duplicate dragging (whole layers and selected pixels), linked layer-mask
movement, hover layer bounds and the additional-options gear, More alignment
options, cross-document dragging, and reference pixel fixtures for these cases.
Passing the repaired cases does not close those items.

Native Photoshop AX observation, menu selection and keyboard shortcuts work.
Coordinate clicks repeatedly fail with `Computer Use server error -10005:
noWindowsAvailable`, including after raising the exact reference window. This
path was stopped rather than repeatedly retrying captures. The disposable
reference document was closed without saving; the original document and its
Marquee tool were restored. No reference pixel-drag experiment is claimed.

The full parity acceptance gate remains pending for Move. Following the user's
2026-10-13 instruction to avoid desktop contention, implementation may proceed
through offline stages in toolbar order while live reference comparisons remain
explicitly pending. No native input or screen capture runs while the user is
using the same Mac.

Move repair validation on 2026-10-13: the two targeted egui interaction tests
and the nested-group model test pass. Workspace tests: 549 passed, 0 failed,
51 ignored (the existing opt-in screenshot/fixture tests remain ignored).
`cargo fmt --all -- --check`, `git diff --check` and
`cargo clippy --workspace --all-targets -- -D warnings` pass. The strict lint
check also required an equivalent range predicate in an existing color-match
test; that changes test construction syntax, not editor behavior.

### Move implementation follow-up

Option-drag now copies selected layer blocks or selected pixels on the active
layer. Copies preserve the originals, including linked originals and nested
group children. Whole-raster-layer movement carries its mask. Core regression
tests and an egui drag/undo test verify these changes. These are implementation
checks; native drag comparison remains pending due to the input-service failure
recorded in `local-analysis.md`. Move has not passed the full 1:1 acceptance gate.

### Offline Move stage

Implemented without desktop interaction: Shift axis/45-degree projection,
functional gear options, canvas and Layers hover bounds, automatic ancestor-group
expansion on selection, all alignment/distribution choices in More, single-layer
canvas alignment, mask-preserving alignment, ancestor mask/opacity hit testing,
and Escape/zero-displacement rollback (including Option copies). Headless tests
exercise input, pixels, options, history and transaction failures.

Implemented in the subsequent offline Move stage: cross-document drag interaction, linked/unlinked mask
controls, exact popup/hover styling and Photoshop pixel references, especially
constraint thresholds and rounding. This stage is not full 1:1 acceptance.

Offline Move validation (2026-10-13): workspace tests 560 passed, 0 failed,
51 ignored; final targeted Move UI tests 8 passed, 1 screenshot test ignored;
Move model tests 13 passed. Formatting, whitespace validation and strict
workspace/all-target clippy pass. No native app was launched, no development
server or port was opened, and no system input or screen capture was used in
this offline stage.


### Move cross-document and mask-link stage

Tab-crossing Move gestures restore the source and copy selected layer blocks or pixels on destination drop. Escape/outside drops cancel. Mask linking can be toggled and undone; unlinked mask and pixels move independently. Link state round-trips in the OpenPhoto PSD extension; Adobe link serialization and live drag timing/Shift placement remain pending reference checks. Proceed to Artboard after offline tests and a separate pushed commit.

Move stage validation: 23 core tests and 9 UI tests passed, one screenshot-only test ignored; mask PSD round-trip passed. Formatting, whitespace checks and strict workspace/all-targets Clippy passed. Tests use no OS windows or system mouse events.

### Artboard offline implementation stage

Artboards now have semantic group geometry and backgrounds, child clipping/hit testing, creation with optional nesting, resize without pixel loss, child/mask movement, automatic canvas expansion, eight resize handles, adjacent plus controls, Option-copy, presets, portrait/landscape, left/top alignment, background/dimension options and undo/cancel transactions. Standard artb/artd PSD descriptors preserve board metadata and are checked with an independent parser/fixture; Photoshop-native opening and a full native option catalog remain pending. Live 1:1 acceptance stays open while the user prohibits desktop interaction. The next toolbar tool is Rectangular Marquee.

Artboard final validation (2026-10-13): workspace tests 577 passed, 0 failed, 51 ignored. Six board model tests, three descriptor tests and four windowless Artboard UI tests passed, covering all eight handles, view compensation, auto-nesting/undo, custom background retention, copy/cancel and exact-size transparent export. Formatting, whitespace checks and strict workspace/all-targets Clippy passed. ag-psd 31.0.3 independently recognized the emitted artb rectangle/background, child group and artd board count. No desktop mouse events, desktop capture, dev/debug servers or OS-window test harnesses were used.

Execution update: subsequent tools are committed and pushed directly to main, as explicitly requested. Rectangular Marquee now latches pointer-down modifiers and supports Escape/Space preview transactions; native algorithm and visual parity remain pending.

Elliptical Marquee offline stage extends outline movement and keyboard nudging and validates circle/center geometry, edge coverage, intersections, feather and undo. The existing 4×4 supersampled mask remains an approximation pending Photoshop output fixtures.

Single Row Marquee offline stage adds live full-width drag previews, edge clamping, feather, cancellation and transactional history.

The WGPU single-row regression exceeded its 180-second deadline after the pixel-center correction; the runner stopped it. Its interaction coverage is moved to a CPU-only egui canvas harness, so semantic input testing does not depend on GPU progress. The initially pushed row commit is not considered validated until this follow-up passes.

Single Row CPU-only regression passed after cleanup. Single Column adds full-height drag preview, feather, cancellation, keyboard movement and undo. Both line tools ignore fixed styles and preserve layer pixels.

Selection Brush offline follow-up adds snapshot cancellation and no-op history suppression, with live add/subtract and undo exercised through the CPU harness.

Freehand Lasso offline follow-up adds Escape cancellation, Enter closure and pointer-down combination modifiers; real native curve/feather comparison is still pending.

Polygonal Lasso offline follow-up adds Shift-constrained vertices and initial-press modifier latching, and verifies anchor removal, explicit closure, cancellation and undo.
