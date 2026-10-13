//! CPU-only egui canvas interactions. Paint callbacks are constructed but never rendered.
use crate::state::{AppState, DocState};
use egui::{Event, Key, Modifiers, PointerButton, Pos2};
use op_tools::Tool;

struct CanvasHarness {
    ctx: egui::Context,
    app: AppState,
    frame: u64,
}
impl CanvasHarness {
    fn new(tool: Tool) -> Self {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::apply_style(&ctx);
        let mut app = AppState::default();
        app.add_document(
            op_core::Document::new_with_background("offline", 256, 256, op_core::Color::WHITE),
            "Open",
        );
        app.tool = tool;
        app.view.snap = false;
        app.view.smart_guides = false;
        let mut h = Self { ctx, app, frame: 0 };
        for _ in 0..4 {
            h.step(Vec::new(), Modifiers::NONE);
        }
        h
    }
    fn state(&self) -> &DocState {
        &self.app.docs[&self.app.active_doc.unwrap()]
    }
    fn point(&self, x: f32, y: f32) -> Pos2 {
        crate::document_view::to_screen(self.state(), egui::pos2(x, y), self.ctx.pixels_per_point())
    }
    fn step(&mut self, mut events: Vec<Event>, modifiers: Modifiers) {
        self.frame += 1;
        events.insert(0, Event::ModifiersChanged(modifiers));
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1024.0, 768.0),
            )),
            time: Some(self.frame as f64 / 60.0),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut output = self.ctx.run_ui(raw, |ui| {
            let ctx = ui.ctx().clone();
            crate::actions::handle_tool_keys(&ctx, app);
            let id = app.active_doc.unwrap();
            crate::document_view::show(ui, app, id);
        });
        output.textures_delta.clear();
    }
    fn button(&mut self, p: Pos2, down: bool, mods: Modifiers) {
        self.step(
            vec![
                Event::PointerMoved(p),
                Event::PointerButton {
                    pos: p,
                    button: PointerButton::Primary,
                    pressed: down,
                    modifiers: mods,
                },
            ],
            mods,
        );
    }
    fn toolbar_step(&mut self, events: Vec<Event>) -> Pos2 {
        self.frame += 1;
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1024.0, 1400.0),
            )),
            time: Some(self.frame as f64 / 60.0),
            events,
            ..Default::default()
        };
        let mut origin = Pos2::ZERO;
        let mut output = self.ctx.run_ui(raw, |ui| {
            origin = ui.max_rect().min;
            crate::toolbar::show(ui, &mut self.app);
        });
        output.textures_delta.clear();
        origin
    }
    fn toolbar_click(&mut self, p: Pos2) {
        for pressed in [true, false] {
            self.toolbar_step(vec![
                Event::PointerMoved(p),
                Event::PointerButton {
                    pos: p,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ]);
        }
    }
    fn key(&mut self, key: Key) {
        self.step(
            vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            Modifiers::NONE,
        );
    }
    fn drag(&mut self, a: (f32, f32), b: (f32, f32), mods: Modifiers) {
        let a = self.point(a.0, a.1);
        let b = self.point(b.0, b.1);
        self.button(a, true, mods);
        self.step(vec![Event::PointerMoved(b)], mods);
        self.button(b, false, mods);
        self.step(Vec::new(), Modifiers::NONE);
    }
}

pub(super) fn single_row() {
    let mut h = CanvasHarness::new(Tool::SingleRowMarquee);
    let pixels = h.state().doc.composite_rgba8();
    h.drag((20.5, 20.5), (90.5, 50.5), Modifiers::NONE);
    assert_eq!(
        h.state().doc.selection().unwrap().bounds(),
        Some((0, 50, 256, 51))
    );
    h.drag((20.5, 90.5), (100.5, 100.5), Modifiers::SHIFT);
    assert_eq!(h.state().doc.selection().unwrap().get(250, 50), 255);
    assert_eq!(h.state().doc.selection().unwrap().get(250, 100), 255);
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let a = h.point(20.5, 120.5);
    h.button(a, true, Modifiers::NONE);
    let b = h.point(50.5, 130.5);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    assert!(h.state().marquee_drag.is_some());
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().history.states().len(), count);
    h.button(b, false, Modifiers::NONE);
    h.app.marquee.feather = 2.0;
    let p = h.point(20.5, 160.5);
    h.button(p, true, Modifiers::NONE);
    h.button(p, false, Modifiers::NONE);
    let selection = h.state().doc.selection().unwrap();
    assert!(selection.get(250, 160) > 0 && selection.get(250, 160) < 255);
    assert!(selection.get(250, 159) > 0);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.selection(), before.as_ref());
}

#[test]
fn single_column_drag_feather_cancel_nudge_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::SingleColumnMarquee);
    let pixels = h.state().doc.composite_rgba8();
    h.drag((20.5, 20.5), (50.5, 90.5), Modifiers::NONE);
    assert_eq!(
        h.state().doc.selection().unwrap().bounds(),
        Some((50, 0, 51, 256))
    );
    h.step(
        vec![Event::Key {
            key: Key::ArrowRight,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!(
        h.state().doc.selection().unwrap().bounds(),
        Some((51, 0, 52, 256))
    );
    h.drag((90.5, 20.5), (100.5, 90.5), Modifiers::SHIFT);
    assert_eq!(h.state().doc.selection().unwrap().get(51, 250), 255);
    assert_eq!(h.state().doc.selection().unwrap().get(100, 250), 255);
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let a = h.point(120.5, 20.5);
    h.button(a, true, Modifiers::NONE);
    let b = h.point(130.5, 80.5);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(b, false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().history.states().len(), count);
    h.app.marquee.feather = 2.0;
    let p = h.point(160.5, 20.5);
    h.button(p, true, Modifiers::NONE);
    h.button(p, false, Modifiers::NONE);
    let selection = h.state().doc.selection().unwrap();
    assert!(selection.get(160, 250) > 0 && selection.get(160, 250) < 255);
    assert!(selection.get(159, 250) > 0);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.selection(), before.as_ref());
}

#[test]
fn selection_brush_cancel_zero_opacity_subtract_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::SelectionBrush);
    h.app.selection_brush.size = 20.0;
    let pixels = h.state().doc.composite_rgba8();
    let original = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let a = h.point(50.0, 50.0);
    h.button(a, true, Modifiers::NONE);
    let b = h.point(80.0, 50.0);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    assert!(h.state().doc.selection().unwrap().get(60, 50) > 0);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(b, false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection(), original.as_ref());
    assert_eq!(h.state().history.states().len(), count);
    h.app.selection_brush.opacity = 0.0;
    h.drag((50.0, 50.0), (80.0, 50.0), Modifiers::NONE);
    assert_eq!(h.state().history.states().len(), count);
    h.app.selection_brush.opacity = 1.0;
    h.drag((50.0, 50.0), (80.0, 50.0), Modifiers::NONE);
    let before = h.state().doc.selection().cloned();
    assert_eq!(h.state().doc.selection().unwrap().get(60, 50), 255);
    h.drag(
        (60.0, 45.0),
        (60.0, 55.0),
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(h.state().doc.selection().unwrap().get(60, 50), 0);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn lasso_cancel_press_modifier_close_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Lasso);
    h.drag((30.0, 30.0), (60.0, 60.0), Modifiers::NONE);
    assert!(h.state().doc.selection().is_none());
    let a = h.point(30.0, 30.0);
    h.button(a, true, Modifiers::NONE);
    for point in [(80.0, 30.0), (80.0, 80.0), (30.0, 80.0)] {
        let p = h.point(point.0, point.1);
        h.step(vec![Event::PointerMoved(p)], Modifiers::NONE);
    }
    h.button(h.point(30.0, 80.0), false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection().unwrap().get(50, 50), 255);
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let pixels = h.state().doc.composite_rgba8();
    let a = h.point(100.0, 100.0);
    h.button(a, true, Modifiers::SHIFT);
    let p = h.point(150.0, 100.0);
    h.step(vec![Event::PointerMoved(p)], Modifiers::NONE);
    assert_eq!(
        h.state().lasso.as_ref().unwrap().op,
        op_core::SelectionOp::Add
    );
    let p = h.point(150.0, 150.0);
    h.step(vec![Event::PointerMoved(p)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(p, false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().history.states().len(), count);
    let a = h.point(100.0, 100.0);
    h.button(a, true, Modifiers::SHIFT);
    for point in [(150.0, 100.0), (150.0, 150.0), (100.0, 150.0)] {
        let p = h.point(point.0, point.1);
        h.step(vec![Event::PointerMoved(p)], Modifiers::NONE);
    }
    h.button(h.point(100.0, 150.0), false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection().unwrap().get(120, 120), 255);
    assert_eq!(h.state().doc.selection().unwrap().get(50, 50), 255);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn polygonal_lasso_shift_backspace_enter_cancel_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::PolygonalLasso);
    let pixels = h.state().doc.composite_rgba8();
    for point in [(30.0, 30.0), (80.0, 34.0)] {
        let p = h.point(point.0, point.1);
        h.button(p, true, Modifiers::SHIFT);
        h.button(p, false, Modifiers::SHIFT);
    }
    assert!((h.state().lasso.as_ref().unwrap().points[1].y - 30.0).abs() < 0.001);
    let p = h.point(80.0, 80.0);
    h.button(p, true, Modifiers::NONE);
    h.button(p, false, Modifiers::NONE);
    assert_eq!(h.state().lasso.as_ref().unwrap().points.len(), 3);
    h.step(
        vec![Event::Key {
            key: Key::Backspace,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!(h.state().lasso.as_ref().unwrap().points.len(), 2);
    let p = h.point(80.0, 80.0);
    h.button(p, true, Modifiers::NONE);
    h.button(p, false, Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(h.state().lasso.is_none());
    assert!(h.state().doc.selection().unwrap().get(65, 45) > 0);
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let p = h.point(150.0, 150.0);
    h.button(p, true, Modifiers::NONE);
    h.button(p, false, Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(h.state().lasso.is_none());
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().history.states().len(), count);
    assert!(h.app.active().unwrap().undo());
    assert!(h.state().doc.selection().is_none());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn magnetic_lasso_anchors_cancel_close_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::MagneticLasso);
    let pixels = h.state().doc.composite_rgba8();
    for point in [(30.0, 30.0), (80.0, 30.0), (80.0, 80.0)] {
        let p = h.point(point.0, point.1);
        h.button(p, true, Modifiers::NONE);
        h.button(p, false, Modifiers::NONE);
    }
    let anchors = h.state().magnetic.as_ref().unwrap().anchors.len();
    assert!(anchors >= 3);
    h.step(
        vec![Event::Key {
            key: Key::Backspace,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(h.state().magnetic.as_ref().unwrap().anchors.len() < anchors);
    h.step(
        vec![Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(h.state().lasso.is_none());
    assert!(h.state().magnetic.is_none());
    assert!(h.state().doc.selection().unwrap().get(65, 45) > 0);
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let p = h.point(150.0, 150.0);
    h.button(p, true, Modifiers::SHIFT);
    h.button(p, false, Modifiers::NONE);
    assert_eq!(
        h.state().lasso.as_ref().unwrap().op,
        op_core::SelectionOp::Add
    );
    let p = h.point(180.0, 150.0);
    h.step(vec![Event::PointerMoved(p)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(h.state().magnetic.is_none());
    assert!(h.state().lasso.is_none());
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().history.states().len(), count);
    assert!(h.app.active().unwrap().undo());
    assert!(h.state().doc.selection().is_none());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn object_selection_lasso_roi_cancel_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::ObjectSelection);
    let state = h.app.active().unwrap();
    let id = state.doc.active_layer.unwrap();
    for y in 60..110 {
        for x in 60..110 {
            state
                .doc
                .layer_mut(id)
                .unwrap()
                .image_mut()
                .unwrap()
                .set_pixel(x, y, [200, 20, 20, 255]);
        }
    }
    state.doc.mark_dirty();
    state.record("Fixture");
    let pixels = h.state().doc.composite_rgba8();
    h.app.tool_settings.insert("object.mode", "1".into());
    let a = h.point(30.0, 30.0);
    h.button(a, true, Modifiers::NONE);
    for point in [(140.0, 30.0), (140.0, 140.0), (30.0, 140.0)] {
        let p = h.point(point.0, point.1);
        h.step(vec![Event::PointerMoved(p)], Modifiers::NONE);
    }
    h.button(h.point(30.0, 140.0), false, Modifiers::NONE);
    assert!(h.state().doc.selection().unwrap().get(85, 85) > 0);
    assert_eq!(h.state().doc.selection().unwrap().get(40, 40), 0);
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let a = h.point(150.0, 150.0);
    h.button(a, true, Modifiers::NONE);
    let b = h.point(200.0, 200.0);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(b, false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().history.states().len(), count);
    assert!(h.app.active().unwrap().undo());
    assert!(h.state().doc.selection().is_none());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn quick_selection_cancel_mode_transition_noop_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::QuickSelection);
    let state = h.app.active().unwrap();
    let id = state.doc.active_layer.unwrap();
    for y in 50..110 {
        for x in 50..110 {
            state
                .doc
                .layer_mut(id)
                .unwrap()
                .image_mut()
                .unwrap()
                .set_pixel(x, y, [200, 20, 20, 255]);
        }
    }
    state.doc.mark_dirty();
    state.record("Fixture");
    let pixels = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    let a = h.point(75.0, 75.0);
    h.button(a, true, Modifiers::NONE);
    assert!(h.state().quick.is_some());
    assert!(h.state().doc.selection().unwrap().get(75, 75) > 0);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(a, false, Modifiers::NONE);
    assert!(h.state().doc.selection().is_none());
    assert_eq!(h.state().history.states().len(), count);
    assert_ne!(
        h.app.tool_settings.get("quick.mode").map(String::as_str),
        Some("1")
    );
    h.drag((75.0, 75.0), (85.0, 75.0), Modifiers::NONE);
    assert_eq!(
        h.app.tool_settings.get("quick.mode").map(String::as_str),
        Some("1")
    );
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    h.drag((75.0, 75.0), (85.0, 75.0), Modifiers::NONE);
    assert_eq!(h.state().history.states().len(), count);
    h.drag(
        (75.0, 75.0),
        (85.0, 75.0),
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(h.state().doc.selection().map_or(0, |s| s.get(75, 75)), 0);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    h.app.tool_settings.insert("quick.size", "NaN".into());
    assert_eq!(crate::options_tools::quick_options(&mut h.app).1, 30.0);
}

#[test]
fn magic_wand_sampling_latched_add_noop_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::MagicWand);
    let state = h.app.active().unwrap();
    let id = state.doc.active_layer.unwrap();
    let image = state.doc.layer_mut(id).unwrap().image_mut().unwrap();
    for y in 0..256 {
        for x in 0..256 {
            image.set_pixel(x, y, [100, 100, 100, 255]);
        }
    }
    image.set_pixel(50, 50, [190, 190, 190, 255]);
    image.set_pixel(100, 100, [110, 110, 110, 255]);
    state.doc.mark_dirty();
    state.record("Fixture");
    let pixels = h.state().doc.composite_rgba8();
    h.app.wand.region.tolerance = 0;
    h.app.wand.region.anti_alias = false;
    h.app.wand.region.contiguous = false;
    h.drag((50.5, 50.5), (50.5, 50.5), Modifiers::NONE);
    assert_eq!(h.state().doc.selection().unwrap().get(50, 50), 255);
    assert_eq!(h.state().doc.selection().unwrap().get(100, 100), 0);
    h.app.tool_settings.insert("wand.sample_size", "1".into());
    h.drag((50.5, 50.5), (50.5, 50.5), Modifiers::NONE);
    assert_eq!(h.state().doc.selection().unwrap().get(100, 100), 255);
    assert_eq!(h.state().doc.selection().unwrap().get(50, 50), 0);
    let before = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    h.drag((50.5, 50.5), (50.5, 50.5), Modifiers::NONE);
    assert_eq!(h.state().history.states().len(), count);
    let p = h.point(150.5, 150.5);
    h.button(p, true, Modifiers::SHIFT);
    h.button(p, false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection().unwrap().get(100, 100), 255);
    assert_eq!(h.state().doc.selection().unwrap().get(150, 150), 255);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.selection(), before.as_ref());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn crop_resolution_only_commit_cancel_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Crop);
    let pixels = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    h.app.crop_options.resolution = "300".into();
    h.step(
        vec![Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!(h.state().doc.resolution, 300.0);
    assert_eq!(h.state().history.states().len(), count + 1);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.resolution, 72.0);
    h.app.crop_options.classic = true;
    h.drag((256.0, 256.0), (180.0, 180.0), Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!((h.state().doc.width, h.state().doc.height), (256, 256));
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert!(crate::crop_tool::parse_length("inf px").is_none());
    h.app.crop_options.width = "4 in".into();
    h.app.crop_options.height = "5 in".into();
    h.app.crop_options.resolution = "inf".into();
    assert!(h.app.crop_options.output_size().is_none());
}

#[test]
fn perspective_crop_rejects_crossing_plane_and_undoes_valid_commit_cpu() {
    let mut h = CanvasHarness::new(Tool::PerspectiveCrop);
    let pixels = h.state().doc.composite_rgba8();
    let options = crate::perspective_crop::PerspectiveOptions {
        width: "80".into(),
        height: "60".into(),
        resolution: "144".into(),
        ..Default::default()
    };
    let invalid = [
        egui::pos2(20.0, 20.0),
        egui::pos2(200.0, 200.0),
        egui::pos2(200.0, 20.0),
        egui::pos2(20.0, 200.0),
    ];
    let count = h.state().history.states().len();
    h.app.active().unwrap().perspective_crop = Some(crate::perspective_crop::PerspectiveBox {
        quad: Some(invalid),
        ..Default::default()
    });
    assert!(!crate::perspective_crop::commit(
        h.app.active().unwrap(),
        &options
    ));
    assert!(h.state().perspective_crop.is_some());
    assert_eq!(h.state().history.states().len(), count);
    h.app.active().unwrap().perspective_crop = None;
    h.drag((20.0, 20.0), (200.0, 180.0), Modifiers::NONE);
    assert!(h.state().perspective_crop.as_ref().unwrap().quad.is_some());
    assert!(crate::perspective_crop::commit(
        h.app.active().unwrap(),
        &options
    ));
    assert_eq!((h.state().doc.width, h.state().doc.height), (80, 60));
    assert_eq!(h.state().doc.resolution, 144.0);
    assert!(h.app.active().unwrap().undo());
    assert_eq!((h.state().doc.width, h.state().doc.height), (256, 256));
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    let invalid_size = crate::perspective_crop::PerspectiveOptions {
        width: "4 in".into(),
        height: "4 in".into(),
        resolution: "inf".into(),
        ..Default::default()
    };
    assert!(invalid_size.size().is_none());
}

#[test]
fn slice_creation_styles_cancel_export_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Slice);
    let pixels = h.state().doc.composite_rgba8();
    h.drag((20.0, 30.0), (100.0, 90.0), Modifiers::NONE);
    assert_eq!(h.state().doc.slices[0].bounds, (20, 30, 100, 90));
    let id = h.state().doc.slices[0].id;
    let (w, hh, bytes) = op_core::slice::rgba8(&h.state().doc, id).unwrap();
    assert_eq!((w, hh, bytes.len()), (80, 60, 80 * 60 * 4));
    assert!(
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [255, 255, 255, 255])
    );
    h.app.tool_settings.insert("slice.style", "1".into());
    h.app.tool_settings.insert("slice.w", "2".into());
    h.app.tool_settings.insert("slice.h", "1".into());
    h.drag((120.0, 120.0), (180.0, 150.0), Modifiers::NONE);
    assert_eq!(h.state().doc.slices[1].bounds, (120, 120, 180, 150));
    h.app.tool_settings.insert("slice.style", "2".into());
    h.app.tool_settings.insert("slice.w", "20".into());
    h.app.tool_settings.insert("slice.h", "10".into());
    h.drag((200.0, 200.0), (210.0, 210.0), Modifiers::NONE);
    assert_eq!(h.state().doc.slices[2].bounds, (200, 200, 220, 210));
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.slices.len(), 2);
    let count = h.state().history.states().len();
    let p = h.point(40.0, 140.0);
    h.button(p, true, Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(p, false, Modifiers::NONE);
    assert_eq!(h.state().history.states().len(), count);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn slice_guide_grid_replaces_user_slices_and_deduplicates_cpu() {
    let mut h = CanvasHarness::new(Tool::Slice);
    let d = &mut h.app.active().unwrap().doc;
    d.guides = vec![
        op_core::Guide {
            vertical: true,
            position: 128.0,
            ..Default::default()
        },
        op_core::Guide {
            vertical: true,
            position: 128.0,
            ..Default::default()
        },
        op_core::Guide {
            vertical: false,
            position: 64.0,
            ..Default::default()
        },
    ];
    assert!(op_core::slice::from_guides(d));
    assert_eq!(d.slices.len(), 4);
    assert_eq!(d.slices[0].bounds, (0, 0, 128, 64));
    assert_eq!(d.slices[3].bounds, (128, 64, 256, 256));
    assert!(!op_core::slice::from_guides(d));
}

#[test]
fn slice_select_move_resize_multiselect_cancel_delete_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Slice);
    h.drag((20.0, 20.0), (100.0, 100.0), Modifiers::NONE);
    h.drag((150.0, 20.0), (220.0, 100.0), Modifiers::NONE);
    let pixels = h.state().doc.composite_rgba8();
    h.app.tool = Tool::SliceSelect;
    h.drag((60.0, 60.0), (80.0, 80.0), Modifiers::NONE);
    assert_eq!(h.state().doc.slices[0].bounds, (40, 40, 120, 120));
    h.drag((120.0, 120.0), (140.0, 130.0), Modifiers::NONE);
    assert_eq!(h.state().doc.slices[0].bounds, (40, 40, 140, 130));
    let p = h.point(180.0, 60.0);
    h.button(p, true, Modifiers::SHIFT);
    h.button(p, false, Modifiers::SHIFT);
    assert_eq!(h.state().selected_slices.len(), 2);
    h.drag((80.0, 80.0), (90.0, 90.0), Modifiers::NONE);
    assert_eq!(h.state().doc.slices[0].bounds, (50, 50, 150, 140));
    assert_eq!(h.state().doc.slices[1].bounds, (160, 30, 230, 110));
    let original = h.state().doc.slices.clone();
    let count = h.state().history.states().len();
    let p = h.point(90.0, 90.0);
    h.button(p, true, Modifiers::NONE);
    let q = h.point(120.0, 130.0);
    h.step(vec![Event::PointerMoved(q)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(q, false, Modifiers::NONE);
    assert_eq!(h.state().doc.slices, original);
    assert_eq!(h.state().history.states().len(), count);
    h.step(
        vec![Event::Key {
            key: Key::Delete,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(h.state().doc.slices.is_empty());
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.slices, original);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn frame_rectangle_ellipse_clip_move_cancel_replace_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Frame);
    h.drag((40.0, 40.0), (140.0, 120.0), Modifiers::NONE);
    let id = h.state().doc.active_layer.unwrap();
    let f = h.state().doc.layer(id).unwrap().frame.unwrap();
    assert_eq!(f.bounds, (40, 40, 140, 120));
    assert_eq!(h.state().doc.composite_rgba8()[3], 0);
    let original = h.state().doc.composite_rgba8();
    h.drag((80.0, 80.0), (100.0, 100.0), Modifiers::NONE);
    assert_eq!(
        h.state().doc.layer(id).unwrap().frame.unwrap().bounds,
        (60, 60, 160, 140)
    );
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.composite_rgba8(), original);
    let p = h.point(80.0, 80.0);
    h.button(p, true, Modifiers::NONE);
    let q = h.point(110.0, 110.0);
    h.step(vec![Event::PointerMoved(q)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(q, false, Modifiers::NONE);
    assert_eq!(h.state().doc.layer(id).unwrap().frame, Some(f));
    let ellipse = op_core::frame::Frame {
        elliptical: true,
        ..f
    };
    assert!(op_core::frame::set(
        &mut h.app.active().unwrap().doc,
        id,
        ellipse,
        false
    ));
    h.app.active().unwrap().record("Ellipse Frame");
    let path = std::env::temp_dir().join(format!("openphoto-frame-{}.png", h.state().doc.id.0));
    let source = op_core::Document::new_with_background(
        "red",
        8,
        4,
        op_core::Color::from_rgba8([255, 0, 0, 255]),
    );
    op_io::save(&source, &path).unwrap();
    let p = h.point(90.0, 80.0);
    let ppp = h.ctx.pixels_per_point();
    crate::actions::drop_paths(&mut h.app, vec![path.clone()], Some(p), ppp);
    std::fs::remove_file(path).unwrap();
    assert_eq!(h.app.docs.len(), 1);
    let image = h.state().doc.composite_rgba8();
    let center = ((80 * 256 + 90) * 4) as usize;
    assert_eq!(&image[center..center + 4], &[255, 0, 0, 255]);
    let corner = ((40 * 256 + 40) * 4) as usize;
    assert_eq!(image[corner + 3], 0);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.layer(id).unwrap().frame, Some(ellipse));
}

#[test]
fn frame_resize_preserves_content_and_move_tool_tracks_geometry_cpu() {
    let mut h = CanvasHarness::new(Tool::Frame);
    h.drag((40.0, 40.0), (140.0, 120.0), Modifiers::NONE);
    let id = h.state().doc.active_layer.unwrap();
    let content = h.state().doc.descendants(id)[0];
    let pixels = h
        .state()
        .doc
        .layer(content)
        .unwrap()
        .image()
        .unwrap()
        .clone();
    h.drag((140.0, 120.0), (170.0, 150.0), Modifiers::NONE);
    assert_eq!(
        h.state().doc.layer(id).unwrap().frame.unwrap().bounds,
        (40, 40, 170, 150)
    );
    assert_eq!(
        h.state()
            .doc
            .layer(content)
            .unwrap()
            .image()
            .unwrap()
            .pixel(20, 20),
        pixels.pixel(20, 20)
    );
    let movement = op_core::move_tool::Move::begin(&h.state().doc, [0; 3]).unwrap();
    movement.apply(&mut h.app.active().unwrap().doc, 200, 200);
    assert_eq!(
        h.state().doc.layer(id).unwrap().frame.unwrap().bounds,
        (126, 146, 256, 256)
    );
    movement.cancel(&mut h.app.active().unwrap().doc);
    assert_eq!(
        h.state().doc.layer(id).unwrap().frame.unwrap().bounds,
        (40, 40, 170, 150)
    );
}

#[test]
fn eyedropper_alpha_average_scope_alt_and_transparent_noop_cpu() {
    let mut h = CanvasHarness::new(Tool::Eyedropper);
    let state = h.app.active().unwrap();
    let id = state.doc.active_layer.unwrap();
    let l = state.doc.layer_mut(id).unwrap();
    l.is_background = false;
    *l.image_mut().unwrap() = op_core::TiledImage::new(256, 256);
    l.image_mut().unwrap().set_pixel(50, 50, [255, 0, 0, 128]);
    l.image_mut().unwrap().set_pixel(51, 50, [0, 0, 255, 255]);
    state.doc.mark_dirty();
    state.record("Fixture");
    let pixels = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    h.app.eyedropper.sample = op_core::SampleScope::Current;
    h.app.eyedropper.size = 3;
    h.drag((50.5, 50.5), (50.5, 50.5), Modifiers::NONE);
    assert_eq!(h.app.foreground.to_rgba8(), [85, 0, 170, 255]);
    let foreground = h.app.foreground;
    h.app.eyedropper.size = 1;
    h.drag((51.5, 50.5), (51.5, 50.5), Modifiers::ALT);
    assert_eq!(h.app.background.to_rgba8(), [0, 0, 255, 255]);
    assert_eq!(h.app.foreground, foreground);
    h.drag((100.5, 100.5), (100.5, 100.5), Modifiers::NONE);
    assert_eq!(h.app.foreground, foreground);
    h.drag((-10.5, -10.5), (-10.5, -10.5), Modifiers::NONE);
    assert_eq!(h.app.foreground, foreground);
    assert_eq!(h.state().history.states().len(), count);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    let state = h.app.active().unwrap();
    let below = op_core::Layer::raster(
        state.doc.new_layer_id(),
        "below",
        op_core::TiledImage::filled(256, 256, [0, 255, 0, 255]),
    );
    state.doc.layers.insert(0, below);
    let above = op_core::Layer::raster(
        state.doc.new_layer_id(),
        "above",
        op_core::TiledImage::filled(256, 256, [0, 0, 255, 255]),
    );
    state.doc.layers.push(above);
    state.doc.mark_dirty();
    let current = state
        .sample_average(50, 50, 0, op_core::SampleScope::Current)
        .unwrap()
        .to_rgba8();
    assert_eq!(current, [255, 0, 0, 255]);
    let below = state
        .sample_average(50, 50, 1, op_core::SampleScope::CurrentAndBelow)
        .unwrap()
        .to_rgba8();
    assert!(below[0] > 0 && below[1] > 0 && below[2] == 0);
    assert_eq!(
        state
            .sample_average(50, 50, 1, op_core::SampleScope::All)
            .unwrap()
            .to_rgba8(),
        [0, 0, 255, 255]
    );
}

#[test]
fn color_sampler_limit_cancel_move_alt_delete_and_tool_switch_cpu() {
    let mut h = CanvasHarness::new(Tool::ColorSampler);
    let count = h.state().history.states().len();
    let pixels = h.state().doc.composite_rgba8();
    for x in 0..10 {
        h.drag(
            (20.5 + x as f32 * 20.0, 40.5),
            (20.5 + x as f32 * 20.0, 40.5),
            Modifiers::NONE,
        );
    }
    assert_eq!(h.state().color_samplers.len(), 10);
    h.drag((100.5, 150.5), (100.5, 150.5), Modifiers::NONE);
    assert_eq!(h.state().color_samplers.len(), 10);
    let before = h.state().color_samplers.clone();
    let p = h.point(20.5, 40.5);
    h.button(p, true, Modifiers::NONE);
    let q = h.point(40.5, 100.5);
    h.step(vec![Event::PointerMoved(q)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(q, false, Modifiers::NONE);
    assert_eq!(h.state().color_samplers, before);
    h.drag((20.5, 40.5), (30.5, 80.5), Modifiers::NONE);
    assert_eq!(h.state().color_samplers[0], egui::pos2(30.0, 80.0));
    h.drag((30.5, 80.5), (30.5, 80.5), Modifiers::ALT);
    assert_eq!(h.state().color_samplers.len(), 9);
    let before = h.state().color_samplers.clone();
    let p = h.point(100.5, 150.5);
    h.button(p, true, Modifiers::NONE);
    assert_eq!(h.state().color_samplers.len(), 10);
    h.app.tool = Tool::Eyedropper;
    h.step(Vec::new(), Modifiers::NONE);
    h.button(p, false, Modifiers::NONE);
    assert_eq!(h.state().color_samplers, before);
    assert!(h.state().sampler_drag.is_none());
    assert_eq!(h.state().history.states().len(), count);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn ruler_line_protractor_drag_cancel_readout_straighten_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Ruler);
    let s = h.app.active().unwrap();
    let id = s.doc.active_layer.unwrap();
    s.doc.layer_mut(id).unwrap().is_background = false;
    s.doc.mark_dirty();
    s.record("Fixture");
    let pixels = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    h.drag((20.0, 20.0), (100.0, 80.0), Modifiers::NONE);
    let m = h.state().measurement.unwrap();
    assert_eq!(m.readout("L1: 0.00").unwrap(), "L1: 100.00");
    assert_eq!(m.readout("A: 0.0°").unwrap(), "A: 36.9°");
    h.drag((20.0, 20.0), (20.0, 120.0), Modifiers::ALT);
    let m = h.state().measurement.unwrap();
    assert!(m.c.unwrap().distance(egui::pos2(20.0, 120.0)) < 1e-3);
    assert_eq!(m.readout("A: 0.0°").unwrap(), "A: 53.1°");
    assert_eq!(m.readout("L2:").unwrap(), "L2: 100.00");
    let p = h.point(20.0, 120.0);
    h.button(p, true, Modifiers::NONE);
    let q = h.point(40.0, 160.0);
    h.step(vec![Event::PointerMoved(q)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(q, false, Modifiers::NONE);
    assert_eq!(h.state().measurement, Some(m));
    h.drag((60.0, 50.0), (70.0, 60.0), Modifiers::NONE);
    assert!(
        h.state()
            .measurement
            .unwrap()
            .a
            .distance(egui::pos2(30.0, 30.0))
            < 1e-3
    );
    assert_eq!(h.state().history.states().len(), count);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    crate::measure_tool::straighten(&mut h.app);
    assert!(h.state().measurement.is_none());
    assert_eq!(h.state().history.states().len(), count + 1);
    assert_ne!(h.state().doc.composite_rgba8(), pixels);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    h.app.active().unwrap().measurement = None;
    h.drag((20.0, 20.0), (100.0, 80.0), Modifiers::SHIFT);
    let m = h.state().measurement.unwrap();
    assert!(((m.b - m.a) - egui::vec2(70.0, 70.0)).length() < 1e-3);
}

#[test]
fn note_creation_unicode_edit_move_cancel_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Note);
    h.app.tool_settings.insert("note.author", "Alice".into());
    h.app
        .tool_settings
        .insert("note.color", "100,150,200".into());
    let pixels = h.state().doc.composite_rgba8();
    h.drag((10.0, 20.0), (10.0, 20.0), Modifiers::NONE);
    assert_eq!(h.state().doc.notes.len(), 1);
    assert_eq!(h.state().doc.notes[0].author, "Alice");
    assert_eq!(h.state().doc.notes[0].color, [100, 150, 200]);
    let edit = h.app.active().unwrap().note_edit.as_mut().unwrap();
    edit.text = "First line\n第二行 ✨".into();
    edit.color = [255, 200, 0];
    crate::note_tool::commit(h.app.active().unwrap());
    assert_eq!(h.state().doc.notes[0].text, "First line\n第二行 ✨");
    crate::note_tool::toggle(h.app.active().unwrap());
    assert!(h.state().note_edit.is_some());
    crate::note_tool::toggle(h.app.active().unwrap());
    assert!(h.state().note_edit.is_none());
    h.drag((10.0, 20.0), (50.0, 60.0), Modifiers::NONE);
    assert_eq!(
        (h.state().doc.notes[0].x, h.state().doc.notes[0].y),
        (50, 60)
    );
    assert!(h.app.active().unwrap().undo());
    assert_eq!(
        (h.state().doc.notes[0].x, h.state().doc.notes[0].y),
        (10, 20)
    );
    h.app.active().unwrap().note_edit = None;
    let count = h.state().history.states().len();
    let p = h.point(10.0, 20.0);
    h.button(p, true, Modifiers::NONE);
    let q = h.point(100.0, 120.0);
    h.step(vec![Event::PointerMoved(q)], Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(q, false, Modifiers::NONE);
    assert_eq!(
        (h.state().doc.notes[0].x, h.state().doc.notes[0].y),
        (10, 20)
    );
    assert_eq!(h.state().history.states().len(), count);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn saving_a_document_commits_pending_note_text_cpu() {
    let mut h = CanvasHarness::new(Tool::Note);
    h.drag((20.0, 20.0), (20.0, 20.0), Modifiers::NONE);
    h.app.active().unwrap().mark_saved();
    assert!(!h.state().is_dirty());
    h.app.active().unwrap().note_edit.as_mut().unwrap().text = "Pending text — 保存".into();
    assert!(h.state().is_dirty());
    let id = h.state().doc.id;
    let path = std::env::temp_dir().join(format!("openphoto-note-{}.psd", id.0));
    assert!(crate::actions::write_to(
        &mut h.app,
        id,
        path.clone(),
        false,
        &op_io::ExportOptions::default()
    ));
    let back = op_io::open(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(back.notes[0].text, "Pending text — 保存");
    assert!(h.state().note_edit.is_none());
    assert!(!h.state().is_dirty());
}

#[test]
fn count_points_groups_styles_cancel_visibility_clear_and_undo_cpu() {
    let mut h = CanvasHarness::new(Tool::Count);
    let pixels = h.state().doc.composite_rgba8();
    h.drag((30.0, 30.0), (30.0, 30.0), Modifiers::NONE);
    h.drag((100.0, 50.0), (100.0, 50.0), Modifiers::NONE);
    assert_eq!(h.state().doc.count_groups.len(), 1);
    assert_eq!(
        h.state().doc.count_groups[0].points,
        vec![(30, 30), (100, 50)]
    );
    h.drag((30.0, 30.0), (60.0, 80.0), Modifiers::NONE);
    assert_eq!(h.state().doc.count_groups[0].points[0], (60, 80));
    h.drag((100.0, 50.0), (100.0, 50.0), Modifiers::ALT);
    assert_eq!(h.state().doc.count_groups[0].points.len(), 1);
    let original = h.state().doc.count_groups.clone();
    let count = h.state().history.states().len();
    let p = h.point(200.0, 200.0);
    h.button(p, true, Modifiers::NONE);
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    h.button(p, false, Modifiers::NONE);
    assert_eq!(h.state().doc.count_groups, original);
    assert_eq!(h.state().history.states().len(), count);
    crate::count_tool::new_group(&mut h.app);
    h.drag((200.0, 50.0), (200.0, 50.0), Modifiers::NONE);
    assert_eq!(h.state().doc.count_groups.len(), 2);
    assert_eq!(h.state().doc.count_groups[1].points, vec![(200, 50)]);
    crate::count_tool::group_action(&mut h.app, "visibility");
    assert!(!h.state().doc.count_groups[1].visible);
    crate::count_tool::apply(
        h.app.active().unwrap(),
        crate::count_tool::Options {
            marker: 4,
            label: 12,
            color: [200, 50, 100],
        },
    );
    crate::count_tool::sync(&mut h.app);
    assert_eq!(h.state().doc.count_groups[1].marker, 4);
    assert!(h.app.active().unwrap().undo());
    let history = h.state().history.current_id();
    h.step(Vec::new(), Modifiers::NONE);
    assert_eq!(h.state().doc.count_groups[1].marker, 2);
    assert_eq!(h.state().history.current_id(), history);
    crate::count_tool::group_action(&mut h.app, "clear");
    assert!(h.state().doc.count_groups[1].points.is_empty());
    assert!(h.app.active().unwrap().undo());
    let original = h.state().doc.count_groups.clone();
    crate::count_tool::group_action(&mut h.app, "delete");
    assert_eq!(h.state().doc.count_groups.len(), 1);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.count_groups, original);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn count_group_name_draft_commit_and_saved_metadata_cpu() {
    let mut h = CanvasHarness::new(Tool::Count);
    h.drag((20.0, 20.0), (20.0, 20.0), Modifiers::NONE);
    let id = h.state().doc.count_groups[0].id;
    h.app.active().unwrap().mark_saved();
    h.app.active().unwrap().count_name_edit = Some((id, "星点 ✨".into()));
    assert!(h.state().is_dirty());
    let doc = h.state().doc.id;
    let path = std::env::temp_dir().join(format!("openphoto-count-{}.psd", doc.0));
    assert!(crate::actions::write_to(
        &mut h.app,
        doc,
        path.clone(),
        false,
        &op_io::ExportOptions::default()
    ));
    let back = op_io::open(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(back.count_groups[0].name, "星点 ✨");
    assert!(!h.state().is_dirty());
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.count_groups[0].name, "Count Group 1");
}

#[test]
fn frame_creates_independent_siblings_and_retains_existing_contents_cpu() {
    let mut h = CanvasHarness::new(Tool::Frame);
    h.drag((20.0, 20.0), (100.0, 100.0), Modifiers::NONE);
    let first = h.state().doc.active_layer.unwrap();
    let original = h.state().doc.descendants(first);
    h.drag((150.0, 20.0), (230.0, 100.0), Modifiers::NONE);
    let second = h.state().doc.active_layer.unwrap();
    assert_ne!(first, second);
    assert_eq!(h.state().doc.layer(second).unwrap().parent, None);
    assert_eq!(h.state().doc.descendants(first), original);
    let source = op_core::TiledImage::filled(8, 8, [255, 0, 0, 255]);
    assert!(op_core::frame::place(
        &mut h.app.active().unwrap().doc,
        second,
        &source,
        "red"
    ));
    let image = h.state().doc.composite_rgba8();
    let i = ((60 * 256 + 180) * 4) as usize;
    assert_eq!(&image[i..i + 4], &[255, 0, 0, 255]);
    let content = h.state().doc.descendants(first)[0];
    h.app.active().unwrap().doc.select_layer(content);
    h.drag((150.0, 150.0), (220.0, 220.0), Modifiers::NONE);
    let third = h.state().doc.active_layer.unwrap();
    assert_eq!(h.state().doc.layer(third).unwrap().parent, None);
    assert_eq!(h.state().doc.descendants(first), original);
}

#[test]
fn patch_escape_preserves_selection_pixels_and_history_cpu() {
    let mut h = CanvasHarness::new(Tool::Patch);
    h.app
        .active()
        .unwrap()
        .doc
        .set_selection(Some(op_core::Selection::rect(
            256,
            256,
            op_core::selection::Rect::new(40.0, 40.0, 80.0, 80.0),
        )));
    let pixels = h.state().doc.composite_rgba8();
    let selection = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let a = h.point(50.5, 50.5);
    let b = h.point(130.5, 110.5);
    h.button(a, true, Modifiers::NONE);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    assert!(h.state().patch_drag.is_some());
    assert!(h.state().busy());
    h.step(
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(h.state().patch_drag.is_none());
    h.button(b, false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection(), selection.as_ref());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert_eq!(h.state().history.states().len(), count);
}

#[test]
fn clone_stamp_samples_and_writes_only_the_active_mask_cpu() {
    let mut h = CanvasHarness::new(Tool::CloneStamp);
    h.app.clone_stamp.size = 12.0;
    let state = h.app.active().unwrap();
    let id = state.doc.active_layer.unwrap();
    let mut mask = op_core::layer::LayerMask::filled(256, 256, 255);
    for y in 30..70 {
        for x in 30..70 {
            mask.image.set_pixel(x, y, [0, 0, 0, 255]);
        }
    }
    state.doc.layer_mut(id).unwrap().mask = Some(mask);
    state.doc.mask_target = true;
    state.clone_source = Some(egui::pos2(50.5, 50.5));
    state.record("Mask");
    let pixels = state.doc.layer(id).unwrap().image().unwrap().to_rgba8();
    let count = state.history.states().len();
    h.drag((120.5, 120.5), (130.5, 120.5), Modifiers::NONE);
    let layer = h.state().doc.layer(id).unwrap();
    assert_eq!(layer.mask.as_ref().unwrap().value(120, 120), 0);
    assert_eq!(layer.mask.as_ref().unwrap().value(200, 200), 255);
    assert_eq!(layer.image().unwrap().to_rgba8(), pixels);
    assert_eq!(h.state().history.states().len(), count + 1);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(
        h.state()
            .doc
            .layer(id)
            .unwrap()
            .mask
            .as_ref()
            .unwrap()
            .value(120, 120),
        255
    );
}

#[test]
fn history_brush_restores_the_corresponding_mask_and_undoes_cpu() {
    for quick in [false, true] {
        let mut h = CanvasHarness::new(Tool::HistoryBrush);
        h.app.history_brush.size = 12.0;
        let state = h.app.active().unwrap();
        let id = state.doc.active_layer.unwrap();
        if quick {
            state.doc.enter_quick_mask();
        } else {
            state.doc.layer_mut(id).unwrap().mask =
                Some(op_core::layer::LayerMask::filled(256, 256, 255));
            state.doc.mask_target = true;
        }
        state
            .doc
            .edit_target()
            .unwrap()
            .image
            .set_pixel(120, 120, [0, 0, 0, 255]);
        state.record("Mask Source");
        state.history_source = Some(state.history.current_id());
        state
            .doc
            .edit_target()
            .unwrap()
            .image
            .set_pixel(120, 120, [255, 255, 255, 255]);
        state.record("Erase Mask");
        let pixels = state.doc.layer(id).unwrap().image().unwrap().to_rgba8();
        h.drag((120.5, 120.5), (122.5, 120.5), Modifiers::NONE);
        assert_eq!(
            h.app
                .active()
                .unwrap()
                .doc
                .edit_target()
                .unwrap()
                .image
                .pixel(120, 120),
            [0, 0, 0, 255]
        );
        assert_eq!(
            h.state().doc.layer(id).unwrap().image().unwrap().to_rgba8(),
            pixels
        );
        assert!(h.app.active().unwrap().undo());
        assert_eq!(
            h.app
                .active()
                .unwrap()
                .doc
                .edit_target()
                .unwrap()
                .image
                .pixel(120, 120),
            [255, 255, 255, 255]
        );
    }
}

#[test]
fn gradient_escape_cancels_before_pixels_and_valid_release_undoes_cpu() {
    let mut h = CanvasHarness::new(Tool::Gradient);
    let pixels = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    let a = h.point(30.5, 100.5);
    let b = h.point(200.5, 100.5);
    h.button(a, true, Modifiers::NONE);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    assert!(h.state().gradient_drag.is_some());
    assert!(h.state().busy());
    h.key(Key::Escape);
    assert!(h.state().gradient_drag.is_none());
    h.button(b, false, Modifiers::NONE);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert_eq!(h.state().history.states().len(), count);
    h.drag((30.5, 100.5), (200.5, 100.5), Modifiers::NONE);
    assert_ne!(h.state().doc.composite_rgba8(), pixels);
    assert_eq!(h.state().history.states().len(), count + 1);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
}

#[test]
fn shape_escape_preserves_layers_and_subsequent_shapes_undo_cpu() {
    for tool in [
        Tool::Rectangle,
        Tool::Ellipse,
        Tool::Triangle,
        Tool::Polygon,
        Tool::Line,
    ] {
        let mut h = CanvasHarness::new(tool);
        let pixels = h.state().doc.composite_rgba8();
        let count = h.state().history.states().len();
        let layers = h.state().doc.layers.len();
        let a = h.point(30.5, 40.5);
        let b = h.point(150.5, 170.5);
        h.button(a, true, Modifiers::NONE);
        h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
        assert!(h.state().shape_drag.is_some(), "{tool:?}");
        h.key(Key::Escape);
        assert!(h.state().shape_drag.is_none(), "{tool:?}");
        h.button(b, false, Modifiers::NONE);
        assert_eq!(h.state().doc.layers.len(), layers, "{tool:?}");
        assert_eq!(h.state().doc.composite_rgba8(), pixels, "{tool:?}");
        assert_eq!(h.state().history.states().len(), count, "{tool:?}");
        h.drag((30.5, 40.5), (150.5, 170.5), Modifiers::NONE);
        assert_eq!(h.state().doc.layers.len(), layers + 1, "{tool:?}");
        assert_eq!(h.state().history.states().len(), count + 1, "{tool:?}");
        assert!(h.app.active().unwrap().undo());
        assert_eq!(h.state().doc.composite_rgba8(), pixels, "{tool:?}");
    }
}

#[test]
fn rotate_view_escape_resets_and_does_not_resume_the_old_drag_cpu() {
    let mut h = CanvasHarness::new(Tool::RotateView);
    let pixels = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    let a = h.point(190.5, 128.0);
    let b = h.point(128.0, 190.5);
    h.button(a, true, Modifiers::NONE);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    assert!((h.state().view.rotation - 90.0).abs() < 0.01);
    h.key(Key::Escape);
    assert_eq!(h.state().view.rotation, 0.0);
    h.step(
        vec![Event::PointerMoved(b + egui::vec2(20.0, 20.0))],
        Modifiers::NONE,
    );
    assert_eq!(h.state().view.rotation, 0.0);
    h.button(b, false, Modifiers::NONE);
    assert_eq!(h.state().view.rotation, 0.0);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert_eq!(h.state().history.states().len(), count);
}

#[test]
fn zoom_escape_restores_scrubby_view_and_discards_zoom_boxes_cpu() {
    for scrubby in [false, true] {
        let mut h = CanvasHarness::new(Tool::Zoom);
        h.app.set_flag("zoom.scrubby", scrubby);
        let pixels = h.state().doc.composite_rgba8();
        let count = h.state().history.states().len();
        let zoom = h.state().view.zoom;
        let offset = h.state().view.offset;
        let a = h.point(100.5, 100.5);
        let b = a + egui::vec2(100.0, 80.0);
        h.button(a, true, Modifiers::NONE);
        h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
        if scrubby {
            assert!(h.state().view.zoom > zoom);
        }
        h.key(Key::Escape);
        assert!((h.state().view.zoom - zoom).abs() < 0.0001);
        assert!((h.state().view.offset - offset).length() < 0.001);
        h.step(
            vec![Event::PointerMoved(b + egui::vec2(10.0, 10.0))],
            Modifiers::NONE,
        );
        h.button(b, false, Modifiers::NONE);
        assert!((h.state().view.zoom - zoom).abs() < 0.0001);
        assert!((h.state().view.offset - offset).length() < 0.001);
        assert_eq!(h.state().doc.composite_rgba8(), pixels);
        assert_eq!(h.state().history.states().len(), count);
    }
}

#[test]
fn toolbar_screen_mode_button_cycles_and_preserves_the_full_screen_prompt_cpu() {
    use crate::state::ScreenMode;
    let mut h = CanvasHarness::new(Tool::Hand);
    let pixels = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    let origin = h.toolbar_step(Vec::new());
    let p = origin + egui::vec2(crate::theme::pt(18.0), crate::theme::pt(721.5));
    h.toolbar_click(p);
    assert_eq!(h.app.screen_mode, ScreenMode::FullWithMenus);
    h.toolbar_click(p);
    assert!(h.app.full_screen_prompt.is_some());
    assert_eq!(h.app.screen_mode, ScreenMode::FullWithMenus);
    h.app.full_screen_prompt = None;
    h.app.enter_screen_mode(ScreenMode::Full);
    assert!(h.app.hide_tools && h.app.hide_panels);
    h.key(Key::F);
    assert_eq!(h.app.screen_mode, ScreenMode::Standard);
    assert!(!h.app.hide_tools && !h.app.hide_panels);
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert_eq!(h.state().history.states().len(), count);
}

#[test]
fn content_aware_move_escape_preserves_selection_pixels_and_history_cpu() {
    let mut h = CanvasHarness::new(Tool::ContentAwareMove);
    h.app
        .active()
        .unwrap()
        .doc
        .set_selection(Some(op_core::Selection::rect(
            256,
            256,
            op_core::selection::Rect::new(40.0, 40.0, 80.0, 80.0),
        )));
    let pixels = h.state().doc.composite_rgba8();
    let selection = h.state().doc.selection().cloned();
    let count = h.state().history.states().len();
    let a = h.point(50.5, 50.5);
    let b = h.point(130.5, 110.5);
    h.button(a, true, Modifiers::NONE);
    h.step(vec![Event::PointerMoved(b)], Modifiers::NONE);
    assert!(h.state().patch_drag.is_some());
    h.key(Key::Escape);
    assert!(h.state().patch_drag.is_none());
    h.button(b, false, Modifiers::NONE);
    assert_eq!(h.state().doc.selection(), selection.as_ref());
    assert_eq!(h.state().doc.composite_rgba8(), pixels);
    assert_eq!(h.state().history.states().len(), count);
}

fn removal_fixture(h: &mut CanvasHarness) {
    h.app.tool_settings.insert("remove.size", "20".into());
    h.app.tool_settings.insert("remove.ai_mode", "2".into());
    let state = h.app.active().unwrap();
    let id = state.doc.active_layer.unwrap();
    for y in 99..107 {
        for x in 99..107 {
            state
                .doc
                .layer_mut(id)
                .unwrap()
                .image_mut()
                .unwrap()
                .set_pixel(x, y, [0, 0, 0, 255]);
        }
    }
    state.doc.mark_dirty();
    state.record("Fixture");
}
#[test]
fn remove_accumulates_subtracts_cancels_commits_and_undoes_cpu() {
    let mut h = CanvasHarness::new(Tool::Remove);
    removal_fixture(&mut h);
    h.app.set_flag("remove.after_stroke", false);
    let before = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    h.drag((101.5, 101.5), (105.5, 103.5), Modifiers::NONE);
    assert!(
        h.state()
            .remove_session
            .as_ref()
            .unwrap()
            .area
            .as_ref()
            .unwrap()
            .get(101, 101)
            > 200
    );
    assert_eq!(h.state().doc.composite_rgba8(), before);
    assert_eq!(h.state().history.states().len(), count);
    h.drag((101.5, 101.5), (101.5, 101.5), Modifiers::ALT);
    assert_eq!(
        h.state()
            .remove_session
            .as_ref()
            .unwrap()
            .area
            .as_ref()
            .unwrap()
            .get(101, 101),
        0
    );
    h.key(Key::Escape);
    assert!(h.state().remove_session.is_none());
    h.drag((101.5, 101.5), (105.5, 103.5), Modifiers::NONE);
    h.key(Key::Enter);
    assert!(h.state().remove_session.is_none());
    assert_eq!(h.state().doc.composite_rgba8(), vec![255; 256 * 256 * 4]);
    assert_eq!(h.state().history.states().len(), count + 1);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.composite_rgba8(), before);
    assert!(h.app.active().unwrap().redo());
    assert_ne!(h.state().doc.composite_rgba8(), before);
}
#[test]
fn remove_new_layer_sample_all_and_psd_roundtrip_cpu() {
    let mut h = CanvasHarness::new(Tool::Remove);
    removal_fixture(&mut h);
    h.app.set_flag("remove.all_layers", true);
    h.app.set_flag("remove.new_layer", true);
    let old = h.state().doc.active_layer.unwrap();
    let before = h
        .state()
        .doc
        .layer(old)
        .unwrap()
        .image()
        .unwrap()
        .to_rgba8();
    h.drag((101.5, 101.5), (105.5, 103.5), Modifiers::NONE);
    assert_eq!(h.state().doc.layers.len(), 2);
    assert_eq!(
        h.state()
            .doc
            .layer(old)
            .unwrap()
            .image()
            .unwrap()
            .to_rgba8(),
        before
    );
    let active = h
        .state()
        .doc
        .layer(h.state().doc.active_layer.unwrap())
        .unwrap();
    assert_eq!(active.image().unwrap().pixel(103, 103), [255; 4]);
    assert_eq!(active.image().unwrap().pixel(20, 20), [0; 4]);
    let path = std::env::temp_dir().join(format!("openphoto-remove-{}.psd", h.state().doc.id.0));
    op_io::save(&h.state().doc, &path).unwrap();
    let back = op_io::open(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(back.composite_rgba8(), h.state().doc.composite_rgba8());
    assert_eq!(back.layers.len(), 2);
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.layers.len(), 1);
}
#[test]
fn remove_genai_rejects_without_fallback_and_can_retry_offline_cpu() {
    let mut h = CanvasHarness::new(Tool::Remove);
    removal_fixture(&mut h);
    h.app.tool_settings.insert("remove.ai_mode", "1".into());
    let before = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    h.drag((101.5, 101.5), (105.5, 103.5), Modifiers::NONE);
    assert!(
        h.app
            .alert
            .as_ref()
            .is_some_and(|s| s.contains("Generative AI"))
    );
    assert!(h.state().remove_session.as_ref().unwrap().area.is_some());
    assert_eq!(h.state().doc.composite_rgba8(), before);
    assert_eq!(h.state().history.states().len(), count);
    h.app.alert = None;
    h.app.tool_settings.insert("remove.ai_mode", "2".into());
    h.key(Key::Enter);
    assert!(h.state().remove_session.is_none());
    assert_ne!(h.state().doc.composite_rgba8(), before);
}

#[test]
fn remove_pressure_cancel_and_tool_switch_preserve_pixels_cpu() {
    let mut h = CanvasHarness::new(Tool::Remove);
    removal_fixture(&mut h);
    h.app.set_flag("remove.after_stroke", false);
    h.app.pen_pressure = Some(0.25);
    let before = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    h.drag((101.5, 101.5), (101.5, 101.5), Modifiers::NONE);
    let saved = h.state().remove_session.as_ref().unwrap().area.clone();
    assert_eq!(saved.as_ref().unwrap().get(108, 101), 0);
    assert!(saved.as_ref().unwrap().get(101, 101) > 200);
    let a = h.point(140.5, 140.5);
    h.button(a, true, Modifiers::NONE);
    h.key(Key::Escape);
    h.button(a, false, Modifiers::NONE);
    assert_eq!(h.state().remove_session.as_ref().unwrap().area, saved);
    h.app.tool = Tool::Move;
    h.step(Vec::new(), Modifiers::NONE);
    assert!(h.state().remove_session.is_none());
    assert!(!h.state().busy());
    assert_eq!(h.state().doc.composite_rgba8(), before);
    assert_eq!(h.state().history.states().len(), count);
}

#[test]
fn remove_pending_undo_redo_changes_marks_and_not_document_history_cpu() {
    let mut h = CanvasHarness::new(Tool::Remove);
    removal_fixture(&mut h);
    h.app.set_flag("remove.after_stroke", false);
    let before = h.state().doc.composite_rgba8();
    let count = h.state().history.states().len();
    h.drag((101.5, 101.5), (105.5, 103.5), Modifiers::NONE);
    let area = h.state().remove_session.as_ref().unwrap().area.clone();
    assert!(h.app.active().unwrap().undo());
    assert!(h.state().remove_session.as_ref().unwrap().area.is_none());
    assert!(h.app.active().unwrap().redo());
    assert_eq!(h.state().remove_session.as_ref().unwrap().area, area);
    assert_eq!(h.state().doc.composite_rgba8(), before);
    assert_eq!(h.state().history.states().len(), count);
    h.key(Key::Enter);
    assert!(h.state().remove_session.is_none());
    assert!(h.app.active().unwrap().undo());
    assert_eq!(h.state().doc.composite_rgba8(), before);
}
