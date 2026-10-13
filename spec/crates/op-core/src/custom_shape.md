# custom_shape.rs: Named-path Custom Shape geometry

Custom Shapes use document named Bezier paths, including paths read from standard PSD path resources. Exact cubic extrema determine source bounds; scaling maps every anchor/control into the dragged rectangle. All contours fill together with even/odd rules, preserving holes. Open contours close for filling. Raster coverage uses adaptive cubic display subdivision and 4x4 supersampling, bounded to 16,777,216 canvas pixels; this is not native anti-aliasing acceptance.

add_layer creates a foreground-colored raster result above the active layer, preserving the source path. Empty/degenerate/unavailable geometry adds nothing. Native CSH presets, editable vector fill/stroke layers and custom-shape AA are still unsupported. Tests verify hole preservation, non-square scaling and exact curved bounds rather than control-point bounds.
