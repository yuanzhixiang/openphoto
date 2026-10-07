# panels/navigator.rs: Navigator panel

## Component responsibilities

A thumbnail of the document and the current view position, plus zoom controls.

## Content and interaction

- Thumbnail: the composite image scaled proportionally to fit the panel (`DocState::composite_texture`, cached by revision, longest side 256 pixels), with a checkerboard where transparent.
- View box: the part of the document visible in the window (`document_view::visible_rect`), clipped to the thumbnail, a 1 pt red line (Photoshop's default color).
- Clicking or dragging in the thumbnail area: moves the document position under the pointer to the window center (`document_view::center_on`).
- Bottom: the current zoom percentage; a zoom out icon, a logarithmic zoom slider (1%–12800%) and a zoom in icon. Dragging the slider zooms about the window center (`zoom_to`).

## Known limitations

- The zoom percentage cannot be typed; the zoom out/in icons cannot be clicked; there are no panel options (view box color).
