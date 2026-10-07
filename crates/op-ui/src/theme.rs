//! Visual spec: colors, sizes and fonts, taken from Photoshop's default
//! medium-gray interface.
//!
//! Sizes are measured in reference-screenshot pixels and scaled by [`UI_SCALE`]
//! to match Photoshop's actual on-screen size.

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke};

pub mod color {
    use egui::Color32;

    const fn gray(v: u8) -> Color32 {
        Color32::from_rgb(v, v, v)
    }

    /// Background of panels, the toolbar and the options bar.
    pub const PANEL: Color32 = gray(0x53);
    /// Pasteboard around the canvas.
    pub const PASTEBOARD: Color32 = gray(0x28);
    /// Tab bar background (document and panel tabs).
    pub const TAB_BAR: Color32 = gray(0x42);
    pub const TAB_INACTIVE: Color32 = gray(0x4c);
    pub const TAB_ACTIVE: Color32 = PANEL;
    /// Text field background.
    pub const FIELD: Color32 = gray(0x45);
    pub const FIELD_BORDER: Color32 = gray(0x3a);
    /// Selected tool in the toolbar.
    pub const TOOL_ACTIVE: Color32 = gray(0x38);
    pub const HOVER: Color32 = gray(0x5e);
    /// Layers panel list background and selected row.
    pub const LIST_BG: Color32 = gray(0x4d);
    pub const ROW_SELECTED: Color32 = gray(0x6b);
    /// Dark separator between sections.
    pub const SEPARATOR: Color32 = gray(0x39);
    pub const SEPARATOR_LIGHT: Color32 = gray(0x44);
    /// The options bar (Photoshop 2026, measured).
    pub const OPTIONS_BAR: Color32 = gray(0x53);
    /// The 3 pt dividers between the canvas, the icon strip and the panels,
    /// and the lines of the collapse bars (Photoshop 2026, measured).
    pub const DIVIDER_DARK: Color32 = gray(0x38);
    pub const DIVIDER_LIGHT: Color32 = gray(0x47);
    pub const COLLAPSE_BAR: Color32 = gray(0x42);
    pub const COLLAPSE_CHEVRON: Color32 = gray(0xc8);
    pub const TAB_TEXT_ACTIVE: Color32 = gray(0xf0);
    pub const TAB_TEXT: Color32 = gray(0xb0);
    pub const OPTIONS_SEPARATOR: Color32 = gray(0x3e);
    pub const OPTIONS_ICON: Color32 = gray(0xdd);
    pub const OPTIONS_BELL: Color32 = gray(0xb9);
    /// Disabled options-bar icons, such as the align buttons with one layer.
    pub const OPTIONS_ICON_DISABLED: Color32 = gray(0x98);
    /// Options-bar and dialog labels.
    pub const TEXT_BRIGHT: Color32 = gray(0xf0);
    pub const CHECKBOX: Color32 = gray(0xd4);
    pub const CHECK_MARK: Color32 = gray(0x32);
    pub const DROPDOWN_BORDER: Color32 = gray(0x66);
    pub const DROPDOWN_BORDER_HOVER: Color32 = gray(0x80);

    pub const TEXT: Color32 = gray(0xea);
    pub const TEXT_DIM: Color32 = gray(0xbc);
    pub const TEXT_DISABLED: Color32 = gray(0x7c);
    pub const ICON: Color32 = gray(0xd8);
    pub const ACCENT: Color32 = Color32::from_rgb(0x2c, 0x8b, 0xe8);
}

pub mod size {
    use super::pt;

    // Window frame, measured in Photoshop at 1:1 (points)
    /// Title bar including its 1 pt bottom line.
    pub const TITLE_BAR: f32 = pt(29.0);
    pub const OPTIONS_BAR: f32 = pt(33.0);
    /// Toolbar including its 3 pt dark right border.
    pub const TOOLBAR: f32 = pt(42.0);
    pub const STATUS_BAR: f32 = pt(16.0);
    /// Icon strip including the 3 pt border on its left and the divider on its right.
    pub const ICON_STRIP: f32 = pt(43.0);
    pub const PANEL_COLUMN: f32 = pt(322.0);

    // Measured on the reference screenshot (scaled by UI_SCALE)
    /// Panel tab bar: 27 pt of tabs and a 1 pt line under them.
    pub const PANEL_TAB_BAR: f32 = super::pt(28.0);
    pub const FIELD_HEIGHT: f32 = 26.0;
}

pub mod font {
    use super::pt;

    /// Matches the cap height of Photoshop's panel text (8 pt).
    pub const BODY: f32 = pt(11.5);
    pub const SMALL: f32 = pt(10.5);
    pub const ICON: f32 = 20.0;
}

/// Global UI scale (like Photoshop's UI Scaling preference).
/// Affects the UI only; canvas zoom is always in physical pixels.
pub const UI_SCALE: f32 = 0.675;

/// Converts a size measured in points on Photoshop's screen into egui units
/// (which [`UI_SCALE`] scales back down). Newer UI is measured directly in
/// Photoshop at 1:1, so it uses this instead of reference-screenshot pixels.
pub const fn pt(points: f32) -> f32 {
    points / UI_SCALE
}

/// Name of the semibold font family (panel tabs, headings).
pub const SEMIBOLD: &str = "semibold";

pub fn body() -> FontId {
    FontId::proportional(font::BODY)
}

pub fn small() -> FontId {
    FontId::proportional(font::SMALL)
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// Dialog text as AppKit draws it: the macOS system font (SF) with the
/// Text optical size, loaded at run time when present (see
/// [`install_fonts`]), else Source Sans 3. Photoshop's classic dialogs
/// (Duplicate Layer, Image Size), alerts and window titles use it.
pub fn dialog(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DIALOG.into()))
}

/// Bold [`dialog`] text (window titles).
pub fn dialog_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DIALOG_BOLD.into()))
}

/// Text in Photoshop's UXP dialogs (New Layer, New Group): Spectrum's
/// Adobe Clean, like the panels, so Source Sans 3 here. Matched on
/// Photoshop 2026's New Layer dialog (12 pt widths agree within 3%).
pub fn uxp(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Bold [`uxp`] text (the buttons): Adobe Clean Bold, Source Sans 3
/// Semibold here.
pub fn uxp_bold(size: f32) -> FontId {
    semibold(size)
}

/// The system font's tracking at `size` points (its `trak` table, normal
/// track, in 1/2048 em), which AppKit applies and egui doesn't: none at
/// 12 pt, tighter above (−12 at 13 pt, −22 at 14 pt). Returned as extra
/// letter spacing in egui points.
pub fn system_tracking(size: f32) -> f32 {
    const TRAK: [(f32, f32); 12] = [
        (9.0, 38.0),
        (10.0, 24.0),
        (11.0, 12.0),
        (12.0, 0.0),
        (13.0, -12.0),
        (14.0, -22.0),
        (15.0, -32.0),
        (16.0, -40.0),
        (17.0, -52.0),
        (20.0, -46.0),
        (22.0, -24.0),
        (24.0, 6.0),
    ];
    let units = match TRAK.iter().position(|&(s, _)| s >= size) {
        Some(0) => TRAK[0].1,
        None => TRAK[TRAK.len() - 1].1,
        Some(i) => {
            let ((s0, t0), (s1, t1)) = (TRAK[i - 1], TRAK[i]);
            t0 + (t1 - t0) * (size - s0) / (s1 - s0)
        }
    };
    pt(size * units / 2048.0)
}

/// Lays out `text` in a system-font `font` with the font's tracking, as
/// macOS draws it (see [`system_tracking`]).
pub fn tracked_galley(
    painter: &egui::Painter,
    text: &str,
    font: FontId,
    color: egui::Color32,
) -> std::sync::Arc<egui::Galley> {
    let size = font.size / pt(1.0);
    let mut job = egui::text::LayoutJob::default();
    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id: font,
            color,
            extra_letter_spacing: system_tracking(size),
            ..Default::default()
        },
    );
    painter.layout_job(job)
}

const DIALOG: &str = "dialog";
const DIALOG_BOLD: &str = "dialog-bold";
/// Where macOS keeps its system font, a variable font with a weight axis.
const SYSTEM_FONT: &str = "/System/Library/Fonts/SFNS.ttf";

pub fn icon(size: f32) -> FontId {
    FontId::proportional(size)
}

/// The toolbar's tool icons: Phosphor Regular, whose strokes (about 1 pt
/// at the toolbar's size) match Photoshop's; Bold's were half again as
/// heavy.
pub fn tool_icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("phosphor-regular".into()))
}

/// The bundled interface fonts, also the Type tool's fonts.
pub const SOURCE_SANS_REGULAR: &[u8] = include_bytes!("../assets/fonts/SourceSans3-Regular.ttf");
pub const SOURCE_SANS_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/SourceSans3-Semibold.ttf");

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "source-sans".into(),
        FontData::from_static(SOURCE_SANS_REGULAR).into(),
    );
    fonts.font_data.insert(
        "source-sans-semibold".into(),
        FontData::from_static(SOURCE_SANS_SEMIBOLD).into(),
    );

    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(0, "source-sans".into());

    // The icon font is appended to Proportional as a fallback
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    // The toolbar's icons on their own, without text fonts in front
    egui_phosphor::add_font_bytes_as_family(
        &mut fonts,
        "phosphor-regular",
        egui_phosphor::Variant::Regular.font_bytes(),
    );

    let mut semibold = vec!["source-sans-semibold".to_owned()];
    semibold.extend(
        fonts.families[&FontFamily::Proportional]
            .iter()
            .skip(1)
            .cloned(),
    );
    fonts
        .families
        .insert(FontFamily::Name(SEMIBOLD.into()), semibold);

    // Dialogs: the system font when it can be read (never bundled), with
    // the interface fonts behind it for anything it lacks
    let system = std::fs::read(SYSTEM_FONT).ok();
    // AppKit sets text under 17 pt with the Text optical size (opsz 17),
    // looser than the font's default Display (28)
    for (family, weight, opsz, fallback) in [
        (DIALOG, 400.0, 17.0, FontFamily::Proportional),
        (DIALOG_BOLD, 700.0, 17.0, FontFamily::Name(SEMIBOLD.into())),
    ] {
        let mut chain = Vec::new();
        if let Some(bytes) = &system {
            let tweak = egui::FontTweak {
                coords: egui::epaint::text::VariationCoords::new([
                    (b"wght", weight),
                    (b"opsz", opsz),
                ]),
                ..Default::default()
            };
            let key = format!("system-{family}");
            fonts.font_data.insert(
                key.clone(),
                FontData::from_owned(bytes.clone()).tweak(tweak).into(),
            );
            chain.push(key);
        }
        chain.extend(fonts.families[&fallback].iter().cloned());
        fonts
            .families
            .insert(FontFamily::Name(family.into()), chain);
    }

    ctx.set_fonts(fonts);
}

pub fn apply_style(ctx: &egui::Context) {
    use egui::{CornerRadius, TextStyle};

    ctx.set_theme(egui::Theme::Dark);
    ctx.set_zoom_factor(UI_SCALE);
    ctx.options_mut(|o| {
        // Keep Cmd +/- for canvas zoom instead of scaling the whole UI
        o.zoom_with_keyboard = false;
    });

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.text_styles = [
            (TextStyle::Heading, semibold(16.0)),
            (TextStyle::Body, body()),
            (TextStyle::Button, body()),
            (TextStyle::Small, small()),
            (TextStyle::Monospace, FontId::monospace(13.0)),
        ]
        .into();

        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
        style.spacing.interact_size.y = size::FIELD_HEIGHT;
        style.spacing.combo_height = 400.0;
        // Photoshop's lists don't fade out at their edges
        style.spacing.scroll.fade.strength = 0.0;

        let v = &mut style.visuals;
        v.dark_mode = true;
        v.override_text_color = Some(color::TEXT);
        v.panel_fill = color::PANEL;
        v.window_fill = color::PANEL;
        v.window_stroke = Stroke::new(1.0, color::SEPARATOR);
        v.window_corner_radius = CornerRadius::same(4);
        v.menu_corner_radius = CornerRadius::same(4);
        v.extreme_bg_color = color::FIELD;
        v.text_edit_bg_color = Some(color::FIELD);
        v.faint_bg_color = color::LIST_BG;
        v.selection.bg_fill = color::ACCENT;
        v.selection.stroke = Stroke::new(1.0, color::TEXT);

        let r = CornerRadius::same(3);
        let w = &mut v.widgets;
        w.noninteractive.bg_fill = color::PANEL;
        w.noninteractive.weak_bg_fill = color::PANEL;
        w.noninteractive.bg_stroke = Stroke::new(1.0, color::SEPARATOR_LIGHT);
        w.noninteractive.fg_stroke = Stroke::new(1.0, color::TEXT);

        w.inactive.bg_fill = color::FIELD;
        w.inactive.weak_bg_fill = color::FIELD;
        w.inactive.bg_stroke = Stroke::new(1.0, color::FIELD_BORDER);
        w.inactive.fg_stroke = Stroke::new(1.0, color::ICON);
        w.inactive.corner_radius = r;

        w.hovered.bg_fill = color::HOVER;
        w.hovered.weak_bg_fill = color::HOVER;
        w.hovered.bg_stroke = Stroke::new(1.0, Color32::from_gray(0x80));
        w.hovered.fg_stroke = Stroke::new(1.0, color::TEXT);
        w.hovered.corner_radius = r;
        w.hovered.expansion = 0.0;

        w.active.bg_fill = color::TOOL_ACTIVE;
        w.active.weak_bg_fill = color::TOOL_ACTIVE;
        w.active.bg_stroke = Stroke::new(1.0, Color32::from_gray(0x90));
        w.active.fg_stroke = Stroke::new(1.0, color::TEXT);
        w.active.corner_radius = r;
        w.active.expansion = 0.0;

        w.open = w.active;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Text widths (device pixels at 2x, ink plus about 3 px of side
    /// bearings) measured on Photoshop 2026's dialogs. They catch a lost
    /// optical size (opsz 17), lost tracking, or the wrong font for UXP
    /// dialogs.
    #[test]
    fn dialog_text_is_as_wide_as_photoshops() {
        if !std::path::Path::new(SYSTEM_FONT).exists() {
            return;
        }
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut out = ctx.run_ui(Default::default(), |_| {});
        out.textures_delta.clear();
        let width = |text: &str, font: FontId, tracked: bool| {
            let size = font.size / pt(1.0);
            let mut job = egui::text::LayoutJob::default();
            job.append(
                text,
                0.0,
                egui::TextFormat {
                    font_id: font,
                    extra_letter_spacing: if tracked { system_tracking(size) } else { 0.0 },
                    ..Default::default()
                },
            );
            ctx.fonts_mut(|f| f.layout_job(job)).size().x / pt(1.0) * 2.0
        };
        for (text, font, tracked, photoshop) in [
            // Image Size and Duplicate Layer: AppKit's 12 pt
            ("Resolution:", dialog(pt(12.0)), false, 125.0),
            ("Original Size", dialog(pt(12.0)), false, 142.0),
            ("Duplicate:", dialog(pt(12.0)), false, 113.0),
            // Window titles: 13 pt bold, tracked
            ("Duplicate Layer", dialog_bold(pt(13.0)), true, 202.0),
            ("Image Size", dialog_bold(pt(13.0)), true, 140.0),
            // Alerts: 13 pt, tracked
            ("Discard hidden layers?", dialog_bold(pt(13.0)), true, 293.0),
            ("Don\u{2019}t show again", dialog(pt(13.0)), true, 208.0),
            // New Layer (UXP): Adobe Clean, Source Sans 3 here
            ("Opacity", uxp(pt(12.0)), false, 77.0),
            ("Cancel", uxp_bold(pt(12.0)), false, 69.0),
        ] {
            let got = width(text, font, tracked);
            assert!(
                (got - photoshop).abs() <= photoshop * 0.03,
                "{text}: {got:.1} px, Photoshop {photoshop}"
            );
        }
    }
}
