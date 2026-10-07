# history.rs

## Responsibilities

Implements an undo history modeled on Photoshop's History panel: a chronologically ordered list of named states that supports undo, redo, jumping to any state, deleting from a given state onward, and Edit > Toggle Last State. Each state holds a complete document snapshot; tiles are shared between snapshots, so only edited tiles take extra memory.

## Public interface

- `DEFAULT_LIMIT = 50`: the maximum number of states, matching the default of Photoshop's "History States" preference. The limit includes the first state. `History` has no interface for changing the limit; it is always 50.
- `HistoryState`: public fields `name` (the step name shown in the panel, e.g. `"Open"`, `"New"`, `"Canvas Size"`) and `id` (a number unique within the process, taken from an atomic counter and never reused even after the state is deleted); the snapshot is private.
- `current_id()`: the `id` of the current state. The UI layer remembers `current_id()` at save time; whenever it differs afterwards, there are unsaved changes (undoing back to the saved state counts as no changes).
- `History::new(doc, name)`: takes the document's current contents as the first state (usually named `"Open"` or `"New"`).
- `record(doc, name)`: called after an edit; records the document's current contents as a new state.
- `states()`, `current()`: all states and the current state's index.
- `can_undo()` / `can_redo()`, `undo_name()` / `redo_name()`.
- `undo(doc)` / `redo(doc)`: step back / forward one step.
- `jump(index, doc)`: jump to any state (clicking a row in the History panel).
- `delete_from(index, doc)`: the History panel's delete (trash can) button.
- `toggle_last_state(doc)`: Edit > Toggle Last State.

Every method that changes the current state returns `bool`, indicating whether a jump actually happened; when it is `true`, the document has been `restore`d (and its revision number has therefore been incremented).

## Behavior rules

### Recording

- An edit first modifies the document and then calls `record`; `record` itself does not modify the document.
- If the current state is not the last one (some states have been undone), those later states are discarded first, as in Photoshop.
- The new state is appended to the end and becomes the current state.
- When the limit is exceeded, the state at index 1 (the earliest edit) is deleted and the first state is kept, so the document can always return to how it looked when opened/created. Because snapshots are complete, deleting an intermediate state does not affect the correctness of the other states.
- `record` ends any Toggle Last State in progress.
- `record` does not deduplicate: consecutive edits with identical content still produce new states.

### Undo and redo

- `can_undo` is true if and only if the current index is greater than 0; `can_redo` is true if and only if there are states after the current one.
- `undo_name` returns the current state's name (i.e. the step that undo would revert), and `redo_name` returns the next state's name; both return `None` when undo/redo is not possible.
- `undo` / `redo` are implemented through `jump`, so they also end Toggle Last State.

### Jumping

- `jump` to an out-of-range index or to the current index returns `false` and does nothing.
- Jumping discards no states; after jumping to an earlier state, the later states can still be redone until the next `record`.

### Deleting

- `delete_from(index)` deletes `index` and all states after it, restores the document to the snapshot at `index - 1`, and makes that the current state.
- The first state cannot be deleted: `index == 0` or an out-of-range index returns `false`.
- Deleting ends Toggle Last State.
- It also works when `index` is after the current state: the document is restored to `index - 1`, even if that is a state not yet redone, in which case the document moves forward.

### Toggle Last State

- When not toggling: if the current index is greater than 0, jumps to the previous state and remembers the starting point; returns `false` when the current state is the first one.
- When toggling: jumps back to the remembered starting point and clears the toggle flag. So two toggles in a row return to the original place, and a third steps back again.
- If the remembered starting point is out of range because the history was modified, it is treated as "not toggling".
- Ordinary undo, redo, jump, delete and record all clear the toggle flag.

## Edge cases

- With only one state, undo, Toggle Last State and delete all return `false`.
- Once the number of states reaches the limit, every `record` deletes the earliest edit step, keeping the count at 50.
- The undo history covers only the fields contained in `Snapshot` (width and height, resolution, layers, active layer); changes to the document title, color mode and bit depth are not undone.

## Relationship to other modules

- Depends on `document.rs`'s `snapshot()` / `restore()`.
- Each document state in `op-ui` holds a `History`; menus, shortcuts and the History panel call the methods here.

## Known limitations

- History exists only in memory and is not saved with the file.
- There is no history Snapshot feature and no non-linear history.
- The limit is fixed at 50 and is not configurable.

## Test coverage

- `undo_redo_and_truncate`: after recording, `undo_name` is correct; undo restores the size and the first state cannot be undone further; redo restores; recording after an undo discards the redoable states.
- `delete_from_drops_later_states`: after deleting from index 2, only two states remain, the current one is 1, and the document is restored to the corresponding size; deleting the first state returns `false`.
- `toggle_last_state_round_trips`: two toggles in a row return to the original state; an ordinary undo after a toggle ends the toggle, leaving the first state current, and toggling again returns `false`.
- `limit_keeps_first_state`: after recording 55 edits there are 50 states, the first is still `"Open"`, and jumping back to the first restores the original size.
