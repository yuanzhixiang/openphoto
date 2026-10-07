//! Info panel (F8): the color under the pointer in RGB and CMYK, the
//! pointer's position, the selection's size and the document's size, laid
//! out in Photoshop's two columns.

use egui::{Align2, Color32, Pos2, Rect, Stroke, Ui, Vec2};

use super::floating::small;
use crate::rulers::RulerUnit;
use crate::state::AppState;
use crate::theme::pt;

// Photoshop 2026's layout, measured from a 2x capture, in points from the
// top-left corner of the panel's body (below its tab bar), 214 pt wide.
const WIDTH: f32 = pt(214.0);
/// The second column's offset from the first.
const COLUMN: f32 = pt(106.0);
const LINE: f32 = pt(13.25);
/// The readouts' first line (cap-height center), "8-bit" four lines on.
const READOUT_Y: f32 = pt(19.75);
const READOUTS_END: f32 = pt(85.0);
const POSITION_END: f32 = pt(125.0);
/// One row of color samplers: three lines and their margins.
pub const SAMPLER_ROW: f32 = pt(66.0);
/// The status lines' section, for one line.
const STATUS: f32 = pt(28.0);
/// The tool hint's section, for two lines.
const HINT: f32 = pt(46.0);
/// Labels end with the colon here; values end here (first column).
const COLON: f32 = pt(45.0);
const VALUE: f32 = pt(83.5);
const INK: Color32 = Color32::from_gray(0xd6);
const DIVIDER: Color32 = Color32::from_gray(0x42);
const DIVIDER_LOW: Color32 = Color32::from_gray(0x3e);

/// The Info panel's body height for the active document: the sections
/// shown, as in Photoshop.
pub fn height(app: &AppState) -> f32 {
    let options = app.info_options;
    let status = options.status.iter().filter(|on| **on).count();
    let mut h = POSITION_END + sampler_rows(app) as f32 * SAMPLER_ROW;
    if status > 0 {
        h += STATUS + (status - 1) as f32 * LINE;
    }
    if options.tool_hints && tool_hint(app.tool).is_some() {
        h += HINT;
    }
    h + pt(8.0)
}

/// Show Tool Hints' text for a tool, as Photoshop words it (only the
/// hints measured from Photoshop 2026 are known).
pub fn tool_hint(tool: op_tools::Tool) -> Option<[&'static str; 2]> {
    match tool {
        op_tools::Tool::ColorSampler => Some([
            "Click image to place new color sampler.",
            "Use Cmd for additional options.",
        ]),
        _ => None,
    }
}

fn eyedropper(painter: &egui::Painter, c: Pos2) {
    painter.text(
        c,
        Align2::CENTER_CENTER,
        crate::icons::EYEDROPPER,
        crate::theme::icon(pt(14.0)),
        INK,
    );
    menu_triangle(painter, c + Vec2::new(pt(4.5), pt(5.5)));
}

/// The small triangle beside an icon that opens its menu.
fn menu_triangle(painter: &egui::Painter, c: Pos2) {
    painter.add(egui::Shape::convex_polygon(
        vec![
            c + Vec2::new(-pt(3.0), -pt(1.5)),
            c + Vec2::new(pt(3.0), -pt(1.5)),
            c + Vec2::new(0.0, pt(1.5)),
        ],
        INK,
        Stroke::NONE,
    ));
}

/// The pointer position's crosshair.
fn crosshair(painter: &egui::Painter, c: Pos2) {
    let s = Stroke::new(pt(1.0), INK);
    painter.line_segment(
        [c - Vec2::new(pt(6.5), 0.0), c + Vec2::new(pt(6.5), 0.0)],
        s,
    );
    painter.line_segment(
        [c - Vec2::new(0.0, pt(6.5)), c + Vec2::new(0.0, pt(6.5))],
        s,
    );
    menu_triangle(painter, c + Vec2::new(pt(5.5), pt(6.5)));
}

/// The selection size's icon: two arrows round a dotted corner.
fn size_icon(painter: &egui::Painter, c: Pos2) {
    let s = Stroke::new(pt(1.0), INK);
    let (l, t, r, b) = (c.x - pt(6.0), c.y - pt(5.0), c.x + pt(6.0), c.y + pt(7.0));
    painter.line_segment([Pos2::new(l, b - pt(1.5)), Pos2::new(l, t)], s);
    painter.line_segment([Pos2::new(l, t), Pos2::new(r - pt(1.5), t)], s);
    for (tip, dir) in [(Pos2::new(r, t), Vec2::X), (Pos2::new(l, b), Vec2::Y)] {
        let side = Vec2::new(dir.y, dir.x);
        painter.add(egui::Shape::convex_polygon(
            vec![
                tip,
                tip - dir * pt(2.5) + side * pt(2.5),
                tip - dir * pt(2.5) - side * pt(2.5),
            ],
            INK,
            Stroke::NONE,
        ));
    }
    for k in 0..5 {
        let d = k as f32 * pt(2.0);
        painter.rect_filled(
            Rect::from_center_size(Pos2::new(l + pt(3.0) + d, b), Vec2::splat(pt(1.0))),
            0,
            INK,
        );
        painter.rect_filled(
            Rect::from_center_size(
                Pos2::new(r - pt(1.0), t + pt(3.5) + d),
                Vec2::splat(pt(1.0)),
            ),
            0,
            INK,
        );
    }
}

pub fn show(ui: &mut Ui, app: &mut AppState) {
    // The body without the floating frame's margin: Photoshop draws its
    // dividers edge to edge
    let body = ui.max_rect().expand(pt(10.0));
    let o = body.min;
    let options = app.info_options;
    // The mouse coordinates' unit: Info Panel Options', else the rulers'
    let app_units = options.units.unwrap_or(app.ruler_units);
    // Info Panel Options' status lines
    let status_lines: Vec<String> = app
        .active_doc
        .and_then(|id| app.docs.get(&id))
        .map(|d| {
            super::panel_options::STATUS
                .iter()
                .zip(options.status)
                .filter(|(_, on)| *on)
                .map(|(s, _)| s.text(app, d))
                .collect()
        })
        .unwrap_or_default();
    let hint = options.tool_hints.then(|| tool_hint(app.tool)).flatten();
    // The Color Sampler's Sample Size
    let sampler_size = app
        .setting("sampler.size", "0")
        .parse::<usize>()
        .ok()
        .and_then(|i| crate::state::EyedropperOptions::SIZES.get(i))
        .map_or(1, |s| s.0);
    let Some(state) = app.active() else {
        return;
    };
    let pointer = state.pointer;
    let color = pointer.and_then(|p| {
        (p.x >= 0.0 && p.y >= 0.0)
            .then(|| state.sample_average(p.x as u32, p.y as u32, 1, op_core::SampleScope::All))?
    });
    let sel = state.doc.selection().and_then(|s| s.bounds());
    let painter = ui.painter_at(body);
    let font = small();
    let at = |x: f32, y: f32| o + Vec2::new(x, y);
    let text = |s: &str, x: f32, y: f32, align: Align2| {
        painter.text(at(x, y), align, s, font.clone(), INK);
    };
    // "R:" with its letter and colon apart, the colon ending at `x`;
    // the value right-aligned 38.5 pt further on
    let pair = |label: &str, value: &str, x: f32, y: f32| {
        let letter = label.trim_end_matches(':');
        text(letter, x - pt(4.5), y, Align2::RIGHT_CENTER);
        text(":", x, y, Align2::RIGHT_CENTER);
        text(value, x + VALUE - COLON, y, Align2::RIGHT_CENTER);
    };
    let hline = |y: f32, inset: f32, ink: Color32| {
        painter.rect_filled(
            Rect::from_min_max(at(inset, y), at(WIDTH - inset, y + pt(1.0))),
            0,
            ink,
        );
    };
    let vline = |y0: f32, y1: f32| {
        painter.rect_filled(
            Rect::from_min_max(at(pt(106.5), y0), at(pt(107.5), y1)),
            0,
            DIVIDER,
        );
    };

    // The two color readouts (Info Panel Options), each with its bit depth
    for (readout, x) in [(options.first, 0.0), (options.second, COLUMN)] {
        eyedropper(&painter, at(x + pt(14.25), pt(33.75)));
        for (i, (label, value)) in readout.lines(color).iter().enumerate() {
            pair(label, value, x + COLON, READOUT_Y + i as f32 * LINE);
        }
        text(
            "8-bit",
            x + COLON + pt(0.5),
            READOUT_Y + 4.0 * LINE,
            Align2::RIGHT_CENTER,
        );
    }
    vline(pt(3.0), pt(82.0));
    hline(READOUTS_END, pt(1.0), DIVIDER);

    // In the rulers' unit, from their origin
    let unit = app_units;
    let (x, y) = pointer.map_or((String::new(), String::new()), |p| {
        let o = state.ruler_origin;
        (
            unit_value(unit, &state.doc, true, p.x - o.x),
            unit_value(unit, &state.doc, false, p.y - o.y),
        )
    });
    let (sw, sh) = sel.map_or((String::new(), String::new()), |(x0, y0, x1, y1)| {
        (
            unit_value(unit, &state.doc, true, (x1 - x0) as f32),
            unit_value(unit, &state.doc, false, (y1 - y0) as f32),
        )
    });
    crosshair(&painter, at(pt(14.5), pt(106.5)));
    size_icon(&painter, at(COLUMN + pt(14.25), pt(106.5)));
    let row = pt(99.75);
    pair("X:", &x, COLON, row);
    pair("Y:", &y, COLON, row + LINE);
    pair("W:", &sw, COLUMN + COLON, row);
    pair("H:", &sh, COLUMN + COLON, row + LINE);

    // The color samplers, two to a row as in Photoshop: "#1", an
    // eyedropper and R, G, B
    let samplers = state.color_samplers.clone();
    let rows = samplers.len().div_ceil(2);
    let mut y = POSITION_END;
    hline(y, pt(1.0), DIVIDER);
    for (k, s) in samplers.iter().enumerate() {
        let x = if k % 2 == 0 { 0.0 } else { COLUMN };
        let top = POSITION_END + (k / 2) as f32 * SAMPLER_ROW;
        text(
            &format!("#{}", k + 1),
            x + pt(7.0),
            top + pt(11.25),
            Align2::LEFT_CENTER,
        );
        eyedropper(&painter, at(x + pt(14.25), top + pt(24.25)));
        let c = state
            .sample_average(
                s.x as u32,
                s.y as u32,
                sampler_size,
                op_core::SampleScope::All,
            )
            .map(|c| c.to_rgba8());
        for (i, label) in ["R:", "G:", "B:"].iter().enumerate() {
            let value = c.map_or(String::new(), |c| c[i].to_string());
            pair(label, &value, x + COLON, top + pt(11.25) + i as f32 * LINE);
        }
    }
    y += rows as f32 * SAMPLER_ROW;
    vline(pt(83.0), y);
    if !status_lines.is_empty() || hint.is_some() {
        hline(y, 0.0, DIVIDER_LOW);
    }
    if !status_lines.is_empty() {
        for (k, s) in status_lines.iter().enumerate() {
            text(
                s,
                pt(10.0),
                y + pt(16.0) + k as f32 * LINE,
                Align2::LEFT_CENTER,
            );
        }
        y += STATUS + (status_lines.len() - 1) as f32 * LINE;
        if hint.is_some() {
            hline(y, 0.0, DIVIDER_LOW);
        }
    }
    if let Some(lines) = hint {
        for (k, s) in lines.iter().enumerate() {
            text(
                s,
                pt(10.0),
                y + pt(16.5) + k as f32 * LINE,
                Align2::LEFT_CENTER,
            );
        }
    }
}

/// How many rows of color samplers the Info panel shows for the active
/// document.
pub fn sampler_rows(app: &AppState) -> usize {
    app.active_doc
        .and_then(|id| app.docs.get(&id))
        .map_or(0, |d| d.color_samplers.len().div_ceil(2))
}

/// A length in document pixels in the rulers' unit: whole pixels, inches
/// to three decimals, centimeters and picas to two, the rest to one.
fn unit_value(unit: RulerUnit, doc: &op_core::Document, horizontal: bool, px: f32) -> String {
    let v = px / unit.pixels(doc, horizontal);
    match unit {
        RulerUnit::Pixels => format!("{:.0}", v.floor()),
        RulerUnit::Inches => format!("{v:.3}"),
        RulerUnit::Centimeters | RulerUnit::Picas => format!("{v:.2}"),
        RulerUnit::Millimeters | RulerUnit::Points | RulerUnit::Percent => format!("{v:.1}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_in_the_rulers_unit() {
        let doc = op_core::Document::from_rgba8("t", 200, 100, &vec![0; 200 * 100 * 4]);
        assert_eq!(unit_value(RulerUnit::Pixels, &doc, true, 36.6), "36");
        // 72 ppi: 36 px are half an inch, 1.27 cm, 36 points, 3 picas
        let res = doc.resolution;
        assert_eq!(
            unit_value(RulerUnit::Inches, &doc, true, res / 2.0),
            "0.500"
        );
        assert_eq!(
            unit_value(RulerUnit::Centimeters, &doc, true, res / 2.0),
            "1.27"
        );
        assert_eq!(unit_value(RulerUnit::Percent, &doc, true, 50.0), "25.0");
        assert_eq!(unit_value(RulerUnit::Percent, &doc, false, 50.0), "50.0");
    }
}
