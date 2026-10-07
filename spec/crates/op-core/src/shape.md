# shape.rs: Shape tools

## Responsibility

The Rectangle, Ellipse, Triangle, Polygon, and Line tools: generates a shape's coverage and adds it to the document as a new layer (filled with a color). Records no history.

## Public interface

- `ShapeKind`: `Rectangle`, `Ellipse`, `Triangle`, `Polygon(sides)`, `Line`. `layer_name()` is the prefix of the new layer's name ("Rectangle", "Ellipse", "Triangle", "Polygon", "Line").
- `coverage(kind, w, h, a, b, weight)`: the shape's coverage on the canvas (a `Selection`, fully anti-aliased).
  - Rectangle, Ellipse, Triangle, and Polygon fill the rectangle formed by points `a` and `b`: the rectangle is that rectangle; the ellipse is the inscribed ellipse; the triangle has its vertices at the midpoint of the top edge and the two bottom corners; the polygon is the regular polygon inscribed in the rectangle (fewer than 3 sides is treated as 3), with the first vertex straight up.
  - Line: a rectangle from `a` to `b`, `weight` pixels wide; empty when the two points coincide.
- `add_shape_layer(doc, kind, a, b, weight, color)`: using the coverage as alpha and `color` as the color, creates a new layer inserted above the current layer, selects it, and returns its ID. The layer name is the prefix plus a number, where the number is the highest existing number for that prefix + 1 (e.g. "Rectangle 1", "Rectangle 2"). When the coverage is empty, no layer is added and `None` is returned.

## Known limitations

- Shapes are raster layers, not Photoshop's vector shape layers: no paths, live shape properties, rounded corners, or strokes, and the shape cannot be edited afterwards.
- No Custom Shape tool and no path/pixel modes.

## Test coverage

- `shapes_cover_their_box`: rectangle extent, ellipse center and corners, triangle, regular hexagon, a 2-pixel-thick horizontal line.
- `shape_layers_are_numbered`: two consecutive rectangles produce "Rectangle 1" and "Rectangle 2", with pixels in the corresponding colors; a zero-size shape adds no layer.
