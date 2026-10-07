//! Navigator panel: a thumbnail of the document with the part shown in the
//! window outlined in red; dragging on the thumbnail moves the view. Below,
//! the zoom percentage and a zoom slider between the zoom-out and zoom-in
//! buttons.

use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

use super::floating::small;
use crate::document_view;
use crate::icons;
use crate::state::AppState;
use crate::theme::{self, pt};

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let rect = ui.max_rect();
    let box_color = app.navigator_box;
    let ppp = ui.ctx().pixels_per_point();
    let Some(state) = app.active() else {
        return;
    };
    let area = Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.bottom() - pt(32.0)));
    let (w, h) = (state.doc.width as f32, state.doc.height as f32);
    let scale = (area.width() / w).min(area.height() / h);
    let image = Rect::from_center_size(area.center(), Vec2::new(w, h) * scale);
    if let Some(tex) = state.composite_texture(ui.ctx(), 256) {
        crate::widgets::checkerboard(ui.painter(), image, 4.0);
        ui.painter().image(
            tex.id(),
            image,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    // The visible part of the document, clipped to the thumbnail
    let [x0, y0, x1, y1] = document_view::visible_rect(state, ppp);
    let view = Rect::from_min_max(
        image.min + Vec2::new(x0, y0) * scale,
        image.min + Vec2::new(x1, y1) * scale,
    )
    .intersect(image);
    let [r, g, b] = super::panel_options::VIEW_BOX_COLORS
        [box_color.min(super::panel_options::VIEW_BOX_COLORS.len() - 1)]
    .1;
    ui.painter().rect_stroke(
        view,
        0,
        Stroke::new(1.0, Color32::from_rgb(r, g, b)),
        StrokeKind::Inside,
    );
    // Dragging (or clicking) centers the view there
    let response = ui.interact(area, ui.id().with("navigator"), Sense::click_and_drag());
    if (response.dragged() || response.clicked())
        && let Some(p) = response.interact_pointer_pos()
    {
        let doc = (p - image.min) / scale;
        document_view::center_on(state, Pos2::new(doc.x, doc.y), ppp);
    }

    // Zoom percentage (typed: Enter zooms there), zoom out, slider, zoom in
    let row = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - pt(24.0)), rect.max);
    let field = Rect::from_min_size(
        Pos2::new(row.left(), row.center().y - pt(9.0)),
        Vec2::new(pt(54.0), pt(18.0)),
    );
    let id = ui.id().with("navigator-zoom");
    let mut text = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| document_view::zoom_label(state.view.zoom));
    let editor = ui.put(
        field,
        egui::TextEdit::singleline(&mut text)
            .id(id)
            .font(small())
            .desired_width(field.width()),
    );
    if editor.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, text.clone()));
    } else {
        ui.data_mut(|d| d.remove::<String>(id));
    }
    if editor.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && let Some(v) = crate::options_bar::typed_number(&text)
        && v > 0.0
    {
        document_view::zoom_to(state, (v / 100.0).clamp(0.01, 128.0), ppp);
    }
    let mut row_ui = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_max(
        Pos2::new(row.left() + pt(60.0), row.top()),
        row.max,
    )));
    row_ui.horizontal(|ui| {
        let small_mountains =
            egui::Button::new(egui::RichText::new(icons::MOUNTAINS).font(theme::icon(pt(10.0))))
                .frame(false);
        if ui.add(small_mountains).on_hover_text("Zoom Out").clicked() {
            document_view::zoom_step(state, false, ppp);
        }
        // Logarithmic slider over Photoshop's zoom range
        let mut log = state.view.zoom.ln();
        let slider = egui::Slider::new(&mut log, 0.01f32.ln()..=128f32.ln()).show_value(false);
        if ui.add_sized([pt(120.0), pt(18.0)], slider).changed() {
            document_view::zoom_to(state, log.exp(), ppp);
        }
        let big_mountains =
            egui::Button::new(egui::RichText::new(icons::MOUNTAINS).font(theme::icon(pt(15.0))))
                .frame(false);
        if ui.add(big_mountains).on_hover_text("Zoom In").clicked() {
            document_view::zoom_step(state, true, ppp);
        }
    });
}
