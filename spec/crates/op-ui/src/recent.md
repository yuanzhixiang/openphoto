# recent.rs: File › Open Recent's list

## Responsibilities

The files opened and saved lately, newest first, for File › Open Recent, kept across launches.

## Behavior

- `add(path)`: a file opened (`actions::open_paths`, including files given on the command line) or saved (Save, Save As, Save a Copy) moves to the top, once (paths are canonicalized so the same file isn't listed twice); the list keeps at most `LIMIT` = 20, Photoshop's default "Recent File List Contains".
- `clear()`: Clear Recent File List.
- Every change bumps `revision` (the native menu rebuilds the submenu when it differs) and rewrites the store.
- Store: a plain text file, one path per line, newest first, at `~/Library/Application Support/OpenPhoto/recent-files.txt` (`default_store`); the real app loads it at startup (`lib.rs`), then adds the files it was started with. An unreadable or missing file reads as an empty list; a failed write is logged. Without a store (`RecentFiles::default()`, as in the headless tests) the list lives in memory only.

## Menu and commands

- File › Open Recent (`menu.rs`, `NativeMenu::sync_open_recent`): the files' names, newest first, then a separator and "Clear Recent File List" (disabled when the list is empty), as in Photoshop 2026.
- `Command::OpenRecent(i)`: when the file is already open, its document is brought forward instead; when it no longer exists, the alert "Could not open “<path>” because the file was not found." appears and the list is unchanged; otherwise it opens (and moves to the top). Items past the list are disabled. `Command::ClearRecent` clears the list.

## Test coverage

- `newest_first_without_repeats_and_kept_on_disk` (unit test, in a temporary folder): the limit, moving a repeated file to the top, reading back what was written, clearing.
- `open_recent_reopens_files` (UI test): see `ui_tests.md`.
