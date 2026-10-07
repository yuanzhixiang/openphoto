//! Document window: the canvas (zoom/pan/tool input) and the status bar.

use egui::{Align2, Color32, CursorIcon, Key, PointerButton, Pos2, Rect, Sense, Ui, Vec2};
use op_core::DocId;
use op_tools::Tool;

use crate::state::{AppState, DocState};
use crate::theme::{color, pt, size};

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
    // Where p shows in the (turned, flipped) view, relative to the
    // document's center, which both leave in place
    let p = mirror(state, turn(state, p, view_angle(state)));
    state.view.offset = doc_size_pt(state, zoom, ppp) / 2.0 - p.to_vec2() * zoom / ppp;
}

/// View > Print Size: one inch of the document (its resolution in pixels)
/// as 72 points on screen.
pub fn print_size(state: &mut DocState, ppp: f32) {
    let zoom = 72.0 / state.doc.resolution * ppp;
    zoom_to(state, zoom, ppp);
}

/// View > Actual Size: the image at its physical size on the display, one
/// inch of it (at its resolution) an inch on the screen, using the display's
/// pixel density (72 per point elsewhere than macOS).
pub fn actual_size(state: &mut DocState, ppp: f32) {
    #[cfg(target_os = "macos")]
    let ppi = crate::app_kit::screen_ppi().unwrap_or(72.0 * ppp);
    #[cfg(not(target_os = "macos"))]
    let ppi = 72.0 * ppp;
    zoom_to(state, ppi / state.doc.resolution, ppp);
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
    let tool_gradient = (app.tool == Tool::Gradient).then(|| app.tool_gradient());
    let dither = app.flag("gradient.dither", true);
    let perspective_options = crate::perspective_crop::PerspectiveOptions::from_app(app);
    // A dialog over the window takes Enter and Escape
    let modal = app.modal_open();
    // The Brush's and Pencil's Mode, and the Eraser's (Brush, Pencil, Block)
    let paint_mode = crate::options_tools::paint_mode(app, tool);
    let eraser_mode = crate::options_tools::eraser_mode(app);
    let clone_scope = crate::options_tools::clone_scope(app);
    let rgb3 = |c: op_core::Color| {
        let [r, g, b, _] = c.to_rgba8();
        [r, g, b]
    };
    let matching =
        crate::options_tools::color_match(app, tool, (rgb3(app.foreground), rgb3(app.background)));
    let replace_mode = crate::options_tools::replace_mode(app);
    let magic_eraser = crate::options_tools::magic_eraser_options(app);
    let red_eye = crate::options_tools::red_eye_options(app);
    let heal = crate::options_tools::heal_options(app);
    let magnetic = crate::options_tools::magnetic_options(app);
    let quick = crate::options_tools::quick_options(app);
    let object = crate::options_tools::object_options(app);
    let mut quick_switch_to_add = false;
    let sampling_ring = tool == Tool::Eyedropper && app.flag("eyedropper.ring", true);
    // The Zoom tool's Zoom Out button swaps what a click and Alt-click do
    let zoom_out = tool == Tool::Zoom && app.flag("zoom.out", false);
    let mut straightened = false;
    let mut paint_error = None;
    let status_info = app.status_info;
    let overlay =
        crate::selection_brush::OVERLAY_COLORS[crate::options_tools::overlay_color(app)].1;
    let selbrush_subtract = crate::options_tools::selection_brush_subtracts(app);
    let ruler_units = app.ruler_units;
    let status_text = app
        .docs
        .get(&id)
        .map(|d| status_info.text(app, d))
        .unwrap_or_default();
    let Some(state) = app.docs.get_mut(&id) else {
        return;
    };
    let ppp = ui.ctx().pixels_per_point();
    let full = ui.max_rect();
    // Full Screen Mode's pasteboard is black, as in Photoshop
    let pasteboard = if app.screen_mode == crate::state::ScreenMode::Full {
        Color32::BLACK
    } else {
        color::PASTEBOARD
    };
    ui.painter().rect_filled(full, 0, pasteboard);

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
    // Dragging out of a ruler makes a guide, out of their corner moves
    // their origin; right-clicking one asks for the unit menu
    if let Some((top, left)) = rulers {
        crate::rulers::origin_input(ui, state, &view_options, window_rect, ppp);
        for (rect, vertical) in [(top, false), (left, true)] {
            let r = ui.interact(
                rect,
                ui.id().with(("ruler", vertical)),
                Sense::click_and_drag(),
            );
            if r.secondary_clicked() {
                app.ruler_menu = true;
            }
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
    // View › Snap: what this drag can snap to, gathered as it starts
    if response.drag_started_by(PointerButton::Primary) {
        let snapping = matches!(
            tool,
            Tool::RectangularMarquee
                | Tool::EllipticalMarquee
                | Tool::Move
                | Tool::Rectangle
                | Tool::Ellipse
                | Tool::Triangle
                | Tool::Polygon
                | Tool::Line
                | Tool::CustomShape
        );
        state.snap = None;
        if snapping {
            crate::snap::begin(state, &view_options, tool == Tool::Move);
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
            Tool::RotateView => rotate_view_input(ui, &response, state, ppp),
            Tool::Zoom if response.clicked() => {
                if let Some(p) = response.interact_pointer_pos() {
                    let z = next_zoom_step(state.view.zoom, alt == zoom_out);
                    zoom_at(state, z, p, ppp);
                }
            }
            Tool::ColorSampler => sampler_input(ui, &response, state, alt, ppp),
            Tool::SelectionBrush => {
                if let Some(opts) = paint {
                    paint_error = crate::selection_brush::input(
                        ui,
                        &response,
                        state,
                        opts,
                        selbrush_subtract,
                        ppp,
                    );
                }
            }
            // Alt-click picks the background color
            Tool::Eyedropper if response.is_pointer_button_down_on() => {
                if let Some(p) = response.interact_pointer_pos() {
                    // The color being replaced, kept from the press for the ring
                    let original_id = ui.id().with("eyedropper-original");
                    if ui.input(|i| i.pointer.primary_pressed()) {
                        let c = if alt { app.background } else { app.foreground };
                        ui.data_mut(|d| d.insert_temp(original_id, c));
                    }
                    let d = to_doc(state, p, ppp);
                    let o = app.eyedropper;
                    if d.x >= 0.0
                        && d.y >= 0.0
                        && let Some(c) =
                            state.sample_average(d.x as u32, d.y as u32, o.size, o.sample)
                    {
                        if alt {
                            app.background = c;
                        } else {
                            app.foreground = c;
                        }
                    }
                    if sampling_ring {
                        let new = if alt { app.background } else { app.foreground };
                        let old = ui.data(|d| d.get_temp(original_id)).unwrap_or(new);
                        // Over the canvas, which is painted after input
                        let painter = ui.ctx().layer_painter(egui::LayerId::new(
                            egui::Order::Foreground,
                            ui.id().with("sampling-ring"),
                        ));
                        sampling_ring_shape(&painter, p, new, old);
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
            Tool::Patch | Tool::ContentAwareMove => {
                let lasso = (
                    app.marquee.mode,
                    app.marquee.feather,
                    app.marquee.anti_alias,
                );
                patch_input(ui, &response, state, tool, heal, lasso, ppp);
            }
            Tool::MagicEraser | Tool::RedEye if response.clicked() => {
                if let Some(p) = response.interact_pointer_pos() {
                    let d = to_doc(state, p, ppp);
                    if d.x >= 0.0 && d.y >= 0.0 {
                        let at = (d.x as u32, d.y as u32);
                        let [r, g, b, _] = background.to_rgba8();
                        let (result, name, label) = if tool == Tool::MagicEraser {
                            let (options, opacity) = magic_eraser;
                            let r = op_core::fill::magic_erase(
                                &mut state.doc,
                                at,
                                &options,
                                opacity,
                                [r, g, b],
                            );
                            (r, "Magic Eraser", "magic eraser")
                        } else {
                            let r =
                                op_core::fill::red_eye(&mut state.doc, at, red_eye.0, red_eye.1);
                            (r, "Red Eye", "red eye tool")
                        };
                        match result {
                            Ok(true) => state.record(name),
                            Ok(false) => {}
                            Err(e) => paint_error = Some(fill_alert(e, label)),
                        }
                    }
                }
            }
            Tool::Crop if !modal => {
                straightened =
                    crate::crop_tool::input(ui, &response, state, &crop_options, background, ppp);
            }
            Tool::PerspectiveCrop if !modal => {
                crate::perspective_crop::input(ui, &response, state, &perspective_options, ppp);
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
                let g = tool_gradient.clone().expect("the Gradient tool's");
                paint_error = gradient_input(ui, &response, state, (&g, dither), app.gradient, ppp);
            }
            Tool::Lasso | Tool::PolygonalLasso => {
                let options = (
                    app.marquee.mode,
                    app.marquee.feather,
                    app.marquee.anti_alias,
                );
                lasso_input(ui, &response, state, tool, options, ppp);
            }
            Tool::QuickSelection => {
                quick_switch_to_add =
                    quick_input(ui, &response, state, quick, app.marquee.feather, ppp);
            }
            Tool::ObjectSelection => {
                object_input(ui, &response, state, app.marquee.mode, object, ppp);
            }
            Tool::MagneticLasso => {
                let options = (
                    app.marquee.mode,
                    app.marquee.feather,
                    app.marquee.anti_alias,
                );
                magnetic_input(ui, &response, state, options, magnetic, ppp);
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
                            Ok(true) => state.record_fadeable("Paint Bucket"),
                            Ok(false) => {}
                            Err(e) => paint_error = Some(e.message("Paint Bucket")),
                        }
                    }
                }
            }
            // Alt-click with the Clone Stamp or Healing Brush picks its
            // source point
            Tool::CloneStamp | Tool::HealingBrush
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
            Tool::CloneStamp | Tool::HealingBrush if state.picking_clone_source => {
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
            | Tool::HistoryBrush
            | Tool::Smudge
            | Tool::PatternStamp
            | Tool::BackgroundEraser
            | Tool::ColorReplacement
            | Tool::HealingBrush
            | Tool::SpotHealingBrush => {
                if let Some(opts) = paint {
                    let settings = StrokeSettings {
                        opts,
                        colors: (foreground, background),
                        retouch,
                        mode: paint_mode,
                        eraser_mode,
                        clone_scope,
                        matching,
                        replace_mode,
                        heal,
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

    // Quick Selection's New turns into Add after the first stroke, as in
    // Photoshop
    if quick_switch_to_add {
        *app.setting("quick.mode", "0") = "1".into();
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
            crate::crop_tool::cursor(state, &crop_options, p, ppp)
        } else if tool == Tool::PerspectiveCrop {
            let p = response.hover_pos().unwrap_or_default();
            crate::perspective_crop::cursor(state, p, ppp)
        } else {
            match tool {
                Tool::Zoom if alt != zoom_out => CursorIcon::ZoomOut,
                Tool::Zoom => CursorIcon::ZoomIn,
                // Painting tools draw their own brush outline instead
                _ if paint.is_some_and(|p| p.size * state.view.zoom / ppp >= 4.0) => {
                    CursorIcon::None
                }
                Tool::Eyedropper
                | Tool::ColorSampler
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
        // The crop's own turn, or the view's (the shader maps screen points
        // to the image by the opposite angle)
        rotation: crate::crop_tool::rotation(state, &crop_options).or_else(|| {
            let a = view_angle(state);
            (a != 0.0).then(|| {
                (
                    [state.doc.width as f32 / 2.0, state.doc.height as f32 / 2.0],
                    -a,
                )
            })
        }),
        mirror: view_flipped(state).then_some(state.doc.width as f32 / 2.0),
        pixel_grid: view_options.extras && view_options.pixel_grid,
    };
    ui.painter()
        .with_clip_rect(canvas_rect)
        .add(op_render::paint_callback(canvas_rect, image, view));

    if view_options.grid_visible() {
        crate::rulers::draw_grid(ui, state, canvas_rect, ppp);
    }
    if view_options.extras && view_options.layer_edges {
        draw_layer_edges(ui, state, canvas_rect, ppp);
    }
    crate::smart_guides::draw(ui, state, canvas_rect, ppp);
    // The color samplers show with the tools that use them
    if view_options.extras && matches!(tool, Tool::Eyedropper | Tool::ColorSampler) {
        draw_samplers(ui, state, canvas_rect, ppp);
    }
    // Extras or Show › Selection Edges off hides the selection edges (the
    // selection stays)
    if tool == Tool::SelectionBrush {
        // This tool shows the selection as an overlay instead of edges
        crate::selection_brush::draw_overlay(ui, state, canvas_rect, overlay, ppp);
    } else if view_options.extras && view_options.selection_edges {
        draw_selection(ui, state, canvas_rect, ppp, tool, shape_options);
    }
    if view_options.guides_visible() || state.guide_drag.is_some() {
        crate::rulers::draw_guides(ui, state, canvas_rect, ppp);
    }
    if rulers.is_some() {
        let pointer = ui.input(|i| i.pointer.hover_pos());
        crate::rulers::draw_rulers(ui, state, ruler_units, window_rect, pointer, ppp);
    }
    crate::free_transform::draw(ui, state, canvas_rect, ppp);
    if let Some(bounds) = controls_bounds
        && view_options.extras
    {
        crate::free_transform::draw_controls(ui, state, bounds, canvas_rect, ppp);
    }
    // The crop box belongs to the Crop tool: picking another tool crops to
    // a changed box (Photoshop 2026 doesn't ask) and drops it
    if tool != Tool::Crop && state.crop.is_some() {
        if crate::crop_tool::modified(state) {
            crate::crop_tool::commit(state, &crop_options, background);
        }
        state.crop = None;
    }
    // ...and the Perspective Crop tool's box likewise
    if tool != Tool::PerspectiveCrop && state.perspective_crop.is_some() {
        crate::perspective_crop::commit(state, &perspective_options);
        state.perspective_crop = None;
    }
    // Picking another tool keeps the text being typed
    if tool != Tool::HorizontalType && state.text_edit.is_some() {
        crate::type_tool::commit(state);
    }
    crate::type_tool::draw_caret(ui, state, type_options, canvas_rect, ppp);
    crate::crop_tool::draw(ui, state, &crop_options, canvas_rect, ppp);
    crate::perspective_crop::draw(ui, state, &perspective_options, canvas_rect, ppp);
    let transforming = state.free_transform.is_some();
    if let (Some(opts), Some(p), false) = (paint, response.hover_pos(), transforming) {
        brush_cursor(ui, canvas_rect, p, opts.size * state.view.zoom / ppp);
    }
    if status_bar(ui, state, status_rect, (status_info, &status_text), ppp) {
        app.status_menu = true;
    }
    vertical_scrollbar(ui, state, vscroll_rect, ppp);
}

/// Screen point → document pixel coordinates (not clamped).
pub(crate) fn to_doc(state: &DocState, p: Pos2, ppp: f32) -> Pos2 {
    let d = (p - origin(state, ppp)) * ppp / state.view.zoom;
    turn(
        state,
        mirror(state, Pos2::new(d.x, d.y)),
        -view_angle(state),
    )
}

/// Whether the view is flipped left to right (not while cropping, like
/// the rotation).
pub(crate) fn view_flipped(state: &DocState) -> bool {
    state.view.flip && state.crop.is_none()
}

/// `p` mirrored about the document's vertical center line when the view
/// is flipped.
fn mirror(state: &DocState, p: Pos2) -> Pos2 {
    if view_flipped(state) {
        Pos2::new(state.doc.width as f32 - p.x, p.y)
    } else {
        p
    }
}

/// The view's rotation in radians (clockwise), none while cropping (the
/// Crop tool turns the image its own way).
pub(crate) fn view_angle(state: &DocState) -> f32 {
    if state.crop.is_some() {
        0.0
    } else {
        state.view.rotation.to_radians()
    }
}

/// `p` (document pixels) turned clockwise by `angle` about the document's
/// center.
fn turn(state: &DocState, p: Pos2, angle: f32) -> Pos2 {
    if angle == 0.0 {
        return p;
    }
    let c = Pos2::new(state.doc.width as f32 / 2.0, state.doc.height as f32 / 2.0);
    let (s, k) = angle.sin_cos();
    let q = p - c;
    c + Vec2::new(q.x * k - q.y * s, q.x * s + q.y * k)
}

/// The part of the document shown in the window, in document pixels
/// (x0, y0, x1, y1; may extend past the canvas).
pub fn visible_rect(state: &DocState, ppp: f32) -> [f32; 4] {
    // With the view rotated, the bounds of all four corners
    let v = state.view.viewport;
    let corners = [
        v.left_top(),
        v.right_top(),
        v.left_bottom(),
        v.right_bottom(),
    ]
    .map(|p| to_doc(state, p, ppp));
    let (xs, ys) = (corners.map(|p| p.x), corners.map(|p| p.y));
    let min = |a: [f32; 4]| a.into_iter().fold(f32::INFINITY, f32::min);
    let max = |a: [f32; 4]| a.into_iter().fold(f32::NEG_INFINITY, f32::max);
    [min(xs), min(ys), max(xs), max(ys)]
}

/// Document pixel → screen point.
pub(crate) fn to_screen(state: &DocState, d: Pos2, ppp: f32) -> Pos2 {
    let d = mirror(state, turn(state, d, view_angle(state)));
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
        let start = crate::snap::point(ui, state, to_doc(state, p, ppp), ppp);
        state.shape_drag = Some((start, start));
    }
    let Some((start, _)) = state.shape_drag else {
        return;
    };
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let end = crate::snap::point(ui, state, to_doc(state, p, ppp), ppp);
        state.shape_drag = Some((start, end));
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
    (gradient, dither): (&op_core::gradient::Gradient, bool),
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
    let result = op_core::gradient::gradient_with(
        &mut state.doc,
        (start.x, start.y),
        (end.x, end.y),
        gradient,
        options,
        dither,
    );
    match result {
        Ok(()) => {
            state.record_fadeable("Gradient");
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
        if state.lasso.is_none()
            && response.drag_started_by(PointerButton::Primary)
            && let Some(p) = ui.input(|i| i.pointer.press_origin())
        {
            let (op, _, _) = selection_op(mods, has_selection, mode);
            state.lasso = Some(LassoPath {
                points: vec![to_doc(state, p, ppp)],
                op,
                polygonal: false,
                held: false,
            });
        }
        if let Some(lasso) = &mut state.lasso {
            let down = ui.input(|i| i.pointer.primary_down());
            if down {
                // Dragging draws freehand (also between Alt-clicked corners)
                if let Some(p) = pointer
                    && lasso
                        .points
                        .last()
                        .is_none_or(|l| l.distance(p) * state.view.zoom >= ppp)
                {
                    lasso.points.push(p);
                }
            } else if mods.alt {
                // Let go with Alt: clicks add straight edges until Alt is up
                lasso.held = true;
                if response.clicked()
                    && let Some(p) = pointer
                {
                    lasso.points.push(p);
                }
            } else {
                close = true;
            }
            if !close {
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
                held: false,
            });
        }
    }

    if close && let Some(lasso) = state.lasso.take() {
        let name = if lasso.polygonal {
            "Polygonal Lasso"
        } else {
            "Lasso"
        };
        finish_lasso(state, lasso, (feather, anti_alias), name);
    }
}

/// Closes a lasso outline into a selection (feathered, combined with the
/// current one by its operation) and records `name`. An outline enclosing
/// nothing changes nothing.
fn finish_lasso(
    state: &mut DocState,
    lasso: crate::state::LassoPath,
    (feather, anti_alias): (f32, bool),
    name: &str,
) {
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
    state.record(name);
}

/// The Magnetic Lasso: click to start, then move the pointer: the outline
/// follows the strongest edges between the last anchor and the pointer
/// (snapped to an edge within Width), anchors fall every so often
/// (Frequency) and where you click. Double-click, Enter or clicking the
/// start closes it; Backspace removes the last anchor; Escape cancels.
fn magnetic_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    (mode, feather, anti_alias): (crate::state::SelectionMode, f32, bool),
    (width, contrast, frequency): (u32, f32, f32),
    ppp: f32,
) {
    use crate::state::{LassoPath, MagneticPath};
    let pointer = ui
        .input(|i| i.pointer.hover_pos())
        .map(|p| to_doc(state, p, ppp));
    let (w, h) = (state.doc.width, state.doc.height);
    let cell = |p: Pos2| {
        (
            (p.x.max(0.0) as u32).min(w.saturating_sub(1)),
            (p.y.max(0.0) as u32).min(h.saturating_sub(1)),
        )
    };
    let center = |(x, y): (u32, u32)| Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
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

    let Some(mut magnetic) = state.magnetic.take() else {
        if response.clicked()
            && let Some(p) = pointer
            && p.x >= 0.0
            && p.y >= 0.0
        {
            let image = state.canvas_image();
            let edges = op_core::magnetic::EdgeMap::new(w, h, &image.pixels, contrast);
            let start = edges.snap(cell(p), width);
            let mods = ui.input(|i| i.modifiers);
            let (op, _, _) = selection_op(mods, state.doc.selection().is_some(), mode);
            state.lasso = Some(LassoPath {
                points: vec![center(start)],
                op,
                polygonal: false,
                held: false,
            });
            state.magnetic = Some(MagneticPath {
                edges: std::sync::Arc::new(edges),
                anchors: vec![0],
                fixed: 1,
            });
            ui.ctx().request_repaint();
        }
        return;
    };
    let Some(lasso) = state.lasso.as_mut() else {
        return;
    };
    if escape {
        state.lasso = None;
        return;
    }
    if backspace {
        magnetic.anchors.pop();
        let Some(&last) = magnetic.anchors.last() else {
            state.lasso = None;
            return;
        };
        magnetic.fixed = last + 1;
        lasso.points.truncate(magnetic.fixed);
    }
    // The live wire from the last fixed point to the (snapped) pointer
    if let Some(p) = pointer {
        let from = cell(lasso.points[magnetic.fixed - 1]);
        let to = magnetic.edges.snap(cell(p), width);
        let path = magnetic.edges.trace(from, to, width);
        lasso.points.truncate(magnetic.fixed);
        lasso.points.extend(path.into_iter().skip(1).map(center));
        // Anchors fall every so many points: fewer at a low Frequency
        let spacing = (10.0 + (100.0 - frequency) * 0.9) as usize;
        while lasso.points.len() - magnetic.fixed > spacing {
            magnetic.fixed += spacing;
            magnetic.anchors.push(magnetic.fixed - 1);
        }
    }
    let first = lasso.points[0];
    let near_start = pointer.is_some_and(|p| {
        lasso.points.len() > 3 && first.distance(p) * state.view.zoom / ppp <= 5.0
    });
    let close = enter || response.double_clicked() || (response.clicked() && near_start);
    if !close && response.clicked() {
        // A click fixes the outline so far with an anchor
        magnetic.fixed = lasso.points.len();
        magnetic.anchors.push(magnetic.fixed - 1);
    }
    if close {
        let lasso = state.lasso.take().expect("checked");
        finish_lasso(state, lasso, (feather, anti_alias), "Magnetic Lasso");
        return;
    }
    state.magnetic = Some(magnetic);
    ui.ctx().request_repaint();
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
        let start = crate::snap::point(ui, state, to_doc(state, p, ppp), ppp);
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
        .map(|p| crate::snap::point(ui, state, to_doc(state, p, ppp), ppp));
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
    let Some((_, start)) = &state.move_drag else {
        return None;
    };
    let raw = pointer.map_or(egui::Vec2::ZERO, |p| p - *start);
    let mut delta = crate::snap::offset(ui, state, raw, ppp);
    // Smart guides pull the move into line too (Control held: not)
    let tolerance = crate::snap::tolerance(state, ppp);
    if let Some(smart) = &mut state.smart_guides {
        if ui.input(|i| i.modifiers.ctrl) {
            smart.lines.clear();
        } else {
            delta = smart.align_with(raw, delta, tolerance);
        }
    }
    let Some((m, _)) = &state.move_drag else {
        return None;
    };
    let (dx, dy) = (delta.x.round() as i64, delta.y.round() as i64);
    m.apply(&mut state.doc, dx, dy);
    if !ui.input(|i| i.pointer.primary_down()) {
        state.move_drag = None;
        state.smart_guides = None;
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
        Tool::Smudge => ("smudge tool", "Smudge Tool"),
        Tool::PatternStamp => ("pattern stamp", "Pattern Stamp"),
        Tool::BackgroundEraser => ("background eraser", "Background Eraser"),
        Tool::ColorReplacement => ("color replacement tool", "Color Replacement Tool"),
        Tool::HealingBrush => ("healing brush", "Healing Brush"),
        Tool::SpotHealingBrush => ("spot healing brush", "Spot Healing Brush"),
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
    (retouch, clone_scope): (crate::state::RetouchOptions, op_core::SampleScope),
    (matching, replace_mode, strength): (
        Option<op_core::paint::ColorMatch>,
        op_core::BlendMode,
        f32,
    ),
    heal: crate::options_tools::HealOptions,
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
            // Sample: the layer itself, the layers up to it, or all merged
            let image = if clone_scope == op_core::SampleScope::Current {
                active_image(state).ok_or("")?
            } else {
                state.doc.sample_source(clone_scope).ok_or("")?
            };
            StrokeKind::Source {
                image,
                dx: offset.x.round() as i64,
                dy: offset.y.round() as i64,
            }
        }
        Tool::HistoryBrush => {
            // Paints from its source in the History panel (by default the
            // document as it was opened)
            let id = state.doc.active_layer.ok_or("")?;
            let snapshot = state.history_brush_snapshot().ok_or("")?;
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
        Tool::HealingBrush => {
            let source = state.clone_source.ok_or_else(|| {
                "Could not use the healing brush because the area to heal has not been defined \
                 (option-click to define a source point)."
                    .to_string()
            })?;
            let offset = match state.clone_offset {
                Some(o) if heal.aligned => o,
                _ => start - source,
            };
            state.clone_offset = Some(offset);
            let image = state.doc.sample_source(heal.scope).ok_or("")?;
            StrokeKind::Heal {
                source: image,
                dx: offset.x.round() as i64,
                dy: offset.y.round() as i64,
            }
        }
        Tool::SpotHealingBrush => {
            let scope = if heal.spot_all_layers {
                op_core::SampleScope::All
            } else {
                op_core::SampleScope::Current
            };
            StrokeKind::SpotHeal(state.doc.sample_source(scope).ok_or("")?)
        }
        Tool::Smudge => StrokeKind::Smudge(strength),
        Tool::PatternStamp => StrokeKind::Pattern(crate::state::default_pattern()),
        Tool::BackgroundEraser => {
            // Photoshop turns the background into a regular layer first
            if state
                .doc
                .active_layer
                .and_then(|id| state.doc.layer(id))
                .is_some_and(|l| l.is_background)
            {
                op_core::layer_ops::layer_from_background(&mut state.doc);
            }
            StrokeKind::BackgroundErase(matching.ok_or("")?)
        }
        Tool::ColorReplacement => StrokeKind::ReplaceColor {
            color: rgb(foreground),
            mode: replace_mode,
            matching: matching.ok_or("")?,
        },
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
    /// The Clone Stamp's Sample.
    clone_scope: op_core::SampleScope,
    /// The Background Eraser's and Color Replacement's color matching.
    matching: Option<op_core::paint::ColorMatch>,
    /// The Color Replacement's Mode.
    replace_mode: op_core::BlendMode,
    heal: crate::options_tools::HealOptions,
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
        clone_scope,
        matching,
        replace_mode,
        heal,
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
        let kind = match stroke_kind(
            tool,
            state,
            start,
            colors,
            (retouch, clone_scope),
            (matching, replace_mode, opts.opacity),
            heal,
        ) {
            Ok(kind) => kind,
            Err(message) => {
                // Only once per press, and never an empty message
                return (just_pressed && !message.is_empty()).then_some(message);
            }
        };
        let hard = tool == Tool::Pencil || eraser.is_some_and(|m| m != EraserMode::Brush);
        let flow = if hard { 1.0 } else { opts.flow };
        // The Smudge's Strength is its stroke's own, not an opacity
        let opacity = if block || tool == Tool::Smudge {
            1.0
        } else {
            opts.opacity
        };
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
            state.record_fadeable(name);
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

    // The Patch and Content-Aware Move tools drag the outline
    let shift = state.patch_drag.map_or(Vec2::ZERO, |(a, b)| {
        let d = b - a;
        Vec2::new(d.x.round(), d.y.round())
    });
    if let Some(segs) = state.selection_outline() {
        let visible = canvas.expand(2.0);
        for s in segs.iter() {
            let a = to_screen(state, Pos2::new(s[0] as f32, s[1] as f32) + shift, ppp);
            let b = to_screen(state, Pos2::new(s[2] as f32, s[3] as f32) + shift, ppp);
            if visible.intersects(Rect::from_two_pos(a, b)) {
                ants(a, b);
            }
        }
    }

    // Outlines are worked out in document pixels and mapped point by point,
    // so they turn with the view
    let corners = |r: Rect| {
        [
            r.left_top(),
            r.right_top(),
            r.right_bottom(),
            r.left_bottom(),
            r.left_top(),
        ]
        .map(|p| to_screen(state, p, ppp))
    };
    let oval = |r: Rect| -> Vec<Pos2> {
        (0..=96)
            .map(|i| {
                let t = i as f32 / 96.0 * std::f32::consts::TAU;
                to_screen(
                    state,
                    r.center() + Vec2::new(t.cos() * r.width() / 2.0, t.sin() * r.height() / 2.0),
                    ppp,
                )
            })
            .collect()
    };
    if let Some((a, b)) = state.object_drag {
        for w in corners(Rect::from_two_pos(a, b)).windows(2) {
            ants(w[0], w[1]);
        }
    }
    if let Some(drag) = state.marquee_drag {
        let mods = ui.input(|i| i.modifiers);
        let r = marquee_rect(&drag, mods.shift, mods.alt);
        let rect = Rect::from_two_pos(Pos2::new(r.x0, r.y0), Pos2::new(r.x1, r.y1));
        // The ellipse preview is a polyline approximating it
        let pts: Vec<Pos2> = if tool == Tool::EllipticalMarquee {
            oval(rect)
        } else {
            corners(rect).to_vec()
        };
        for w in pts.windows(2) {
            ants(w[0], w[1]);
        }
    }
    // A shape being dragged: its outline as a thin blue path
    if let Some((s, e)) = state.shape_drag {
        let mods = ui.input(|i| i.modifiers);
        let (a, b) = shape_points(tool, s, e, mods);
        let r = Rect::from_two_pos(a, b);
        let blue = egui::Stroke::new(1.0, Color32::from_rgb(0x2c, 0x8b, 0xe8));
        let painter = ui.painter_at(canvas);
        let doc_outline: Vec<Pos2> = match tool {
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
        let outline: Vec<Pos2> = doc_outline
            .iter()
            .map(|&p| to_screen(state, p, ppp))
            .collect();
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
        if (lasso.polygonal || lasso.held)
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
const ZOOM_BOX_W: f32 = pt(59.0);
/// The info's cell (its text centered in it) and the arrow's after it.
const INFO_X: f32 = pt(60.0);
const INFO_END: f32 = pt(230.0);
const HSCROLL_X: f32 = pt(244.0);
/// Photoshop sets the zoom and the info in Adobe Clean; Source Sans 3 at
/// these sizes covers the same widths.
const ZOOM_FONT: f32 = pt(13.5);
const INFO_FONT: f32 = pt(12.25);
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

/// The status bar's zoom box: clicking selects the percentage; a typed
/// one (with or without "%") zooms around the window's center on Enter
/// or when the box loses focus; Escape leaves the zoom as it was.
fn zoom_box(ui: &mut Ui, state: &mut DocState, rect: Rect, ppp: f32) {
    use egui::text::{CCursor, CCursorRange};
    let id = ui.id().with("zoom-box");
    let edit_id = id.with("edit");
    let focused = ui.memory(|m| m.has_focus(edit_id));
    let mut text = ui
        .data(|d| d.get_temp::<String>(id))
        .filter(|_| focused)
        .unwrap_or_else(|| zoom_label(state.view.zoom));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    child.style_mut().visuals.extreme_bg_color = Color32::TRANSPARENT;
    child.style_mut().visuals.selection.bg_fill = Color32::from_rgb(0x2c, 0x5f, 0xb8);
    let output = egui::TextEdit::singleline(&mut text)
        .id(edit_id)
        .frame(egui::Frame::NONE)
        .font(egui::FontId::proportional(ZOOM_FONT))
        // Until it's being edited, the percentage is painted as a label
        // (the text edit would put it a fraction of a pixel off)
        .text_color(if focused {
            Color32::from_gray(0xf0)
        } else {
            Color32::TRANSPARENT
        })
        .horizontal_align(egui::Align::Center)
        .vertical_align(egui::Align::Center)
        .desired_width(rect.width())
        .min_size(Vec2::new(rect.width(), rect.height()))
        .show(&mut child);
    let response = output.response.response;
    if !focused {
        ui.painter().text(
            Pos2::new(rect.center().x, rect.top() + pt(8.0)),
            Align2::CENTER_CENTER,
            zoom_label(state.view.zoom),
            egui::FontId::proportional(ZOOM_FONT),
            Color32::from_gray(0xf0),
        );
    }
    if response.gained_focus() {
        let mut s = output.state;
        let end = CCursor::new(text.chars().count());
        s.cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(0), end)));
        s.store(ui.ctx(), edit_id);
    }
    if response.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, text));
    } else if response.lost_focus() {
        ui.data_mut(|d| d.remove::<String>(id));
        if !ui.input(|i| i.key_pressed(Key::Escape))
            && let Some(percent) = crate::options_bar::typed_number(&text)
            && percent > 0.0
        {
            zoom_to(state, percent / 100.0, ppp);
        }
    }
}

/// Photoshop's status bar: zoom box, document info, the info menu caret,
/// then the horizontal scrollbar filling the rest.
fn status_bar(
    ui: &mut Ui,
    state: &mut DocState,
    rect: Rect,
    (info, text): (crate::status_info::StatusInfo, &str),
    ppp: f32,
) -> bool {
    let painter = ui.painter_at(rect);
    let x = |v: f32| rect.left() + v;
    let span = |x0: f32, x1: f32, y0: f32, y1: f32| {
        Rect::from_min_max(
            Pos2::new(x(x0), rect.top() + y0),
            Pos2::new(x(x1), rect.top() + y1),
        )
    };
    let (top, bottom) = (pt(1.0), rect.height());
    painter.rect_filled(rect, 0, color::PANEL);
    // The 1 pt top line: darker over the arrow's cell; the 1 pt dividers
    // after the zoom box and before the scrollbar run the full height
    let divider = Color32::from_gray(0x42);
    painter.rect_filled(span(0.0, INFO_END, 0.0, top), 0, divider);
    painter.rect_filled(
        span(INFO_END, HSCROLL_X - pt(1.0), 0.0, top),
        0,
        Color32::from_gray(0x3e),
    );
    painter.rect_filled(span(ZOOM_BOX_W, INFO_X, top, bottom), 0, divider);
    painter.rect_filled(
        span(HSCROLL_X - pt(1.0), HSCROLL_X, 0.0, bottom),
        0,
        divider,
    );

    // The zoom box, with a half-point lighter top edge
    let zoom_rect = span(0.0, ZOOM_BOX_W, top, bottom);
    painter.rect_filled(zoom_rect, 0, Color32::from_gray(0x45));
    painter.rect_filled(span(0.0, ZOOM_BOX_W, top, top + pt(0.5)), 0, TRACK);
    zoom_box(ui, state, zoom_rect, ppp);

    // The info, centered left of the arrow's cell; the progress items show
    // an empty dark track instead (Download Progress with its cancel button)
    let center_y = rect.top() + pt(8.75);
    if info.is_progress() {
        painter.rect_filled(span(INFO_X, INFO_END, top, bottom), 0, color::DIVIDER_DARK);
        if info == crate::status_info::StatusInfo::DownloadProgress {
            let c = Pos2::new(x(pt(219.0)), rect.top() + pt(8.5));
            painter.circle_filled(c, pt(5.5), Color32::from_gray(0xb7));
            let ink = egui::Stroke::new(pt(1.0), Color32::from_gray(0x11));
            let r = pt(2.5);
            painter.line_segment([c - Vec2::splat(r), c + Vec2::splat(r)], ink);
            painter.line_segment([c + Vec2::new(-r, r), c + Vec2::new(r, -r)], ink);
        }
    } else {
        painter.text(
            Pos2::new((x(INFO_X) + x(INFO_END)) / 2.0, center_y),
            Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(INFO_FONT),
            Color32::from_gray(0xd6),
        );
    }
    // The arrow: a thin chevron
    let p = |dx: f32, dy: f32| Pos2::new(x(pt(dx)), rect.top() + pt(dy));
    painter.add(egui::Shape::line(
        vec![p(234.5, 4.75), p(237.5, 8.75), p(234.5, 12.75)],
        egui::Stroke::new(pt(1.0), Color32::from_gray(0xe0)),
    ));
    let arrow = ui.interact(
        span(INFO_END, HSCROLL_X - pt(1.0), 0.0, bottom),
        ui.id().with("status-menu"),
        egui::Sense::click(),
    );

    // The horizontal scrollbar stops where the vertical one's column starts;
    // the corner below that column stays panel-colored
    let track = Rect::from_min_max(
        Pos2::new(x(HSCROLL_X), rect.top() + top),
        Pos2::new(rect.right() - VSCROLL_W, rect.bottom()),
    );
    if track.width() > 0.0 {
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(track.left(), rect.top()), track.max),
            0,
            TRACK,
        );
        let doc = doc_size_pt(state, state.view.zoom, ppp);
        let fraction = scroll_fraction(doc.x, state.view.viewport.width(), state.view.offset.x);
        let extent = doc.x + state.view.viewport.width();
        let d = scrollbar(ui, track, false, fraction, THUMB_THICKNESS, "hscroll");
        state.view.offset.x -= d * extent;
        clamp_offset(state, ppp);
    }
    arrow.clicked()
}

/// Photoshop's sampling ring around the Eyedropper while it's held: the
/// color being picked on the top half, the one it replaces on the bottom
/// half, inside a gray ring.
fn sampling_ring_shape(painter: &egui::Painter, c: Pos2, new: op_core::Color, old: op_core::Color) {
    use egui::{Color32, Mesh};
    let to32 = |c: op_core::Color| {
        let [r, g, b, _] = c.to_rgba8();
        Color32::from_rgb(r, g, b)
    };
    let (inner, outer) = (crate::theme::pt(38.0), crate::theme::pt(58.0));
    let half = |from: f32, color: Color32| {
        let mut mesh = Mesh::default();
        let n = 48;
        for k in 0..=n {
            let a = from + std::f32::consts::PI * k as f32 / n as f32;
            let d = egui::vec2(a.cos(), a.sin());
            mesh.colored_vertex(c + d * inner, color);
            mesh.colored_vertex(c + d * outer, color);
            if k > 0 {
                let i = 2 * k as u32;
                mesh.add_triangle(i - 2, i - 1, i);
                mesh.add_triangle(i - 1, i, i + 1);
            }
        }
        painter.add(egui::Shape::mesh(mesh));
    };
    // Top half (angles π..2π point up in screen space), then bottom
    half(std::f32::consts::PI, to32(new));
    half(0.0, to32(old));
    let gray = egui::Stroke::new(crate::theme::pt(2.0), Color32::from_gray(0x80));
    painter.circle_stroke(c, inner, gray);
    painter.circle_stroke(c, outer, gray);
}

/// Photoshop's "Could not use the …" alert for a tool acting on the layer.
fn fill_alert(e: op_core::fill::FillError, tool: &str) -> String {
    use op_core::fill::FillError;
    use op_core::paint::StrokeError;
    match e {
        FillError::NoLayer => StrokeError::NoLayer,
        FillError::Locked => StrokeError::Locked,
        FillError::Hidden => StrokeError::Hidden,
        FillError::Group => StrokeError::Group,
    }
    .message(tool)
}

/// The Patch tool and Content-Aware Move: without a selection, or pressing
/// outside it, they draw one freehand like the Lasso; dragging the
/// selection moves its outline, and letting go patches (or moves) it.
fn patch_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    tool: Tool,
    heal: crate::options_tools::HealOptions,
    lasso: (crate::state::SelectionMode, f32, bool),
    ppp: f32,
) {
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    if let Some((start, _)) = state.patch_drag {
        if let Some(p) = pointer {
            state.patch_drag = Some((start, p));
        }
        if ui.input(|i| i.pointer.primary_down()) {
            ui.ctx().request_repaint();
            return;
        }
        let Some((a, b)) = state.patch_drag.take() else {
            return;
        };
        let (dx, dy) = ((b.x - a.x).round() as i64, (b.y - a.y).round() as i64);
        let Some(selection) = state.doc.selection().cloned() else {
            return;
        };
        if dx == 0 && dy == 0 {
            return;
        }
        let (changed, name, moved) = match tool {
            Tool::Patch if heal.patch_destination => (
                op_core::heal::patch_to(&mut state.doc, &selection, (dx, dy)),
                "Patch Tool",
                true,
            ),
            Tool::Patch => (
                op_core::heal::patch(&mut state.doc, &selection, (dx, dy)),
                "Patch Tool",
                false,
            ),
            _ if heal.extend => (
                op_core::heal::patch_to(&mut state.doc, &selection, (dx, dy)),
                "Content-Aware Move",
                true,
            ),
            _ => (
                op_core::heal::content_aware_move(&mut state.doc, &selection, (dx, dy)),
                "Content-Aware Move",
                true,
            ),
        };
        if changed {
            // The selection follows the pixels to their new place
            if moved {
                state.doc.set_selection(Some(selection.translated(dx, dy)));
            }
            state.record(name);
        }
        return;
    }
    let inside = ui
        .input(|i| i.pointer.press_origin())
        .map(|p| to_doc(state, p, ppp))
        .is_some_and(|p| {
            p.x >= 0.0
                && p.y >= 0.0
                && state
                    .doc
                    .selection()
                    .is_some_and(|s| s.get(p.x as u32, p.y as u32) > 0)
        });
    if state.lasso.is_none() && inside && response.drag_started_by(PointerButton::Primary) {
        if let Some(p) = ui.input(|i| i.pointer.press_origin()) {
            let p = to_doc(state, p, ppp);
            state.patch_drag = Some((p, p));
        }
        return;
    }
    lasso_input(ui, response, state, Tool::Lasso, lasso, ppp);
}

/// The pixels the Quick Selection and Object Selection tools look at: the
/// merged image or the active layer.
fn sampler(state: &mut DocState, all_layers: bool) -> Option<op_core::smart_select::Sampler> {
    let (w, h) = (state.doc.width, state.doc.height);
    let rgba = if all_layers {
        state.canvas_image().pixels.clone()
    } else {
        let id = state.doc.active_layer?;
        state.doc.layer(id)?.image()?.to_rgba8()
    };
    Some(op_core::smart_select::Sampler::new(w, h, &rgba))
}

/// The Quick Selection tool: drag to grow a selection from the colors
/// under the brush (New, Add or Subtract; Shift adds, Alt subtracts).
/// Returns whether its New mode should turn into Add (after a stroke).
fn quick_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    (mode, size, all_layers, enhance): (usize, f32, bool, bool),
    feather: f32,
    ppp: f32,
) -> bool {
    use op_core::SelectionOp;
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    let down = ui.input(|i| i.pointer.primary_down());
    let radius = (size / 2.0).max(1.0);
    if state.quick.is_none() && response.is_pointer_button_down_on() && down {
        let Some(p) = pointer else {
            return false;
        };
        let Some(sampler) = sampler(state, all_layers) else {
            return false;
        };
        let mods = ui.input(|i| i.modifiers);
        let base = state.doc.selection().cloned();
        let op = if mods.alt || mode == 2 {
            SelectionOp::Subtract
        } else if mods.shift || mode == 1 {
            SelectionOp::Add
        } else {
            SelectionOp::Replace
        };
        let mut select = op_core::smart_select::QuickSelect::new(sampler);
        select.dab((p.x, p.y), radius);
        state.quick = Some(crate::state::QuickStroke {
            select,
            base,
            op,
            last: p,
        });
    } else if let Some(stroke) = &mut state.quick
        && down
        && let Some(p) = pointer
        && p.distance(stroke.last) >= radius / 2.0
    {
        stroke.select.dab((p.x, p.y), radius);
        stroke.last = p;
    }
    let Some(stroke) = &state.quick else {
        return false;
    };
    // The selection follows the stroke as it grows
    let mut region = stroke.select.selection();
    if enhance {
        region = region.feather(1.0);
    }
    if feather > 0.0 {
        region = region.feather(feather);
    }
    let combined = op_core::Selection::combine(stroke.base.as_ref(), region, stroke.op);
    let replaced = stroke.op == SelectionOp::Replace;
    state.doc.set_selection(Some(combined));
    if down {
        ui.ctx().request_repaint();
        return false;
    }
    state.quick = None;
    state.record("Quick Selection");
    replaced && mode == 0
}

/// The Object Selection tool: drag a rectangle around an object; the
/// object found in it becomes the selection (combined by the bar's mode
/// and Shift/Alt).
fn object_input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    mode: crate::state::SelectionMode,
    (all_layers, hard_edge): (bool, bool),
    ppp: f32,
) {
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    if state.object_drag.is_none()
        && response.drag_started_by(PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        let p = to_doc(state, p, ppp);
        state.object_drag = Some((p, p));
    }
    let Some((a, _)) = state.object_drag else {
        return;
    };
    if let Some(p) = pointer {
        state.object_drag = Some((a, p));
    }
    if ui.input(|i| i.pointer.primary_down()) {
        ui.ctx().request_repaint();
        return;
    }
    let Some((a, b)) = state.object_drag.take() else {
        return;
    };
    let (w, h) = (state.doc.width as f32, state.doc.height as f32);
    let x0 = a.x.min(b.x).clamp(0.0, w) as u32;
    let y0 = a.y.min(b.y).clamp(0.0, h) as u32;
    let x1 = a.x.max(b.x).clamp(0.0, w) as u32;
    let y1 = a.y.max(b.y).clamp(0.0, h) as u32;
    let Some(sampler) = sampler(state, all_layers) else {
        return;
    };
    let Some(mut region) = op_core::smart_select::object_in_rect(&sampler, (x0, y0, x1, y1)) else {
        return;
    };
    if !hard_edge {
        region = region.feather(1.0);
    }
    let mods = ui.input(|i| i.modifiers);
    let (op, _, _) = selection_op(mods, state.doc.selection().is_some(), mode);
    let combined = op_core::Selection::combine(state.doc.selection(), region, op);
    state.doc.set_selection(Some(combined));
    state.record("Object Selection");
}

/// The Rotate View tool: dragging turns the canvas about the document's
/// center by the angle the pointer sweeps around it (Shift snaps to 15°),
/// showing a compass; Escape turns it back upright.
fn rotate_view_input(ui: &Ui, response: &egui::Response, state: &mut DocState, ppp: f32) {
    let center = to_screen(
        state,
        Pos2::new(state.doc.width as f32 / 2.0, state.doc.height as f32 / 2.0),
        ppp,
    );
    let start_id = ui.id().with("rotate-view-start");
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        state.view.rotation = 0.0;
    }
    let angle_of = |p: Pos2| (p.y - center.y).atan2(p.x - center.x).to_degrees();
    if response.drag_started_by(PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        ui.data_mut(|d| d.insert_temp(start_id, (state.view.rotation, angle_of(p))));
    }
    let Some((start, from)) = ui.data(|d| d.get_temp::<(f32, f32)>(start_id)) else {
        return;
    };
    if !ui.input(|i| i.pointer.primary_down()) {
        ui.data_mut(|d| d.remove::<(f32, f32)>(start_id));
        return;
    }
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        let mut a = crate::options_tools::normalize_angle(start + angle_of(p) - from);
        if ui.input(|i| i.modifiers.shift) {
            a = crate::options_tools::normalize_angle((a / 15.0).round() * 15.0);
        }
        state.view.rotation = a;
    }
    // The compass: a ring and a needle pointing to the image's top
    let painter = ui.painter();
    let r = 60.0;
    painter.circle(
        center,
        r,
        Color32::from_black_alpha(90),
        egui::Stroke::new(1.5, Color32::from_gray(0xd0)),
    );
    let a = (state.view.rotation - 90.0).to_radians();
    let dir = Vec2::new(a.cos(), a.sin());
    painter.line_segment(
        [center - dir * r * 0.8, center],
        egui::Stroke::new(3.0, Color32::from_gray(0xd0)),
    );
    painter.line_segment(
        [center, center + dir * r * 0.8],
        egui::Stroke::new(3.0, Color32::from_rgb(0xe0, 0x30, 0x30)),
    );
    ui.ctx().request_repaint();
}

/// View › Show › Layer Edges: a thin blue outline around the pixels of
/// each selected layer.
fn draw_layer_edges(ui: &Ui, state: &DocState, clip: Rect, ppp: f32) {
    let painter = ui.painter().with_clip_rect(clip);
    let stroke = egui::Stroke::new(1.0, Color32::from_rgb(0x2c, 0x8b, 0xe8));
    for id in state.doc.selected_layers() {
        let Some((x0, y0, x1, y1)) = state
            .doc
            .layer(id)
            .and_then(|l| l.image())
            .and_then(|i| i.content_bounds())
        else {
            continue;
        };
        let (x0, y0, x1, y1) = (x0 as f32, y0 as f32, x1 as f32, y1 as f32);
        let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
            .map(|(x, y)| to_screen(state, Pos2::new(x, y), ppp));
        for i in 0..4 {
            painter.line_segment([corners[i], corners[(i + 1) % 4]], stroke);
        }
    }
}

/// Photoshop allows ten color samplers.
pub const MAX_SAMPLERS: usize = 10;

/// The color sampler whose marker is under the screen point `p`.
fn sampler_at(state: &DocState, p: Pos2, ppp: f32) -> Option<usize> {
    state
        .color_samplers
        .iter()
        .position(|&s| (sampler_screen(state, s, ppp) - p).length() <= pt(7.0))
}

/// Where a sampler's marker sits: the center of its pixel.
fn sampler_screen(state: &DocState, s: Pos2, ppp: f32) -> Pos2 {
    to_screen(state, s + Vec2::splat(0.5), ppp)
}

/// The Color Sampler tool: a click places a sampler on the pixel (up to
/// ten) and can drag it on; pressing on a marker drags it, Alt-clicking one
/// removes it, and one dragged off the canvas is removed. Samplers are not
/// recorded in the history, as in Photoshop.
fn sampler_input(ui: &Ui, response: &egui::Response, state: &mut DocState, alt: bool, ppp: f32) {
    let (w, h) = (state.doc.width as f32, state.doc.height as f32);
    let pixel = |d: Pos2| Pos2::new(d.x.floor(), d.y.floor());
    let inside = |d: Pos2| d.x >= 0.0 && d.y >= 0.0 && d.x < w && d.y < h;
    if ui.input(|i| i.pointer.primary_pressed())
        && response.hovered()
        && let Some(p) = ui.input(|i| i.pointer.interact_pos())
    {
        match sampler_at(state, p, ppp) {
            Some(i) if alt => {
                state.color_samplers.remove(i);
            }
            Some(i) => state.sampler_drag = Some(i),
            None if !alt => {
                let d = to_doc(state, p, ppp);
                if inside(d) && state.color_samplers.len() < MAX_SAMPLERS {
                    state.color_samplers.push(pixel(d));
                    state.sampler_drag = Some(state.color_samplers.len() - 1);
                }
            }
            None => {}
        }
    }
    let Some(i) = state.sampler_drag else {
        return;
    };
    let pointer = ui.input(|i| i.pointer.interact_pos());
    if let Some(p) = pointer {
        let d = to_doc(state, p, ppp);
        if i < state.color_samplers.len() {
            state.color_samplers[i] = pixel(d);
        }
    }
    if !ui.input(|i| i.pointer.primary_down()) {
        state.sampler_drag = None;
        if i < state.color_samplers.len() && !inside(state.color_samplers[i]) {
            state.color_samplers.remove(i);
        }
    } else {
        ui.ctx().request_repaint();
    }
}

/// Photoshop's sampler markers (measured on Photoshop 2026): a `#ececec`
/// ring 6.75 pt in outer radius and 2 pt thick, a 2 pt dot at the center,
/// 2 pt arms from the ring out to 11.75 pt, and the sampler's number below
/// right (its digits from 4.5 to 11.5 pt below the center, 5.5 pt right of
/// it).
fn draw_samplers(ui: &Ui, state: &DocState, clip: Rect, ppp: f32) {
    let painter = ui.painter().with_clip_rect(clip);
    let ink = Color32::from_gray(0xec);
    let stroke = egui::Stroke::new(pt(2.0), ink);
    for (k, &s) in state.color_samplers.iter().enumerate() {
        let c = sampler_screen(state, s, ppp);
        painter.circle_stroke(c, pt(5.75), stroke);
        painter.circle_filled(c, pt(1.0), ink);
        for d in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
            painter.line_segment([c + d * pt(6.75), c + d * pt(11.75)], stroke);
        }
        painter.text(
            c + Vec2::new(pt(5.1), pt(1.1)),
            Align2::LEFT_TOP,
            (k + 1).to_string(),
            egui::FontId::proportional(pt(10.5)),
            ink,
        );
    }
}
