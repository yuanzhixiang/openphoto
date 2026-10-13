# Stages 7–14 offline audit (2026-10-13)

The requested full implementation and 1:1 acceptance of stages 7–14 are **not
complete**. The initial audit produces 16 source/test commits and 19 additional offline
regression tests. Desktop mouse input, desktop capture, native Photoshop execution
and UI-based disassembly are absent from this run.

## Initial delivered repairs

| Commit | Tool / repaired contract |
| --- | --- |
| `2a95b89` | Patch Escape/release preserves selection, pixels and history |
| `0abe8ab` | Spot Healing uses interpolated size pressure for the repair window/tip/source search |
| `b135f21` | Sampled Healing repairs the selected mask rather than the image |
| `e903eb1` | Red Eye ignores transparent pixels and reports unchanged output accurately |
| `dd0e1f0` | Mixer deposits opaque grayscale into mask targets |
| `ff42a41` | Art History writes into the selected mask |
| `8a9fd32` | Smudge and Finger Painting use mask targets and preserve image pixels |
| `ac03193` | Clone Stamp samples the active mask; sampled mask writes stay opaque/grayscale |
| `bc9b5d5` | History Brush reads the corresponding layer/Quick Mask history source |
| `490ac5d` | Gradient Escape discards pending lines before pixel edits |
| `aeec9c0` | Five existing raster shape tools cancel before layer creation |
| `4c59a93` | Rotate View Escape clears its cached drag so movement cannot restore it |
| `5fa8611` | Zoom cancels scrubby/box gestures and restores the anchored view |
| `8ad8585` | Screen Mode button/menu use the existing mode/warning state machine |
| `79bc63f` | Patch rejects unwritable/channel targets and pixel-identical results |
| `cf8b0ef` | Content-Aware Move merged-target safety, no-op and canvas cancellation regressions |

Each commit includes mirrored English specifications. Shared painting/gesture
code repairs affect multiple tools; these commits are repairs, not individual
tool parity approvals.

## Initial audit validation

- `cargo fmt --all -- --check` passes.
- `cargo test --workspace`: **629 passed, 0 failed, 51 ignored** across 15 suites.
- `cargo clippy --workspace --all-targets -- -D warnings` passes on the same source.
- CPU canvas tests use synthetic egui events; offscreen workspace tests use their
  existing renderer. Neither accesses the user's desktop input or capture.
- The 51 ignored tests are excluded from acceptance. A passing local contract
  regression does not establish native pixel/UI parity.
- Commands have explicit deadlines. No dev/debug server is started. The static
  REA session is closed; no native target process is launched.

## Static Photoshop evidence

Observed installed version: **27.8.0**, read from the application's Info.plist.
The REA Mach-O inspection is execution-free:

- Evidence: `ev_0ba18beb20928d0236e555554a7b6e2a7eb2862cebfcacacf82aabdb9d50b950`.
- Main executable SHA-256:
  `1abfea49908e828a8bd4c75d7bab43ebce8956c6f12dc003a303ca7656b0f4c4`.
- The executable contains arm64 and x86_64 slices; the selected architecture is
  arm64. Its imports include names for AdbePM inpainting crop, hole classification
  and compositing helpers. Their call sites, data layouts and relation to the
  Remove action are not recovered by this inspection.
- `Required/layouts/Painting/Flyouts/RemoveToolGenAIModeFlyout.eve` declares
  generative AI modes, cloud/device processing choices and model download/error
  states. File SHA-256:
  `7de1623d1660c1be5498028a4e09c0a3e91116379c9a314ca0451b360f5192e6`.
  These declarations describe packaged UI, not measured runtime settings/results.
- The bounded application-file inventory does not establish a usable Remove
  model or ABI. It does not search user caches or prove models are absent there.
- REA reports Hopper available with `may_show_ui: true`; no Hopper analysis is
  executed. Ghidra is unconfigured and IDA is unregistered. Native metadata
  inspection remains available and is used above.

Inference: native inpainting helpers and model selection are useful leads for
further reconstruction. This evidence supplies neither synthesis implementation
nor a native output oracle, and does not justify substituting a smooth fill,
Proximity Match, mirrored texture or a different AI model as 1:1 behavior.

## Remaining implementation and acceptance

The stage table in [the implementation plan](photoshop-toolbar-plan.md) records
all eight stages as pending parity acceptance. Significant implementation gaps
remain in native Remove synthesis, Adjustment Brush native records/properties,
complete path operations/magnetic Freeform, editable text/vector/gradient layers,
native Custom Shape presets and toolbar customization.
Existing healing, Mixer/Art History and tonal/retouch approximations also require
actual native output comparisons and reconstruction. Mask Fade remains pending.

A reproducible native input/settings/output fixture set and recovered native
algorithms/model interfaces are necessary for numerical 1:1 acceptance. Static
layout strings and self-generated expected images cannot provide that evidence.
Remaining data-model, renderer, interaction and PSD changes still require
implementation; none is marked complete by this audit.

## Subsequent canvas implementation run

The following individual changes are implemented, tested offline and pushed to main:

| Commit | Working behavior | Remaining native gate |
| --- | --- | --- |
| `8482082` | Remove marks, subtraction, pressure, local undo/redo, atomic exemplar fill, new-layer/composite sampling | Proprietary synthesis/models, distraction detection and native output |
| `7aa8718` | Horizontal Type Mask selection preview/commit/cancel | Typography, AA and exact overlay |
| `6c97d53` | Upright Vertical Type columns, editing and raster PSD | Native metrics, Roman rotation and editable type records |
| `bd41332` | Vertical Type Mask and live orientation toggle | Native glyph/layout acceptance |
| `e93d6c3` | Editable Pen cubic paths, cancellation/history, canvas remapping and standard PSD paths | Shape/Pixels modes, native path panels/operations and native reopening |
| `a384006` | Exact de Casteljau Add Anchor Point | Native picking/cursor/options |
| `67fa699` | Delete Anchor Point topology/history | Native editing comparison |
| `4a97067` | Convert Point corner/smooth/independent handles | Native gesture/cursor comparison |
| `d25e64d` | Whole-path selection, movement, Shift multi-selection and Alt-copy | Rectangle selection, transforms/alignment/arrangement |
| `b996714` | Direct anchor/linked-handle editing, Alt separation and nudge | Complete node operations, selection UI/native comparison |
| `f42f2d8` | Freeform sampled editable spline, cancellation/history/PSD | Native fitting and magnetic Freeform; no silent magnetic fallback |
| `18ac3f6` | Curvature interpolating topology/closure | Native fitting; own spline is not 1:1 acceptance |
| `a749dcb` | Six retained Adjustment Brush kinds, masks, source preservation, editable settings and own PSD persistence | Native adjustment tags, subject detection and exact properties/history |
| `4d39139` | Functional one/two-column toolbar switching | Native measurements/auxiliary arrangement and customization |
| `fe33187` | Named-path Custom Shape picker, scaling, multi-contour holes and raster layer creation | Native CSH library, vector fill/stroke layers and AA/preview |

Latest full validation: **659 passed, 0 failed, 51 ignored** across 15 suites;
strict workspace clippy passes. No desktop input/capture, native Photoshop execution
or dev/debug server is used. These are actual canvas/data-model implementations,
but they do not close the 7–14 1:1 gate. No substituted spline, exemplar fill,
private PSD extension or raster shape is counted as native parity.
