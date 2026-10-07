# commands.rs: Commands and shortcuts

## Responsibilities

Unifies menu items and shortcuts into `Command`, executed in one place by `run()`. Menus are only responsible for issuing commands, and shortcuts only for recognizing keys.

## Commands and shortcuts

Shortcuts match the Photoshop 2026 defaults (obtained by reading Photoshop's menus through the accessibility interface). ⌘ corresponds to Ctrl on other platforms.

| Command | Menu location | Shortcut |
|---|---|---|
| New | File › New... | ⌘N |
| Open | File › Open... | ⌘O |
| Close | File › Close | ⌘W |
| CloseAll | File › Close All | ⌥⌘W |
| CloseOthers | File › Close Others | ⌥⌘P |
| Save | File › Save | ⌘S |
| SaveAs | File › Save As... | ⇧⌘S |
| SaveACopy | File › Save a Copy... | ⌥⌘S |
| Revert | File › Revert | F12 |
| Quit | OpenPhoto › Quit OpenPhoto | ⌘Q |
| ExportAs | File › Export › Export As... | ⌥⇧⌘W |
| Undo | Edit › Undo | ⌘Z |
| Redo | Edit › Redo | ⇧⌘Z |
| ToggleLastState | Edit › Toggle Last State | ⌥⌘Z |
| CanvasSize | Image › Canvas Size... | ⌥⌘C |
| ImageSize | Image › Image Size... | ⌥⌘I |
| Rotate180 | Image › Image Rotation › 180° | None |
| TransformSelection | Select › Transform Selection: transforms only the selection outline with the transform box (`free_transform.md`); available when there is a selection | None |
| RotateArbitrary | Image › Image Rotation › Arbitrary...: opens the Rotate Canvas dialog (`dialogs/rotate_canvas.md`) | None |
| Rotate90Clockwise | Image › Image Rotation › 90° Clockwise | None |
| Rotate90CounterClockwise | Image › Image Rotation › 90° Counter Clockwise | None |
| FlipCanvasHorizontal | Image › Image Rotation › Flip Canvas Horizontal | None |
| FlipCanvasVertical | Image › Image Rotation › Flip Canvas Vertical | None |
| Crop | Image › Crop | None |
| Trim | Image › Trim... | None |
| RevealAll | Image › Reveal All: enlarges the canvas to the extent of all layer pixels (including those outside the canvas), extending the background layer with the background color (`image_ops::reveal_all`), and records "Reveal All"; does nothing when there are no pixels outside the canvas | None |
| Invert | Image › Adjustments › Invert | ⌘I |
| Desaturate | Image › Adjustments › Desaturate | ⇧⌘U |
| Equalize | Image › Adjustments › Equalize | None |
| Threshold | Image › Adjustments › Threshold... | None |
| Posterize | Image › Adjustments › Posterize... | None |
| Levels | Image › Adjustments › Levels... | ⌘L |
| HueSaturation | Image › Adjustments › Hue/Saturation... | ⌘U |
| Exposure | Image › Adjustments › Exposure... | None |
| BrightnessContrast | Image › Adjustments › Brightness/Contrast... | None |
| Curves | Image › Adjustments › Curves... | ⌘M |
| ColorBalance | Image › Adjustments › Color Balance... | ⌘B |
| BlackWhite | Image › Adjustments › Black & White... | ⌥⇧⌘B |
| Vibrance | Image › Adjustments › Vibrance... | None |
| ChannelMixer | Image › Adjustments › Channel Mixer... | None |
| SelectiveColor | Image › Adjustments › Selective Color... | None |
| PhotoFilter | Image › Adjustments › Photo Filter... | None |
| GradientMap | Image › Adjustments › Gradient Map... | None |
| AutoTone | Image › Auto Tone | ⇧⌘L |
| AutoContrast | Image › Auto Contrast | ⌥⇧⌘L |
| AutoColor | Image › Auto Color | ⇧⌘B |
| LastFilter | Filter › Last Filter | ⌃⌘F |
| Average | Filter › Blur › Average | None |
| BoxBlur | Filter › Blur › Box Blur... | None |
| GaussianBlur | Filter › Blur › Gaussian Blur... | None |
| AddNoise | Filter › Noise › Add Noise... | None |
| Median | Filter › Noise › Median... | None |
| Mosaic | Filter › Pixelate › Mosaic... | None |
| UnsharpMask | Filter › Sharpen › Unsharp Mask... | None |
| Solarize | Filter › Stylize › Solarize | None |
| Clouds, DifferenceClouds | Filter › Render › Clouds, Difference Clouds | None |
| HighPass | Filter › Other › High Pass... | None |
| Maximum | Filter › Other › Maximum... | None |
| Minimum | Filter › Other › Minimum... | None |
| Offset | Filter › Other › Offset... | None |
| NewLayer | Layer › New › Layer...: opens the New Layer dialog (`dialogs/new_layer.md`), with the next "Layer N" as the default name | ⇧⌘N |
| NewLayerNoDialog | Creates a new layer directly without a dialog (`panels::new_layer`), consistent with Photoshop's ⌥⇧⌘N; not in the menu | ⌥⇧⌘N |
| DeleteLayer | Layer › Delete › Layer | None |
| ToggleLayerVisibility | Layer › Hide Layers / Show Layers | ⌘, |
| LayerFromBackground | Layer › New › Layer from Background...: opens the Layer from Background version of the New Layer dialog (`dialogs/new_layer.md`) | None |
| LayerViaCopy | Layer › New › Layer Via Copy | ⌘J |
| LayerViaCut | Layer › New › Layer Via Cut | ⇧⌘J |
| DuplicateLayer | Layer › Duplicate Layer...: opens the Duplicate Layer dialog (`dialogs/duplicate_layer.md`); the document list has the current document first, then the other open documents in tab order; the default title for a new document is the next "Untitled-N" | None |
| DeleteHiddenLayers | Layer › Delete › Hidden Layers | None |
| BringToFront | Layer › Arrange › Bring to Front | ⇧⌘] |
| BringForward | Layer › Arrange › Bring Forward | ⌘] |
| SendBackward | Layer › Arrange › Send Backward | ⌘[ |
| SendToBack | Layer › Arrange › Send to Back | ⇧⌘[ |
| MergeDown | Layer › Merge Down | ⌘E |
| MergeVisible | Layer › Merge Visible | ⇧⌘E |
| FlattenImage | Layer › Flatten Image: when there are hidden layers, first asks "Discard hidden layers?" with a macOS alert (warning icon, "Don’t show again" checkbox, Cancel and OK); after OK, flattens (discarding hidden layers) and records "Flatten Image"; once "Don’t show again" is checked, it does not ask again for the current run | None |
| SelectAllLayers | Select › All Layers: selects all layers except the background | ⌥⌘A |
| DeselectLayers | Select › Deselect Layers: selects no layers | None |
| Align(how) | Layer › Align › Top Edges…Right Edges (`op_core::align::align`), records e.g. "Align Left Edges"; the align buttons in the Move tool options bar also execute it | None |
| Distribute(how) | Layer › Distribute › Top Edges…Vertically (`op_core::align::distribute`), records e.g. "Distribute Vertical Centers" | None |
| ArrangeReverse | Layer › Arrange › Reverse (`layer_ops::reverse_selected`), records "Reverse" | None |
| GroupLayers | Layer › Group Layers: groups the selected layers (`layer_ops::group_selected`), records "Group Layers"; unavailable when the background layer is selected | ⌘G |
| UngroupLayers | Layer › Ungroup Layers: ungroups the current group (`layer_ops::ungroup`), records "Ungroup Layers"; available when the current layer is a group | ⇧⌘G |
| NewGroup | Layer › New › Group...: opens the New Group dialog | None |
| NewGroupFromLayers | Layer › New › Group from Layers...: opens the New Group from Layers dialog; same availability as GroupLayers | None |
| LockLayers | Layer › Lock Layers...: opens the Lock Layers dialog (`dialogs/lock_layers.md`); on confirm, sets the locks on the selected non-background layers and records "Lock Layers"; unavailable when only the background is selected. The menu item shows ⌘/, but pressing ⌘/ runs ToggleLockAll; only choosing it from the menu opens the dialog (on macOS, distinguished by whether the menu event occurs during a key event, see `app_kit::handling_key_press`), consistent with Photoshop | ⌘/ (display only) |
| ToggleLockAll | ⌘/: when not all selected layers are "Lock All", turns on Lock All and records "Lock Layer"; when all are fully locked, clears all locks and records "Unlock Layer" (`layer_ops::toggle_lock_all`). When only the background is selected, shows "The command “Set” is not currently available."; in that case the menu item is unavailable and ⌘/ is received by egui (`from_shortcuts_beside_menu`). Not in the menu | ⌘/ |
| LinkLayers | Layer › Link Layers / Unlink Layers and the link button in the Layers panel: when all selected layers are already linked, unlinks them (records "Unlink Layers"); otherwise links the selected layers, together with the layers already linked to them, into one group (records "Link Layers"). Available when two or more layers are selected, or when the selected layers are all linked; the menu label changes to "Unlink Layers" accordingly (`link::can_unlink`) | None |
| SelectLinkedLayers | Layer › Select Linked Layers: adds the layers linked to the selected layers to the selection, leaving the active layer unchanged; available only when it would select more layers. No history is recorded | None |
| RenameLayer | Layer › Rename Layer...: renames the current layer in place in the Layers panel (no dialog, as in Photoshop 2026); unavailable when the current layer is the background layer | None |
| Fill | Edit › Fill... | ⇧F5 |
| FillForeground | None (a hidden Photoshop shortcut) | ⌥⌫ |
| FillBackground | None (a hidden Photoshop shortcut) | ⌘⌫ |
| Clear | Edit › Clear | ⌫ / Delete |
| FreeTransform | Edit › Free Transform | ⌘T |
| TransformAgain | Edit › Transform › Again | ⇧⌘T |
| TransformRotate180 | Edit › Transform › Rotate 180° | None |
| TransformRotate90Clockwise | Edit › Transform › Rotate 90° Clockwise | None |
| TransformRotate90CounterClockwise | Edit › Transform › Rotate 90° Counter Clockwise | None |
| TransformFlipHorizontal | Edit › Transform › Flip Horizontal | None |
| TransformFlipVertical | Edit › Transform › Flip Vertical | None |
| Cut | Edit › Cut | ⌘X |
| Copy | Edit › Copy | ⌘C |
| CopyMerged | Edit › Copy Merged | ⇧⌘C |
| Paste | Edit › Paste | ⌘V |
| PasteInPlace | Edit › Paste Special › Paste in Place | ⇧⌘V |
| PasteInto | Edit › Paste Special › Paste Into | ⌥⇧⌘V |
| PasteOutside | Edit › Paste Special › Paste Outside | None |
| MaskRevealAll | Layer › Layer Mask › Reveal All | None |
| MaskHideAll | Layer › Layer Mask › Hide All | None |
| MaskRevealSelection | Layer › Layer Mask › Reveal Selection | None |
| MaskHideSelection | Layer › Layer Mask › Hide Selection | None |
| MaskDelete | Layer › Layer Mask › Delete | None |
| MaskApply | Layer › Layer Mask › Apply | None |
| MaskToggle | Layer › Layer Mask › Disable / Enable | None |
| SelectAll | Select › All | ⌘A |
| Deselect | Select › Deselect | ⌘D |
| Reselect | Select › Reselect | ⇧⌘D |
| SelectInverse | Select › Inverse | ⇧⌘I |
| ModifyBorder | Select › Modify › Border... | None |
| ModifySmooth | Select › Modify › Smooth... | None |
| ModifyExpand | Select › Modify › Expand... | None |
| ModifyContract | Select › Modify › Contract... | None |
| ModifyFeather | Select › Modify › Feather... | ⇧F6 |
| Grow | Select › Grow | None |
| Similar | Select › Similar | None |
| QuickMask | Select › Edit in Quick Mask Mode | Q (single key, see `actions.md`) |
| ZoomIn | View › Zoom In | ⌘+ (also accepts ⌘=) |
| ZoomOut | View › Zoom Out | ⌘- |
| FitOnScreen | View › Fit on Screen | ⌘0 |
| FitLayers | View › Fit Layer(s) on Screen | None |
| ActualPixels | View › 100% | ⌘1 |
| Zoom200 | View › 200% | None |
| PrintSize | View › Print Size | None |
| ActualSize | View › Actual Size | None |
| Fade | Edit › Fade <edit>... (opens the Fade dialog) | ⇧⌘F |
| ToggleExtras | View › Extras | ⌘H |
| ToggleGrid | View › Show › Grid | ⌘' |
| ToggleGuides | View › Show › Guides | ⌘; |
| FlipView | View › Flip Horizontal | — |
| TogglePixelGrid | View › Show › Pixel Grid | — |
| ToggleSelectionEdges | View › Show › Selection Edges | — |
| ToggleLayerEdges | View › Show › Layer Edges | — |
| ShowAllExtras | View › Show › All | — |
| ShowNoExtras | View › Show › None | — |
| ToggleSnap | View › Snap | ⇧⌘; |
| RulerUnits(unit) | a ruler's right-click menu: the rulers' unit (checked) | — |
| StatusInfo(item) | the status bar's menu: what the bar shows (checked) | — |
| ScreenMode(mode) | View › Screen Mode › Standard Screen Mode, Full Screen Mode With Menu Bar, Full Screen Mode | — (F cycles, see `actions.md`) |
| SnapToGuides, SnapToGrid, SnapToLayers, SnapToSlices, SnapToBounds | View › Snap To › Guides, Grid, Layers, Slices, Document Bounds | — |
| SnapToAll, SnapToNone | View › Snap To › All, None | — |
| ToggleRulers | View › Rulers | ⌘R |
| LockGuides | View › Guides › Lock Guides | ⌥⌘; |
| ClearGuides | View › Guides › Clear Guides | None |
| NewGuide | View › Guides › New Guide... | None |
| HideApp | OpenPhoto › Hide OpenPhoto | ⌃⌘H |
| ToggleHistory | Window › History | None |
| ToggleHistogram | Window › Histogram | None |
| ToggleInfo | Window › Info | F8 |
| ToggleNavigator | Window › Navigator | None |

## Availability (`enabled`)

Determines both whether a command can execute and whether its menu item is grayed out:

- When an input box has keyboard focus (`AppState::typing`), Cut, Copy, CopyMerged, Paste and PasteInPlace are always available (including input boxes in modal dialogs), and act on the input box's text when executed (see "Clipboard" below).
- Apart from that, all commands are unavailable while a modal dialog is open.
- New, Open, ToggleHistory, ToggleInfo, ToggleNavigator, ToggleHistogram, Quit, HideApp and the view toggles (Rulers, Extras, Guides, Grid, Lock Guides, Pixel Grid, Selection Edges, Layer Edges, Show › All / None, Snap and the Snap To items, Screen Mode, the status bar's items, the ruler units) are always available (except when a modal dialog is open); ClearGuides is available when the current document has guides; Fade is available while the last edit can be faded (`DocState::can_fade`, see `dialogs/fade.md`). FitLayers, Zoom200, PrintSize, ActualSize, NewGuide and FlipView are available when there is a current document.
- Revert: available when the current document has a file and unsaved changes. Save, Save As, Save a Copy: available when there is a current document.
- Undo, ToggleLastState: available when the current document can undo; Redo: available when it can redo.
- DeleteLayer: available when the current document has more than 1 layer.
- CloseOthers: available when more than 1 document is open.
- ToggleLayerVisibility acts on all selected layers (shows them when all are hidden, with the menu label correspondingly Show Layers / Hide Layers); DeleteLayer deletes all selected layers and is available when the selection is not all layers; Align requires `can_align`, Distribute requires `can_distribute`; SelectAllLayers requires a non-background layer, DeselectLayers requires an active layer.
- ToggleLayerVisibility, DuplicateLayer, LayerViaCopy: available when the current document has selected layers; LayerViaCut additionally requires a selection.
- Crop, PasteInto, PasteOutside: available when the current document has a selection. MaskRevealAll, MaskHideAll: available when a mask can be added to the current layer; MaskRevealSelection, MaskHideSelection additionally require a selection; MaskDelete, MaskApply, MaskToggle: available when the current layer has a mask. TransformAgain: available when there is a current document and a transform has been done in the current run. LastFilter: available when there is a current document and a filter has been used in the current run.
- LayerFromBackground: available when the document has a background layer. DeleteHiddenLayers: available when there are both hidden and visible layers.
- The four Arrange items: available when the current layer can move by that command (`layer_ops::arrange_target`). MergeDown: when the current layer is a non-empty group, it is Merge Group (`layer_ops::merge_group`, records "Merge Group", menu label "Merge Group"); when multiple layers are selected, it is Merge Layers (`layer_ops::merge_selected`, records "Merge Layers", with the menu label changing to "Merge Layers"), available when at least two of them are visible; when only one is selected, it is Merge Down, available when both the current layer and the layer below it are visible; MergeVisible: available when more than one layer is visible; FlattenImage: available when the document is not just a single background layer.
- Deselect, SelectInverse, the five Modify items, Grow, Similar: available when the current document has a selection; Reselect: available when there is no selection and there is a selection that can be restored; SelectAll: available when there is a current document.
- All other commands: available when there is a current document. Paste does not check whether the clipboard has content (reading an image from the system clipboard is expensive and unsuitable for checking every frame); executing Paste with an empty clipboard does nothing.

## Fill and Clear

- Fill: opens the Fill dialog (see `dialogs/fill.md`).
- FillForeground / FillBackground: immediately fills the current layer's selection (the whole layer when there is no selection) with the foreground / background color, Normal, 100%, recording "Fill". When filling is not possible, shows Photoshop's alert.
- Clear: with a selection, clears the selection's content (fills with the background color on the background layer), recording "Clear"; with no selection, deletes the current layer (when there is more than one layer), consistent with Photoshop 2026.
- ⌥⌫, ⌘⌫ and ⌫ are not menu shortcuts: they are recognized in egui by `from_shortcuts_beside_menu` on macOS and by `from_shortcuts` on other platforms. These keys are not handled while typing in an input box, so as not to interfere with deleting text.

## History records for selection commands

QuickMask enters or exits Quick Mask (`toolbar::toggle_quick_mask`), recording "Quick Mask" in both cases; the menu item carries a check mark to show that the current document is in Quick Mask. The five Modify items open the corresponding dialogs (`dialogs/modify_selection.md`); Grow and Similar immediately enlarge the selection using the Magic Wand options (`AppState::wand`'s tolerance, anti-alias, sample all layers), recording "Grow" and "Similar".


Select All, Deselect, Reselect and Inverse each record one history entry, named "Select All", "Deselect", "Reselect" and "Select Inverse" respectively, consistent with Photoshop.

## View commands

- ToggleInfo, ToggleNavigator and ToggleHistogram open or close the corresponding floating panels (`panels/floating.md`), with check marks on the menu items; the History menu item likewise carries a check mark.
- The five view toggles switch the corresponding fields of `AppState::view` (see `rulers.md`), and the menu items show their checked state (`Command::checked`).
- Zoom200: zooms to 200% around the window center. PrintSize: zooms so that 1 inch (the document resolution's number of pixels) is 72 pt on screen. FitLayers: makes the current layer's non-transparent pixels fill the window and centers them (no change for an empty layer).
- ClearGuides: deletes all guides in the current document, recording "Clear Guides". NewGuide: opens the New Guide dialog (`dialogs/new_guide.md`).
- HideApp: hides the application (`app_kit::hide_app`, effective only on macOS). Photoshop changes the system "Hide" to ⌃⌘H and gives ⌘H to Extras; this is the same here.

## Transform commands

- FreeTransform: starts Free Transform (`free_transform.md`); shows an alert when transforming is not possible.
- The five fixed transforms under Transform: apply a `FixedTransform` mapping around the center of the `transform::bounds` extent, record the corresponding name (e.g. "Flip Horizontal"), and are stored as `last_transform`.
- TransformAgain: applies the `last_transform` mapping once more, recording "Transform Again".
- On failure, shows Photoshop's alert.

## Canvas commands

- The five Image Rotation items: rotate or flip the whole document (`image_ops::reorient`); rotation records "Rotate Canvas", flipping records "Flip Canvas Horizontal" / "Flip Canvas Vertical".
- Crop: crops to the selection's bounding rectangle (`image_ops::crop_to_selection`), recording "Crop".
- Trim: opens the Trim dialog (see `dialogs/trim.md`).
- ImageSize: opens the Image Size dialog with the current document's width, height and resolution (see `dialogs/image_size.md`).

## Adjustment commands

- Invert, Desaturate, Equalize, Auto Tone, Auto Contrast, Auto Color: apply immediately to the current layer (within the selection) (`adjust.md` in `op-core`), recording history under the same name; when the layer is hidden or its pixels are locked, Photoshop's alert is shown (e.g. "Could not complete the Invert command because the target layer is hidden.").
- The other adjustments with dialogs (Threshold, Posterize, Levels, Curves, Hue/Saturation, Exposure, Brightness/Contrast, Color Balance, Black & White, Vibrance, Channel Mixer, Selective Color, Photo Filter, Gradient Map): perform the same checks first, showing an alert on failure; if they pass, compute the histogram (Threshold and others use the luminance histogram), save a document snapshot, and open the dialog (see `dialogs/adjust.md`; see `lib.md` for the preview flow); Gradient Map takes the foreground and background colors, Hue/Saturation's Colorize starts from the foreground color's hue; Levels and Curves additionally take the histograms of the red, green and blue channels (`adjust::rgb_histograms`).

## Filter commands

- Clouds, Difference Clouds: apply immediately with the current foreground and background colors and a new pattern each time (`AppState::next_seed`; `seed_override` fixes it in tests); Last Filter repeats them the same way, with the colors at that moment.
- Average, Solarize: apply immediately to the current layer (`filter.md` in `op-core`), recording history under the same name, and are stored as `last_filter`.
- Filters with settings (Box Blur, Gaussian Blur, Add Noise, Median, Mosaic, Unsharp Mask, High Pass, Maximum, Minimum, Offset): checked the same way as adjustment commands; if the checks pass, save a snapshot and open the dialog (`dialogs/adjust.md`).
- LastFilter: immediately applies again with `last_filter`'s settings, recording that filter's name, without showing a dialog.
- On failure, shows Photoshop's alert, e.g. "Could not complete the Gaussian Blur command because the target layer is hidden.".

## Layer commands

Layer mask commands call the mask functions in `layer_ops`, recording "Add Layer Mask", "Delete Layer Mask", "Apply Layer Mask", and "Disable Layer Mask" / "Enable Layer Mask" respectively.


See `layer_ops.md` in `op-core` for pixel and ordering rules. History names recorded on success: Duplicate Layer → "Duplicate Layer", Layer Via Copy / Cut → "Layer Via Copy" / "Layer Via Cut", Layer from Background → "Layer From Background", Delete › Hidden Layers → "Delete Hidden Layers", the four Arrange items → "Layer Order", Merge Down / Merge Visible / Flatten Image → the same names. When Layer Via Copy / Cut fails, Photoshop's alert is shown (e.g. when the selection contains only transparent pixels, "Could not complete the Layer Via Copy command because the selected area is empty.").

The `[` and `]` in menu shortcuts are written as the symbols themselves when passed to muda (muda does not recognize egui's `OpenBracket` / `CloseBracket` names).

## Clipboard

Cut, Copy, CopyMerged, Paste and PasteInPlace are executed by `actions::clipboard`:

- When an input box has focus, the operation is forwarded to the input box: Cut/Copy (CopyMerged is treated as Copy) become egui's `Event::Cut` / `Event::Copy`, and Paste/PasteInPlace read the text from the system clipboard and become `Event::Paste` (doing nothing when there is no text). These events are put into `AppState::forward_events`, injected into egui's next frame of input by `OpenPhotoApp::raw_input_hook`, and a repaint is requested. On macOS these menu items carry the ⌘X/⌘C/⌘V shortcuts, and the key presses are caught by the native menu so egui never receives them; this forwarding is therefore needed for cut, copy and paste to work in input boxes.
- Otherwise they act on the current document (see `clipboard.md` in `op-core` for pixel rules):
  - Copy / Copy Merged: on success, put the copy on the clipboard (see `clipboard.md`); no history is recorded.
  - Cut: copies and clears, recording "Cut".
  - Paste Into / Paste Outside: the content is centered on the center of the selection's bounding rectangle, pasted as a new layer, and then given a mask from the original selection (Into reveals inside the selection, Outside reveals outside it); the selection is deselected, recording "Paste Into" / "Paste Outside".
  - Paste / Paste in Place: takes the content from the clipboard, computes the position from the visible area of the current view (`document_view::visible_rect`), pastes it as a new layer, and records "Paste". Does nothing when the clipboard is empty.
  - On failure, shows Photoshop's alert, e.g. "Could not complete the Copy command because the selected area is empty.".

`run()` checks `enabled` once more before executing; unavailable commands are simply ignored.

## Shortcut recognition

- **macOS**: shortcuts with ⌘ are handled by the native menu's key equivalents, and egui does not receive those key presses. The only exception is Zoom In: the menu item displays Photoshop's ⌘+, but macOS only matches "+" while Shift is held, so `from_shortcuts_beside_menu` additionally captures ⌘= in egui and turns it into ZoomIn.
- `Shortcut` has four modifiers: `cmd` (Command on macOS), `shift`, `alt`, `ctrl` (the Control key on macOS). When passed to muda, `ctrl` is written as `Ctrl+`, which is the Control key on macOS.
- **Without a native menu** (other platforms, and windowless tests on macOS): while a modal dialog is open, `from_shortcuts` consumes no keys (all commands are unavailable then, and keys are left to the dialog, e.g. ⌘D "Don't Save" in a confirmation box); otherwise it matches all shortcuts in egui in `SHORTCUT_ORDER`. egui's `consume_key` ignores extra Shift/Alt, so the list must put more specific combinations first (e.g. ⇧⌘Z before ⌘Z, ⌥⌘W before ⌘W, ⇧⌘C before ⌘C), otherwise they would be mistakenly triggered as another command. ZoomIn accepts both ⌘= and ⌘+.
- egui-winit does not send ⌘X/⌘C/⌘V as key presses, but turns them into `Event::Cut`, `Event::Copy` and `Event::Paste` (and only sends `Paste` when the system clipboard has text). `from_shortcuts` recognizes these events as Cut, Copy (CopyMerged when Shift is held) and Paste (PasteInPlace when Shift is held). While typing in an input box, clipboard shortcuts and events are not recognized and are left to the input box itself.

## Differences from Photoshop

- On platforms without a native menu, ⌘V receives no event when the system clipboard holds only an image (see above); pasting is only possible through the menu.

- Equalize: with a selection, it does not execute directly but first shows the Equalize prompt (`dialogs/equalize.md`).
- Filter › Blur › Blur, Blur More, Sharpen › Sharpen, Sharpen More, Stylize › Find Edges: execute directly like Average and Solarize (checked the same as adjustments), recording history under the same name and becoming the Last Filter. Motion Blur... and Emboss... open filter dialogs (`dialogs/adjust.md`).
- Filter › Distort › Twirl..., Pinch..., Spherize..., Polar Coordinates... open filter dialogs.
- Filter › Blur › Surface Blur... and Noise › Dust & Scratches... open filter dialogs; Pixelate › Fragment executes directly (like Blur), recording "Fragment" and becoming the Last Filter. Other › Custom... opens the Custom filter dialog (`dialogs/custom_filter.md`), recording "Custom". Noise › Despeckle and Sharpen › Sharpen Edges execute directly (like Blur); Stylize › Trace Contour... and Wind... open filter dialogs.
- When opening a filter dialog, if `AppState::filter_settings` holds the settings from the last time OK was pressed in that dialog, they are first put back with `restore` (Photoshop's filter dialogs remember the last values; see `dialogs/adjust.md`).
- When a text input box has focus, Select All (⌘A) selects all the text in the input box rather than the whole canvas (the shortcut is handed to the input box; when the macOS native menu receives ⌘A first, it is turned into a key event and handed to the input box).
