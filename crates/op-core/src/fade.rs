//! Edit › Fade: the last edit's result blended back toward the pixels
//! before it, with an opacity and a blend mode.

use crate::blend::blend;
use crate::layer::BlendMode;
use crate::tile::TiledImage;

/// `after` faded toward `before` (both straight RGBA of the same layer):
/// the edit's result, in `mode` over what was there, mixed with the
/// earlier pixels at `opacity` (premultiplied, so transparent pixels mix
/// cleanly). At 100% Normal the result is `after` itself; at 0% it is
/// `before`.
pub fn fade(before: &TiledImage, after: &TiledImage, opacity: f32, mode: BlendMode) -> TiledImage {
    let o = opacity.clamp(0.0, 1.0);
    let mut out = after.clone();
    let bounds = [before.content_bounds(), after.content_bounds()]
        .into_iter()
        .flatten()
        .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)));
    let Some((x0, y0, x1, y1)) = bounds else {
        return out;
    };
    for y in y0..y1 {
        for x in x0..x1 {
            let b = before.pixel_at(x, y);
            let a = after.pixel_at(x, y);
            if a == b {
                continue;
            }
            out.set_pixel_at(x, y, fade_pixel(b, a, o, mode));
        }
    }
    out
}

/// One pixel: see [`fade`].
pub fn fade_pixel(b: [u8; 4], a: [u8; 4], o: f32, mode: BlendMode) -> [u8; 4] {
    let f = |v: u8| v as f32 / 255.0;
    let (bc, ba) = ([f(b[0]), f(b[1]), f(b[2])], f(b[3]));
    let (ac, aa) = ([f(a[0]), f(a[1]), f(a[2])], f(a[3]));
    // The edit's pixel in the mode: where there was something before,
    // blended with it (the W3C general formula)
    let mc = match mode {
        BlendMode::Normal | BlendMode::Dissolve | BlendMode::PassThrough => ac,
        _ => {
            let mixed = blend(mode, bc, ac);
            [0, 1, 2].map(|c| (1.0 - ba) * ac[c] + ba * mixed[c])
        }
    };
    let alpha = ba * (1.0 - o) + aa * o;
    if alpha <= 0.0 {
        return [0, 0, 0, 0];
    }
    let rgb = [0, 1, 2].map(|c| (bc[c] * ba * (1.0 - o) + mc[c] * aa * o) / alpha);
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [q(rgb[0]), q(rgb[1]), q(rgb[2]), q(alpha)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_mixes_the_before_and_after() {
        let b = [100, 100, 100, 255];
        let a = [200, 0, 50, 255];
        assert_eq!(fade_pixel(b, a, 1.0, BlendMode::Normal), a);
        assert_eq!(fade_pixel(b, a, 0.0, BlendMode::Normal), b);
        assert_eq!(fade_pixel(b, a, 0.5, BlendMode::Normal), [150, 50, 75, 255]);
        // Multiply at 100%: the edit multiplied over what was there
        assert_eq!(fade_pixel(b, a, 1.0, BlendMode::Multiply), [78, 0, 20, 255]);
        // An erased pixel comes half back
        assert_eq!(
            fade_pixel([10, 20, 30, 255], [0, 0, 0, 0], 0.5, BlendMode::Normal),
            [10, 20, 30, 128]
        );
    }

    #[test]
    fn whole_images_fade() {
        let before = TiledImage::filled(4, 4, [0, 0, 0, 255]);
        let after = TiledImage::filled(4, 4, [255, 255, 255, 255]);
        let out = fade(&before, &after, 0.25, BlendMode::Normal);
        assert_eq!(out.pixel(2, 2), [64, 64, 64, 255]);
    }
}
