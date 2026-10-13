# frame_tool.rs
A valid image press draws a rectangle/ellipse frame, or selects a containing frame to move it with contents. Shift constrains creation to a square/circle; Alt draws from center. Previews remain transient; moving restores the starting snapshot before each absolute update. Escape/tool switch cancels, release records one changed action. Blue outlines with diagonals show frame bounds. Creation and movement have CPU-only tests. Content-only gestures, strokes and Smart Object UI are not implemented.

Eight selected frame handles resize the clipping geometry independently of child pixels. Resize release records Resize Frame; cancelled resizing restores the original snapshot.
