//! Photoshop's alerts are macOS alerts: a 260 pt wide light panel with the
//! app's icon (or the caution triangle with the app's badge), the message
//! in bold, an optional "Don't show again" checkbox and full-width pill
//! buttons. Laid out from Photoshop 2026's alerts measured at 2x; sizes are
//! in points from the alert's top-left corner.

use egui::{
    Align2, Color32, CornerRadius, Key, Modifiers, Pos2, Rect, Sense, Shape, Stroke, StrokeKind,
    Ui, vec2,
};

use crate::theme::{self, pt};

const WIDTH: f32 = pt(260.0);
const BACKGROUND: Color32 = Color32::from_gray(0xb3);
const EDGE: Color32 = Color32::from_gray(0xeb);
const CANCEL: Color32 = Color32::from_gray(0xa4);
const OK: Color32 = Color32::from_rgb(0x34, 0x78, 0xf6);
const CHECKBOX: Color32 = Color32::from_gray(0x9f);
const TEXT: Color32 = Color32::from_gray(0x1c);
/// Message lines are this far apart, the first one's capitals from y 103.
const LINE: f32 = pt(16.0);
const MESSAGE_WIDTH: f32 = pt(216.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    /// The app's icon: errors and notices.
    App,
    /// The caution triangle with the app's badge: questions with a risk.
    Caution,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alert {
    pub message: String,
    pub icon: Icon,
    /// Show Cancel next to OK.
    pub cancel: bool,
    /// A "Don't show again" checkbox, and whether it is ticked.
    pub dont_show_again: Option<bool>,
    /// Buttons stacked full width instead of OK/Cancel, the first one the
    /// default (blue), the last one taken by Escape.
    pub choices: Vec<String>,
    /// The default button's label ("OK" unless renamed, e.g. "Full
    /// Screen").
    pub ok_label: &'static str,
    /// The other button's label ("Cancel" unless renamed, e.g. "Learn
    /// More").
    pub cancel_label: &'static str,
}

impl Alert {
    /// An error alert, with Photoshop's Learn More beside OK for the alerts
    /// that have it (Learn More just closes it: there is no help site).
    pub fn for_message(message: impl Into<String>) -> Self {
        let message = message.into();
        let learn_more = message == crate::selection_brush::NO_LAYER;
        let mut alert = Self::error(message);
        if learn_more {
            alert.cancel = true;
            alert.cancel_label = "Learn More";
        }
        alert
    }

    /// An error or notice with only OK, like Photoshop's "Could not ..."
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            icon: Icon::App,
            cancel: false,
            dont_show_again: None,
            choices: Vec::new(),
            ok_label: "OK",
            cancel_label: "Cancel",
        }
    }

    /// A question with Cancel and OK and a "Don't show again" checkbox.
    pub fn caution(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            icon: Icon::Caution,
            cancel: true,
            dont_show_again: Some(false),
            choices: Vec::new(),
            ok_label: "OK",
            cancel_label: "Cancel",
        }
    }

    /// A question with its own answers, stacked (e.g. deleting a group:
    /// "Group and Contents", "Group Only", "Cancel").
    pub fn choose(message: impl Into<String>, choices: &[&str]) -> Self {
        Self {
            message: message.into(),
            icon: Icon::Caution,
            cancel: true,
            dont_show_again: None,
            choices: choices.iter().map(|c| c.to_string()).collect(),
            ok_label: "OK",
            cancel_label: "Cancel",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// OK, with whether "Don't show again" was ticked.
    Ok {
        dont_show_again: bool,
    },
    Cancel,
    /// One of the stacked choices, by position.
    Choice(usize),
}

/// Shows `alert`; returns the answer once given. Enter means OK, Escape
/// Cancel (or OK when there's no Cancel).
pub fn show(ctx: &egui::Context, alert: &mut Alert) -> Option<Answer> {
    // NSAlert's text: the message in 13 pt bold, tracked as AppKit tracks
    // it, wrapped to the alert's width
    let font = theme::dialog_bold(pt(13.0));
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = MESSAGE_WIDTH;
    job.append(
        &alert.message,
        0.0,
        egui::TextFormat {
            font_id: font,
            color: TEXT,
            extra_letter_spacing: theme::system_tracking(13.0),
            ..Default::default()
        },
    );
    let galley = ctx.fonts_mut(|f| f.layout_job(job));
    let lines = galley.rows.len().max(1) as f32;
    let content_end = pt(103.0) + LINE * lines;
    let buttons_top = match alert.dont_show_again {
        Some(_) => content_end + pt(13.0) + pt(16.0) + pt(16.0),
        None => content_end + pt(13.0),
    };
    // Stacked choices are 28 pt tall, 34 pt apart
    let buttons_h = if alert.choices.is_empty() {
        pt(28.0)
    } else {
        pt(34.0) * alert.choices.len() as f32 - pt(6.0)
    };
    let height = buttons_top + buttons_h + pt(16.0);

    let mut answer = None;
    egui::Modal::new(egui::Id::new("alert"))
        .frame(egui::Frame::NONE)
        .backdrop_color(Color32::TRANSPARENT)
        .show(ctx, |ui| {
            let (frame, _) = ui.allocate_exact_size(vec2(WIDTH, height), Sense::hover());
            answer = draw(ui, frame, alert, galley, buttons_top, content_end);
        });
    let ok = Answer::Ok {
        dont_show_again: alert.dont_show_again.unwrap_or(false),
    };
    let choices = alert.choices.len();
    ctx.input_mut(|i| {
        if i.consume_key(Modifiers::NONE, Key::Enter) {
            answer = Some(if choices > 0 { Answer::Choice(0) } else { ok });
        } else if i.consume_key(Modifiers::NONE, Key::Escape) {
            answer = Some(if choices > 0 {
                Answer::Choice(choices - 1)
            } else if alert.cancel {
                Answer::Cancel
            } else {
                ok
            });
        }
    });
    answer
}

fn draw(
    ui: &mut Ui,
    frame: Rect,
    alert: &mut Alert,
    galley: std::sync::Arc<egui::Galley>,
    buttons_top: f32,
    content_end: f32,
) -> Option<Answer> {
    let at = |x: f32, y: f32| frame.min + vec2(x, y);
    let painter = ui.painter().clone();
    let radius = CornerRadius::same(pt(16.0) as u8);
    painter.add(
        egui::Shadow {
            offset: [0, 10],
            blur: 40,
            spread: 0,
            color: Color32::from_black_alpha(90),
        }
        .as_shape(frame, radius),
    );
    painter.rect(
        frame,
        radius,
        BACKGROUND,
        Stroke::new(pt(0.5), EDGE),
        StrokeKind::Inside,
    );

    match alert.icon {
        Icon::App => app_icon(
            &painter,
            Rect::from_min_size(at(pt(26.5), pt(26.5)), vec2(pt(51.0), pt(51.0))),
        ),
        Icon::Caution => {
            caution(
                &painter,
                Rect::from_min_max(at(pt(22.0), pt(24.0)), at(pt(80.0), pt(76.0))),
            );
            app_icon(
                &painter,
                Rect::from_min_size(at(pt(55.0), pt(55.0)), vec2(pt(27.0), pt(27.0))),
            );
        }
    }

    // The message: the first line's capitals start at y 103
    let first_line_top = at(0.0, pt(103.0)).y - pt(3.0);
    painter.galley(Pos2::new(at(pt(22.5), 0.0).x, first_line_top), galley, TEXT);

    let mut answer = None;
    if let Some(ticked) = &mut alert.dont_show_again {
        let b = Rect::from_min_size(
            at(pt(22.0), content_end + pt(13.0)),
            vec2(pt(16.0), pt(16.0)),
        );
        let label = "Don\u{2019}t show again";
        let text = theme::tracked_galley(&painter, label, theme::dialog(pt(13.0)), TEXT);
        let hit = Rect::from_min_max(
            b.min,
            Pos2::new(b.right() + pt(7.0) + text.size().x, b.bottom()),
        );
        if ui
            .interact(hit, ui.id().with("dont-show"), Sense::click())
            .clicked()
        {
            *ticked = !*ticked;
        }
        if *ticked {
            painter.rect_filled(b, CornerRadius::same(pt(4.0) as u8), OK);
            let p = |x: f32, y: f32| b.min + vec2(pt(x), pt(y));
            painter.add(Shape::line(
                vec![p(4.0, 8.0), p(7.0, 11.0), p(12.0, 5.0)],
                Stroke::new(pt(2.0), Color32::WHITE),
            ));
        } else {
            painter.rect_filled(b, CornerRadius::same(pt(4.0) as u8), CHECKBOX);
        }
        painter.galley(
            Pos2::new(b.right() + pt(7.0), b.center().y - text.size().y / 2.0),
            text,
            TEXT,
        );
    }

    let button = |rect: Rect, label: &str, fill: Color32, text: Color32, id: &str| {
        let response = ui.interact(rect, ui.id().with(id), Sense::click());
        let fill = if response.is_pointer_button_down_on() {
            fill.gamma_multiply(0.85)
        } else {
            fill
        };
        painter.rect_filled(rect, CornerRadius::same(255), fill);
        let galley = theme::tracked_galley(&painter, label, theme::dialog(pt(13.0)), text);
        // AppKit centers the label 1 pt higher than its line box would
        let at = rect.center() - galley.size() / 2.0 - vec2(0.0, pt(1.0));
        painter.galley(at, galley, text);
        response.clicked()
    };
    let row = |x0: f32, x1: f32| {
        Rect::from_min_max(at(pt(x0), buttons_top), at(pt(x1), buttons_top + pt(28.0)))
    };
    let ok = Answer::Ok {
        dont_show_again: alert.dont_show_again.unwrap_or(false),
    };
    if !alert.choices.is_empty() {
        for (k, label) in alert.choices.iter().enumerate() {
            let top = buttons_top + pt(34.0) * k as f32;
            let rect = Rect::from_min_max(at(pt(16.0), top), at(pt(244.0), top + pt(28.0)));
            let (fill, text) = if k == 0 {
                (OK, Color32::WHITE)
            } else {
                (CANCEL, TEXT)
            };
            if button(rect, label, fill, text, label) {
                answer = Some(Answer::Choice(k));
            }
        }
    } else if alert.cancel {
        if button(row(16.0, 126.0), alert.cancel_label, CANCEL, TEXT, "cancel") {
            answer = Some(Answer::Cancel);
        }
        if button(row(134.0, 244.0), alert.ok_label, OK, Color32::WHITE, "ok") {
            answer = Some(ok);
        }
    } else if button(row(16.0, 244.0), "OK", OK, Color32::WHITE, "ok") {
        answer = Some(ok);
    }
    answer
}

/// OpenPhoto's app icon: a rounded square with "Op" (its own design, not
/// Photoshop's).
fn app_icon(painter: &egui::Painter, rect: Rect) {
    let radius = CornerRadius::same((rect.width() * 0.22) as u8);
    painter.rect_filled(rect, radius, Color32::from_rgb(0x1b, 0x2b, 0x26));
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(pt(0.5), Color32::from_rgb(0x3d, 0x5a, 0x50)),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "Op",
        theme::dialog_bold(rect.height() * 0.48),
        Color32::from_rgb(0x7d, 0xe0, 0xb8),
    );
}

/// macOS's caution sign: a yellow rounded triangle with an exclamation mark.
fn caution(painter: &egui::Painter, rect: Rect) {
    let top = Pos2::new(rect.center().x, rect.top());
    let points = vec![top, rect.right_bottom(), rect.left_bottom()];
    painter.add(Shape::convex_polygon(
        points.clone(),
        Color32::from_rgb(0xf2, 0xc9, 0x48),
        Stroke::new(pt(2.5), Color32::WHITE),
    ));
    let c = Pos2::new(rect.center().x, rect.top() + rect.height() * 0.62);
    painter.rect_filled(
        Rect::from_center_size(c, vec2(pt(4.5), rect.height() * 0.42)),
        CornerRadius::same(pt(2.0) as u8),
        Color32::WHITE,
    );
    painter.circle_filled(
        Pos2::new(rect.center().x, rect.bottom() - rect.height() * 0.12),
        pt(2.6),
        Color32::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors() {
        let e = Alert::error("x");
        assert!(!e.cancel && e.dont_show_again.is_none() && e.icon == Icon::App);
        let c = Alert::caution("y");
        assert!(c.cancel && c.dont_show_again == Some(false) && c.icon == Icon::Caution);
    }
}
