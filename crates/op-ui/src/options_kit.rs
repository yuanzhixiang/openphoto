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
}

impl<'a> Bar<'a> {
    pub fn new(ui: &'a mut Ui, bar: Rect) -> Self {
        Self {
            left: bar.left(),
            top: bar.top() - pt(0.5),
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
                ui.add_enabled_ui(enabled, |ui| widgets::checkbox(ui, value, label))
                    .inner
            })
            .inner
    }

    /// A text field from `x0` to `x1`.
    pub fn field(
        &mut self,
        x0: f32,
        x1: f32,
        id: &str,
        text: &mut String,
        enabled: bool,
    ) -> egui::Response {
        let rect = self.rect(x0, FIELD_Y, x1);
        widgets::text_box_inset(self.ui, rect, text, ("options-field", id), enabled, pt(4.5))
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
        let key = self.ui.id().with(("options-value", id));
        let mut text = self
            .ui
            .data(|d| d.get_temp::<String>(key))
            .unwrap_or_else(|| shown.clone());
        let response = self.field(x0, x1, id, &mut text, enabled);
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

    /// The brush preset picker centered at `cx`: the brush's tip (a 14 pt
    /// dot), its size below it and a chevron 20.75 pt to the right.
    pub fn brush_picker(&mut self, cx: f32, size: &str, hardness: f32) -> egui::Response {
        let rect = Rect::from_min_max(self.at(cx - 13.0, 4.0), self.at(cx + 26.0, 31.0));
        let response = self
            .ui
            .interact(rect, self.ui.id().with("brush-picker"), Sense::click());
        let p = self.ui.painter();
        if response.hovered() {
            p.rect_filled(rect, CornerRadius::same(pt(3.0) as u8), color::HOVER);
        }
        let c = self.at(cx, 11.0);
        let _ = hardness;
        p.circle_filled(c, pt(7.0), Color32::from_gray(0xf0));
        p.text(
            self.at(cx, 26.25),
            Align2::CENTER_CENTER,
            size,
            egui::FontId::proportional(pt(10.5)),
            LABEL,
        );
        crate::ps_icons::paint(
            p,
            self.at(cx + 20.75, 18.25),
            Icon::Caret,
            color::OPTIONS_ICON,
            color::OPTIONS_BAR,
        );
        response
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
