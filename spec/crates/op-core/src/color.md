# color.rs

## Responsibilities

Defines `Color`: a straight (non-premultiplied) RGBA color in the document color space. All four components are `f32`, by convention in the range 0..=1. The document color space is currently always sRGB, so the components are sRGB-encoded values, not linear-light values. Foreground color, background color, canvas extension color and the like are all represented with it.

## Public interface

- Fields `r`, `g`, `b`, `a` are all public and can be read and written directly.
- Constants: `BLACK` (0,0,0,1), `WHITE` (1,1,1,1), `TRANSPARENT` (0,0,0,0).
- `rgb(r, g, b)`: alpha is fixed at 1; `rgba(r, g, b, a)`: stores the four components as given. Both are `const fn` and do no range checking.
- `from_rgba8([u8; 4])`: divides each component by 255.
- `to_rgba8()`: clamps each component to 0..=1, multiplies by 255, adds 0.5 and truncates to `u8`, i.e. rounds to the nearest value.

## Behavior rules and edge cases

- Constructors do not clamp: values outside 0..=1 are kept and are only cut to 0 or 255 in `to_rgba8`.
- `from_rgba8` followed by `to_rgba8` restores any 8-bit input exactly.
- A `NaN` component is still `NaN` after `clamp`, and becomes 0 under Rust's saturating float-to-integer conversion rules.
- `Color` implements `PartialEq`, comparing floats component by component.

## Relationship to other modules

- `Document::new_with_background` and `Document::resize_canvas` use `to_rgba8` to write colors into pixels.
- `op-color` builds HSB and hexadecimal conversion on top of it.
- `op-ui` uses it to store the foreground/background colors and so on.

## Known limitations

- Represents RGB colors only; carries no color space or ICC information.

## Test coverage

This file has no unit tests; the quantization rules are covered indirectly by the tests in `document.rs` and `op-color`.
