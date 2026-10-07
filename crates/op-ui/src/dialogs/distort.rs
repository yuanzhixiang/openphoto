//! The plug-in style dialogs of Filter > Distort (Twirl, Pinch, Spherize,
//! Polar Coordinates), measured on Photoshop 2026: a large framed preview
//! with scroll gutters and a zoom bar at its bottom left, OK and Cancel at
//! the top right, the setting under the preview (a field over a slider
//! with a pentagon thumb, or radio buttons) and a 128 pt diagram of the
//! distortion at the bottom right. Coordinates are Photoshop points from
//! the dialog's top-left corner.

use egui::{Color32, ColorImage, Pos2, Rect, Shape, Stroke, StrokeKind, Ui, vec2};
use op_core::filter::{self, Filter};

use super::appkit;
use crate::theme::pt;

/// The preview's frame; inside it, gutters run down the right and along the
/// bottom, the zoom bar sitting at the bottom's left.
pub const FRAME: [f32; 4] = [8.0, 36.0, 281.0, 309.0];
/// Where the image shows, centered.
pub const VIEW: [f32; 4] = [9.0, 37.0, 265.0, 293.0];
const FRAME_LINE: Color32 = Color32::from_gray(0x3e);
const VIEW_FILL: Color32 = Color32::from_gray(0x4d);
const GUTTER: Color32 = Color32::from_gray(0x50);
const TRACK: Color32 = Color32::from_gray(0x75);
const THUMB_LINE: Color32 = Color32::from_gray(0x21);
const ZOOM_LINE: Color32 = Color32::from_gray(0xdd);
const COMBO_LINE: Color32 = Color32::from_gray(0x66);
const GROUP_LINE: Color32 = Color32::from_gray(0x4f);
const DIALOG: Color32 = Color32::from_gray(0x53);

/// The setting under the preview.
pub enum Control {
    /// A labelled field over a slider: label center y, field left, track
    /// right end.
    Slider {
        label_y: f32,
        field_x: f32,
        track_x1: f32,
    },
    /// Radio buttons in group boxes, one group per setting (Polar
    /// Coordinates, Wind).
    Radios(&'static [RadioGroup]),
    /// Nothing but the pop-up (Mezzotint).
    None,
    /// Several field-over-slider rows, one per setting (ZigZag), at these
    /// label centers.
    Sliders {
        label_ys: &'static [f32],
        field_x: f32,
        track_x1: f32,
    },
}

pub struct Layout {
    pub size: (f32, f32),
    /// OK and Cancel's left edge (89 pt wide).
    pub buttons_x: f32,
    pub control: Control,
    /// A pop-up for the last setting (Spherize's Mode, Ripple's Size,
    /// Mezzotint's Type): its rect (x0, y0, x1, y1) and its label's left.
    pub mode: Option<([f32; 4], f32)>,
    /// The diagram's top-left corner.
    pub diagram: Option<(f32, f32)>,
}

pub const TWIRL: Layout = Layout {
    size: (433.0, 367.0),
    buttons_x: 312.5,
    control: Control::Slider {
        label_y: 327.0,
        field_x: 223.0,
        track_x1: 270.5,
    },
    mode: None,
    diagram: Some((293.0, 227.0)),
};

/// Pixelate > Crystallize and Pointillize: "Cell Size" over a slider, no
/// diagram (measured on Photoshop 2026, 457 × 367 pt).
pub const CELL_SIZE: Layout = Layout {
    size: (457.0, 367.0),
    buttons_x: 350.5,
    control: Control::Slider {
        label_y: 327.0,
        field_x: 284.0,
        track_x1: 322.5,
    },
    mode: None,
    diagram: None,
};

/// Distort > Ripple: Amount over a slider like Spherize's, and the Size
/// pop-up (405 × 404 pt).
pub const RIPPLE: Layout = Layout {
    size: (405.0, 404.0),
    buttons_x: 298.5,
    control: Control::Slider {
        label_y: 327.0,
        field_x: 218.0,
        track_x1: 270.5,
    },
    mode: Some(([52.0, 368.0, 180.0, 386.0], 19.5)),
    diagram: None,
};

/// Pixelate > Mezzotint: only the Type pop-up (405 × 354 pt).
pub const MEZZOTINT: Layout = Layout {
    size: (405.0, 354.0),
    buttons_x: 298.5,
    control: Control::None,
    mode: Some(([55.0, 318.0, 253.0, 336.0], 21.0)),
    diagram: None,
};

/// Distort > ZigZag: Amount and Ridges over sliders, the Style pop-up and
/// the diagram (433 × 454 pt).
pub const ZIGZAG: Layout = Layout {
    size: (433.0, 454.0),
    buttons_x: 312.5,
    control: Control::Sliders {
        label_ys: &[327.0, 377.0],
        field_x: 231.0,
        track_x1: 270.5,
    },
    mode: Some(([56.0, 418.0, 203.5, 436.0], 18.0)),
    diagram: Some((293.0, 314.0)),
};

pub const PINCH: Layout = Layout {
    size: (468.0, 367.0),
    buttons_x: 347.5,
    control: Control::Slider {
        label_y: 324.0,
        field_x: 253.0,
        track_x1: 305.5,
    },
    mode: None,
    diagram: Some((328.0, 227.0)),
};

pub const SPHERIZE: Layout = Layout {
    size: (433.0, 404.0),
    buttons_x: 312.5,
    control: Control::Slider {
        label_y: 327.0,
        field_x: 218.0,
        track_x1: 270.5,
    },
    mode: Some(([59.0, 368.0, 207.0, 386.0], 18.0)),
    diagram: Some((293.0, 264.0)),
};

pub const POLAR: Layout = Layout {
    size: (405.0, 375.0),
    buttons_x: 298.5,
    control: Control::Radios(&[RadioGroup {
        title: None,
        rect: [8.0, 317.0, 281.0, 366.0],
        x: 21.0,
        ys: &[330.0, 354.0],
    }]),
    mode: None,
    diagram: None,
};

/// The field: 36 × 20 pt from y 317; the unit 8.5 pt after it.
pub const FIELD_Y: (f32, f32) = (317.0, 337.0);
pub const FIELD_W: f32 = 36.0;
pub const UNIT_GAP: f32 = 8.5;
/// The slider's 3 pt track starts at x 18.5, y 345; the thumb's center
/// travels 4.5 pt in from either end.
pub const TRACK_X0: f32 = 18.5;
pub const TRACK_Y: f32 = 345.0;
pub const THUMB_INSET: f32 = 4.5;
/// A group box of radio buttons, with an optional title (text, cap
/// center y) inside its top-left corner at x 27.
pub struct RadioGroup {
    pub title: Option<(&'static str, f32)>,
    pub rect: [f32; 4],
    /// The buttons' centers.
    pub x: f32,
    pub ys: &'static [f32],
}

pub const WIND: Layout = Layout {
    size: (405.0, 461.0),
    buttons_x: 298.5,
    control: Control::Radios(&[
        RadioGroup {
            title: Some(("Method", 334.0)),
            rect: [8.0, 332.0, 281.0, 396.0],
            x: 20.0,
            ys: &[349.0, 365.0, 381.0],
        },
        RadioGroup {
            title: Some(("Direction", 405.75)),
            rect: [8.0, 404.0, 281.0, 452.0],
            x: 20.0,
            ys: &[421.0, 437.0],
        },
    ]),
    mode: None,
    diagram: None,
};

/// The preview's frame, view fill and gutters.
pub fn frame(ui: &Ui, at: impl Fn(f32, f32) -> Pos2) {
    let painter = ui.painter();
    let r = |b: [f32; 4]| Rect::from_min_max(at(b[0], b[1]), at(b[2], b[3]));
    painter.rect_filled(r(FRAME), 0, VIEW_FILL);
    // The right gutter and the bottom one, right of the zoom bar
    painter.rect_filled(r([265.0, 37.0, 280.0, 293.0]), 0, GUTTER);
    painter.rect_filled(r([119.0, 293.0, 265.0, 308.0]), 0, GUTTER);
    painter.rect_filled(r([9.0, 293.0, 119.0, 308.0]), 0, DIALOG);
    painter.rect_filled(r([265.0, 293.0, 280.0, 308.0]), 0, DIALOG);
    painter.rect_stroke(
        r(FRAME),
        0,
        Stroke::new(pt(1.0), FRAME_LINE),
        StrokeKind::Inside,
    );
}

/// The image in the view at 100%, centered, with a one-pixel black border.
pub fn image(ui: &Ui, at: impl Fn(f32, f32) -> Pos2, texture: &egui::TextureHandle) {
    let view = Rect::from_min_max(at(VIEW[0], VIEW[1]), at(VIEW[2], VIEW[3]));
    let size = texture.size_vec2() * pt(0.5);
    let shown = Rect::from_center_size(view.center(), size);
    let painter = ui.painter_at(view);
    painter.rect_stroke(
        shown,
        0,
        Stroke::new(pt(0.5), Color32::BLACK),
        StrokeKind::Outside,
    );
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    painter.image(texture.id(), shown, uv, Color32::WHITE);
}

/// The zoom bar: − and + boxes, then the zoom in a combo box.
pub fn zoom_bar(ui: &Ui, at: impl Fn(f32, f32) -> Pos2) {
    let painter = ui.painter();
    let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
    let line = Stroke::new(pt(1.0), ZOOM_LINE);
    for (x, plus) in [(15.0, false), (32.0, true)] {
        let b = r(x, 296.0, x + 11.0, 307.0);
        painter.rect_stroke(b, 0, line, StrokeKind::Inside);
        let c = b.center();
        painter.line_segment([c - vec2(pt(3.0), 0.0), c + vec2(pt(3.0), 0.0)], line);
        if plus {
            painter.line_segment([c - vec2(0.0, pt(3.0)), c + vec2(0.0, pt(3.0))], line);
        }
    }
    let combo = r(47.5, 293.0, 117.5, 307.5);
    painter.rect_filled(r(98.5, 293.0, 117.5, 307.5), 0, appkit::FIELD);
    painter.rect_stroke(
        combo,
        0,
        Stroke::new(pt(1.0), COMBO_LINE),
        StrokeKind::Inside,
    );
    painter.line_segment(
        [at(98.0, 293.0), at(98.0, 307.5)],
        Stroke::new(pt(1.0), COMBO_LINE),
    );
    appkit::text(
        ui,
        at(73.0, 300.25),
        egui::Align2::CENTER_CENTER,
        "100%",
        appkit::TEXT,
    );
    let c = at(108.0, 300.5);
    painter.add(Shape::line(
        vec![
            c + vec2(-pt(3.0), -pt(1.5)),
            c + vec2(0.0, pt(1.5)),
            c + vec2(pt(3.0), -pt(1.5)),
        ],
        Stroke::new(pt(1.0), Color32::from_gray(0xcc)),
    ));
}

/// The slider's track and its pentagon thumb at `place` (0–1).
pub fn slider(ui: &Ui, at: impl Fn(f32, f32) -> Pos2, x1: f32, place: f32) {
    let painter = ui.painter();
    painter.rect_filled(
        Rect::from_min_max(at(TRACK_X0, TRACK_Y), at(x1, TRACK_Y + 3.0)),
        pt(1.5),
        TRACK,
    );
    let (a, b) = (TRACK_X0 + THUMB_INSET, x1 - THUMB_INSET);
    let cx = a + (b - a) * place.clamp(0.0, 1.0);
    let points = vec![
        at(cx, 342.5),
        at(cx + 4.5, 346.0),
        at(cx + 4.5, 349.5),
        at(cx - 4.5, 349.5),
        at(cx - 4.5, 346.0),
    ];
    painter.add(Shape::convex_polygon(
        points,
        appkit::TEXT,
        Stroke::new(pt(1.0), THUMB_LINE),
    ));
}

/// The slider's hit area, to drag along.
pub fn slider_rect(at: impl Fn(f32, f32) -> Pos2, x1: f32) -> Rect {
    Rect::from_min_max(at(TRACK_X0 - 4.0, TRACK_Y - 5.0), at(x1 + 4.0, 352.0))
}

/// The place (0–1) under `x` (screen) along the slider ending at `x1`.
pub fn slider_place(at: impl Fn(f32, f32) -> Pos2, x1: f32, x: f32) -> f32 {
    let (a, b) = (
        at(TRACK_X0 + THUMB_INSET, 0.0).x,
        at(x1 - THUMB_INSET, 0.0).x,
    );
    ((x - a) / (b - a)).clamp(0.0, 1.0)
}

/// The plug-in dialogs' labels are 11 pt (their fields and pop-ups keep
/// 12 pt).
pub fn label_font() -> egui::FontId {
    crate::theme::dialog(pt(11.0))
}

/// An 11 pt label, `align` picking which end `pos` is.
pub fn label(ui: &Ui, pos: Pos2, align: egui::Align2, text: &str) {
    let galley = crate::theme::tracked_galley(ui.painter(), text, label_font(), appkit::TEXT);
    let rect = align.anchor_size(pos, galley.size());
    ui.painter().galley(rect.min, galley, appkit::TEXT);
}

/// A radio group's box and title.
pub fn group(ui: &Ui, at: impl Fn(f32, f32) -> Pos2, group: &RadioGroup) {
    let r = group.rect;
    ui.painter().rect_stroke(
        Rect::from_min_max(at(r[0], r[1]), at(r[2], r[3])),
        pt(3.0),
        Stroke::new(pt(1.0), GROUP_LINE),
        StrokeKind::Inside,
    );
    if let Some((title, y)) = group.title {
        label(ui, at(27.0, y), egui::Align2::LEFT_CENTER, title);
    }
}

/// A radio button: chosen, light with a dark dot; otherwise a gray ring.
pub fn radio(ui: &mut Ui, center: Pos2, label: &str, chosen: bool) -> bool {
    let galley = crate::theme::tracked_galley(ui.painter(), label, label_font(), appkit::TEXT);
    let hit = Rect::from_min_max(
        center - vec2(pt(7.0), pt(8.0)),
        Pos2::new(center.x + pt(14.0) + galley.size().x, center.y + pt(8.0)),
    );
    let clicked = ui
        .interact(
            hit,
            ui.id().with(("distort-radio", label)),
            egui::Sense::click(),
        )
        .clicked();
    let painter = ui.painter();
    if chosen {
        painter.circle_filled(center, pt(6.5), Color32::from_gray(0xb1));
        painter.circle_filled(center, pt(2.0), Color32::from_gray(0x3f));
    } else {
        painter.circle_filled(center, pt(6.5), appkit::FIELD);
        painter.circle_stroke(
            center,
            pt(6.0),
            Stroke::new(pt(1.0), Color32::from_gray(0x80)),
        );
    }
    painter.galley(
        Pos2::new(center.x + pt(14.0), center.y - galley.size().y / 2.0),
        galley,
        appkit::TEXT,
    );
    clicked
}

/// The diagram, `size` pixels square: a grid of lines every `size` / 16
/// pixels (Pinch, Spherize) or a cross through the center (Twirl), bent
/// by the distortion. Black one-pixel lines on white, as Photoshop draws
/// them.
pub fn diagram(filter: Filter, size: usize) -> ColorImage {
    let cross = matches!(filter, Filter::Twirl { .. } | Filter::ZigZag { .. });
    let n = size as f32;
    let step = n / 16.0;
    let source = |x: usize, y: usize| filter::distortion_source(filter, x as f32, y as f32, n, n);
    // Which band between lines a source coordinate falls in
    let band = |v: f32| {
        if !(0.0..=n).contains(&v) {
            None
        } else if cross {
            Some((v >= n / 2.0) as i32)
        } else {
            Some((v / step).floor() as i32)
        }
    };
    // A pixel is on a line where its source crosses one since the pixel to
    // its left or above, so lines stay one pixel wide however the
    // distortion stretches or squeezes them
    let crosses = |a: f32, b: f32| matches!((band(a), band(b)), (Some(p), Some(q)) if p != q);
    let mut pixels = vec![Color32::WHITE; size * size];
    for y in 1..size {
        for x in 1..size {
            let (sx, sy) = source(x, y);
            let (lx, ly) = source(x - 1, y);
            let (tx, ty) = source(x, y - 1);
            if crosses(lx, sx) || crosses(tx, sx) || crosses(ly, sy) || crosses(ty, sy) {
                pixels[y * size + x] = Color32::BLACK;
            }
        }
    }
    ColorImage::new([size, size], pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn black(image: &ColorImage) -> usize {
        image
            .pixels
            .iter()
            .filter(|&&c| c == Color32::BLACK)
            .count()
    }

    #[test]
    fn diagrams_bend_with_the_filter() {
        // Undistorted, Pinch's grid is straight lines every 16 pixels
        let flat = diagram(Filter::Pinch { amount: 0 }, 256);
        assert_eq!(flat.pixels[8 * 256 + 3], Color32::WHITE);
        assert_eq!(flat.pixels[16 * 256 + 3], Color32::BLACK);
        assert_eq!(flat.pixels[3 * 256 + 32], Color32::BLACK);
        // Pinched, the grid bends: many pixels change
        let pinched = diagram(Filter::Pinch { amount: 100 }, 256);
        let moved = (pinched.pixels.iter())
            .zip(&flat.pixels)
            .filter(|(a, b)| a != b)
            .count();
        assert!(moved > 1000, "{moved}");
        // Twirl's cross: an untwirled cross is two lines through the center
        let cross = diagram(Filter::Twirl { angle: 0 }, 256);
        assert_eq!(cross.pixels[10 * 256 + 128], Color32::BLACK);
        assert_eq!(cross.pixels[10 * 256 + 100], Color32::WHITE);
        // Twirled, the top of the vertical line stays (no turn at the edge)
        // while near the center it turns away
        let twirled = diagram(Filter::Twirl { angle: 120 }, 256);
        assert_eq!(twirled.pixels[256 + 128], Color32::BLACK);
        assert_eq!(twirled.pixels[100 * 256 + 128], Color32::WHITE);
        // Lines stay about a pixel wide
        assert!(black(&twirled) < 256 * 2 * 3, "{}", black(&twirled));
    }
}
