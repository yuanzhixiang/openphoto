//! The options bar's controls at Photoshop 2026's places. Every `x` and
//! `y` is in Photoshop points measured on its options bar ("bar points":
//! `x` from the bar's left, `y` from half a point above the bar's top, as
//! the measurements were cropped), so each tool's bar is a list of
//! measured positions.

use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};

use crate::ps_icons::Icon;
use crate::state::SelectionMode;
use crate::theme::{self, color, pt};
use crate::widgets;

/// Label text (`#dddddd`) and disabled text (`#878787`), measured.
pub const LABEL: Color32 = Color32::from_gray(0xdd);
pub const LABEL_OFF: Color32 = Color32::from_gray(0x87);

/// Vertical extents, measured: fields 9–25.5, pop-up menus 8–27, buttons
/// 5.5–29.5, checkboxes 12.5–22.5, framed toggles 4–31.
const FIELD_Y: (f32, f32) = (9.0, 25.5);
/// Fields joined to a chevron box (the font, its size) and disabled
/// fields: 8–26.
const COMBO_Y: (f32, f32) = (8.0, 26.0);
const POPUP_Y: (f32, f32) = (8.0, 27.0);
const BUTTON_Y: (f32, f32) = (5.5, 29.5);
const TOGGLE_Y: (f32, f32) = (4.0, 31.0);
/// The text's vertical center.
const TEXT_Y: f32 = 17.25;

/// One tool's options bar being laid out.
pub struct Bar<'a> {
    pub ui: &'a mut Ui,
    /// The bar's left edge and its measuring origin's top on screen.
    left: f32,
    top: f32,
    /// The bar's width in points.
    width: f32,
}

impl<'a> Bar<'a> {
    pub fn new(ui: &'a mut Ui, bar: Rect) -> Self {
        Self {
            left: bar.left(),
            top: bar.top() - pt(0.5),
            width: bar.width() / pt(1.0),
            ui,
        }
    }

    pub fn at(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.left + pt(x), self.top + pt(y))
    }

    fn rect(&self, x0: f32, (y0, y1): (f32, f32), x1: f32) -> Rect {
        Rect::from_min_max(self.at(x0, y0), self.at(x1, y1))
    }

    /// A 1 pt separator at `x`.
    pub fn sep(&mut self, x: f32) {
        self.ui.painter().rect_filled(
            Rect::from_min_max(self.at(x, 6.5), self.at(x + 1.0, 29.0)),
            0,
            color::OPTIONS_SEPARATOR,
        );
    }

    /// Text starting at `x`.
    pub fn label(&mut self, x: f32, text: &str, enabled: bool) {
        // Source Sans starts half a point further right than Adobe Clean
        self.ui.painter().text(
            self.at(x - 0.75, TEXT_Y),
            Align2::LEFT_CENTER,
            text,
            theme::body(),
            if enabled { LABEL } else { LABEL_OFF },
        );
    }

    /// The bar's width in points.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Text centered on `cx` (the anchor point tools' note).
    pub fn centered_label(&mut self, cx: f32, text: &str) {
        self.ui.painter().text(
            self.at(cx, TEXT_Y),
            Align2::CENTER_CENTER,
            text,
            theme::body(),
            LABEL,
        );
    }

    /// A checkbox whose box starts at `x`, its label 8 pt after the box.
    pub fn check(
        &mut self,
        x: f32,
        label: &str,
        value: &mut bool,
        enabled: bool,
    ) -> egui::Response {
        let rect = Rect::from_min_max(self.at(x, 8.0), self.at(x + 200.0, 27.0));
        self.ui
            .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                // The checkbox draws its own disabled look, unfaded
                ui.visuals_mut().disabled_alpha = 1.0;
                ui.add_enabled_ui(enabled, |ui| widgets::checkbox(ui, value, label))
                    .inner
            })
            .inner
    }

    /// A text field from `x0` to `x1`, `y` its top and bottom.
    fn field_y(
        &mut self,
        x0: f32,
        x1: f32,
        y: (f32, f32),
        id: &str,
        text: &mut String,
        enabled: bool,
    ) -> egui::Response {
        let rect = self.rect(x0, y, x1);
        widgets::text_box_inset(self.ui, rect, text, ("options-field", id), enabled, pt(4.5))
    }

    /// An empty field that can't be used (8–26, like Photoshop's).
    pub fn field_off(&mut self, x0: f32, x1: f32) {
        self.ui.painter().rect(
            self.rect(x0, COMBO_Y, x1),
            CornerRadius::same(pt(2.0) as u8),
            Color32::from_gray(0x4d),
            Stroke::new(pt(1.0), Color32::from_gray(0x5e)),
            StrokeKind::Inside,
        );
    }

    /// A field from `x0` to `x1` joined to a chevron box ending at `x2`
    /// whose menu offers `options` (the font, its style and size, the
    /// stroke width): returns the text typed or the option picked.
    #[allow(clippy::too_many_arguments)]
    pub fn combo(
        &mut self,
        x0: f32,
        x1: f32,
        x2: f32,
        id: &str,
        shown: String,
        options: &[&str],
        enabled: bool,
    ) -> Option<String> {
        let typed = self.value_y(x0, x1, COMBO_Y, id, shown, enabled);
        let rect = self.rect(x1 - 1.0, COMBO_Y, x2);
        let (fill, border, tint) = if enabled {
            (color::FIELD, Color32::from_gray(0x66), color::OPTIONS_ICON)
        } else {
            (
                Color32::from_gray(0x4d),
                Color32::from_gray(0x5e),
                Color32::from_gray(0x6a),
            )
        };
        self.ui.painter().rect(
            rect,
            CornerRadius::same(pt(2.0) as u8),
            fill,
            Stroke::new(pt(1.0), border),
            StrokeKind::Inside,
        );
        crate::ps_icons::paint(
            self.ui.painter(),
            rect.center() + egui::vec2(0.0, pt(0.5)),
            Icon::Caret,
            tint,
            fill,
        );
        if !enabled {
            return typed;
        }
        let response = self.ui.interact(
            rect,
            self.ui.id().with(("options-combo", id)),
            Sense::click(),
        );
        let mut picked = None;
        egui::Popup::menu(&response)
            .id(self.ui.id().with(("options-combo-menu", id)))
            .show(|ui| {
                for o in options {
                    if ui.button(*o).clicked() {
                        picked = Some((*o).to_owned());
                    }
                }
            });
        typed.or(picked)
    }

    /// A color in a frame (the shapes' Fill and Stroke): the frame from
    /// `x0` to `x1` (y 7.5–27.5), the color inset 3 pt; `None` is no
    /// color (white with a red diagonal).
    pub fn framed_swatch(&mut self, x0: f32, x1: f32, fill: Option<Color32>) -> egui::Response {
        let frame = self.rect(x0, (7.5, 27.5), x1);
        let p = self.ui.painter();
        p.rect(
            frame,
            CornerRadius::same(pt(2.0) as u8),
            color::OPTIONS_BAR,
            Stroke::new(pt(1.0), Color32::from_gray(0x66)),
            StrokeKind::Inside,
        );
        let inner = frame.shrink(pt(3.0));
        match fill {
            Some(c) => {
                p.rect_filled(inner, 0, c);
            }
            None => {
                p.rect_filled(inner, 0, Color32::WHITE);
                let mid = inner.center();
                let d = egui::vec2(pt(6.0), -pt(3.0));
                p.line_segment(
                    [mid - d, mid + d],
                    Stroke::new(pt(2.0), Color32::from_rgb(0xe0, 0x30, 0x30)),
                );
            }
        }
        self.ui.interact(
            frame,
            self.ui.id().with(("framed-swatch", x0 as i32)),
            Sense::click(),
        )
    }

    /// A pop-up menu from `x0` to `x1` showing a line style (the stroke's
    /// type), choosing one of `options`.
    pub fn line_popup(&mut self, x0: f32, x1: f32, id: &str, options: &[&str], value: &mut usize) {
        let mut chosen = *value;
        self.popup(x0, x1, id, "", true, |ui| {
            for (i, o) in options.iter().enumerate() {
                ui.selectable_value(&mut chosen, i, *o);
            }
        });
        let dash = match chosen {
            1 => 3.0,
            2 => 1.0,
            _ => 0.0,
        };
        let y = 17.5;
        let (a, b) = (x0 + 7.0, x1 - 18.0);
        let ink = Stroke::new(pt(1.5), LABEL);
        if dash == 0.0 {
            self.ui
                .painter()
                .line_segment([self.at(a, y), self.at(b, y)], ink);
        } else {
            let mut x = a;
            while x < b {
                let e = (x + dash).min(b);
                self.ui
                    .painter()
                    .line_segment([self.at(x, y), self.at(e, y)], ink);
                x += dash * 2.0;
            }
        }
        *value = chosen;
    }

    /// A color box from `x0` to `x1` in a dark 1 pt frame (the type
    /// color, an artboard's background).
    pub fn framed_color(
        &mut self,
        x0: f32,
        x1: f32,
        y: (f32, f32),
        fill: Color32,
    ) -> egui::Response {
        let rect = self.rect(x0, y, x1);
        self.ui.painter().rect(
            rect,
            0,
            fill,
            Stroke::new(pt(1.0), Color32::from_gray(0x36)),
            StrokeKind::Inside,
        );
        self.ui.interact(
            rect,
            self.ui.id().with(("text-color", x0 as i32)),
            Sense::click(),
        )
    }

    /// A field showing `shown` that can be typed in: returns the typed
    /// text once the typing ends (focus leaves or Enter).
    pub fn value(
        &mut self,
        x0: f32,
        x1: f32,
        id: &str,
        shown: String,
        enabled: bool,
    ) -> Option<String> {
        self.value_y(x0, x1, FIELD_Y, id, shown, enabled)
    }

    fn value_y(
        &mut self,
        x0: f32,
        x1: f32,
        y: (f32, f32),
        id: &str,
        shown: String,
        enabled: bool,
    ) -> Option<String> {
        let key = self.ui.id().with(("options-value", id));
        let mut text = self
            .ui
            .data(|d| d.get_temp::<String>(key))
            .unwrap_or_else(|| shown.clone());
        let response = self.field_y(x0, x1, y, id, &mut text, enabled);
        if response.has_focus() {
            self.ui.data_mut(|d| d.insert_temp(key, text));
            None
        } else {
            self.ui.data_mut(|d| d.remove::<String>(key));
            if response.lost_focus() {
                self.ui
                    .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            }
            response.lost_focus().then_some(text)
        }
    }

    /// A pop-up menu from `x0` to `x1` showing `current`; `menu` fills it.
    pub fn popup(
        &mut self,
        x0: f32,
        x1: f32,
        id: &str,
        current: &str,
        enabled: bool,
        menu: impl FnOnce(&mut Ui),
    ) -> egui::Response {
        let rect = self.rect(x0, POPUP_Y, x1);
        self.ui
            .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                widgets::dropdown_with(
                    ui,
                    ("options-popup", id),
                    rect.width(),
                    current,
                    enabled,
                    menu,
                )
            })
            .inner
    }

    /// A pop-up menu choosing one of `options`.
    pub fn choice(
        &mut self,
        x0: f32,
        x1: f32,
        id: &str,
        options: &[&str],
        value: &mut usize,
        enabled: bool,
    ) {
        let current = options[(*value).min(options.len() - 1)];
        let mut chosen = *value;
        self.popup(x0, x1, id, current, enabled, |ui| {
            for (i, o) in options.iter().enumerate() {
                ui.selectable_value(&mut chosen, i, *o);
            }
        });
        *value = chosen;
    }

    /// A push button from `x0` to `x1`.
    pub fn button(&mut self, x0: f32, x1: f32, text: &str, enabled: bool) -> egui::Response {
        self.button_y(x0, x1, BUTTON_Y, text, enabled)
    }

    fn button_y(
        &mut self,
        x0: f32,
        x1: f32,
        y: (f32, f32),
        text: &str,
        enabled: bool,
    ) -> egui::Response {
        let rect = self.rect(x0, y, x1);
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = self
            .ui
            .interact(rect, self.ui.id().with(("options-button", text)), sense);
        // Measured: #454545 in a #666666 frame, disabled #4d4d4d in #5e5e5e
        let (fill, border, ink) = if !enabled {
            (
                Color32::from_gray(0x4d),
                Color32::from_gray(0x5e),
                LABEL_OFF,
            )
        } else if response.hovered() {
            (Color32::from_gray(0x50), Color32::from_gray(0x70), LABEL)
        } else {
            (color::FIELD, Color32::from_gray(0x66), LABEL)
        };
        self.ui.painter().rect(
            rect,
            CornerRadius::same(pt(3.0) as u8),
            fill,
            Stroke::new(pt(1.0), border),
            StrokeKind::Inside,
        );
        self.ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            theme::body(),
            ink,
        );
        response
    }

    /// A framed toggle box from `x0` to `x1` (Photoshop's pressed look:
    /// `#383838` with a light edge), painted when `on`.
    pub fn toggle_frame(&mut self, x0: f32, x1: f32, on: bool) -> Rect {
        let rect = self.rect(x0, TOGGLE_Y, x1);
        if on {
            self.ui.painter().rect(
                rect,
                CornerRadius::same(pt(3.0) as u8),
                color::TOOL_ACTIVE,
                Stroke::new(pt(0.5), Color32::from_gray(0x63)),
                StrokeKind::Inside,
            );
        }
        rect
    }

    /// An icon button centered at `cx`; `on` frames it as pressed.
    pub fn icon(
        &mut self,
        cx: f32,
        icon: Icon,
        tip: &str,
        on: bool,
        enabled: bool,
    ) -> egui::Response {
        let rect = self.toggle_frame(cx - 13.0, cx + 13.0, on);
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = self
            .ui
            .interact(rect, self.ui.id().with(("options-icon", tip)), sense);
        if !on && enabled && response.hovered() {
            self.ui
                .painter()
                .rect_filled(rect, CornerRadius::same(pt(3.0) as u8), color::HOVER);
        }
        let tint = if enabled {
            color::OPTIONS_ICON
        } else {
            color::OPTIONS_ICON_DISABLED
        };
        let bg = if on {
            color::TOOL_ACTIVE
        } else {
            color::OPTIONS_BAR
        };
        crate::ps_icons::paint(self.ui.painter(), rect.center(), icon, tint, bg);
        response.on_hover_text(tip)
    }

    /// The brush preset picker centered at `cx`: the brush's tip as
    /// Photoshop previews it (its size in device pixels up to 28, soft
    /// brushes fading out; centered 11 pt down), the size below it (y
    /// 26.25) and a chevron 21 pt to the right.
    pub fn brush_picker(
        &mut self,
        cx: f32,
        label: &str,
        size: f32,
        hardness: f32,
    ) -> egui::Response {
        let rect = Rect::from_min_max(self.at(cx - 13.0, 4.0), self.at(cx + 27.0, 31.0));
        let response = self
            .ui
            .interact(rect, self.ui.id().with("brush-picker"), Sense::click());
        let p = self.ui.painter();
        if response.hovered() {
            p.rect_filled(rect, CornerRadius::same(pt(3.0) as u8), color::HOVER);
        }
        let c = self.at(cx, 11.0);
        let ppp = self.ui.ctx().pixels_per_point();
        let radius = size.clamp(1.0, 28.0) / 2.0 / ppp;
        let hard = hardness.clamp(0.0, 1.0);
        // From the edge in: each ring a little more opaque
        let rings = 12;
        for k in 0..rings {
            let t = 1.0 - k as f32 / rings as f32;
            let r = radius * t;
            let alpha = if t <= hard.max(0.02) {
                1.0
            } else {
                ((1.0 - t) / (1.0 - hard.max(0.02))).clamp(0.0, 1.0)
            };
            p.circle_filled(
                c,
                r,
                Color32::from_gray(0xf0).gamma_multiply(alpha.powf(1.5)),
            );
        }
        p.text(
            self.at(cx, 26.25),
            Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(pt(10.5)),
            LABEL,
        );
        crate::ps_icons::paint(
            p,
            self.at(cx + 21.0, 18.25),
            Icon::Caret,
            color::OPTIONS_ICON,
            color::OPTIONS_BAR,
        );
        response
    }

    /// The chevron box after a percentage field (`x0` to `x1`, sharing its
    /// edge); a click opens a slider. Returns the slider's new value.
    pub fn slider_box(&mut self, x0: f32, x1: f32, id: &str, current: f32) -> Option<f32> {
        let rect = self.rect(x0, FIELD_Y, x1);
        let response =
            self.ui
                .interact(rect, self.ui.id().with(("slider-box", id)), Sense::click());
        let fill = if response.hovered() {
            Color32::from_gray(0x4f)
        } else {
            color::FIELD
        };
        self.ui.painter().rect(
            rect,
            CornerRadius::same(pt(2.0) as u8),
            fill,
            Stroke::new(pt(1.0), Color32::from_gray(0x66)),
            StrokeKind::Inside,
        );
        crate::ps_icons::paint(
            self.ui.painter(),
            rect.center() + egui::vec2(0.0, pt(0.5)),
            Icon::Caret,
            color::OPTIONS_ICON,
            fill,
        );
        let mut value = current;
        let mut changed = false;
        egui::Popup::from_response(&response)
            .open_memory(response.clicked().then_some(egui::SetOpenCommand::Toggle))
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                changed = ui
                    .add(
                        egui::Slider::new(&mut value, 0.0..=100.0)
                            .max_decimals(0)
                            .suffix("%"),
                    )
                    .changed();
            });
        changed.then_some(value.round())
    }

    /// A swatch from `x0` to `x1` and its chevron: right after it (the
    /// Mixer Brush's load color) or in a box (`chevron_box`, the Pattern
    /// Stamp's pattern).
    pub fn swatch(&mut self, x0: f32, x1: f32, fill: Color32, chevron_box: Option<(f32, f32)>) {
        let rect = self.rect(x0, (5.0, 30.0), x1);
        self.ui.painter().rect_filled(rect, 0, fill);
        if chevron_box.is_some() {
            // The default pattern: leaves of green on dark green
            let leaf = Color32::from_rgb(0x3f, 0x7a, 0x3a);
            for k in 0..24 {
                let (i, j) = (k % 6, k / 6);
                let (x, y) = (
                    x0 + 2.5 + 4.5 * i as f32 + (j % 2) as f32 * 2.0,
                    7.5 + 5.5 * j as f32,
                );
                self.ui
                    .painter()
                    .circle_filled(self.at(x, y), pt(1.3), leaf);
            }
            self.ui.painter().rect_stroke(
                rect,
                0,
                Stroke::new(pt(1.0), Color32::from_gray(0x66)),
                StrokeKind::Inside,
            );
        }
        let cx = match chevron_box {
            Some((b0, b1)) => {
                self.ui.painter().rect(
                    self.rect(b0, (5.0, 30.0), b1),
                    CornerRadius::same(pt(2.0) as u8),
                    color::FIELD,
                    Stroke::new(pt(1.0), Color32::from_gray(0x66)),
                    StrokeKind::Inside,
                );
                (b0 + b1) / 2.0
            }
            None => x1 + 4.0,
        };
        crate::ps_icons::paint(
            self.ui.painter(),
            self.at(cx, 18.0),
            Icon::Caret,
            color::OPTIONS_ICON,
            color::OPTIONS_BAR,
        );
    }

    /// Segmented buttons: one box split at `edges` (its left edge first and
    /// its right edge last) with a label per segment, the `chosen` one
    /// pressed. Returns the segment clicked.
    pub fn segmented(
        &mut self,
        edges: &[f32],
        labels: &[&str],
        chosen: usize,
        enabled: bool,
    ) -> Option<usize> {
        let mut clicked = None;
        let whole = self.rect(edges[0], (5.0, 30.0), edges[edges.len() - 1]);
        self.ui.painter().rect(
            whole,
            CornerRadius::same(pt(3.0) as u8),
            color::FIELD,
            Stroke::new(pt(1.0), Color32::from_gray(0x66)),
            StrokeKind::Inside,
        );
        for (k, w) in edges.windows(2).enumerate() {
            let rect = self.rect(w[0], (5.0, 30.0), w[1]);
            if k > 0 {
                self.ui.painter().rect_filled(
                    Rect::from_min_max(self.at(w[0], 5.0), self.at(w[0] + 1.0, 30.0)),
                    0,
                    Color32::from_gray(0x66),
                );
            }
            if k == chosen {
                self.ui.painter().rect_filled(
                    rect.shrink(pt(1.0)),
                    CornerRadius::same(pt(2.0) as u8),
                    color::TOOL_ACTIVE,
                );
            }
            let sense = if enabled {
                Sense::click()
            } else {
                Sense::hover()
            };
            if self
                .ui
                .interact(rect, self.ui.id().with(("segment", labels[k])), sense)
                .clicked()
            {
                clicked = Some(k);
            }
            self.ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                labels[k],
                theme::body(),
                if enabled { LABEL } else { LABEL_OFF },
            );
        }
        clicked
    }

    /// The gradient swatch: `a` to `b` from `x0` to `x1` (y 5–30), its
    /// chevron box to `x2`.
    /// The gradient picker: the gradient (over a checkerboard where it is
    /// see-through) and its chevron. Returns the swatch's and the
    /// chevron's responses (the Gradient Editor; the presets).
    pub fn gradient_swatch(
        &mut self,
        x0: f32,
        x1: f32,
        x2: f32,
        gradient: &op_core::gradient::Gradient,
    ) -> (egui::Response, egui::Response) {
        let rect = self.rect(x0, (5.0, 30.0), x1);
        crate::widgets::checkerboard(self.ui.painter(), rect, pt(4.0));
        crate::dialogs::gradient_editor::paint_gradient(self.ui.painter(), rect, gradient);
        let swatch = self.ui.interact(
            rect,
            self.ui.id().with(("gradient-swatch", x0 as i32)),
            Sense::click(),
        );
        self.ui.painter().rect_stroke(
            rect,
            0,
            Stroke::new(pt(1.0), Color32::from_gray(0x66)),
            StrokeKind::Inside,
        );
        let chevron = self.rect(x1 - 1.0, (5.0, 30.0), x2);
        self.ui.painter().rect(
            chevron,
            CornerRadius::same(pt(2.0) as u8),
            color::FIELD,
            Stroke::new(pt(1.0), Color32::from_gray(0x66)),
            StrokeKind::Inside,
        );
        crate::ps_icons::paint(
            self.ui.painter(),
            chevron.center() + egui::vec2(0.0, pt(0.5)),
            Icon::Caret,
            color::OPTIONS_ICON,
            color::FIELD,
        );
        let menu = self.ui.interact(
            chevron,
            self.ui.id().with(("gradient-chevron", x0 as i32)),
            Sense::click(),
        );
        (swatch, menu)
    }

    /// A plain color box from `x0` to `x1` (y 5–30), unframed.
    pub fn color_box(&mut self, x0: f32, x1: f32, fill: Color32) -> egui::Response {
        let rect = self.rect(x0, (5.0, 30.0), x1);
        self.ui.painter().rect_filled(rect, 0, fill);
        self.ui.interact(
            rect,
            self.ui.id().with(("color-box", x0 as i32)),
            Sense::click(),
        )
    }

    /// An empty, unusable pattern box (x0–x1) with its chevron box (to x2).
    pub fn empty_pattern(&mut self, x0: f32, x1: f32, x2: f32) {
        for (a, b) in [(x0, x1), (x1 - 1.0, x2)] {
            self.ui.painter().rect(
                self.rect(a, (5.0, 30.0), b),
                CornerRadius::same(pt(2.0) as u8),
                Color32::from_gray(0x4d),
                Stroke::new(pt(1.0), Color32::from_gray(0x5e)),
                StrokeKind::Inside,
            );
        }
        crate::ps_icons::paint(
            self.ui.painter(),
            self.at((x1 + x2) / 2.0 - 0.5, 18.0),
            Icon::Caret,
            Color32::from_gray(0x6a),
            Color32::from_gray(0x4d),
        );
    }

    /// A pressed-look box around `x0`–`x1` with a small triangle at its
    /// bottom right (a button with a menu, e.g. Select people), its text
    /// centered.
    pub fn menu_button(&mut self, x0: f32, x1: f32, text: &str) -> egui::Response {
        let r = self.button_y(x0, x1, (4.5, 30.5), text, true);
        let p = self.ui.painter();
        let br = self.at(x1 - 3.0, 27.0);
        p.add(egui::Shape::convex_polygon(
            vec![
                br,
                br + egui::vec2(-pt(3.5), 0.0),
                br + egui::vec2(0.0, -pt(3.5)),
            ],
            LABEL,
            Stroke::NONE,
        ));
        r
    }

    /// A small square button from `x0` to `x1` with a chevron (the menu
    /// next to Select Subject).
    pub fn chevron_button(&mut self, x0: f32, x1: f32, id: &str, enabled: bool) -> egui::Response {
        let r = self.button_y(x0, x1, (5.0, 30.0), "", enabled);
        let _ = id;
        let tint = if enabled {
            color::OPTIONS_ICON
        } else {
            color::OPTIONS_ICON_DISABLED
        };
        // A 12.5 pt chevron in a 1 pt line, measured
        let cx = (x0 + x1) / 2.0;
        let pts = vec![
            self.at(cx - 6.25, 14.25),
            self.at(cx, 20.25),
            self.at(cx + 6.25, 14.25),
        ];
        self.ui
            .painter()
            .add(egui::Shape::line(pts, Stroke::new(pt(1.1), tint)));
        r
    }

    /// The selection tools' four combine modes: New, Add, Subtract,
    /// Intersect, 26 pt apart from the first box's left at `x`.
    pub fn modes(&mut self, x: f32, current: &mut SelectionMode) {
        let modes = [
            (SelectionMode::New, "New selection"),
            (SelectionMode::Add, "Add to selection"),
            (SelectionMode::Subtract, "Subtract from selection"),
            (SelectionMode::Intersect, "Intersect with selection"),
        ];
        for (k, (mode, tip)) in modes.into_iter().enumerate() {
            let x0 = x + 26.0 * k as f32;
            let on = *current == mode;
            let rect = self.toggle_frame(x0, x0 + 26.0, on);
            let response = self.ui.interact(
                rect,
                self.ui.id().with(("selection-mode", k)),
                Sense::click(),
            );
            if !on && response.hovered() {
                self.ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(pt(3.0) as u8),
                    color::HOVER,
                );
            }
            let bg = if on {
                color::TOOL_ACTIVE
            } else if response.hovered() {
                color::HOVER
            } else {
                color::OPTIONS_BAR
            };
            mode_icon(self, x0 + 13.5, mode, bg);
            if response.on_hover_text(tip).clicked() {
                *current = mode;
            }
        }
    }
}

/// A selection mode's icon centered at (`cx`, 17.5), measured: New a
/// filled 10 pt square; Add two overlapping filled squares; Subtract the
/// first square less the second, outlined; Intersect both outlined with
/// their overlap filled.
fn mode_icon(bar: &Bar, cx: f32, mode: SelectionMode, bg: Color32) {
    let p = bar.ui.painter();
    let r = |x0: f32, y0: f32, x1: f32, y1: f32| {
        Rect::from_min_max(bar.at(cx + x0, 17.5 + y0), bar.at(cx + x1, 17.5 + y1))
    };
    let ink = color::OPTIONS_ICON;
    let ring = |rect: Rect| {
        p.rect_stroke(rect, 0, Stroke::new(pt(1.0), ink), StrokeKind::Inside);
    };
    match mode {
        SelectionMode::New => {
            p.rect_filled(r(-5.0, -5.0, 5.0, 5.0), pt(0.5), ink);
        }
        SelectionMode::Add => {
            p.rect_filled(r(-7.0, -7.0, 3.0, 3.0), 0, ink);
            p.rect_filled(r(-3.0, -3.0, 7.0, 7.0), 0, ink);
        }
        SelectionMode::Subtract => {
            p.rect_filled(r(-7.0, -6.5, 2.5, 3.0), 0, ink);
            p.rect_filled(r(-2.5, -3.0, 7.0, 6.5), 0, bg);
            ring(r(-2.5, -3.0, 7.0, 6.5));
        }
        SelectionMode::Intersect => {
            ring(r(-7.0, -6.5, 2.5, 3.0));
            ring(r(-2.5, -3.0, 7.0, 6.5));
            p.rect_filled(r(-2.5, -3.0, 2.5, 3.0), 0, ink);
        }
    }
}
