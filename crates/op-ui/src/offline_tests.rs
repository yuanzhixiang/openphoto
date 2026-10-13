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
