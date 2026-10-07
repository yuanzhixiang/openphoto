//! Top options bar; its contents depend on the current tool.

use egui::{Align, Color32, Layout, Pos2, Rect, Sense, Ui, Vec2};
use op_tools::Tool;

use crate::commands::Command;
use crate::icons;
use crate::ps_icons::Icon;
use crate::state::{AppState, MarqueeStyle};
use crate::theme::{self, color, pt, size};
use crate::widgets;

/// Where the tool's own options start, right of the Tool Presets group.
const CONTENT_LEFT: f32 = pt(110.0);
/// The options bar's vertical center, from its top (Photoshop 2026).
const CENTER_Y: f32 = pt(17.0);

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let bar = Rect::from_min_size(
        ui.cursor().min,
        Vec2::new(ui.available_width(), size::OPTIONS_BAR),
    );
    let cy = bar.top() + CENTER_Y;
    let at = |x: f32, dy: f32| Pos2::new(bar.left() + pt(x), cy + pt(dy));
    let button = Vec2::splat(pt(24.0));
    let painter = ui.painter().clone();

    // The grip: a column of ten 2 × 1 pt dots
    for k in 0..10 {
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(bar.left() + pt(5.5), bar.top() + pt(8.0 + 2.0 * k as f32)),
                Vec2::new(pt(2.0), pt(1.0)),
            ),
            0,
            Color32::from_gray(0x45),
        );
    }
    // While transforming, Photoshop dims what can't be used
    let busy = app.transforming();
    let home = Rect::from_center_size(at(28.0, 0.0), button);
    if busy {
        crate::ps_icons::paint(
            &painter,
            home.center(),
            Icon::Home,
            color::OPTIONS_ICON_DISABLED,
            color::OPTIONS_BAR,
        );
    } else {
        ps_button(ui, home, Icon::Home).on_hover_text("Home");
    }
    separator(&painter, bar, 53.0);

    // Tool Presets: the current tool's icon and a chevron
    let preset = Rect::from_center_size(at(68.5, 0.5), button);
    let response = if busy {
        crate::ps_icons::paint(
            &painter,
            Pos2::new(at(68.0, 0.0).x, cy + pt(0.25)),
            Icon::TransformPreset,
            color::OPTIONS_ICON_DISABLED,
            color::OPTIONS_BAR,
        );
        ui.interact(preset, ui.id().with("tool-presets"), Sense::hover())
    } else if app.tool == Tool::Move {
        ps_button(ui, preset, Icon::Move)
    } else {
        let r = ui.interact(preset, ui.id().with("tool-presets"), Sense::click());
        if r.hovered() {
            painter.rect_filled(preset, 4, color::HOVER);
        }
        let bg = if r.hovered() {
            color::HOVER
        } else {
            color::OPTIONS_BAR
        };
        let center = Pos2::new(at(68.0, 0.0).x, cy);
        if !crate::tool_icons::paint_scaled(
            &painter,
            center,
            app.tool,
            color::OPTIONS_ICON,
            bg,
            1.0,
        ) {
            painter.text(
                preset.center(),
                egui::Align2::CENTER_CENTER,
                icons::tool(app.tool),
                theme::tool_icon(pt(17.5)),
                color::OPTIONS_ICON,
            );
        }
        r
    };
    response.on_hover_text("Tool Presets");
    let tint = if busy {
        color::OPTIONS_ICON_DISABLED
    } else {
        color::OPTIONS_ICON
    };
    crate::ps_icons::paint(
        &painter,
        at(90.25, 1.25),
        Icon::Caret,
        tint,
        color::OPTIONS_BAR,
    );
    separator(&painter, bar, 102.0);

    // The right end: the special buttons of a tool in progress, or the
    // app-wide buttons
    let right = Rect::from_min_max(Pos2::new(bar.right() - pt(230.0), bar.top()), bar.max);
    let special = app.typing_text();
    let cropping = app.tool == Tool::Crop && !app.transforming();
    if special {
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(right.shrink2(Vec2::new(pt(8.0), 0.0)))
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                type_buttons(ui, app);
            },
        );
    } else {
        // The Crop tool's bar is wider than the window: like Photoshop's, it
        // pushes these right (the avatar is cut off at 1350 pt)
        let push = if cropping {
            (pt(CROP_BAR_END) - bar.width()).max(0.0)
        } else {
            0.0
        };
        let from_right = |x: f32, dy: f32| Pos2::new(bar.right() + push - pt(x), cy + pt(dy));
        for (x, dy, icon, tip) in [
            (200.0, 0.0, Icon::Share, "Share"),
            (163.5, 0.5, Icon::Bell, "Notifications"),
            (131.0, 0.5, Icon::Search, "Search"),
            (98.5, -0.25, Icon::Lightbulb, "Discover"),
            (69.5, 1.0, Icon::Workspace, "Workspace"),
        ] {
            let tint = if icon == Icon::Bell {
                color::OPTIONS_BELL
            } else {
                color::OPTIONS_ICON
            };
            let rect = Rect::from_center_size(from_right(x, dy), button);
            if busy && matches!(icon, Icon::Share | Icon::Workspace) {
                crate::ps_icons::paint(
                    &painter,
                    rect.center(),
                    icon,
                    color::OPTIONS_ICON_DISABLED,
                    color::OPTIONS_BAR,
                );
            } else {
                ps_button_tinted(ui, rect, icon, tint).on_hover_text(tip);
            }
        }
        crate::ps_icons::paint(
            &painter,
            from_right(48.0, 1.25),
            Icon::Caret,
            if busy {
                color::OPTIONS_ICON_DISABLED
            } else {
                color::OPTIONS_ICON
            },
            color::OPTIONS_BAR,
        );
        // The account avatar (a placeholder)
        painter.circle_filled(
            from_right(21.5, 0.5),
            pt(12.0),
            Color32::from_rgb(0x4f, 0x8f, 0xd9),
        );
    }

    if cropping {
        crop_bar(ui, app, bar);
        ui.allocate_rect(bar, Sense::hover());
        return;
    }
    if app.transforming() {
        transform_bar(ui, app, bar);
        ui.allocate_rect(bar, Sense::hover());
        return;
    }
    if measured_bar(ui, app, bar) {
        ui.allocate_rect(bar, Sense::hover());
        return;
    }
    let content = Rect::from_min_max(
        Pos2::new(bar.left() + CONTENT_LEFT, bar.top()),
        Pos2::new(right.left(), bar.bottom()),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(content)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            tool_options(ui, app);
        },
    );
    ui.allocate_rect(bar, Sense::hover());
}

/// The tools whose options are laid out at Photoshop 2026's measured
/// places (`options_kit`); false for the others, laid out in a row.
fn measured_bar(ui: &mut Ui, app: &mut AppState, bar: Rect) -> bool {
    use crate::options_kit::Bar;
    let mut b = Bar::new(ui, bar);
    match app.tool {
        Tool::RectangularMarquee
        | Tool::EllipticalMarquee
        | Tool::SingleRowMarquee
        | Tool::SingleColumnMarquee
        | Tool::Lasso
        | Tool::PolygonalLasso
        | Tool::MagneticLasso => selection_bar(&mut b, app),
        Tool::MagicWand => wand_bar(&mut b, app),
        Tool::ObjectSelection => object_selection_bar(&mut b, app),
        Tool::QuickSelection => quick_selection_bar(&mut b, app),
        tool => match crate::options_tools::layout(tool) {
            Some(items) => crate::options_tools::show(&mut b, app, items),
            None => return false,
        },
    }
    if app.tool == Tool::PerspectiveCrop {
        perspective_commit_buttons(ui, app, bar);
    }
    true
}

/// The Perspective Crop tool's Cancel ⦸ and Commit ✓, at the Crop tool's
/// places, while it has a box.
fn perspective_commit_buttons(ui: &mut Ui, app: &mut AppState, bar: Rect) {
    let options = crate::perspective_crop::PerspectiveOptions::from_app(app);
    let Some(state) = app.active() else {
        return;
    };
    if !state
        .perspective_crop
        .as_ref()
        .is_some_and(|b| b.quad.is_some())
    {
        return;
    }
    let cy = bar.top() + CENTER_Y;
    let at = |x: f32| bar.left() + pt(x);
    let cancel = Rect::from_center_size(Pos2::new(at(1103.75), cy), Vec2::splat(pt(24.0)));
    if ps_button(ui, cancel, Icon::CropCancel)
        .on_hover_text("Cancel current crop operation (Esc)")
        .clicked()
    {
        state.perspective_crop = None;
    }
    let commit = Rect::from_center_size(Pos2::new(at(1138.5), cy), Vec2::splat(pt(24.0)));
    if ps_button(ui, commit, Icon::CropCommit)
        .on_hover_text("Commit current crop operation (Return)")
        .clicked()
    {
        crate::perspective_crop::commit(state, &options);
    }
}

/// A number with its unit, as the fields show it ("0 px", "10%").
fn with_unit(v: f32, unit: &str) -> String {
    let v = (v * 10.0).round() / 10.0;
    if unit.is_empty() {
        format!("{v}")
    } else if unit == "%" {
        format!("{v}%")
    } else {
        format!("{v} {unit}")
    }
}

/// The marquee and lasso tools' options (measured on Photoshop 2026):
/// the combine modes, Feather and Anti-alias; the marquees' Style with
/// Width and Height; the Magnetic Lasso's Width, Contrast, Frequency and
/// pen pressure; Select and Mask.
fn selection_bar(b: &mut crate::options_kit::Bar, app: &mut AppState) {
    let tool = app.tool;
    let marquee = matches!(
        tool,
        Tool::RectangularMarquee
            | Tool::EllipticalMarquee
            | Tool::SingleRowMarquee
            | Tool::SingleColumnMarquee
    );
    let single = matches!(tool, Tool::SingleRowMarquee | Tool::SingleColumnMarquee);
    let has_selection = app.active().is_some_and(|d| d.doc.selection().is_some());
    b.modes(110.0, &mut app.marquee.mode);
    b.sep(222.0);
    b.label(233.0, "Feather:", true);
    // The single row and column marquees' fields sit 2 pt further left
    let dx = if single { -2.0 } else { 0.0 };
    let shown = with_unit(app.marquee.feather, "px");
    if let Some(t) = b.value(276.0 + dx, 330.0 + dx, "feather", shown, true)
        && let Some(v) = typed_number(&t)
    {
        app.marquee.feather = v.clamp(0.0, 1000.0);
    }
    // Anti-alias only applies to curved edges
    let curved = !matches!(
        tool,
        Tool::RectangularMarquee | Tool::SingleRowMarquee | Tool::SingleColumnMarquee
    );
    // Photoshop shows it unchecked where it doesn't apply
    let mut shown = app.marquee.anti_alias && curved;
    b.check(338.0 + dx, "Anti-alias", &mut shown, curved);
    if curved {
        app.marquee.anti_alias = shown;
    }
    b.sep(412.0 + dx);
    if marquee {
        let fixed = !single && app.marquee.style != MarqueeStyle::Normal;
        b.label(421.5 + dx, "Style:", !single);
        let mut style = MarqueeStyle::ALL
            .iter()
            .position(|s| *s == app.marquee.style)
            .unwrap_or(0);
        b.choice(
            453.5 + dx,
            531.0 + dx,
            "marquee-style",
            &["Normal", "Fixed Ratio", "Fixed Size"],
            &mut style,
            !single,
        );
        app.marquee.style = MarqueeStyle::ALL[style];
        b.label(541.0 + dx, "Width:", fixed);
        let w = app.setting("marquee.width", "").clone();
        if let Some(t) = b.value(574.0 + dx, 615.5 + dx, "marquee-width", w, fixed) {
            *app.setting("marquee.width", "") = t;
        }
        b.icon(
            638.5 + dx,
            Icon::Swap,
            "Swap height and width",
            false,
            fixed,
        );
        b.label(663.5 + dx, "Height:", fixed);
        let h = app.setting("marquee.height", "").clone();
        if let Some(t) = b.value(699.5 + dx, 741.0 + dx, "marquee-height", h, fixed) {
            *app.setting("marquee.height", "") = t;
        }
        b.sep(749.0 + dx);
        b.button(760.5 + dx, 870.5 + dx, "Select and Mask...", has_selection);
    } else if tool == Tool::MagneticLasso {
        for (x, label, x0, x1, key, default) in [
            (422.5, "Width:", 457.5, 500.5, "lasso.width", "10 px"),
            (509.0, "Contrast:", 557.5, 600.5, "lasso.contrast", "10%"),
            (609.5, "Frequency:", 666.0, 693.0, "lasso.frequency", "57"),
        ] {
            b.label(x, label, true);
            let shown = app.setting(key, default).clone();
            if let Some(t) = b.value(x0, x1, key, shown, true) {
                *app.setting(key, default) = t;
            }
        }
        b.sep(701.0);
        let on = app.flag("lasso.pressure", false);
        if b.icon(
            725.0,
            Icon::PenPressure,
            "Use tablet pressure to change pen width",
            on,
            true,
        )
        .clicked()
        {
            app.set_flag("lasso.pressure", !on);
        }
        b.sep(748.0);
        b.button(759.5, 869.5, "Select and Mask...", has_selection);
    } else {
        b.button(423.5, 533.5, "Select and Mask...", has_selection);
    }
}

/// Select Subject (with its menu) from `x` and Select and Mask from
/// `mask_x`, at the bar's end.
fn subject_and_mask(b: &mut crate::options_kit::Bar, x: f32, mask_x: f32, has_selection: bool) {
    b.button(x, x + 91.0, "Select Subject", false);
    b.chevron_button(x + 92.0, x + 111.0, "select-subject-menu", true);
    b.button(mask_x, mask_x + 110.0, "Select and Mask...", has_selection);
}

/// A field of a setting nothing reads yet.
fn setting_field(
    b: &mut crate::options_kit::Bar,
    app: &mut AppState,
    (x0, x1): (f32, f32),
    key: &'static str,
    default: &str,
    enabled: bool,
) {
    let shown = app.setting(key, default).clone();
    if let Some(t) = b.value(x0, x1, key, shown, enabled) {
        *app.setting(key, default) = t;
    }
}

/// A checkbox of a setting nothing reads yet.
fn setting_check(
    b: &mut crate::options_kit::Bar,
    app: &mut AppState,
    x: f32,
    label: &str,
    key: &'static str,
    default: bool,
) {
    let mut on = app.flag(key, default);
    if b.check(x, label, &mut on, true).changed() {
        app.set_flag(key, on);
    }
}

/// A pop-up menu of a setting nothing reads yet.
fn setting_choice(
    b: &mut crate::options_kit::Bar,
    app: &mut AppState,
    (x0, x1): (f32, f32),
    key: &'static str,
    options: &[&str],
    enabled: bool,
) {
    let mut i: usize = app.setting(key, "0").parse().unwrap_or(0);
    b.choice(x0, x1, key, options, &mut i, enabled);
    *app.setting(key, "0") = i.to_string();
}

/// Magic Wand: the modes, Sample Size, Tolerance, Anti-alias, Contiguous,
/// Sample All Layers, Select Subject, Select and Mask.
fn wand_bar(b: &mut crate::options_kit::Bar, app: &mut AppState) {
    let has_selection = app.active().is_some_and(|d| d.doc.selection().is_some());
    b.modes(110.0, &mut app.wand.mode);
    b.sep(222.0);
    b.label(231.5, "Sample Size:", true);
    setting_choice(
        b,
        app,
        (298.0, 413.5),
        "wand.sample_size",
        &[
            "Point Sample",
            "3 by 3 Average",
            "5 by 5 Average",
            "11 by 11 Average",
            "31 by 31 Average",
            "51 by 51 Average",
            "101 by 101 Average",
        ],
        true,
    );
    b.label(422.5, "Tolerance:", true);
    let region = &mut app.wand.region;
    if let Some(t) = b.value(
        476.5,
        524.5,
        "wand-tolerance",
        region.tolerance.to_string(),
        true,
    ) && let Some(v) = typed_number(&t)
    {
        region.tolerance = v.clamp(0.0, 255.0) as u8;
    }
    b.check(532.5, "Anti-alias", &mut region.anti_alias, true);
    b.check(606.5, "Contiguous", &mut region.contiguous, true);
    b.check(690.0, "Sample All Layers", &mut region.all_layers, true);
    b.sep(803.5);
    subject_and_mask(b, 815.0, 945.0, has_selection);
}

/// Object Selection: the modes, Select people (its menu), refresh, show all
/// objects, settings, Mode (Rectangle / Lasso), Sample All Layers, Hard
/// Edge, feedback, Select Subject, Select and Mask.
fn object_selection_bar(b: &mut crate::options_kit::Bar, app: &mut AppState) {
    let has_selection = app.active().is_some_and(|d| d.doc.selection().is_some());
    b.modes(104.0, &mut app.marquee.mode);
    b.sep(212.0);
    b.menu_button(217.5, 305.5, "Select people");
    b.icon(323.0, Icon::Refresh, "Refresh object finder", false, true);
    let on = app.flag("object.finder", false);
    if b.icon(353.0, Icon::ObjectFinder, "Show all objects", on, true)
        .clicked()
    {
        app.set_flag("object.finder", !on);
    }
    b.icon(383.0, Icon::Gear, "Set additional options", false, true);
    b.sep(400.0);
    setting_choice(
        b,
        app,
        (405.0, 483.0),
        "object.mode",
        &["Rectangle", "Lasso"],
        true,
    );
    b.sep(487.0);
    setting_check(
        b,
        app,
        492.0,
        "Sample All Layers",
        "object.all_layers",
        false,
    );
    setting_check(b, app, 602.0, "Hard Edge", "object.hard_edge", true);
    b.sep(677.0);
    b.icon(696.0, Icon::Feedback, "Send feedback", false, true);
    b.sep(712.0);
    b.button(717.5, 808.5, "Select Subject", false);
    b.chevron_button(813.5, 832.5, "select-subject-menu", true);
    b.button(837.5, 947.5, "Select and Mask...", has_selection);
}

/// Quick Selection: its three modes, the brush, the angle, Sample All
/// Layers, Enhance Edge, Select Subject, Select and Mask.
fn quick_selection_bar(b: &mut crate::options_kit::Bar, app: &mut AppState) {
    let has_selection = app.active().is_some_and(|d| d.doc.selection().is_some());
    let mode: usize = app.setting("quick.mode", "0").parse().unwrap_or(0);
    for (k, (icon, tip)) in [
        (Icon::QuickNew, "New selection"),
        (Icon::QuickAdd, "Add to selection"),
        (Icon::QuickSubtract, "Subtract from selection"),
    ]
    .into_iter()
    .enumerate()
    {
        if b.icon(129.0 + 28.0 * k as f32, icon, tip, mode == k, true)
            .clicked()
        {
            *app.setting("quick.mode", "0") = k.to_string();
        }
    }
    b.sep(202.0);
    let size = app.setting("quick.size", "30").clone();
    b.brush_picker(221.5, &size, size.parse().unwrap_or(30.0), 1.0);
    b.sep(253.0);
    b.icon(266.0, Icon::Angle, "Set the brush angle", false, true);
    setting_field(b, app, (277.0, 318.5), "quick.angle", "0°", true);
    b.sep(322.5);
    setting_check(
        b,
        app,
        327.5,
        "Sample All Layers",
        "quick.all_layers",
        false,
    );
    setting_check(b, app, 437.0, "Enhance Edge", "quick.enhance_edge", false);
    b.sep(530.0);
    subject_and_mask(b, 543.5, 669.5, has_selection);
}

/// A 1 pt separator at `x` points from the bar's left, as in Photoshop.
fn separator(painter: &egui::Painter, bar: Rect, x: f32) {
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(bar.left() + pt(x), bar.top() + pt(6.0)),
            Vec2::new(pt(1.0), pt(22.5)),
        ),
        0,
        color::OPTIONS_SEPARATOR,
    );
}

fn ps_button(ui: &mut Ui, rect: Rect, icon: Icon) -> egui::Response {
    ps_button_tinted(ui, rect, icon, color::OPTIONS_ICON)
}

fn ps_button_tinted(ui: &mut Ui, rect: Rect, icon: Icon, tint: Color32) -> egui::Response {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        widgets::ps_icon_button(ui, rect.size(), icon, tint)
    })
    .inner
}

/// Marquee and Lasso tools: mode, Feather, Anti-alias; the marquees also
/// have Style with Width and Height.
fn tool_options(ui: &mut Ui, app: &mut AppState) {
    if app.tool == Tool::Move {
        move_options(ui, app);
    }
}

/// While typing: Cancel (Esc) and Commit (Cmd+Enter).
fn type_buttons(ui: &mut Ui, app: &mut AppState) {
    let Some(state) = app.active() else {
        return;
    };
    if widgets::icon_button(ui, icons::CHECK, 34.0, false)
        .on_hover_text("Commit any current edits")
        .clicked()
    {
        crate::type_tool::commit(state);
        return;
    }
    if widgets::icon_button(ui, icons::PROHIBIT, 34.0, false)
        .on_hover_text("Cancel any current edits")
        .clicked()
    {
        crate::type_tool::cancel(state);
    }
}

/// Where the Crop tool's bar would need the window to end so the app's
/// buttons sit at their usual place (Photoshop 2026: Share lands at
/// 1166.5 pt in a 1350 pt window).
const CROP_BAR_END: f32 = 1366.5;

/// The Crop tool's options, at Photoshop 2026's positions (points from the
/// bar's left): the ratio menu, Width, swap, Height (and in W x H x
/// Resolution the resolution with its unit), Clear, Straighten, the
/// overlay and gear menus, Delete Cropped Pixels, Fill, info and reset;
/// Cancel and Commit once the box changed.
fn crop_bar(ui: &mut Ui, app: &mut AppState, bar: Rect) {
    let mut chosen_user = false;
    let cy = bar.top() + CENTER_Y;
    let at = |x: f32| bar.left() + pt(x);
    let span = |x0: f32, x1: f32, h: f32| {
        Rect::from_min_max(
            Pos2::new(at(x0), cy - pt(h / 2.0)),
            Pos2::new(at(x1), cy + pt(h / 2.0)),
        )
    };
    let painter = ui.painter().clone();
    let background = app.background;
    let mut options = app.crop_options.clone();
    let sized = options.preset.sized();
    let user_name = options
        .user
        .and_then(|i| app.crop_presets.list().get(i))
        .map(|p| p.name.clone());
    let label = match &user_name {
        // Cut short like Photoshop's "W x H x Reso..."
        Some(name) if name.chars().count() > 15 => {
            format!("{}...", name.chars().take(12).collect::<String>())
        }
        Some(name) => name.clone(),
        None if options.preset == crate::crop_tool::CropPreset::SizeResolution => {
            "W x H x Reso...".to_owned()
        }
        None => options.preset.label().to_owned(),
    };
    // Photoshop's menu: the built-in groups, the saved presets, then New
    // and Delete Crop Preset...
    #[derive(Clone, Copy)]
    enum Pick {
        Builtin(crate::crop_tool::CropPreset),
        User(usize),
        New,
        Delete,
        Nothing,
    }
    let mut menu: Vec<(crate::native_popup::Entry, Pick)> = Vec::new();
    let sep = || (crate::native_popup::Entry::Separator, Pick::Nothing);
    for (g, group) in crate::crop_tool::PRESET_GROUPS.into_iter().enumerate() {
        if g > 0 {
            menu.push(sep());
        }
        for &p in group {
            let on = options.user.is_none() && p == options.preset;
            menu.push((
                crate::native_popup::Entry::item(p.label(), on),
                Pick::Builtin(p),
            ));
        }
    }
    if !app.crop_presets.list().is_empty() {
        menu.push(sep());
        for (i, p) in app.crop_presets.list().iter().enumerate() {
            let on = options.user == Some(i);
            menu.push((
                crate::native_popup::Entry::item(p.name.clone(), on),
                Pick::User(i),
            ));
        }
    }
    menu.push(sep());
    menu.push((
        crate::native_popup::Entry::item("New Crop Preset...", false),
        Pick::New,
    ));
    menu.push((
        crate::native_popup::Entry::item("Delete Crop Preset...", false)
            .enabled(!app.crop_presets.list().is_empty()),
        Pick::Delete,
    ));
    let entries: Vec<_> = menu.iter().map(|(e, _)| e.clone()).collect();
    let mut picked = None;
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(span(110.0, 201.0, 17.0)),
        |ui| {
            picked = widgets::dropdown_entries(ui, "crop-preset", pt(91.0), &label, true, &entries)
                .map(|k| menu[k].1);
        },
    );
    let mut chosen = None;
    match picked {
        Some(Pick::Builtin(p)) => chosen = Some(p),
        Some(Pick::User(i)) => {
            if let Some(preset) = app.crop_presets.list().get(i).cloned() {
                options.choose_user(&preset, i);
                // Refit the box like any other ratio or size
                chosen_user = true;
            }
        }
        Some(Pick::New) => {
            app.new_crop_preset = Some(crate::dialogs::new_preset::NewPresetDialog::new(
                "New Crop Preset",
                options.preset_name(),
            ));
        }
        Some(Pick::Delete) => {
            let names = app
                .crop_presets
                .list()
                .iter()
                .map(|p| p.name.clone())
                .collect();
            app.delete_crop_preset =
                Some(crate::dialogs::size_presets::DeletePresetDialog::titled(
                    "Delete Crop Preset",
                    names,
                ));
        }
        Some(Pick::Nothing) | None => {}
    }
    // Width, swap, Height
    let w = widgets::text_box(
        ui,
        span(205.0, 272.5, 17.0),
        &mut options.width,
        "crop-w",
        true,
    );
    let swap = Rect::from_center_size(Pos2::new(at(290.25), cy - pt(0.75)), Vec2::splat(pt(24.0)));
    let swapped = ps_button(ui, swap, Icon::Swap)
        .on_hover_text("Swap height and width")
        .clicked();
    let h = widgets::text_box(
        ui,
        span(310.0, 376.0, 17.0),
        &mut options.height,
        "crop-h",
        true,
    );
    // The rest moves left when there's no resolution
    let dx = if sized { 0.0 } else { -121.0 };
    let mut res_changed = false;
    if sized {
        separator(&painter, bar, 380.0);
        res_changed = widgets::text_box(
            ui,
            span(385.0, 439.5, 17.0),
            &mut options.resolution,
            "crop-res",
            true,
        )
        .changed();
        let unit = if options.per_cm { "px/cm" } else { "px/in" };
        let units = [
            crate::native_popup::Entry::item("px/in", !options.per_cm),
            crate::native_popup::Entry::item("px/cm", options.per_cm),
        ];
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(span(442.5, 496.5, 17.0)),
            |ui| {
                if let Some(k) =
                    widgets::dropdown_entries(ui, "crop-res-unit", pt(54.0), unit, true, &units)
                {
                    options.per_cm = k == 1;
                }
            },
        );
        separator(&painter, bar, 500.5);
    }
    let edited = w.changed() || h.changed() || res_changed;
    // Clear: a field-like button (#454545, a #666666 border)
    let clear_rect = span(506.0 + dx, 552.5 + dx, 24.0);
    let clear_response = ui.interact(clear_rect, ui.id().with("crop-clear"), Sense::click());
    let fill = if clear_response.is_pointer_button_down_on() {
        color::TOOL_ACTIVE
    } else if clear_response.hovered() {
        color::HOVER
    } else {
        color::FIELD
    };
    painter.rect(
        clear_rect,
        egui::CornerRadius::same(pt(2.0) as u8),
        fill,
        egui::Stroke::new(pt(1.0), Color32::from_gray(0x66)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        clear_rect.center() - Vec2::new(0.0, pt(0.5)),
        egui::Align2::CENTER_CENTER,
        "Clear",
        theme::body(),
        color::TEXT,
    );
    let clear = clear_response.clicked();

    // Straighten, overlay, gear
    let icon_button = |ui: &mut Ui, x: f32, dy: f32, icon: Icon, tip: &str| {
        let rect =
            Rect::from_center_size(Pos2::new(at(x + dx), cy + pt(dy)), Vec2::splat(pt(24.0)));
        ps_button(ui, rect, icon).on_hover_text(tip)
    };
    // Straighten: on while waiting for the line, as in Photoshop
    let straighten_rect = Rect::from_center_size(
        Pos2::new(at(574.0 + dx), cy - pt(1.75)),
        Vec2::new(pt(80.0), pt(24.0)),
    );
    if options.straightening {
        painter.rect_filled(
            straighten_rect.translate(Vec2::new(pt(28.0), 0.0)),
            pt(3.0),
            color::TOOL_ACTIVE,
        );
    }
    let label_click = ui
        .interact(
            Rect::from_min_max(
                Pos2::new(at(587.0 + dx), cy - pt(10.0)),
                Pos2::new(at(642.0 + dx), cy + pt(10.0)),
            ),
            ui.id().with("straighten-label"),
            Sense::click(),
        )
        .clicked();
    if icon_button(
        ui,
        574.0,
        -1.75,
        Icon::Straighten,
        "Straighten the image by drawing a line on it",
    )
    .clicked()
        || label_click
    {
        options.straightening = !options.straightening;
    }
    painter.text(
        Pos2::new(at(592.0 + dx), cy - pt(0.5)),
        egui::Align2::LEFT_CENTER,
        "Straighten",
        theme::body(),
        color::TEXT,
    );
    separator(&painter, bar, 649.5 + dx);
    let overlay = icon_button(
        ui,
        670.75,
        -0.25,
        Icon::CropOverlay,
        "Set the overlay options for the Crop Tool",
    );
    {
        use crate::crop_tool::{Overlay, OverlayShow};
        use crate::native_popup::Entry;
        let shows = [
            (OverlayShow::Auto, "Auto Show Overlay"),
            (OverlayShow::Always, "Always Show Overlay"),
            (OverlayShow::Never, "Never Show Overlay"),
        ];
        let mut entries: Vec<Entry> = Overlay::ALL
            .iter()
            .map(|&o| Entry::item(o.label(), o == options.overlay))
            .collect();
        entries.push(Entry::Separator);
        entries.extend(
            shows
                .iter()
                .map(|&(s, label)| Entry::item(label, s == options.overlay_show)),
        );
        entries.push(Entry::Separator);
        entries.push(Entry::item("Cycle Overlay", false));
        entries.push(Entry::item("Cycle Orientation", false).enabled(false));
        let picked = crate::native_popup::dropdown(
            ui,
            &overlay,
            ui.id().with("crop-overlay-menu"),
            &entries,
        );
        let n = Overlay::ALL.len();
        match picked {
            Some(k) if k < n => options.overlay = Overlay::ALL[k],
            Some(k) if (n + 1..n + 4).contains(&k) => options.overlay_show = shows[k - n - 1].0,
            Some(k) if k == n + 5 => {
                let i = Overlay::ALL
                    .iter()
                    .position(|&o| o == options.overlay)
                    .unwrap_or(0);
                options.overlay = Overlay::ALL[(i + 1) % n];
            }
            _ => {}
        }
    }
    let gear = icon_button(ui, 705.0, -0.75, Icon::Gear, "Set additional Crop options");
    let mut pick_shield = false;
    egui::Popup::menu(&gear)
        .id(ui.id().with("crop-gear-menu"))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.checkbox(&mut options.classic, "Use Classic Mode");
            ui.checkbox(&mut options.show_cropped_area, "Show Cropped Area");
            ui.add_enabled(
                !options.classic,
                egui::Checkbox::new(&mut options.auto_center, "Auto Center Preview"),
            );
            ui.separator();
            ui.checkbox(&mut options.shield, "Enable Crop Shield");
            ui.add_enabled_ui(options.shield, |ui| {
                // Match Canvas, or a custom color picked in the Color
                // Picker (also from the swatch)
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    let custom = options.shield_color.is_some();
                    let label = if custom { "Custom" } else { "Match Canvas" };
                    let entries = [
                        crate::native_popup::Entry::item("Match Canvas", !custom),
                        crate::native_popup::Entry::item("Custom", custom),
                    ];
                    let response = ui.button(label);
                    match crate::native_popup::dropdown(
                        ui,
                        &response,
                        ui.id().with("crop-shield-color"),
                        &entries,
                    ) {
                        Some(0) => options.shield_color = None,
                        Some(_) => pick_shield = true,
                        None => {}
                    }
                    let swatch = options.shield_color.unwrap_or(color::PASTEBOARD);
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(pt(17.0)), Sense::click());
                    ui.painter().rect(
                        rect,
                        0.0,
                        swatch,
                        egui::Stroke::new(pt(1.0), Color32::from_gray(0x66)),
                        egui::StrokeKind::Inside,
                    );
                    if response.clicked() {
                        pick_shield = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Opacity:");
                    let mut percent = (options.shield_opacity * 100.0).round();
                    ui.add(
                        egui::DragValue::new(&mut percent)
                            .range(0.0..=100.0)
                            .suffix("%"),
                    );
                    options.shield_opacity = percent / 100.0;
                });
                ui.checkbox(&mut options.auto_adjust_opacity, "Auto Adjust Opacity");
            });
        });
    separator(&painter, bar, 726.5 + dx);
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(span(735.0 + dx, 870.0 + dx, 20.0).translate(Vec2::new(0.0, pt(1.5)))),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            widgets::checkbox(ui, &mut options.delete_cropped, "Delete Cropped Pixels")
        },
    );
    painter.text(
        Pos2::new(at(870.5 + dx), cy),
        egui::Align2::LEFT_CENTER,
        "Fill:",
        theme::body(),
        color::TEXT,
    );
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(span(892.0 + dx, 1023.5 + dx, 17.0)),
        |ui| {
            use crate::crop_tool::CropFill;
            use crate::native_popup::Entry;
            // Both need Adobe's services or a content-aware engine
            let fills = [
                Entry::item(
                    CropFill::Background.label(),
                    options.fill == CropFill::Background,
                ),
                Entry::item(CropFill::GenerativeExpand.label(), false).enabled(false),
                Entry::item(CropFill::ContentAware.label(), false).enabled(false),
            ];
            if widgets::dropdown_entries(
                ui,
                "crop-fill",
                pt(131.5),
                options.fill.label(),
                true,
                &fills,
            ) == Some(0)
            {
                options.fill = CropFill::Background;
            }
        },
    );
    icon_button(
        ui,
        1042.0,
        -0.5,
        Icon::Info,
        "Learn more about the Crop tool",
    );

    if pick_shield {
        app.crop_options = options.clone();
        app.open_color_picker(crate::state::PickerTarget::CropShield);
    }
    let Some(state) = app.active() else {
        app.crop_options = options;
        return;
    };
    if let Some(p) = chosen {
        options.choose(p);
        if p == crate::crop_tool::CropPreset::FrontImage {
            options.front_image(state.doc.width, state.doc.height, state.doc.resolution);
        }
    }
    if swapped || clear || edited {
        options.user = None;
    }
    if swapped {
        options.swap();
    }
    if clear {
        options.clear();
    }
    // A new ratio or size refits the box to the image
    if (chosen.is_some() || chosen_user || swapped || edited)
        && let Some(a) = options.aspect((state.doc.width, state.doc.height))
    {
        state.crop = Some(crate::crop_tool::fitted(state, a));
    }
    let modified = crate::crop_tool::modified(state);
    let reset = Rect::from_center_size(Pos2::new(at(1070.5), cy - pt(1.5)), Vec2::splat(pt(24.0)));
    if modified {
        if ps_button(ui, reset, Icon::CropReset)
            .on_hover_text("Reset crop box, image rotation and aspect ratio settings")
            .clicked()
        {
            state.crop = Some(crate::crop_tool::full(state));
            crate::crop_tool::center_on_box(state, ui.ctx().pixels_per_point());
            options = crate::crop_tool::CropOptions {
                preset: crate::crop_tool::CropPreset::SizeResolution,
                width: String::new(),
                height: String::new(),
                resolution: String::new(),
                ..options
            };
        }
        let cancel = Rect::from_center_size(Pos2::new(at(1103.75), cy), Vec2::splat(pt(24.0)));
        if ps_button(ui, cancel, Icon::CropCancel)
            .on_hover_text("Cancel current crop operation (Esc)")
            .clicked()
        {
            state.crop = Some(crate::crop_tool::full(state));
            crate::crop_tool::center_on_box(state, ui.ctx().pixels_per_point());
        }
        let commit = Rect::from_center_size(Pos2::new(at(1138.5), cy), Vec2::splat(pt(24.0)));
        if ps_button(ui, commit, Icon::CropCommit)
            .on_hover_text("Commit current crop operation (Return)")
            .clicked()
        {
            crate::crop_tool::commit(state, &options, background);
            crate::crop_tool::center_on_box(state, ui.ctx().pixels_per_point());
        }
    } else {
        crate::ps_icons::paint(
            &painter,
            reset.center(),
            Icon::CropReset,
            color::OPTIONS_ICON_DISABLED,
            color::OPTIONS_BAR,
        );
    }
    app.crop_options = options;
}

/// An options-bar number box that keeps what's typed while it has the
/// focus and hands it over when Enter is pressed or the focus leaves.
fn value_box(ui: &mut Ui, rect: Rect, id: &str, shown: String, enabled: bool) -> Option<String> {
    let key = ui.id().with(("value-box", id));
    let mut text = ui
        .data(|d| d.get_temp::<String>(key))
        .unwrap_or_else(|| shown.clone());
    let response = widgets::text_box(ui, rect, &mut text, key, enabled);
    if response.has_focus() {
        ui.data_mut(|d| d.insert_temp(key, text.clone()));
        None
    } else {
        ui.data_mut(|d| d.remove::<String>(key));
        if response.lost_focus() {
            // The Enter that ends the typing isn't the canvas's (it would
            // commit the transform)
            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        }
        response.lost_focus().then_some(text)
    }
}

/// A number typed with or without its unit ("60 px", "50%", "15°").
pub fn typed_number(text: &str) -> Option<f32> {
    // The number up to its unit (px, pt, %, °, …)
    let t = text.trim();
    let end = t
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || c == '.' || (i == 0 && (c == '-' || c == '+'))))
        .map_or(t.len(), |(i, _)| i);
    t[..end].parse().ok()
}

/// Free Transform's options, at Photoshop 2026's positions (points from
/// the bar's left): the reference point switch and grid, X and Y with
/// the relative switch, W and H with the link, the angle, the H and V
/// skew, Interpolation, the Warp switch, Cancel and Commit.
fn transform_bar(ui: &mut Ui, app: &mut AppState, bar: Rect) {
    let cy = bar.top() + CENTER_Y;
    let at = |x: f32| bar.left() + pt(x);
    let span = |x0: f32, x1: f32| {
        Rect::from_min_max(
            Pos2::new(at(x0), cy - pt(8.5)),
            Pos2::new(at(x1), cy + pt(8.5)),
        )
    };
    let painter = ui.painter().clone();
    let label = |x: f32, text: &str| {
        painter.text(
            Pos2::new(at(x), cy),
            egui::Align2::LEFT_CENTER,
            text,
            theme::body(),
            color::TEXT,
        );
    };
    let Some(state) = app.active() else {
        return;
    };
    let Some(t) = state.free_transform.as_mut() else {
        return;
    };
    if t.mode == crate::state::TransformMode::Warp {
        warp_bar(ui, app, bar);
        return;
    }
    let editable = t.quad.is_none();

    // The reference point: a checkbox, then a 3 × 3 grid (dim when off)
    let check = Rect::from_center_size(Pos2::new(at(120.25), cy + pt(0.25)), Vec2::splat(pt(13.5)));
    if ui
        .interact(check, ui.id().with("ref-check"), Sense::click())
        .on_hover_text("Toggle reference point")
        .clicked()
    {
        t.show_reference = !t.show_reference;
    }
    painter.rect_stroke(
        check,
        egui::CornerRadius::same(pt(3.0) as u8),
        egui::Stroke::new(pt(1.0), color::OPTIONS_ICON),
        egui::StrokeKind::Inside,
    );
    if t.show_reference {
        crate::ps_icons::paint(
            &painter,
            check.center(),
            Icon::CropCommit,
            color::OPTIONS_ICON,
            color::OPTIONS_BAR,
        );
    }
    let grid_tint = if t.show_reference {
        Color32::from_gray(0xb0)
    } else {
        Color32::from_gray(0x60)
    };
    for gy in -1i8..=1 {
        for gx in -1i8..=1 {
            let c = Pos2::new(at(146.0 + 7.0 * gx as f32), cy + pt(0.75 + 7.0 * gy as f32));
            let cell = Rect::from_center_size(c, Vec2::splat(pt(5.5)));
            let chosen = t.reference == (gx, gy);
            if chosen {
                painter.rect_filled(cell, 0, grid_tint);
            } else {
                painter.rect_stroke(
                    cell,
                    0,
                    egui::Stroke::new(pt(1.5), grid_tint),
                    egui::StrokeKind::Inside,
                );
            }
            if t.show_reference
                && ui
                    .interact(
                        cell.expand(pt(1.0)),
                        ui.id().with(("ref", gx, gy)),
                        Sense::click(),
                    )
                    .clicked()
            {
                t.reference = (gx, gy);
            }
        }
    }

    // X, Y
    let now = t.reference_now();
    let start = t.reference_point();
    let (xv, yv) = if t.relative {
        (now.0 - start.0, now.1 - start.1)
    } else {
        now
    };
    label(163.0, "X:");
    if let Some(v) = value_box(
        ui,
        span(174.5, 235.5),
        "tx",
        format!("{xv:.2} px"),
        editable,
    )
    .and_then(|s| typed_number(&s))
    {
        t.offset.0 += v - xv;
    }
    let triangle =
        Rect::from_center_size(Pos2::new(at(250.75), cy + pt(0.25)), Vec2::splat(pt(20.0)));
    if ui
        .interact(triangle, ui.id().with("relative"), Sense::click())
        .on_hover_text("Use relative positioning for reference point")
        .clicked()
    {
        t.relative = !t.relative;
    }
    if t.relative {
        painter.rect_filled(triangle, pt(3.0), color::TOOL_ACTIVE);
    }
    painter.add(egui::Shape::closed_line(
        vec![
            triangle.center() + Vec2::new(0.0, pt(-5.0)),
            triangle.center() + Vec2::new(pt(5.25), pt(4.5)),
            triangle.center() + Vec2::new(pt(-5.25), pt(4.5)),
        ],
        egui::Stroke::new(pt(1.0), color::OPTIONS_ICON),
    ));
    label(268.5, "Y:");
    if let Some(v) = value_box(
        ui,
        span(279.5, 340.0),
        "ty",
        format!("{yv:.2} px"),
        editable,
    )
    .and_then(|s| typed_number(&s))
    {
        t.offset.1 += v - yv;
    }
    separator(&painter, bar, 343.5);

    // W, link, H
    label(349.0, "W:");
    let (w, h) = (t.scale.0 * 100.0, t.scale.1 * 100.0);
    if let Some(v) = value_box(ui, span(362.5, 417.0), "tw", format!("{w:.2}%"), editable)
        .and_then(|s| typed_number(&s))
        .filter(|v| *v != 0.0)
    {
        let linked = t.linked;
        t.pivoting(|t| {
            let k = v / 100.0 / t.scale.0;
            t.scale.0 = v / 100.0;
            if linked {
                t.scale.1 *= k;
            }
        });
    }
    let link = Rect::from_min_max(
        Pos2::new(at(420.0), bar.top() + pt(4.0)),
        Pos2::new(at(446.0), bar.top() + pt(31.0)),
    );
    if ui
        .interact(link, ui.id().with("link-wh"), Sense::click())
        .on_hover_text("Maintain aspect ratio")
        .clicked()
    {
        t.linked = !t.linked;
    }
    if t.linked {
        painter.rect(
            link,
            egui::CornerRadius::same(pt(3.0) as u8),
            Color32::from_gray(0x38),
            egui::Stroke::new(pt(1.0), Color32::from_gray(0x63)),
            egui::StrokeKind::Inside,
        );
    }
    crate::ps_icons::paint(
        &painter,
        Pos2::new(at(433.0), cy + pt(0.25)),
        Icon::LinkLayers,
        color::OPTIONS_ICON,
        color::OPTIONS_BAR,
    );
    label(450.5, "H:");
    if let Some(v) = value_box(ui, span(463.0, 517.0), "th", format!("{h:.2}%"), editable)
        .and_then(|s| typed_number(&s))
        .filter(|v| *v != 0.0)
    {
        let linked = t.linked;
        t.pivoting(|t| {
            let k = v / 100.0 / t.scale.1;
            t.scale.1 = v / 100.0;
            if linked {
                t.scale.0 *= k;
            }
        });
    }
    separator(&painter, bar, 520.5);

    // The angle
    let a = Pos2::new(at(532.25), cy + pt(0.75));
    let stroke = egui::Stroke::new(pt(1.0), color::OPTIONS_ICON);
    painter.line_segment(
        [
            a + Vec2::new(pt(-6.0), pt(5.5)),
            a + Vec2::new(pt(6.0), pt(5.5)),
        ],
        stroke,
    );
    painter.line_segment(
        [
            a + Vec2::new(pt(-6.0), pt(5.5)),
            a + Vec2::new(pt(5.0), pt(-5.5)),
        ],
        stroke,
    );
    let degrees = t.angle.to_degrees();
    if let Some(v) = value_box(
        ui,
        span(542.5, 597.0),
        "ta",
        format!("{degrees:.2}"),
        editable,
    )
    .and_then(|s| typed_number(&s))
    {
        t.pivoting(|t| t.angle = v.to_radians());
    }
    label(599.5, "°");
    separator(&painter, bar, 609.5);

    // Skew
    label(617.0, "H:");
    let (hs, vs) = (t.skew.0.to_degrees(), t.skew.1.to_degrees());
    if let Some(v) = value_box(ui, span(629.5, 677.5), "tsh", format!("{hs:.2}"), editable)
        .and_then(|s| typed_number(&s))
        .filter(|v| v.abs() < 90.0)
    {
        t.pivoting(|t| t.skew.0 = v.to_radians());
    }
    label(680.0, "°");
    label(691.5, "V:");
    if let Some(v) = value_box(ui, span(702.5, 750.5), "tsv", format!("{vs:.2}"), editable)
        .and_then(|s| typed_number(&s))
        .filter(|v| v.abs() < 90.0)
    {
        t.pivoting(|t| t.skew.1 = v.to_radians());
    }
    label(753.0, "°");
    separator(&painter, bar, 763.5);

    // Interpolation
    label(770.0, "Interpolation:");
    let mut how = t.interpolation;
    ui.scope_builder(egui::UiBuilder::new().max_rect(span(839.5, 901.5)), |ui| {
        widgets::dropdown_with(
            ui,
            "transform-interpolation",
            pt(62.0),
            how.label(),
            true,
            |ui| {
                for m in op_core::transform::Interpolation::ALL {
                    ui.selectable_value(&mut how, m, m.label());
                }
            },
        );
    });
    t.interpolation = how;
    // Switch to Warp
    let toggle = Rect::from_center_size(Pos2::new(at(919.0), cy), Vec2::new(pt(24.0), pt(24.0)));
    if ps_button(ui, toggle, Icon::WarpToggle)
        .on_hover_text("Switch between free transform and warp modes")
        .clicked()
    {
        crate::free_transform::set_mode(t, crate::state::TransformMode::Warp);
    }
    separator(&painter, bar, 935.0);

    let cancel = Rect::from_center_size(Pos2::new(at(983.0), cy), Vec2::splat(pt(24.0)));
    if ps_button(ui, cancel, Icon::CropCancel)
        .on_hover_text("Cancel transform (Esc)")
        .clicked()
    {
        crate::free_transform::cancel(state);
        return;
    }
    let commit = Rect::from_center_size(Pos2::new(at(1011.5), cy), Vec2::splat(pt(24.0)));
    if ps_button(ui, commit, Icon::CropCommit)
        .on_hover_text("Commit transform (Return)")
        .clicked()
        && let Some(crate::free_transform::Outcome::Committed(m)) =
            crate::free_transform::commit(state)
    {
        app.last_transform = Some(m);
    }
}

/// Free Transform's bar while warping (Photoshop 2026's positions): Split
/// and Grid, the Warp style with its orientation and Bend / H / V, the
/// Warp switch (on), reset, Cancel and Commit. Only the Custom style is
/// here: Split, Grid and the styles are drawn dim.
fn warp_bar(ui: &mut Ui, app: &mut AppState, bar: Rect) {
    let cy = bar.top() + CENTER_Y;
    let at = |x: f32| bar.left() + pt(x);
    let span = |x0: f32, x1: f32| {
        Rect::from_min_max(
            Pos2::new(at(x0), cy - pt(8.5)),
            Pos2::new(at(x1), cy + pt(8.5)),
        )
    };
    let painter = ui.painter().clone();
    let dim = color::OPTIONS_ICON_DISABLED;
    let label = |x: f32, text: &str, c: Color32| {
        painter.text(
            Pos2::new(at(x), cy),
            egui::Align2::LEFT_CENTER,
            text,
            theme::body(),
            c,
        );
    };
    // The reference point switch and grid stay, dim
    let check = Rect::from_center_size(Pos2::new(at(120.25), cy + pt(0.25)), Vec2::splat(pt(13.5)));
    painter.rect_stroke(
        check,
        egui::CornerRadius::same(pt(3.0) as u8),
        egui::Stroke::new(pt(1.0), Color32::from_gray(0x60)),
        egui::StrokeKind::Inside,
    );
    separator(&painter, bar, 161.5);
    label(171.5, "Split:", color::TEXT);
    let stroke = egui::Stroke::new(pt(1.0), dim);
    for (x, kind) in [(208.5, 0), (234.0, 1), (260.0, 2)] {
        let r = Rect::from_center_size(Pos2::new(at(x), cy), Vec2::splat(pt(11.0)));
        painter.rect_stroke(r, 0, stroke, egui::StrokeKind::Inside);
        let c = r.center();
        if kind != 2 {
            painter.line_segment(
                [Pos2::new(c.x, r.top()), Pos2::new(c.x, r.bottom())],
                stroke,
            );
        }
        if kind != 1 {
            painter.line_segment(
                [Pos2::new(r.left(), c.y), Pos2::new(r.right(), c.y)],
                stroke,
            );
        }
    }
    separator(&painter, bar, 282.0);
    label(290.5, "Grid:", color::TEXT);
    ui.scope_builder(egui::UiBuilder::new().max_rect(span(319.5, 378.5)), |ui| {
        widgets::dropdown_with(ui, "warp-grid", pt(59.0), "Default", false, |_| {});
    });
    separator(&painter, bar, 386.5);
    label(395.0, "Warp:", color::TEXT);
    ui.scope_builder(egui::UiBuilder::new().max_rect(span(428.5, 518.5)), |ui| {
        widgets::dropdown_with(ui, "warp-style", pt(90.0), "Custom", true, |ui| {
            let _ = ui.selectable_label(true, "Custom");
            ui.separator();
            for style in [
                "Arc",
                "Arc Lower",
                "Arc Upper",
                "Arch",
                "Bulge",
                "Shell Lower",
                "Shell Upper",
                "Flag",
                "Wave",
                "Fish",
                "Rise",
                "Fisheye",
                "Inflate",
                "Squeeze",
                "Twist",
            ] {
                ui.add_enabled(false, egui::Button::new(style));
            }
        });
    });
    crate::ps_icons::paint(
        &painter,
        Pos2::new(at(540.0), cy),
        Icon::Gear,
        dim,
        color::OPTIONS_BAR,
    );
    crate::ps_icons::paint(
        &painter,
        Pos2::new(at(575.0), cy),
        Icon::WarpToggle,
        dim,
        color::OPTIONS_BAR,
    );
    let dim_text = Color32::from_gray(0x87);
    for (lx, text, x0, x1, px) in [
        (600.5, "Bend:", 631.0, 678.0, 685.0),
        (700.0, "H:", 713.0, 760.0, 763.0),
        (779.0, "V:", 793.0, 840.0, 843.0),
    ] {
        label(lx, text, dim_text);
        let mut zero = "0.0".to_string();
        widgets::text_box(ui, span(x0, x1), &mut zero, ("warp-num", text), false);
        label(px, "%", dim_text);
    }
    let Some(state) = app.active() else {
        return;
    };
    let Some(t) = state.free_transform.as_mut() else {
        return;
    };
    // The Warp switch, on
    let toggle = Rect::from_min_max(
        Pos2::new(at(907.0), bar.top() + pt(4.0)),
        Pos2::new(at(931.0), bar.top() + pt(31.0)),
    );
    painter.rect(
        toggle,
        egui::CornerRadius::same(pt(3.0) as u8),
        Color32::from_gray(0x38),
        egui::Stroke::new(pt(1.0), Color32::from_gray(0x63)),
        egui::StrokeKind::Inside,
    );
    if ui
        .interact(toggle, ui.id().with("warp-off"), Sense::click())
        .on_hover_text("Switch between free transform and warp modes")
        .clicked()
    {
        t.mode = crate::state::TransformMode::Free;
    }
    crate::ps_icons::paint(
        &painter,
        toggle.center(),
        Icon::WarpToggle,
        color::OPTIONS_ICON,
        color::OPTIONS_BAR,
    );
    separator(&painter, bar, 935.0);
    let reset = Rect::from_center_size(Pos2::new(at(951.5), cy - pt(1.5)), Vec2::splat(pt(24.0)));
    if ps_button(ui, reset, Icon::CropReset)
        .on_hover_text("Reset warp")
        .clicked()
    {
        let m = t.mapping();
        let mut mesh = op_core::transform::WarpMesh::flat(t.bounds);
        for p in &mut mesh.points {
            *p = m.apply(*p);
        }
        t.warp = Some(mesh);
    }
    let cancel = Rect::from_center_size(Pos2::new(at(983.0), cy), Vec2::splat(pt(24.0)));
    if ps_button(ui, cancel, Icon::CropCancel)
        .on_hover_text("Cancel transform (Esc)")
        .clicked()
    {
        crate::free_transform::cancel(state);
        return;
    }
    let commit = Rect::from_center_size(Pos2::new(at(1013.0), cy), Vec2::splat(pt(24.0)));
    if ps_button(ui, commit, Icon::CropCommit)
        .on_hover_text("Commit transform (Return)")
        .clicked()
    {
        crate::free_transform::commit(state);
    }
}

/// Move: Auto-Select (Layer), Show Transform Controls, and the align and
/// distribute buttons (disabled: they need two or more layers selected).
fn move_options(ui: &mut Ui, app: &mut AppState) {
    let opts = &mut app.move_options;
    ui.spacing_mut().item_spacing.x = 0.0;
    widgets::checkbox(ui, &mut opts.auto_select, "Auto-Select:");
    ui.add_space(pt(8.5));
    widgets::dropdown(ui, "move-auto-select", pt(55.0), "Layer", |ui| {
        let _ = ui.selectable_label(true, "Layer");
    });
    ui.add_space(pt(4.5));
    sep(ui);
    ui.add_space(pt(4.0));
    widgets::checkbox(
        ui,
        &mut opts.show_transform_controls,
        "Show Transform Controls",
    );
    ui.add_space(pt(5.5));
    sep(ui);
    // Offsets of the icon centers from the separator before them
    let start = ui.cursor().left();
    let groups: [&[(f32, Icon, &str)]; 3] = [
        &[
            (17.75, Icon::AlignLeft, "Align left edges"),
            (
                44.0,
                Icon::AlignHorizontalCenter,
                "Align horizontal centers",
            ),
            (69.75, Icon::AlignRight, "Align right edges"),
            (100.0, Icon::DistributeVertically, "Distribute vertically"),
        ],
        &[
            (135.0, Icon::AlignTop, "Align top edges"),
            (161.0, Icon::AlignVerticalCenter, "Align vertical centers"),
            (187.0, Icon::AlignBottom, "Align bottom edges"),
            (
                217.0,
                Icon::DistributeHorizontally,
                "Distribute horizontally",
            ),
        ],
        &[(252.0, Icon::More, "More options")],
    ];
    let cy = ui.max_rect().top() + CENTER_Y;
    let button = Vec2::new(pt(24.0), pt(24.0));
    for (g, group) in groups.into_iter().enumerate() {
        for &(x, icon, tip) in group {
            let rect = Rect::from_center_size(Pos2::new(start + pt(x), cy), button);
            use op_core::align::{Align, Distribute};
            let command = match icon {
                Icon::AlignLeft => Some(Command::Align(Align::Left)),
                Icon::AlignHorizontalCenter => Some(Command::Align(Align::HorizontalCenter)),
                Icon::AlignRight => Some(Command::Align(Align::Right)),
                Icon::DistributeVertically => Some(Command::Distribute(Distribute::Vertically)),
                Icon::AlignTop => Some(Command::Align(Align::Top)),
                Icon::AlignVerticalCenter => Some(Command::Align(Align::VerticalCenter)),
                Icon::AlignBottom => Some(Command::Align(Align::Bottom)),
                Icon::DistributeHorizontally => Some(Command::Distribute(Distribute::Horizontally)),
                _ => None,
            };
            // Enabled when Photoshop's are: two or more layers to line up
            // (or a pixel selection), three or more to distribute
            let enabled = command.is_none_or(|c| c.enabled(app));
            if enabled {
                let clicked = ps_button(ui, rect, icon).on_hover_text(tip).clicked();
                if clicked && let Some(c) = command {
                    let ctx = ui.ctx().clone();
                    crate::commands::run(c, &ctx, app);
                }
            } else {
                // Drawn directly: a disabled Ui would fade Photoshop's gray
                crate::ps_icons::paint(
                    ui.painter(),
                    rect.center(),
                    icon,
                    color::OPTIONS_ICON_DISABLED,
                    color::OPTIONS_BAR,
                );
                ui.interact(rect, ui.id().with(tip), Sense::hover())
                    .on_hover_text(tip);
            }
        }
        let sep_x = [117.0, 234.0, 269.0][g];
        ui.painter().rect_filled(
            Rect::from_min_size(
                Pos2::new(start + pt(sep_x), ui.max_rect().top() + pt(6.0)),
                Vec2::new(pt(1.0), pt(22.5)),
            ),
            0,
            color::OPTIONS_SEPARATOR,
        );
    }
    let gear = Rect::from_center_size(Pos2::new(start + pt(291.0), cy), button);
    ui.scope_builder(egui::UiBuilder::new().max_rect(gear), |ui| {
        widgets::ps_icon_button(ui, gear.size(), Icon::Gear, color::OPTIONS_ICON)
            .on_hover_text("Set additional options");
    });
}

/// A separator in the tool's options, taking 1 pt of the row.
fn sep(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(pt(1.0), pt(22.5)), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::OPTIONS_SEPARATOR);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_typed_with_or_without_units() {
        assert_eq!(typed_number("60.00 px"), Some(60.0));
        assert_eq!(typed_number("50%"), Some(50.0));
        assert_eq!(typed_number(" 15° "), Some(15.0));
        assert_eq!(typed_number("-2.5"), Some(-2.5));
        assert_eq!(typed_number("12 pt"), Some(12.0));
        assert_eq!(typed_number("abc"), None);
    }
}
