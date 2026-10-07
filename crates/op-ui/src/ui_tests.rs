//! Headless UI tests: the whole app runs in egui_kittest with the wgpu
//! renderer, so interactions can be simulated and frames rendered to PNG.
//!
//! Rendering uses 2 physical pixels per point, like the Retina screen the
//! Photoshop references were taken on (so 100% zoom matches). Screenshots are
//! written to `target/ui-shots/` both at 2× and downscaled to 1 pixel per
//! Photoshop point (`*_1x.png`), which lines up with Photoshop screenshots. The tests
//! that only write screenshots are `#[ignore]`d; run them with
//! `cargo test -p op-ui ui_tests -- --ignored`.

use std::path::PathBuf;

use egui::{Modifiers, Pos2, Vec2};
use egui_kittest::Harness;
use op_core::Color;

use crate::OpenPhotoApp;
use crate::theme::{UI_SCALE, pt};

/// Photoshop's default window on the reference screen, in points.
pub const WINDOW_PT: Vec2 = Vec2::new(1350.0, 800.0);

pub fn harness(files: Vec<PathBuf>) -> Harness<'static, OpenPhotoApp> {
    let mut harness = Harness::builder()
        .with_size(WINDOW_PT / UI_SCALE)
        .with_pixels_per_point(2.0)
        // Real frame times, so a double click fits in egui's 0.3 s window
        .with_step_dt(1.0 / 60.0)
        .wgpu()
        .build_eframe(|cc| OpenPhotoApp::new_headless(cc, files));
    // The app sets egui's zoom factor on the first frame
    harness.run_steps(4);
    harness
}

/// Renders the current frame to `target/ui-shots/{name}.png`.
pub fn shot(harness: &mut Harness<'_, OpenPhotoApp>, name: &str) -> PathBuf {
    let image = harness.render().expect("render frame");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-shots");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.png"));
    image.save(&path).unwrap();
    let small = image::imageops::resize(
        &image,
        image.width() / 2,
        image.height() / 2,
        image::imageops::FilterType::Triangle,
    );
    small.save(dir.join(format!("{name}_1x.png"))).unwrap();
    path
}

/// A point given in Photoshop points from the window's top-left corner.
pub fn at_pt(x: f32, y: f32) -> Pos2 {
    Pos2::new(pt(x), pt(y))
}

pub fn click(harness: &mut Harness<'_, OpenPhotoApp>, pos: Pos2) {
    harness.hover_at(pos);
    harness.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.step();
    harness.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(3);
}

/// A click with modifier keys held (Cmd-click, Shift-click).
pub fn click_with(harness: &mut Harness<'_, OpenPhotoApp>, pos: Pos2, modifiers: Modifiers) {
    harness.event(egui::Event::ModifiersChanged(modifiers));
    click(harness, pos);
    harness.event(egui::Event::ModifiersChanged(Modifiers::NONE));
    harness.step();
}

pub fn double_click(harness: &mut Harness<'_, OpenPhotoApp>, pos: Pos2) {
    harness.hover_at(pos);
    for _ in 0..2 {
        for pressed in [true, false] {
            harness.event(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
            harness.step();
        }
    }
    harness.run_steps(3);
}

pub fn drag(harness: &mut Harness<'_, OpenPhotoApp>, from: Pos2, to: Pos2, modifiers: Modifiers) {
    harness.hover_at(from);
    harness.event_modifiers(
        egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers,
        },
        modifiers,
    );
    harness.step();
    for i in 1..=4 {
        let p = from + (to - from) * (i as f32 / 4.0);
        harness.event_modifiers(egui::Event::PointerMoved(p), modifiers);
        harness.step();
    }
    harness.event_modifiers(
        egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers,
        },
        modifiers,
    );
    harness.run_steps(3);
}

fn active<'a>(h: &'a Harness<'_, OpenPhotoApp>) -> &'a crate::state::DocState {
    let app = &h.state().state;
    &app.docs[&app.active_doc.unwrap()]
}

/// Screen position (egui points) of a document pixel in the active document.
fn doc_point(h: &Harness<'_, OpenPhotoApp>, x: f32, y: f32) -> Pos2 {
    let state = active(h);
    let ppp = 2.0 * UI_SCALE;
    crate::document_view::origin(state, ppp) + egui::vec2(x, y) * state.view.zoom / ppp
}

#[test]
fn marquee_selects_and_click_deselects() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    assert_eq!(h.state().state.tool, op_tools::Tool::RectangularMarquee);

    let (a, b) = (doc_point(&h, 100.0, 120.0), doc_point(&h, 300.0, 220.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let sel = active(&h).doc.selection().expect("selection made");
    let (x0, y0, x1, y1) = sel.bounds().unwrap();
    assert!(
        (x0 as i32 - 100).abs() <= 1 && (y0 as i32 - 120).abs() <= 1,
        "{x0},{y0}"
    );
    assert!(
        (x1 as i32 - 300).abs() <= 1 && (y1 as i32 - 220).abs() <= 1,
        "{x1},{y1}"
    );
    let states = active(&h).history.states();
    assert_eq!(states.last().unwrap().name, "Rectangular Marquee");

    // Shift-drag adds to the selection
    let (c, d) = (doc_point(&h, 400.0, 400.0), doc_point(&h, 500.0, 500.0));
    drag(&mut h, c, d, Modifiers::SHIFT);
    let (_, _, x1, y1) = active(&h).doc.selection().unwrap().bounds().unwrap();
    assert!(x1 >= 499 && y1 >= 499);

    // A click deselects
    let p = doc_point(&h, 50.0, 50.0);
    click(&mut h, p);
    assert!(active(&h).doc.selection().is_none());
    assert_eq!(active(&h).history.states().last().unwrap().name, "Deselect");

    // Cmd+Shift+D reselects, Cmd+A selects all, Cmd+D deselects
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::D);
    h.run_steps(2);
    assert!(active(&h).doc.selection().is_some());
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.run_steps(2);
    assert_eq!(
        active(&h).doc.selection().unwrap().bounds(),
        Some((0, 0, 734, 811))
    );
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    assert!(active(&h).doc.selection().is_none());
}

#[test]
fn shift_m_cycles_marquee_tools() {
    let mut h = harness(Vec::new());
    h.key_press_modifiers(Modifiers::SHIFT, egui::Key::M);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::EllipticalMarquee);
    h.key_press(egui::Key::V);
    h.run_steps(2);
    h.key_press(egui::Key::M);
    h.run_steps(2);
    // The slot remembers the elliptical marquee
    assert_eq!(h.state().state.tool, op_tools::Tool::EllipticalMarquee);
}

#[test]
#[ignore]
fn screenshot_painting() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([0x14, 0xa5, 0xdc, 255]);
    h.key_press(egui::Key::B);
    for _ in 0..6 {
        h.key_press(egui::Key::CloseBracket);
    }
    h.run_steps(2);
    let (a, b) = (doc_point(&h, 120.0, 200.0), doc_point(&h, 600.0, 260.0));
    drag(&mut h, a, b, Modifiers::NONE);
    // A softer, half-opacity stroke crossing it
    h.key_press(egui::Key::Num5);
    let (c, d) = (doc_point(&h, 300.0, 100.0), doc_point(&h, 360.0, 600.0));
    drag(&mut h, c, d, Modifiers::NONE);
    h.key_press(egui::Key::E);
    let (e, f) = (doc_point(&h, 100.0, 500.0), doc_point(&h, 650.0, 520.0));
    drag(&mut h, e, f, Modifiers::NONE);
    h.key_press(egui::Key::B);
    h.hover_at(doc_point(&h, 450.0, 420.0));
    h.run_steps(3);
    shot(&mut h, "painting");
}

#[test]
#[ignore]
fn screenshot_fill_dialog() {
    let mut h = harness(Vec::new());
    probe_document(&mut h);
    h.key_press_modifiers(Modifiers::SHIFT, egui::Key::F5);
    h.run_steps(4);
    shot(&mut h, "fill_dialog");
    shot_dialog(&mut h, "fill", 348.0, 293.0);
}

#[test]
#[ignore]
fn screenshot_selection() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let (a, b) = (doc_point(&h, 100.0, 120.0), doc_point(&h, 400.0, 420.0));
    drag(&mut h, a, b, Modifiers::NONE);
    h.key_press_modifiers(Modifiers::SHIFT, egui::Key::M);
    h.run_steps(2);
    let (c, d) = (doc_point(&h, 300.0, 300.0), doc_point(&h, 650.0, 700.0));
    drag(&mut h, c, d, Modifiers::SHIFT);
    shot(&mut h, "selection");
}

fn composite_pixel(h: &mut Harness<'_, OpenPhotoApp>, x: u32, y: u32) -> [u8; 4] {
    let app = &mut h.state_mut().state;
    let id = app.active_doc.unwrap();
    let state = app.docs.get_mut(&id).unwrap();
    state.sample(x, y).unwrap().to_rgba8()
}

#[test]
fn brush_paints_with_the_foreground_color() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    h.key_press(egui::Key::B);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::Brush);

    // ] grows the 30 px brush by Photoshop's step (5 px below 50)
    h.key_press(egui::Key::CloseBracket);
    h.run_steps(2);
    assert_eq!(h.state().state.brush.size, 35.0);
    // 5 sets 50% opacity, 0 back to 100%
    h.key_press(egui::Key::Num5);
    h.key_press(egui::Key::Num0);
    h.run_steps(2);
    assert_eq!(h.state().state.brush.opacity, 1.0);

    let (a, b) = (doc_point(&h, 100.0, 300.0), doc_point(&h, 500.0, 300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let px = composite_pixel(&mut h, 300, 300);
    assert!(px[0] > 240 && px[1] < 20 && px[2] < 20, "{px:?}");
    // Untouched pixels keep the document color
    assert_eq!(composite_pixel(&mut h, 300, 600), [0x14, 0x14, 0x14, 255]);
    assert_eq!(
        active(&h).history.states().last().unwrap().name,
        "Brush Tool"
    );
}

#[test]
fn painting_a_hidden_layer_shows_photoshops_alert() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    {
        let app = &mut h.state_mut().state;
        let id = app.active_doc.unwrap();
        let doc = &mut app.docs.get_mut(&id).unwrap().doc;
        doc.layers[0].visible = false;
    }
    h.key_press(egui::Key::B);
    h.run_steps(2);
    let p = doc_point(&h, 300.0, 300.0);
    click(&mut h, p);
    assert_eq!(
        h.state().state.alert.as_deref(),
        Some("Could not use the brush tool because the target layer is hidden.")
    );
}

fn select_rect(h: &mut Harness<'_, OpenPhotoApp>, x0: f32, y0: f32, x1: f32, y1: f32) {
    let app = &mut h.state_mut().state;
    let id = app.active_doc.unwrap();
    let doc = &mut app.docs.get_mut(&id).unwrap().doc;
    let (w, ht) = (doc.width, doc.height);
    doc.set_selection(Some(op_core::Selection::rect(
        w,
        ht,
        op_core::selection::Rect::new(x0, y0, x1, y1),
    )));
}

#[test]
fn fill_and_clear_keys() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    h.state_mut().state.background = Color::from_rgba8([0, 0, 255, 255]);
    select_rect(&mut h, 0.0, 0.0, 100.0, 100.0);

    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    assert_eq!(composite_pixel(&mut h, 50, 50), [255, 0, 0, 255]);
    assert_eq!(composite_pixel(&mut h, 150, 150), [0x14, 0x14, 0x14, 255]);
    assert_eq!(active(&h).history.states().last().unwrap().name, "Fill");

    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Backspace);
    h.run_steps(2);
    assert_eq!(composite_pixel(&mut h, 50, 50), [0, 0, 255, 255]);

    // Delete on a regular layer clears the selection to transparency
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    h.key_press(egui::Key::Backspace);
    h.run_steps(2);
    assert_eq!(active(&h).history.states().last().unwrap().name, "Clear");
    assert_eq!(composite_pixel(&mut h, 50, 50), [0, 0, 255, 255]);

    // Without a selection, Delete removes the layer
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    let layers = active(&h).doc.layers.len();
    h.key_press(egui::Key::Backspace);
    h.run_steps(2);
    assert_eq!(active(&h).doc.layers.len(), layers - 1);
}

#[test]
fn fill_dialog_applies_on_enter() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([0, 255, 0, 255]);
    h.key_press_modifiers(Modifiers::SHIFT, egui::Key::F5);
    h.run_steps(3);
    assert!(h.state().state.fill_dialog.is_some());
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().state.fill_dialog.is_none());
    assert_eq!(composite_pixel(&mut h, 10, 10), [0, 255, 0, 255]);
}

#[test]
fn paint_bucket_fills_contiguous_area() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([255, 255, 0, 255]);
    h.state_mut().state.select_tool(op_tools::Tool::PaintBucket);
    let p = doc_point(&h, 200.0, 200.0);
    click(&mut h, p);
    assert_eq!(composite_pixel(&mut h, 700, 800), [255, 255, 0, 255]);
    assert_eq!(
        active(&h).history.states().last().unwrap().name,
        "Paint Bucket"
    );
}

#[test]
fn move_tool_drags_and_nudges() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A red square on a new layer
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);

    h.key_press(egui::Key::V);
    h.run_steps(2);
    let (a, b) = (doc_point(&h, 120.0, 120.0), doc_point(&h, 220.0, 170.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert_eq!(composite_pixel(&mut h, 220, 170), [255, 0, 0, 255]);
    assert_eq!(composite_pixel(&mut h, 110, 110), [0x14, 0x14, 0x14, 255]);
    assert_eq!(active(&h).history.states().last().unwrap().name, "Move");

    // The square now spans x 200..240; nudge it right by 1 and then 10
    h.key_press(egui::Key::ArrowRight);
    h.key_press_modifiers(Modifiers::SHIFT, egui::Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(composite_pixel(&mut h, 250, 170), [255, 0, 0, 255]);
    assert_eq!(composite_pixel(&mut h, 205, 170), [0x14, 0x14, 0x14, 255]);
    assert_eq!(active(&h).history.states().last().unwrap().name, "Nudge");
}

#[test]
fn move_auto_select_and_transform_controls() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A red square on Layer 1, then an empty Layer 2 on top (active)
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    let square = active(&h).doc.layers[1].id;

    h.key_press(egui::Key::V);
    h.run_steps(2);
    // Auto-Select's checkbox in the options bar
    click(&mut h, at_pt(115.0, 45.0));
    assert!(h.state().state.move_options.auto_select);
    let (a, b) = (doc_point(&h, 120.0, 120.0), doc_point(&h, 220.0, 170.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert_eq!(active(&h).doc.active_layer, Some(square));
    assert_eq!(layer_pixel(&h, 1, 220, 170), [255, 0, 0, 255]);

    // Show Transform Controls: the square (now 200..240 × 150..190) gets a
    // box, and dragging its corner starts Free Transform
    click(&mut h, at_pt(264.0, 45.0));
    assert!(h.state().state.move_options.show_transform_controls);
    h.run_steps(2);
    let (a, b) = (doc_point(&h, 240.0, 190.0), doc_point(&h, 260.0, 210.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert!(active(&h).free_transform.is_some());
}

#[test]
fn moving_the_background_without_selection_is_refused() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press(egui::Key::V);
    h.run_steps(2);
    let (a, b) = (doc_point(&h, 120.0, 120.0), doc_point(&h, 220.0, 170.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert_eq!(
        h.state().state.alert.as_deref(),
        Some("Could not use the move tool because the layer is locked.")
    );
}

#[test]
fn toolbar_foreground_opens_color_picker() {
    let mut h = harness(Vec::new());
    h.state_mut().state.foreground = Color::from_rgba8([0x00, 0xaf, 0xdc, 255]);
    // The foreground swatch at the bottom of the toolbar
    click(&mut h, at_pt(13.0, 720.0));
    let session = h
        .state()
        .state
        .color_picker
        .as_ref()
        .expect("picker opened");
    assert_eq!(session.target, crate::state::PickerTarget::Foreground);
    assert!(h.state().state.modal_open());

    // Escape cancels and leaves the color alone
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(h.state().state.color_picker.is_none());
    assert_eq!(
        h.state().state.foreground.to_rgba8(),
        [0x00, 0xaf, 0xdc, 255]
    );
}

#[test]
#[ignore]
fn screenshot_color_picker() {
    let mut h = harness(Vec::new());
    h.state_mut().state.foreground = Color::from_rgba8([0x00, 0xaf, 0xdc, 255]);
    h.state_mut()
        .state
        .open_color_picker(crate::state::PickerTarget::Foreground);
    h.run_steps(4);
    shot(&mut h, "color_picker");
    // White, as in the Photoshop capture it's compared with
    h.state_mut().state.color_picker = None;
    h.state_mut().state.foreground = Color::WHITE;
    h.state_mut()
        .state
        .open_color_picker(crate::state::PickerTarget::Foreground);
    h.run_steps(4);
    shot_dialog(&mut h, "color_picker_dialog", 534.0, 374.0);
}

/// The document open in the Photoshop reference screenshots: 734×811,
/// without an embedded profile, shown at 100%.
pub fn reference_document(h: &mut Harness<'_, OpenPhotoApp>) {
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let doc = op_core::Document::new_with_background(
        "Weixin Image_20260909100239_11307_249.png",
        734,
        811,
        Color::from_rgba8([0x14, 0x14, 0x14, 255]),
    );
    app.add_document(doc, "Open");
    h.run_steps(6);
}

#[test]
#[ignore]
fn screenshot_main_window() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    shot(&mut h, "main_window");
}

fn layer_pixel(h: &Harness<'_, OpenPhotoApp>, layer: usize, x: u32, y: u32) -> [u8; 4] {
    let image = active(h).doc.layers[layer].image().unwrap();
    image.pixel(x, y)
}

#[test]
fn copy_and_paste_stack_a_new_layer() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::C);
    h.run_steps(2);
    // egui-winit delivers Cmd+V as a Paste event
    h.event(egui::Event::Paste(String::new()));
    h.run_steps(2);
    let doc = &active(&h).doc;
    assert_eq!(doc.layers.len(), 2);
    assert_eq!(doc.layers[1].name, "Layer 1");
    assert!(doc.selection().is_none());
    assert_eq!(active(&h).history.states().last().unwrap().name, "Paste");
    // Pasted in place over the original; nothing outside it
    assert_eq!(layer_pixel(&h, 1, 120, 120), [255, 0, 0, 255]);
    assert_eq!(layer_pixel(&h, 1, 150, 120), [0, 0, 0, 0]);

    // Cut on the new layer leaves a hole that shows the background
    select_rect(&mut h, 100.0, 100.0, 120.0, 140.0);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::X);
    h.run_steps(2);
    assert_eq!(layer_pixel(&h, 1, 110, 120)[3], 0);
    assert_eq!(layer_pixel(&h, 1, 130, 120), [255, 0, 0, 255]);
    assert_eq!(active(&h).history.states().last().unwrap().name, "Cut");

    // Copying only transparent pixels is refused like in Photoshop
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::C);
    h.run_steps(2);
    assert_eq!(
        h.state().state.alert.as_deref(),
        Some("Could not complete the Copy command because the selected area is empty.")
    );
}

fn layer_names(h: &Harness<'_, OpenPhotoApp>) -> Vec<String> {
    active(h)
        .doc
        .layers
        .iter()
        .map(|l| l.name.clone())
        .collect()
}

fn last_history(h: &Harness<'_, OpenPhotoApp>) -> String {
    active(h).history.states().last().unwrap().name.clone()
}

#[test]
fn layer_shortcuts_copy_arrange_and_merge() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Cmd+J on the background without a selection: "Layer 1"
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Layer 1"]);
    assert_eq!(last_history(&h), "Layer Via Copy");
    // ...and on a regular layer: "Layer 1 copy"
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Layer 1", "Layer 1 copy"]);
    // Cmd+[ sends it backward
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::OpenBracket);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Layer 1 copy", "Layer 1"]);
    assert_eq!(last_history(&h), "Layer Order");
    // Cmd+E merges it down into the background
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::E);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Layer 1"]);
    assert_eq!(last_history(&h), "Merge Down");
    // Shift+Cmd+E merges everything visible
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::E);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background"]);
    assert_eq!(last_history(&h), "Merge Visible");
}

#[test]
fn dragging_several_selected_rows() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    for _ in 0..3 {
        h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    }
    h.run_steps(3);
    assert_eq!(
        layer_names(&h),
        ["Background", "Layer 1", "Layer 1 copy", "Layer 1 copy 2"]
    );
    // Select the top row and "Layer 1" (third), then drag the top row to
    // just above the background (the line at 632 + 42.5 + 42 + 42)
    let doc = &mut h.state_mut().state.active().unwrap().doc;
    let (l1, top) = (doc.layers[1].id, doc.layers[3].id);
    doc.set_selected_layers(vec![l1, top]);
    h.run_steps(2);
    drag(
        &mut h,
        at_pt(1130.0, 654.0),
        at_pt(1130.0, 758.5),
        Modifiers::NONE,
    );
    assert_eq!(
        layer_names(&h),
        ["Background", "Layer 1", "Layer 1 copy 2", "Layer 1 copy"]
    );
    assert_eq!(last_history(&h), "Layer Order");
}

#[test]
fn layers_panel_drag_and_rename() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.run_steps(3);
    // Rows (top to bottom): "Layer 1 copy", "Layer 1", "Background" from
    // 632 pt: the first 42.5 pt (a 33.5 pt thumbnail + 8 pt + a 1 pt line),
    // the others 42. Drag the top row below "Layer 1".
    drag(
        &mut h,
        at_pt(1130.0, 654.0),
        at_pt(1130.0, 716.0),
        Modifiers::NONE,
    );
    assert_eq!(layer_names(&h), ["Background", "Layer 1 copy", "Layer 1"]);
    assert_eq!(last_history(&h), "Layer Order");
    // Nothing goes below the background
    drag(
        &mut h,
        at_pt(1130.0, 654.0),
        at_pt(1130.0, 760.0),
        Modifiers::NONE,
    );
    assert_eq!(layer_names(&h), ["Background", "Layer 1 copy", "Layer 1"]);

    // Double-click the middle row's name and type a new one
    double_click(&mut h, at_pt(1130.0, 694.0));
    assert!(active(&h).renaming.is_some());
    h.event(egui::Event::Text("Sky".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(active(&h).renaming.is_none());
    assert_eq!(layer_names(&h), ["Background", "Sky", "Layer 1"]);
    assert_eq!(last_history(&h), "Rename Layer");

    // Clicking the background's lock makes it "Layer 0"
    click(&mut h, at_pt(1312.5, 737.75));
    assert_eq!(layer_names(&h), ["Layer 0", "Sky", "Layer 1"]);
    assert!(!active(&h).doc.layers[0].is_background);
}

#[test]
#[ignore]
fn screenshot_layers() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.run_steps(3);
    shot(&mut h, "layers");
    double_click(&mut h, at_pt(1130.0, 694.0));
    shot(&mut h, "layers_rename");
}

fn run_command(h: &mut Harness<'_, OpenPhotoApp>, command: crate::commands::Command) {
    let ctx = h.ctx.clone();
    crate::commands::run(command, &ctx, &mut h.state_mut().state);
    h.run_steps(3);
}

#[test]
fn rotate_crop_and_trim() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    run_command(&mut h, Command::Rotate90Clockwise);
    assert_eq!((active(&h).doc.width, active(&h).doc.height), (811, 734));
    assert_eq!(last_history(&h), "Rotate Canvas");

    // Crop is only available with a selection
    assert!(!Command::Crop.enabled(&h.state().state));
    select_rect(&mut h, 10.0, 20.0, 110.0, 70.0);
    run_command(&mut h, Command::Crop);
    assert_eq!((active(&h).doc.width, active(&h).doc.height), (100, 50));
    assert_eq!(last_history(&h), "Crop");

    // A red square on white: Trim based on the top-left color keeps it
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.state_mut().state.foreground = Color::from_rgba8([255, 255, 255, 255]);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 30.0, 10.0, 40.0, 30.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    run_command(&mut h, Command::Trim);
    assert!(h.state().state.trim_dialog.is_some());
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().state.trim_dialog.is_none());
    assert_eq!((active(&h).doc.width, active(&h).doc.height), (10, 20));
    assert_eq!(last_history(&h), "Trim");
}

#[test]
#[ignore]
fn screenshot_trim_dialog() {
    let mut h = harness(Vec::new());
    probe_document(&mut h);
    run_command(&mut h, crate::commands::Command::Trim);
    shot(&mut h, "trim_dialog");
    shot_dialog(&mut h, "trim", 258.0, 248.0);
}

#[test]
fn adjustments_invert_and_threshold_preview() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // The reference document is #141414 gray
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::I);
    h.run_steps(2);
    assert_eq!(composite_pixel(&mut h, 5, 5), [0xeb, 0xeb, 0xeb, 255]);
    assert_eq!(last_history(&h), "Invert");

    // Threshold previews while open; Cancel puts the pixels back
    run_command(&mut h, Command::Threshold);
    assert!(h.state().state.adjust_dialog.is_some());
    h.run_steps(2);
    assert_eq!(composite_pixel(&mut h, 5, 5), [255, 255, 255, 255]);
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(h.state().state.adjust_dialog.is_none());
    assert_eq!(composite_pixel(&mut h, 5, 5), [0xeb, 0xeb, 0xeb, 255]);
    assert_eq!(last_history(&h), "Invert");

    // Posterize to 2 levels with Enter
    run_command(&mut h, Command::Posterize);
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(composite_pixel(&mut h, 5, 5), [255, 255, 255, 255]);
    assert_eq!(last_history(&h), "Posterize");
}

#[test]
#[ignore]
fn screenshot_threshold_dialog() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    run_command(&mut h, crate::commands::Command::Threshold);
    shot(&mut h, "threshold_dialog");
}

#[test]
fn levels_dialog_sets_the_black_point() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::L);
    h.run_steps(3);
    assert!(h.state().state.adjust_dialog.is_some());
    // The input black field has focus with its text selected
    h.event(egui::Event::Text("20".into()));
    h.run_steps(3);
    assert_eq!(composite_pixel(&mut h, 5, 5), [0, 0, 0, 255]);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().state.adjust_dialog.is_none());
    assert_eq!(composite_pixel(&mut h, 5, 5), [0, 0, 0, 255]);
    assert_eq!(last_history(&h), "Levels");
}

#[test]
#[ignore]
fn screenshot_adjustment_dialogs() {
    use crate::commands::Command;
    for (command, name) in [
        (Command::Levels, "levels_dialog"),
        (Command::HueSaturation, "hue_saturation_dialog"),
        (Command::Exposure, "exposure_dialog"),
        (Command::BrightnessContrast, "brightness_contrast_dialog"),
        (Command::ColorBalance, "color_balance_dialog"),
        (Command::Curves, "curves_dialog"),
        (Command::ChannelMixer, "channel_mixer_dialog"),
        (Command::SelectiveColor, "selective_color_dialog"),
        (Command::Vibrance, "vibrance_dialog"),
        (Command::Posterize, "posterize_dialog"),
        (Command::PhotoFilter, "photo_filter_dialog"),
        (Command::BlackWhite, "black_white_dialog"),
        (Command::Threshold, "threshold_dialog"),
        (Command::GradientMap, "gradient_map_dialog"),
    ] {
        let mut h = harness(Vec::new());
        reference_document(&mut h);
        run_command(&mut h, command);
        shot(&mut h, name);
    }
}

#[test]
fn filters_apply_and_repeat_with_last_filter() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A white square on the dark background
    h.state_mut().state.foreground = Color::from_rgba8([255, 255, 255, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    assert!(!Command::LastFilter.enabled(&h.state().state));

    run_command(&mut h, Command::GaussianBlur);
    h.event(egui::Event::Text("4".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Gaussian Blur");
    let once = composite_pixel(&mut h, 100, 120);
    assert!(once[0] > 0x14 && once[0] < 255, "{once:?}");

    // Ctrl+Cmd+F runs it again with the same radius
    let states = active(&h).history.states().len();
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::CTRL, egui::Key::F);
    h.run_steps(3);
    assert_eq!(active(&h).history.states().len(), states + 1);
    assert_eq!(last_history(&h), "Gaussian Blur");
    // Blurred further: the edge pixel moves on toward the background
    let twice = composite_pixel(&mut h, 100, 120);
    assert!(twice[0] < once[0], "{once:?} {twice:?}");
}

#[test]
#[ignore]
fn screenshot_filter_dialogs() {
    use crate::commands::Command;
    for (command, name) in [
        (Command::GaussianBlur, "gaussian_blur_dialog"),
        (Command::AddNoise, "add_noise_dialog"),
        (Command::Offset, "offset_dialog"),
        (Command::UnsharpMask, "unsharp_mask_dialog"),
        (Command::MotionBlur, "motion_blur_dialog"),
        (Command::Minimum, "minimum_dialog"),
        (Command::Emboss, "emboss_dialog"),
        (Command::SurfaceBlur, "surface_blur_dialog"),
        (Command::DustAndScratches, "dust_and_scratches_dialog"),
        (Command::Mosaic, "mosaic_dialog"),
        (Command::BoxBlur, "box_blur_dialog"),
        (Command::CustomFilter, "custom_filter_dialog"),
        (Command::TraceContour, "trace_contour_dialog"),
    ] {
        // A small flat document: the pane shows it whole, centered
        let mut h = harness(Vec::new());
        color_document(&mut h, [120, 160, 200]);
        run_command(&mut h, command);
        h.run_steps(2);
        shot(&mut h, name);
    }
    // The Distort dialogs, at the values of the Photoshop captures
    for (command, name, value) in [
        (Command::Twirl, "twirl_dialog", "120"),
        (Command::Pinch, "pinch_dialog", "-40"),
        (Command::Spherize, "spherize_dialog", "70"),
        (Command::PolarCoordinates, "polar_coordinates_dialog", ""),
        (Command::Wind, "wind_dialog", ""),
    ] {
        let mut h = harness(Vec::new());
        color_document(&mut h, [120, 160, 200]);
        run_command(&mut h, command);
        if !value.is_empty() {
            h.event(egui::Event::Text(value.into()));
        }
        h.run_steps(3);
        shot(&mut h, name);
    }
}

#[test]
fn saving_reverting_and_closing_with_unsaved_changes() {
    let mut h = harness(Vec::new());
    let id = h.state().state.active_doc.unwrap();
    let title = |h: &Harness<'_, OpenPhotoApp>| crate::doc_tabs::title(active(h));
    assert!(!title(&h).ends_with('*'));

    // An edit marks the document as changed
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    assert!(title(&h).ends_with(" *"), "{}", title(&h));

    // Saving as a Photoshop document takes its name and clears the mark
    let dir = std::env::temp_dir().join(format!("openphoto-ui-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("Saved.psd");
    assert!(crate::actions::save_to(
        &mut h.state_mut().state,
        id,
        path.clone(),
        false
    ));
    assert!(title(&h).starts_with("Saved.psd @"), "{}", title(&h));
    assert!(!title(&h).ends_with('*'));
    // Undoing past the save is a change; redoing back to it is not
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Z);
    h.run_steps(2);
    assert!(active(&h).is_dirty());
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::Z);
    h.run_steps(2);
    assert!(!active(&h).is_dirty());

    // Revert (F12) goes back to the file
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Backspace);
    h.run_steps(2);
    let filled = composite_pixel(&mut h, 5, 5);
    h.key_press(egui::Key::F12);
    h.run_steps(2);
    assert_ne!(composite_pixel(&mut h, 5, 5), filled);
    assert_eq!(last_history(&h), "Revert");
    assert!(!active(&h).is_dirty());

    // Closing a changed document asks first: Escape keeps it open...
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::W);
    h.run_steps(2);
    assert_eq!(h.state().state.save_prompt, Some(id));
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert_eq!(h.state().state.save_prompt, None);
    assert!(h.state().state.docs.contains_key(&id));
    // ...and Don't Save (Cmd+D) closes it
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::W);
    h.run_steps(2);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    assert!(!h.state().state.docs.contains_key(&id));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
#[ignore]
fn screenshot_save_prompt() {
    let mut h = harness(Vec::new());
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::W);
    h.run_steps(3);
    shot(&mut h, "save_prompt");
}

#[test]
fn eyedropper_magic_wand_and_lassos() {
    use op_tools::Tool;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A red square at 100..140
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);

    // Eyedropper: click picks the foreground, Alt-click the background
    h.state_mut().state.foreground = Color::WHITE;
    h.state_mut().state.select_tool(Tool::Eyedropper);
    let inside = doc_point(&h, 120.0, 120.0);
    click(&mut h, inside);
    assert_eq!(h.state().state.foreground.to_rgba8(), [255, 0, 0, 255]);
    let outside = doc_point(&h, 50.0, 50.0);
    h.hover_at(outside);
    for pressed in [true, false] {
        h.event_modifiers(
            egui::Event::PointerButton {
                pos: outside,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::ALT,
            },
            Modifiers::ALT,
        );
        h.step();
    }
    h.run_steps(2);
    assert_eq!(
        h.state().state.background.to_rgba8(),
        [0x14, 0x14, 0x14, 255]
    );
    // A 5×5 average across the square's corner mixes the two colors
    h.state_mut().state.eyedropper.size = 5;
    let corner = doc_point(&h, 100.5, 100.5);
    click(&mut h, corner);
    let mixed = h.state().state.foreground.to_rgba8();
    assert!(mixed[0] > 0x14 && mixed[0] < 255, "{mixed:?}");

    // Magic Wand selects the square (anti-aliased: the ring around it is
    // partly selected)
    h.state_mut().state.select_tool(Tool::MagicWand);
    click(&mut h, inside);
    let s = active(&h).doc.selection().unwrap();
    assert_eq!((s.get(100, 100), s.get(139, 139)), (255, 255));
    assert!(s.get(99, 120) < 255 && s.get(97, 120) == 0);
    assert_eq!(last_history(&h), "Magic Wand");
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);

    // Polygonal Lasso: three corners, then Enter
    h.state_mut().state.select_tool(Tool::PolygonalLasso);
    for (x, y) in [(10.0, 10.0), (60.0, 10.0), (10.0, 60.0)] {
        let p = doc_point(&h, x, y);
        click(&mut h, p);
    }
    assert_eq!(active(&h).lasso.as_ref().map(|l| l.points.len()), Some(3));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    let s = active(&h).doc.selection().unwrap();
    assert_eq!(s.get(20, 20), 255);
    assert_eq!(s.get(50, 50), 0);
    assert_eq!(last_history(&h), "Polygonal Lasso");

    // Lasso: a freehand drag (right, then down-left) replaces it
    h.state_mut().state.select_tool(Tool::Lasso);
    let a = doc_point(&h, 200.0, 200.0);
    let path = [(260.0, 200.0), (200.0, 260.0)].map(|(x, y)| doc_point(&h, x, y));
    h.hover_at(a);
    h.event(egui::Event::PointerButton {
        pos: a,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for p in path {
        for k in 1..=4 {
            let from = h.ctx.input(|i| i.pointer.latest_pos()).unwrap_or(a);
            h.event(egui::Event::PointerMoved(
                from + (p - from) * (k as f32 / 4.0),
            ));
            h.step();
        }
    }
    h.event(egui::Event::PointerButton {
        pos: path[1],
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(3);
    assert_eq!(last_history(&h), "Lasso");
    assert!(active(&h).lasso.is_none());
    let s = active(&h).doc.selection().unwrap();
    assert_eq!((s.get(210, 210), s.get(20, 20)), (255, 0));
}

#[test]
fn gradient_tool_paints_foreground_to_background() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([0, 0, 0, 255]);
    h.state_mut().state.background = Color::from_rgba8([255, 255, 255, 255]);
    h.key_press(egui::Key::G);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::Gradient);
    let (a, b) = (doc_point(&h, 100.0, 300.0), doc_point(&h, 600.0, 300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert_eq!(last_history(&h), "Gradient");
    assert_eq!(composite_pixel(&mut h, 50, 10), [0, 0, 0, 255]);
    assert_eq!(composite_pixel(&mut h, 700, 700), [255, 255, 255, 255]);
    let mid = composite_pixel(&mut h, 350, 500)[0];
    assert!((118..=138).contains(&mid), "{mid}");
    shot(&mut h, "gradient");
}

#[test]
fn transform_bar_takes_typed_numbers() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::T);
    h.run_steps(2);
    // W: 50 (the bar's center is 45 pt down), Enter: W and H halve, linked
    click(&mut h, at_pt(390.0, 45.0));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("50".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let t = active(&h)
        .free_transform
        .as_ref()
        .expect("still transforming");
    assert!(
        (t.scale.0 - 0.5).abs() < 1e-4 && (t.scale.1 - 0.5).abs() < 1e-4,
        "{:?}",
        t.scale
    );
    // The angle: 90
    click(&mut h, at_pt(570.0, 45.0));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("90".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let t = active(&h).free_transform.as_ref().unwrap();
    assert!((t.angle.to_degrees() - 90.0).abs() < 1e-3);
    // The center hasn't moved
    let (cx, cy) = t.reference_now();
    assert!((cx - 120.0).abs() < 0.01 && (cy - 120.0).abs() < 0.01);
}

#[test]
fn transform_selection_moves_only_the_outline() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    assert!(!Command::TransformSelection.enabled(&h.state().state));
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    let before = composite_pixel(&mut h, 120, 120);
    run_command(&mut h, Command::TransformSelection);
    assert!(active(&h).free_transform.as_ref().unwrap().selection_only);
    let (a, b) = (doc_point(&h, 120.0, 120.0), doc_point(&h, 170.0, 130.0));
    drag(&mut h, a, b, Modifiers::NONE);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    let sel = active(&h).doc.selection().unwrap().bounds().unwrap();
    assert!(
        (sel.0 as i32 - 150).abs() <= 1 && (sel.1 as i32 - 110).abs() <= 1,
        "{sel:?}"
    );
    assert_eq!(composite_pixel(&mut h, 120, 120), before);
    assert_eq!(last_history(&h), "Transform Selection");
}

#[test]
fn warp_pulls_the_surface() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 160.0, 160.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    run_command(&mut h, crate::menu::WARP);
    let t = active(&h).free_transform.as_ref().unwrap();
    assert!(t.warp.is_some());
    // A corner control point dragged out by (30, 30)
    let (a, b) = (doc_point(&h, 160.0, 160.0), doc_point(&h, 190.0, 190.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let mesh = active(&h).free_transform.as_ref().unwrap().warp.unwrap();
    assert!(
        (mesh.points[15].0 - 190.0).abs() < 1.0,
        "{:?}",
        mesh.points[15]
    );
    // Pulling the middle of the surface up
    let (a, b) = (doc_point(&h, 130.0, 130.0), doc_point(&h, 130.0, 110.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let mesh = active(&h).free_transform.as_ref().unwrap().warp.unwrap();
    let mid = mesh.at(0.5, 0.5);
    assert!(mid.1 < 135.0, "{mid:?}");
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(active(&h).free_transform.is_none());
    assert_eq!(last_history(&h), "Warp");
    // The stretched corner is red now
    assert!(layer_pixel(&h, 1, 180, 180)[3] > 0);
}

#[test]
fn transform_distort_from_the_menu() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    // Edit > Transform > Distort: the bottom-right corner alone goes out
    run_command(&mut h, crate::menu::TRANSFORM_MODES[3]);
    assert_eq!(
        active(&h).free_transform.as_ref().unwrap().mode,
        crate::state::TransformMode::Distort
    );
    let (a, b) = (doc_point(&h, 140.0, 140.0), doc_point(&h, 200.0, 200.0));
    drag(&mut h, a, b, Modifiers::NONE);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(active(&h).free_transform.is_none());
    assert_eq!(last_history(&h), "Free Transform");
    // The far corner is red now; the top-left one hasn't moved
    assert_eq!(layer_pixel(&h, 1, 185, 185)[0], 255);
    assert!(layer_pixel(&h, 1, 185, 185)[3] > 200);
    assert_eq!(layer_pixel(&h, 1, 101, 101), [255, 0, 0, 255]);
    assert_eq!(layer_pixel(&h, 1, 180, 105)[3], 0);
    // Transform Again repeats it
    assert!(Command::TransformAgain.enabled(&h.state().state));
}

#[test]
fn free_transform_moves_scales_and_cancels() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // The background can't be transformed without a selection
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::T);
    h.run_steps(2);
    assert_eq!(
        h.state().state.alert.as_deref(),
        Some("Could not complete the Free Transform command because the layer is locked.")
    );
    h.state_mut().state.alert = None;

    // A red square 100..140 on a new layer
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);

    // Cmd+T, drag inside the box by (100, 50), Enter
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::T);
    h.run_steps(2);
    assert!(active(&h).free_transform.is_some());
    let (a, b) = (doc_point(&h, 120.0, 120.0), doc_point(&h, 220.0, 170.0));
    drag(&mut h, a, b, Modifiers::NONE);
    // Previewed while still transforming
    assert_eq!(layer_pixel(&h, 1, 210, 160), [255, 0, 0, 255]);
    shot(&mut h, "free_transform");
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(active(&h).free_transform.is_none());
    assert_eq!(last_history(&h), "Free Transform");
    assert_eq!(layer_pixel(&h, 1, 210, 160), [255, 0, 0, 255]);
    assert_eq!(layer_pixel(&h, 1, 110, 110)[3], 0);

    // Scale by the bottom-right handle, then Escape: nothing changes
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::T);
    h.run_steps(2);
    let (a, b) = (doc_point(&h, 240.0, 190.0), doc_point(&h, 280.0, 230.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let t = active(&h).free_transform.as_ref().unwrap();
    assert!((t.scale.0 - 2.0).abs() < 0.05, "{:?}", t.scale);
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(active(&h).free_transform.is_none());
    assert_eq!(layer_pixel(&h, 1, 260, 210)[3], 0);
    assert_eq!(last_history(&h), "Free Transform");

    // Shift+Cmd+T moves it by (100, 50) again
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::T);
    h.run_steps(2);
    assert_eq!(layer_pixel(&h, 1, 310, 210), [255, 0, 0, 255]);
    assert_eq!(last_history(&h), "Transform Again");
}

#[test]
fn crop_tool_crops_to_the_box() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press(egui::Key::C);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::Crop);
    // The box starts on the whole canvas
    let full = active(&h).crop.unwrap().rect;
    assert_eq!(full.size(), egui::vec2(734.0, 811.0));
    // Pull the bottom-right handle in by (234, 311). By default (not
    // Classic Mode) the box stays in the middle of the view and the top
    // left corner stays on the image, so the corner moves twice as far
    // on the image, as in Photoshop: 266 × 189
    let (a, b) = (doc_point(&h, 734.0, 811.0), doc_point(&h, 500.0, 500.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let r = active(&h).crop.unwrap().rect;
    assert!(
        (r.width() - 266.0).abs() < 2.0 && (r.height() - 189.0).abs() < 2.0,
        "{r:?}"
    );
    assert_eq!(r.min, egui::Pos2::ZERO);
    // ...and the view follows: the box is centered
    let state = active(&h);
    let center = crate::document_view::to_screen(state, r.center(), 2.0 * crate::theme::UI_SCALE);
    assert!((center - state.view.viewport.center()).length() < 1.0);
    // In Classic Mode the box itself moves
    h.key_press(egui::Key::Escape);
    h.state_mut().state.crop_options.classic = true;
    h.run_steps(2);
    let (a, b) = (doc_point(&h, 734.0, 811.0), doc_point(&h, 500.0, 500.0));
    drag(&mut h, a, b, Modifiers::NONE);
    shot(&mut h, "crop");
    let r = active(&h).crop.unwrap().rect;
    assert!(
        (r.width() - 500.0).abs() < 2.0 && (r.height() - 500.0).abs() < 2.0,
        "{r:?}"
    );
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let d = &active(&h).doc;
    assert!((d.width as i32 - 500).abs() <= 2 && (d.height as i32 - 500).abs() <= 2);
    assert_eq!(last_history(&h), "Crop");
    // The box covers the new canvas again
    let r = active(&h).crop.unwrap().rect;
    assert_eq!(r.size(), egui::vec2(d.width as f32, d.height as f32));
    // Another tool drops the box
    h.key_press(egui::Key::M);
    h.run_steps(2);
    assert!(active(&h).crop.is_none());
}

#[test]
fn crop_shield_presets_and_growing_the_canvas() {
    let mut h = harness(Vec::new());
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let doc = op_core::Document::new_with_background("white", 400, 300, Color::WHITE);
    app.add_document(doc, "New");
    app.background = Color::from_rgba8([0, 0, 255, 255]);
    h.run_steps(6);
    h.key_press(egui::Key::C);
    h.run_steps(2);
    // 1 : 1 (Square): the largest centered square
    h.state_mut()
        .state
        .crop_options
        .choose(crate::crop_tool::PRESET_GROUPS[2][0]);
    let state = h.state_mut().state.active().unwrap();
    let aspect = 1.0;
    state.crop = Some(crate::crop_tool::fitted(state, aspect));
    h.run_steps(3);
    let r = active(&h).crop.unwrap().rect;
    assert_eq!(
        r,
        egui::Rect::from_min_max(egui::pos2(50.0, 0.0), egui::pos2(350.0, 300.0))
    );
    // The shield over white, outside the box: 75% of the pasteboard gray
    // mixed in linear light is 141 (Photoshop 2026, measured)
    let image = h.render().expect("render frame");
    let px = |p: egui::Pos2| ((p.x * 2.0 * UI_SCALE) as u32, (p.y * 2.0 * UI_SCALE) as u32);
    let p = px(doc_point(&h, 20.0, 150.0));
    let gray = image.get_pixel(p.0, p.1).0[0];
    assert!(gray.abs_diff(141) <= 2, "{gray}");
    let p = px(doc_point(&h, 200.0, 150.0));
    assert_eq!(image.get_pixel(p.0, p.1).0[0], 255);
    // A box past the right edge grows the canvas, the new part in the
    // background color
    let state = h.state_mut().state.active().unwrap();
    state.crop = Some(crate::state::CropBox {
        rect: egui::Rect::from_min_max(egui::pos2(300.0, 0.0), egui::pos2(500.0, 300.0)),
        angle: 0.0,
        drag: None,
    });
    h.state_mut()
        .state
        .crop_options
        .choose(crate::crop_tool::CropPreset::SizeResolution);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let d = &active(&h).doc;
    assert_eq!((d.width, d.height), (200, 300));
    let bg = d.layers[0].image().unwrap();
    assert_eq!(bg.pixel(50, 10), [255, 255, 255, 255]);
    assert_eq!(bg.pixel(150, 10), [0, 0, 255, 255]);
    assert_eq!(last_history(&h), "Crop");
}

#[test]
fn crop_rotation_and_straighten() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press(egui::Key::C);
    h.run_steps(2);
    let angle = |h: &Harness<'_, OpenPhotoApp>| active(h).crop.unwrap().angle.to_degrees();
    // Dragging outside the box, a quarter turn clockwise about its center,
    // turns the image clockwise: the box turns the other way on it
    let center = doc_point(&h, 367.0, 405.5);
    let r = 500.0;
    drag(
        &mut h,
        center + egui::vec2(r, 0.0),
        center + egui::vec2(0.0, r),
        Modifiers::SHIFT,
    );
    assert!((angle(&h) + 90.0).abs() < 0.5, "{}", angle(&h));
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert_eq!(angle(&h), 0.0);
    // Straighten: a line 10° below level levels the image, and the box
    // shrinks to fit on it
    h.state_mut().state.crop_options.straightening = true;
    h.run_steps(1);
    let a = doc_point(&h, 100.0, 300.0);
    let t = 10f32.to_radians();
    drag(
        &mut h,
        a,
        a + egui::vec2(t.cos(), t.sin()) * 300.0,
        Modifiers::NONE,
    );
    assert!((angle(&h) - 10.0).abs() < 0.5, "{}", angle(&h));
    assert!(!h.state().state.crop_options.straightening);
    let r = active(&h).crop.unwrap().rect;
    assert!(r.width() < 734.0 && r.height() < 811.0);
    // Cropping turns the image; the box was inside it, so no corner shows
    // the background color
    h.state_mut().state.background = Color::from_rgba8([0, 255, 0, 255]);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let d = &active(&h).doc;
    assert!(
        (d.width as f32 - r.width()).abs() <= 1.0,
        "{} {}",
        d.width,
        r.width()
    );
    let bg = d.layers[0].image().unwrap();
    for (x, y) in [
        (0, 0),
        (d.width - 1, 0),
        (0, d.height - 1),
        (d.width - 1, d.height - 1),
    ] {
        assert_ne!(bg.pixel(x, y)[1], 255, "corner ({x}, {y})");
    }
    assert_eq!(last_history(&h), "Crop");
}

#[test]
fn image_size_resamples_proportionally() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::I);
    h.run_steps(3);
    assert!(h.state().state.image_size_dialog.is_some());
    // Width (in inches, as Photoshop opens it) has focus with its text
    // selected; the chain updates Height. 5.097 in at 72 ppi is 367 px
    h.event(egui::Event::Text("5.097".into()));
    h.run_steps(3);
    shot(&mut h, "image_size");
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().state.image_size_dialog.is_none());
    let d = &active(&h).doc;
    assert_eq!((d.width, d.height), (367, 405));
    assert_eq!(last_history(&h), "Image Size");
    assert_eq!(composite_pixel(&mut h, 100, 100), [0x14, 0x14, 0x14, 255]);
}

#[test]
fn rulers_and_guides() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::R);
    h.run_steps(3);
    assert!(h.state().state.view.rulers);
    // The top ruler runs along the top of the document window (y ≈ 100 pt)
    drag(
        &mut h,
        at_pt(500.0, 100.0),
        at_pt(500.0, 300.0),
        Modifiers::NONE,
    );
    let guides = active(&h).doc.guides.clone();
    assert_eq!(guides.len(), 1);
    assert!(!guides[0].vertical);
    assert_eq!(last_history(&h), "New Guide");
    let y = guides[0].position;
    // Grid on, for the screenshot
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Quote);
    h.run_steps(2);
    shot(&mut h, "rulers");

    // The Move tool drags it down by 50 points
    h.key_press(egui::Key::V);
    h.run_steps(2);
    drag(
        &mut h,
        at_pt(500.0, 300.0),
        at_pt(500.0, 350.0),
        Modifiers::NONE,
    );
    assert_eq!(last_history(&h), "Move Guide");
    let moved = active(&h).doc.guides[0].position;
    assert!(moved > y + 10.0, "{y} -> {moved}");
    // Dropped on the ruler it is deleted; undo brings it back
    drag(
        &mut h,
        at_pt(500.0, 350.0),
        at_pt(500.0, 100.0),
        Modifiers::NONE,
    );
    assert!(active(&h).doc.guides.is_empty());
    assert_eq!(last_history(&h), "Delete Guide");
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Z);
    h.run_steps(2);
    assert_eq!(active(&h).doc.guides.len(), 1);
    // Locked guides stay put
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::Semicolon);
    h.run_steps(2);
    assert!(h.state().state.view.lock_guides);
}

#[test]
fn modify_selection_and_grow() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    assert!(!Command::ModifyExpand.enabled(&h.state().state));
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    run_command(&mut h, Command::ModifyExpand);
    h.event(egui::Event::Text("5".into()));
    h.run_steps(2);
    shot(&mut h, "expand_selection");
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(
        active(&h).doc.selection().unwrap().bounds(),
        Some((95, 95, 145, 145))
    );
    assert_eq!(last_history(&h), "Expand");

    // Shift+F6 opens Feather
    h.key_press_modifiers(Modifiers::SHIFT, egui::Key::F6);
    h.run_steps(2);
    assert!(h.state().state.modify_dialog.is_some());
    h.key_press(egui::Key::Escape);
    h.run_steps(2);

    // The whole document is one color: Similar selects all of it
    run_command(&mut h, Command::Similar);
    assert_eq!(
        active(&h).doc.selection().unwrap().bounds(),
        Some((0, 0, 734, 811))
    );
    assert_eq!(last_history(&h), "Similar");
}

fn alt_click(h: &mut Harness<'_, OpenPhotoApp>, pos: Pos2) {
    h.hover_at(pos);
    for pressed in [true, false] {
        h.event_modifiers(
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::ALT,
            },
            Modifiers::ALT,
        );
        h.step();
    }
    h.run_steps(2);
}

#[test]
fn retouching_tools() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A red square at 100..140 to clone, then fill the rest gray
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);

    // Dodge (O) lightens the dark gray background
    h.key_press(egui::Key::O);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::Dodge);
    let p = doc_point(&h, 300.0, 300.0);
    click(&mut h, p);
    assert_eq!(last_history(&h), "Dodge Tool");
    assert!(composite_pixel(&mut h, 300, 300)[0] > 0x14);

    // Clone Stamp (S) without a source point refuses, like Photoshop
    h.key_press(egui::Key::S);
    h.run_steps(2);
    let target = doc_point(&h, 400.0, 400.0);
    click(&mut h, target);
    assert!(
        h.state()
            .state
            .alert
            .as_deref()
            .unwrap()
            .contains("area to clone has not been defined")
    );
    h.state_mut().state.alert = None;
    // Alt-click the square's center, then paint at (400, 400)
    let source = doc_point(&h, 120.0, 120.0);
    alt_click(&mut h, source);
    assert!(active(&h).clone_source.is_some());
    click(&mut h, target);
    assert_eq!(last_history(&h), "Clone Stamp");
    // A soft brush: nearly full strength at its center
    let cloned = composite_pixel(&mut h, 400, 400);
    assert!(cloned[0] >= 250 && cloned[1] <= 5, "{cloned:?}");

    // History Brush (Y) paints the document back as it was opened
    h.key_press(egui::Key::Y);
    h.run_steps(2);
    click(&mut h, target);
    assert_eq!(last_history(&h), "History Brush");
    let restored = composite_pixel(&mut h, 400, 400);
    assert!(restored[0] <= 0x18 && restored[1] == 0x14, "{restored:?}");
}

#[test]
fn layer_masks() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A red layer over the dark background
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    // The background can't take a mask
    assert!(Command::MaskRevealAll.enabled(&h.state().state));

    // Hide Selection: the selected square shows the background through
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    run_command(&mut h, Command::MaskHideSelection);
    assert_eq!(last_history(&h), "Add Layer Mask");
    assert!(active(&h).doc.editing_mask());
    assert_eq!(composite_pixel(&mut h, 120, 120), [0x14, 0x14, 0x14, 255]);
    assert_eq!(composite_pixel(&mut h, 300, 300), [255, 0, 0, 255]);
    shot(&mut h, "layer_mask");

    // Painting black on the mask hides more; the layer's pixels stay red
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    select_rect(&mut h, 200.0, 200.0, 240.0, 240.0);
    h.state_mut().state.foreground = Color::from_rgba8([0, 0, 0, 255]);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    assert_eq!(composite_pixel(&mut h, 220, 220), [0x14, 0x14, 0x14, 255]);
    assert_eq!(layer_pixel(&h, 1, 220, 220), [255, 0, 0, 255]);

    // Disabled, the mask shows everything; Apply bakes it into the layer
    run_command(&mut h, Command::MaskToggle);
    assert_eq!(last_history(&h), "Disable Layer Mask");
    assert_eq!(composite_pixel(&mut h, 120, 120), [255, 0, 0, 255]);
    run_command(&mut h, Command::MaskToggle);
    run_command(&mut h, Command::MaskApply);
    assert!(active(&h).doc.layers[1].mask.is_none());
    assert_eq!(layer_pixel(&h, 1, 120, 120)[3], 0);
}

#[test]
fn quick_mask_mode() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    select_rect(&mut h, 100.0, 100.0, 400.0, 400.0);
    h.key_press(egui::Key::Q);
    h.run_steps(2);
    assert!(active(&h).doc.quick_mask.is_some());
    assert_eq!(last_history(&h), "Quick Mask");
    assert!(crate::doc_tabs::title(active(&h)).contains("(Quick Mask/8"));
    // Outside the old selection is tinted red
    let tinted = active_canvas_pixel(&mut h, 50, 50);
    assert!(tinted[0] > tinted[1] + 50, "{tinted:?}");
    // Paint black inside it with the brush
    h.state_mut().state.foreground = Color::from_rgba8([0, 0, 0, 255]);
    h.key_press(egui::Key::B);
    h.run_steps(2);
    let p = doc_point(&h, 250.0, 250.0);
    click(&mut h, p);
    shot(&mut h, "quick_mask");
    h.key_press(egui::Key::Q);
    h.run_steps(2);
    let s = active(&h).doc.selection().unwrap();
    // The soft brush leaves its center almost fully masked
    assert!(s.get(250, 250) < 10);
    assert_eq!(s.get(150, 150), 255);
    assert_eq!(s.get(50, 50), 0);
    assert!(active(&h).doc.quick_mask.is_none());
}

/// A pixel of the canvas as displayed (with the Quick Mask tint).
fn active_canvas_pixel(h: &mut Harness<'_, OpenPhotoApp>, x: u32, y: u32) -> [u8; 4] {
    let app = &mut h.state_mut().state;
    let id = app.active_doc.unwrap();
    let img = app.docs.get_mut(&id).unwrap().canvas_image();
    let i = ((y * img.width + x) * 4) as usize;
    img.pixels[i..i + 4].try_into().unwrap()
}

#[test]
fn shape_tools_make_layers() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([0, 128, 255, 255]);
    h.key_press(egui::Key::U);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::Rectangle);
    let (a, b) = (doc_point(&h, 100.0, 100.0), doc_point(&h, 300.0, 200.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert_eq!(layer_names(&h), ["Background", "Rectangle 1"]);
    assert_eq!(last_history(&h), "Rectangle Tool");
    assert_eq!(composite_pixel(&mut h, 200, 150), [0, 128, 255, 255]);
    assert_eq!(composite_pixel(&mut h, 350, 150), [0x14, 0x14, 0x14, 255]);

    // Shift+U: the Ellipse; Alt draws it around the press point
    h.key_press_modifiers(Modifiers::SHIFT, egui::Key::U);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::Ellipse);
    let (c, d) = (doc_point(&h, 500.0, 500.0), doc_point(&h, 560.0, 560.0));
    drag(&mut h, c, d, Modifiers::ALT);
    assert_eq!(layer_names(&h), ["Background", "Rectangle 1", "Ellipse 1"]);
    // The ellipse spans about 440..560 around (500, 500)
    assert_eq!(composite_pixel(&mut h, 450, 500), [0, 128, 255, 255]);
    assert_eq!(composite_pixel(&mut h, 445, 445), [0x14, 0x14, 0x14, 255]);
    shot(&mut h, "shapes");
}

#[test]
fn type_tool_sets_text() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::WHITE;
    h.state_mut().state.type_options.size_pt = 48.0;
    h.key_press(egui::Key::T);
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::HorizontalType);
    let p = doc_point(&h, 100.0, 200.0);
    click(&mut h, p);
    assert!(active(&h).text_edit.is_some());
    // Single-key tool shortcuts are off while typing: "M" is text
    h.event(egui::Event::Text("Hello".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.event(egui::Event::Text("World".into()));
    h.run_steps(2);
    assert_eq!(h.state().state.tool, op_tools::Tool::HorizontalType);
    // Previewed live as a layer
    assert_eq!(layer_names(&h), ["Background", "Hello"]);
    shot(&mut h, "type_tool");
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Enter);
    h.run_steps(2);
    assert!(active(&h).text_edit.is_none());
    assert_eq!(last_history(&h), "Type Tool");
    // White pixels where the "H" stands, above the baseline at y = 200
    let lit = (100..130).any(|x| composite_pixel(&mut h, x, 180)[0] == 255);
    assert!(lit);
    assert_eq!(composite_pixel(&mut h, 100, 230)[0], 0x14);

    // Escape throws the text away
    let q = doc_point(&h, 100.0, 500.0);
    click(&mut h, q);
    h.event(egui::Event::Text("Gone".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Hello"]);
}

#[test]
fn new_document_dialog() {
    let mut h = harness(Vec::new());
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::N);
    h.run_steps(3);
    assert!(h.state().state.new_document_dialog.is_some());
    shot(&mut h, "new_document");
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().state.new_document_dialog.is_none());
    let d = &active(&h).doc;
    assert_eq!(
        (d.title.as_str(), d.width, d.height),
        ("Untitled-2", 1920, 1080)
    );
    assert!(d.layers[0].is_background);

    // A transparent background is a regular "Layer 1"
    let app = &mut h.state_mut().state;
    crate::actions::create_document(
        app,
        "Clear".into(),
        (300, 200),
        150.0,
        crate::dialogs::NewContents::Transparent,
    );
    h.run_steps(2);
    let d = &active(&h).doc;
    assert_eq!(d.layers[0].name, "Layer 1");
    assert!(!d.layers[0].is_background);
    assert_eq!(d.resolution, 150.0);
    assert_eq!(composite_pixel(&mut h, 10, 10)[3], 0);
}

#[test]
fn more_adjustments() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    // Black & White (Alt+Shift+Cmd+B) with the default preset: red is 40%
    h.key_press_modifiers(
        Modifiers::COMMAND | Modifiers::ALT | Modifiers::SHIFT,
        egui::Key::B,
    );
    h.run_steps(3);
    assert!(h.state().state.adjust_dialog.is_some());
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(composite_pixel(&mut h, 5, 5), [102, 102, 102, 255]);
    assert_eq!(last_history(&h), "Black & White");

    // Gradient Map from the foreground (black) to the background (white)
    h.state_mut().state.foreground = Color::from_rgba8([0, 0, 0, 255]);
    run_command(&mut h, Command::GradientMap);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    // 102 gray, Smooth (the dialog's default method)
    let want = op_core::gradient::blend_colors(
        [0; 3],
        [255; 3],
        102.0 / 255.0,
        op_core::gradient::Method::Smooth,
    );
    let px = composite_pixel(&mut h, 5, 5);
    assert_eq!([px[0], px[1], px[2]], want);

    run_command(&mut h, Command::PhotoFilter);
    shot(&mut h, "photo_filter");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
}

#[test]
fn curves_dialog_adds_points() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A mid gray to bend
    h.state_mut().state.foreground = Color::from_rgba8([128, 128, 128, 255]);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.run_steps(2);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::M);
    h.run_steps(3);
    assert!(h.state().state.adjust_dialog.is_some());
    // The graph spans (89, 108)–(346, 365) of the 658 × 445 dialog: click
    // the curve point (128, 192)
    click_dialog(
        &mut h,
        (658.0, 445.0),
        89.0 + 128.0 / 255.0 * 257.0,
        365.0 - 192.0 / 255.0 * 257.0,
    );
    shot(&mut h, "curves");
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Curves");
    let v = composite_pixel(&mut h, 5, 5)[0];
    assert!((185..=198).contains(&v), "{v}");
}

#[test]
fn info_navigator_and_histogram_panels() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press(egui::Key::F8);
    h.run_steps(2);
    assert!(h.state().state.floating.info);
    run_command(&mut h, Command::ToggleNavigator);
    run_command(&mut h, Command::ToggleHistogram);
    // Hovering the document tells the Info panel where the pointer is
    let p = doc_point(&h, 120.0, 80.0);
    h.hover_at(p);
    h.run_steps(3);
    let pointer = active(&h).pointer.unwrap();
    assert!((pointer.x - 120.0).abs() < 1.0 && (pointer.y - 80.0).abs() < 1.0);
    // Each panel in its own place
    let rect = |name: &str| {
        h.ctx
            .memory(|m| m.area_rect(egui::Id::new(("floating-panel", name))))
            .unwrap()
    };
    assert!(!rect("Info").intersects(rect("Navigator")));
    assert!(!rect("Info").intersects(rect("Histogram")));
    shot(&mut h, "floating_panels");
    // The Navigator moves the view: clicking its thumbnail's corner
    let before = active(&h).view.offset;
    let ppp = 2.0;
    crate::document_view::center_on(
        h.state_mut().state.active().unwrap(),
        egui::pos2(0.0, 0.0),
        ppp,
    );
    assert_ne!(active(&h).view.offset, before);
    h.key_press(egui::Key::F8);
    h.run_steps(2);
    assert!(!h.state().state.floating.info);
}

#[test]
#[ignore]
fn screenshot_move_tool() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::WHITE;
    h.state_mut().state.background = Color::from_rgba8([0x14, 0xa5, 0xdc, 255]);
    h.key_press(egui::Key::V);
    h.run_steps(3);
    shot(&mut h, "move_tool");
}

/// Gray levels Photoshop 2026 shows at these points (in points from the
/// window's top-left corner, sampled from a 2x capture of its default
/// workspace with the Move tool). Each one pins a measured edge: the 1 pt
/// lines of dividers and collapse bars, separators, frames and icons. A
/// layout change that moves any of them by a point fails here.
const PHOTOSHOP_PIXELS: &[(&str, f32, f32, u8)] = &[
    ("collapse bar top line", 1305.0, 62.5, 56),
    ("collapse bar", 1305.0, 68.0, 66),
    ("panel tab bar", 1305.0, 90.0, 67),
    ("tab bar line", 1305.0, 102.5, 56),
    ("color body", 1300.0, 150.0, 83),
    ("group gap dark", 1300.0, 222.5, 56),
    ("group gap light", 1300.0, 223.5, 71),
    ("group gap dark2", 1300.0, 224.5, 56),
    ("props tab line", 1300.0, 252.5, 56),
    ("layers gap light", 1300.0, 512.5, 71),
    ("layers tab bar", 1300.0, 530.0, 66),
    ("strip left dark", 985.5, 300.0, 56),
    ("strip left light", 986.5, 300.0, 71),
    ("strip left dark2", 987.5, 300.0, 56),
    ("strip right dark", 1025.5, 300.0, 56),
    ("strip right light", 1026.5, 300.0, 71),
    ("panel body left", 1028.5, 300.0, 83),
    ("toolbar divider dark", 39.5, 300.0, 56),
    ("toolbar divider light", 40.5, 300.0, 71),
    ("toolbar bar line", 20.0, 62.5, 56),
    ("toolbar collapse", 30.0, 68.0, 66),
    ("options sep1", 53.5, 45.0, 62),
    ("options sep2", 102.5, 45.0, 62),
    ("home leg", 24.0, 50.0, 221),
    ("home door", 28.0, 50.0, 83),
    ("fg swatch", 13.0, 720.0, 255),
    ("fg frame", 4.5, 720.0, 54),
    ("bg frame", 33.5, 735.0, 54),
    ("bg white", 32.5, 735.0, 255),
    ("panel fg frame light", 1036.5, 120.0, 140),
    ("sv field left", 1072.5, 150.0, 83),
    ("hue left", 1313.5, 150.0, 83),
    ("doc box border", 1038.5, 270.0, 99),
    ("doc box fill", 1040.5, 262.0, 56),
    ("W field border", 1092.5, 325.0, 102),
    ("W field fill", 1093.5, 320.0, 69),
    ("X field fill", 1172.0, 320.0, 77),
    ("strip line", 1000.0, 143.5, 56),
    ("strip grip", 997.5, 80.0, 69),
    ("menu line", 1340.0, 85.5, 168),
    ("menu gap", 1340.0, 86.5, 66),
    ("layers sep 1", 1300.0, 573.5, 62),
    ("layers sep 2", 1300.0, 601.5, 62),
    ("kind border", 1031.5, 560.0, 94),
    ("blend fill", 1100.0, 590.0, 77),
    ("opacity border", 1208.5, 590.0, 94),
    ("fill field border", 1208.5, 615.0, 94),
    ("eye column", 1031.0, 645.0, 83),
    ("eye divider", 1058.0, 645.0, 69),
    ("row selected", 1200.0, 640.0, 107),
    ("list bg", 1200.0, 700.0, 77),
    ("footer line", 1200.0, 775.5, 62),
    ("new layer outline", 1286.5, 787.5, 221),
    ("eye stroke", 1037.5, 653.0, 221),
    ("thumb frame", 1062.25, 650.0, 46),
];

#[test]
fn layout_matches_photoshop_2026() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::WHITE;
    h.state_mut().state.background = Color::from_rgba8([0x14, 0xa5, 0xdc, 255]);
    h.key_press(egui::Key::V);
    h.run_steps(3);
    let image = h.render().expect("render frame");
    let mut wrong = Vec::new();
    for &(name, x, y, expected) in PHOTOSHOP_PIXELS {
        let [r, g, b, _] = image.get_pixel((x * 2.0) as u32, (y * 2.0) as u32).0;
        let gray = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32).round() as i32;
        if (gray - expected as i32).abs() > 6 {
            wrong.push(format!(
                "{name} at ({x}, {y}): {gray}, Photoshop {expected}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "differs from Photoshop:
{}",
        wrong.join(
            "
"
        )
    );
}

#[test]
fn layers_panel_footer_and_lock_buttons() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Footer: Create a new layer (58.25 pt from the right edge)
    click(&mut h, at_pt(1350.0 - 58.25, 787.5));
    assert_eq!(layer_names(&h), ["Background", "Layer 1"]);
    assert_eq!(last_history(&h), "New Layer");
    // Lock transparent pixels, then Lock position
    click(&mut h, at_pt(1028.0 + 43.5, 542.0 + 73.75));
    click(&mut h, at_pt(1028.0 + 88.0, 542.0 + 73.75));
    let layer = &active(&h).doc.layers[1];
    assert!(layer.lock_transparency && layer.lock_position && !layer.lock_pixels);
    assert_eq!(last_history(&h), "Lock Layer");
    // Lock all locks everything; while it's on the other buttons are inert
    click(&mut h, at_pt(1028.0 + 129.0, 542.0 + 73.75));
    click(&mut h, at_pt(1028.0 + 88.0, 542.0 + 73.75));
    let layer = &active(&h).doc.layers[1];
    assert!(layer.lock_all && layer.pixels_locked() && layer.lock_position);
    // As in Photoshop, only Lock all's button shows set: the other four
    // show unset (their own flags stay underneath)
    let image = h.render().expect("render frame");
    let corner = |x: f32| {
        image
            .get_pixel(
                ((1028.0 + x - 8.0) * 2.0) as u32,
                ((542.0 + 73.75 - 8.0) * 2.0) as u32,
            )
            .0
    };
    let (all_bg, position_bg, panel) = (corner(129.0), corner(88.0), corner(150.0));
    assert_ne!(all_bg, panel, "Lock all shows set");
    assert_eq!(position_bg, panel, "Lock position shows unset");
    assert_eq!(corner(43.5), panel, "Lock transparent pixels shows unset");
    // and the opacity and fill can't be dragged
    for y in [36.5 + 9.5, 64.5 + 9.0] {
        drag(
            &mut h,
            at_pt(1028.0 + 195.0, 542.0 + y),
            at_pt(1028.0 + 150.0, 542.0 + y),
            Modifiers::NONE,
        );
    }
    let layer = &active(&h).doc.layers[1];
    assert_eq!((layer.opacity, layer.fill), (1.0, 1.0));
    // Turning it off brings back the locks underneath
    click(&mut h, at_pt(1028.0 + 129.0, 542.0 + 73.75));
    let layer = &active(&h).doc.layers[1];
    assert!(!layer.lock_all && layer.lock_transparency && layer.lock_position);
    assert!(!layer.pixels_locked());
    // Unlocked, dragging the opacity field changes it
    drag(
        &mut h,
        at_pt(1028.0 + 195.0, 542.0 + 46.0),
        at_pt(1028.0 + 150.0, 542.0 + 46.0),
        Modifiers::NONE,
    );
    assert!(active(&h).doc.layers[1].opacity < 1.0);
    // Prevent auto-nesting has its own flag
    click(&mut h, at_pt(1028.0 + 110.5, 542.0 + 73.75));
    assert!(active(&h).doc.layers[1].lock_nesting);
    // The background's locks can't be changed
    click(&mut h, at_pt(1100.0, 632.0 + 43.5 + 21.0));
    click(&mut h, at_pt(1028.0 + 43.5, 542.0 + 73.75));
    assert!(!active(&h).doc.layers[0].lock_transparency);
    // Footer: Delete layer removes Layer 1 again
    click(&mut h, at_pt(1100.0, 652.0));
    click(&mut h, at_pt(1350.0 - 30.25, 787.5));
    assert_eq!(layer_names(&h), ["Background"]);
    assert_eq!(last_history(&h), "Delete Layer");
}

#[test]
fn lock_layers_dialog() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Only the background: nothing to lock
    assert!(!Command::LockLayers.enabled(&h.state().state));
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    h.state_mut().state.active().unwrap().doc.layers[1].lock_position = true;
    // The menu item opens it with the layer's locks ticked
    run_command(&mut h, Command::LockLayers);
    assert!(h.state().state.lock_dialog.is_some());
    // The dialog is centered: its corner at (540.5, 297) pt
    let at = |x: f32, y: f32| at_pt(540.5 + x, 297.0 + y);
    // Tick Image and Prevent auto-nest, then OK
    click(&mut h, at(58.0, 86.0));
    click(&mut h, at(58.0, 138.0));
    click(&mut h, at(214.0, 60.0));
    assert!(h.state().state.lock_dialog.is_none());
    let layer = &active(&h).doc.layers[1];
    assert!(layer.lock_pixels && layer.lock_position && layer.lock_nesting);
    assert!(!layer.lock_transparency && !layer.lock_all);
    assert_eq!(last_history(&h), "Lock Layers");
    // All (Enter for OK)
    run_command(&mut h, Command::LockLayers);
    click(&mut h, at(58.0, 174.0));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(active(&h).doc.layers[1].lock_all);
    // Cancel leaves the locks alone
    run_command(&mut h, Command::LockLayers);
    click(&mut h, at(58.0, 174.0));
    click(&mut h, at(214.0, 96.0));
    assert!(h.state().state.lock_dialog.is_none());
    assert!(active(&h).doc.layers[1].lock_all);
}

#[test]
fn cmd_slash_toggles_lock_all() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // On the background Photoshop answers with an alert
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Slash);
    h.run_steps(2);
    assert_eq!(
        h.state().state.alert.as_deref(),
        Some("The command \u{201c}Set\u{201d} is not currently available.")
    );
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(h.state().state.alert.is_none());
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    h.state_mut().state.active().unwrap().doc.layers[1].lock_position = true;
    // Locks all without a dialog...
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Slash);
    h.run_steps(2);
    assert!(h.state().state.lock_dialog.is_none());
    assert!(active(&h).doc.layers[1].lock_all);
    assert_eq!(last_history(&h), "Lock Layer");
    // ...and unlocks everything
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Slash);
    h.run_steps(2);
    assert_eq!(active(&h).doc.layers[1].locks(), op_core::Locks::default());
    assert_eq!(last_history(&h), "Unlock Layer");
}

#[test]
fn image_size_dialog_controls() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::I);
    h.run_steps(3);
    // The dialog is centered: its corner at (342.5, 221) pt
    let at = |x: f32, y: f32| at_pt(342.5 + x, 221.0 + y);
    let model = |h: &Harness<'_, OpenPhotoApp>| {
        h.state()
            .state
            .image_size_dialog
            .as_ref()
            .unwrap()
            .model
            .clone()
    };
    // Alt+5 picks Bicubic Sharper
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Num5);
    h.run_steps(2);
    assert_eq!(
        model(&h).method,
        op_core::image_ops::Resample::BicubicSharper
    );
    // The chain button unlinks the proportions
    click(&mut h, at(357.0, 151.0));
    assert!(!model(&h).constrain);
    // Resample off: the pixels are the document's and the chain comes back
    click(&mut h, at(340.5, 223.5));
    let m = model(&h);
    assert!(!m.resample && m.constrain);
    assert_eq!(m.result(), Some((734, 811, 72.0)));
    // ...and Alt+number does nothing
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Num1);
    h.run_steps(2);
    assert_eq!(
        model(&h).method,
        op_core::image_ops::Resample::BicubicSharper
    );
    // Cancel
    click(&mut h, at(400.0, 324.5));
    assert!(h.state().state.image_size_dialog.is_none());
}

#[test]
fn rotate_canvas_dialog() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    run_command(&mut h, Command::RotateArbitrary);
    assert!(h.state().state.rotate_dialog.is_some());
    // The angle is selected: type 90, choose Counter Clockwise (the dialog
    // is centered, its corner at (500, 336) pt), then Enter
    h.event(egui::Event::Text("90".into()));
    h.run_steps(2);
    click(&mut h, at_pt(500.0 + 142.75, 336.0 + 83.75));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(h.state().state.rotate_dialog.is_none());
    let d = &active(&h).doc;
    assert_eq!((d.width, d.height), (811, 734));
    assert_eq!(last_history(&h), "Rotate Canvas");
}

#[test]
#[ignore]
fn screenshot_rotate_canvas_dialog() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    run_command(&mut h, crate::commands::Command::RotateArbitrary);
    h.run_steps(3);
    shot(&mut h, "rotate_canvas");
}

#[test]
#[ignore]
fn screenshot_transform_bar() {
    let mut h = harness(Vec::new());
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let mut doc = op_core::Document::new_with_background("tf", 200, 200, Color::WHITE);
    let mut image = op_core::TiledImage::new(200, 200);
    for y in 20..100 {
        for x in 20..100 {
            image.set_pixel(x, y, [255, 0, 0, 255]);
        }
    }
    let id = doc.new_layer_id();
    doc.layers
        .push(op_core::Layer::raster(id, "Layer 1", image));
    doc.select_layer(id);
    app.add_document(doc, "New");
    h.run_steps(6);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::T);
    h.run_steps(3);
    shot(&mut h, "transform_bar");
}

#[test]
#[ignore]
fn screenshot_warp() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    select_rect(&mut h, 100.0, 100.0, 300.0, 300.0);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    run_command(&mut h, crate::menu::WARP);
    let (a, b) = (doc_point(&h, 200.0, 200.0), doc_point(&h, 240.0, 160.0));
    drag(&mut h, a, b, Modifiers::NONE);
    h.run_steps(2);
    shot(&mut h, "warp");
}

#[test]
#[ignore]
fn screenshot_crop_tool() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press(egui::Key::C);
    h.run_steps(3);
    shot(&mut h, "crop_tool");
    let state = h.state_mut().state.active().unwrap();
    state.crop = Some(crate::crop_tool::fitted_turned(
        state,
        734.0 / 811.0,
        15f32.to_radians(),
    ));
    h.run_steps(3);
    shot(&mut h, "crop_tool_turned");
}

#[test]
#[ignore]
fn screenshot_image_size_dialog() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::I);
    h.run_steps(3);
    shot(&mut h, "image_size_open");
}

#[test]
#[ignore]
fn screenshot_new_layer_dialog() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::N);
    h.run_steps(3);
    shot(&mut h, "new_layer");
}

#[test]
#[ignore]
fn screenshot_lock_layers_dialog() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    run_command(&mut h, crate::commands::Command::LockLayers);
    h.run_steps(3);
    shot(&mut h, "lock_layers");
}

#[test]
fn duplicate_layer_and_layer_from_background_dialogs() {
    use egui_kittest::kittest::Queryable;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let center = |w: f32, hgt: f32| (675.0 - w / 2.0, 400.0 - hgt / 2.0);

    // Duplicate Layer... into this document under a new name
    run_command(&mut h, crate::commands::Command::DuplicateLayer);
    assert!(h.state().state.duplicate_dialog.is_some());
    h.event(egui::Event::Text("Copy A".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Copy A"]);
    assert_eq!(last_history(&h), "Duplicate Layer");

    // ...and to a new document, which opens
    let first = h.state().state.active_doc;
    run_command(&mut h, crate::commands::Command::DuplicateLayer);
    let (x0, y0) = center(447.0, 208.0);
    click(&mut h, at_pt(x0 + 200.0, y0 + 121.5));
    h.get_by_label("New").click();
    h.run_steps(2);
    click(&mut h, at_pt(x0 + 408.0, y0 + 51.0));
    h.run_steps(2);
    let app = &h.state().state;
    assert_ne!(app.active_doc, first);
    assert_eq!(app.docs.len(), 2);
    // The next "Untitled-N" (the counter keeps counting, as in Photoshop)
    assert!(active(&h).doc.title.starts_with("Untitled-"));
    assert_eq!(layer_names(&h), ["Copy A copy"]);

    // Back on the first document: Layer from Background... asks for a name
    h.state_mut().state.active_doc = first;
    h.run_steps(2);
    run_command(&mut h, crate::commands::Command::LayerFromBackground);
    assert!(
        h.state()
            .state
            .new_layer_dialog
            .as_ref()
            .is_some_and(|d| d.kind() == crate::dialogs::NewLayerKind::FromBackground)
    );
    h.event(egui::Event::Text("Base".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Base", "Copy A"]);
    assert!(!active(&h).doc.layers[0].is_background);
    assert_eq!(last_history(&h), "Layer From Background");
}

#[test]
fn double_clicking_the_background_asks_for_a_name() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    double_click(&mut h, at_pt(1130.0, 652.0));
    assert!(
        h.state()
            .state
            .new_layer_dialog
            .as_ref()
            .is_some_and(|d| d.kind() == crate::dialogs::NewLayerKind::FromBackground)
    );
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(active(&h).doc.layers[0].is_background);
}

#[test]
#[ignore]
fn screenshot_duplicate_layer_dialog() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    run_command(&mut h, crate::commands::Command::DuplicateLayer);
    shot(&mut h, "duplicate_layer");
}

#[test]
fn flatten_asks_before_discarding_hidden_layers() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    crate::panels::toggle_active_visibility(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    // Cancel keeps both layers
    run_command(&mut h, crate::commands::Command::FlattenImage);
    assert!(h.state().state.flatten_prompt.is_some());
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(h.state().state.flatten_prompt.is_none());
    assert_eq!(active(&h).doc.layers.len(), 2);
    // "Don't show again", then OK: flattened, and not asked next time
    run_command(&mut h, crate::commands::Command::FlattenImage);
    let (x0, y0) = (675.0 - 130.0, 400.0 - 104.0);
    click(&mut h, at_pt(x0 + 30.0, y0 + 140.0));
    click(&mut h, at_pt(x0 + 189.0, y0 + 178.0));
    assert_eq!(layer_names(&h), ["Background"]);
    assert_eq!(last_history(&h), "Flatten Image");
    assert!(h.state().state.skip_flatten_prompt);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    crate::panels::toggle_active_visibility(h.state_mut().state.active().unwrap());
    run_command(&mut h, crate::commands::Command::FlattenImage);
    assert!(h.state().state.flatten_prompt.is_none());
    assert_eq!(layer_names(&h), ["Background"]);
}

#[test]
fn rename_layer_and_alerts() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // The background can't be renamed this way
    assert!(!Command::RenameLayer.enabled(&h.state().state));
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    run_command(&mut h, Command::RenameLayer);
    assert!(active(&h).renaming.is_some());
    h.event(egui::Event::Text("Ink".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(layer_names(&h), ["Background", "Ink"]);

    // An error alert goes away with Enter
    h.state_mut().state.alert = Some("Could not complete the Copy command.".into());
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert!(h.state().state.alert.is_none());
}

#[test]
#[ignore]
fn screenshot_alerts() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.alert =
        Some("Could not complete the Copy command because the selected area is empty.".into());
    h.run_steps(3);
    shot(&mut h, "alert_error");
    h.state_mut().state.alert = None;
    h.state_mut().state.flatten_prompt = Some(crate::dialogs::alert::Alert::caution(
        "Discard hidden layers?",
    ));
    h.run_steps(3);
    shot(&mut h, "alert_caution");
}

#[test]
fn layers_multi_selection() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    for _ in 0..3 {
        crate::panels::new_layer(h.state_mut().state.active().unwrap());
    }
    h.run_steps(2);
    // Rows 42.5 pt apart from 632: Layer 3, Layer 2, Layer 1, Background
    let row = |k: f32| at_pt(1130.0, 652.0 + 42.5 * k);
    let ids: Vec<op_core::LayerId> = active(&h).doc.layers.iter().map(|l| l.id).collect();
    click(&mut h, row(0.0));
    click_with(&mut h, row(1.0), Modifiers::COMMAND);
    assert_eq!(active(&h).doc.selected_layers(), [ids[2], ids[3]]);
    // Cmd+E with two selected merges them (Merge Layers)
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::E);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Layer 1", "Layer 3"]);
    assert_eq!(last_history(&h), "Merge Layers");
    // Shift-click selects a range; Cmd+, hides them all
    click(&mut h, row(0.0));
    click_with(&mut h, row(1.0), Modifiers::SHIFT);
    assert_eq!(active(&h).doc.selected_layers().len(), 2);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Comma);
    h.run_steps(2);
    assert!(active(&h).doc.layers[1..].iter().all(|l| !l.visible));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Comma);
    h.run_steps(2);
    assert!(active(&h).doc.layers.iter().all(|l| l.visible));
    // Alt+Cmd+A: every layer but the background; delete them with the footer
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::A);
    h.run_steps(2);
    assert_eq!(active(&h).doc.selected_layers().len(), 2);
    click(&mut h, at_pt(1350.0 - 30.25, 787.5));
    assert_eq!(layer_names(&h), ["Background"]);
    run_command(&mut h, Command::DeselectLayers);
    assert!(active(&h).doc.selected_layers().is_empty());
}

#[test]
fn align_buttons_line_up_selected_layers() {
    use crate::commands::Command;
    use op_core::align::Distribute;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Three layers with a 40 px square each, at different places
    for (x0, y0) in [(100u32, 100u32), (200, 150), (300, 300)] {
        let state = h.state_mut().state.active().unwrap();
        crate::panels::new_layer(state);
        let id = state.doc.active_layer.unwrap();
        let image = state.doc.layer_mut(id).unwrap().image_mut().unwrap();
        for y in y0..y0 + 40 {
            for x in x0..x0 + 40 {
                image.set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        state.doc.mark_dirty();
    }
    h.key_press(egui::Key::V);
    h.run_steps(2);
    // One layer: the align buttons are off
    assert!(!Command::Align(op_core::align::Align::Left).enabled(&h.state().state));
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::A);
    h.run_steps(2);
    assert!(Command::Align(op_core::align::Align::Left).enabled(&h.state().state));
    // The options bar's "Align left edges" (the first align button)
    click(&mut h, at_pt(423.0, 45.0));
    assert_eq!(last_history(&h), "Align Left Edges");
    let lefts: Vec<i64> = active(&h).doc.layers[1..]
        .iter()
        .map(|l| {
            let image = l.image().unwrap();
            image.content_bounds().unwrap().0
        })
        .collect();
    assert!(lefts.iter().all(|&x| x == lefts[0]), "{lefts:?}");
    run_command(&mut h, Command::Distribute(Distribute::VerticalCenter));
    assert_eq!(last_history(&h), "Distribute Vertical Centers");
    // Locked layers can't move: the (cached) enabled state follows
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Slash);
    h.run_steps(2);
    assert!(!Command::Align(op_core::align::Align::Left).enabled(&h.state().state));
    assert!(!Command::Distribute(Distribute::Left).enabled(&h.state().state));
}

#[test]
fn layer_groups_in_the_layers_panel() {
    use op_core::LayerKind;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    for _ in 0..2 {
        crate::panels::new_layer(h.state_mut().state.active().unwrap());
    }
    h.run_steps(2);
    // Cmd+G groups the selected layers
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::A);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::G);
    h.run_steps(2);
    assert_eq!(
        layer_names(&h),
        ["Background", "Layer 1", "Layer 2", "Group 1"]
    );
    assert_eq!(last_history(&h), "Group Layers");
    let group = active(&h).doc.layers[3].id;
    assert!(
        active(&h).doc.layers[1..3]
            .iter()
            .all(|l| l.parent == Some(group))
    );
    // Rows: Group 1 (632, 25 pt), Layer 2, Layer 1 (42.5 pt each), Background.
    // The arrow collapses the group (and isn't a history state)
    click(&mut h, at_pt(1028.0 + 38.0, 632.0 + 11.75));
    assert!(matches!(
        active(&h).doc.layers[3].kind,
        LayerKind::Group { collapsed: true }
    ));
    assert_eq!(last_history(&h), "Group Layers");
    click(&mut h, at_pt(1028.0 + 38.0, 632.0 + 11.75));
    assert!(matches!(
        active(&h).doc.layers[3].kind,
        LayerKind::Group { collapsed: false }
    ));
    // Drag Layer 1 (the third row) above the group: out of it
    drag(
        &mut h,
        at_pt(1150.0, 632.0 + 25.0 + 42.5 + 20.0),
        at_pt(1150.0, 633.0),
        Modifiers::NONE,
    );
    assert_eq!(
        layer_names(&h),
        ["Background", "Layer 2", "Group 1", "Layer 1"]
    );
    assert_eq!(active(&h).doc.layers[3].parent, None);
    assert_eq!(last_history(&h), "Layer Order");
    // Shift+Cmd+G on the group ungroups it
    click(&mut h, at_pt(1150.0, 632.0 + 42.5 + 12.0));
    assert_eq!(active(&h).doc.active_layer, Some(group));
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::G);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Layer 2", "Layer 1"]);
    assert_eq!(last_history(&h), "Ungroup Layers");
    // The footer's folder button makes an empty group
    click(&mut h, at_pt(1350.0 - 86.5, 787.5));
    assert!(
        active(&h)
            .doc
            .layers
            .iter()
            .any(|l| l.is_group() && l.name == "Group 1")
    );
    assert_eq!(last_history(&h), "New Group");
}

#[test]
#[ignore]
fn screenshot_layer_groups() {
    let mut h = harness(Vec::new());
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let mut doc = op_core::Document::new_with_background("groups", 200, 200, Color::WHITE);
    let ids: Vec<op_core::LayerId> = (0..5).map(|_| doc.new_layer_id()).collect();
    let (g1, l1, g2, l2, g3) = (ids[0], ids[1], ids[2], ids[3], ids[4]);
    let layer = |id, name: &str, parent| {
        let mut l = op_core::Layer::raster(id, name, op_core::TiledImage::new(200, 200));
        l.parent = parent;
        l
    };
    let a = layer(l1, "Layer 1", Some(g1));
    let b = layer(l2, "Layer 2", Some(g2));
    let mut group2 = op_core::Layer::group(g2, "Group 2");
    group2.parent = Some(g1);
    doc.layers.extend([
        a,
        b,
        group2,
        op_core::Layer::group(g1, "Group 1"),
        op_core::Layer::group(g3, "Group 3"),
    ]);
    doc.select_layer(l1);
    app.add_document(doc, "New");
    h.run_steps(6);
    shot(&mut h, "layer_groups");
}

/// A 200 × 200 document with A, B and C over the background, A and C
/// linked and selected: the panel Photoshop 2026 was captured with.
fn linked_document(h: &mut Harness<'_, OpenPhotoApp>) {
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let mut doc = op_core::Document::new_with_background("link-tmp", 200, 200, Color::WHITE);
    let mut ids = Vec::new();
    for name in ["A", "B", "C"] {
        let id = doc.new_layer_id();
        doc.layers.push(op_core::Layer::raster(
            id,
            name,
            op_core::TiledImage::new(200, 200),
        ));
        ids.push(id);
    }
    doc.set_selected_layers(vec![ids[0], ids[2]]);
    op_core::link::link_selected(&mut doc);
    app.add_document(doc, "New");
    h.run_steps(6);
}

#[test]
#[ignore]
fn screenshot_linked_layers() {
    let mut h = harness(Vec::new());
    linked_document(&mut h);
    shot(&mut h, "linked_layers");
}

/// Row lines, highlights, thumbnail frames, link icons' column and the
/// scrollbar thumb, at the device pixels (2x) of Photoshop 2026's capture.
#[test]
fn layers_rows_and_scrollbar_match_photoshop() {
    let mut h = harness(Vec::new());
    linked_document(&mut h);
    let image = h.render().expect("render frame");
    let gray = |x: u32, y: u32| image.get_pixel(x, y).0[0];
    let mut wrong = Vec::new();
    let mut check = |what: &str, x: u32, y: u32, want: u8| {
        let got = gray(x, y);
        if got.abs_diff(want) > 3 {
            wrong.push(format!("{what} at ({x}, {y}): {got}, Photoshop {want}"));
        }
    };
    // Rows: C (selected) 81 px, then 80 px rows under 2 px lines
    for (y, want) in [
        (1264, 107),
        (1344, 107),
        (1345, 69),
        (1346, 69),
        (1347, 83),
        (1426, 83),
        (1427, 69),
        (1429, 107),
        (1508, 107),
        (1509, 69),
        (1511, 83),
    ] {
        check("row", 2400, y, want);
    }
    // The thumbnails' frames start 9 px into each 82 px row
    for (y, want) in [(1271, 107), (1272, 46), (1353, 83), (1354, 46)] {
        check("thumbnail frame", 2124, y, want);
    }
    // Track and thumb
    for (x, y, want) in [
        (2670, 1300, 74),
        (2683, 1266, 74),
        (2683, 1270, 105),
        (2683, 1508, 105),
        (2683, 1511, 74),
        (2696, 1300, 74),
    ] {
        check("scrollbar", x, y, want);
    }
    assert!(
        wrong.is_empty(),
        "differs from Photoshop:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn alt_click_opens_nested_groups_and_the_list_follows_the_active_layer() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Group 2 inside Group 1, both closed
    let doc = &mut h.state_mut().state.active().unwrap().doc;
    let (outer, inner) = (doc.new_layer_id(), doc.new_layer_id());
    let mut g2 = op_core::Layer::group(inner, "Group 2");
    g2.parent = Some(outer);
    g2.kind = op_core::LayerKind::Group { collapsed: true };
    let mut g1 = op_core::Layer::group(outer, "Group 1");
    g1.kind = op_core::LayerKind::Group { collapsed: true };
    doc.layers.extend([g2, g1]);
    doc.select_layer(outer);
    h.run_steps(2);
    let collapsed = |h: &Harness<'_, OpenPhotoApp>, id| {
        matches!(
            active(h).doc.layer(id).unwrap().kind,
            op_core::LayerKind::Group { collapsed: true }
        )
    };
    // Alt-click Group 1's arrow (x 38, its row on top): both open
    let arrow = at_pt(1028.0 + 38.0, 632.0 + 12.25);
    click_with(&mut h, arrow, Modifiers::ALT);
    assert!(!collapsed(&h, outer) && !collapsed(&h, inner));
    click_with(&mut h, arrow, Modifiers::ALT);
    assert!(collapsed(&h, outer) && collapsed(&h, inner));
    // A plain click opens Group 1 only
    click(&mut h, arrow);
    assert!(!collapsed(&h, outer) && collapsed(&h, inner));

    // Enough layers to scroll; choosing the background brings it into view
    for _ in 0..12 {
        crate::panels::new_layer(h.state_mut().state.active().unwrap());
    }
    h.run_steps(2);
    let bg = active(&h).doc.layers[0].id;
    h.state_mut().state.active().unwrap().doc.select_layer(bg);
    h.run_steps(4);
    let top = active(&h).doc.layers.last().unwrap().id;
    // The list's last row is the background: clicking just above the
    // footer selects it, not some other layer
    h.state_mut().state.active().unwrap().doc.select_layer(top);
    h.run_steps(4);
    h.state_mut().state.active().unwrap().doc.select_layer(bg);
    h.run_steps(4);
    click(&mut h, at_pt(1100.0, 770.0));
    assert_eq!(active(&h).doc.active_layer, Some(bg));
}

#[test]
fn dragging_layers_onto_the_footer_buttons() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    // The top row (Layer 1) and the footer buttons (from the right edge)
    let top_row = at_pt(1100.0, 652.0);
    let button = |x: f32| at_pt(1350.0 - x, 787.5);
    // Onto "Create a new layer": a copy
    drag(&mut h, top_row, button(58.25), Modifiers::NONE);
    assert_eq!(layer_names(&h), ["Background", "Layer 1", "Layer 1 copy"]);
    assert_eq!(last_history(&h), "Duplicate Layer");
    // Onto the trash: deleted without asking
    drag(&mut h, top_row, button(30.25), Modifiers::NONE);
    assert_eq!(layer_names(&h), ["Background", "Layer 1"]);
    assert_eq!(last_history(&h), "Delete Layer");
    // Onto "Create a new group": grouped
    drag(&mut h, top_row, button(86.5), Modifiers::NONE);
    assert_eq!(layer_names(&h), ["Background", "Layer 1", "Group 1"]);
    assert_eq!(last_history(&h), "Create Group from Layers");
    // A group onto the trash goes with its layers, without asking
    drag(&mut h, top_row, button(30.25), Modifiers::NONE);
    assert!(h.state().state.delete_group_prompt.is_none());
    assert_eq!(layer_names(&h), ["Background"]);
    assert_eq!(last_history(&h), "Delete Group");
    // Onto the mask button: a mask on the dragged layer
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.run_steps(2);
    drag(&mut h, top_row, button(139.75), Modifiers::NONE);
    assert!(active(&h).doc.layers[1].mask.is_some());
    assert_eq!(last_history(&h), "Add Layer Mask");
}

/// none, full (Lock all), partial (Lock position), G (Lock all) with
/// "child", and the background: the document Photoshop 2026 was captured
/// with. Returns the child's id.
fn locked_rows_document(h: &mut Harness<'_, OpenPhotoApp>) -> op_core::LayerId {
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let mut doc = op_core::Document::new_with_background("lock-rows", 200, 200, Color::WHITE);
    let layer = |doc: &op_core::Document, name: &str| {
        op_core::Layer::raster(doc.new_layer_id(), name, op_core::TiledImage::new(200, 200))
    };
    let mut child = layer(&doc, "child");
    let mut group = op_core::Layer::group(doc.new_layer_id(), "G");
    group.lock_all = true;
    child.parent = Some(group.id);
    let child_id = child.id;
    let mut partial = layer(&doc, "partial");
    partial.lock_position = true;
    let mut full = layer(&doc, "full");
    full.lock_all = true;
    let none = layer(&doc, "none");
    let none_id = none.id;
    doc.layers.extend([child, group, partial, full, none]);
    doc.select_layer(none_id);
    app.add_document(doc, "New");
    h.run_steps(6);
    child_id
}

/// The lock icons on the rows, at the device pixels of Photoshop 2026's
/// captures: solid for Lock all, hollow for some locks, dimmed for a
/// locked group's layer; and that layer can't be deleted.
#[test]
fn lock_icons_on_the_rows_match_photoshop() {
    let mut h = harness(Vec::new());
    let child = locked_rows_document(&mut h);
    let image = h.render().expect("render frame");
    let gray = |x, y| image.get_pixel(x, y).0[0];
    // "full": the body is solid around the keyhole
    assert!(gray(2617, 1385).abs_diff(221) <= 4, "{}", gray(2617, 1385));
    assert!(gray(2623, 1388) < 120, "keyhole {}", gray(2623, 1388));
    // "partial": a frame with the panel inside
    assert!(gray(2614, 1468).abs_diff(221) <= 30, "{}", gray(2614, 1468));
    assert!(gray(2618, 1468) < 110, "{}", gray(2618, 1468));
    // The child, scrolled into view when chosen: a dimmed solid lock
    h.state_mut()
        .state
        .active()
        .unwrap()
        .doc
        .select_layer(child);
    h.run_steps(6);
    let image = h.render().expect("render frame");
    let g = image.get_pixel(2617, 1507).0[0];
    assert!(g.abs_diff(166) <= 4, "{g}");
    assert!(!crate::commands::Command::DeleteLayer.enabled(&h.state().state));
}

#[test]
#[ignore]
fn screenshot_locked_rows() {
    let mut h = harness(Vec::new());
    let child = locked_rows_document(&mut h);
    shot(&mut h, "locked_rows");
    h.state_mut()
        .state
        .active()
        .unwrap()
        .doc
        .select_layer(child);
    h.run_steps(6);
    shot(&mut h, "locked_rows_child");
}

#[test]
fn link_layers_from_the_panel_and_the_menu() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    for _ in 0..3 {
        crate::panels::new_layer(h.state_mut().state.active().unwrap());
    }
    h.run_steps(2);
    let id = |h: &Harness<'_, OpenPhotoApp>, k: usize| active(h).doc.layers[k].id;
    let (l1, l3) = (id(&h, 1), id(&h, 3));
    // One layer: the link button and the menu item are off
    assert!(!Command::LinkLayers.enabled(&h.state().state));
    // Layer 1 and Layer 3, linked with the footer's link button
    let doc = &mut h.state_mut().state.active().unwrap().doc;
    doc.set_selected_layers(vec![l1, l3]);
    let link_button = at_pt(1350.0 - 196.5, 787.5);
    click(&mut h, link_button);
    assert_eq!(last_history(&h), "Link Layers");
    assert_eq!(op_core::link::linked_with(&active(&h).doc, l1), [l3]);
    assert!(op_core::link::can_unlink(&active(&h).doc));
    // Select Linked Layers from Layer 1 alone selects Layer 3 too
    h.state_mut().state.active().unwrap().doc.select_layer(l1);
    assert!(Command::SelectLinkedLayers.enabled(&h.state().state));
    run_command(&mut h, Command::SelectLinkedLayers);
    assert_eq!(active(&h).doc.selected_layers().len(), 2);
    assert_eq!(active(&h).doc.active_layer, Some(l1));
    // Unlinking Layer 1 leaves Layer 3 linked to nothing
    h.state_mut().state.active().unwrap().doc.select_layer(l1);
    run_command(&mut h, Command::LinkLayers);
    assert_eq!(last_history(&h), "Unlink Layers");
    assert!(!op_core::link::is_linked(&active(&h).doc, l3));
}

#[test]
fn deleting_a_group_asks_what_to_delete() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    for _ in 0..2 {
        crate::panels::new_layer(h.state_mut().state.active().unwrap());
    }
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::A);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::G);
    h.run_steps(2);
    // Group Only (the second of the stacked answers, 34 pt below the first):
    // the layers stay
    run_command(&mut h, Command::DeleteLayer);
    let prompt = h.state().state.delete_group_prompt.clone().expect("asked");
    assert_eq!(
        prompt.message,
        "Delete the group \u{201c}Group 1\u{201d} and its contents or delete only the group?"
    );
    assert_eq!(
        prompt.choices,
        ["Group and Contents", "Group Only", "Cancel"]
    );
    // Three lines of message: buttons from 164 pt in a 276 pt alert
    let (x0, y0) = (675.0 - 130.0, 400.0 - 138.0);
    click(&mut h, at_pt(x0 + 130.0, y0 + 164.0 + 34.0 + 14.0));
    assert_eq!(layer_names(&h), ["Background", "Layer 1", "Layer 2"]);
    assert!(active(&h).doc.layers.iter().all(|l| l.parent.is_none()));
    assert_eq!(last_history(&h), "Delete Layer");
    // Group and Contents (Enter): all of it goes
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::A);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::G);
    h.run_steps(2);
    run_command(&mut h, Command::DeleteLayer);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background"]);
    // Duplicating a group copies its layers
    crate::panels::new_layer(h.state_mut().state.active().unwrap());
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::G);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.run_steps(2);
    assert_eq!(
        layer_names(&h),
        [
            "Background",
            "Layer 1",
            "Group 1",
            "Layer 1",
            "Group 1 copy"
        ]
    );
}

#[test]
fn merge_group_and_reverse() {
    use crate::commands::Command;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    for _ in 0..3 {
        crate::panels::new_layer(h.state_mut().state.active().unwrap());
    }
    // Reverse Layer 1..3
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, egui::Key::A);
    h.run_steps(2);
    run_command(&mut h, Command::ArrangeReverse);
    assert_eq!(
        layer_names(&h),
        ["Background", "Layer 3", "Layer 2", "Layer 1"]
    );
    assert_eq!(last_history(&h), "Reverse");
    // Group them, then Cmd+E merges the group into one layer
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::G);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::E);
    h.run_steps(2);
    assert_eq!(layer_names(&h), ["Background", "Group 1"]);
    assert!(!active(&h).doc.layers[1].is_group());
    assert_eq!(last_history(&h), "Merge Group");
}

/// A point in Photoshop points from the top-left corner of a centered
/// dialog `w` × `h` points large.
fn in_dialog(h: &Harness<'_, OpenPhotoApp>, (w, hgt): (f32, f32), x: f32, y: f32) -> Pos2 {
    let screen = h.ctx.content_rect();
    screen.center() - egui::vec2(pt(w), pt(hgt)) / 2.0 + egui::vec2(pt(x), pt(y))
}

fn click_dialog(h: &mut Harness<'_, OpenPhotoApp>, size: (f32, f32), x: f32, y: f32) {
    let p = in_dialog(h, size, x, y);
    click(h, p);
}

/// A one-color document for the adjustment dialogs.
fn color_document(h: &mut Harness<'_, OpenPhotoApp>, rgb: [u8; 3]) {
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let [r, g, b] = rgb;
    let doc =
        op_core::Document::new_with_background("t", 40, 40, Color::from_rgba8([r, g, b, 255]));
    app.add_document(doc, "Open");
    h.run_steps(4);
}

#[test]
fn brightness_contrast_dialog_types_and_applies() {
    use op_core::adjust::Adjustment;
    let mut h = harness(Vec::new());
    color_document(&mut h, [100, 100, 100]);
    run_command(&mut h, crate::commands::Command::BrightnessContrast);
    // Brightness has the focus with its text selected
    h.event(egui::Event::Text("50".into()));
    h.run_steps(3);
    // Use Legacy
    click_dialog(&mut h, (401.0, 212.0), 26.0, 170.0);
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().state.adjust_dialog.is_none());
    assert_eq!(last_history(&h), "Brightness/Contrast");
    // Legacy brightness shifts every level by 50
    assert_eq!(composite_pixel(&mut h, 5, 5), [150, 150, 150, 255]);

    run_command(&mut h, crate::commands::Command::BrightnessContrast);
    h.event(egui::Event::Text("50".into()));
    h.run_steps(3);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let t = Adjustment::BrightnessContrast {
        brightness: 50,
        contrast: 0,
        legacy: false,
    }
    .tables()
    .unwrap()[0];
    let v = t[150];
    assert_eq!(composite_pixel(&mut h, 5, 5), [v, v, v, 255]);
}

#[test]
fn color_balance_tones_keep_their_values() {
    use op_core::adjust::Adjustment;
    let mut h = harness(Vec::new());
    color_document(&mut h, [100, 100, 100]);
    run_command(&mut h, crate::commands::Command::ColorBalance);
    // Midtones' Cyan to Red has the focus
    h.event(egui::Event::Text("40".into()));
    h.run_steps(3);
    // Shadows: its own values, its first box focused
    click_dialog(&mut h, (447.0, 295.0), 68.0, 60.0);
    h.run_steps(3);
    h.event(egui::Event::Text("-30".into()));
    h.run_steps(3);
    // Preserve Luminosity off
    click_dialog(&mut h, (447.0, 295.0), 26.0, 263.0);
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Color Balance");
    let t = Adjustment::ColorBalance {
        shadows: [-30, 0, 0],
        midtones: [40, 0, 0],
        highlights: [0; 3],
        preserve_luminosity: false,
    }
    .tables()
    .unwrap();
    let px = composite_pixel(&mut h, 5, 5);
    assert_eq!(px, [t[0][100], t[1][100], t[2][100], 255]);
    assert_ne!(px[0], 100);
}

#[test]
fn hue_saturation_ranges_and_colorize() {
    use op_core::adjust::{Adjustment, HueSaturation};
    const SIZE: (f32, f32) = (437.0, 413.0);
    let mut h = harness(Vec::new());
    color_document(&mut h, [230, 40, 40]);
    run_command(&mut h, crate::commands::Command::HueSaturation);
    // Reds, then 60 in its Hue box
    click_dialog(&mut h, SIZE, 104.0, 96.0);
    h.run_steps(2);
    click_dialog(&mut h, SIZE, 280.0, 132.0);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("60".into()));
    h.run_steps(3);
    shot(&mut h, "hue_saturation_reds");
    // Back to Master: its box still says 0
    click_dialog(&mut h, SIZE, 68.0, 96.0);
    h.run_steps(2);
    // Click away from the boxes so Enter goes to OK
    click_dialog(&mut h, SIZE, 150.0, 331.0);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Hue/Saturation");
    let mut reds = HueSaturation::master(0, 0, 0);
    reds.ranges[0].hue = 60;
    let want = Adjustment::HueSaturation(reds);
    let Adjustment::HueSaturation(hs) = want else {
        unreachable!()
    };
    assert_eq!(composite_pixel(&mut h, 5, 5), hs.apply([230, 40, 40, 255]));

    // Colorize starts at saturation 25 with the foreground color's hue
    let [r, g, b, _] = h.state().state.foreground.to_rgba8();
    let hue = op_core::adjust::hue_of([r, g, b]).round() as i32;
    run_command(&mut h, crate::commands::Command::HueSaturation);
    click_dialog(&mut h, SIZE, 26.0, 298.0);
    h.run_steps(3);
    shot(&mut h, "hue_saturation_colorize");
    click_dialog(&mut h, SIZE, 150.0, 331.0);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let colorize = HueSaturation {
        colorize: true,
        ..HueSaturation::master(hue, 25, 0)
    };
    let before = hs.apply([230, 40, 40, 255]);
    assert_eq!(composite_pixel(&mut h, 5, 5), colorize.apply(before));
}

#[test]
fn levels_and_curves_edit_one_channel() {
    let mut h = harness(Vec::new());
    color_document(&mut h, [100, 100, 100]);
    // Levels: ⌥3 picks Red; its input black point 50
    run_command(&mut h, crate::commands::Command::Levels);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Num3);
    h.run_steps(2);
    click_dialog(&mut h, (412.0, 371.0), 80.0, 258.5);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("50".into()));
    h.run_steps(2);
    shot(&mut h, "levels_red");
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Levels");
    // (100 − 50) / 205 of the way: 62
    assert_eq!(composite_pixel(&mut h, 5, 5), [62, 100, 100, 255]);

    // Curves: ⌥4 picks Green; a point at (128, 192) lifts it
    run_command(&mut h, crate::commands::Command::Curves);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Num4);
    h.run_steps(2);
    click_dialog(
        &mut h,
        (658.0, 445.0),
        89.0 + 128.0 / 255.0 * 257.0,
        365.0 - 192.0 / 255.0 * 257.0,
    );
    shot(&mut h, "curves_green");
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Curves");
    let px = composite_pixel(&mut h, 5, 5);
    assert_eq!((px[0], px[2]), (62, 100));
    assert!(px[1] > 130, "{px:?}");
}

#[test]
fn channel_mixer_and_selective_color_apply() {
    let mut h = harness(Vec::new());
    color_document(&mut h, [200, 100, 50]);
    // Channel Mixer: Red output takes 50% red (its first box is focused)
    run_command(&mut h, crate::commands::Command::ChannelMixer);
    h.event(egui::Event::Text("50".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Channel Mixer");
    assert_eq!(composite_pixel(&mut h, 5, 5), [100, 100, 50, 255]);

    // Selective Color: Yellows (second swatch), cyan 100, Absolute
    run_command(&mut h, crate::commands::Command::SelectiveColor);
    click_dialog(&mut h, (473.0, 400.0), 68.0, 96.0);
    h.run_steps(2);
    h.event(egui::Event::Text("100".into()));
    click_dialog(&mut h, (473.0, 400.0), 98.0, 368.5);
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Selective Color");
    let mut colors = [[0; 4]; 9];
    colors[1] = [100, 0, 0, 0];
    let want = op_core::adjust::Adjustment::SelectiveColor {
        colors,
        absolute: true,
    };
    let mut doc =
        op_core::Document::new_with_background("t", 1, 1, Color::from_rgba8([100, 100, 50, 255]));
    op_core::adjust::apply(&mut doc, want).unwrap();
    let px = composite_pixel(&mut h, 5, 5);
    assert_eq!(&px[..], &doc.composite_rgba8()[..4]);
}

#[test]
fn small_uxp_adjustment_dialogs_apply() {
    use crate::commands::Command;
    use op_core::adjust::Adjustment;
    let expect = |h: &mut Harness<'_, OpenPhotoApp>, adjustment: Adjustment, before: [u8; 3]| {
        let [r, g, b] = before;
        let mut doc =
            op_core::Document::new_with_background("t", 1, 1, Color::from_rgba8([r, g, b, 255]));
        op_core::adjust::apply(&mut doc, adjustment).unwrap();
        let want: [u8; 4] = doc.composite_rgba8()[..4].try_into().unwrap();
        assert_eq!(composite_pixel(h, 5, 5), want, "{adjustment:?}");
        let [r, g, b, _] = want;
        [r, g, b]
    };
    let mut h = harness(Vec::new());
    color_document(&mut h, [180, 120, 60]);
    let mut color = [180, 120, 60];
    // Each dialog's first box is focused with its text selected
    for (command, typed, adjustment) in [
        (
            Command::Vibrance,
            "40",
            Adjustment::Vibrance {
                vibrance: 40,
                saturation: 0,
            },
        ),
        (Command::Posterize, "6", Adjustment::Posterize(6)),
        (
            Command::Exposure,
            "0.5",
            Adjustment::Exposure {
                exposure: 0.5,
                offset: 0.0,
                gamma: 1.0,
            },
        ),
        (
            Command::BlackWhite,
            "70",
            Adjustment::BlackWhite {
                weights: [70, 60, 40, 60, 20, 80],
                tint: None,
            },
        ),
    ] {
        run_command(&mut h, command);
        h.event(egui::Event::Text(typed.into()));
        h.run_steps(2);
        h.key_press(egui::Key::Enter);
        h.run_steps(3);
        assert!(h.state().state.adjust_dialog.is_none());
        color = expect(&mut h, adjustment, color);
    }
    // Photo Filter: no box has the focus; Enter applies the default
    run_command(&mut h, Command::PhotoFilter);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Photo Filter");
    expect(
        &mut h,
        Adjustment::PhotoFilter {
            color: [0xec, 0x8a, 0x00],
            density: 25,
            preserve_luminosity: true,
        },
        color,
    );
}

#[test]
fn equalize_asks_about_the_selection() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Without a selection it runs at once
    run_command(&mut h, crate::commands::Command::Equalize);
    assert!(h.state().state.equalize_dialog.is_none());
    assert_eq!(last_history(&h), "Equalize");
    // With one it asks, the entire image chosen
    select_rect(&mut h, 100.0, 100.0, 140.0, 140.0);
    run_command(&mut h, crate::commands::Command::Equalize);
    assert!(
        h.state()
            .state
            .equalize_dialog
            .as_ref()
            .is_some_and(|d| d.entire_image)
    );
    shot(&mut h, "equalize_dialog");
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().state.equalize_dialog.is_none());
    assert_eq!(last_history(&h), "Equalize");
}

#[test]
fn more_filters_from_the_menu() {
    use crate::commands::Command;
    use op_core::filter::Filter;
    let mut h = harness(Vec::new());
    color_document(&mut h, [200, 100, 50]);
    // A dark dot to blur and sharpen
    select_rect(&mut h, 10.0, 10.0, 12.0, 12.0);
    h.state_mut().state.foreground = Color::from_rgba8([0, 0, 0, 255]);
    h.key_press_modifiers(Modifiers::ALT, egui::Key::Backspace);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::D);
    h.run_steps(2);
    for (command, name) in [
        (Command::Blur, "Blur"),
        (Command::BlurMore, "Blur More"),
        (Command::Sharpen, "Sharpen"),
        (Command::SharpenMore, "Sharpen More"),
        (Command::FindEdges, "Find Edges"),
        (Command::Despeckle, "Despeckle"),
        (Command::SharpenEdges, "Sharpen Edges"),
    ] {
        run_command(&mut h, command);
        assert_eq!(last_history(&h), name);
        assert_eq!(h.state().state.last_filter.map(Filter::name), Some(name));
    }
    // Motion Blur and Emboss open their dialogs; Enter applies the defaults
    for (command, filter) in [
        (
            Command::MotionBlur,
            Filter::MotionBlur {
                angle: 0,
                distance: 10,
            },
        ),
        (
            Command::Emboss,
            Filter::Emboss {
                angle: 135,
                height: 3,
                amount: 100,
            },
        ),
        (
            Command::SurfaceBlur,
            Filter::SurfaceBlur {
                radius: 5,
                threshold: 15,
            },
        ),
        (
            Command::DustAndScratches,
            Filter::DustAndScratches {
                radius: 1,
                threshold: 0,
            },
        ),
        (
            Command::TraceContour,
            Filter::TraceContour {
                level: 128,
                upper: true,
            },
        ),
        (
            Command::Wind,
            Filter::Wind {
                method: op_core::filter::WindMethod::Wind,
                from_left: false,
            },
        ),
    ] {
        run_command(&mut h, command);
        assert!(h.state().state.adjust_dialog.is_some());
        h.key_press(egui::Key::Enter);
        h.run_steps(3);
        assert_eq!(h.state().state.last_filter, Some(filter));
    }
}

#[test]
fn distort_filters_from_the_menu() {
    use crate::commands::Command;
    use op_core::filter::{Filter, SpherizeMode};
    let mut h = harness(Vec::new());
    color_document(&mut h, [200, 100, 50]);
    for (command, filter) in [
        (Command::Twirl, Filter::Twirl { angle: 50 }),
        (Command::Pinch, Filter::Pinch { amount: 50 }),
        (
            Command::Spherize,
            Filter::Spherize {
                amount: 100,
                mode: SpherizeMode::Normal,
            },
        ),
        (
            Command::PolarCoordinates,
            Filter::PolarCoordinates { to_polar: true },
        ),
    ] {
        run_command(&mut h, command);
        assert!(h.state().state.adjust_dialog.is_some());
        h.key_press(egui::Key::Enter);
        h.run_steps(3);
        assert_eq!(h.state().state.last_filter, Some(filter));
        assert_eq!(last_history(&h), filter.name());
    }
}

#[test]
fn filter_dialogs_remember_their_last_values() {
    use crate::commands::Command;
    use op_core::filter::Filter;
    let mut h = harness(Vec::new());
    color_document(&mut h, [120, 160, 200]);
    run_command(&mut h, Command::GaussianBlur);
    // The radius field has the focus with its text selected
    h.event(egui::Event::Text("4".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(
        h.state().state.last_filter,
        Some(Filter::GaussianBlur { radius: 4.0 })
    );
    // Opened again, the dialog starts at 4; Enter applies it again
    run_command(&mut h, Command::GaussianBlur);
    let effect = h
        .state()
        .state
        .adjust_dialog
        .as_ref()
        .and_then(|d| d.effect());
    assert_eq!(
        effect,
        Some(crate::dialogs::Effect::Filter(Filter::GaussianBlur {
            radius: 4.0
        }))
    );
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    // Another filter keeps its own defaults
    run_command(&mut h, Command::BoxBlur);
    let effect = h
        .state()
        .state
        .adjust_dialog
        .as_ref()
        .and_then(|d| d.effect());
    assert_eq!(
        effect,
        Some(crate::dialogs::Effect::Filter(Filter::BoxBlur {
            radius: 1
        }))
    );
}

#[test]
fn custom_filter_types_a_kernel_and_remembers_it() {
    use crate::commands::Command;
    use op_core::filter::Filter;
    let mut h = harness(Vec::new());
    color_document(&mut h, [120, 160, 200]);
    run_command(&mut h, Command::CustomFilter);
    // The top-left cell has the focus
    h.event(egui::Event::Text("2".into()));
    h.run_steps(2);
    let kernel = |h: &Harness<'_, OpenPhotoApp>| match h
        .state()
        .state
        .adjust_dialog
        .as_ref()
        .and_then(|d| d.effect())
    {
        Some(crate::dialogs::Effect::Filter(Filter::Custom { kernel, .. })) => kernel,
        other => panic!("{other:?}"),
    };
    let k = kernel(&h);
    assert_eq!((k[0], k[12], k[7]), (2, 5, -1));
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(last_history(&h), "Custom");
    assert!(matches!(
        h.state().state.last_filter,
        Some(Filter::Custom {
            scale: 1,
            offset: 0,
            ..
        })
    ));
    // Opened again, it starts from the kernel just applied
    run_command(&mut h, Command::CustomFilter);
    assert_eq!(kernel(&h)[0], 2);
}

/// Dragging with the Move tool, Brush and Eraser on a 3000 × 1080
/// document keeps up: each frame composites the document again, so this
/// catches compositing, layer-bounds scans or thumbnails going slow again
/// (moving took 56 ms a frame before compositing went parallel and bounds
/// scans stopped reading every pixel).
#[test]
fn dragging_on_a_large_document_keeps_up() {
    for key in [egui::Key::V, egui::Key::B, egui::Key::E] {
        let mut h = harness(Vec::new());
        {
            let app = &mut h.state_mut().state;
            crate::actions::close_all(app);
            let mut doc = op_core::Document::new_with_background("big", 3000, 1080, Color::WHITE);
            let mut img = op_core::TiledImage::new(3000, 1080);
            for y in 100..900 {
                for x in 500..2500 {
                    img.set_pixel(x, y, [200, 30, 30, 255]);
                }
            }
            let id = doc.new_layer_id();
            doc.insert_above_active(op_core::Layer::raster(id, "L", img));
            app.add_document(doc, "Open");
        }
        h.run_steps(6);
        h.key_press(key);
        h.run_steps(2);
        let from = at_pt(700.0, 450.0);
        h.hover_at(from);
        h.event(egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.step();
        let before = active(&h).doc.revision();
        let frames = 20;
        let t = std::time::Instant::now();
        for i in 1..=frames {
            h.event(egui::Event::PointerMoved(
                from + egui::vec2(i as f32 * 3.0, (i % 7) as f32),
            ));
            h.step();
        }
        let per_frame = t.elapsed() / frames;
        assert_ne!(
            active(&h).doc.revision(),
            before,
            "{key:?} changed the document"
        );
        // Generous: about 10 ms here, with room for a loaded machine
        assert!(
            per_frame < std::time::Duration::from_millis(40),
            "{key:?}: {per_frame:?} a frame"
        );
    }
}

fn right_click(h: &mut Harness<'_, OpenPhotoApp>, pos: Pos2) {
    h.hover_at(pos);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        h.step();
    }
    h.run_steps(3);
}

#[test]
#[ignore]
fn screenshot_tool_flyout() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    right_click(&mut h, at_pt(19.5, 105.5));
    shot(&mut h, "tool_flyout");
    // The healing tools (with the Remove Tool)
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    right_click(&mut h, at_pt(19.5, 283.0));
    shot(&mut h, "heal_flyout");
}

/// The frame color's columns and rows along a line through the flyout.
fn flyout_frame(image: &image::RgbaImage) -> (Vec<u32>, Vec<u32>) {
    let frame = |x: u32, y: u32| image.get_pixel(x, y).0[..3] == [0x3e, 0x3e, 0x3e];
    let xs = (60..400).filter(|&x| frame(x, 230)).collect();
    let ys = (150..300).filter(|&y| frame(300, y)).collect();
    (xs, ys)
}

#[test]
fn tool_flyout_matches_photoshop_and_switches_tools() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Right-click the Move tool: Photoshop 2026's flyout is 130 x 40 pt
    // with its frame from (36, 92) to (166, 132)
    right_click(&mut h, at_pt(19.5, 105.5));
    let image = h.render().expect("render frame");
    let (xs, ys) = flyout_frame(&image);
    let (left, right) = (xs[0] as f32 / 2.0, *xs.last().unwrap() as f32 / 2.0 + 0.5);
    let (top, bottom) = (ys[0] as f32 / 2.0, *ys.last().unwrap() as f32 / 2.0 + 0.5);
    assert!(
        (left - 36.0).abs() <= 1.0 && (top - 92.0).abs() <= 1.0,
        "{left}, {top}"
    );
    assert!(
        (right - left - 130.0).abs() <= 1.5,
        "width {}",
        right - left
    );
    assert!(
        (bottom - top - 40.0).abs() <= 0.5,
        "height {}",
        bottom - top
    );
    // The second row (19 pt rows inside a 1 pt frame) picks the Artboard tool
    click(&mut h, at_pt(80.0, 92.0 + 1.0 + 19.0 + 9.5));
    assert_eq!(h.state().state.tool, op_tools::Tool::Artboard);
    let image = h.render().expect("render frame");
    assert!(flyout_frame(&image).0.is_empty(), "the flyout closes");
}

/// Each tool's toolbar icon against Photoshop's, cropped from its toolbar
/// at 2x into `$PS_ICONS/<Tool>.png`: prints the overlap of the bright
/// pixels and writes a sheet (ours over Photoshop's) to
/// `target/ui-shots/icon_compare.png`.
#[test]
#[ignore]
fn compare_tool_icons_with_photoshop() {
    let Ok(dir) = std::env::var("PS_ICONS") else {
        return;
    };
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let only = std::env::var("ICONS").ok();
    let mut tiles = Vec::new();
    let mut scores = Vec::new();
    for slot in op_tools::TOOLBAR {
        for &tool in *slot {
            let name = format!("{tool:?}");
            if only
                .as_ref()
                .is_some_and(|o| !o.split(',').any(|n| n == name))
            {
                continue;
            }
            let Ok(ps) = image::open(format!("{dir}/{name}.png")) else {
                continue;
            };
            let ps = ps.to_rgba8();
            h.state_mut().state.select_tool(tool);
            h.run_steps(2);
            let image = h.render().expect("render frame");
            let cy = (211.0 + 51.8 * tool.slot() as f32) as u32;
            let ours = image::imageops::crop_imm(&image, 10, cy - 26, 56, 52).to_image();
            let bright = |img: &image::RgbaImage, x: u32, y: u32| {
                let p = img.get_pixel(x, y).0;
                (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3 > 140
            };
            let (mut both, mut either) = (0, 0);
            for y in 3..46 {
                for x in 0..46 {
                    let (a, b) = (bright(&ours, x, y), bright(&ps, x, y));
                    both += (a && b) as u32;
                    either += (a || b) as u32;
                }
            }
            let iou = both as f32 / either.max(1) as f32;
            // The shift of ours that would overlap best
            let mut best = (0.0f32, 0i32, 0i32);
            for dy in -4i32..=4 {
                for dx in -4i32..=4 {
                    let (mut b2, mut e2) = (0, 0);
                    for y in 6..42i32 {
                        for x in 4..42i32 {
                            let a = bright(&ours, (x + dx) as u32, (y + dy) as u32);
                            let b = bright(&ps, x as u32, y as u32);
                            b2 += (a && b) as u32;
                            e2 += (a || b) as u32;
                        }
                    }
                    let v = b2 as f32 / e2.max(1) as f32;
                    if v > best.0 {
                        best = (v, dx, dy);
                    }
                }
            }
            eprintln!(
                "{name}: best {:.2} at ours shifted by ({}, {})",
                best.0, best.1, best.2
            );
            scores.push((iou, name.clone()));
            tiles.push((name, ours, ps));
        }
    }
    scores.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (iou, name) in &scores {
        eprintln!("{iou:.2} {name}");
    }
    let rows = tiles.len().div_ceil(10) as u32;
    let mut sheet =
        image::RgbaImage::from_pixel(10 * 60 * 3, rows * 110 * 3, image::Rgba([255, 0, 255, 255]));
    for (i, (_, ours, ps)) in tiles.iter().enumerate() {
        let (x, y) = ((i % 10) as u32 * 60 * 3, (i / 10) as u32 * 110 * 3);
        let big = |img: &image::RgbaImage| {
            image::imageops::resize(img, 56 * 3, 52 * 3, image::imageops::FilterType::Nearest)
        };
        image::imageops::overlay(&mut sheet, &big(ours), x as i64, y as i64);
        image::imageops::overlay(&mut sheet, &big(ps), x as i64, (y + 54 * 3) as i64);
    }
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-shots");
    std::fs::create_dir_all(&dir).unwrap();
    sheet.save(dir.join("icon_compare.png")).unwrap();
}

/// The bright pixels' box of each tool icon in a toolbar cell (56 x 52
/// device pixels around the button, as `compare_tool_icons_with_photoshop`
/// crops it), measured on Photoshop 2026's toolbar: x0, y0, x1, y1.
const PS_ICON_EXTENTS: &[(&str, [u32; 4])] = &[
    ("AddAnchorPoint", [10, 11, 43, 42]),
    ("AdjustmentBrush", [12, 10, 44, 41]),
    ("ArtHistoryBrush", [12, 7, 45, 40]),
    ("Artboard", [8, 5, 41, 36]),
    ("BackgroundEraser", [10, 8, 43, 39]),
    ("Blur", [18, 10, 37, 39]),
    ("Brush", [12, 11, 41, 40]),
    ("Burn", [12, 14, 43, 36]),
    ("CloneStamp", [16, 11, 41, 40]),
    ("ColorReplacement", [6, 7, 41, 40]),
    ("ColorSampler", [6, 7, 44, 40]),
    ("ContentAwareMove", [11, 13, 44, 38]),
    ("ConvertPoint", [22, 13, 40, 38]),
    ("Count", [11, 10, 45, 38]),
    ("Crop", [8, 6, 45, 39]),
    ("CurvaturePen", [10, 11, 45, 40]),
    ("CustomShape", [10, 11, 45, 40]),
    ("DeleteAnchorPoint", [10, 13, 43, 42]),
    ("DirectSelection", [18, 11, 37, 43]),
    ("Dodge", [12, 12, 41, 41]),
    ("Ellipse", [12, 13, 41, 38]),
    ("EllipticalMarquee", [12, 10, 43, 37]),
    ("Eraser", [12, 13, 41, 39]),
    ("Eyedropper", [12, 10, 42, 40]),
    ("Frame", [12, 10, 45, 37]),
    ("FreeformPen", [6, 11, 45, 40]),
    ("Gradient", [12, 12, 43, 37]),
    ("Hand", [12, 11, 43, 43]),
    ("HealingBrush", [13, 10, 42, 39]),
    ("HistoryBrush", [10, 11, 43, 40]),
    ("HorizontalType", [18, 13, 41, 40]),
    ("HorizontalTypeMask", [16, 13, 43, 40]),
    ("Lasso", [14, 10, 45, 41]),
    ("Line", [14, 13, 41, 40]),
    ("MagicEraser", [7, 9, 43, 39]),
    ("MagicWand", [13, 7, 45, 40]),
    ("MagneticLasso", [13, 8, 43, 41]),
    ("MixerBrush", [6, 11, 43, 40]),
    ("Move", [13, 10, 40, 37]),
    ("Note", [16, 11, 41, 38]),
    ("ObjectSelection", [14, 8, 44, 41]),
    ("PaintBucket", [12, 9, 45, 41]),
    ("Patch", [12, 10, 43, 39]),
    ("PathSelection", [20, 12, 39, 43]),
    ("PatternStamp", [6, 11, 41, 40]),
    ("Pen", [14, 13, 43, 42]),
    ("Pencil", [12, 9, 43, 40]),
    ("PerspectiveCrop", [16, 6, 41, 39]),
    ("Polygon", [14, 13, 43, 38]),
    ("PolygonalLasso", [12, 8, 43, 41]),
    ("QuickSelection", [10, 11, 45, 37]),
    ("Rectangle", [12, 13, 43, 38]),
    ("RectangularMarquee", [12, 10, 43, 35]),
    ("RedEye", [10, 7, 45, 39]),
    ("RotateView", [12, 9, 45, 42]),
    ("Ruler", [12, 17, 45, 30]),
    ("SelectionBrush", [12, 6, 43, 40]),
    ("Sharpen", [16, 11, 41, 39]),
    ("SingleColumnMarquee", [24, 8, 29, 37]),
    ("SingleRowMarquee", [12, 22, 43, 27]),
    ("Slice", [13, 14, 45, 36]),
    ("SliceSelect", [10, 10, 45, 36]),
    ("Smudge", [16, 12, 41, 41]),
    ("Sponge", [12, 12, 43, 39]),
    ("SpotHealingBrush", [10, 7, 42, 39]),
    ("Triangle", [13, 12, 42, 38]),
    ("VerticalType", [7, 13, 41, 40]),
    ("VerticalTypeMask", [7, 13, 43, 40]),
    ("Zoom", [12, 12, 43, 43]),
];

#[test]
fn tool_icons_match_photoshops_extents() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let mut off = Vec::new();
    for slot in op_tools::TOOLBAR {
        for &tool in *slot {
            let name = format!("{tool:?}");
            let Some((_, want)) = PS_ICON_EXTENTS.iter().find(|(n, _)| *n == name) else {
                continue;
            };
            h.state_mut().state.select_tool(tool);
            h.run_steps(2);
            let image = h.render().expect("render frame");
            let cy = (211.0 + 51.8 * tool.slot() as f32) as u32;
            let cell = image::imageops::crop_imm(&image, 10, cy - 26, 56, 52).to_image();
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
            for y in 3..46 {
                for x in 0..46 {
                    let p = cell.get_pixel(x, y).0;
                    if (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3 > 140 {
                        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                    }
                }
            }
            let got = [x0, y0, x1, y1];
            if got.iter().zip(want).any(|(g, w)| g.abs_diff(*w) > 2) {
                off.push(format!("{name}: {got:?}, Photoshop {want:?}"));
            }
        }
    }
    assert!(off.is_empty(), "{}", off.join("\n"));
    // Every tool has a drawing but Remove (its Photoshop icon is unmeasured)
    assert_eq!(PS_ICON_EXTENTS.len(), 69);
}

/// Every tool's options bar, cropped as the Photoshop captures are (window
/// rows 56–126 at 2x), into `target/ui-shots/bars/<Tool>.png`.
#[test]
#[ignore]
fn screenshot_options_bars() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-shots/bars");
    std::fs::create_dir_all(&dir).unwrap();
    for slot in op_tools::TOOLBAR {
        for &tool in *slot {
            h.state_mut().state.select_tool(tool);
            h.run_steps(3);
            let image = h.render().expect("render frame");
            let bar = image::imageops::crop_imm(&image, 0, 56, image.width(), 70).to_image();
            bar.save(dir.join(format!("{tool:?}.png"))).unwrap();
        }
    }
}

/// Separators and frame edges (fields, pop-ups, buttons, pressed boxes)
/// along each tool's options bar, measured on Photoshop 2026 (bar points,
/// from x 104): the bars laid out with `options_kit`.
type BarMarks = (&'static str, &'static [f32], &'static [f32]);
const PS_BAR_MARKS: &[BarMarks] = &[
    (
        "RectangularMarquee",
        &[222.0, 412.0, 749.0],
        &[
            110.0, 135.0, 276.0, 329.0, 453.5, 530.5, 574.0, 614.5, 699.5, 740.0, 760.5, 869.5,
        ],
    ),
    (
        "EllipticalMarquee",
        &[222.0, 412.0, 749.0],
        &[
            110.0, 135.0, 276.0, 329.0, 453.5, 530.5, 574.0, 614.5, 699.5, 740.0, 760.5, 869.5,
        ],
    ),
    (
        "SingleRowMarquee",
        &[222.0, 410.0, 747.0],
        &[
            110.0, 135.0, 274.0, 327.0, 451.5, 528.5, 572.0, 612.5, 697.5, 738.0, 758.5, 867.5,
        ],
    ),
    (
        "SingleColumnMarquee",
        &[222.0, 410.0, 747.0],
        &[
            110.0, 135.0, 274.0, 327.0, 451.5, 528.5, 572.0, 612.5, 697.5, 738.0, 758.5, 867.5,
        ],
    ),
    (
        "Lasso",
        &[222.0, 412.0],
        &[110.0, 135.0, 276.0, 329.0, 423.5, 532.5],
    ),
    (
        "PolygonalLasso",
        &[222.0, 412.0],
        &[110.0, 135.0, 276.0, 329.0, 423.5, 532.5],
    ),
    (
        "MagneticLasso",
        &[222.0, 412.0, 701.0, 748.0],
        &[
            110.0, 135.0, 276.0, 329.0, 457.5, 498.5, 557.5, 598.5, 666.0, 692.0, 759.5, 868.5,
        ],
    ),
    (
        "MagicWand",
        &[222.0, 803.5],
        &[
            110.0, 135.0, 298.0, 412.5, 476.5, 523.5, 815.0, 905.0, 907.0, 925.0, 945.0, 1054.0,
        ],
    ),
    (
        "ObjectSelection",
        &[212.0, 400.0, 487.0, 677.0, 712.0],
        &[
            104.0, 129.0, 217.5, 304.5, 405.0, 482.0, 717.5, 807.5, 813.5, 831.5, 837.5, 946.5,
        ],
    ),
    (
        "QuickSelection",
        &[202.0, 253.0, 322.5, 530.0],
        &[
            116.0, 141.0, 277.0, 317.5, 543.5, 633.5, 635.5, 653.5, 669.5, 778.5,
        ],
    ),
    (
        "Brush",
        &[190.0, 368.0, 513.5, 638.5, 789.5, 859.0],
        &[
            230.0, 359.0, 422.5, 459.0, 473.5, 547.5, 584.0, 598.5, 702.5, 739.0, 753.5, 813.5,
            854.0,
        ],
    ),
    (
        "Pencil",
        &[190.0, 493.0, 644.0, 713.5],
        &[
            230.0, 329.0, 398.0, 434.5, 449.0, 557.0, 593.5, 608.0, 668.0, 708.5,
        ],
    ),
    (
        "Eraser",
        &[190.0, 423.5, 548.5, 699.5, 769.5],
        &[
            230.0, 283.0, 332.5, 369.0, 383.5, 457.5, 494.0, 508.5, 612.5, 649.0, 663.5, 723.5,
            764.0,
        ],
    ),
    (
        "BackgroundEraser",
        &[160.0, 255.0, 394.0, 514.0, 591.5],
        &[
            169.0, 194.0, 297.0, 389.0, 453.0, 489.5, 504.0, 542.0, 582.5,
        ],
    ),
    (
        "MagicEraser",
        &[282.5, 488.5, 604.0],
        &[165.0, 199.0, 543.0, 579.5, 594.0],
    ),
    (
        "ColorReplacement",
        &[156.0, 277.5, 679.5, 749.0],
        &[
            196.0, 272.5, 282.5, 307.5, 401.5, 493.5, 552.5, 589.0, 603.5, 703.5, 744.0,
        ],
    ),
    (
        "MixerBrush",
        &[186.0],
        &[
            233.0, 258.0, 261.0, 286.0, 289.0, 418.0, 446.0, 482.5, 497.0, 530.5, 567.0, 581.5,
            608.5, 645.0, 659.5, 691.5, 728.0, 742.5, 802.5, 839.0, 853.5, 904.5, 945.0,
        ],
    ),
    (
        "CloneStamp",
        &[221.0, 500.5, 622.5, 690.5, 754.0, 940.5],
        &[
            260.0, 363.0, 411.5, 448.0, 462.5, 533.5, 570.0, 584.5, 645.5, 686.0, 801.5, 903.5,
        ],
    ),
    (
        "PatternStamp",
        &[188.0, 482.5, 605.0, 672.5, 869.0],
        &[
            227.0, 345.5, 393.5, 430.0, 444.5, 516.0, 552.5, 567.0, 628.0, 668.5, 676.5, 705.5,
            717.5,
        ],
    ),
    (
        "HistoryBrush",
        &[190.0, 467.5, 592.5, 662.5],
        &[
            230.0, 329.0, 377.5, 414.0, 428.5, 501.5, 538.0, 552.5, 616.5, 657.0,
        ],
    ),
    (
        "ArtHistoryBrush",
        &[190.0, 467.5, 801.0, 871.0],
        &[
            230.0, 329.0, 377.5, 414.0, 428.5, 505.0, 606.5, 640.5, 685.0, 744.0, 780.5, 795.0,
            825.0, 865.5,
        ],
    ),
    (
        "Blur",
        &[198.0, 471.0, 548.5],
        &[242.0, 341.0, 410.0, 446.5, 461.0, 499.0, 539.5],
    ),
    (
        "Sharpen",
        &[198.0, 471.0, 548.5, 765.5],
        &[242.0, 341.0, 410.0, 446.5, 461.0, 499.0, 539.5],
    ),
    (
        "Smudge",
        &[198.0, 471.0, 548.5],
        &[242.0, 341.0, 410.0, 446.5, 461.0, 499.0, 539.5],
    ),
    (
        "Dodge",
        &[198.0, 478.5, 556.0, 660.5],
        &[245.5, 318.0, 379.5, 416.0, 430.5, 506.5, 547.0],
    ),
    (
        "Burn",
        &[198.0, 486.0, 563.5, 667.5],
        &[245.5, 318.0, 387.0, 423.5, 438.0, 514.0, 554.5],
    ),
    (
        "Sponge",
        &[198.0, 457.0, 535.0, 615.0],
        &[242.0, 319.0, 358.0, 394.5, 409.0, 485.0, 525.5],
    ),
    (
        "SpotHealingBrush",
        &[156.0, 300.0, 613.0, 727.5, 797.0],
        &[196.0, 295.0, 336.5, 425.0, 513.5, 608.0, 751.5, 792.0],
    ),
    (
        "HealingBrush",
        &[190.0, 334.0, 497.0, 548.0, 878.5, 948.0, 987.0],
        &[
            230.0, 329.0, 381.0, 440.0, 492.0, 502.0, 531.0, 543.0, 738.0, 839.5, 902.5, 943.0,
            1051.5, 1079.0, 1093.5,
        ],
    ),
    (
        "Patch",
        &[222.0, 371.0, 597.5, 738.5, 855.0],
        &[
            110.0, 135.0, 266.5, 362.0, 380.0, 431.0, 502.5, 607.5, 684.0, 688.5, 717.5, 729.5,
            803.0, 830.5, 845.0,
        ],
    ),
    (
        "ContentAwareMove",
        &[222.0, 332.0, 528.5],
        &[
            110.0, 135.0, 266.0, 323.0, 401.5, 424.0, 438.5, 481.5, 504.0, 518.5,
        ],
    ),
    (
        "RedEye",
        &[359.5],
        &[164.0, 200.5, 215.0, 302.5, 339.0, 353.5],
    ),
    (
        "Gradient",
        &[230.0, 338.0, 489.0, 629.0],
        &[
            115.0, 221.0, 241.0, 313.0, 327.0, 349.0, 359.0, 374.0, 685.0, 760.0,
        ],
    ),
    (
        "PaintBucket",
        &[568.5],
        &[
            110.0, 189.5, 193.5, 222.5, 234.5, 273.5, 371.5, 419.0, 455.5, 470.0, 530.0, 564.0,
        ],
    ),
    ("Eyedropper", &[532.5], &[177.0, 291.5, 344.0, 523.5]),
    (
        "ColorSampler",
        &[300.0, 379.5],
        &[177.0, 291.5, 310.0, 370.0],
    ),
    (
        "Ruler",
        &[230.0, 356.5, 549.0, 694.5],
        &[702.0, 801.5, 811.5, 857.0],
    ),
    (
        "Note",
        &[333.0, 411.5, 491.0],
        &[150.5, 324.0, 421.5, 482.0],
    ),
    (
        "Count",
        &[223.0, 542.0],
        &[232.0, 381.0, 480.5, 532.5, 652.0, 690.5, 757.0, 795.5],
    ),
    (
        "Pen",
        &[183.0, 404.5, 437.5, 470.5, 503.5],
        &[110.0, 179.0, 220.5, 294.5, 299.5, 345.0, 350.0, 400.0],
    ),
    (
        "FreeformPen",
        &[183.0, 404.5, 437.5, 470.5, 503.5],
        &[110.0, 179.0, 220.5, 294.5, 299.5, 345.0, 350.0, 400.0],
    ),
    (
        "CurvaturePen",
        &[183.0, 404.5, 437.5, 470.5, 503.5],
        &[110.0, 179.0, 220.5, 294.5, 299.5, 345.0, 350.0, 400.0],
    ),
    ("AddAnchorPoint", &[], &[]),
    ("DeleteAnchorPoint", &[], &[]),
    ("ConvertPoint", &[], &[]),
    (
        "HorizontalType",
        &[146.0, 482.5, 600.5, 691.5, 730.5, 807.5, 851.5],
        &[
            152.0, 295.5, 310.0, 320.0, 461.0, 475.5, 516.5, 579.0, 593.5, 607.5, 632.5,
        ],
    ),
    (
        "VerticalType",
        &[146.0, 482.5, 600.5, 691.5, 730.5, 807.5, 851.5],
        &[
            152.0, 295.5, 310.0, 320.0, 461.0, 475.5, 516.5, 579.0, 593.5, 607.5, 632.5,
        ],
    ),
    (
        "HorizontalTypeMask",
        &[146.0, 482.5, 600.5, 691.5, 730.5, 807.5, 851.5],
        &[
            152.0, 295.5, 310.0, 320.0, 461.0, 475.5, 516.5, 579.0, 593.5, 607.5, 632.5,
        ],
    ),
    (
        "VerticalTypeMask",
        &[146.0, 482.5, 600.5, 691.5, 730.5, 807.5, 851.5],
        &[
            152.0, 295.5, 310.0, 320.0, 461.0, 475.5, 516.5, 579.0, 593.5, 607.5, 632.5,
        ],
    ),
    (
        "PathSelection",
        &[255.5, 512.5, 685.0, 728.0, 771.0, 814.0, 908.5],
        &[
            148.0, 246.5, 284.0, 313.0, 353.5, 382.5, 387.5, 438.0, 452.5, 459.0, 507.0, 531.0,
            575.5, 626.0, 670.5,
        ],
    ),
    (
        "DirectSelection",
        &[255.5, 512.5, 685.0, 728.0, 771.0, 814.0, 908.5],
        &[
            148.0, 246.5, 284.0, 313.0, 353.5, 382.5, 387.5, 438.0, 452.5, 459.0, 507.0, 531.0,
            575.5, 626.0, 670.5,
        ],
    ),
    (
        "Rectangle",
        &[183.0, 435.0, 603.0, 636.0, 669.0, 702.0, 735.0, 807.0],
        &[
            110.0, 179.0, 207.0, 236.0, 276.0, 305.0, 310.0, 360.5, 375.0, 381.5, 429.5, 453.5,
            498.0, 548.5, 593.0, 759.0, 803.5,
        ],
    ),
    (
        "Ellipse",
        &[183.0, 435.0, 603.0, 636.0, 669.0, 702.0],
        &[
            110.0, 179.0, 207.0, 236.0, 276.0, 305.0, 310.0, 360.5, 375.0, 381.5, 429.5, 453.5,
            498.0, 548.5, 593.0,
        ],
    ),
    (
        "Triangle",
        &[183.0, 435.0, 603.0, 636.0, 669.0, 702.0, 735.0, 807.0],
        &[
            110.0, 179.0, 207.0, 236.0, 276.0, 305.0, 310.0, 360.5, 375.0, 381.5, 429.5, 453.5,
            498.0, 548.5, 593.0, 759.0, 803.5,
        ],
    ),
    (
        "Polygon",
        &[
            183.0, 435.0, 603.0, 636.0, 669.0, 702.0, 735.0, 809.0, 881.5,
        ],
        &[
            110.0, 179.0, 207.0, 236.0, 276.0, 305.0, 310.0, 360.5, 375.0, 381.5, 429.5, 453.5,
            498.0, 548.5, 593.0, 761.0, 805.5, 833.0, 877.5,
        ],
    ),
    (
        "Line",
        &[183.0, 435.0, 603.0, 636.0, 669.0, 702.0],
        &[
            110.0, 179.0, 207.0, 236.0, 276.0, 305.0, 310.0, 360.5, 375.0, 381.5, 429.5, 453.5,
            498.0, 548.5, 593.0, 776.5, 821.0,
        ],
    ),
    (
        "CustomShape",
        &[183.0, 435.0, 603.0, 636.0, 669.0, 702.0],
        &[
            110.0, 179.0, 207.0, 236.0, 276.0, 305.0, 310.0, 360.5, 375.0, 381.5, 429.5, 453.5,
            498.0, 548.5, 593.0, 772.0, 801.0, 813.0,
        ],
    ),
    (
        "Hand",
        &[227.0, 454.0],
        &[238.5, 285.5, 295.5, 364.5, 374.0, 444.5],
    ),
    ("RotateView", &[], &[188.5, 233.0, 271.5, 345.0]),
    (
        "Zoom",
        &[169.0],
        &[110.0, 135.0, 524.5, 571.5, 581.5, 650.0, 660.0, 730.5],
    ),
    (
        "Artboard",
        &[742.5, 819.5, 862.5, 905.5],
        &[138.5, 378.0, 423.5, 498.0, 547.0, 621.5, 651.5, 733.5],
    ),
    (
        "PerspectiveCrop",
        &[323.0, 537.0, 685.0],
        &[
            126.5, 199.5, 245.0, 318.0, 385.5, 458.5, 462.5, 532.0, 544.5, 622.0, 632.0, 677.5,
        ],
    ),
    (
        "Slice",
        &[478.5],
        &[142.5, 256.0, 301.5, 361.0, 410.0, 469.5, 488.0, 600.5],
    ),
    (
        "SliceSelect",
        &[362.0, 491.0, 620.0, 663.0, 781.0],
        &[222.5, 283.5, 293.0, 352.5, 672.5, 771.5],
    ),
    (
        "Frame",
        &[250.0],
        &[
            110.0, 135.0, 290.5, 319.5, 324.5, 375.0, 389.5, 395.5, 489.5, 518.5, 563.0,
        ],
    ),
    (
        "Move",
        &[253.5, 404.5, 521.5, 638.5, 673.5],
        &[195.0, 248.5],
    ),
    ("SelectionBrush", &[], &[108.0, 157.0, 288.0, 331.5, 346.0]),
    (
        "Remove",
        &[294.5, 329.5, 444.0, 561.0, 923.5, 958.5],
        &[
            116.0, 141.0, 198.5, 234.0, 248.5, 262.5, 287.5, 339.0, 438.5, 453.5, 556.0,
        ],
    ),
    (
        "AdjustmentBrush",
        &[326.0, 391.0, 476.0, 511.0, 579.0, 720.5],
        &[
            173.0, 321.0, 359.0, 384.0, 629.5, 666.0, 680.5, 754.5, 791.0, 805.5,
        ],
    ),
];

/// The x (bar points) of separators and of frame edges in an options bar
/// cropped as `screenshot_options_bars` crops it (2x).
fn bar_marks(bar: &image::RgbaImage) -> (Vec<f32>, Vec<f32>) {
    let gray = |x: u32, y: u32| {
        let p = bar.get_pixel(x, y).0;
        (p[0] as i32 + p[1] as i32 + p[2] as i32) / 3
    };
    let width = bar.width().min(2200);
    let runs = |hit: &dyn Fn(u32) -> bool| {
        let mut out = Vec::new();
        let mut inside = false;
        for x in 208..width {
            let h = hit(x);
            if h && !inside {
                out.push(x as f32 / 2.0);
            }
            inside = h;
        }
        out
    };
    let seps = runs(&|x| (14..54).all(|y| (gray(x, y) - 62).abs() <= 3));
    let edges = runs(&|x| {
        (20..46)
            .filter(|&y| (91..=105).contains(&gray(x, y)))
            .count()
            >= 20
    });
    (seps, edges)
}

#[test]
fn options_bars_match_photoshops_layout() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let mut off = Vec::new();
    for &(name, seps, edges) in PS_BAR_MARKS {
        let tool = op_tools::TOOLBAR
            .iter()
            .flat_map(|s| s.iter())
            .find(|t| format!("{t:?}") == name)
            .copied()
            .unwrap();
        h.state_mut().state.select_tool(tool);
        h.run_steps(3);
        let image = h.render().expect("render frame");
        let bar = image::imageops::crop_imm(&image, 0, 56, image.width(), 70).to_image();
        let (got_seps, got_edges) = bar_marks(&bar);
        for (what, want, got) in [
            ("separators", seps, &got_seps),
            ("edges", edges, &got_edges),
        ] {
            let missing: Vec<_> = want
                .iter()
                .filter(|w| !got.iter().any(|g| (*g - **w).abs() <= 1.0))
                .collect();
            let extra: Vec<_> = got
                .iter()
                .filter(|g| !want.iter().any(|w| (**g - *w).abs() <= 1.0))
                .collect();
            if !missing.is_empty() || !extra.is_empty() {
                off.push(format!(
                    "{name} {what}: missing {missing:?}, extra {extra:?}"
                ));
            }
        }
    }
    assert!(off.is_empty(), "{}", off.join("\n"));
}

#[test]
fn selection_options_bars_edit_their_settings() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut()
        .state
        .select_tool(op_tools::Tool::RectangularMarquee);
    h.run_steps(2);
    // Feather: the field at 276–330 (bar points), the bar's middle at
    // window y 45.25
    click(&mut h, at_pt(300.0, 45.25));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("5".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().state.marquee.feather, 5.0);
    // The second mode button: Add to selection
    click(&mut h, at_pt(149.0, 45.25));
    assert_eq!(
        h.state().state.marquee.mode,
        crate::state::SelectionMode::Add
    );
    // Magic Wand: Tolerance at 476.5–524.5
    h.state_mut().state.select_tool(op_tools::Tool::MagicWand);
    h.run_steps(2);
    click(&mut h, at_pt(500.0, 45.25));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("60".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().state.wand.region.tolerance, 60);
    // Contiguous (box at 606.5) toggles off
    click(&mut h, at_pt(611.0, 45.25));
    assert!(!h.state().state.wand.region.contiguous);
    // Magnetic Lasso's Frequency keeps what's typed
    h.state_mut()
        .state
        .select_tool(op_tools::Tool::MagneticLasso);
    h.run_steps(2);
    click(&mut h, at_pt(680.0, 45.25));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("80".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(
        h.state()
            .state
            .tool_settings
            .get("lasso.frequency")
            .map(String::as_str),
        Some("80")
    );
}

#[test]
fn brush_options_bars_edit_their_settings() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.select_tool(op_tools::Tool::Brush);
    h.run_steps(2);
    // Opacity: the field at 422.5–460
    click(&mut h, at_pt(440.0, 45.25));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("50".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().state.brush.opacity, 0.5);
    // The airbrush toggle (center 619.25) turns on
    click(&mut h, at_pt(619.25, 45.25));
    assert_eq!(
        h.state()
            .state
            .tool_settings
            .get("brush.airbrush")
            .map(String::as_str),
        Some("1")
    );
    // Clone Stamp's Aligned (box at 694.5) is the real setting
    h.state_mut().state.select_tool(op_tools::Tool::CloneStamp);
    h.run_steps(2);
    assert!(h.state().state.retouch.clone_aligned);
    click(&mut h, at_pt(699.0, 45.25));
    assert!(!h.state().state.retouch.clone_aligned);
}

#[test]
fn fill_and_sample_options_bars_edit_their_settings() {
    use op_core::gradient::GradientKind;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // Gradient: the second type button (center 388) is Radial
    h.state_mut().state.select_tool(op_tools::Tool::Gradient);
    h.run_steps(2);
    click(&mut h, at_pt(388.0, 45.25));
    assert_eq!(h.state().state.gradient.kind, GradientKind::Radial);
    // Reverse (box at 500)
    click(&mut h, at_pt(505.0, 45.25));
    assert!(h.state().state.gradient.reverse);
    // Paint Bucket: Tolerance at 530–565
    h.state_mut().state.select_tool(op_tools::Tool::PaintBucket);
    h.run_steps(2);
    click(&mut h, at_pt(548.0, 45.25));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("10".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().state.bucket.tolerance, 10);
    // Spot Healing: Create Texture (the second segment, 425.5–513.5)
    h.state_mut()
        .state
        .select_tool(op_tools::Tool::SpotHealingBrush);
    h.run_steps(2);
    click(&mut h, at_pt(470.0, 45.25));
    assert_eq!(
        h.state()
            .state
            .tool_settings
            .get("spotheal.type")
            .map(String::as_str),
        Some("1")
    );
}

/// Types `text` into the options bar field at bar point `x`.
fn type_in_bar(h: &mut Harness<'_, OpenPhotoApp>, x: f32, text: &str) {
    click(h, at_pt(x, 45.25));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text(text.into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
}

fn setting(h: &Harness<'_, OpenPhotoApp>, key: &str) -> Option<String> {
    h.state().state.tool_settings.get(key).cloned()
}

#[test]
fn type_shape_and_view_options_bars_edit_their_settings() {
    use op_tools::Tool;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let select = |h: &mut Harness<'_, OpenPhotoApp>, tool| {
        h.state_mut().state.select_tool(tool);
        h.run_steps(2);
    };
    // Type: the style (320–462) and size (516.5–580) fields
    select(&mut h, Tool::HorizontalType);
    type_in_bar(&mut h, 380.0, "Semibold");
    assert!(h.state().state.type_options.semibold);
    type_in_bar(&mut h, 540.0, "24 pt");
    assert_eq!(h.state().state.type_options.size_pt, 24.0);
    // Center text (646.5)
    click(&mut h, at_pt(646.5, 45.5));
    assert_eq!(setting(&h, "type.align").as_deref(), Some("1"));
    // Polygon sides (761–806.5) and the Line's weight (776.5–822)
    select(&mut h, Tool::Polygon);
    type_in_bar(&mut h, 780.0, "7");
    assert_eq!(h.state().state.shape.sides, 7);
    select(&mut h, Tool::Line);
    type_in_bar(&mut h, 795.0, "3 px");
    assert_eq!(h.state().state.shape.weight, 3.0);
    // Pen: Auto Add/Delete starts on
    select(&mut h, Tool::Pen);
    click(&mut h, at_pt(541.5, 45.25));
    assert_eq!(setting(&h, "pen.auto_add").as_deref(), Some("0"));
    // Hand: 100%, Fit Screen and Fill Screen
    select(&mut h, Tool::Hand);
    click(&mut h, at_pt(263.0, 45.5));
    assert_eq!(active(&h).view.zoom, 1.0);
    click(&mut h, at_pt(330.0, 45.5));
    let fit = active(&h).view.zoom;
    click(&mut h, at_pt(410.0, 45.5));
    let fill = active(&h).view.zoom;
    assert!(fit != 1.0 && fill > fit, "fit {fit}, fill {fill}");
    // Zoom: with Zoom Out pressed (152) a click zooms out
    select(&mut h, Tool::Zoom);
    click(&mut h, at_pt(152.0, 45.5));
    assert_eq!(setting(&h, "zoom.out").as_deref(), Some("1"));
    let before = active(&h).view.zoom;
    let p = doc_point(&h, 367.0, 405.0);
    click(&mut h, p);
    assert!(active(&h).view.zoom < before);
    // Rotate View: the angle wraps into ±180°; Reset View clears it
    select(&mut h, Tool::RotateView);
    type_in_bar(&mut h, 210.0, "200");
    assert_eq!(setting(&h, "rotate.angle").as_deref(), Some("-160°"));
    click(&mut h, at_pt(309.0, 45.5));
    assert_eq!(setting(&h, "rotate.angle").as_deref(), Some("0°"));
    // Perspective Crop: Front Image takes the document's size
    select(&mut h, Tool::PerspectiveCrop);
    click(&mut h, at_pt(584.0, 45.5));
    assert_eq!(setting(&h, "pcrop.w").as_deref(), Some("734 px"));
    assert_eq!(setting(&h, "pcrop.h").as_deref(), Some("811 px"));
    click(&mut h, at_pt(655.0, 45.5));
    assert_eq!(setting(&h, "pcrop.w").as_deref(), Some(""));
    // Slice: Width can be typed only once a fixed style is chosen
    select(&mut h, Tool::Slice);
    type_in_bar(&mut h, 330.0, "100");
    assert_eq!(setting(&h, "slice.w"), None);
    *h.state_mut().state.setting("slice.style", "0") = "2".into();
    h.run_steps(2);
    type_in_bar(&mut h, 330.0, "100");
    assert_eq!(setting(&h, "slice.w").as_deref(), Some("100"));
}

/// The open dialog's top-left corner in Photoshop points: the extent of
/// its title bar's fill.
fn dialog_origin(h: &mut Harness<'_, OpenPhotoApp>) -> (f32, f32) {
    let image = h.render().expect("render frame");
    let (mut x0, mut y0) = (u32::MAX, u32::MAX);
    for (x, y, p) in image.enumerate_pixels() {
        if p.0[..3] == [0xd0, 0xd2, 0xd4] {
            x0 = x0.min(x);
            y0 = y0.min(y);
        }
    }
    assert!(x0 != u32::MAX, "no dialog");
    (x0 as f32 / 2.0, y0 as f32 / 2.0)
}

/// A 64 × 72 transparent document like the probe opened in Photoshop
/// (one "Layer 0", no background).
fn probe_document(h: &mut Harness<'_, OpenPhotoApp>) {
    let app = &mut h.state_mut().state;
    crate::actions::close_all(app);
    let doc = op_core::Document::from_rgba8("probe.png", 64, 72, &vec![0; 64 * 72 * 4]);
    app.add_document(doc, "Open");
    h.run_steps(6);
}

#[test]
#[ignore]
fn screenshot_canvas_size_dialog() {
    let mut h = harness(Vec::new());
    probe_document(&mut h);
    h.state_mut().state.background = Color::from_rgba8([0x52, 0x65, 0x6e, 255]);
    run_command(&mut h, crate::commands::Command::CanvasSize);
    h.run_steps(3);
    shot_dialog(&mut h, "canvas_size", 458.0, 372.0);
}

/// Saves the open dialog, `w` × `h` pt, as `target/ui-shots/<name>.png`
/// (2x, like the Photoshop captures it is compared with).
fn shot_dialog(h: &mut Harness<'_, OpenPhotoApp>, name: &str, w: f32, ht: f32) {
    let (x, y) = dialog_origin(h);
    let image = h.render().expect("render frame");
    let crop = image::imageops::crop_imm(
        &image,
        (x * 2.0) as u32,
        (y * 2.0) as u32,
        (w * 2.0) as u32,
        (ht * 2.0) as u32,
    );
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-shots");
    std::fs::create_dir_all(&dir).unwrap();
    crop.to_image()
        .save(dir.join(format!("{name}.png")))
        .unwrap();
}

#[test]
fn canvas_size_dialog_resizes_around_the_anchor() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    run_command(&mut h, crate::commands::Command::CanvasSize);
    h.run_steps(3);
    let (x, y) = dialog_origin(&mut h);
    // The width field opens focused with its text selected
    h.event(egui::Event::Text("800".into()));
    h.run_steps(2);
    // Anchor at the top left (the grid's first cell at 64, 246)
    click(&mut h, at_pt(x + 75.5, y + 257.5));
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let doc = &active(&h).doc;
    assert_eq!((doc.width, doc.height), (800, 811));
    assert_eq!(last_history(&h), "Canvas Size");
    // The image stays at the left; the background extends on the right
    // in the background color (white)
    assert_eq!(layer_pixel(&h, 0, 0, 0), [0x14, 0x14, 0x14, 255]);
    assert_eq!(layer_pixel(&h, 0, 799, 0), [255, 255, 255, 255]);

    // Relative: 10 more pixels of height, centered
    run_command(&mut h, crate::commands::Command::CanvasSize);
    h.run_steps(3);
    let (x, y) = dialog_origin(&mut h);
    click(&mut h, at_pt(x + 70.0, y + 222.0));
    click(&mut h, at_pt(x + 100.0, y + 186.0));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("10".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let doc = &active(&h).doc;
    assert_eq!((doc.width, doc.height), (800, 821));

    // Escape cancels
    run_command(&mut h, crate::commands::Command::CanvasSize);
    h.run_steps(3);
    h.event(egui::Event::Text("5".into()));
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert_eq!(active(&h).doc.width, 800);
}

#[test]
#[ignore]
fn screenshot_new_guide_dialog() {
    let mut h = harness(Vec::new());
    probe_document(&mut h);
    run_command(&mut h, crate::commands::Command::NewGuide);
    h.run_steps(3);
    shot_dialog(&mut h, "new_guide", 390.0, 188.0);
}

#[test]
fn new_guide_dialog_adds_a_guide() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    run_command(&mut h, crate::commands::Command::NewGuide);
    h.run_steps(3);
    let (x, y) = dialog_origin(&mut h);
    // Vertical, then 100 in Position
    click(&mut h, at_pt(x + 108.0, y + 84.0));
    click(&mut h, at_pt(x + 110.0, y + 120.0));
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("100 px".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let guides = &active(&h).doc.guides;
    assert_eq!(guides.len(), 1);
    assert!(guides[0].vertical);
    assert_eq!(guides[0].position, 100.0);
}

#[test]
#[ignore]
fn screenshot_feather_dialog() {
    let mut h = harness(Vec::new());
    probe_document(&mut h);
    run_command(&mut h, crate::commands::Command::SelectAll);
    run_command(&mut h, crate::commands::Command::ModifyFeather);
    h.event(egui::Event::Text("5".into()));
    h.run_steps(3);
    shot_dialog(&mut h, "feather", 295.0, 128.0);
}

/// Scrolls whatever is under `pos` by `dy` points (negative: down).
fn scroll_at(h: &mut Harness<'_, OpenPhotoApp>, pos: Pos2, dy: f32) {
    h.event(egui::Event::PointerMoved(pos));
    h.run_steps(1);
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, dy),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(4);
}

#[test]
#[ignore]
fn screenshot_properties_sections() {
    let mut h = harness(Vec::new());
    probe_document(&mut h);
    shot(&mut h, "properties_top");
    scroll_at(&mut h, at_pt(1150.0, 450.0), -120.0);
    shot(&mut h, "properties_mid");
    scroll_at(&mut h, at_pt(1150.0, 450.0), -2000.0);
    shot(&mut h, "properties_end");
}

#[test]
fn properties_sections_toggle_views_and_run_quick_actions() {
    let mut h = harness(Vec::new());
    probe_document(&mut h);
    // The panel's content starts at (1028, 253) in the default workspace;
    // its sections' rules are at 33 + the heights above them
    let p = |x: f32, y: f32| at_pt(1028.0 + x, 253.0 + y);
    // Collapse Canvas: Rulers & Grids moves up to the rule at 65
    click(&mut h, p(40.0, 33.0 + 16.0));
    click(&mut h, p(23.0, 65.0 + 44.5));
    assert!(h.state().state.view.rulers);
    click(&mut h, p(56.0, 65.0 + 44.5));
    assert!(h.state().state.view.grid);
    // Collapse Rulers & Grids: Guides' rule at 97; Lock Guides
    click(&mut h, p(40.0, 65.0 + 16.0));
    click(&mut h, p(56.0, 97.0 + 45.0));
    assert!(h.state().state.view.lock_guides);
    // Collapse Guides: Quick Actions' rule at 129. Crop picks the tool,
    // Trim opens its dialog
    click(&mut h, p(40.0, 97.0 + 16.0));
    click(&mut h, p(150.0, 129.0 + 44.5));
    assert_eq!(h.state().state.tool, op_tools::Tool::Crop);
    click(&mut h, p(55.0, 129.0 + 74.5));
    assert!(h.state().state.trim_dialog.is_some());
}

#[test]
#[ignore]
fn screenshot_layer_drag_and_rename() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::J);
    h.run_steps(3);
    // Hold "Layer 1" (the middle row) over the top row's upper half
    let (from, to) = (at_pt(1130.0, 695.0), at_pt(1130.0, 640.0));
    h.hover_at(from);
    h.event(egui::Event::PointerButton {
        pos: from,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for i in 1..=4 {
        h.event(egui::Event::PointerMoved(
            from + (to - from) * (i as f32 / 4.0),
        ));
        h.step();
    }
    shot(&mut h, "layer_drag");
    h.event(egui::Event::PointerMoved(from));
    h.step();
    h.event(egui::Event::PointerButton {
        pos: from,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(3);
    double_click(&mut h, at_pt(1130.0, 695.0));
    h.run_steps(2);
    shot(&mut h, "layer_rename");
}

#[test]
fn selection_brush_remove_and_adjustment_brush_bars_edit_their_settings() {
    use op_tools::Tool;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let setting =
        |h: &Harness<'_, OpenPhotoApp>, k: &str| h.state().state.tool_settings.get(k).cloned();
    // Selection Brush: Subtract (160–233)
    h.state_mut().state.select_tool(Tool::SelectionBrush);
    h.run_steps(2);
    click(&mut h, at_pt(200.0, 45.5));
    assert_eq!(setting(&h, "selbrush.mode").as_deref(), Some("1"));
    // Remove: Create new layer (box at 818.5); pressure starts on
    h.state_mut().state.select_tool(Tool::Remove);
    h.run_steps(2);
    click(&mut h, at_pt(823.5, 45.25));
    assert_eq!(setting(&h, "remove.new_layer").as_deref(), Some("1"));
    click(&mut h, at_pt(275.0, 45.5));
    assert_eq!(setting(&h, "remove.pressure").as_deref(), Some("0"));
    // Adjustment Brush: Overlay (517)
    h.state_mut().state.select_tool(Tool::AdjustmentBrush);
    h.run_steps(2);
    click(&mut h, at_pt(522.0, 45.25));
    assert_eq!(setting(&h, "adjbrush.overlay").as_deref(), Some("1"));
}

#[test]
#[ignore]
fn screenshot_new_document_dialog() {
    let mut h = harness(Vec::new());
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::N);
    h.run_steps(3);
    shot(&mut h, "new_document_recent");
    // The Photo tab (its text from x 172 in the dialog)
    let (x, y) = new_document_origin(&mut h);
    click(&mut h, at_pt(x + 188.0, y + 51.0));
    shot(&mut h, "new_document_photo");
}

/// The New Document dialog's top-left corner (its title bar's color).
fn new_document_origin(h: &mut Harness<'_, OpenPhotoApp>) -> (f32, f32) {
    let image = h.render().expect("render frame");
    let (mut x0, mut y0) = (u32::MAX, u32::MAX);
    for (x, y, p) in image.enumerate_pixels() {
        if p.0[..3] == [0xd3, 0xd4, 0xd5] {
            x0 = x0.min(x);
            y0 = y0.min(y);
        }
    }
    (x0 as f32 / 2.0, y0 as f32 / 2.0)
}

#[test]
fn new_document_dialog_presets_recent_and_saved() {
    let mut h = harness(Vec::new());
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::N);
    h.run_steps(3);
    let (x, y) = new_document_origin(&mut h);
    let at = |dx: f32, dy: f32| at_pt(x + dx, y + dy);
    // The Photo tab picks its first preset, Default Photoshop Size
    click(&mut h, at(188.0, 51.0));
    // Landscape, 6 x 4 (the third card, from x 387.8)
    click(&mut h, at(26.0 + 2.0 * 180.9 + 84.0, 124.0 + 84.0));
    // Save it as a preset with the save icon, then create
    click(&mut h, at(1048.5, 131.0));
    assert_eq!(h.state().state.new_document_saved.len(), 1);
    click(&mut h, at(1024.0, 674.0));
    let d = &active(&h).doc;
    assert_eq!((d.width, d.height, d.resolution), (1800, 1200, 300.0));
    // Reopened, Recent starts with it and the Saved tab lists the preset
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::N);
    h.run_steps(3);
    let recent = &h.state().state.new_document_recent;
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].size_label(), "6 x 4 in @ 300 ppi");
    click(&mut h, at(128.0, 51.0));
    shot(&mut h, "new_document_saved");
    // Close leaves without a document
    let docs = h.state().state.docs.len();
    click(&mut h, at(936.0, 674.0));
    assert!(h.state().state.new_document_dialog.is_none());
    assert_eq!(h.state().state.docs.len(), docs);
}

#[test]
fn brush_modes_and_eraser_block() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    h.state_mut().state.select_tool(op_tools::Tool::Brush);
    // Multiply (the fifth entry of the Mode menu: Normal, Dissolve, Behind,
    // Clear, Darken, Multiply)
    *h.state_mut().state.setting("brush.mode", "0") = "5".into();
    h.run_steps(2);
    let (a, b) = (doc_point(&h, 100.0, 300.0), doc_point(&h, 500.0, 300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    // Red multiplied into the dark gray keeps only its red channel
    assert_eq!(composite_pixel(&mut h, 300, 300), [0x14, 0, 0, 255]);

    // The Eraser's Block: a hard square of 16 screen pixels, painting the
    // background color (white) on the background layer
    h.state_mut().state.background = Color::WHITE;
    h.state_mut().state.select_tool(op_tools::Tool::Eraser);
    *h.state_mut().state.setting("eraser.mode", "0") = "2".into();
    h.run_steps(2);
    let zoom = active(&h).view.zoom;
    let p = doc_point(&h, 300.0, 600.0);
    click(&mut h, p);
    assert_eq!(composite_pixel(&mut h, 300, 600), [255, 255, 255, 255]);
    // The square's corner is erased too; outside it isn't
    let half = 8.0 / zoom;
    let corner = (300.0 + half - 1.0) as u32;
    assert_eq!(composite_pixel(&mut h, corner, corner + 300)[0], 255);
    let outside = (300.0 + half + 2.0) as u32;
    assert_eq!(
        composite_pixel(&mut h, outside, 600),
        [0x14, 0x14, 0x14, 255]
    );
}

#[test]
fn holding_a_toolbar_button_opens_its_flyout() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let before = h.state().state.tool;
    let p = at_pt(19.5, 105.5);
    h.hover_at(p);
    h.event(egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    // A short press doesn't open it
    h.run_steps(3);
    assert!(flyout_frame(&h.render().unwrap()).0.is_empty());
    // Held past 0.4 s it does, and stays open once released
    let start = h.ctx.input(|i| i.time);
    while h.ctx.input(|i| i.time) - start < 0.5 {
        h.step();
    }
    assert!(
        !flyout_frame(&h.render().unwrap()).0.is_empty(),
        "open while held"
    );
    h.event(egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    assert!(!flyout_frame(&h.render().unwrap()).0.is_empty());
    // The hold didn't pick a tool; a row does, and closes the flyout
    assert_eq!(h.state().state.tool, before);
    click(&mut h, at_pt(80.0, 92.0 + 1.0 + 19.0 + 9.5));
    assert_eq!(h.state().state.tool, op_tools::Tool::Artboard);
    assert!(flyout_frame(&h.render().unwrap()).0.is_empty());
}

#[test]
fn lasso_with_alt_draws_straight_edges() {
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    h.state_mut().state.select_tool(op_tools::Tool::Lasso);
    h.run_steps(2);
    let press = |h: &mut Harness<'_, OpenPhotoApp>, p: Pos2, down: bool, m: Modifiers| {
        h.event(egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed: down,
            modifiers: m,
        });
        h.step();
    };
    // Drag freehand from (100, 100) to (300, 100)
    let (a, b) = (doc_point(&h, 100.0, 100.0), doc_point(&h, 300.0, 100.0));
    h.hover_at(a);
    press(&mut h, a, true, Modifiers::NONE);
    for i in 1..=4 {
        h.event(egui::Event::PointerMoved(a + (b - a) * (i as f32 / 4.0)));
        h.step();
    }
    // Let go with Alt held: no selection yet
    h.event(egui::Event::ModifiersChanged(Modifiers::ALT));
    press(&mut h, b, false, Modifiers::ALT);
    h.run_steps(2);
    assert!(active(&h).doc.selection().is_none());
    assert!(active(&h).lasso.as_ref().is_some_and(|l| l.held));
    // Alt-click a corner at (300, 400), then release Alt to close
    let c = doc_point(&h, 300.0, 400.0);
    h.event(egui::Event::PointerMoved(c));
    h.step();
    press(&mut h, c, true, Modifiers::ALT);
    press(&mut h, c, false, Modifiers::ALT);
    h.event(egui::Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(3);
    let bounds = active(&h).doc.selection().and_then(|s| s.bounds());
    let (x0, y0, x1, y1) = bounds.expect("a selection");
    assert!(
        x0 <= 101 && y0 <= 101 && x1 >= 299 && y1 >= 399,
        "{:?}",
        bounds
    );
    assert_eq!(last_history(&h), "Lasso");
}

#[test]
fn sampling_scopes_and_the_sampling_ring() {
    use op_tools::Tool;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    // A red square on "Layer 1", an empty "Layer 2" on top (active)
    {
        let app = &mut h.state_mut().state;
        let state = app.active().unwrap();
        let doc = &mut state.doc;
        let mut red = op_core::TiledImage::new(doc.width, doc.height);
        for y in 50..150 {
            for x in 50..150 {
                red.set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        let id = doc.new_layer_id();
        doc.layers.push(op_core::Layer::raster(id, "Layer 1", red));
        let top = doc.new_layer_id();
        let (w, hgt) = (doc.width, doc.height);
        doc.layers.push(op_core::Layer::raster(
            top,
            "Layer 2",
            op_core::TiledImage::new(w, hgt),
        ));
        doc.select_layer(top);
        doc.mark_dirty();
    }
    h.run_steps(2);
    let red = Color::from_rgba8([255, 0, 0, 255]);
    // Eyedropper: the empty current layer gives nothing; Current & Below
    // finds the red under it
    h.state_mut().state.select_tool(Tool::Eyedropper);
    h.state_mut().state.foreground = Color::WHITE;
    *h.state_mut().state.setting("eyedropper.ring", "1") = "0".into();
    h.state_mut().state.eyedropper.sample = op_core::SampleScope::Current;
    h.run_steps(1);
    let p = doc_point(&h, 100.0, 100.0);
    click(&mut h, p);
    assert_eq!(h.state().state.foreground, Color::WHITE);
    h.state_mut().state.eyedropper.sample = op_core::SampleScope::CurrentAndBelow;
    click(&mut h, p);
    assert_eq!(h.state().state.foreground, red);

    // The sampling ring while held: the picked color above the pointer,
    // the one it replaces below
    h.state_mut().state.foreground = Color::from_rgba8([0, 0, 255, 255]);
    *h.state_mut().state.setting("eyedropper.ring", "1") = "1".into();
    h.hover_at(p);
    h.event(egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    let image = h.render().unwrap();
    let k = 2.0 * UI_SCALE;
    let px = |q: Pos2| image.get_pixel((q.x * k) as u32, (q.y * k) as u32).0;
    let ring = crate::theme::pt(48.0);
    assert_eq!(px(p - egui::vec2(0.0, ring))[..3], [255, 0, 0]);
    assert_eq!(px(p + egui::vec2(0.0, ring))[..3], [0, 0, 255]);
    h.event(egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);

    // Clone Stamp sampling All Layers paints the red onto the empty layer
    h.state_mut().state.select_tool(Tool::CloneStamp);
    *h.state_mut().state.setting("clone.sample", "0") = "2".into();
    h.run_steps(1);
    alt_click(&mut h, p);
    let q = doc_point(&h, 400.0, 400.0);
    click(&mut h, q);
    let top = active(&h).doc.active_layer.unwrap();
    let pixel = active(&h)
        .doc
        .layer(top)
        .unwrap()
        .image()
        .unwrap()
        .pixel(400, 400);
    // (the soft brush's center is just short of opaque)
    assert_eq!(pixel[..3], [255, 0, 0]);
    assert!(pixel[3] >= 250, "{pixel:?}");
}

#[test]
fn smudge_pattern_background_eraser_and_color_replacement() {
    use op_tools::Tool;
    let mut h = harness(Vec::new());
    reference_document(&mut h);
    let stroke = |h: &mut Harness<'_, OpenPhotoApp>, tool: Tool, from: (f32, f32), to: (f32, f32)| {
        h.state_mut().state.select_tool(tool);
        h.run_steps(1);
        let (a, b) = (doc_point(h, from.0, from.1), doc_point(h, to.0, to.1));
        drag(h, a, b, Modifiers::NONE);
    };
    // Pattern Stamp paints the default green pattern
    stroke(&mut h, Tool::PatternStamp, (100.0, 100.0), (200.0, 100.0));
    let p = composite_pixel(&mut h, 150, 100);
    assert!(p[1] > p[0] && p[1] > p[2], "{p:?}");
    assert_eq!(last_history(&h), "Pattern Stamp");

    // Color Replacement gives the dark gray the foreground's hue
    h.state_mut().state.foreground = Color::from_rgba8([255, 0, 0, 255]);
    stroke(&mut h, Tool::ColorReplacement, (100.0, 400.0), (200.0, 400.0));
    let p = composite_pixel(&mut h, 150, 400);
    assert!(p[0] > p[1] && p[0] > p[2], "{p:?}");

    // Smudge pulls the green into the gray
    stroke(&mut h, Tool::Smudge, (150.0, 100.0), (150.0, 140.0));
    let p = composite_pixel(&mut h, 150, 125);
    assert!(p[1] > 0x14, "{p:?}");

    // The Background Eraser makes the background a layer and erases the gray
    stroke(&mut h, Tool::BackgroundEraser, (400.0, 600.0), (600.0, 600.0));
    let doc = &active(&h).doc;
    assert!(!doc.layers[0].is_background);
    assert_eq!(doc.layers[0].name, "Layer 0");
    assert_eq!(doc.layers[0].image().unwrap().pixel(500, 600)[3], 0);
    assert_eq!(doc.layers[0].image().unwrap().pixel(500, 700)[3], 255);
    assert_eq!(last_history(&h), "Background Eraser");
}
