//! The Gradients and Patterns panels: their presets as swatches; a click
//! makes one the Gradient tool's gradient or the Pattern Stamp's pattern.

use egui::{Color32, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::state::AppState;
use crate::theme::{color, pt};

/// A swatch and the gap after it.
const SWATCH: f32 = pt(36.0);
const GAP: f32 = pt(6.0);

/// The swatch cells for `n` items across the panel, from its top-left
/// with a margin.
fn cells(ui: &Ui, n: usize) -> Vec<Rect> {
    let area = ui.max_rect().shrink(pt(10.0));
    let per_row = (((area.width() + GAP) / (SWATCH + GAP)).floor() as usize).max(1);
    (0..n)
        .map(|k| {
            let (col, row) = (k % per_row, k / per_row);
            Rect::from_min_size(
                area.min + Vec2::new(col as f32 * (SWATCH + GAP), row as f32 * (SWATCH + GAP)),
                Vec2::splat(SWATCH),
            )
        })
        .collect()
}

/// The swatch's edge: white when chosen or hovered.
fn frame(ui: &Ui, rect: Rect, lit: bool) {
    ui.painter().rect_stroke(
        rect,
        0,
        Stroke::new(
            pt(1.0),
            if lit {
                Color32::WHITE
            } else {
                color::DIVIDER_DARK
            },
        ),
        StrokeKind::Outside,
    );
}

/// Gradients panel: the editor's presets (with the current colors) and the
/// ones made with New. The tool's gradient is outlined.
pub fn gradients(ui: &mut Ui, app: &mut AppState) {
    let presets = app.gradient_presets();
    let current = app.tool_gradient();
    let mut picked = None;
    for (k, (g, rect)) in presets.iter().zip(cells(ui, presets.len())).enumerate() {
        crate::widgets::checkerboard(ui.painter(), rect, pt(4.0));
        crate::dialogs::gradient_editor::paint_gradient(ui.painter(), rect, g);
        let response = ui
            .interact(rect, ui.id().with(("gradient-swatch", k)), Sense::click())
            .on_hover_text(&g.name);
        frame(
            ui,
            rect,
            response.hovered() || (g.colors == current.colors && g.opacities == current.opacities),
        );
        if response.clicked() {
            picked = Some(g.clone());
        }
    }
    if let Some(g) = picked {
        app.gradient_preset = Some(g);
    }
}

/// Patterns panel: the patterns, tiled; the Pattern Stamp's is outlined.
pub fn patterns(ui: &mut Ui, app: &mut AppState) {
    let n = app.patterns.len();
    let mut picked = None;
    for (k, rect) in cells(ui, n).into_iter().enumerate() {
        let pattern = &app.patterns[k];
        let image = &pattern.image;
        let painter = ui.painter_at(rect);
        let (pw, ph) = (image.width().max(1), image.height().max(1));
        let side = (rect.width() / pt(1.0)).ceil() as u32;
        for y in 0..side {
            for x in 0..side {
                let [r, g, b, a] = image.pixel(x % pw, y % ph);
                let cell = Rect::from_min_size(
                    rect.min + Vec2::new(x as f32 * pt(1.0), y as f32 * pt(1.0)),
                    Vec2::splat(pt(1.0)),
                );
                painter.rect_filled(cell, 0, Color32::from_rgba_unmultiplied(r, g, b, a));
            }
        }
        let response = ui
            .interact(rect, ui.id().with(("pattern-swatch", k)), Sense::click())
            .on_hover_text(&pattern.name);
        frame(ui, rect, response.hovered() || app.pattern == k);
        if response.clicked() {
            picked = Some(k);
        }
    }
    if let Some(k) = picked {
        app.pattern = k;
    }
}
