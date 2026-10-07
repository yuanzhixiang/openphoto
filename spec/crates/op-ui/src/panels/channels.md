# panels/channels.rs: Channels panel

## Responsibilities

The Channels tab of the third panel group (`mod.md`). It lists the composite (RGB) and the red, green and blue channels, chooses which channels show and which channels edits reach, and loads a channel as a selection.

## Layout

- **Rows:** RGB, Red, Green, Blue, each 37 pt high, with a 1 pt `#454545` line under each.
- **Eye column** (29.5 pt): the eye icon when the row's channel shows. For RGB, the eye shows when all three channels show.
- **Body:**
  - a 28 pt thumbnail, 5 pt from the body's left (`DocState::channel_thumbnails`: the composite in color and each channel in gray, at most 64 pixels on a side, cached by revision);
  - the name, 10 pt after the thumbnail;
  - the shortcut right-aligned 16 pt from the edge (⌘2, ⌘3, ⌘4, ⌘5) in dimmed text.
- **Highlight:** targeted rows are drawn in the row-selected color, a hovered row in the hover color. RGB is highlighted when all three channels are targeted.

## Interactions

- **Click** (`choose`):
  - On a channel, it targets that channel alone and shows it alone.
  - On RGB, it targets and shows all three.
  - ⌘2–⌘5 do the same (`Command::Channel`).
- **Shift-click** on a channel adds it to the targeted (and shown) channels or takes it out. At least one channel stays targeted.
- **Eye click:** on a channel, it toggles whether the channel shows (one always stays). On RGB, it shows all three.
- **⌘-click** (`load_selection`) loads a channel as the selection, recorded as "Load Selection":
  - the channel's values (RGB: the luminosity 0.299 R + 0.587 G + 0.114 B) times the composite's alpha;
  - an all-zero channel leaves no selection.

## Effect on the document (`state.rs`)

- **Display** (`canvas_image`):
  - With one channel shown, the canvas shows it in gray.
  - With two, the hidden one is left out (set to 0).
  - Which channels show is part of the picture's revision, so the canvas updates when it changes.
  - Sampling (the Eyedropper, `DocState::sample`) reads what is shown.
- **Edits** (`DocState::record` → `keep_untargeted_channels`):
  - With only some channels targeted, recording an edit puts the other channels of the active layer's pixels back as they were in the last recorded state.
  - It does nothing with all channels targeted, in Quick Mask, or while a layer mask is the target.
  - Previews (adjustment dialogs, strokes in progress) still show every channel changing until the edit is recorded.

## Known limitations

- There are no alpha channels or spot channels, and no panel menu, New Channel, Delete Channel, Save Selection as Channel or Channel Options buttons.
- Channels show in gray, not in color (Photoshop's "Show Channels in Color" preference).
- Edits limited to channels are applied when recorded, not as they are previewed.

## Test coverage

`ui_tests::channels_panel_targets_shows_and_loads`:

- the Red row targets and shows red alone, and a white fill then changes only red;
- ⌘2 targets all channels again;
- ⌘-click on Red loads a selection, recorded as "Load Selection";
- screenshot `channels_panel.png`.
