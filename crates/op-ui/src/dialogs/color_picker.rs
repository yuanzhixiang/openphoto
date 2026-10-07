//! Photoshop's Color Picker dialog. Every size is measured in Photoshop 2026
//! at 1:1, in points, relative to the dialog's top-left corner (title bar
//! included).

use egui::{
    Align, Align2, Color32, ColorImage, FontId, Key, Pos2, Rect, Sense, Shape, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Ui, UiBuilder, Vec2, pos2, vec2,
};
use op_color::{Cmyk, Hsb, Lab, from_hex, is_web_safe, to_hex, web_safe};
use op_core::Color;

use super::common;
use crate::theme::{self, color, pt};

const SIZE: Vec2 = vec2(pt(534.0), pt(374.0));
const FIELD: Rect = Rect::from_min_max(pos2(pt(10.0), pt(60.0)), pos2(pt(266.0), pt(316.0)));
const SLIDER: Rect = Rect::from_min_max(pos2(pt(278.0), pt(60.0)), pos2(pt(298.0), pt(316.0)));
const SWATCH: Rect = Rect::from_min_max(pos2(pt(310.0), pt(77.0)), pos2(pt(370.0), pt(145.0)));
const BUTTON_X: f32 = pt(408.0);
const BUTTON_SIZE: Vec2 = vec2(pt(115.0), pt(25.0));
const FIELD_H: f32 = pt(20.0);
/// Photoshop's Color Picker is a classic AppKit dialog: 12 pt system
/// text, 13 pt buttons.
const FONT: f32 = pt(12.0);
/// Resolution of the generated field and slider images.
const TEXTURE_PX: usize = 256;

const INPUT_FILL: Color32 = Color32::from_gray(0x48);
const INPUT_BORDER: Color32 = Color32::from_gray(0x66);
const FOCUS: Color32 = Color32::from_rgb(0x14, 0x73, 0xe6);

/// The nine channels the picker can be driven by (the radio buttons).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Hue,
    Saturation,
    Brightness,
    Red,
    Green,
    Blue,
    L,
    A,
    B,
}

impl Channel {
    /// Value range as shown in the field next to the radio button.
    fn range(self) -> (f32, f32) {
        match self {
            Self::Hue => (0.0, 360.0),
            Self::Saturation | Self::Brightness | Self::L => (0.0, 100.0),
            Self::Red | Self::Green | Self::Blue => (0.0, 255.0),
            Self::A | Self::B => (-128.0, 127.0),
        }
    }

    /// The (x, y) channels of the large field when this channel drives the
    /// slider. Matches Photoshop's arrangement.
    fn field_axes(self) -> (Channel, Channel) {
        match self {
            Self::Hue => (Self::Saturation, Self::Brightness),
            Self::Saturation => (Self::Hue, Self::Brightness),
            Self::Brightness => (Self::Hue, Self::Saturation),
            Self::Red => (Self::Blue, Self::Green),
            Self::Green => (Self::Blue, Self::Red),
            Self::Blue => (Self::Red, Self::Green),
            Self::L => (Self::A, Self::B),
            Self::A => (Self::B, Self::L),
            Self::B => (Self::A, Self::L),
        }
    }
}

/// The color being edited, kept in HSB so the hue survives grays and blacks.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Model {
    hsb: Hsb,
}

impl Model {
    fn new(c: Color) -> Self {
        Self {
            hsb: Hsb::from_color(c),
        }
    }

    fn color(&self) -> Color {
        self.hsb.to_color()
    }

    /// Value of `ch` in its display range.
    fn get(&self, ch: Channel) -> f32 {
        let c = self.color();
        match ch {
            Channel::Hue => self.hsb.h,
            Channel::Saturation => self.hsb.s * 100.0,
            Channel::Brightness => self.hsb.b * 100.0,
            Channel::Red => c.r * 255.0,
            Channel::Green => c.g * 255.0,
            Channel::Blue => c.b * 255.0,
            Channel::L => Lab::from_color(c).l,
            Channel::A => Lab::from_color(c).a,
            Channel::B => Lab::from_color(c).b,
        }
    }

    /// Sets `ch` (in its display range, clamped).
    fn set(&mut self, ch: Channel, v: f32) {
        let (lo, hi) = ch.range();
        let v = v.clamp(lo, hi);
        match ch {
            Channel::Hue => self.hsb.h = v,
            Channel::Saturation => self.hsb.s = v / 100.0,
            Channel::Brightness => self.hsb.b = v / 100.0,
            Channel::Red | Channel::Green | Channel::Blue => {
                let mut c = self.color();
                match ch {
                    Channel::Red => c.r = v / 255.0,
                    Channel::Green => c.g = v / 255.0,
                    _ => c.b = v / 255.0,
                }
                self.set_color(c);
            }
            Channel::L | Channel::A | Channel::B => {
                let mut lab = Lab::from_color(self.color());
                match ch {
                    Channel::L => lab.l = v,
                    Channel::A => lab.a = v,
                    _ => lab.b = v,
                }
                self.set_color(lab.to_color());
            }
        }
    }

    /// Replaces the color, keeping the current hue (and saturation) where the
    /// new color leaves them undefined.
    fn set_color(&mut self, c: Color) {
        let mut hsb = Hsb::from_color(c);
        if hsb.b == 0.0 {
            hsb.s = self.hsb.s;
        }
        if hsb.s == 0.0 || hsb.b == 0.0 {
            hsb.h = self.hsb.h;
        }
        self.hsb = hsb;
    }

    /// A copy with `ch` set; used to render the field and slider.
    fn with(&self, ch: Channel, v: f32) -> Self {
        let mut m = *self;
        m.set(ch, v);
        m
    }
}

/// The fields beside the radio buttons, plus CMYK and hex.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Input {
    Channel(Channel),
    C,
    M,
    Y,
    K,
    Hex,
}

pub enum Outcome {
    Open,
    Cancel,
    Ok(Color),
}

pub struct ColorPicker {
    title: String,
    /// The color the dialog was opened with ("current").
    original: Color,
    model: Model,
    mode: Channel,
    web_only: bool,
    /// Text of the input being edited, so partial input isn't overwritten.
    editing: Option<(Input, String)>,
    field_texture: Option<(TextureKey, TextureHandle)>,
    slider_texture: Option<(TextureKey, TextureHandle)>,
    /// Colors chosen with "Add to Swatches", collected by the host.
    added_swatches: Vec<Color>,
    /// Until the first frame is drawn: the hex field then takes the focus
    /// with its text selected, as in Photoshop.
    first_frame: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct TextureKey {
    mode: Channel,
    model: [u32; 3],
    web_only: bool,
}

impl ColorPicker {
    /// `title` is the window title, e.g. "Color Picker (Foreground Color)".
    pub fn new(title: impl Into<String>, color: Color) -> Self {
        Self {
            title: title.into(),
            original: color,
            model: Model::new(color),
            mode: Channel::Hue,
            web_only: false,
            editing: None,
            field_texture: None,
            slider_texture: None,
            added_swatches: Vec::new(),
            first_frame: true,
        }
    }

    /// Colors added with "Add to Swatches" since the last call.
    pub fn take_added_swatches(&mut self) -> Vec<Color> {
        std::mem::take(&mut self.added_swatches)
    }

    fn color(&self) -> Color {
        self.model.color()
    }

    fn set_model(&mut self, model: Model) {
        self.model = model;
        if self.web_only {
            self.model.set_color(web_safe(self.model.color()));
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("color-picker"))
            .frame(egui::Frame::NONE)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
                outcome = self.ui(ui, rect);
            });
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    fn ui(&mut self, ui: &mut Ui, frame: Rect) -> Outcome {
        let at = |r: Rect| r.translate(frame.min.to_vec2());
        let font = theme::dialog(FONT);
        common::frame(ui, frame, &self.title, theme::dialog_bold(pt(13.0)));

        self.field(ui, at(FIELD));
        self.slider(ui, at(SLIDER));
        self.swatch(ui, at(SWATCH), &font);

        // Inputs: HSB and RGB on the left, Lab and CMYK on the right
        let left = [
            (Channel::Hue, "H:", Some("°"), 209.0),
            (Channel::Saturation, "S:", Some("%"), 232.0),
            (Channel::Brightness, "B:", Some("%"), 255.0),
            (Channel::Red, "R:", None, 281.5),
            (Channel::Green, "G:", None, 304.5),
            (Channel::Blue, "B:", None, 327.5),
        ];
        for (ch, label, unit, y) in left {
            self.radio_row(
                ui,
                frame,
                ch,
                label,
                pt(316.0),
                pt(330.0),
                pt(355.0),
                pt(35.0),
                pt(y),
                &font,
            );
            if let Some(unit) = unit {
                ui.painter().text(
                    frame.min + vec2(pt(397.0), pt(y)),
                    Align2::LEFT_CENTER,
                    unit,
                    font.clone(),
                    color::TEXT,
                );
            }
        }
        for (ch, label, y) in [
            (Channel::L, "L:", 209.0),
            (Channel::A, "a:", 232.0),
            (Channel::B, "b:", 255.0),
        ] {
            self.radio_row(
                ui,
                frame,
                ch,
                label,
                pt(429.0),
                pt(442.0),
                pt(466.0),
                pt(43.0),
                pt(y),
                &font,
            );
        }
        for (input, label, y) in [
            (Input::C, "C:", 281.5),
            (Input::M, "M:", 304.5),
            (Input::Y, "Y:", 327.5),
            (Input::K, "K:", 351.0),
        ] {
            ui.painter().text(
                frame.min + vec2(pt(462.0), pt(y)),
                Align2::RIGHT_CENTER,
                label,
                font.clone(),
                color::TEXT,
            );
            self.input(
                ui,
                input,
                Rect::from_min_size(
                    frame.min + vec2(pt(466.0), pt(y) - FIELD_H / 2.0),
                    vec2(pt(34.0), FIELD_H),
                ),
            );
            ui.painter().text(
                frame.min + vec2(pt(506.0), pt(y)),
                Align2::LEFT_CENTER,
                "%",
                font.clone(),
                color::TEXT,
            );
        }
        ui.painter().text(
            frame.min + vec2(pt(314.0), pt(354.0)),
            Align2::CENTER_CENTER,
            "#",
            font.clone(),
            color::TEXT,
        );
        self.input(
            ui,
            Input::Hex,
            Rect::from_min_size(
                frame.min + vec2(pt(322.0), pt(354.0) - FIELD_H / 2.0),
                vec2(pt(83.0), FIELD_H),
            ),
        );

        // Only Web Colors
        let check =
            Rect::from_min_size(frame.min + vec2(pt(10.0), pt(330.0)), Vec2::splat(pt(13.0)));
        let check_hit = check.union(Rect::from_min_size(
            check.right_top(),
            vec2(pt(110.0), check.height()),
        ));
        let r = ui.interact(check_hit, ui.id().with("web-only"), Sense::click());
        if r.clicked() {
            self.web_only = !self.web_only;
            let m = self.model;
            self.set_model(m);
        }
        checkbox(ui.painter(), check, self.web_only);
        ui.painter().text(
            frame.min + vec2(pt(32.0), pt(336.5)),
            Align2::LEFT_CENTER,
            "Only Web Colors",
            font.clone(),
            color::TEXT,
        );

        // Buttons
        let button = |y: f32| Rect::from_min_size(frame.min + vec2(BUTTON_X, pt(y)), BUTTON_SIZE);
        let button_font = theme::dialog(pt(13.0));
        let ok = common::pill_button(ui, button(39.0), "OK", button_font.clone(), true);
        let cancel = common::pill_button(ui, button(73.0), "Cancel", button_font.clone(), true);
        let add = common::pill_button(
            ui,
            button(116.0),
            "Add to Swatches",
            button_font.clone(),
            true,
        );
        common::pill_button(ui, button(158.0), "Color Libraries", button_font, false);

        if add.clicked() {
            self.added_swatches.push(self.color());
        }
        if cancel.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if ok.clicked() || enter {
            return Outcome::Ok(self.color());
        }
        Outcome::Open
    }

    /// The large field: the two channels not driven by the slider.
    fn field(&mut self, ui: &mut Ui, rect: Rect) {
        let (xc, yc) = self.mode.field_axes();
        let response = ui.interact(rect, ui.id().with("field"), Sense::click_and_drag());
        if let Some(p) = response
            .interact_pointer_pos()
            .filter(|_| response.is_pointer_button_down_on())
        {
            let (fx, fy) = (
                (p.x - rect.left()) / rect.width(),
                (p.y - rect.top()) / rect.height(),
            );
            let mut m = self.model;
            m.set(xc, lerp(xc.range(), fx.clamp(0.0, 1.0)));
            m.set(yc, lerp(yc.range(), 1.0 - fy.clamp(0.0, 1.0)));
            self.set_model(m);
            self.editing = None;
        }

        let key = self.texture_key();
        if self.field_texture.as_ref().is_none_or(|(k, _)| *k != key) {
            let image = self.render(
                |m, x, y| {
                    m.with(xc, lerp(xc.range(), x))
                        .with(yc, lerp(yc.range(), 1.0 - y))
                },
                TEXTURE_PX,
            );
            let tex = ui
                .ctx()
                .load_texture("color-picker-field", image, TextureOptions::LINEAR);
            self.field_texture = Some((key, tex));
        }
        let tex = &self.field_texture.as_ref().unwrap().1;
        let painter = ui.painter_at(rect);
        painter.image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        // Marker: a ring, dark on light colors and light on dark ones
        let pos = pos2(
            rect.left() + unlerp(xc.range(), self.model.get(xc)) * rect.width(),
            rect.top() + (1.0 - unlerp(yc.range(), self.model.get(yc))) * rect.height(),
        );
        let ring = if Lab::from_color(self.color()).l > 60.0 {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        // Clipped to the field, as in Photoshop (a quarter at a corner)
        ui.painter_at(rect)
            .circle_stroke(pos, pt(5.0), Stroke::new(1.0, ring));
        ui.painter().rect_stroke(
            rect,
            0,
            Stroke::new(1.0, Color32::from_gray(0x3c)),
            StrokeKind::Outside,
        );
    }

    /// The vertical slider for the selected channel, with arrows on both sides.
    fn slider(&mut self, ui: &mut Ui, rect: Rect) {
        let ch = self.mode;
        let hit = rect.expand2(vec2(pt(10.0), pt(4.0)));
        let response = ui.interact(hit, ui.id().with("slider"), Sense::click_and_drag());
        if let Some(p) = response
            .interact_pointer_pos()
            .filter(|_| response.is_pointer_button_down_on())
        {
            let f = 1.0 - ((p.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            let mut m = self.model;
            m.set(ch, lerp(ch.range(), f));
            self.set_model(m);
            self.editing = None;
        }

        let key = self.texture_key();
        if self.slider_texture.as_ref().is_none_or(|(k, _)| *k != key) {
            // In hue mode Photoshop shows pure hues, independent of S and B
            let image = self.render(
                |m, _x, y| {
                    let base = if ch == Channel::Hue {
                        Model {
                            hsb: Hsb {
                                h: m.hsb.h,
                                s: 1.0,
                                b: 1.0,
                            },
                        }
                    } else {
                        *m
                    };
                    base.with(ch, lerp(ch.range(), 1.0 - y))
                },
                1,
            );
            let tex = ui
                .ctx()
                .load_texture("color-picker-slider", image, TextureOptions::LINEAR);
            self.slider_texture = Some((key, tex));
        }
        let tex = &self.slider_texture.as_ref().unwrap().1;
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.painter().rect_stroke(
            rect,
            0,
            Stroke::new(1.0, Color32::from_gray(0x3c)),
            StrokeKind::Outside,
        );

        let y = rect.top() + (1.0 - unlerp(ch.range(), self.model.get(ch))) * rect.height();
        // Both arrows point right, as in Photoshop: one into the slider from
        // the left, one away from it on the right
        arrow(ui.painter(), pos2(rect.left() - pt(2.0), y));
        arrow(ui.painter(), pos2(rect.right() + pt(9.0), y));
    }

    /// "new" over "current", the web-safe warning and its swatch.
    fn swatch(&mut self, ui: &mut Ui, rect: Rect, font: &FontId) {
        let painter = ui.painter().clone();
        let half = rect.height() / 2.0;
        let new = Rect::from_min_size(rect.min, vec2(rect.width(), half));
        let current = Rect::from_min_size(rect.min + vec2(0.0, half), vec2(rect.width(), half));
        painter.rect_filled(new, 0, to_c32(self.color()));
        painter.rect_filled(current, 0, to_c32(self.original));
        painter.rect_stroke(
            rect,
            0,
            Stroke::new(1.0, Color32::from_gray(0x2e)),
            StrokeKind::Outside,
        );
        painter.text(
            rect.center_top() - vec2(0.0, pt(9.0)),
            Align2::CENTER_CENTER,
            "new",
            font.clone(),
            color::TEXT,
        );
        painter.text(
            rect.center_bottom() + vec2(0.0, pt(10.0)),
            Align2::CENTER_CENTER,
            "current",
            font.clone(),
            color::TEXT,
        );

        // Clicking "current" restores the original color
        if ui
            .interact(current, ui.id().with("current"), Sense::click())
            .clicked()
        {
            self.set_model(Model::new(self.original));
            self.editing = None;
        }

        // Not web-safe: a cube icon and the nearest web-safe color, which can
        // be clicked to snap to it
        if !is_web_safe(self.color()) {
            let cube = pos2(rect.right() + pt(15.0), rect.top() + pt(15.0));
            painter.text(
                cube,
                Align2::CENTER_CENTER,
                crate::icons::CUBE,
                theme::icon(pt(13.0)),
                color::TEXT,
            );
            let safe =
                Rect::from_center_size(pos2(cube.x, rect.top() + pt(31.0)), Vec2::splat(pt(12.0)));
            painter.rect(
                safe,
                0,
                to_c32(web_safe(self.color())),
                Stroke::new(1.0, Color32::from_gray(0xd0)),
                StrokeKind::Inside,
            );
            let hit = Rect::from_min_max(pos2(safe.left(), cube.y - pt(8.0)), safe.max);
            let r = ui
                .interact(hit, ui.id().with("web-safe"), Sense::click())
                .on_hover_text("Click to select web safe color");
            if r.clicked() {
                let mut m = self.model;
                m.set_color(web_safe(self.color()));
                self.set_model(m);
                self.editing = None;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn radio_row(
        &mut self,
        ui: &mut Ui,
        frame: Rect,
        ch: Channel,
        label: &str,
        radio_x: f32,
        label_x: f32,
        field_x: f32,
        field_w: f32,
        y: f32,
        font: &FontId,
    ) {
        let center = frame.min + vec2(radio_x, y);
        let hit = Rect::from_center_size(center, Vec2::splat(pt(16.0))).union(Rect::from_min_size(
            frame.min + vec2(label_x, y - pt(8.0)),
            vec2(field_x - label_x, pt(16.0)),
        ));
        if ui
            .interact(hit, ui.id().with(("radio", ch)), Sense::click())
            .clicked()
        {
            self.mode = ch;
        }
        radio(ui.painter(), center, self.mode == ch);
        ui.painter().text(
            frame.min + vec2(label_x, y),
            Align2::LEFT_CENTER,
            label,
            font.clone(),
            color::TEXT,
        );
        self.input(
            ui,
            Input::Channel(ch),
            Rect::from_min_size(
                frame.min + vec2(field_x, y - FIELD_H / 2.0),
                vec2(field_w, FIELD_H),
            ),
        );
    }

    fn input_text(&self, input: Input) -> String {
        let c = self.color();
        match input {
            Input::Channel(ch) => format!("{:.0}", self.model.get(ch)),
            Input::Hex => to_hex(c),
            _ => {
                let cmyk = Cmyk::from_color(c);
                let v = match input {
                    Input::C => cmyk.c,
                    Input::M => cmyk.m,
                    Input::Y => cmyk.y,
                    _ => cmyk.k,
                };
                format!("{:.0}", v * 100.0)
            }
        }
    }

    /// Applies typed text; invalid or partial input is ignored.
    fn apply_input(&mut self, input: Input, text: &str) {
        let mut m = self.model;
        match input {
            Input::Hex => match from_hex(text) {
                Some(c) => m.set_color(c),
                None => return,
            },
            Input::Channel(ch) => match text.trim().parse::<f32>() {
                Ok(v) => m.set(ch, v),
                Err(_) => return,
            },
            _ => {
                let Ok(v) = text.trim().parse::<f32>() else {
                    return;
                };
                let mut cmyk = Cmyk::from_color(m.color());
                let v = (v / 100.0).clamp(0.0, 1.0);
                match input {
                    Input::C => cmyk.c = v,
                    Input::M => cmyk.m = v,
                    Input::Y => cmyk.y = v,
                    _ => cmyk.k = v,
                }
                m.set_color(cmyk.to_color());
            }
        }
        self.set_model(m);
    }

    fn input(&mut self, ui: &mut Ui, input: Input, rect: Rect) {
        let id = ui.id().with(("input", input));
        let focused = ui.memory(|m| m.has_focus(id));
        let mut text = match &self.editing {
            Some((i, t)) if *i == input && focused => t.clone(),
            _ => self.input_text(input),
        };

        ui.painter().rect(
            rect,
            0,
            INPUT_FILL,
            Stroke::new(1.0, INPUT_BORDER),
            StrokeKind::Inside,
        );
        if focused {
            ui.painter()
                .rect_stroke(rect, 1, Stroke::new(2.0, FOCUS), StrokeKind::Outside);
        }
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect.shrink2(vec2(pt(4.0), 0.0))));
        let response = egui::TextEdit::singleline(&mut text)
            .id(id)
            .frame(egui::Frame::NONE)
            .font(theme::dialog(FONT))
            .vertical_align(Align::Center)
            .desired_width(rect.width() - pt(8.0))
            .min_size(vec2(0.0, rect.height()))
            .show(&mut child)
            .response;

        if input == Input::Hex && self.first_frame {
            self.first_frame = false;
            response.request_focus();
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
                let all = egui::text::CCursorRange::two(
                    egui::text::CCursor::new(0),
                    egui::text::CCursor::new(text.chars().count()),
                );
                state.cursor.set_char_range(Some(all));
                state.store(ui.ctx(), id);
            }
        }
        if response.changed() {
            self.apply_input(input, &text);
            self.editing = Some((input, text));
        } else if response.lost_focus() {
            self.editing = None;
        }
    }

    fn texture_key(&self) -> TextureKey {
        let h = self.model.hsb;
        TextureKey {
            mode: self.mode,
            model: [h.h.to_bits(), h.s.to_bits(), h.b.to_bits()],
            web_only: self.web_only,
        }
    }

    /// Renders a `size`×256 image (x, y in 0..1) from per-pixel models.
    fn render(&self, at: impl Fn(&Model, f32, f32) -> Model, width: usize) -> ColorImage {
        let height = TEXTURE_PX;
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            let fy = (y as f32 + 0.5) / height as f32;
            for x in 0..width {
                let fx = (x as f32 + 0.5) / width as f32;
                let mut c = at(&self.model, fx, fy).color();
                if self.web_only {
                    c = web_safe(c);
                }
                pixels.push(to_c32(c));
            }
        }
        ColorImage::new([width, height], pixels)
    }
}

fn lerp((lo, hi): (f32, f32), t: f32) -> f32 {
    lo + (hi - lo) * t
}

fn unlerp((lo, hi): (f32, f32), v: f32) -> f32 {
    ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
}

fn to_c32(c: Color) -> Color32 {
    let [r, g, b, _] = c.to_rgba8();
    Color32::from_rgb(r, g, b)
}

/// A small white right-pointing arrow with its tip at `tip`.
fn arrow(painter: &egui::Painter, tip: Pos2) {
    let back = tip.x - pt(8.0);
    painter.add(Shape::convex_polygon(
        vec![
            tip,
            pos2(back, tip.y - pt(4.5)),
            pos2(back, tip.y + pt(4.5)),
        ],
        Color32::WHITE,
        Stroke::new(1.0, Color32::from_gray(0x30)),
    ));
}

fn radio(painter: &egui::Painter, center: Pos2, selected: bool) {
    let r = pt(6.5);
    if selected {
        painter.circle_filled(center, r, Color32::from_gray(0xe6));
        painter.circle_filled(center, pt(2.5), Color32::from_gray(0x33));
    } else {
        painter.circle(
            center,
            r,
            Color32::from_gray(0x4a),
            Stroke::new(1.0, Color32::from_gray(0x8a)),
        );
    }
}

fn checkbox(painter: &egui::Painter, rect: Rect, checked: bool) {
    painter.rect(
        rect,
        2,
        Color32::from_gray(0x4a),
        Stroke::new(1.0, Color32::from_gray(0x8a)),
        StrokeKind::Inside,
    );
    if checked {
        let s = Stroke::new(1.5, color::TEXT);
        let a = rect.left_center() + vec2(rect.width() * 0.2, 0.0);
        let b = rect.center_bottom() - vec2(rect.width() * 0.1, rect.height() * 0.25);
        let c = rect.right_top() + vec2(-rect.width() * 0.2, rect.height() * 0.25);
        painter.line_segment([a, b], s);
        painter.line_segment([b, c], s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Color {
        from_hex(s).unwrap()
    }

    #[test]
    fn displays_photoshop_values() {
        let p = ColorPicker::new("t", hex("00afdc"));
        assert_eq!(p.input_text(Input::Channel(Channel::Hue)), "192");
        assert_eq!(p.input_text(Input::Channel(Channel::Saturation)), "100");
        assert_eq!(p.input_text(Input::Channel(Channel::Brightness)), "86");
        assert_eq!(p.input_text(Input::Channel(Channel::Green)), "175");
        assert_eq!(p.input_text(Input::Channel(Channel::L)), "66");
        assert_eq!(p.input_text(Input::Channel(Channel::A)), "-27");
        assert_eq!(p.input_text(Input::Channel(Channel::B)), "-34");
        assert_eq!(p.input_text(Input::Hex), "00afdc");
    }

    #[test]
    fn typed_inputs_change_the_color() {
        let mut p = ColorPicker::new("t", hex("00afdc"));
        p.apply_input(Input::Hex, "ff0000");
        assert_eq!(to_hex(p.color()), "ff0000");
        p.apply_input(Input::Channel(Channel::Green), "255");
        assert_eq!(to_hex(p.color()), "ffff00");
        p.apply_input(Input::Channel(Channel::Hue), "240");
        assert_eq!(to_hex(p.color()), "0000ff");
        // Partial input is ignored
        p.apply_input(Input::Hex, "12");
        assert_eq!(to_hex(p.color()), "0000ff");
        // Out of range values are clamped
        p.apply_input(Input::Channel(Channel::Red), "999");
        assert_eq!(to_hex(p.color()), "ff00ff");
    }

    #[test]
    fn hue_survives_black() {
        let mut p = ColorPicker::new("t", hex("00afdc"));
        p.apply_input(Input::Channel(Channel::Brightness), "0");
        assert_eq!(p.input_text(Input::Channel(Channel::Hue)), "192");
        // Back from black, the hue and saturation are still there
        p.apply_input(Input::Channel(Channel::Brightness), "86");
        assert_eq!(p.input_text(Input::Channel(Channel::Hue)), "192");
        assert_eq!(p.input_text(Input::Channel(Channel::Saturation)), "100");
    }

    #[test]
    fn web_only_snaps() {
        let mut p = ColorPicker::new("t", hex("00afdc"));
        p.web_only = true;
        let m = p.model;
        p.set_model(m);
        assert_eq!(to_hex(p.color()), "0099cc");
    }
}
