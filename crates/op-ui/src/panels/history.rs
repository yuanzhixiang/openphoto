//! History panel, popped out from the icon strip like Photoshop's collapsed
//! panels. Every size here is measured in Photoshop 2026 at 1:1, in points.

use egui::{
    Align2, Color32, CursorIcon, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};

use crate::icons;
use crate::state::AppState;
use crate::theme::{self, color, pt};

pub const WIDTH: f32 = pt(235.0);
const DEFAULT_HEIGHT: f32 = pt(155.0);
const MIN_HEIGHT: f32 = pt(100.0);
const TAB_BAR: f32 = pt(28.0);
const TAB_PAD: f32 = pt(9.5);
const ROW: f32 = pt(22.0);
const SNAPSHOT_ROW: f32 = pt(42.0);
const BOTTOM_BAR: f32 = pt(23.0);
const GRIP: f32 = pt(5.0);
const FONT: f32 = pt(12.0);

// Horizontal positions inside a row, from the panel's left edge
const CHECKBOX_X: f32 = pt(6.0);
const CHECKBOX: f32 = pt(11.0);
const ICON_X: f32 = pt(39.0);
const TEXT_X: f32 = pt(57.0);
const SNAPSHOT_SOURCE_X: f32 = pt(11.0);
const THUMB_X: f32 = pt(24.0);
const THUMB: f32 = pt(30.0);
const SNAPSHOT_TEXT_X: f32 = pt(58.0);

const BORDER: Color32 = Color32::from_gray(0x33);
const ROW_LINE: Color32 = Color32::from_gray(0x47);
const UNDONE: Color32 = Color32::from_gray(0x86);
const SCROLL_TRACK: Color32 = Color32::from_gray(0x4a);
/// Width of the scroll track; the handle inside it is a narrower pill.
const SCROLL_TRACK_W: f32 = pt(12.0);
const SCROLL_HANDLE_W: f32 = pt(7.0);
const TAB_TEXT_INACTIVE: Color32 = Color32::from_gray(0xa8);
const SCROLL_HANDLE: Color32 = Color32::from_gray(0x6e);
const GRIP_FILL: Color32 = Color32::from_gray(0x45);
const GRIP_LINE: Color32 = Color32::from_gray(0x38);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    History,
    Comments,
}

/// Per-session popout state: active tab and the height set with the grip.
pub struct PanelState {
    tab: Tab,
    height: f32,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            tab: Tab::History,
            height: DEFAULT_HEIGHT,
        }
    }
}

impl PanelState {
    pub fn size(&self) -> Vec2 {
        vec2(WIDTH, self.height)
    }
}

/// What the panel asks its host to do.
pub enum Action {
    None,
    /// The ">>" button collapses the popout back into the icon strip.
    Collapse,
}

pub fn show(ui: &mut Ui, app: &mut AppState) -> Action {
    let full = ui.max_rect();
    let mut action = Action::None;

    let bar = Rect::from_min_size(full.min, vec2(full.width(), TAB_BAR));
    let grip = Rect::from_min_max(pos2(full.left(), full.bottom() - GRIP), full.max);
    let bottom = Rect::from_min_max(
        pos2(full.left(), grip.top() - BOTTOM_BAR),
        pos2(full.right(), grip.top()),
    );
    let list = Rect::from_min_max(bar.left_bottom(), bottom.right_top());

    ui.painter().rect_filled(full, 0, color::PANEL);
    if tab_bar(ui, bar, &mut app.history_panel.tab) {
        action = Action::Collapse;
    }

    match app.history_panel.tab {
        Tab::History => history_list(ui, list, app),
        Tab::Comments => {
            ui.painter().text(
                list.center(),
                Align2::CENTER_CENTER,
                "Comments — not implemented yet",
                egui::FontId::proportional(FONT),
                color::TEXT_DISABLED,
            );
        }
    }
    bottom_bar(ui, bottom, app);
    resize_grip(ui, grip, &mut app.history_panel.height);

    ui.painter()
        .rect_stroke(full, 0, Stroke::new(1.0, BORDER), StrokeKind::Outside);
    action
}

/// Returns true when the ">>" (collapse) button was clicked.
fn tab_bar(ui: &mut Ui, bar: Rect, active: &mut Tab) -> bool {
    let painter = ui.painter().clone();
    painter.rect_filled(bar, 0, color::TAB_BAR);

    let font = theme::semibold(FONT);
    let mut x = bar.left();
    for (tab, label) in [(Tab::History, "History"), (Tab::Comments, "Comments")] {
        let galley = painter.layout_no_wrap(label.into(), font.clone(), Color32::WHITE);
        let rect = Rect::from_min_size(
            pos2(x, bar.top()),
            vec2(galley.size().x + 2.0 * TAB_PAD, bar.height()),
        );
        let response = ui.interact(rect, ui.id().with(("history-tab", label)), Sense::click());
        if response.clicked() {
            *active = tab;
        }
        let selected = *active == tab;
        if selected {
            painter.rect_filled(rect, 0, color::TAB_ACTIVE);
        } else {
            painter.line_segment(
                [rect.right_top(), rect.right_bottom()],
                Stroke::new(1.0, color::SEPARATOR),
            );
        }
        let text = if selected || response.hovered() {
            color::TEXT
        } else {
            TAB_TEXT_INACTIVE
        };
        painter.galley(rect.center() - galley.size() / 2.0, galley, text);
        x = rect.right();
    }

    // Right side: collapse ">>", a divider, then the panel menu
    let cy = bar.center().y;
    let collapse = Rect::from_center_size(pos2(bar.right() - pt(33.0), cy), Vec2::splat(pt(16.0)));
    let collapse_response = ui.interact(collapse, ui.id().with("history-collapse"), Sense::click());
    let tint = if collapse_response.hovered() {
        color::TEXT
    } else {
        color::ICON
    };
    painter.text(
        collapse.center(),
        Align2::CENTER_CENTER,
        icons::CARET_DOUBLE_RIGHT,
        theme::icon(pt(11.0)),
        tint,
    );
    let divider_x = bar.right() - pt(21.0);
    painter.line_segment(
        [pos2(divider_x, cy - pt(7.0)), pos2(divider_x, cy + pt(7.0))],
        Stroke::new(1.0, color::TEXT_DIM),
    );
    painter.text(
        pos2(bar.right() - pt(9.0), cy),
        Align2::CENTER_CENTER,
        icons::LIST,
        theme::icon(pt(12.0)),
        color::ICON,
    );
    collapse_response
        .on_hover_text("Collapse to Icons")
        .clicked()
}

fn history_list(ui: &mut Ui, list: Rect, app: &mut AppState) {
    let Some(state) = app.active() else {
        return;
    };
    let thumb = state.snapshot_thumbnail(ui.ctx());
    let mut jump = None;
    let mut new_source = None;

    // egui draws the track only as wide as the handle, so the wider track is
    // painted here whenever the list overflows.
    let content_height = SNAPSHOT_ROW + state.history.states().len() as f32 * ROW;
    if content_height > list.height() {
        let track = Rect::from_min_max(pos2(list.right() - SCROLL_TRACK_W, list.top()), list.max);
        ui.painter().rect_filled(track, 0, SCROLL_TRACK);
    }

    let mut list_ui = ui.new_child(egui::UiBuilder::new().max_rect(list));
    list_ui.set_clip_rect(list);
    {
        let margin = (SCROLL_TRACK_W - SCROLL_HANDLE_W) / 2.0;
        let style = list_ui.style_mut();
        style.spacing.scroll = egui::style::ScrollStyle {
            floating: false,
            bar_width: SCROLL_HANDLE_W,
            handle_min_length: pt(20.0),
            bar_inner_margin: margin,
            bar_outer_margin: margin,
            ..egui::style::ScrollStyle::solid()
        };
        style.spacing.scroll.fade.strength = 0.0;
        style.visuals.extreme_bg_color = SCROLL_TRACK;
        for w in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
        ] {
            w.bg_fill = SCROLL_HANDLE;
            w.fg_stroke.color = SCROLL_HANDLE;
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(&mut list_ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let current = state.history.current();
            let title = state.doc.title.clone();
            // The source shown is the one the History Brush would use
            let source = state
                .history_source
                .filter(|&id| state.history.index_of(id).is_some());

            let (row, pick) = snapshot_row(ui, &title, thumb.as_ref(), source.is_none());
            if pick {
                new_source = Some(None);
            } else if row.clicked() {
                jump = Some(0);
            }
            for (i, h) in state.history.states().iter().enumerate() {
                // States after the current one have been undone and are drawn dimmed
                let (row, pick) = state_row(
                    ui,
                    icons::FILE_TEXT,
                    &h.name,
                    (i == current, i > current),
                    source == Some(h.id),
                );
                if pick {
                    new_source = Some(Some(h.id));
                } else if row.clicked() {
                    jump = Some(i);
                }
            }
        });
    if let Some(source) = new_source {
        state.history_source = source;
    }
    if let Some(i) = jump {
        state.jump_to_state(i);
    }
}

/// The snapshot of the document as opened, shown above the states. Its left
/// column carries the history-brush source marker when it is the source
/// (an empty box otherwise); the second value is a click on that column,
/// which makes it the source.
fn snapshot_row(
    ui: &mut Ui,
    title: &str,
    thumb: Option<&egui::TextureHandle>,
    is_source: bool,
) -> (egui::Response, bool) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), SNAPSHOT_ROW), Sense::click());
    let pick = source_column(ui, rect, SNAPSHOT_SOURCE_X, is_source, "snapshot");
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, 0, color::HOVER);
    }
    let cy = rect.center().y;
    source_marker(
        painter,
        pos2(rect.left() + SNAPSHOT_SOURCE_X, cy),
        is_source,
    );

    let thumb_box = Rect::from_min_size(
        pos2(rect.left() + THUMB_X, cy - THUMB / 2.0),
        Vec2::splat(THUMB),
    );
    painter.rect_filled(thumb_box, 0, color::PASTEBOARD);
    if let Some(tex) = thumb {
        let size = tex.size_vec2();
        let scale = (THUMB / size.x).min(THUMB / size.y);
        let r = Rect::from_center_size(thumb_box.center(), size * scale);
        crate::widgets::checkerboard(painter, r, pt(3.0));
        painter.image(
            tex.id(),
            r,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    painter.rect_stroke(
        thumb_box,
        0,
        Stroke::new(1.0, color::SEPARATOR),
        StrokeKind::Outside,
    );

    // Long names are cut with an ellipsis, as in Photoshop
    let text_left = rect.left() + SNAPSHOT_TEXT_X;
    let mut job = egui::text::LayoutJob::simple_singleline(
        title.to_owned(),
        egui::FontId::proportional(FONT),
        color::TEXT,
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(rect.right() - pt(6.0) - text_left);
    let galley = painter.layout_job(job);
    painter.galley(
        pos2(text_left, cy - galley.size().y / 2.0),
        galley,
        color::TEXT,
    );

    // Heavier divider between the snapshot and the states
    let line = Rect::from_min_max(
        pos2(rect.left(), rect.bottom() - pt(2.0)),
        rect.right_bottom(),
    );
    painter.rect_filled(line, 0, GRIP_LINE);
    (response, pick)
}

/// The left column of a row: clicking it makes the row the History Brush's
/// source.
fn source_column(ui: &Ui, row: Rect, center_x: f32, is_source: bool, id: &str) -> bool {
    let column = Rect::from_min_max(row.min, pos2(row.left() + center_x * 2.0, row.bottom()));
    let response = ui.interact(column, ui.id().with(("history-source", id)), Sense::click());
    response.clicked() && !is_source
}

/// The source marker: the history brush, or an empty box on rows that
/// aren't the source.
fn source_marker(painter: &egui::Painter, center: Pos2, is_source: bool) {
    if is_source {
        painter.text(
            center,
            Align2::CENTER_CENTER,
            icons::PAINT_BRUSH_BROAD,
            theme::icon(pt(13.0)),
            color::ICON,
        );
    } else {
        painter.rect(
            Rect::from_center_size(center, Vec2::splat(CHECKBOX)),
            1,
            Color32::from_black_alpha(25),
            Stroke::new(1.0, Color32::from_gray(0x44)),
            StrokeKind::Inside,
        );
    }
}

fn state_row(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    (selected, undone): (bool, bool),
    is_source: bool,
) -> (egui::Response, bool) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    let pick = source_column(
        ui,
        rect,
        CHECKBOX_X + CHECKBOX / 2.0,
        is_source,
        &format!("{}", rect.top()),
    );
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 0, color::ROW_SELECTED);
    } else if response.hovered() {
        painter.rect_filled(rect, 0, color::HOVER);
    }
    painter.line_segment(
        [
            rect.left_bottom() - vec2(0.0, 0.5),
            rect.right_bottom() - vec2(0.0, 0.5),
        ],
        Stroke::new(1.0, ROW_LINE),
    );

    let cy = rect.center().y;
    // The History Brush's source box
    source_marker(
        painter,
        pos2(rect.left() + CHECKBOX_X + CHECKBOX / 2.0, cy),
        is_source,
    );

    let tint = if undone { UNDONE } else { color::TEXT };
    painter.text(
        pos2(rect.left() + ICON_X, cy),
        Align2::CENTER_CENTER,
        icon,
        theme::icon(pt(14.0)),
        tint,
    );
    painter.text(
        pos2(rect.left() + TEXT_X, cy),
        Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(FONT),
        tint,
    );
    (response, pick)
}

fn bottom_bar(ui: &mut Ui, rect: Rect, app: &mut AppState) {
    ui.painter().rect_filled(rect, 0, color::PANEL);
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, ROW_LINE),
    );

    let current = app.active().map(|s| s.history.current());
    let cy = rect.center().y;
    let buttons = [
        (
            pt(98.0),
            icons::FILE_PLUS,
            "Create new document from current state",
            false,
        ),
        (pt(65.0), icons::CAMERA, "Create new snapshot", false),
        (
            pt(36.0),
            icons::TRASH,
            "Delete current state",
            current.is_some_and(|c| c > 0),
        ),
    ];
    for (from_right, icon, tip, enabled) in buttons {
        let r = Rect::from_center_size(pos2(rect.right() - from_right, cy), Vec2::splat(pt(18.0)));
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui
            .interact(r, ui.id().with(("history-button", icon)), sense)
            .on_hover_text(tip);
        let tint = if !enabled {
            color::TEXT_DISABLED
        } else if response.hovered() {
            color::TEXT
        } else {
            color::ICON
        };
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            icon,
            theme::icon(pt(13.0)),
            tint,
        );
        if enabled
            && response.clicked()
            && let (Some(state), Some(c)) = (app.active(), current)
        {
            state.delete_states_from(c);
        }
    }
}

/// The strip under the panel; dragging it changes the panel's height.
fn resize_grip(ui: &mut Ui, rect: Rect, height: &mut f32) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0, GRIP_FILL);
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, GRIP_LINE),
    );
    let c = rect.center();
    for i in -5..=5 {
        let x = c.x + i as f32 * pt(1.6);
        painter.line_segment(
            [pos2(x, c.y - pt(1.2)), pos2(x, c.y + pt(1.2))],
            Stroke::new(0.8, color::TEXT_DIM),
        );
    }

    let response = ui.interact(rect, ui.id().with("history-grip"), Sense::drag());
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }
    if response.dragged() {
        *height = (*height + response.drag_delta().y).max(MIN_HEIGHT);
    }
}
