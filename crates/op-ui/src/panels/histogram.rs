//! Histogram panel: the merged image's histogram, compact (luminosity)
//! or expanded (a channel, and the statistics of what it shows).

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use super::floating::small;
use crate::state::AppState;
use crate::theme::{color, pt};

/// The expanded view's channels, as its menu lists them.
pub const CHANNELS: [&str; 6] = ["Luminosity", "Red", "Green", "Blue", "RGB", "Colors"];

/// The expanded view's height (its compact one is the panel's default).
pub const EXPANDED: f32 = pt(260.0);

/// Statistics of a histogram: mean, standard deviation, median, pixels.
pub fn statistics(hist: &[u64; 256]) -> (f32, f32, usize, u64) {
    let n: u64 = hist.iter().sum();
    if n == 0 {
        return (0.0, 0.0, 0, 0);
    }
    let mean = hist
        .iter()
        .enumerate()
        .map(|(v, &c)| v as f64 * c as f64)
        .sum::<f64>()
        / n as f64;
    let var = hist
        .iter()
        .enumerate()
        .map(|(v, &c)| (v as f64 - mean).powi(2) * c as f64)
        .sum::<f64>()
        / n as f64;
    let mut acc = 0;
    let median = hist
        .iter()
        .position(|&c| {
            acc += c;
            acc * 2 >= n
        })
        .unwrap_or(0);
    (mean as f32, var.sqrt() as f32, median, n)
}

fn draw(ui: &Ui, rect: Rect, hists: &[([u64; 256], Color32)]) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0, Color32::from_gray(0x3c));
    let max = hists
        .iter()
        .flat_map(|(h, _)| h.iter().copied())
        .max()
        .unwrap_or(0)
        .max(1) as f32;
    let bar = rect.width() / 256.0;
    for (hist, ink) in hists {
        for (i, &count) in hist.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let h = count as f32 / max * rect.height();
            let x = rect.left() + (i as f32 + 0.5) * bar;
            painter.line_segment(
                [Pos2::new(x, rect.bottom()), Pos2::new(x, rect.bottom() - h)],
                Stroke::new(bar.max(1.0), *ink),
            );
        }
    }
}

pub fn show(ui: &mut Ui, app: &mut AppState) {
    let rect = ui.max_rect();
    // The Expanded View toggle at the top right
    let toggle = Rect::from_min_size(
        rect.right_top() - Vec2::new(pt(16.0), 0.0),
        Vec2::splat(pt(14.0)),
    );
    let expanded = app.histogram_expanded;
    let response = ui
        .interact(toggle, ui.id().with("histogram-expand"), Sense::click())
        .on_hover_text(if expanded {
            "Compact View"
        } else {
            "Expanded View"
        });
    ui.painter().text(
        toggle.center(),
        Align2::CENTER_CENTER,
        if expanded { "−" } else { "+" },
        small(),
        if response.hovered() {
            color::TEXT
        } else {
            color::TEXT_DIM
        },
    );
    if response.clicked() {
        app.histogram_expanded = !expanded;
    }
    let channel = app.histogram_channel.min(CHANNELS.len() - 1);
    let Some(state) = app.active() else {
        return;
    };
    let hists = state.channel_histograms();
    if !expanded {
        let graph = Rect::from_min_max(rect.min, Pos2::new(toggle.left() - pt(4.0), rect.bottom()));
        draw(ui, graph, &[(hists[0], color::TEXT)]);
        return;
    }
    // Expanded: the channel menu, the graph, the statistics
    let mut picked = None;
    let menu = Rect::from_min_size(rect.min, Vec2::new(pt(120.0), pt(18.0)));
    ui.scope_builder(egui::UiBuilder::new().max_rect(menu), |ui| {
        egui::ComboBox::from_id_salt("histogram-channel")
            .selected_text(CHANNELS[channel])
            .width(pt(110.0))
            .show_ui(ui, |ui| {
                for (k, name) in CHANNELS.iter().enumerate() {
                    if ui.selectable_label(k == channel, *name).clicked() {
                        picked = Some(k);
                    }
                }
            });
    });
    let graph = Rect::from_min_max(
        rect.min + Vec2::new(0.0, pt(26.0)),
        Pos2::new(rect.right(), rect.top() + pt(146.0)),
    );
    let shown: Vec<([u64; 256], Color32)> = match channel {
        0 => vec![(hists[0], color::TEXT)],
        1 => vec![(hists[1], Color32::from_rgb(0xe0, 0x40, 0x40))],
        2 => vec![(hists[2], Color32::from_rgb(0x40, 0xd0, 0x40))],
        3 => vec![(hists[3], Color32::from_rgb(0x50, 0x70, 0xff))],
        4 => vec![(hists[4], color::TEXT)],
        _ => vec![
            (
                hists[1],
                Color32::from_rgba_unmultiplied(0xe0, 0x40, 0x40, 0xa0),
            ),
            (
                hists[2],
                Color32::from_rgba_unmultiplied(0x40, 0xd0, 0x40, 0xa0),
            ),
            (
                hists[3],
                Color32::from_rgba_unmultiplied(0x50, 0x70, 0xff, 0xa0),
            ),
        ],
    };
    draw(ui, graph, &shown);
    // Statistics of the channel (Colors: of RGB), and of the level under
    // the pointer
    let stats_of = if channel == 5 {
        hists[4]
    } else {
        hists[channel.min(4)]
    };
    let (mean, dev, median, pixels) = statistics(&stats_of);
    let hover = ui
        .interact(graph, ui.id().with("histogram-graph"), Sense::hover())
        .hover_pos()
        .map(|p| (((p.x - graph.left()) / graph.width() * 256.0) as usize).min(255));
    let level = hover.map(|l| {
        let count = stats_of[l];
        let below: u64 = stats_of[..=l].iter().sum();
        let pct = if pixels > 0 {
            below as f32 / pixels as f32 * 100.0
        } else {
            0.0
        };
        (l, count, pct)
    });
    let rows: [(&str, String, &str, String); 4] = [
        (
            "Mean:",
            format!("{mean:.2}"),
            "Level:",
            level.map_or(String::new(), |l| l.0.to_string()),
        ),
        (
            "Std Dev:",
            format!("{dev:.2}"),
            "Count:",
            level.map_or(String::new(), |l| l.1.to_string()),
        ),
        (
            "Median:",
            median.to_string(),
            "Percentile:",
            level.map_or(String::new(), |l| format!("{:.2}", l.2)),
        ),
        ("Pixels:", pixels.to_string(), "Cache Level:", "1".into()),
    ];
    for (k, (a, av, b, bv)) in rows.iter().enumerate() {
        let y = graph.bottom() + pt(16.0) + k as f32 * pt(16.0);
        let text = |x: f32, align: Align2, s: &str| {
            ui.painter().text(
                Pos2::new(rect.left() + pt(x), y),
                align,
                s,
                small(),
                color::TEXT,
            );
        };
        text(60.0, Align2::RIGHT_CENTER, a);
        text(66.0, Align2::LEFT_CENTER, av);
        text(180.0, Align2::RIGHT_CENTER, b);
        text(186.0, Align2::LEFT_CENTER, bv);
    }
    if let Some(k) = picked {
        app.histogram_channel = k;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_of_a_histogram() {
        let mut h = [0u64; 256];
        h[10] = 1;
        h[20] = 3;
        let (mean, dev, median, n) = statistics(&h);
        assert_eq!((mean, median, n), (17.5, 20, 4));
        assert!((dev - 4.330127).abs() < 1e-4);
        assert_eq!(statistics(&[0; 256]), (0.0, 0.0, 0, 0));
    }
}
