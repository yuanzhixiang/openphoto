//! File > New... (Cmd+N): Photoshop 2026's New Document dialog. Tabs of
//! blank document presets (Recent, Saved, Photo, Print, Art & Illustration,
//! Web, Mobile, Film & Video) on the left, the chosen preset's details on
//! the right, laid out at the positions measured on Photoshop (1080 × 718
//! pt). Sizes are in Photoshop points from the dialog's top-left corner.

use egui::{
    Align2, Color32, CornerRadius, Key, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    pos2, vec2,
};

use super::canvas_size::MAX_DIMENSION;
use super::document_presets;
pub use super::document_presets::{Kind, Preset, Unit};
use crate::theme::{self, pt};

const SIZE: Vec2 = vec2(pt(1080.0), pt(718.0));
/// The window's title bar, and its edge.
const TITLE_BAR: Color32 = Color32::from_rgb(0xd3, 0xd4, 0xd5);
/// The tab bar and the details panel; the preset list behind the cards.
const PANEL: Color32 = Color32::from_gray(0x32);
const LIST: Color32 = Color32::from_gray(0x25);
const TAB_LINE: Color32 = Color32::from_gray(0x3e);
/// Text: values and the current tab, labels and other tabs, quieter text.
const TEXT: Color32 = Color32::from_gray(0xe3);
const LABEL: Color32 = Color32::from_gray(0xc3);
const DIM: Color32 = Color32::from_gray(0xa8);
const BORDER: Color32 = Color32::from_gray(0x4a);
const FIELD: Color32 = Color32::from_gray(0x25);
const POPUP: Color32 = Color32::from_gray(0x2f);
const BLUE: Color32 = Color32::from_rgb(0x41, 0x7e, 0xe4);
const CREATE: Color32 = Color32::from_rgb(0x36, 0x71, 0xdf);
const LINK: Color32 = Color32::from_rgb(0x4b, 0x9c, 0xf5);
/// The preset cards: 168 pt square, 180.9 pt apart, the first at x 26.
const CARD: f32 = 168.0;
const PITCH: f32 = 180.9;
/// "Background Contents".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Contents {
    #[default]
    White,
    Black,
    BackgroundColor,
    Transparent,
}

impl Contents {
    const ALL: [Self; 4] = [
        Self::White,
        Self::Black,
        Self::BackgroundColor,
        Self::Transparent,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::White => "White",
            Self::Black => "Black",
            Self::BackgroundColor => "Background Color",
            Self::Transparent => "Transparent",
        }
    }
}

pub enum Outcome {
    Open,
    Cancel,
    Create {
        name: String,
        width: u32,
        height: u32,
        resolution: f32,
        contents: Contents,
        /// The settings as entered, for the Recent tab.
        preset: Preset,
    },
}

/// The tabs and their text's left edges: Recent, Saved, then the preset
/// categories (`document_presets::category`).
const TABS: [(&str, f32); 8] = [
    ("Recent", 49.5),
    ("Saved", 112.5),
    ("Photo", 172.0),
    ("Print", 230.0),
    ("Art & Illustration", 280.0),
    ("Web", 398.0),
    ("Mobile", 448.5),
    ("Film & Video", 513.0),
];

pub struct NewDocumentDialog {
    tab: usize,
    recent: Vec<Preset>,
    saved: Vec<Preset>,
    /// Presets saved in this session, for the host to keep.
    new_saved: Vec<Preset>,
    /// The card chosen: its tab and position.
    selected: Option<(usize, usize)>,
    welcome_closed: bool,
    name: String,
    width: String,
    height: String,
    unit: Unit,
    resolution: String,
    /// Pixels per centimeter instead of per inch.
    per_cm: bool,
    contents: Contents,
    advanced: bool,
    /// The app's background color, for Background Color's swatch.
    background: Color32,
}

impl NewDocumentDialog {
    /// `name` is the next "Untitled-N"; `size` the clipboard image's size
    /// when there is one (Photoshop's "Clipboard" preset). Otherwise the
    /// most recent preset, else 1920 × 1080 pixels at 72 ppi.
    pub fn new(
        name: String,
        size: Option<(u32, u32)>,
        recent: Vec<Preset>,
        saved: Vec<Preset>,
        welcome_closed: bool,
    ) -> Self {
        let mut d = Self {
            tab: 0,
            recent,
            saved,
            new_saved: Vec::new(),
            selected: None,
            welcome_closed,
            name,
            width: String::new(),
            height: String::new(),
            unit: Unit::Pixels,
            resolution: String::new(),
            per_cm: false,
            contents: Contents::White,
            advanced: true,
            background: Color32::WHITE,
        };
        let start = match size {
            Some((w, h)) => custom(w as f64, h as f64, Unit::Pixels, 72.0),
            None => match d.recent.first() {
                Some(p) => {
                    d.selected = Some((0, 0));
                    p.clone()
                }
                None => custom(1920.0, 1080.0, Unit::Pixels, 72.0),
            },
        };
        d.apply(&start);
        d
    }

    /// The color Background Contents' "Background Color" shows.
    pub fn with_background(mut self, color: Color32) -> Self {
        self.background = color;
        self
    }

    /// Presets saved with the save icon since the last call.
    pub fn take_saved(&mut self) -> Vec<Preset> {
        std::mem::take(&mut self.new_saved)
    }

    /// Whether the Recent tab's welcome box was closed.
    pub fn welcome_closed(&self) -> bool {
        self.welcome_closed
    }

    fn apply(&mut self, p: &Preset) {
        self.unit = p.unit;
        self.width = p.unit.format(p.width);
        self.height = p.unit.format(p.height);
        self.per_cm = false;
        self.resolution = Unit::Pixels.format(p.ppi);
    }

    fn ppi(&self) -> Option<f64> {
        let r: f64 = self.resolution.trim().parse().ok()?;
        let ppi = if self.per_cm { r * 2.54 } else { r };
        (1.0..=10_000.0).contains(&ppi).then_some(ppi)
    }

    /// Width and height in the unit, if they parse.
    fn dims(&self) -> Option<(f64, f64)> {
        Some((
            self.width.trim().parse().ok()?,
            self.height.trim().parse().ok()?,
        ))
    }

    /// The document to create: whole pixels within Photoshop's limits.
    fn values(&self) -> Option<(u32, u32, f32)> {
        let ppi = self.ppi()?;
        let (w, h) = self.dims()?;
        let k = self.unit.pixels(ppi);
        let px = |v: f64| {
            let p = (v * k).round();
            (1.0..=MAX_DIMENSION as f64)
                .contains(&p)
                .then_some(p as u32)
        };
        Some((px(w)?, px(h)?, ppi as f32))
    }

    /// The current settings as a preset.
    fn preset(&self, name: &str) -> Option<Preset> {
        let (w, h) = self.dims()?;
        Some(Preset {
            name: name.into(),
            width: w,
            height: h,
            unit: self.unit,
            ppi: self.ppi()?,
            kind: Kind::Custom,
        })
    }

    fn set_unit(&mut self, unit: Unit) {
        if let (Some((w, h)), Some(ppi)) = (self.dims(), self.ppi()) {
            let k = self.unit.pixels(ppi) / unit.pixels(ppi);
            self.width = unit.format(w * k);
            self.height = unit.format(h * k);
        }
        self.unit = unit;
    }

    /// Shows a tab: like Photoshop, its first preset is chosen.
    fn select_tab(&mut self, tab: usize) {
        self.tab = tab;
        if let Some(p) = self.presets(tab).first() {
            self.selected = Some((tab, 0));
            self.apply(p);
        }
    }

    fn presets(&self, tab: usize) -> Vec<Preset> {
        match tab {
            0 => self.recent.clone(),
            1 => self.saved.clone(),
            t => document_presets::category(t - 2),
        }
    }

    fn create(&self) -> Option<Outcome> {
        let (width, height, resolution) = self.values()?;
        let name = if self.name.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            self.name.trim().to_owned()
        };
        Some(Outcome::Create {
            name,
            width,
            height,
            resolution,
            contents: self.contents,
            preset: self.preset("Custom")?,
        })
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome {
        let mut outcome = Outcome::Open;
        egui::Modal::new(egui::Id::new("new-document"))
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
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();
        window(&painter, frame);
        painter.rect_filled(r(0.0, 28.5, 1080.0, 76.0), 0, PANEL);
        painter.rect_filled(r(0.0, 74.0, 1080.0, 76.0), 0, TAB_LINE);
        painter.rect_filled(
            r(0.0, 76.0, 770.0, 718.0),
            CornerRadius {
                sw: pt(10.0) as u8,
                ..Default::default()
            },
            LIST,
        );
        painter.rect_filled(
            r(770.0, 76.0, 1080.0, 718.0),
            CornerRadius {
                se: pt(10.0) as u8,
                ..Default::default()
            },
            PANEL,
        );

        // Tabs: the current one in white over a 2 pt white line
        for (k, (label, x)) in TABS.into_iter().enumerate() {
            let on = self.tab == k;
            let galley = painter.layout_no_wrap(label.into(), theme::uxp(pt(14.0)), TEXT);
            let text = Rect::from_min_size(at(x, 51.0 - 8.0), vec2(galley.size().x, pt(16.0)));
            let hit = Rect::from_min_max(
                pos2(
                    text.left() - if k == 0 { pt(24.0) } else { pt(8.0) },
                    at(0.0, 30.0).y,
                ),
                pos2(text.right() + pt(8.0), at(0.0, 74.0).y),
            );
            let resp = ui.interact(hit, ui.id().with(("new-doc-tab", k)), Sense::click());
            if resp.clicked() && self.tab != k {
                self.select_tab(k);
            }
            let ink = if on || resp.hovered() {
                Color32::WHITE
            } else {
                DIM
            };
            painter.text(
                pos2(text.left(), at(0.0, 51.0).y),
                Align2::LEFT_CENTER,
                label,
                theme::uxp(pt(14.0)),
                ink,
            );
            if k == 0 {
                clock(&painter, at(35.0, 51.0), ink);
            }
            if on {
                let left = if k == 0 { at(27.0, 0.0).x } else { text.left() };
                painter.rect_filled(
                    Rect::from_min_max(
                        pos2(left, at(0.0, 74.0).y),
                        pos2(text.right(), at(0.0, 76.0).y),
                    ),
                    0,
                    Color32::WHITE,
                );
            }
        }

        let mut create = false;
        self.list(ui, r(0.0, 76.0, 770.0, 650.0), &mut create);
        search_bar(ui, frame);
        self.details(ui, frame);

        let values = self.values();
        let close = pill(ui, r(900.0, 658.0, 972.0, 690.0), "Close", false, true);
        let create_button = pill(
            ui,
            r(988.0, 658.0, 1060.0, 690.0),
            "Create",
            true,
            values.is_some(),
        );
        if close.clicked() {
            return Outcome::Cancel;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if (create || create_button.clicked() || enter)
            && let Some(o) = self.create()
        {
            return o;
        }
        Outcome::Open
    }

    /// The left side: the welcome box (Recent, until closed) or the empty
    /// Saved note, a heading and the cards, scrolling.
    fn list(&mut self, ui: &mut Ui, area: Rect, create: &mut bool) {
        let presets = self.presets(self.tab);
        let scales = document_presets::scales(&presets);
        let welcome = self.tab == 0 && !self.welcome_closed;
        let empty_saved = self.tab == 1 && presets.is_empty();
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area));
        let mut close_welcome = false;
        let mut go_photo = false;
        let mut chosen = None;
        egui::ScrollArea::vertical()
            .id_salt(("new-doc-list", self.tab))
            .show(&mut child, |ui| {
                let top = if welcome { 286.0 } else { 103.0 } - 76.0;
                let rows = presets.len().div_ceil(4);
                let height = if empty_saved {
                    200.0
                } else {
                    top + 21.0 + rows as f32 * PITCH + 20.0
                };
                let (content, _) =
                    ui.allocate_exact_size(vec2(area.width(), pt(height)), Sense::hover());
                let at = |x: f32, y: f32| content.min + vec2(pt(x), pt(y - 76.0));
                let painter = ui.painter().clone();
                if welcome || empty_saved {
                    let boxr = Rect::from_min_max(at(13.0, 89.0), at(741.0, 262.0));
                    painter.rect_filled(boxr, pt(4.0), Color32::from_gray(0x2e));
                    let x = ui.interact(
                        Rect::from_center_size(at(720.0, 110.0), Vec2::splat(pt(20.0))),
                        ui.id().with("welcome-close"),
                        Sense::click(),
                    );
                    cross(
                        &painter,
                        at(720.0, 110.0),
                        if x.hovered() { Color32::WHITE } else { TEXT },
                    );
                    if x.clicked() {
                        close_welcome = true;
                    }
                    let (title, lines) = if welcome {
                        (
                            "Let’s start something new.",
                            "Start with your own document settings or from a range of",
                        )
                    } else {
                        (
                            "You can always find it here.",
                            "This is where you’ll find your saved",
                        )
                    };
                    painter.text(
                        at(377.0, 147.0),
                        Align2::CENTER_CENTER,
                        title,
                        theme::uxp_bold(pt(28.0)),
                        Color32::WHITE,
                    );
                    if welcome {
                        // "...range of document presets to / get working quickly."
                        let font = theme::uxp(pt(16.0));
                        let a = painter.layout_no_wrap(format!("{lines} "), font.clone(), TEXT);
                        let l =
                            painter.layout_no_wrap("document presets".into(), font.clone(), LINK);
                        let b = painter.layout_no_wrap(" to".into(), font.clone(), TEXT);
                        let total = a.size().x + l.size().x + b.size().x;
                        let y = at(0.0, 187.0).y - a.size().y / 2.0;
                        let mut x = at(377.0, 0.0).x - total / 2.0;
                        painter.galley(pos2(x, y), a.clone(), TEXT);
                        x += a.size().x;
                        let link_rect = Rect::from_min_size(pos2(x, y), l.size());
                        painter.galley(pos2(x, y), l.clone(), LINK);
                        painter.line_segment(
                            [
                                pos2(link_rect.left(), link_rect.bottom() - pt(1.5)),
                                pos2(link_rect.right(), link_rect.bottom() - pt(1.5)),
                            ],
                            Stroke::new(pt(1.0), LINK),
                        );
                        if ui
                            .interact(link_rect, ui.id().with("presets-link"), Sense::click())
                            .clicked()
                        {
                            go_photo = true;
                        }
                        painter.galley(pos2(x + l.size().x, y), b, TEXT);
                        painter.text(
                            at(377.0, 211.0),
                            Align2::CENTER_CENTER,
                            "get working quickly.",
                            font,
                            TEXT,
                        );
                    } else {
                        for (k, line) in [
                            "This is where you’ll find your saved presets",
                            "that you’ve saved with the icon next to the name.",
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            painter.text(
                                at(377.0, 187.0 + 24.0 * k as f32),
                                Align2::CENTER_CENTER,
                                line,
                                theme::uxp(pt(16.0)),
                                TEXT,
                            );
                        }
                    }
                }
                if empty_saved {
                    return;
                }
                let heading = match self.tab {
                    0 => "YOUR RECENT ITEMS",
                    1 => "YOUR SAVED PRESETS",
                    _ => "BLANK DOCUMENT PRESETS",
                };
                let header_y = if welcome { 286.0 } else { 103.0 };
                let h = painter.text(
                    at(26.5, header_y),
                    Align2::LEFT_CENTER,
                    heading,
                    theme::uxp_bold(pt(13.0)),
                    TEXT,
                );
                painter.text(
                    pos2(h.right() + pt(6.0), h.center().y),
                    Align2::LEFT_CENTER,
                    format!("({})", presets.len()),
                    theme::uxp(pt(13.0)),
                    TEXT,
                );
                let first_top = header_y + 21.0;
                for (k, p) in presets.iter().enumerate() {
                    let (col, row) = (k % 4, k / 4);
                    let x0 = 26.0 + PITCH * col as f32;
                    let y0 = first_top + PITCH * row as f32;
                    let card = Rect::from_min_size(at(x0, y0), Vec2::splat(pt(CARD)));
                    let resp =
                        ui.interact(card, ui.id().with(("card", self.tab, k)), Sense::click());
                    let selected = self.selected == Some((self.tab, k));
                    if resp.hovered() && !selected {
                        painter.rect_filled(card, 0, Color32::from_gray(0x32));
                    }
                    if selected {
                        painter.rect_stroke(
                            card,
                            0,
                            Stroke::new(pt(3.0), BLUE),
                            StrokeKind::Inside,
                        );
                    }
                    let ink = Color32::from_gray(0xb9);
                    document_presets::paint_icon(
                        &painter,
                        at(x0 + CARD / 2.0, y0 + 62.0),
                        p,
                        scales[k],
                        ink,
                    );
                    let name = if resp.hovered() && !selected {
                        LINK
                    } else {
                        Color32::WHITE
                    };
                    painter.text(
                        at(x0 + CARD / 2.0, y0 + 121.5),
                        Align2::CENTER_CENTER,
                        super::common::elide(ui, &p.name, pt(CARD - 12.0)),
                        theme::uxp(pt(12.0)),
                        name,
                    );
                    painter.text(
                        at(x0 + CARD / 2.0, y0 + 140.0),
                        Align2::CENTER_CENTER,
                        p.size_label(),
                        theme::uxp(pt(12.0)),
                        Color32::from_gray(0xc8),
                    );
                    if resp.clicked() {
                        chosen = Some((k, false));
                    }
                    if resp.double_clicked() {
                        chosen = Some((k, true));
                    }
                }
            });
        if close_welcome {
            self.welcome_closed = true;
        }
        if go_photo {
            self.select_tab(2);
        }
        if let Some((k, now)) = chosen {
            self.selected = Some((self.tab, k));
            self.apply(&presets[k]);
            // Double-clicking a card creates the document at once
            *create |= now;
        }
    }

    /// The right side: the preset's name and details.
    fn details(&mut self, ui: &mut Ui, frame: Rect) {
        let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
        let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));
        let painter = ui.painter().clone();
        let label = |x: f32, cy: f32, text: &str| {
            painter.text(
                at(x, cy),
                Align2::LEFT_CENTER,
                text,
                theme::uxp(pt(12.0)),
                LABEL,
            );
        };
        painter.text(
            at(791.0, 100.75),
            Align2::LEFT_CENTER,
            "PRESET DETAILS",
            theme::uxp(pt(12.5)),
            LABEL,
        );
        // The name, on an underline, and the save icon
        let name_rect = r(790.0, 120.0, 1015.0, 149.0);
        let mut child =
            ui.new_child(egui::UiBuilder::new().max_rect(name_rect.shrink2(vec2(pt(4.0), 0.0))));
        child.add_sized(
            name_rect.shrink2(vec2(pt(4.0), 0.0)).size(),
            egui::TextEdit::singleline(&mut self.name)
                .id_salt("new-doc-name")
                .frame(egui::Frame::NONE)
                .font(theme::uxp(pt(13.5)))
                .text_color(TEXT)
                .vertical_align(egui::Align::Center),
        );
        painter.rect_filled(r(790.0, 149.0, 1015.0, 150.0), 0, Color32::from_gray(0x90));
        let save = r(1034.0, 118.0, 1063.0, 144.0);
        let save_r = ui.interact(save, ui.id().with("save-preset"), Sense::click());
        save_icon(
            &painter,
            save.center(),
            if save_r.hovered() {
                Color32::WHITE
            } else {
                TEXT
            },
        );
        if save_r.on_hover_text("Save Preset").clicked()
            && let Some(p) = self.preset(self.name.trim())
        {
            self.saved.push(p.clone());
            self.new_saved.push(p);
        }

        label(791.0, 168.5, "Width");
        field(
            ui,
            r(790.0, 180.5, 862.0, 212.5),
            &mut self.width,
            "nd-width",
        );
        let mut unit = self.unit;
        popup(
            ui,
            r(874.0, 180.5, 1060.0, 212.5),
            "nd-unit",
            unit.label(),
            true,
            |ui| {
                for u in Unit::ALL {
                    ui.selectable_value(&mut unit, u, u.label());
                }
            },
        );
        if unit != self.unit {
            self.set_unit(unit);
        }

        label(791.0, 232.0, "Height");
        label(874.5, 231.0, "Orientation");
        label(944.0, 231.0, "Artboards");
        field(
            ui,
            r(790.0, 243.5, 862.0, 275.5),
            &mut self.height,
            "nd-height",
        );
        // Orientation: the current one blue; clicking the other swaps
        let landscape = self.dims().is_some_and(|(w, h)| w > h);
        for (k, rect) in [r(875.0, 247.0, 897.0, 272.0), r(905.5, 250.0, 930.5, 269.5)]
            .into_iter()
            .enumerate()
        {
            let on = (k == 1) == landscape;
            orientation(&painter, rect, k == 0, on);
            if ui
                .interact(
                    rect.expand(pt(2.0)),
                    ui.id().with(("orient", k)),
                    Sense::click(),
                )
                .clicked()
                && !on
            {
                std::mem::swap(&mut self.width, &mut self.height);
            }
        }
        // Artboards aren't supported
        painter.rect(
            r(944.0, 252.0, 958.0, 266.0),
            pt(2.0),
            Color32::from_gray(0x48),
            Stroke::new(pt(1.0), Color32::from_gray(0x85)),
            StrokeKind::Inside,
        );

        label(791.0, 292.0, "Resolution");
        field(
            ui,
            r(790.0, 304.0, 862.0, 336.0),
            &mut self.resolution,
            "nd-res",
        );
        let mut per_cm = self.per_cm;
        let res_label = if per_cm {
            "Pixels/Centimeter"
        } else {
            "Pixels/Inch"
        };
        popup(
            ui,
            r(874.0, 304.0, 1060.0, 336.0),
            "nd-res-unit",
            res_label,
            true,
            |ui| {
                ui.selectable_value(&mut per_cm, false, "Pixels/Inch");
                ui.selectable_value(&mut per_cm, true, "Pixels/Centimeter");
            },
        );
        if per_cm != self.per_cm {
            if let Ok(v) = self.resolution.trim().parse::<f64>() {
                let v = if per_cm { v / 2.54 } else { v * 2.54 };
                self.resolution = Unit::Inches.format(v);
            }
            self.per_cm = per_cm;
        }

        label(791.0, 354.5, "Color Mode");
        popup(
            ui,
            r(790.0, 367.0, 957.5, 399.0),
            "nd-mode",
            "RGB Color",
            true,
            |ui| {
                for (m, on) in [
                    ("Bitmap", false),
                    ("Grayscale", false),
                    ("RGB Color", true),
                    ("CMYK Color", false),
                    ("Lab Color", false),
                ] {
                    ui.add_enabled_ui(on, |ui| {
                        let _ = ui.selectable_label(on, m);
                    });
                }
            },
        );
        popup(
            ui,
            r(969.5, 367.0, 1060.0, 399.0),
            "nd-bits",
            "8 bit",
            true,
            |ui| {
                for (b, on) in [("8 bit", true), ("16 bit", false), ("32 bit", false)] {
                    ui.add_enabled_ui(on, |ui| {
                        let _ = ui.selectable_label(on, b);
                    });
                }
            },
        );

        label(791.0, 415.5, "Background Contents");
        let mut contents = self.contents;
        popup(
            ui,
            r(790.0, 428.0, 1016.0, 460.0),
            "nd-contents",
            contents.label(),
            true,
            |ui| {
                for c in Contents::ALL {
                    ui.selectable_value(&mut contents, c, c.label());
                }
                ui.add_enabled_ui(false, |ui| {
                    let _ = ui.selectable_label(false, "Custom...");
                });
            },
        );
        self.contents = contents;
        let swatch = r(1028.0, 428.0, 1060.0, 460.0);
        match self.contents {
            Contents::Transparent => crate::widgets::checkerboard(&painter, swatch, pt(4.0)),
            c => {
                let fill = match c {
                    Contents::Black => Color32::BLACK,
                    Contents::BackgroundColor => self.background,
                    _ => Color32::WHITE,
                };
                painter.rect_filled(swatch, pt(3.0), fill);
            }
        }
        painter.rect_stroke(
            swatch,
            pt(3.0),
            Stroke::new(pt(1.0), Color32::from_gray(0x5b)),
            StrokeKind::Inside,
        );

        // Advanced Options, open by default
        let adv = r(790.0, 468.0, 960.0, 490.0);
        if ui
            .interact(adv, ui.id().with("advanced"), Sense::click())
            .clicked()
        {
            self.advanced = !self.advanced;
        }
        chevron(&painter, at(797.0, 478.75), self.advanced, TEXT);
        painter.text(
            at(805.0, 478.75),
            Align2::LEFT_CENTER,
            "Advanced Options",
            theme::uxp(pt(12.0)),
            LABEL,
        );
        if self.advanced {
            label(791.0, 507.5, "Color Profile");
            popup(
                ui,
                r(790.0, 519.5, 1060.0, 551.5),
                "nd-profile",
                "Working RGB: sRGB IEC61966-2.1",
                true,
                |ui| {
                    let _ = ui.selectable_label(true, "Working RGB: sRGB IEC61966-2.1");
                    ui.add_enabled_ui(false, |ui| {
                        let _ = ui.selectable_label(false, "Don't Color Manage this Document");
                    });
                },
            );
            label(791.0, 568.0, "Pixel Aspect Ratio");
            popup(
                ui,
                r(790.0, 580.5, 1060.0, 612.5),
                "nd-aspect",
                "Square Pixels",
                true,
                |ui| {
                    let _ = ui.selectable_label(true, "Square Pixels");
                },
            );
        }
    }
}

/// A preset for the settings entered by hand.
fn custom(w: f64, h: f64, unit: Unit, ppi: f64) -> Preset {
    Preset {
        name: "Custom".into(),
        width: w,
        height: h,
        unit,
        ppi,
        kind: Kind::Custom,
    }
}

/// The macOS window: rounded, its light title bar with the traffic lights
/// and the bold title.
fn window(painter: &egui::Painter, frame: Rect) {
    let radius = CornerRadius::same(pt(10.0) as u8);
    painter.add(
        egui::Shadow {
            offset: [0, 12],
            blur: 40,
            spread: 0,
            color: Color32::from_black_alpha(110),
        }
        .as_shape(frame, radius),
    );
    painter.rect_filled(frame, radius, PANEL);
    let bar = Rect::from_min_size(frame.min, vec2(frame.width(), pt(28.0)));
    painter.rect_filled(
        bar,
        CornerRadius {
            nw: pt(10.0) as u8,
            ne: pt(10.0) as u8,
            ..Default::default()
        },
        TITLE_BAR,
    );
    painter.rect_filled(
        Rect::from_min_size(bar.left_bottom(), vec2(frame.width(), pt(0.5))),
        0,
        Color32::from_gray(0x0c),
    );
    painter.text(
        bar.center(),
        Align2::CENTER_CENTER,
        "New Document",
        theme::dialog_bold(pt(13.0)),
        Color32::from_gray(0x3c),
    );
    for (x, fill, rim) in [
        (
            14.0,
            Color32::from_rgb(0xed, 0x6b, 0x5f),
            Color32::from_rgb(0xcf, 0x4e, 0x40),
        ),
        (34.0, Color32::from_gray(0xbb), Color32::from_gray(0x97)),
        (
            54.0,
            Color32::from_rgb(0x63, 0xc9, 0x55),
            Color32::from_rgb(0x4f, 0xa6, 0x3b),
        ),
    ] {
        let c = frame.min + vec2(pt(x), pt(14.0));
        painter.circle_filled(c, pt(6.0), fill);
        painter.circle_stroke(c, pt(5.75), Stroke::new(pt(0.5), rim));
    }
}

/// A field: `#252525` with a 1 pt `#4a4a4a` edge, its value 15.5 pt in.
fn field(ui: &mut Ui, rect: Rect, text: &mut String, id: &str) {
    let edit_id = ui.id().with(id);
    let focused = ui.memory(|m| m.has_focus(edit_id));
    ui.painter().rect(
        rect,
        pt(4.0),
        FIELD,
        Stroke::new(pt(1.0), if focused { BLUE } else { BORDER }),
        StrokeKind::Inside,
    );
    let inner = Rect::from_min_max(
        rect.min + vec2(pt(15.5), 0.0),
        rect.max - vec2(pt(4.0), 0.0),
    );
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    egui::TextEdit::singleline(text)
        .id(edit_id)
        .frame(egui::Frame::NONE)
        .font(theme::uxp(pt(15.0)))
        .text_color(TEXT)
        .vertical_align(egui::Align::Center)
        .desired_width(inner.width())
        .min_size(vec2(0.0, inner.height()))
        .show(&mut child);
}

/// A pop-up menu: `#2f2f2f` with a 1 pt `#4a4a4a` edge, its value 13 pt in
/// and a chevron 19 pt from the right.
fn popup(
    ui: &mut Ui,
    rect: Rect,
    id: &str,
    value: &str,
    enabled: bool,
    menu: impl FnOnce(&mut Ui),
) {
    let resp = ui.interact(rect, ui.id().with(id), Sense::click());
    let fill = if enabled && resp.hovered() {
        Color32::from_gray(0x38)
    } else {
        POPUP
    };
    ui.painter().rect(
        rect,
        pt(4.0),
        fill,
        Stroke::new(pt(1.0), BORDER),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_center() + vec2(pt(12.5), 0.0),
        Align2::LEFT_CENTER,
        super::common::elide(ui, value, rect.width() - pt(44.0)),
        theme::uxp(pt(15.0)),
        TEXT,
    );
    chevron(
        ui.painter(),
        pos2(rect.right() - pt(19.0), rect.center().y),
        true,
        TEXT,
    );
    if enabled {
        egui::Popup::menu(&resp)
            .id(ui.id().with((id, "menu")))
            .show(menu);
    }
}

/// A 9 pt chevron, pointing down (open) or right.
fn chevron(painter: &egui::Painter, c: Pos2, down: bool, ink: Color32) {
    let s = pt(4.5);
    let pts = if down {
        vec![
            c + vec2(-s, -s / 2.0),
            c + vec2(0.0, s / 2.0),
            c + vec2(s, -s / 2.0),
        ]
    } else {
        vec![
            c + vec2(-s / 2.0, -s),
            c + vec2(s / 2.0, 0.0),
            c + vec2(-s / 2.0, s),
        ]
    };
    painter.add(Shape::line(pts, Stroke::new(pt(1.5), ink)));
}

/// Close (outlined) and Create (blue) pills.
fn pill(ui: &mut Ui, rect: Rect, label: &str, primary: bool, enabled: bool) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let resp = ui.interact(rect, ui.id().with(("nd-button", label)), sense);
    let painter = ui.painter();
    if primary {
        let fill = if !enabled {
            Color32::from_gray(0x50)
        } else if resp.hovered() {
            Color32::from_rgb(0x2a, 0x63, 0xcf)
        } else {
            CREATE
        };
        painter.rect_filled(rect, CornerRadius::same(255), fill);
    } else {
        let fill = if resp.hovered() {
            Color32::from_gray(0x45)
        } else {
            PANEL
        };
        painter.rect(
            rect,
            CornerRadius::same(255),
            fill,
            Stroke::new(pt(2.0), TEXT),
            StrokeKind::Inside,
        );
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        theme::uxp_bold(pt(15.0)),
        if enabled {
            Color32::WHITE
        } else {
            Color32::from_gray(0x90)
        },
    );
    resp
}

/// The Adobe Stock search under the list: it searches a service this app
/// doesn't use, so it only shows.
fn search_bar(ui: &mut Ui, frame: Rect) {
    let at = |x: f32, y: f32| frame.min + vec2(pt(x), pt(y));
    let painter = ui.painter();
    painter.rect_filled(
        Rect::from_min_max(at(26.0, 655.0), at(744.0, 703.0)),
        0,
        PANEL,
    );
    let c = at(49.0, 678.0);
    painter.circle_stroke(c, pt(6.0), Stroke::new(pt(1.5), TEXT));
    painter.line_segment(
        [c + vec2(-pt(4.5), pt(4.5)), c + vec2(-pt(9.0), pt(9.0))],
        Stroke::new(pt(2.0), TEXT),
    );
    painter.text(
        at(70.5, 679.0),
        Align2::LEFT_CENTER,
        "Search for an image or template from Adobe Stock",
        theme::uxp(pt(14.0)),
        DIM,
    );
    painter.rect_filled(
        Rect::from_min_max(at(68.0, 695.0), at(645.0, 695.5)),
        0,
        Color32::from_gray(0x55),
    );
    let b = Rect::from_min_max(at(656.5, 664.0), at(727.0, 694.5));
    painter.rect_stroke(
        b,
        CornerRadius::same(255),
        Stroke::new(pt(1.5), Color32::from_gray(0xb0)),
        StrokeKind::Inside,
    );
    painter.text(
        b.center(),
        Align2::CENTER_CENTER,
        "Search",
        theme::uxp_bold(pt(14.0)),
        Color32::from_gray(0xb0),
    );
}

/// The Recent tab's clock.
fn clock(painter: &egui::Painter, c: Pos2, ink: Color32) {
    painter.circle_stroke(c, pt(7.5), Stroke::new(pt(1.25), ink));
    painter.add(Shape::line(
        vec![c + vec2(0.0, -pt(4.5)), c, c + vec2(pt(3.0), pt(2.5))],
        Stroke::new(pt(1.25), ink),
    ));
}

/// The welcome box's close cross.
fn cross(painter: &egui::Painter, c: Pos2, ink: Color32) {
    let s = pt(5.5);
    let stroke = Stroke::new(pt(1.5), ink);
    painter.line_segment([c + vec2(-s, -s), c + vec2(s, s)], stroke);
    painter.line_segment([c + vec2(s, -s), c + vec2(-s, s)], stroke);
}

/// The save preset icon: a tray with an arrow into it.
fn save_icon(painter: &egui::Painter, c: Pos2, ink: Color32) {
    let stroke = Stroke::new(pt(1.5), ink);
    let p = |x: f32, y: f32| c + vec2(pt(x), pt(y));
    painter.add(Shape::line(
        vec![
            p(-6.0, -4.0),
            p(-10.0, -4.0),
            p(-10.0, 9.0),
            p(10.0, 9.0),
            p(10.0, -4.0),
            p(6.0, -4.0),
        ],
        stroke,
    ));
    painter.line_segment([p(0.0, -10.0), p(0.0, 4.0)], stroke);
    painter.add(Shape::line(
        vec![p(-4.0, 0.0), p(0.0, 4.0), p(4.0, 0.0)],
        stroke,
    ));
}

/// An orientation button: a frame with a person, blue when it's the
/// current orientation.
fn orientation(painter: &egui::Painter, rect: Rect, portrait: bool, on: bool) {
    let (fill, ink) = if on {
        (
            Color32::from_rgb(0x50, 0x8c, 0xe9),
            Color32::from_gray(0x32),
        )
    } else {
        (Color32::TRANSPARENT, Color32::from_gray(0xb9))
    };
    painter.rect(
        rect,
        0,
        fill,
        Stroke::new(pt(1.0), if on { fill } else { ink }),
        StrokeKind::Inside,
    );
    let c = rect.center();
    let s = if portrait { 1.0 } else { 0.85 };
    let head = c + vec2(0.0, -pt(4.0) * s);
    painter.circle_filled(
        head,
        pt(2.5) * s,
        if on { ink } else { Color32::from_gray(0xb9) },
    );
    painter.rect_filled(
        Rect::from_min_max(
            c + vec2(-pt(5.0) * s, pt(0.0)),
            c + vec2(pt(5.0) * s, pt(8.0) * s),
        ),
        pt(1.5),
        if on { ink } else { Color32::from_gray(0xb9) },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_units_and_validation() {
        let d = NewDocumentDialog::new("Untitled-1".into(), None, Vec::new(), Vec::new(), false);
        assert_eq!(d.values(), Some((1920, 1080, 72.0)));
        let mut d = NewDocumentDialog::new(
            "Untitled-2".into(),
            Some((640, 480)),
            Vec::new(),
            Vec::new(),
            false,
        );
        assert_eq!(d.values(), Some((640, 480, 72.0)));
        // Inches keep the size: 640 px at 72 ppi is 8.889 in
        d.set_unit(Unit::Inches);
        assert_eq!(d.width, "8.889");
        assert_eq!(d.values(), Some((640, 480, 72.0)));
        // A preset: 7 x 5 in at 300 ppi
        d.apply(&document_presets::category(0)[0]);
        assert_eq!(d.values(), Some((2100, 1500, 300.0)));
        d.width = "0".into();
        assert_eq!(d.values(), None);
        // The most recent preset is the starting one
        let recent = vec![custom(400.0, 300.0, Unit::Pixels, 72.0)];
        let d = NewDocumentDialog::new("U".into(), None, recent, Vec::new(), false);
        assert_eq!(d.values(), Some((400, 300, 72.0)));
        assert_eq!(d.selected, Some((0, 0)));
    }
}
