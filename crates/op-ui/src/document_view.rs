//! Document window: the canvas (zoom/pan/tool input) and the status bar.

use egui::{Align2, Color32, CursorIcon, Key, PointerButton, Pos2, Rect, Sense, Ui, Vec2};
use op_core::DocId;
use op_tools::Tool;

use crate::state::{AppState, DocState};
use crate::theme::{self, color, pt, size};

/// Photoshop's preset zoom levels, in percent.
const ZOOM_STEPS: &[f32] = &[
    1.0, 2.0, 3.0, 4.0, 5.0, 6.25, 8.33, 12.5, 16.67, 25.0, 33.33, 50.0, 66.67, 100.0, 150.0,
    200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 1200.0, 1600.0, 2400.0, 3200.0, 6400.0,
    12800.0,
];
const MIN_ZOOM: f32 = 0.01;
const MAX_ZOOM: f32 = 128.0;

pub fn zoom_label(zoom: f32) -> String {
    let pct = zoom * 100.0;
    if (pct - pct.round()).abs() < 0.01 {
        format!("{pct:.0}%")
    } else {
        format!("{pct:.2}%")
    }
}

pub fn next_zoom_step(zoom: f32, zoom_in: bool) -> f32 {
    let pct = zoom * 100.0;
    let step = if zoom_in {
        ZOOM_STEPS.iter().copied().find(|s| *s > pct * 1.001)
    } else {
        ZOOM_STEPS.iter().rev().copied().find(|s| *s < pct * 0.999)
    };
    step.map_or(zoom, |s| s / 100.0)
}

fn doc_size_pt(state: &DocState, zoom: f32, ppp: f32) -> Vec2 {
    Vec2::new(state.doc.width as f32, state.doc.height as f32) * zoom / ppp
}

/// Screen position of the document's top-left corner, in points.
pub fn origin(state: &DocState, ppp: f32) -> Pos2 {
    let v = &state.view;
    v.viewport.center() + v.offset - doc_size_pt(state, v.zoom, ppp) / 2.0
}

/// Zooms around `anchor` (a screen point), keeping the document point under it fixed.
pub fn zoom_at(state: &mut DocState, new_zoom: f32, anchor: Pos2, ppp: f32) {
    let new_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    let doc_pt = (anchor - origin(state, ppp)) * ppp / state.view.zoom;
    let new_origin = anchor - doc_pt * new_zoom / ppp;
    let new_center = new_origin + doc_size_pt(state, new_zoom, ppp) / 2.0;
    state.view.offset = new_center - state.view.viewport.center();
    state.view.zoom = new_zoom;
}

pub fn zoom_step(state: &mut DocState, zoom_in: bool, ppp: f32) {
    let z = next_zoom_step(state.view.zoom, zoom_in);
    zoom_at(state, z, state.view.viewport.center(), ppp);
}

/// View > Fit on Screen.
pub fn fit_on_screen(state: &mut DocState, ppp: f32) {
    let avail = state.view.viewport.size() * ppp;
    let z = (avail.x / state.doc.width as f32).min(avail.y / state.doc.height as f32);
    state.view.zoom = z.clamp(MIN_ZOOM, MAX_ZOOM);
    state.view.offset = Vec2::ZERO;
}

/// The Hand and Zoom tools' Fill Screen: the zoom at which the document
/// covers the whole window.
pub fn fill_screen(state: &mut DocState, ppp: f32) {
    let avail = state.view.viewport.size() * ppp;
    let z = (avail.x / state.doc.width as f32).max(avail.y / state.doc.height as f32);
    state.view.zoom = z.clamp(MIN_ZOOM, MAX_ZOOM);
    state.view.offset = Vec2::ZERO;
}

/// View > 100%.
pub fn actual_pixels(state: &mut DocState, _ppp: f32) {
    state.view.zoom = 1.0;
    state.view.offset = Vec2::ZERO;
}

/// View > 200% and View > Print Size: a fixed zoom around the window's
/// center.
pub fn zoom_to(state: &mut DocState, zoom: f32, ppp: f32) {
    zoom_at(state, zoom, state.view.viewport.center(), ppp);
}

/// Scrolls so the document point `p` is at the window's center (the
/// Navigator).
pub fn center_on(state: &mut DocState, p: Pos2, ppp: f32) {
    let zoom = state.view.zoom;
    state.view.offset = doc_size_pt(state, zoom, ppp) / 2.0 - p.to_vec2() * zoom / ppp;
}

/// View > Print Size: one inch of the document (its resolution in pixels)
/// as 72 points on screen.
pub fn print_size(state: &mut DocState, ppp: f32) {
    let zoom = 72.0 / state.doc.resolution * ppp;
    zoom_to(state, zoom, ppp);
}

/// View > Fit Layer(s) on Screen: the active layer's pixels fill the window
/// and are centered in it. Nothing happens for an empty layer.
pub fn fit_layers(state: &mut DocState, ppp: f32) {
    let Some(layer) = state.doc.active_layer.and_then(|id| state.doc.layer(id)) else {
        return;
    };
    let Some(image) = layer.image() else {
        return;
    };
    let (w, h) = (state.doc.width, state.doc.height);
    let mut b: Option<(u32, u32, u32, u32)> = None;
    for y in 0..h {
        for x in 0..w {
            if image.pixel(x, y)[3] > 0 {
                b = Some(match b {
                    None => (x, y, x + 1, y + 1),
                    Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1)),
                });
            }
        }
    }
    let Some((x0, y0, x1, y1)) = b else {
        return;
    };
    let avail = state.view.viewport.size() * ppp;
    let z = (avail.x / (x1 - x0) as f32).min(avail.y / (y1 - y0) as f32);
    state.view.zoom = z.clamp(MIN_ZOOM, MAX_ZOOM);
    // Put the layer's center at the window's center
    let center = Vec2::new((x0 + x1) as f32, (y0 + y1) as f32) / 2.0;
    state.view.offset =
        doc_size_pt(state, state.view.zoom, ppp) / 2.0 - center * state.view.zoom / ppp;
}

pub fn show(ui: &mut Ui, app: &mut AppState, id: DocId) {
    let tool = app.tool;
    let paint = app.paint_options(tool).copied();
    let (foreground, background) = (app.foreground, app.background);
    let bucket = app.bucket;
    let view_options = app.view;
    let retouch = app.retouch;
    let shape_options = app.shape;
    let move_options = app.move_options;
    let type_options = app.type_options;
    let crop_options = app.crop_options.clone();
    // The Brush's and Pencil's Mode, and the Eraser's (Brush, Pencil, Block)
    let paint_mode = crate::options_tools::paint_mode(app, tool);
    let eraser_mode = crate::options_tools::eraser_mode(app);
    // The Zoom tool's Zoom Out button swaps what a click and Alt-click do
    let zoom_out = tool == Tool::Zoom && app.flag("zoom.out", false);
    let mut straightened = false;
    let mut paint_error = None;
    let Some(state) = app.docs.get_mut(&id) else {
        return;
    };
    let ppp = ui.ctx().pixels_per_point();
    let full = ui.max_rect();
    ui.painter().rect_filled(full, 0, color::PASTEBOARD);

    // Photoshop's document window: canvas, a vertical scrollbar column on
    // the right, and the status bar (with the horizontal scrollbar) below
    let status_rect = Rect::from_min_max(
        Pos2::new(full.left(), full.bottom() - size::STATUS_BAR),
        full.max,
    );
    let vscroll_rect = Rect::from_min_max(
        Pos2::new(full.right() - VSCROLL_W, full.top()),
        Pos2::new(full.right(), status_rect.top()),
    );
    let window_rect =
        Rect::from_min_max(full.min, Pos2::new(vscroll_rect.left(), status_rect.top()));
    // Rulers take a strip along the top and left of the window
    let (canvas_rect, rulers) = if view_options.rulers {
        let (top, left, canvas) = crate::rulers::layout(window_rect);
        (canvas, Some((top, left)))
    } else {
        (window_rect, None)
    };

    // Layout may not be settled in the first frames; pick the initial zoom only
    // once the viewport size is the same for two consecutive frames
    let stable = state.view.viewport == canvas_rect;
    state.view.viewport = canvas_rect;
    if !state.view.initialized && !stable {
        ui.ctx().request_repaint();
    }
    if !state.view.initialized && stable && canvas_rect.width() > 1.0 {
        state.view.initialized = true;
        let fits = state.doc.width as f32 <= canvas_rect.width() * ppp
            && state.doc.height as f32 <= canvas_rect.height() * ppp;
        if fits {
            actual_pixels(state, ppp);
        } else {
            fit_on_screen(state, ppp);
        }
    }

    let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());
    // Dragging out of a ruler makes a guide
    if let Some((top, left)) = rulers {
        for (rect, vertical) in [(top, false), (left, true)] {
            let r = ui.interact(rect, ui.id().with(("ruler", vertical)), Sense::drag());
            if r.drag_started()
                && let Some(p) = r.interact_pointer_pos()
            {
                crate::rulers::start_new(state, vertical, p, ppp);
            }
            if r.hovered() {
                ui.ctx().set_cursor_icon(if vertical {
                    CursorIcon::ResizeColumn
                } else {
                    CursorIcon::ResizeRow
                });
            }
        }
    }
    // With the Move tool (or Cmd held), guides can be grabbed
    let guides_live = view_options.guides_visible() && !view_options.lock_guides;
    let cmd = ui.input(|i| i.modifiers.command);
    let guide_hover = (guides_live && (tool == Tool::Move || cmd))
        .then(|| response.hover_pos())
        .flatten()
        .and_then(|p| crate::rulers::guide_at(state, p, ppp));
    if guides_live
        && (tool == Tool::Move || cmd)
        && response.drag_started_by(PointerButton::Primary)
        && let Some(i) = ui
            .input(|i| i.pointer.press_origin())
            .and_then(|p| crate::rulers::guide_at(state, p, ppp))
    {
        crate::rulers::start_move(state, i);
    }
    let (space, alt, zoom_delta, scroll) = ui.input(|i| {
        (
            i.key_down(Key::Space),
            i.modifiers.alt,
            i.zoom_delta(),
            i.smooth_scroll_delta,
        )
    });
    // Show Transform Controls: dragging a handle of the box starts Free
    // Transform, which takes the drag from here on
    let controls = tool == Tool::Move
        && move_options.show_transform_controls
        && state.free_transform.is_none()
        && state.move_drag.is_none()
        && state.guide_drag.is_none();
    let controls_bounds = if controls {
        state.transform_controls_bounds()
    } else {
        None
    };
    if let Some(bounds) = controls_bounds
        && !space
        && response.drag_started_by(PointerButton::Primary)
        && ui
            .input(|i| i.pointer.press_origin())
            .is_some_and(|p| crate::free_transform::controls_handle_at(state, bounds, p, ppp))
    {
        let _ = crate::free_transform::start(state);
    }
    // Auto-Select (inverted while Cmd is held) picks the layer under the
    // pointer when the button goes down, except on a transform handle
    let on_handle = |state: &DocState, p: Pos2| {
        controls_bounds.is_some_and(|b| crate::free_transform::controls_handle_at(state, b, p, ppp))
    };
    if tool == Tool::Move
        && !space
        && state.free_transform.is_none()
        && state.guide_drag.is_none()
        && move_options.auto_select != cmd
        && response.is_pointer_button_down_on()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
        && !on_handle(state, p)
    {
        let d = to_doc(state, p, ppp);
        if d.x >= 0.0
            && d.y >= 0.0
            && let Some(id) = state.doc.layer_at(d.x as u32, d.y as u32)
        {
            state.doc.active_layer = Some(id);
        }
    }
    let hover = response.hover_pos();
    // For the Info panel
    state.pointer = hover.map(|p| to_doc(state, p, ppp));

    // Pinch / Cmd+scroll zooms; plain scrolling pans
    if let Some(p) = hover {
        if zoom_delta != 1.0 {
            let z = state.view.zoom * zoom_delta;
            zoom_at(state, z, p, ppp);
        }
        if scroll != Vec2::ZERO {
            state.view.offset += scroll;
        }
    }

    let panning = space || tool == Tool::Hand;
    let middle_drag = response.dragged_by(PointerButton::Middle);
    if state.guide_drag.is_some() {
        crate::rulers::drag(ui, state, canvas_rect, ppp);
    } else if (panning && response.dragged_by(PointerButton::Primary)) || middle_drag {
        state.view.offset += response.drag_delta();
    } else if !space && state.free_transform.is_some() {
        let [r, g, b, _] = background.to_rgba8();
        let outcome = crate::free_transform::input(ui, &response, state, [r, g, b], ppp);
        response.context_menu(|ui| {
            if let Some(t) = &mut state.free_transform {
                crate::free_transform::context_menu(ui, t);
            }
        });
        if let Some(crate::free_transform::Outcome::Committed(m)) = outcome {
            app.last_transform = Some(m);
        }
    } else if !space {
        match tool {
            Tool::Zoom if response.clicked() => {
                if let Some(p) = response.interact_pointer_pos() {
                    let z = next_zoom_step(state.view.zoom, alt == zoom_out);
                    zoom_at(state, z, p, ppp);
                }
            }
            // Alt-click picks the background color
            Tool::Eyedropper if response.is_pointer_button_down_on() => {
                if let Some(p) = response.interact_pointer_pos() {
                    let d = to_doc(state, p, ppp);
                    let o = app.eyedropper;
                    if d.x >= 0.0
                        && d.y >= 0.0
                        && let Some(c) =
                            state.sample_average(d.x as u32, d.y as u32, o.size, o.all_layers)
                    {
                        if alt {
                            app.background = c;
                        } else {
                            app.foreground = c;
                        }
                    }
                }
            }
            Tool::MagicWand if response.clicked() => {
                if let Some(p) = response.interact_pointer_pos() {
                    let d = to_doc(state, p, ppp);
                    let mods = ui.input(|i| i.modifiers);
                    let has_selection = state.doc.selection().is_some();
                    let (op, _, _) = selection_op(mods, has_selection, app.wand.mode);
                    if d.x >= 0.0
                        && d.y >= 0.0
                        && let Some(shape) = op_core::fill::magic_wand(
                            &state.doc,
                            d.x as u32,
                            d.y as u32,
                            &app.wand.region,
                        )
                    {
                        let combined =
                            op_core::Selection::combine(state.doc.selection(), shape, op);
                        state.doc.set_selection(Some(combined));
                        state.record("Magic Wand");
                    }
                }
            }
            Tool::Crop => {
                straightened =
                    crate::crop_tool::input(ui, &response, state, &crop_options, background, ppp);
            }
            Tool::HorizontalType => {
                let [r, g, b, _] = foreground.to_rgba8();
                crate::type_tool::input(ui, &response, state, type_options, [r, g, b], ppp);
            }
            Tool::Rectangle | Tool::Ellipse | Tool::Triangle | Tool::Polygon | Tool::Line => {
                let [r, g, b, _] = foreground.to_rgba8();
                shape_input(ui, &response, state, tool, shape_options, [r, g, b], ppp);
            }
            Tool::Gradient => {
                let [r0, g0, b0, _] = foreground.to_rgba8();
                let [r1, g1, b1, _] = background.to_rgba8();
                let colors = ([r0, g0, b0], [r1, g1, b1]);
                paint_error = gradient_input(ui, &response, state, colors, app.gradient, ppp);
            }
            Tool::Lasso | Tool::PolygonalLasso => {
                let options = (
                    app.marquee.mode,
                    app.marquee.feather,
                    app.marquee.anti_alias,
                );
                lasso_input(ui, &response, state, tool, options, ppp);
            }
            Tool::Move => {
                let [r, g, b, _] = background.to_rgba8();
                paint_error = move_input(ui, &response, state, [r, g, b], ppp);
            }
            Tool::PaintBucket if response.clicked() => {
                if let Some(p) = response.interact_pointer_pos() {
                    let d = to_doc(state, p, ppp);
                    if d.x >= 0.0 && d.y >= 0.0 {
                        let [r, g, b, _] = foreground.to_rgba8();
                        match op_core::fill::bucket(
                            &mut state.doc,
                            d.x as u32,
                            d.y as u32,
                            [r, g, b],
                            bucket,
                        ) {
                            Ok(true) => state.record("Paint Bucket"),
                            Ok(false) => {}
                            Err(e) => paint_error = Some(e.message("Paint Bucket")),
                        }
                    }
                }
            }
            // Alt-click with the Clone Stamp picks its source point
            Tool::CloneStamp
                if alt && response.is_pointer_button_down_on() && state.stroke.is_none() =>
            {
                if ui.input(|i| i.pointer.primary_pressed())
                    && let Some(p) = response.interact_pointer_pos()
                {
                    state.clone_source = Some(to_doc(state, p, ppp));
                    state.clone_offset = None;
                    state.picking_clone_source = true;
                }
            }
            // The press that picked the source doesn't paint, even if Alt
            // is let go first
            Tool::CloneStamp if state.picking_clone_source => {
                if !ui.input(|i| i.pointer.primary_down()) {
                    state.picking_clone_source = false;
                }
            }
            Tool::Brush
            | Tool::Pencil
            | Tool::Eraser
            | Tool::Dodge
            | Tool::Burn
            | Tool::Sponge
            | Tool::Blur
            | Tool::Sharpen
            | Tool::CloneStamp
            | Tool::HistoryBrush => {
                if let Some(opts) = paint {
                    let settings = StrokeSettings {
                        opts,
                        colors: (foreground, background),
                        retouch,
                        mode: paint_mode,
                        eraser_mode,
                    };
                    paint_error = paint_input(ui, &response, state, tool, settings, ppp);
                }
            }
            Tool::RectangularMarquee
            | Tool::EllipticalMarquee
            | Tool::SingleRowMarquee
            | Tool::SingleColumnMarquee => {
                let options = (
                    app.marquee.mode,
                    app.marquee.feather,
                    app.marquee.anti_alias,
                );
                marquee_input(ui, &response, state, tool, options, ppp);
            }
            _ => {}
        }
    }

    if let Some(message) = paint_error {
        app.alert = Some(message);
    }
    if straightened {
        app.crop_options.straightening = false;
    }
    let state = app.docs.get_mut(&id).unwrap();
    clamp_offset(state, ppp);
    if response.hovered() {
        let cursor = if panning || middle_drag {
            if response.dragged() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Grab
            }
        } else if let Some(i) = guide_hover {
            crate::rulers::guide_cursor(state, i)
        } else if state.free_transform.is_some() {
            let p = response.hover_pos().unwrap_or_default();
            crate::free_transform::cursor(state, p, ppp)
        } else if tool == Tool::Crop {
            let p = response.hover_pos().unwrap_or_default();
            crate::crop_tool::cursor(state, p, ppp)
        } else {
            match tool {
                Tool::Zoom if alt != zoom_out => CursorIcon::ZoomOut,
                Tool::Zoom => CursorIcon::ZoomIn,
                // Painting tools draw their own brush outline instead
                _ if paint.is_some_and(|p| p.size * state.view.zoom / ppp >= 4.0) => {
                    CursorIcon::None
                }
                Tool::Eyedropper
                | Tool::Rectangle
                | Tool::Ellipse
                | Tool::Triangle
                | Tool::Polygon
                | Tool::Line
                | Tool::Gradient
                | Tool::MagicWand
                | Tool::Lasso
                | Tool::PolygonalLasso
                | Tool::PaintBucket
                | Tool::Brush
                | Tool::Pencil
                | Tool::Eraser
                | Tool::Dodge
                | Tool::Burn
                | Tool::Sponge
                | Tool::Blur
                | Tool::Sharpen
                | Tool::CloneStamp
                | Tool::HistoryBrush
                | Tool::RectangularMarquee
                | Tool::EllipticalMarquee
                | Tool::SingleRowMarquee
                | Tool::SingleColumnMarquee => CursorIcon::Crosshair,
                Tool::Move => CursorIcon::Move,
                Tool::HorizontalType => CursorIcon::Text,
                _ => CursorIcon::Default,
            }
        };
        ui.ctx().set_cursor_icon(cursor);
    }

    // Snap to physical pixels so 100% stays crisp
    let origin = origin(state, ppp);
    let origin = Pos2::new(
        (origin.x * ppp).round() / ppp,
        (origin.y * ppp).round() / ppp,
    );
    let image = state.canvas_image();
    let view = op_render::CanvasView {
        origin: [origin.x, origin.y],
        zoom: state.view.zoom,
        shield: crate::crop_tool::shield(state, &crop_options),
        rotation: crate::crop_tool::rotation(state),
        pixel_grid: true,
    };
    ui.painter()
        .with_clip_rect(canvas_rect)
        .add(op_render::paint_callback(canvas_rect, image, view));

    if view_options.grid_visible() {
        crate::rulers::draw_grid(ui, state, canvas_rect, ppp);
    }
    // Extras off hides the selection edges (the selection stays)
    if view_options.extras {
        draw_selection(ui, state, canvas_rect, ppp, tool, shape_options);
    }
    if view_options.guides_visible() || state.guide_drag.is_some() {
        crate::rulers::draw_guides(ui, state, canvas_rect, ppp);
    }
    if rulers.is_some() {
        let pointer = ui.input(|i| i.pointer.hover_pos());
        crate::rulers::draw_rulers(ui, state, window_rect, pointer, ppp);
    }
    crate::free_transform::draw(ui, state, canvas_rect, ppp);
    if let Some(bounds) = controls_bounds
        && view_options.extras
    {
        crate::free_transform::draw_controls(ui, state, bounds, canvas_rect, ppp);
    }
    // The crop box belongs to the Crop tool; picking another tool drops it
    if tool != Tool::Crop {
        state.crop = None;
    }
    // Picking another tool keeps the text being typed
    if tool != Tool::HorizontalType && state.text_edit.is_some() {
        crate::type_tool::commit(state);
    }
    crate::type_tool::draw_caret(ui, state, type_options, canvas_rect, ppp);
    crate::crop_tool::draw(ui, state, &crop_options, canvas_rect, ppp);
    let transforming = state.free_transform.is_some();
    if let (Some(opts), Some(p), false) = (paint, response.hover_pos(), transforming) {
        brush_cursor(ui, canvas_rect, p, opts.size * state.view.zoom / ppp);
    }
    status_bar(ui, state, status_rect, ppp);
    vertical_scrollbar(ui, state, vscroll_rect, ppp);
}

/// Screen point → document pixel coordinates (not clamped).
pub(crate) fn to_doc(state: &DocState, p: Pos2, ppp: f32) -> Pos2 {
    let d = (p - origin(state, ppp)) * ppp / state.view.zoom;
    Pos2::new(d.x, d.y)
}

/// The part of the document shown in the window, in document pixels
/// (x0, y0, x1, y1; may extend past the canvas).
pub fn visible_rect(state: &DocState, ppp: f32) -> [f32; 4] {
    let min = to_doc(state, state.view.viewport.min, ppp);
    let max = to_doc(state, state.view.viewport.max, ppp);
    [min.x, min.y, max.x, max.y]
}

/// Document pixel → screen point.
pub(crate) fn to_screen(state: &DocState, d: Pos2, ppp: f32) -> Pos2 {
    origin(state, ppp) + d.to_vec2() * state.view.zoom / ppp
}

/// The marquee rectangle from a drag, with Photoshop's modifiers: Shift
/// constrains to a square/circle, Alt draws from the center. Rectangles snap
/// to whole pixels.
fn marquee_rect(
    drag: &crate::state::MarqueeDrag,
    shift: bool,
    alt: bool,
) -> op_core::selection::Rect {
    let constrain = shift && !drag.shift_for_op;
    let from_center = alt && !drag.alt_for_op;
    let mut d = drag.current - drag.start;
    if constrain {
        let m = d.x.abs().max(d.y.abs());
        d = Vec2::new(m.copysign(d.x), m.copysign(d.y));
    }
    let (a, b) = if from_center {
        (drag.start - d, drag.start + d)
    } else {
        (drag.start, drag.start + d)
    };
    op_core::selection::Rect::new(a.x.round(), a.y.round(), b.x.round(), b.y.round())
}

/// Marquee tools: drag to select, click to deselect; single row/column
/// marquees select a 1-pixel line on click.
fn shape_kind(tool: Tool, options: crate::state::ShapeOptions) -> op_core::shape::ShapeKind {
    use op_core::shape::ShapeKind;
    match tool {
        Tool::Ellipse => ShapeKind::Ellipse,
        Tool::Triangle => ShapeKind::Triangle,
        Tool::Polygon => ShapeKind::Polygon(options.sides),
        Tool::Line => ShapeKind::Line,
        _ => ShapeKind::Rectangle,
    }
}

/// The two points a shape drag describes, with Photoshop's modifiers:
/// Shift makes box shapes square (and snaps the Line to 45°), Alt draws
/// them from the center.
fn shape_points(tool: Tool, start: Pos2, end: Pos2, mods: egui::Modifiers) -> (Pos2, Pos2) {
    if tool == Tool::Line {
        return (start, if mods.shift { snap_45(start, end) } else { end });
    }
    let mut d = end - start;
    if mods.shift {
        let side = d.x.abs().max(d.y.abs());
        d = Vec2::new(side.copysign(d.x), side.copysign(d.y));
    }
    if mods.alt {
        (start - d, start + d)
    } else {
        (start, start + d)
    }
}

/// Shape tools: drag out the shape; on release it becomes a new layer
/// filled with the foreground color ("Rectangle 1", ...).
fn shape_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    tool: Tool,
    options: crate::state::ShapeOptions,
    color: [u8; 3],
    ppp: f32,
) {
    if response.drag_started_by(PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        let start = to_doc(state, p, ppp);
        state.shape_drag = Some((start, start));
    }
    let Some((start, _)) = state.shape_drag else {
        return;
    };
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        state.shape_drag = Some((start, to_doc(state, p, ppp)));
    }
    if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
        let (s, e) = state.shape_drag.take().expect("checked");
        let mods = ui.input(|i| i.modifiers);
        let (a, b) = shape_points(tool, s, e, mods);
        let kind = shape_kind(tool, options);
        if op_core::shape::add_shape_layer(
            &mut state.doc,
            kind,
            (a.x, a.y),
            (b.x, b.y),
            options.weight,
            color,
        )
        .is_some()
        {
            state.record(&format!("{} Tool", kind.layer_name()));
        }
    } else {
        ui.ctx().request_repaint();
    }
}

/// Snaps the end of a drag from `a` to a multiple of 45°, keeping its
/// length (Shift while dragging a gradient).
fn snap_45(a: Pos2, b: Pos2) -> Pos2 {
    let d = b - a;
    let step = std::f32::consts::FRAC_PI_4;
    let angle = (d.y.atan2(d.x) / step).round() * step;
    a + Vec2::new(angle.cos(), angle.sin()) * d.length()
}

/// Gradient tool: drag from the start color's point to the end color's;
/// Shift constrains the direction to 45° steps. Paints once released.
fn gradient_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    colors: ([u8; 3], [u8; 3]),
    options: op_core::gradient::GradientOptions,
    ppp: f32,
) -> Option<String> {
    if response.drag_started_by(PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        let start = to_doc(state, p, ppp);
        state.gradient_drag = Some((start, start));
    }
    let (start, _) = state.gradient_drag?;
    let shift = ui.input(|i| i.modifiers.shift);
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let p = to_doc(state, p, ppp);
        let end = if shift { snap_45(start, p) } else { p };
        state.gradient_drag = Some((start, end));
    }
    if !(response.drag_stopped() || !ui.input(|i| i.pointer.primary_down())) {
        ui.ctx().request_repaint();
        return None;
    }
    let (start, end) = state.gradient_drag.take()?;
    if start == end {
        return None;
    }
    let result = op_core::gradient::gradient(
        &mut state.doc,
        (start.x, start.y),
        (end.x, end.y),
        colors,
        options,
    );
    match result {
        Ok(()) => {
            state.record("Gradient");
            None
        }
        Err(e) => Some(e.message("Gradient")),
    }
}

/// The combine mode for a new selection shape: with a selection, Shift adds,
/// Alt subtracts and both intersect; otherwise the options bar's mode. Also
/// returns whether Shift and Alt were used for the mode (and so don't
/// constrain the shape).
fn selection_op(
    mods: egui::Modifiers,
    has_selection: bool,
    mode: crate::state::SelectionMode,
) -> (op_core::SelectionOp, bool, bool) {
    use op_core::SelectionOp;
    match (mods.shift, mods.alt) {
        (true, true) if has_selection => (SelectionOp::Intersect, true, true),
        (true, false) if has_selection => (SelectionOp::Add, true, false),
        (false, true) if has_selection => (SelectionOp::Subtract, false, true),
        _ => (mode.op(), false, false),
    }
}

/// Lasso (drag a freehand outline) and Polygonal Lasso (click corners;
/// double-click, Enter or clicking the first point closes; Backspace
/// removes the last corner; Escape cancels).
fn lasso_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    tool: Tool,
    (mode, feather, anti_alias): (crate::state::SelectionMode, f32, bool),
    ppp: f32,
) {
    use crate::state::LassoPath;
    let mods = ui.input(|i| i.modifiers);
    let has_selection = state.doc.selection().is_some();
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    let mut close = false;

    if tool == Tool::Lasso {
        if response.drag_started_by(PointerButton::Primary)
            && let Some(p) = ui.input(|i| i.pointer.press_origin())
        {
            let (op, _, _) = selection_op(mods, has_selection, mode);
            state.lasso = Some(LassoPath {
                points: vec![to_doc(state, p, ppp)],
                op,
                polygonal: false,
            });
        }
        if let Some(lasso) = &mut state.lasso {
            if let Some(p) = pointer
                && lasso
                    .points
                    .last()
                    .is_none_or(|l| l.distance(p) * state.view.zoom >= ppp)
            {
                lasso.points.push(p);
            }
            if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
                close = true;
            } else {
                ui.ctx().request_repaint();
            }
        } else if response.clicked() && has_selection && !mods.shift && !mods.alt {
            state.doc.set_selection(None);
            state.record("Deselect");
        }
    } else {
        let typing = ui.ctx().egui_wants_keyboard_input();
        let (enter, escape, backspace) = ui.input_mut(|i| {
            if typing {
                return (false, false, false);
            }
            (
                i.consume_key(egui::Modifiers::NONE, Key::Enter),
                i.consume_key(egui::Modifiers::NONE, Key::Escape),
                i.consume_key(egui::Modifiers::NONE, Key::Backspace)
                    || i.consume_key(egui::Modifiers::NONE, Key::Delete),
            )
        });
        if escape {
            state.lasso = None;
        }
        if let Some(lasso) = &mut state.lasso {
            if backspace {
                lasso.points.pop();
                if lasso.points.is_empty() {
                    state.lasso = None;
                }
            } else if enter || response.double_clicked() {
                close = true;
            } else if response.clicked()
                && let Some(p) = pointer
            {
                // Clicking near the first corner (within 5 screen points) closes
                let first = lasso.points[0];
                if lasso.points.len() > 2 && first.distance(p) * state.view.zoom / ppp <= 5.0 {
                    close = true;
                } else {
                    lasso.points.push(p);
                }
            }
            ui.ctx().request_repaint();
        } else if response.clicked()
            && let Some(p) = pointer
        {
            let (op, _, _) = selection_op(mods, has_selection, mode);
            state.lasso = Some(LassoPath {
                points: vec![p],
                op,
                polygonal: true,
            });
        }
    }

    if close && let Some(lasso) = state.lasso.take() {
        let (w, h) = (state.doc.width, state.doc.height);
        let points: Vec<(f32, f32)> = lasso.points.iter().map(|p| (p.x, p.y)).collect();
        let mut shape = op_core::Selection::polygon(w, h, &points, anti_alias);
        if shape.is_empty() {
            return;
        }
        if feather > 0.0 {
            shape = shape.feather(feather);
        }
        let combined = op_core::Selection::combine(state.doc.selection(), shape, lasso.op);
        state.doc.set_selection(Some(combined));
        state.record(if lasso.polygonal {
            "Polygonal Lasso"
        } else {
            "Lasso"
        });
    }
}

fn marquee_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    tool: Tool,
    (mode, feather, anti_alias): (crate::state::SelectionMode, f32, bool),
    ppp: f32,
) {
    use op_core::Selection;
    let mods = ui.input(|i| i.modifiers);
    let has_selection = state.doc.selection().is_some();
    // With a selection, Shift/Alt at mouse-down pick the combine mode
    let op_from_mods = || selection_op(mods, has_selection, mode);
    let (w, h) = (state.doc.width, state.doc.height);

    if matches!(tool, Tool::SingleRowMarquee | Tool::SingleColumnMarquee) {
        if response.clicked()
            && let Some(p) = response.interact_pointer_pos()
        {
            let d = to_doc(state, p, ppp);
            let (op, _, _) = op_from_mods();
            let (rect, name) = if tool == Tool::SingleRowMarquee {
                (
                    op_core::selection::Rect::new(0.0, d.y.floor(), w as f32, d.y.floor() + 1.0),
                    "Single Row Marquee",
                )
            } else {
                (
                    op_core::selection::Rect::new(d.x.floor(), 0.0, d.x.floor() + 1.0, h as f32),
                    "Single Column Marquee",
                )
            };
            let shape = Selection::rect(w, h, rect);
            let combined = Selection::combine(state.doc.selection(), shape, op);
            state.doc.set_selection(Some(combined));
            state.record(name);
        }
        return;
    }

    // egui reports a drag only after the pointer moved a little, so the
    // marquee starts where the button went down
    if response.drag_started_by(PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        let (op, shift_for_op, alt_for_op) = op_from_mods();
        let start = to_doc(state, p, ppp);
        state.marquee_drag = Some(crate::state::MarqueeDrag {
            start,
            current: start,
            op,
            shift_for_op,
            alt_for_op,
        });
    }
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    if let Some(drag) = &mut state.marquee_drag {
        if let Some(p) = pointer {
            drag.current = p;
        }
        if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
            let drag = state.marquee_drag.take().unwrap();
            let rect = marquee_rect(&drag, mods.shift, mods.alt);
            if !rect.is_empty() {
                let mut shape = if tool == Tool::EllipticalMarquee {
                    Selection::ellipse(w, h, rect, anti_alias)
                } else {
                    Selection::rect(w, h, rect)
                };
                if feather > 0.0 {
                    shape = shape.feather(feather);
                }
                let combined = Selection::combine(state.doc.selection(), shape, drag.op);
                state.doc.set_selection(Some(combined));
                let name = if tool == Tool::EllipticalMarquee {
                    "Elliptical Marquee"
                } else {
                    "Rectangular Marquee"
                };
                state.record(name);
            }
        } else {
            ui.ctx().request_repaint();
        }
        return;
    }
    // A click without a drag deselects, as in Photoshop
    if response.clicked() && has_selection && !mods.shift && !mods.alt {
        state.doc.set_selection(None);
        state.record("Deselect");
    }
}

/// Move tool: drag to move the active layer (or its selected pixels); the
/// layer is recomputed from its original pixels at every step. One "Move"
/// history state when released, if anything moved.
fn move_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    background: [u8; 3],
    ppp: f32,
) -> Option<String> {
    use op_core::move_tool::Move;
    if response.drag_started_by(PointerButton::Primary) {
        let start = ui
            .input(|i| i.pointer.press_origin())
            .map(|p| to_doc(state, p, ppp))?;
        match Move::begin(&state.doc, background) {
            Ok(m) => state.move_drag = Some((m, start)),
            Err(e) => return Some(e.message().to_owned()),
        }
    }
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    let Some((m, start)) = &state.move_drag else {
        return None;
    };
    let delta = pointer.map_or(egui::Vec2::ZERO, |p| p - *start);
    let (dx, dy) = (delta.x.round() as i64, delta.y.round() as i64);
    m.apply(&mut state.doc, dx, dy);
    if !ui.input(|i| i.pointer.primary_down()) {
        state.move_drag = None;
        if dx != 0 || dy != 0 {
            state.record("Move");
        }
    } else {
        ui.ctx().request_repaint();
    }
    None
}

/// Arrow keys with the Move tool: nudge by 1 pixel, 10 with Shift
/// ("Nudge" in the history, as in Photoshop).
pub fn nudge(state: &mut DocState, dx: i64, dy: i64, background: [u8; 3]) -> Result<(), String> {
    let m = op_core::move_tool::Move::begin(&state.doc, background)
        .map_err(|e| e.message().to_owned())?;
    m.apply(&mut state.doc, dx, dy);
    state.record("Nudge");
    Ok(())
}

/// Brush, Pencil and Eraser: press to start a stroke, drag to continue,
/// release to finish (one history state). Shift-click strokes a straight
/// line from where the last stroke ended. Returns Photoshop's alert text when
/// the layer can't be painted.
/// The tool's name in Photoshop's "Could not use the …" alerts, and its
/// history state.
fn stroke_names(tool: Tool) -> (&'static str, &'static str) {
    match tool {
        Tool::Pencil => ("pencil", "Pencil"),
        Tool::Eraser => ("eraser", "Eraser"),
        Tool::Dodge => ("dodge tool", "Dodge Tool"),
        Tool::Burn => ("burn tool", "Burn Tool"),
        Tool::Sponge => ("sponge tool", "Sponge Tool"),
        Tool::Blur => ("blur tool", "Blur Tool"),
        Tool::Sharpen => ("sharpen tool", "Sharpen Tool"),
        Tool::CloneStamp => ("clone stamp", "Clone Stamp"),
        Tool::HistoryBrush => ("history brush", "History Brush"),
        _ => ("brush tool", "Brush Tool"),
    }
}

/// What a stroke of `tool` does, decided when the button goes down at
/// `start` (document pixels). Errors are Photoshop's alerts.
fn stroke_kind(
    tool: Tool,
    state: &mut DocState,
    start: Pos2,
    (foreground, background): (op_core::Color, op_core::Color),
    retouch: crate::state::RetouchOptions,
) -> Result<op_core::paint::StrokeKind, String> {
    use op_core::paint::StrokeKind;
    let rgb = |c: op_core::Color| {
        let [r, g, b, _] = c.to_rgba8();
        [r, g, b]
    };
    let active_image = |state: &DocState| {
        let layer = state.doc.active_layer.and_then(|id| state.doc.layer(id))?;
        layer.image().cloned()
    };
    Ok(match tool {
        Tool::Eraser => StrokeKind::Erase {
            background: rgb(background),
        },
        Tool::Dodge => StrokeKind::Dodge(retouch.dodge_range),
        Tool::Burn => StrokeKind::Burn(retouch.burn_range),
        Tool::Sponge => StrokeKind::Sponge {
            saturate: retouch.sponge_saturate,
        },
        Tool::Blur => StrokeKind::Blur,
        Tool::Sharpen => StrokeKind::Sharpen,
        Tool::CloneStamp => {
            let source = state.clone_source.ok_or_else(|| {
                "Could not use the clone stamp because the area to clone has not been defined \
                 (option-click to define a source point)."
                    .to_string()
            })?;
            // Aligned keeps the offset of the first stroke after picking
            // the source; otherwise every stroke starts from the source
            let offset = match state.clone_offset {
                Some(o) if retouch.clone_aligned => o,
                _ => start - source,
            };
            state.clone_offset = Some(offset);
            let image = active_image(state).ok_or("")?;
            StrokeKind::Source {
                image,
                dx: offset.x.round() as i64,
                dy: offset.y.round() as i64,
            }
        }
        Tool::HistoryBrush => {
            // Paints from the document as it was opened (the History
            // Brush's default source)
            let id = state.doc.active_layer.ok_or("")?;
            let snapshot = state.history.snapshot(0).ok_or("")?;
            let layer = op_core::Document::snapshot_layer(snapshot, id).ok_or_else(|| {
                "Could not use the history brush because the history state does not contain \
                 a corresponding layer."
                    .to_string()
            })?;
            let image = layer.image().ok_or_else(|| {
                "Could not use the history brush because the history state does not contain \
                 a corresponding layer."
                    .to_string()
            })?;
            if (image.width(), image.height()) != (state.doc.width, state.doc.height) {
                return Err("Could not use the history brush because the history state \
                            does not contain a corresponding layer."
                    .into());
            }
            StrokeKind::Source {
                image: image.clone(),
                dx: 0,
                dy: 0,
            }
        }
        _ => StrokeKind::Paint(rgb(foreground)),
    })
}

/// Everything a stroke needs besides the document: the tool's brush, the
/// foreground and background colors, and the retouching options.
#[derive(Clone, Copy)]
struct StrokeSettings {
    opts: crate::state::PaintOptions,
    colors: (op_core::Color, op_core::Color),
    retouch: crate::state::RetouchOptions,
    mode: op_core::paint::PaintMode,
    eraser_mode: crate::options_tools::EraserMode,
}

fn paint_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    tool: Tool,
    settings: StrokeSettings,
    ppp: f32,
) -> Option<String> {
    let StrokeSettings {
        opts,
        colors,
        retouch,
        mode,
        eraser_mode,
    } = settings;
    use crate::options_tools::EraserMode;
    let eraser = (tool == Tool::Eraser).then_some(eraser_mode);
    use op_core::paint::{BrushTip, Stroke};
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    let pressed = response.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down());

    if pressed && state.stroke.is_none() {
        // The Eraser's Block is a 16 screen-pixel square at full strength
        let block = eraser == Some(EraserMode::Block);
        let tip = BrushTip {
            diameter: if block {
                16.0 / state.view.zoom.max(0.01)
            } else {
                opts.size
            },
            hardness: opts.hardness,
            aliased: tool == Tool::Pencil || eraser == Some(EraserMode::Pencil),
            square: block,
        };
        let just_pressed = ui.input(|i| i.pointer.primary_pressed());
        let start = ui
            .input(|i| i.pointer.press_origin())
            .map(|p| to_doc(state, p, ppp))
            .or(pointer)
            .unwrap_or_default();
        let kind = match stroke_kind(tool, state, start, colors, retouch) {
            Ok(kind) => kind,
            Err(message) => {
                // Only once per press, and never an empty message
                return (just_pressed && !message.is_empty()).then_some(message);
            }
        };
        let hard = tool == Tool::Pencil || eraser.is_some_and(|m| m != EraserMode::Brush);
        let flow = if hard { 1.0 } else { opts.flow };
        let opacity = if block { 1.0 } else { opts.opacity };
        let (label, _) = stroke_names(tool);
        match Stroke::begin(&state.doc, tip, kind, opacity, flow) {
            Ok(stroke) => {
                let mut stroke = if matches!(tool, Tool::Brush | Tool::Pencil) {
                    stroke.with_mode(mode)
                } else {
                    stroke
                };
                let start = ui
                    .input(|i| i.pointer.press_origin())
                    .map(|p| to_doc(state, p, ppp));
                let shift = ui.input(|i| i.modifiers.shift);
                if shift && let Some((x, y)) = state.last_paint_point {
                    stroke.add_point(&mut state.doc, x, y);
                }
                if let Some(p) = start.or(pointer) {
                    stroke.add_point(&mut state.doc, p.x, p.y);
                }
                state.stroke = Some((stroke, tool));
            }
            Err(e) => {
                // Wait for the button to be released before another attempt
                state.stroke = None;
                return response
                    .is_pointer_button_down_on()
                    .then(|| e.message(label))
                    .filter(|_| ui.input(|i| i.pointer.primary_pressed()));
            }
        }
    }

    if let Some((stroke, stroke_tool)) = &mut state.stroke {
        if let Some(p) = pointer {
            stroke.add_point(&mut state.doc, p.x, p.y);
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            let (_, name) = stroke_names(*stroke_tool);
            state.last_paint_point = stroke.last_point();
            state.stroke = None;
            state.record(name);
        } else {
            ui.ctx().request_repaint();
        }
    }
    None
}

/// Photoshop's "normal brush tip" cursor: the brush outline, drawn in white
/// over black so it shows on any background.
fn brush_cursor(ui: &Ui, canvas: Rect, center: Pos2, diameter: f32) {
    if diameter < 4.0 {
        return;
    }
    let painter = ui.painter_at(canvas);
    let r = diameter / 2.0;
    painter.circle_stroke(
        center,
        r,
        egui::Stroke::new(1.5, Color32::from_black_alpha(160)),
    );
    painter.circle_stroke(center, r, egui::Stroke::new(0.75, Color32::WHITE));
}

/// Marching ants around the selection and the marquee being dragged.
/// The selection's marching ants and the outlines of drags in progress
/// (marquee, lasso, shape, gradient).
fn draw_selection(
    ui: &Ui,
    state: &mut DocState,
    canvas: Rect,
    ppp: f32,
    tool: Tool,
    shape: crate::state::ShapeOptions,
) {
    let painter = ui.painter_at(canvas);
    let time = ui.input(|i| i.time);
    // Ants march by one dash every 1/8 s
    let phase = ((time * 8.0) as i64 % 8) as f32;
    let thin = 1.0 / ppp;
    let dash = 4.0 / ppp;
    let mut animate = false;

    // The dash phase depends on the segment's position, so the pattern runs
    // on across the many short steps of a curved outline
    let mut ants = |a: Pos2, b: Pos2| {
        painter.line_segment([a, b], egui::Stroke::new(thin, Color32::WHITE));
        let along = ((a.x + a.y) * ppp + phase).rem_euclid(8.0);
        painter.add(egui::Shape::dashed_line_with_offset(
            &[a, b],
            egui::Stroke::new(thin, Color32::BLACK),
            &[dash],
            &[dash],
            along / ppp,
        ));
        animate = true;
    };

    if let Some(segs) = state.selection_outline() {
        let visible = canvas.expand(2.0);
        for s in segs.iter() {
            let a = to_screen(state, Pos2::new(s[0] as f32, s[1] as f32), ppp);
            let b = to_screen(state, Pos2::new(s[2] as f32, s[3] as f32), ppp);
            if visible.intersects(Rect::from_two_pos(a, b)) {
                ants(a, b);
            }
        }
    }

    if let Some(drag) = state.marquee_drag {
        let mods = ui.input(|i| i.modifiers);
        let r = marquee_rect(&drag, mods.shift, mods.alt);
        let a = to_screen(state, Pos2::new(r.x0, r.y0), ppp);
        let b = to_screen(state, Pos2::new(r.x1, r.y1), ppp);
        let rect = Rect::from_two_pos(a, b);
        if tool == Tool::EllipticalMarquee {
            // Ellipse preview: a polyline approximating the ellipse
            let n = 96;
            let pts: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let t = i as f32 / n as f32 * std::f32::consts::TAU;
                    rect.center()
                        + Vec2::new(t.cos() * rect.width() / 2.0, t.sin() * rect.height() / 2.0)
                })
                .collect();
            for w in pts.windows(2) {
                ants(w[0], w[1]);
            }
        } else {
            for (a, b) in [
                (rect.left_top(), rect.right_top()),
                (rect.right_top(), rect.right_bottom()),
                (rect.right_bottom(), rect.left_bottom()),
                (rect.left_bottom(), rect.left_top()),
            ] {
                ants(a, b);
            }
        }
    }
    // A shape being dragged: its outline as a thin blue path
    if let Some((s, e)) = state.shape_drag {
        let mods = ui.input(|i| i.modifiers);
        let (a, b) = shape_points(tool, s, e, mods);
        let (a, b) = (to_screen(state, a, ppp), to_screen(state, b, ppp));
        let r = Rect::from_two_pos(a, b);
        let blue = egui::Stroke::new(1.0, Color32::from_rgb(0x2c, 0x8b, 0xe8));
        let painter = ui.painter_at(canvas);
        let outline: Vec<Pos2> = match tool {
            Tool::Line => vec![a, b],
            Tool::Ellipse => (0..=96)
                .map(|i| {
                    let t = i as f32 / 96.0 * std::f32::consts::TAU;
                    r.center() + Vec2::new(t.cos() * r.width() / 2.0, t.sin() * r.height() / 2.0)
                })
                .collect(),
            Tool::Triangle => vec![
                r.center_top(),
                r.right_bottom(),
                r.left_bottom(),
                r.center_top(),
            ],
            Tool::Polygon => {
                let n = shape.sides.max(3);
                (0..=n)
                    .map(|k| {
                        let t = -std::f32::consts::FRAC_PI_2
                            + k as f32 * std::f32::consts::TAU / n as f32;
                        r.center()
                            + Vec2::new(t.cos() * r.width() / 2.0, t.sin() * r.height() / 2.0)
                    })
                    .collect()
            }
            _ => vec![
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
                r.left_top(),
            ],
        };
        painter.add(egui::Shape::line(outline, blue));
    }
    // The gradient's direction while dragging: a line with a dot at each end
    if let Some((a, b)) = state.gradient_drag {
        let (a, b) = (to_screen(state, a, ppp), to_screen(state, b, ppp));
        let painter = ui.painter_at(canvas);
        painter.line_segment([a, b], egui::Stroke::new(2.0, Color32::BLACK));
        painter.line_segment([a, b], egui::Stroke::new(1.0, Color32::WHITE));
        for p in [a, b] {
            painter.circle(
                p,
                3.0,
                Color32::WHITE,
                egui::Stroke::new(1.0, Color32::BLACK),
            );
        }
    }
    if let Some(lasso) = &state.lasso {
        let mut pts: Vec<Pos2> = lasso
            .points
            .iter()
            .map(|&p| to_screen(state, p, ppp))
            .collect();
        // The Polygonal Lasso's next edge follows the pointer
        if lasso.polygonal
            && let Some(p) = ui.input(|i| i.pointer.hover_pos())
        {
            pts.push(p);
        }
        for w in pts.windows(2) {
            ants(w[0], w[1]);
        }
    }
    if animate {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(120));
    }
}

/// Width of the vertical scrollbar column, and the status bar's layout.
const VSCROLL_W: f32 = pt(17.0);
const ZOOM_BOX_W: f32 = pt(60.0);
const INFO_X: f32 = pt(89.0);
const CARET_X: f32 = pt(237.0);
const HSCROLL_X: f32 = pt(243.0);
const TRACK: Color32 = Color32::from_gray(0x4a);
const THUMB: Color32 = Color32::from_gray(0x69);
const THUMB_THICKNESS: f32 = pt(10.0);
const VTHUMB_THICKNESS: f32 = pt(12.0);

/// The scrollable extent along one axis: the document plus one viewport, so
/// the document's edge can be scrolled to the middle of the window. Returns
/// (visible start, visible length) as fractions of the extent.
fn scroll_fraction(doc_pt: f32, viewport: f32, offset: f32) -> (f32, f32) {
    let extent = doc_pt + viewport;
    ((doc_pt / 2.0 - offset) / extent, viewport / extent)
}

/// Keeps the document's edge from being scrolled past the middle of the
/// viewport (the same extent the scrollbars show).
fn clamp_offset(state: &mut DocState, ppp: f32) {
    let size = doc_size_pt(state, state.view.zoom, ppp);
    let o = &mut state.view.offset;
    o.x = o.x.clamp(-size.x / 2.0, size.x / 2.0);
    o.y = o.y.clamp(-size.y / 2.0, size.y / 2.0);
}

/// Draws a scrollbar thumb in `track` and returns the drag, as a change of
/// the visible start in fractions of the extent.
fn scrollbar(
    ui: &mut Ui,
    track: Rect,
    vertical: bool,
    (start, len): (f32, f32),
    thickness: f32,
    id: &str,
) -> f32 {
    let along = |r: Rect| if vertical { r.height() } else { r.width() };
    let length = along(track);
    let t0 = start.clamp(0.0, 1.0 - len.min(1.0)) * length;
    let t1 = t0 + len.min(1.0) * length;
    let thumb = if vertical {
        Rect::from_min_max(
            Pos2::new(track.center().x - thickness / 2.0, track.top() + t0),
            Pos2::new(track.center().x + thickness / 2.0, track.top() + t1),
        )
    } else {
        Rect::from_min_max(
            Pos2::new(track.left() + t0, track.center().y - thickness / 2.0),
            Pos2::new(track.left() + t1, track.center().y + thickness / 2.0),
        )
    };
    let response = ui.interact(thumb, ui.id().with(id), Sense::drag());
    ui.painter().rect_filled(
        thumb,
        egui::CornerRadius::same((thickness / 2.0) as u8),
        THUMB,
    );
    if response.dragged() && length > 0.0 {
        let d = response.drag_delta();
        (if vertical { d.y } else { d.x }) / length
    } else {
        0.0
    }
}

fn vertical_scrollbar(ui: &mut Ui, state: &mut DocState, column: Rect, ppp: f32) {
    let painter = ui.painter();
    painter.rect_filled(column, 0, TRACK);
    // 1 pt edges: the canvas border, then a lighter and a darker line
    let edge = |x: f32, c: u8| {
        let r = Rect::from_min_max(
            Pos2::new(x, column.top()),
            Pos2::new(x + pt(1.0), column.bottom()),
        );
        painter.rect_filled(r, 0, Color32::from_gray(c));
    };
    edge(column.left(), 0x2e);
    edge(column.left() + pt(1.0), 0x46);
    edge(column.right() - pt(1.0), 0x45);

    let doc = doc_size_pt(state, state.view.zoom, ppp);
    let fraction = scroll_fraction(doc.y, state.view.viewport.height(), state.view.offset.y);
    let extent = doc.y + state.view.viewport.height();
    let track = column.shrink2(Vec2::new(0.0, pt(1.0)));
    let d = scrollbar(ui, track, true, fraction, VTHUMB_THICKNESS, "vscroll");
    state.view.offset.y -= d * extent;
    clamp_offset(state, ppp);
}

/// Photoshop's status bar: zoom box, document info, the info menu caret,
/// then the horizontal scrollbar filling the rest.
fn status_bar(ui: &mut Ui, state: &mut DocState, rect: Rect, ppp: f32) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0, color::PANEL);
    let line = Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.top() + pt(1.0)));
    painter.rect_filled(line, 0, Color32::from_gray(0x44));
    let body = Rect::from_min_max(Pos2::new(rect.left(), line.bottom()), rect.max);

    let zoom_rect = Rect::from_min_size(body.min, Vec2::new(ZOOM_BOX_W, body.height()));
    painter.rect_filled(zoom_rect, 0, Color32::from_gray(0x41));
    painter.text(
        zoom_rect.center(),
        Align2::CENTER_CENTER,
        zoom_label(state.view.zoom),
        theme::body(),
        color::TEXT,
    );

    let doc = &state.doc;
    let info = format!(
        "{} px x {} px ({} ppi)",
        doc.width,
        doc.height,
        doc.resolution.round()
    );
    painter.text(
        Pos2::new(body.left() + INFO_X, body.center().y),
        Align2::LEFT_CENTER,
        info,
        theme::body(),
        color::TEXT_DIM,
    );
    painter.text(
        Pos2::new(body.left() + CARET_X, body.center().y),
        Align2::CENTER_CENTER,
        crate::icons::CARET_RIGHT,
        theme::icon(pt(9.0)),
        color::TEXT_DIM,
    );

    // The horizontal scrollbar stops where the vertical one's column starts;
    // the corner below that column stays panel-colored
    let track = Rect::from_min_max(
        Pos2::new(body.left() + HSCROLL_X, body.top()),
        Pos2::new(body.right() - VSCROLL_W, body.bottom()),
    );
    if track.width() > 0.0 {
        painter.rect_filled(track, 0, TRACK);
        let doc = doc_size_pt(state, state.view.zoom, ppp);
        let fraction = scroll_fraction(doc.x, state.view.viewport.width(), state.view.offset.x);
        let extent = doc.x + state.view.viewport.width();
        let d = scrollbar(ui, track, false, fraction, THUMB_THICKNESS, "hscroll");
        state.view.offset.x -= d * extent;
        clamp_offset(state, ppp);
    }
}
