//! The New Document dialog's presets: Photoshop 2026's blank document
//! presets by category, their units, and the icons their cards show.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2};

use crate::theme::pt;

/// A length unit for Width and Height.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Pixels,
    Inches,
    Centimeters,
    Millimeters,
    Points,
    Picas,
}

impl Unit {
    pub const ALL: [Self; 6] = [
        Self::Pixels,
        Self::Inches,
        Self::Centimeters,
        Self::Millimeters,
        Self::Points,
        Self::Picas,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Pixels => "Pixels",
            Self::Inches => "Inches",
            Self::Centimeters => "Centimeters",
            Self::Millimeters => "Millimeters",
            Self::Points => "Points",
            Self::Picas => "Picas",
        }
    }

    /// The short form on preset cards ("px", "in", "mm").
    pub fn short(self) -> &'static str {
        match self {
            Self::Pixels => "px",
            Self::Inches => "in",
            Self::Centimeters => "cm",
            Self::Millimeters => "mm",
            Self::Points => "pt",
            Self::Picas => "pica",
        }
    }

    /// Pixels in one unit at `ppi` pixels per inch.
    pub fn pixels(self, ppi: f64) -> f64 {
        match self {
            Self::Pixels => 1.0,
            Self::Inches => ppi,
            Self::Centimeters => ppi / 2.54,
            Self::Millimeters => ppi / 25.4,
            Self::Points => ppi / 72.0,
            Self::Picas => ppi / 6.0,
        }
    }

    /// A value as the fields show it: pixels whole, the others with up to
    /// three decimals.
    pub fn format(self, v: f64) -> String {
        let s = if self == Self::Pixels {
            format!("{}", v.round())
        } else {
            format!("{v:.3}")
        };
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            s
        }
    }
}

/// What a preset's card shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A blank page with a crosshair at its corner (recent and saved).
    Custom,
    Photo,
    Page,
    /// A dotted grid (pixel grids).
    Grid,
    /// A page with a brush (posters, postcards, screens to paint on).
    Brush,
    Browser,
    Phone,
    Tablet,
    Watch,
    /// An app icon: a rounded square of dots.
    AppIcon,
    /// A screen with a stylus (Surface).
    Surface,
    /// A tall strip (Mobile Design).
    Strip,
    /// A video frame with a play button, and its resolution badge.
    Video(Option<&'static str>),
}

/// A blank document preset.
#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub unit: Unit,
    /// Pixels per inch.
    pub ppi: f64,
    pub kind: Kind,
}

impl Preset {
    fn new(name: &str, (width, height): (f64, f64), unit: Unit, ppi: f64, kind: Kind) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            unit,
            ppi,
            kind,
        }
    }

    /// The size line on its card: "7 x 5 in @ 300 ppi".
    pub fn size_label(&self) -> String {
        format!(
            "{} x {} {} @ {} ppi",
            self.unit.format(self.width),
            self.unit.format(self.height),
            self.unit.short(),
            self.ppi.round()
        )
    }

    /// Width and height in pixels.
    pub fn pixels(&self) -> (f64, f64) {
        let k = self.unit.pixels(self.ppi);
        (self.width * k, self.height * k)
    }
}

/// The presets of the category at `index` (Photo, Print, Art &
/// Illustration, Web, Mobile, Film & Video), as Photoshop
/// 2026 lists them.
pub fn category(index: usize) -> Vec<Preset> {
    use Kind::*;
    use Unit::*;
    let px = |name: &str, w: f64, h: f64, ppi: f64, kind: Kind| {
        Preset::new(name, (w, h), Pixels, ppi, kind)
    };
    match index {
        0 => [
            ("Default Photoshop Size", 7.0, 5.0),
            ("Landscape, 3 x 2", 3.0, 2.0),
            ("Landscape, 6 x 4", 6.0, 4.0),
            ("Landscape, 7 x 5", 7.0, 5.0),
            ("Landscape, 10 x 8", 10.0, 8.0),
            ("Portrait, 2 x 3", 2.0, 3.0),
            ("Portrait, 4 x 6", 4.0, 6.0),
            ("Portrait, 5 x 7", 5.0, 7.0),
            ("Portrait, 8 x 10", 8.0, 10.0),
        ]
        .into_iter()
        .map(|(n, w, h)| Preset::new(n, (w, h), Inches, 300.0, Photo))
        .collect(),
        1 => {
            let mut v: Vec<Preset> = [
                ("Letter", 8.5, 11.0),
                ("Legal", 8.5, 14.0),
                ("Tabloid", 11.0, 17.0),
            ]
            .into_iter()
            .map(|(n, w, h)| Preset::new(n, (w, h), Inches, 300.0, Page))
            .collect();
            v.extend(
                [
                    ("A4", 210.0, 297.0),
                    ("A6", 105.0, 148.0),
                    ("A5", 148.0, 210.0),
                    ("A3", 297.0, 420.0),
                    ("B5", 176.0, 250.0),
                    ("B4", 250.0, 353.0),
                    ("B3", 353.0, 500.0),
                    ("C4", 229.0, 324.0),
                    ("C5", 162.0, 229.0),
                ]
                .into_iter()
                .map(|(n, w, h)| Preset::new(n, (w, h), Millimeters, 300.0, Page)),
            );
            v
        }
        2 => vec![
            px("1000 pixel grid", 1000.0, 1000.0, 300.0, Grid),
            px("2000 pixel grid", 2000.0, 2000.0, 300.0, Grid),
            Preset::new("Poster", (18.0, 24.0), Inches, 300.0, Brush),
            Preset::new("Postcard", (4.0, 6.0), Inches, 300.0, Brush),
            px("1080p", 1920.0, 1080.0, 300.0, Brush),
            px("720p", 1280.0, 720.0, 300.0, Brush),
        ],
        3 => [
            ("Web Most Common", 1366.0, 768.0),
            ("Web Large", 1920.0, 1080.0),
            ("Web Medium", 1440.0, 900.0),
            ("Web Minimum", 1024.0, 768.0),
            ("Web Small", 1280.0, 800.0),
            ("MacBook Pro 13 (Retina)", 2560.0, 1600.0),
            ("MacBook Pro 15 (Retina)", 2880.0, 1800.0),
            ("iMac 27", 2560.0, 1440.0),
            ("Desktop HD Design", 1440.0, 1024.0),
        ]
        .into_iter()
        .map(|(n, w, h)| px(n, w, h, 72.0, Browser))
        .collect(),
        4 => [
            ("iPhone X", 1125.0, 2436.0, Phone),
            ("iPhone 8/7/6 Plus", 1242.0, 2208.0, Phone),
            ("iPhone 8/7/6", 750.0, 1334.0, Phone),
            ("iPad Pro 12.9 in", 2048.0, 2732.0, Tablet),
            ("iPad Pro 10.5 in", 1668.0, 2226.0, Tablet),
            ("iPad Retina", 1536.0, 2048.0, Tablet),
            ("Android 1080p", 1080.0, 1920.0, Phone),
            ("Microsoft Surface Pro 4", 2736.0, 1824.0, Surface),
            ("Microsoft Surface Pro 3", 2160.0, 1440.0, Surface),
            ("iPhone 5", 640.0, 1136.0, Phone),
            ("iPad Mini", 768.0, 1024.0, Tablet),
            ("Apple Watch 42mm", 312.0, 390.0, Watch),
            ("Apple Watch 38mm", 272.0, 340.0, Watch),
            ("Mobile Design", 320.0, 1024.0, Strip),
            ("iOS 7 iPad App Icon", 152.0, 152.0, AppIcon),
            ("iOS 7 iPhone App icon", 120.0, 120.0, AppIcon),
            ("iOS 7 Spotlight", 80.0, 80.0, AppIcon),
            ("iOS 7 Settings", 58.0, 58.0, AppIcon),
            ("Mac Icon 1024", 1024.0, 1024.0, AppIcon),
            ("Mac Icon 512", 512.0, 512.0, AppIcon),
            ("Mac Icon 256", 256.0, 256.0, AppIcon),
            ("Mac Icon 128", 128.0, 128.0, AppIcon),
            ("Mac Icon 48", 48.0, 48.0, AppIcon),
            ("Mac Icon 32", 32.0, 32.0, AppIcon),
        ]
        .into_iter()
        .map(|(n, w, h, k)| px(n, w, h, 72.0, k))
        .collect(),
        _ => [
            ("HDTV 1080p", 1920.0, 1080.0, None),
            ("HDV/HDTV 720p", 1280.0, 720.0, None),
            ("HDV 1080p", 1440.0, 1080.0, None),
            ("DVCPRO HD 720p", 960.0, 720.0, None),
            ("DVCPRO HD 1080p", 1280.0, 1080.0, None),
            ("DCI 2K 1080p", 2048.0, 1080.0, Some("2K")),
            ("UHDTV/4K 2160p", 3840.0, 2160.0, Some("4K")),
            ("DCI 4K 2160p", 4096.0, 2160.0, Some("4K")),
            ("FUHDTV/8K 4320p", 7680.0, 4320.0, Some("8K")),
            ("DCI 8K 4320p", 8192.0, 4320.0, Some("8K")),
            ("NTSC DV", 720.0, 480.0, None),
            ("NTSC DV Widescreen", 720.0, 480.0, None),
            ("NTSC D1", 720.0, 486.0, None),
            ("NTSC D1 Widescreen", 720.0, 486.0, None),
            ("NTSC D1 Square Pixel", 720.0, 534.0, None),
            ("NTSC D1 Widescreen Square Pixel", 872.0, 486.0, None),
            ("PAL D1/DV", 720.0, 576.0, None),
            ("PAL D1/DV Widescreen", 720.0, 576.0, None),
            ("PAL D1/DV Square Pixel", 788.0, 576.0, None),
            ("PAL D1/DV Widescreen Square Pixel", 1050.0, 576.0, None),
            ("Cineon Half", 1828.0, 1332.0, None),
            ("Cineon Full", 3656.0, 2664.0, None),
            ("Film (2K)", 2048.0, 1556.0, Some("2K")),
            ("Film (4K)", 4096.0, 3112.0, Some("4K")),
        ]
        .into_iter()
        .map(|(n, w, h, badge)| px(n, w, h, 72.0, Video(badge)))
        .collect(),
    }
}

/// Draws a card's icon centered on `c`: a frame of the preset's shape,
/// scaled by `scale` (0–1, the preset's size among its category's), in
/// 1 pt `ink` lines.
pub fn paint_icon(painter: &Painter, c: Pos2, preset: &Preset, scale: f32, ink: Color32) {
    let stroke = Stroke::new(pt(1.0), ink);
    let (w, h) = preset.pixels();
    let aspect = (w / h.max(1.0)) as f32;
    // The longest side runs from 36 pt (smallest) to 82 pt (largest),
    // the other side at most 62 pt (as Photoshop's photo cards measure)
    let long = pt(36.0 + 46.0 * scale.clamp(0.0, 1.0).sqrt());
    let mut size = if aspect >= 1.0 {
        vec2(long, long / aspect)
    } else {
        vec2(long * aspect, long)
    };
    let short = size.x.min(size.y);
    if short > pt(62.0) {
        size *= pt(62.0) / short;
    }
    let r = Rect::from_center_size(c, size);
    let line = |a: Pos2, b: Pos2| painter.line_segment([a, b], stroke);
    match preset.kind {
        Kind::Custom | Kind::Page => {
            // A page with a folded corner; custom ones have a crosshair off
            // the top left corner
            let fold = r.width().min(r.height()) * 0.18;
            painter.add(Shape::closed_line(
                vec![
                    r.left_top(),
                    pos2(r.right() - fold, r.top()),
                    pos2(r.right(), r.top() + fold),
                    r.right_bottom(),
                    r.left_bottom(),
                ],
                stroke,
            ));
            painter.add(Shape::line(
                vec![
                    pos2(r.right() - fold, r.top()),
                    pos2(r.right() - fold, r.top() + fold),
                    pos2(r.right(), r.top() + fold),
                ],
                stroke,
            ));
            if preset.kind == Kind::Custom {
                let o = r.left_top();
                line(o + vec2(-pt(16.0), 0.0), o + vec2(-pt(8.0), 0.0));
                line(o + vec2(0.0, -pt(16.0)), o + vec2(0.0, -pt(8.0)));
            }
        }
        Kind::Photo => {
            painter.rect_stroke(r, pt(1.0), stroke, StrokeKind::Inside);
            let s = r.width().min(r.height());
            painter.circle_stroke(r.right_top() + vec2(-s * 0.22, s * 0.22), s * 0.1, stroke);
            let b = r.bottom() - s * 0.12;
            painter.add(Shape::line(
                vec![
                    pos2(r.left() + s * 0.08, b),
                    pos2(r.left() + r.width() * 0.38, r.bottom() - r.height() * 0.5),
                    pos2(r.left() + r.width() * 0.62, b),
                ],
                stroke,
            ));
            painter.add(Shape::line(
                vec![
                    pos2(r.left() + r.width() * 0.55, b - s * 0.15),
                    pos2(r.left() + r.width() * 0.72, r.bottom() - r.height() * 0.38),
                    pos2(r.right() - s * 0.08, b),
                ],
                stroke,
            ));
        }
        Kind::Grid => {
            painter.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
            let n = 7;
            for i in 1..n {
                for j in 1..n {
                    let p = r.min
                        + vec2(
                            r.width() * i as f32 / n as f32,
                            r.height() * j as f32 / n as f32,
                        );
                    painter.circle_filled(p, pt(0.6), ink);
                }
            }
        }
        Kind::Brush => {
            painter.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
            let s = r.width().min(r.height());
            let tip = r.center() + vec2(-s * 0.2, s * 0.2);
            line(tip, r.center() + vec2(s * 0.25, -s * 0.25));
            painter.circle_stroke(tip, s * 0.07, stroke);
        }
        Kind::Browser => {
            painter.rect_stroke(r, pt(1.0), stroke, StrokeKind::Inside);
            let y = r.top() + r.height().min(pt(30.0)) * 0.18;
            line(pos2(r.left(), y), pos2(r.right(), y));
        }
        Kind::Phone | Kind::Tablet | Kind::Strip => {
            painter.rect_stroke(r, pt(3.0), stroke, StrokeKind::Inside);
            if preset.kind != Kind::Strip {
                painter.circle_stroke(pos2(r.center().x, r.bottom() - pt(4.0)), pt(1.5), stroke);
            }
        }
        Kind::Surface => {
            painter.rect_stroke(r, pt(1.5), stroke, StrokeKind::Inside);
            line(
                pos2(r.right() + pt(4.0), r.top() + pt(2.0)),
                pos2(r.right() + pt(4.0), r.bottom() - pt(2.0)),
            );
        }
        Kind::Watch => {
            painter.rect_stroke(r, pt(5.0), stroke, StrokeKind::Inside);
            let band = vec2(r.width() * 0.25, 0.0);
            line(
                pos2(r.center().x, r.top()) - band,
                pos2(r.center().x, r.top() - pt(5.0)) - band,
            );
            line(
                pos2(r.center().x, r.top()) + band,
                pos2(r.center().x, r.top() - pt(5.0)) + band,
            );
            line(
                pos2(r.center().x, r.bottom()) - band,
                pos2(r.center().x, r.bottom() + pt(5.0)) - band,
            );
            line(
                pos2(r.center().x, r.bottom()) + band,
                pos2(r.center().x, r.bottom() + pt(5.0)) + band,
            );
        }
        Kind::AppIcon => {
            painter.rect_stroke(r, r.width() * 0.2, stroke, StrokeKind::Inside);
            let n = 6;
            for i in 1..n {
                for j in 1..n {
                    let p = r.min
                        + vec2(
                            r.width() * i as f32 / n as f32,
                            r.height() * j as f32 / n as f32,
                        );
                    painter.circle_filled(p, pt(0.5), ink);
                }
            }
        }
        Kind::Video(badge) => {
            painter.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
            let s = r.height() * 0.35;
            let c = r.center();
            painter.add(Shape::closed_line(
                vec![
                    c + vec2(-s * 0.4, -s * 0.5),
                    c + vec2(s * 0.5, 0.0),
                    c + vec2(-s * 0.4, s * 0.5),
                ],
                stroke,
            ));
            if let Some(b) = badge {
                painter.text(
                    r.right_top() + vec2(-pt(2.5), pt(1.5)),
                    egui::Align2::RIGHT_TOP,
                    b,
                    crate::theme::uxp(pt(8.0)),
                    ink,
                );
            }
        }
    }
}

/// Each preset's size within its list, 0 (smallest) to 1 (largest), by
/// the square root of its area in pixels.
pub fn scales(presets: &[Preset]) -> Vec<f32> {
    let areas: Vec<f64> = presets
        .iter()
        .map(|p| {
            let (w, h) = p.pixels();
            (w * h).sqrt()
        })
        .collect();
    let (lo, hi) = areas
        .iter()
        .fold((f64::MAX, 0.0f64), |(lo, hi), &a| (lo.min(a), hi.max(a)));
    areas
        .iter()
        .map(|a| {
            if hi > lo {
                ((a - lo) / (hi - lo)) as f32
            } else {
                1.0
            }
        })
        .collect()
}

/// Where the New Document dialog's Recent and Saved presets are kept
/// across launches: `~/Library/Application Support/OpenPhoto/
/// new-document-presets.txt`.
pub fn store_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        std::path::PathBuf::from(home)
            .join("Library/Application Support/OpenPhoto/new-document-presets.txt"),
    )
}

fn kind_name(kind: Kind) -> String {
    match kind {
        Kind::Custom => "custom".into(),
        Kind::Photo => "photo".into(),
        Kind::Page => "page".into(),
        Kind::Grid => "grid".into(),
        Kind::Brush => "brush".into(),
        Kind::Browser => "browser".into(),
        Kind::Phone => "phone".into(),
        Kind::Tablet => "tablet".into(),
        Kind::Watch => "watch".into(),
        Kind::AppIcon => "app-icon".into(),
        Kind::Surface => "surface".into(),
        Kind::Strip => "strip".into(),
        Kind::Video(None) => "video".into(),
        Kind::Video(Some(badge)) => format!("video:{badge}"),
    }
}

fn kind_from(name: &str) -> Option<Kind> {
    Some(match name {
        "custom" => Kind::Custom,
        "photo" => Kind::Photo,
        "page" => Kind::Page,
        "grid" => Kind::Grid,
        "brush" => Kind::Brush,
        "browser" => Kind::Browser,
        "phone" => Kind::Phone,
        "tablet" => Kind::Tablet,
        "watch" => Kind::Watch,
        "app-icon" => Kind::AppIcon,
        "surface" => Kind::Surface,
        "strip" => Kind::Strip,
        "video" => Kind::Video(None),
        _ => {
            // A badge is one of Film & Video's
            let badge = name.strip_prefix("video:")?;
            return category(5)
                .into_iter()
                .find_map(|p| match p.kind {
                    Kind::Video(Some(b)) if b == badge => Some(p.kind),
                    _ => None,
                })
                .or(Some(Kind::Video(None)));
        }
    })
}

/// Recent and Saved as lines: `recent` or `saved`, then the name, width,
/// height, unit, ppi and card kind, separated by tabs.
pub fn serialize(recent: &[Preset], saved: &[Preset]) -> String {
    let clean = |t: &str| t.replace(['\t', '\n'], " ");
    let line = |list: &str, p: &Preset| {
        let unit = Unit::ALL.iter().position(|u| *u == p.unit).unwrap_or(0);
        format!(
            "{list}\t{}\t{}\t{}\t{unit}\t{}\t{}\n",
            clean(&p.name),
            p.width,
            p.height,
            p.ppi,
            kind_name(p.kind)
        )
    };
    recent
        .iter()
        .map(|p| line("recent", p))
        .chain(saved.iter().map(|p| line("saved", p)))
        .collect()
}

/// The lists [`serialize`] wrote; lines that don't read are skipped.
pub fn parse(text: &str) -> (Vec<Preset>, Vec<Preset>) {
    let (mut recent, mut saved) = (Vec::new(), Vec::new());
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let [list, name, width, height, unit, ppi, kind] = f[..] else {
            continue;
        };
        let (Ok(width), Ok(height), Ok(unit), Ok(ppi), Some(kind)) = (
            width.parse::<f64>(),
            height.parse::<f64>(),
            unit.parse::<usize>(),
            ppi.parse::<f64>(),
            kind_from(kind),
        ) else {
            continue;
        };
        let Some(&unit) = Unit::ALL.get(unit) else {
            continue;
        };
        let preset = Preset::new(name, (width, height), unit, ppi, kind);
        match list {
            "recent" => recent.push(preset),
            "saved" => saved.push(preset),
            _ => {}
        }
    }
    (recent, saved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_and_saved_round_trip() {
        let recent = vec![category(5)[2].clone(), category(1)[0].clone()];
        let mut mine = Preset::new(
            "My\tSize",
            (640.0, 480.0),
            Unit::Pixels,
            144.0,
            Kind::Custom,
        );
        let saved = vec![mine.clone()];
        let (r, s) = parse(&serialize(&recent, &saved));
        assert_eq!(r, recent);
        mine.name = "My Size".into();
        assert_eq!(s, vec![mine]);
        // Damaged lines are skipped
        assert_eq!(parse("recent\tx\t1\n").0.len(), 0);
    }

    #[test]
    fn presets_convert_to_pixels() {
        let photo = category(0);
        assert_eq!(photo.len(), 9);
        assert_eq!(photo[0].pixels(), (2100.0, 1500.0));
        assert_eq!(photo[0].size_label(), "7 x 5 in @ 300 ppi");
        let a4 = &category(1)[3];
        assert_eq!(a4.name, "A4");
        let (w, h) = a4.pixels();
        assert_eq!((w.round(), h.round()), (2480.0, 3508.0));
        assert_eq!(category(3)[0].size_label(), "1366 x 768 px @ 72 ppi");
        assert_eq!(Unit::Inches.format(2.77777), "2.778");
    }
}
