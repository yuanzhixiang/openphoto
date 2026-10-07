//! Info panel (F8): the color under the pointer in RGB and CMYK, the
//! pointer's position, the selection's size and the document's size, laid
//! out in Photoshop's two columns.

use egui::{Align2, Pos2, Ui, Vec2};

use super::floating::small;
use crate::state::AppState;
use crate::theme::{color, pt};

/// Photoshop's "Doc:" sizes: the flattened image (8-bit RGB) and the
/// layered document (here 3 bytes a pixel per layer, so a one-layer
/// document shows the same size twice, as in Photoshop), in K or M.
fn size_label(bytes: f64) -> String {
    if bytes >= 1024.0 * 1024.0 {
        format!("{:.2}M", bytes / 1024.0 / 1024.0)
    } else {
        format!("{:.1}K", bytes / 1024.0)
    }
}

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let rect = ui.max_rect();
    let Some(state) = app.active() else {
        return;
    };
    let pointer = state.pointer;
    let color = pointer.and_then(|p| {
        (p.x >= 0.0 && p.y >= 0.0)
            .then(|| state.sample_average(p.x as u32, p.y as u32, 1, op_core::SampleScope::All))?
    });
    let sel = state.doc.selection().and_then(|s| s.bounds());
    let (w, h) = (state.doc.width as f64, state.doc.height as f64);
    let layers = state.doc.layers.len() as f64;
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
    let (x, y) = pointer.map_or((String::new(), String::new()), |p| {
        (format!("{:.0}", p.x.floor()), format!("{:.0}", p.y.floor()))
    });
    text(format!("X:  {x}"), pt(18.0), y0);
    text(format!("Y:  {y}"), pt(18.0), y0 + line);
    let (sw, sh) = sel.map_or((String::new(), String::new()), |(x0, y0, x1, y1)| {
        ((x1 - x0).to_string(), (y1 - y0).to_string())
    });
    text(format!("W:  {sw}"), col2, y0);
    text(format!("H:  {sh}"), col2, y0 + line);

    let doc = format!(
        "Doc: {}/{}",
        size_label(w * h * 3.0),
        size_label(w * h * 3.0 * layers)
    );
    text(doc, 0.0, y0 + 2.6 * line);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_like_photoshop() {
        assert_eq!(size_label(1920.0 * 1080.0 * 3.0), "5.93M");
        assert_eq!(size_label(100.0 * 100.0 * 3.0), "29.3K");
    }
}
