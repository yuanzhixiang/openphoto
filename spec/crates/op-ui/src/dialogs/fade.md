# dialogs/fade.rs: Edit › Fade

## Responsibilities

The Fade dialog (Edit › Fade <edit>..., ⇧⌘F): blending the last filter, adjustment, fill, gradient, Paint Bucket fill or brush stroke back toward the pixels before it, with an opacity and a blend mode. Rebuilt against Photoshop 2026's classic dialog (291 × 144 pt); the faded pixels come from `op_core::fade` (`crates/op-core/src/fade.md`).

## When it is available

- An edit recorded with `DocState::record_fadeable` (filters with or without a dialog, adjustments including Equalize, Edit › Fill and the Fill shortcuts, the Gradient, the Paint Bucket, every painting and retouching tool stroke) can be faded while its history state is the current one: any later history change (or undoing it) ends that; undoing a later state back to it makes it fadeable again.
- Also required: an active layer that has pixels both now and in the previous history state, at the same size. Otherwise the item reads "Fade..." and is disabled; when available it names the edit, "Fade Invert..." (as Photoshop's "Fade Blur...").

## Layout (pt from the dialog's top-left, measured on Photoshop 2026)

- Title "Fade". "Opacity:" at x 10, centered at y 47.5; the field 127–169 × 39.5–56.5, focused with its text selected when the dialog opens; "%" at x 174.
- Slider: a 3 pt `#757575` track 18.5–185 at y 68; the thumb (`appkit::pin`, white) with its tip at y 65.5, running from x 21.5 (0%) to 178.25 (100%). Clicking or dragging along it sets the opacity in whole percent.
- "Mode:" at x 10, y 95.5; the mode pop-up 51–184.5 × 85–106 with every blend mode in Photoshop's groups.
- OK 201–280.5 × 38.5–64.5 (default), Cancel below it (73.5–99.5); Preview checkbox (11.5 pt) at (201, 118.5), on by default.
- Labels, the "%", the thumb and the Preview label were checked against Photoshop's dialog within half a point.

## Interaction

- Typing a percentage (0–100) or moving the slider, changing the mode, or toggling Preview updates the layer at once: with Preview on it shows the faded result, off the edit's result. OK is disabled while the field isn't a percentage.
- OK or Enter keeps the faded pixels and records "Fade <edit>" (Photoshop's "Fade Blur"), which can't itself be faded. Cancel or Esc puts the edit's result back.

## Test coverage

- `opacity_must_be_a_percentage` (unit test).
- `fade_blends_the_last_edit_back` (UI test): see `ui_tests.md`. `screenshot_fade_dialog` (`#[ignore]`) captures the dialog for comparison with Photoshop.
