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
    /// OK and Cancel's left edge and width (89 pt in most).
    pub buttons_x: f32,
    pub buttons_w: f32,
    /// A Randomize button (x0, y0, x1, y1) that draws a new pattern
    /// (Fibers).
    pub randomize: Option<[f32; 4]>,
    /// A plain preview box (x0, y0, x1, y1) instead of the framed one with
    /// gutters and the zoom bar (Lens Flare, whose box also places the
    /// flare's center).
    pub view: Option<[f32; 4]>,
    /// Radio buttons for the setting after the sliders (Lens Flare's Lens
    /// Type).
    pub after: Option<RadioGroup>,
    /// How far the slider rows' fields and tracks sit below the usual
    /// place (`FIELD_Y`, `TRACK_Y`): Lens Flare's are 3 pt higher.
    pub rows_dy: f32,
    /// The fields' width (`FIELD_W` in most; Smart Blur's are 42.5).
    pub field_w: f32,
    /// Pop-ups for the settings after the sliders (Smart Blur's Quality
    /// and Mode): each rect (x0, y0, x1, y1) and its label's right end.
    pub popups: &'static [([f32; 4], f32)],
    pub control: Control,
    /// A pop-up for the last setting (Spherize's Mode, Ripple's Size,
    /// Mezzotint's Type): its rect (x0, y0, x1, y1) and its label's left.
    pub mode: Option<([f32; 4], f32)>,
    /// The diagram's top-left corner.
    pub diagram: Option<(f32, f32)>,
}

/// Blur › Smart Blur: Radius and Threshold over sliders, then the
/// Quality and Mode pop-ups (410 × 491 pt).
pub const SMART_BLUR: Layout = Layout {
    size: (410.0, 491.0),
    buttons_x: 303.5,
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: 42.5,
    popups: &[
        ([77.5, 418.0, 276.5, 435.0], 64.0),
        ([77.5, 455.0, 276.5, 472.5], 64.0),
    ],
    control: Control::Sliders {
        label_ys: &[327.0, 377.0],
        field_x: 229.0,
        track_x1: 275.5,
    },
    mode: None,
    diagram: None,
};

/// Render › Fibers: Variance and Strength over sliders and Randomize
/// under them; 128.5 pt buttons (445 × 445 pt).
pub const FIBERS: Layout = Layout {
    size: (445.0, 445.0),
    buttons_x: 298.5,
    buttons_w: 128.5,
    randomize: Some([98.0, 410.0, 191.0, 436.0]),
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
    control: Control::Sliders {
        label_ys: &[327.0, 377.0],
        field_x: 231.0,
        track_x1: 270.5,
    },
    mode: None,
    diagram: None,
};

/// Render › Lens Flare: the plain preview box with the flare's center,
/// Brightness over a slider and Lens Type (430 × 450 pt).
pub const LENS_FLARE: Layout = Layout {
    size: (430.0, 450.0),
    buttons_x: 283.5,
    buttons_w: 129.5,
    randomize: None,
    view: Some([8.0, 36.0, 266.0, 293.5]),
    after: Some(RadioGroup {
        title: Some(("Lens Type", 365.0)),
        rect: [8.0, 363.0, 266.0, 440.0],
        x: 20.0,
        ys: &[380.0, 396.0, 412.0, 428.0],
    }),
    rows_dy: -3.0,
    field_w: FIELD_W,
    popups: &[],
    control: Control::Slider {
        label_y: 324.0,
        field_x: 203.0,
        track_x1: 255.5,
    },
    mode: None,
    diagram: None,
};

pub const TWIRL: Layout = Layout {
    size: (433.0, 367.0),
    buttons_x: 312.5,
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
    control: Control::None,
    mode: Some(([55.0, 318.0, 253.0, 336.0], 21.0)),
    diagram: None,
};

/// Distort > ZigZag: Amount and Ridges over sliders, the Style pop-up and
/// the diagram (433 × 454 pt).
pub const ZIGZAG: Layout = Layout {
    size: (433.0, 454.0),
    buttons_x: 312.5,
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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
    buttons_w: 89.0,
    randomize: None,
    view: None,
    after: None,
    rows_dy: 0.0,
    field_w: FIELD_W,
    popups: &[],
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

/// The zoom bar: − and + boxes, then the zoom in a combo box. Returns
/// whether − (false) or + (true) was clicked.
pub fn zoom_bar(ui: &Ui, at: impl Fn(f32, f32) -> Pos2, label: &str) -> Option<bool> {
    let painter = ui.painter();
    let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
    let line = Stroke::new(pt(1.0), ZOOM_LINE);
    let mut clicked = None;
    for (x, plus) in [(15.0, false), (32.0, true)] {
        let b = r(x, 296.0, x + 11.0, 307.0);
        if ui
            .interact(
                b.expand(pt(2.0)),
                ui.id().with(("distort-zoom", plus)),
                egui::Sense::click(),
            )
            .clicked()
        {
            clicked = Some(plus);
        }
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
        label,
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
    clicked
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
/// Pinch's diagram as Photoshop 2026 draws it, which is not the filter's
/// own mapping: for Amount −100, −75 … 100 (rows), the distance from the
/// middle (in units of half the diagram) that a pixel at distance 0, 0.05,
/// … 1 shows. Measured from captures of the dialog's diagram.
const PINCH_DIAGRAM: [[f32; 21]; 9] = [
    // -100%
    [
        0.0000, 0.0291, 0.0582, 0.0873, 0.1164, 0.1419, 0.1740, 0.2061, 0.2382, 0.2677, 0.3059,
        0.3441, 0.3771, 0.4296, 0.4831, 0.5459, 0.6269, 0.7342, 0.8626, 0.9299, 1.0000,
    ],
    // -75%
    [
        0.0000, 0.0340, 0.0681, 0.1021, 0.1316, 0.1654, 0.2003, 0.2352, 0.2669, 0.3091, 0.3514,
        0.3895, 0.4431, 0.4966, 0.5561, 0.6272, 0.7047, 0.7800, 0.8691, 0.9299, 1.0000,
    ],
    // -50%
    [
        0.0000, 0.0372, 0.0744, 0.1116, 0.1441, 0.1865, 0.2290, 0.2646, 0.3068, 0.3490, 0.3863,
        0.4398, 0.4934, 0.5440, 0.6057, 0.6713, 0.7523, 0.8205, 0.8810, 0.9405, 1.0000,
    ],
    // -25%
    [
        0.0000, 0.0457, 0.0914, 0.1371, 0.1765, 0.2238, 0.2635, 0.3107, 0.3578, 0.4000, 0.4535,
        0.5033, 0.5520, 0.6054, 0.6540, 0.7156, 0.7704, 0.8432, 0.8967, 0.9484, 1.0000,
    ],
    // 0%
    [
        0.0000, 0.0500, 0.1000, 0.1500, 0.2000, 0.2500, 0.3000, 0.3500, 0.4000, 0.4500, 0.5000,
        0.5500, 0.6000, 0.6500, 0.7000, 0.7500, 0.8000, 0.8500, 0.9000, 0.9500, 1.0000,
    ],
    // 25%
    [
        0.0000, 0.0696, 0.1391, 0.1901, 0.2513, 0.3027, 0.3643, 0.4115, 0.4647, 0.5089, 0.5558,
        0.6028, 0.6423, 0.6893, 0.7363, 0.7760, 0.8230, 0.8700, 0.9087, 0.9544, 1.0000,
    ],
    // 50%
    [
        0.0000, 0.0842, 0.1604, 0.2433, 0.3022, 0.3625, 0.4096, 0.4624, 0.5067, 0.5535, 0.6003,
        0.6387, 0.6806, 0.7226, 0.7574, 0.7954, 0.8334, 0.8714, 0.9087, 0.9544, 1.0000,
    ],
    // 75%
    [
        0.0000, 0.1564, 0.2344, 0.3110, 0.3709, 0.4339, 0.4738, 0.5188, 0.5603, 0.6017, 0.6359,
        0.6738, 0.7116, 0.7494, 0.7789, 0.8136, 0.8482, 0.8774, 0.9183, 0.9591, 1.0000,
    ],
    // 100%
    [
        0.0000, 0.2286, 0.3084, 0.3786, 0.4395, 0.5053, 0.5380, 0.5718, 0.6057, 0.6339, 0.6654,
        0.6970, 0.7285, 0.7552, 0.7898, 0.8245, 0.8591, 0.8890, 0.9260, 0.9630, 1.0000,
    ],
];

/// Spherize's diagram as Photoshop 2026 draws it, laid out as
/// `PINCH_DIAGRAM` (its lines don't bunch into a circle at the edge, as
/// the filter's own mapping would draw them).
const SPHERIZE_DIAGRAM: [[f32; 21]; 9] = [
    // -100%
    [
        0.0000, 0.0842, 0.1604, 0.2433, 0.3090, 0.3813, 0.4552, 0.5237, 0.5848, 0.6364, 0.6978,
        0.7549, 0.7976, 0.8445, 0.8809, 0.9007, 0.9206, 0.9404, 0.9603, 0.9801, 1.0000,
    ],
    // -75%
    [
        0.0000, 0.0696, 0.1391, 0.1989, 0.2632, 0.3342, 0.3957, 0.4686, 0.5264, 0.5878, 0.6396,
        0.7011, 0.7541, 0.8002, 0.8471, 0.8824, 0.9059, 0.9294, 0.9530, 0.9765, 1.0000,
    ],
    // -50%
    [
        0.0000, 0.0593, 0.1185, 0.1737, 0.2488, 0.3027, 0.3643, 0.4161, 0.4777, 0.5296, 0.5912,
        0.6430, 0.7046, 0.7558, 0.8028, 0.8498, 0.8849, 0.9137, 0.9424, 0.9712, 1.0000,
    ],
    // -25%
    [
        0.0000, 0.0593, 0.1185, 0.1647, 0.2178, 0.2630, 0.3249, 0.3811, 0.4387, 0.5004, 0.5459,
        0.5993, 0.6442, 0.6975, 0.7509, 0.7958, 0.8492, 0.8890, 0.9260, 0.9630, 1.0000,
    ],
    // 0%
    [
        0.0000, 0.0500, 0.1000, 0.1500, 0.2000, 0.2500, 0.3000, 0.3500, 0.4000, 0.4500, 0.5000,
        0.5500, 0.6000, 0.6500, 0.7000, 0.7500, 0.8000, 0.8500, 0.9000, 0.9500, 1.0000,
    ],
    // 25%
    [
        0.0000, 0.0457, 0.0914, 0.1371, 0.1765, 0.2238, 0.2635, 0.3107, 0.3578, 0.4000, 0.4535,
        0.5033, 0.5520, 0.6054, 0.6504, 0.7038, 0.7523, 0.8205, 0.8810, 0.9405, 1.0000,
    ],
    // 50%
    [
        0.0000, 0.0410, 0.0821, 0.1231, 0.1579, 0.2002, 0.2425, 0.2780, 0.3202, 0.3623, 0.4031,
        0.4566, 0.5027, 0.5552, 0.6086, 0.6577, 0.7194, 0.7749, 0.8477, 0.9147, 1.0000,
    ],
    // 75%
    [
        0.0000, 0.0372, 0.0744, 0.1116, 0.1429, 0.1812, 0.2194, 0.2541, 0.2897, 0.3278, 0.3659,
        0.4029, 0.4501, 0.4973, 0.5417, 0.5951, 0.6450, 0.7179, 0.7952, 0.8911, 1.0000,
    ],
    // 100%
    [
        0.0000, 0.0340, 0.0681, 0.1021, 0.1316, 0.1654, 0.2003, 0.2352, 0.2656, 0.3038, 0.3419,
        0.3774, 0.4159, 0.4581, 0.5003, 0.5449, 0.5984, 0.6495, 0.7224, 0.8227, 1.0000,
    ],
];

/// A distance (0–1) through one of the diagram tables, interpolated
/// between amounts (−100 … 100) and distances.
fn diagram_distance(table: &[[f32; 21]; 9], amount: i32, r: f32) -> f32 {
    let a = (amount.clamp(-100, 100) + 100) as f32 / 25.0;
    let (i, fa) = ((a.floor() as usize).min(7), a - a.floor().min(7.0));
    let t = r.clamp(0.0, 1.0) * 20.0;
    let (j, fr) = ((t.floor() as usize).min(19), t - t.floor().min(19.0));
    let at = |row: usize| table[row][j] + (table[row][j + 1] - table[row][j]) * fr;
    at(i) + (at(i + 1) - at(i)) * fa
}

/// Where Pinch's or Spherize's diagram takes pixel (`x`, `y`) of an
/// `n`-pixel square from: radially through the table (Spherize's
/// Horizontal or Vertical Only along that axis alone); other filters use
/// their own mapping.
fn diagram_source(filter: Filter, x: f32, y: f32, n: f32) -> (f32, f32) {
    let c = n / 2.0;
    let (dx, dy) = ((x - c) / c, (y - c) / c);
    let r = (dx * dx + dy * dy).sqrt();
    let radial = |table: &[[f32; 21]; 9], amount: i32| {
        if r >= 1.0 || r == 0.0 {
            return (x, y);
        }
        let k = diagram_distance(table, amount, r) / r;
        (c + dx * k * c, c + dy * k * c)
    };
    let along = |d: f32, amount: i32| {
        if d.abs() >= 1.0 || d == 0.0 {
            d
        } else {
            d.signum() * diagram_distance(&SPHERIZE_DIAGRAM, amount, d.abs())
        }
    };
    use op_core::filter::SpherizeMode as M;
    match filter {
        Filter::Pinch { amount } => radial(&PINCH_DIAGRAM, amount),
        Filter::Spherize { amount, mode } => match mode {
            M::Normal => radial(&SPHERIZE_DIAGRAM, amount),
            M::HorizontalOnly => (c + along(dx, amount) * c, y),
            M::VerticalOnly => (x, c + along(dy, amount) * c),
        },
        _ => filter::distortion_source(filter, x, y, n, n),
    }
}

pub fn diagram(filter: Filter, size: usize) -> ColorImage {
    let cross = matches!(filter, Filter::Twirl { .. } | Filter::ZigZag { .. });
    let n = size as f32;
    let step = n / 16.0;
    let source = |x: usize, y: usize| diagram_source(filter, x as f32, y as f32, n);
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
    let crosses =
        |a: f32, b: f32| matches!((band(a), band(b)), (Some(p), Some(q)) if (p - q).abs() == 1);
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

#[cfg(test)]
mod dump {
    /// Writes the diagrams for comparing with Photoshop's captures.
    #[test]
    #[ignore]
    fn dump_diagrams() {
        use op_core::filter::Filter;
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-shots");
        std::fs::create_dir_all(&dir).unwrap();
        let mut cases: Vec<(String, Filter)> = Vec::new();
        for a in [100, 50, -50, -100] {
            cases.push((format!("pinch_{a}"), Filter::Pinch { amount: a }));
        }
        use op_core::filter::SpherizeMode as M;
        for a in [100, 75, 50, 25, -25, -50, -75, -100] {
            cases.push((
                format!("sph_{a}"),
                Filter::Spherize {
                    amount: a,
                    mode: M::Normal,
                },
            ));
        }
        cases.push((
            "sph_h100".into(),
            Filter::Spherize {
                amount: 100,
                mode: M::HorizontalOnly,
            },
        ));
        cases.push((
            "sph_h-100".into(),
            Filter::Spherize {
                amount: -100,
                mode: M::HorizontalOnly,
            },
        ));
        cases.push((
            "sph_v100".into(),
            Filter::Spherize {
                amount: 100,
                mode: M::VerticalOnly,
            },
        ));
        for (name, f) in cases {
            let img = super::diagram(f, 256);
            let px: Vec<u8> = img
                .pixels
                .iter()
                .flat_map(|c| [c.r(), c.g(), c.b(), 255])
                .collect();
            image::RgbaImage::from_raw(256, 256, px)
                .unwrap()
                .save(dir.join(format!("ours_{name}.png")))
                .unwrap();
        }
    }
}
