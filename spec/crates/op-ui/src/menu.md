# menu.rs: macOS native menu bar

## Responsibilities

Creates the native menu bar on macOS with muda, structured after Photoshop 2026. Compiled only on macOS.

## Menu structure

Toggle commands (View › Extras, Flip Horizontal, Rulers, Show › Layer Edges, Selection Edges, Grid, Guides, Pixel Grid, Guides › Lock Guides) use menu items with a check mark (`CheckMenuItem`), synchronized with `Command::checked` every frame.


Implemented items emit a `Command`; items not yet implemented are shown grayed out with Photoshop's shortcut text, so the menus read the same as Photoshop's. The menu bar order is the same as Photoshop's: OpenPhoto, File, Edit, Image, Layer, Type, Select, Filter, View, Plugins, Window, Help.

- **OpenPhoto**: About OpenPhoto, Services, Hide Others (⌥⌘H), Show All (system predefined items); Hide OpenPhoto uses a custom item (⌃⌘H, emitting `Command::HideApp`), because ⌘H belongs to View › Extras, as in Photoshop; and Quit OpenPhoto (⌘Q). Quit does not use the system predefined item (which would end the process directly) but emits `Command::Quit`, which first asks about unsaved changes.
- **File**: listed in Photoshop 2026's complete order. Available are New..., Open...; Close, Close All, Close Others; Save (⌘S), Save As... (⇧⌘S), Save a Copy... (⌥⌘S), Revert (F12); Export › Export As.... The rest are grayed out with Photoshop's shortcuts: Browse in Bridge... (⌥⌘O), Open as Smart Object..., Open Recent ›, Close and Go to Bridge... (⇧⌘W), Invite to Edit..., Share for Review, Place Embedded..., Place Linked..., Package..., Automate ›, Scripts ›, Import ›, Import from iPhone or iPad, File Info... (⌥⇧⌘I), Version History, Print... (⌘P), Print One Copy (⌥⇧⌘P).
- **Edit**: Undo, Redo, Toggle Last State; Cut, Copy, Copy Merged, Paste, and in the Paste Special submenu Paste in Place, Paste Into (⌥⇧⌘V), Paste Outside; Clear; Fill...; Free Transform (⌘T); Transform › Again (⇧⌘T), Rotate 180°, Rotate 90° Clockwise, Rotate 90° Counter Clockwise, Flip Horizontal, Flip Vertical (the Transform submenu lists all items as in Photoshop; Scale, Rotate, Skew, Distort, Perspective start a transform in the corresponding mode (`TRANSFORM_MODES`), Warp starts a warp (`WARP`), and the three Split Warp items, Remove Warp Split, Convert warp anchor point and Toggle Guides are grayed out). The remaining items are listed in Photoshop's order and grayed out (Fade..., Search, Check Spelling..., Find and Replace Text..., Stroke..., Content-Aware Fill..., Content-Aware Scale, Puppet Warp, Perspective Warp, Auto-Align/Blend Layers..., Define Brush Preset/Pattern/Custom Shape..., Purge, Color Settings..., Assign/Convert to Profile..., Keyboard Shortcuts..., Menus..., Toolbar...). Cut, Copy, Copy Merged, Paste and Paste in Place carry Photoshop's shortcuts (⌘X, ⌘C, ⇧⌘C, ⌘V, ⇧⌘V); when an input field has focus these items stay available and hand the operation to the input field (see "Clipboard" in `commands.md`), so ⌘C/⌘V work as usual in input fields. Clear has no shortcut; ⌫ is handled by egui. macOS automatically appends Writing Tools, AutoFill, Start Dictation and Emoji & Symbols to a menu named "Edit"; Photoshop has these items too.
- **Image**: items, groups and order match Photoshop (Mode ›, Adjustments ›, Auto Tone, Auto Contrast, Auto Color, Image Size..., Canvas Size..., Image Rotation ›, Crop, Trim..., Reveal All, Duplicate..., Apply Image..., Calculations..., Variables ›, Apply Data Set..., Trap..., Analysis ›, with Generative Upscale... also between Image Size... and Canvas Size...). The Adjustments › submenu lists all 24 items in Photoshop's order, of which Brightness/Contrast..., Levels... (⌘L), Curves... (⌘M), Exposure..., Vibrance..., Hue/Saturation... (⌘U), Color Balance... (⌘B), Black & White... (⌥⇧⌘B), Photo Filter..., Channel Mixer..., Invert (⌘I), Posterize..., Threshold..., Gradient Map..., Selective Color..., Desaturate (⇧⌘U) and Equalize are available (labeled "Equalize..." when there is a selection, as in Photoshop), and the rest are grayed out. Also available are Auto Tone (⇧⌘L), Auto Contrast (⌥⇧⌘L), Auto Color (⇧⌘B), Image Size... (⌥⌘I), Canvas Size...; Image Rotation › 180°, 90° Clockwise, 90° Counter Clockwise, Flip Canvas Horizontal, Flip Canvas Vertical (Arbitrary... opens the Rotate Canvas dialog); Crop; Trim...; Reveal All. Everything else is grayed out.
- **Layer**: listed in Photoshop 2026's complete order. Available are: New › Layer... (opens the New Layer dialog), Group..., Group from Layers..., Layer from Background..., Layer Via Copy, Layer Via Cut; Duplicate Layer...; Delete › Layer, Hidden Layers; Layer Mask › Reveal All, Hide All, Reveal Selection, Hide Selection, Delete, Apply, Disable (labeled Enable when the mask is disabled; From Transparency and Link are grayed out); Rename Layer...; Link Layers (shown as Unlink Layers when all selected layers are already linked), Select Linked Layers; Lock Layers... (shows ⌘/, see LockLayers in `commands.md`); Hide Layers; Arrange › Bring to Front, Bring Forward, Send Backward, Send to Back, Reverse; Align › Top Edges, Vertical Centers, Bottom Edges, Left Edges, Horizontal Centers, Right Edges; Distribute › the same six items plus Horizontally, Vertically (with separators between groups); Group Layers (⌘G); Ungroup Layers (⇧⌘G); Merge Down (labeled Merge Layers with multiple layers selected, Merge Group when the current layer is a group); Merge Visible; Flatten Image. The remaining items are grayed out (the Artboard/Frame items in New, Copy CSS, Copy SVG, Quick Export as PNG, Export As..., Layer Style, Smart Filter, New Fill Layer, New Adjustment Layer, Harmonize, Layer Content Options..., Vector Mask, Create Clipping Mask, Mask All Objects, Smart Objects, Video Layers, Rasterize, New Layer Based Slice, Combine Shapes, Lock Layers..., Link Layers, Select Linked Layers, Matting). Photoshop shows the ⌘E item as "Merge Down" when only one layer is selected; here only single selection is possible, so it is fixed as Merge Down.
- **Select**: All, Deselect, Reselect, Inverse; Modify › Border..., Smooth..., Expand..., Contract..., Feather... (⇧F6); Grow, Similar; Edit in Quick Mask Mode (with check mark). The remaining items are the same as Photoshop's but grayed out (All Layers, Deselect Layers, Find Layers, Isolate Layers, Color Range..., Focus Area..., Subject, Sky, Select and Mask..., Transform Selection, Load Selection..., Save Selection...).
- **Type**: listed in Photoshop's order (Panels ›, Anti-Alias ›, Orientation ›, OpenType ›, Create Work Path, Convert to Shape, Rasterize Type Layer, Convert to Paragraph Text, Convert to Dynamic Text ›, Warp Text..., Match Font..., Font Preview Size ›, Language Options ›, Update All Text Layers, Manage Missing Fonts, Paste Lorem Ipsum, Load/Save Default Type Styles), all grayed out.
- **Filter**: lists all items in Photoshop's order. Available are: Last Filter (⌃⌘F; the label shows the name of the last filter used, e.g. "Gaussian Blur", or "Last Filter" if none has been used); Blur › Average, Blur, Blur More, Box Blur..., Gaussian Blur..., Motion Blur..., Surface Blur...; Distort › Pinch..., Polar Coordinates..., Spherize..., Twirl... (the submenu lists the nine items from Displace... to ZigZag... as in Photoshop 2026, the rest grayed out); Noise › Add Noise..., Despeckle, Dust & Scratches..., Median...; Pixelate › Fragment, Mosaic...; Sharpen › Sharpen, Sharpen Edges, Sharpen More, Unsharp Mask...; Stylize › Emboss..., Find Edges, Solarize, Trace Contour..., Wind...; Other › Custom..., High Pass..., Maximum..., Minimum..., Offset.... The rest are grayed out with Photoshop's shortcuts (Adaptive Wide Angle ⌥⇧⌘A, Camera Raw Filter ⇧⌘A, Lens Correction ⇧⌘R, Liquify ⇧⌘X, Vanishing Point ⌥⌘V), and the Blur Gallery, Render and Video submenus are grayed out as a whole.
- **Plugins**: Plugins Panel, Manage Plugins..., grayed out.
- **View**: listed in Photoshop 2026's complete order. Available are Zoom In, Zoom Out, Fit on Screen, Fit Layer(s) on Screen, 100%, 200%, Print Size; Flip Horizontal (with check mark); Extras (⌘H, with check mark); Show › Layer Edges, Selection Edges, Grid (⌘'), Guides (⌘;), Pixel Grid (with check marks), All, None; Rulers (⌘R, with check mark); Guides › Lock Guides (⌥⌘;, with check mark), Clear Guides, New Guide.... The rest are grayed out with Photoshop's shortcuts: Proof Setup ›, Proof Colors (⌘Y), Gamut Warning (⇧⌘Y), Pixel Aspect Ratio ›, Pixel Aspect Ratio Correction, 32-bit Preview Options..., Fit Artboard on Screen, Actual Size, Pattern Preview, Screen Mode ›, the remaining items in Show (Target Path ⇧⌘H, etc.), Snap (⇧⌘;), Snap To ›, the remaining items in Guides, Lock Slices, Clear Slices.
- **Window**: listed in Photoshop 2026's complete order. Available are Histogram, History, Info (F8), Navigator (with check marks); the rest are grayed out with Photoshop's shortcuts (Arrange ›, Workspace ›, Actions ⌥F9, Brush Settings F5, Color F6, Layers F7, etc., plus Application Frame, Options, Tools, Contextual Task Bar). It is also set as the macOS Window menu, and the system lists windows at the end.
- **Help**: OpenPhoto Help (grayed out).

## Dynamic content

`update()` is called every frame to synchronize menu state, calling the native interface only when a value actually changes:

- Each command item's enabled state comes from `Command::enabled`.
- The Undo / Redo text follows the history: "Undo New Layer", "Redo Canvas Size"; when there is no step to undo/redo, it shows "Undo", "Redo".
- The Hide Layers text becomes "Show Layers" when the current layer is hidden.

## Shortcuts

- The shortcut text is generated as a muda accelerator by `Command::shortcut()`.
- Zoom In uses the logical-key accelerator "⌘+" so the menu displays the same as Photoshop; the actual ⌘= key press is handled additionally by `commands::from_shortcuts`.

## Event delivery

Menu events happen outside the egui frame. The event handler puts the command into a channel and calls `request_repaint`; on the next frame `poll()` takes it out and executes it.

## Known limitations

- Generative Upscale... is not listed in the Image menu.
- The Edit menu and the File › Export submenu list only some of the items.

## Tests

- `every_shortcut_is_a_menu_accelerator`: every menu command's shortcut can be parsed by muda into a menu accelerator; a key that fails to parse silently ends up with no shortcut (this is what happened when ⌘= was once written as "Equals").

## Difference from Photoshop: no Adobe branding

The interface shows no Adobe brand names. The four Photoshop menu items pointing to Adobe services are removed: the File menu's Search Adobe Stock..., Search Adobe Express Templates..., Place Free Adobe Stock Images..., and the Type menu's first item More from Adobe Fonts... and the separator after it (so the Type menu starts with Panels ›). The order of the remaining items is unchanged. The test `menus_show_no_adobe_brand` checks that no menu label in `menu.rs` contains "Adobe".
