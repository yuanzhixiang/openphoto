//! Adjustment Brush paints masks on retained adjustment layers.
use crate::{
    document_view::{to_doc, to_screen},
    state::{AppState, DocState},
};
use egui::{Key, Pos2, Rect, Ui};
use op_core::{
    adjustment_layer::{self, Kind},
    paint::{BrushTip, Pressure, Stroke, StrokeKind},
};
pub struct Session {
    before: op_core::Snapshot,
    before_mask_target: bool,
    stroke: Stroke,
    initial: Vec<u8>,
}
#[derive(Clone, Copy)]
pub struct Options {
    kind: Kind,
    size: f32,
    opacity: f32,
    flow: f32,
    subtract: bool,
    pressure: Pressure,
    airbrush: bool,
    overlay: bool,
}
pub fn options(app: &mut AppState) -> Options {
    let index = app
        .setting("adjbrush.adjustment", "0")
        .parse::<usize>()
        .unwrap_or(0)
        .min(5);
    let mut number = |key: &'static str, default: &str, fallback: f32| {
        crate::options_bar::typed_number(app.setting(key, default))
            .filter(|v| v.is_finite())
            .unwrap_or(fallback)
    };
    let size = number("adjbrush.size", "100", 100.).clamp(1., 5000.);
    let opacity = number("adjbrush.opacity", "100%", 100.).clamp(0., 100.) / 100.;
    let flow = number("adjbrush.flow", "100%", 100.).clamp(0., 100.) / 100.;
    Options {
        kind: Kind::ALL[index],
        size,
        opacity,
        flow,
        subtract: app.setting("adjbrush.mode", "1") == "0",
        pressure: Pressure {
            size: app.flag("adjbrush.pressure", false),
            opacity: app.flag("adjbrush.opacity_pressure", false),
            flow: false,
        },
        airbrush: app.flag("adjbrush.airbrush", false),
        overlay: app.flag("adjbrush.overlay", false),
    }
}
pub fn cancel(state: &mut DocState) {
    if let Some(s) = state.adjustment_session.take() {
        state.doc.restore(&s.before);
        state.doc.mask_target = s.before_mask_target;
    }
}
pub fn input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    options: Options,
    pressure: Option<f32>,
    ppp: f32,
) {
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        cancel(state);
        return;
    }
    if response.hovered()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(p) = ui.input(|i| i.pointer.interact_pos())
    {
        if state.doc.quick_mask.is_some() || options.opacity == 0. || options.flow == 0. {
            return;
        }
        let before = state.doc.snapshot();
        let before_mask_target = state.doc.mask_target;
        let subtract = options.subtract ^ ui.input(|i| i.modifiers.alt);
        let existing = state
            .doc
            .active_layer
            .and_then(|id| state.doc.layer(id))
            .is_some_and(|l| l.adjustment.is_some_and(|a| a.kind == options.kind));
        if subtract && !existing {
            return;
        }
        let created = !existing;
        if created {
            adjustment_layer::create(
                &mut state.doc,
                adjustment_layer::Settings::new(options.kind),
            );
        }
        if existing
            && state
                .doc
                .active_layer
                .and_then(|id| state.doc.layer(id))
                .is_some_and(|l| l.mask.is_none())
        {
            let id = state.doc.active_layer.unwrap();
            state.doc.layer_mut(id).unwrap().mask = Some(op_core::LayerMask::filled(
                state.doc.width,
                state.doc.height,
                0,
            ));
        }
        state.doc.mask_target = true;
        let tip = BrushTip {
            diameter: options.size,
            hardness: 0.,
            aliased: false,
            square: false,
            angle: 0.,
            roundness: 1.,
            spacing: 0.1,
        };
        let stroke = Stroke::begin(
            &state.doc,
            tip,
            StrokeKind::Paint(if subtract { [0; 3] } else { [255; 3] }),
            options.opacity,
            options.flow,
        );
        let Ok(stroke) = stroke else {
            state.doc.restore(&before);
            state.doc.mask_target = before_mask_target;
            return;
        };
        let id = stroke.layer();
        let initial = state
            .doc
            .layer(id)
            .unwrap()
            .mask
            .as_ref()
            .unwrap()
            .image
            .to_rgba8();
        state.adjustment_session = Some(Session {
            before,
            before_mask_target,
            stroke: stroke.with_pressure(options.pressure),
            initial,
        });
        let p = to_doc(state, p, ppp);
        let s = state.adjustment_session.as_mut().unwrap();
        s.stroke
            .add_point_with_pressure(&mut state.doc, p.x, p.y, pressure.unwrap_or(1.));
    }
    if state.adjustment_session.is_some() {
        if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
            let p = to_doc(state, p, ppp);
            let s = state.adjustment_session.as_mut().unwrap();
            s.stroke
                .add_point_with_pressure(&mut state.doc, p.x, p.y, pressure.unwrap_or(1.));
            if options.airbrush && ui.input(|i| i.pointer.primary_down()) {
                s.stroke.build_up(&mut state.doc);
                ui.ctx().request_repaint();
            }
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            let s = state.adjustment_session.take().unwrap();
            let changed = state
                .doc
                .layer(s.stroke.layer())
                .and_then(|l| l.mask.as_ref())
                .is_some_and(|m| m.image.to_rgba8() != s.initial);
            if changed {
                state.record("Adjustment Brush Tool");
            } else {
                state.doc.restore(&s.before);
                state.doc.mask_target = s.before_mask_target;
            }
        }
    }
}
pub fn draw(ui: &Ui, state: &DocState, canvas: Rect, ppp: f32, options: Options) {
    if !options.overlay {
        return;
    }
    let Some(mask) = state
        .doc
        .active_layer
        .and_then(|id| state.doc.layer(id))
        .filter(|l| l.adjustment.is_some())
        .and_then(|l| l.mask.as_ref())
    else {
        return;
    };
    let painter = ui.painter_at(canvas);
    for y in 0..state.doc.height {
        let mut x = 0;
        while x < state.doc.width {
            let v = mask.image.pixel(x, y)[0];
            let start = x;
            x += 1;
            while x < state.doc.width && mask.image.pixel(x, y)[0] == v {
                x += 1;
            }
            if v == 0 {
                continue;
            }
            let corners = [(start, y), (x, y), (x, y + 1), (start, y + 1)]
                .map(|(x, y)| to_screen(state, Pos2::new(x as f32, y as f32), ppp));
            painter.add(egui::Shape::convex_polygon(
                corners.to_vec(),
                egui::Color32::from_rgba_unmultiplied(255, 0, 80, v / 2),
                egui::Stroke::NONE,
            ));
        }
    }
}
pub fn properties(ui: &mut Ui, state: &mut DocState) -> bool {
    let Some(id) = state.doc.active_layer else {
        return false;
    };
    let Some(mut settings) = state.doc.layer(id).and_then(|l| l.adjustment) else {
        return false;
    };
    ui.label(settings.kind.name());
    let labels: &[(&str, f32, f32)] = match settings.kind {
        Kind::Vibrance => &[("Vibrance", -100., 100.), ("Saturation", -100., 100.)],
        Kind::BrightnessContrast => &[("Brightness", -150., 150.), ("Contrast", -50., 100.)],
        Kind::Exposure => &[
            ("Exposure", -20., 20.),
            ("Offset", -0.5, 0.5),
            ("Gamma", 0.01, 9.99),
        ],
        Kind::HueSaturation => &[
            ("Hue", -180., 180.),
            ("Saturation", -100., 100.),
            ("Lightness", -100., 100.),
        ],
        Kind::ColorBalance => &[
            ("Shadows Cyan/Red", -100., 100.),
            ("Shadows Magenta/Green", -100., 100.),
            ("Shadows Yellow/Blue", -100., 100.),
            ("Midtones Cyan/Red", -100., 100.),
            ("Midtones Magenta/Green", -100., 100.),
            ("Midtones Yellow/Blue", -100., 100.),
            ("Highlights Cyan/Red", -100., 100.),
            ("Highlights Magenta/Green", -100., 100.),
            ("Highlights Yellow/Blue", -100., 100.),
        ],
        Kind::BlackWhite => &[
            ("Reds", -200., 300.),
            ("Yellows", -200., 300.),
            ("Greens", -200., 300.),
            ("Cyans", -200., 300.),
            ("Blues", -200., 300.),
            ("Magentas", -200., 300.),
        ],
    };
    let mut changed = false;
    for (i, &(name, lo, hi)) in labels.iter().enumerate() {
        changed |= ui
            .add(egui::Slider::new(&mut settings.values[i], lo..=hi).text(name))
            .changed();
    }
    if changed {
        state.doc.layer_mut(id).unwrap().adjustment = Some(settings);
        state.doc.mark_dirty();
        state.record("Adjustment Settings");
    }
    true
}
