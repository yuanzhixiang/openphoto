# dialogs/equalize.rs: Equalize Prompt

## Component Responsibilities

The classic dialog shown when Image › Adjustments › Equalize... runs with a selection (409 × 127 pt, rebuilt against Photoshop 2026): asks whether to equalize only the selection, or to equalize the entire image based on the selection's histogram.

## Layout and Interaction

- Group box "Options" (11, 46.5)–(322, 116.5); radio buttons "Equalize selected area only" (27, 70.5) and "Equalize entire image based on selected area" (27, 97), with the latter selected by default (Photoshop's default).
- Buttons x 339.5–398.5: OK (45), Cancel (80). Enter = OK, Esc = Cancel.
- OK returns `Outcome::Apply { entire_image }`: `lib.rs` runs `Adjustment::Equalize` (processes only the selection) or `Adjustment::EqualizeEntireImage` (processes the whole layer based on the selection's histogram); the history name is "Equalize" in both cases.

## Test Coverage

- `ui_tests::equalize_asks_about_the_selection`: without a selection it runs directly; with a selection the prompt appears, defaults to the entire image, and Enter applies. Screenshot `equalize_dialog.png`.
