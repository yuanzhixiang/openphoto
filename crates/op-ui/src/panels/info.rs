//! Info panel (F8): the color under the pointer in RGB and CMYK, the
//! pointer's position, the selection's size and the document's size, laid
//! out in Photoshop's two columns.

use egui::{Align2, Pos2, Ui, Vec2};

use super::floating::small;
use crate::rulers::RulerUnit;
use crate::state::AppState;
use crate::theme::{color, pt};

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let rect = ui.max_rect();
    let app_units = app.ruler_units;
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
    let painter = ui.painter();
    let font = small();
    let text = |s: String, x: f32, y: f32| {
        painter.text(
            rect.min + Vec2::new(x, y),
            Align2::LEFT_TOP,
            s,
            font.clone(),
            color::TEXT,
        );
    };
    let line = pt(16.0);
    let col2 = rect.width() / 2.0 + pt(6.0);
    let rgb = color.map(|c| c.to_rgba8());
    for (i, (label, v)) in ["R:", "G:", "B:"].iter().zip(0..3).enumerate() {
        let value = rgb.map_or(String::new(), |c| c[v].to_string());
        text(format!("{label}  {value}"), pt(18.0), i as f32 * line);
    }
    text("8-bit".into(), pt(18.0), 3.0 * line);
    let cmyk = color.map(op_color::Cmyk::from_color);
    let parts = cmyk.map(|c| [c.c, c.m, c.y, c.k]);
    for (i, label) in ["C:", "M:", "Y:", "K:"].iter().enumerate() {
        let value = parts.map_or(String::new(), |p| format!("{:.0}%", p[i] * 100.0));
        text(format!("{label}  {value}"), col2, i as f32 * line);
    }
    text("8-bit".into(), col2, 4.0 * line);

    let y0 = 5.5 * line;
    painter.line_segment(
        [
            rect.min + Vec2::new(0.0, y0 - pt(4.0)),
            Pos2::new(rect.right(), rect.top() + y0 - pt(4.0)),
        ],
        egui::Stroke::new(1.0, color::SEPARATOR_LIGHT),
    );
    // In the rulers' unit, from their origin
    let unit = app_units;
    let (x, y) = pointer.map_or((String::new(), String::new()), |p| {
        let o = state.ruler_origin;
        (
            unit_value(unit, &state.doc, true, p.x - o.x),
            unit_value(unit, &state.doc, false, p.y - o.y),
        )
    });
    text(format!("X:  {x}"), pt(18.0), y0);
    text(format!("Y:  {y}"), pt(18.0), y0 + line);
    let (sw, sh) = sel.map_or((String::new(), String::new()), |(x0, y0, x1, y1)| {
        (
            unit_value(unit, &state.doc, true, (x1 - x0) as f32),
            unit_value(unit, &state.doc, false, (y1 - y0) as f32),
        )
    });
    text(format!("W:  {sw}"), col2, y0);
    text(format!("H:  {sh}"), col2, y0 + line);

    let (flat, layered) = crate::status_info::document_sizes(state);
    let doc = format!(
        "Doc: {}/{}",
        crate::status_info::size(flat),
        crate::status_info::size(layered)
    );
    // The color samplers, two to a row as in Photoshop: "#1" and R, G, B
    let samplers = state.color_samplers.clone();
    let mut doc_y = y0 + 2.6 * line;
    for (k, s) in samplers.iter().enumerate() {
        let (col, row) = (k % 2, k / 2);
        let x = if col == 0 { 0.0 } else { col2 - pt(18.0) };
        let y = y0 + 2.6 * line + row as f32 * SAMPLER_ROW;
        text(format!("#{}", k + 1), x, y);
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
            text(
                format!("{label}  {value}"),
                x + pt(18.0),
                y + i as f32 * line,
            );
        }
        doc_y = y + SAMPLER_ROW;
    }
    text(doc, 0.0, doc_y);
}

/// One row of color samplers: three lines and a gap.
pub const SAMPLER_ROW: f32 = pt(16.0 * 3.0 + 8.0);

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
