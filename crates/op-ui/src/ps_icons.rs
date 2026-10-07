//! Icons traced from Photoshop 2026's options bar at 2x and drawn as vector
//! shapes, so they match its pixels instead of approximating them with an
//! icon font. Coordinates are Photoshop device pixels at 2x (half points),
//! relative to the icon's center.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

use crate::theme::pt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Home,
    Move,
    Caret,
    Share,
    Bell,
    Search,
    Lightbulb,
    Workspace,
    AlignLeft,
    AlignHorizontalCenter,
    AlignRight,
    DistributeVertically,
    AlignTop,
    AlignVerticalCenter,
    AlignBottom,
    DistributeHorizontally,
    More,
    Gear,
    /// The Crop tool's options bar.
    Straighten,
    CropOverlay,
    Info,
    CropReset,
    CropCancel,
    CropCommit,
    /// Swap width and height: two solid arrows.
    Swap,
    /// Free Transform's switch to Warp: a globe over a bent bar.
    WarpToggle,
    /// The Tool Presets icon while transforming: a box with handles and
    /// an arrow.
    TransformPreset,
    /// The toolbar's collapse "»": pixel-aligned and bold.
    CollapseToolbar,
    /// The panel column's thin collapse "»".
    CollapseRight,
    /// The icon strip's thin expand "«".
    CollapseLeft,
    /// The History panel's icon: three stacked squares and a curved arrow.
    History,
    /// The Comments panel's icon: a filled speech bubble.
    Comments,
    /// The larger V of dialog dropdowns (10 × 6 pt).
    DialogChevron,
    /// A Layers panel group row: expanded (V) and collapsed (>) arrows, and
    /// the folder.
    GroupExpanded,
    GroupCollapsed,
    Folder,
    // The Layers panel: filter buttons, lock buttons, the visibility eye,
    // the background's lock badge and the footer buttons
    FilterPixel,
    FilterAdjustment,
    FilterType,
    FilterShape,
    FilterSmartObject,
    LockTransparent,
    LockPixels,
    LockPosition,
    LockArtboards,
    LockAll,
    /// "Use tablet pressure" (the Magnetic Lasso's width, the brushes'
    /// size): rings around a pen.
    PenPressure,
    /// Object Selection: refresh the object finder.
    Refresh,
    /// Object Selection: show all objects (a frame round three blocks).
    ObjectFinder,
    /// Object Selection: send feedback (a dotted frame and a speech bubble).
    Feedback,
    /// Quick Selection's modes: a dotted circle and a brush, plus or minus.
    QuickNew,
    QuickAdd,
    QuickSubtract,
    /// Brush angle.
    Angle,
    /// Toggle the Brush Settings panel: a square with a brush cut out.
    BrushPanel,
    /// Toggle the Clone Source panel: a square with a stamp cut out.
    CloneSourcePanel,
    /// Pressure for opacity: a hatched ring and a pen.
    OpacityPressure,
    /// Airbrush: a spray and a pen.
    Airbrush,
    /// Painting symmetry: a butterfly (with a menu).
    Symmetry,
    /// Ignore adjustment layers when cloning.
    IgnoreAdjustments,
    /// Color Replacement / Background Eraser sampling: continuous, once,
    /// background swatch.
    SampleContinuous,
    SampleOnce,
    SampleSwatch,
    /// Mixer Brush: load the brush after each stroke, clean it.
    MixerLoad,
    MixerClean,
    /// The gradient types: 14 pt squares filled with each kind of ramp.
    GradientLinear,
    GradientRadial,
    GradientAngle,
    GradientReflected,
    GradientDiamond,
    /// The Notes panel (a note with lines).
    NotesPanel,
    /// Path operations, alignment and arrangement (each with a menu).
    PathOperations,
    /// Path operations with nothing to combine yet (a filled square).
    PathCombine,
    PathAlignment,
    PathArrangement,
    /// The gear with a menu.
    GearMenu,
    /// Type: toggle the orientation; the font size; the three
    /// alignments; warp text; 3D; the Character and Paragraph panels.
    TextOrientation,
    FontSize,
    TextLeft,
    TextCenter,
    TextRight,
    WarpText,
    Text3d,
    CharacterPanels,
    /// Link width and height (a chain link).
    Link,
    /// Corner radius (a quarter circle).
    CornerRadius,
    /// The number of polygon sides (a hexagon with #).
    PolygonSides,
    /// Zoom in and zoom out (magnifiers with + and −).
    ZoomIn,
    ZoomOut,
    /// Vertical type's alignments: top, center, bottom.
    TextTop,
    TextMiddle,
    TextBottom,
    /// Bring to front, bring forward, send backward, send to back (the
    /// Slice Select tool).
    ArrangeFront,
    ArrangeForward,
    ArrangeBackward,
    ArrangeBack,
    /// The Artboard tool: make portrait or landscape; add an artboard.
    ArtboardPortrait,
    ArtboardLandscape,
    AddArtboard,
    /// The Frame tool's shapes: rectangle, ellipse, triangle, hexagon and
    /// custom, each crossed.
    FrameRect,
    FrameEllipse,
    FrameTriangle,
    FrameHexagon,
    FrameCustom,
    /// A stroke centered on its edge (a square with handles).
    StrokeCenter,
    /// A paintbrush with a plus or a minus (adding to or taking from a
    /// selection or an area: Selection Brush, Remove, Adjustment Brush).
    BrushAdd,
    BrushSubtract,
    Eye,
    /// A partly locked layer's lock (and the background's): hollow.
    LayerLock,
    /// A fully locked layer's lock: solid, with a keyhole.
    LayerLockFull,
    FooterBrush,
    LinkLayers,
    LayerStyle,
    LayerMask,
    NewAdjustment,
    NewGroup,
    NewLayer,
    DeleteLayer,
    /// Hue/Saturation's targeted adjustment hand.
    TargetedHand,
    Eyedropper,
    EyedropperPlus,
    EyedropperMinus,
    /// Levels' and Curves' Set Black/Gray/White Point eyedroppers: the
    /// tube's lower half holds that color.
    EyedropperBlack,
    EyedropperGray,
    EyedropperWhite,
    /// Curves' point tool: a wave through four points.
    CurvePoints,
    /// Curves' pencil tool.
    Pencil,
    /// Curves' targeted adjustment: a hand with up and down arrows.
    TargetedHandVertical,
    /// Curves' quarter grid button.
    GridQuarters,
    /// Curves' ten-by-ten grid button.
    GridTenths,
    /// The presets menu button (three bars and a corner).
    PresetMenu,
}

#[derive(Clone, Copy)]
struct Pen<'a> {
    painter: &'a Painter,
    center: Pos2,
    color: Color32,
    /// 1 for the size traced; smaller copies (e.g. the Lock Layers
    /// dialog's icons) scale everything.
    scale: f32,
}

impl Pen<'_> {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        self.center + Vec2::new(pt(x / 2.0), pt(y / 2.0)) * self.scale
    }

    fn w(&self, w: f32) -> f32 {
        pt(w / 2.0) * self.scale
    }

    fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.painter.rect_filled(
            Rect::from_min_max(self.p(x0, y0), self.p(x1, y1)),
            0,
            self.color,
        );
    }

    fn poly(&self, points: &[(f32, f32)]) {
        let points = points.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.painter
            .add(Shape::convex_polygon(points, self.color, Stroke::NONE));
    }

    /// A filled outline given as rows of (y, left, right), top to bottom.
    fn profile(&self, rows: &[(f32, f32, f32)]) {
        for pair in rows.windows(2) {
            let ((y0, l0, r0), (y1, l1, r1)) = (pair[0], pair[1]);
            self.poly(&[(l0, y0), (r0, y0), (r1, y1), (l1, y1)]);
        }
    }

    fn line(&self, points: &[(f32, f32)], width: f32) {
        let points = points.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.painter
            .add(Shape::line(points, Stroke::new(self.w(width), self.color)));
    }

    fn round_line(&self, a: (f32, f32), b: (f32, f32), width: f32) {
        self.line(&[a, b], width);
        for (x, y) in [a, b] {
            self.painter
                .circle_filled(self.p(x, y), self.w(width) / 2.0, self.color);
        }
    }

    fn ring(&self, x: f32, y: f32, r: f32, width: f32) {
        self.painter.circle_stroke(
            self.p(x, y),
            self.w(r),
            Stroke::new(self.w(width), self.color),
        );
    }

    /// An arc of radius `r` around (x, y) from `a0` to `a1` (radians,
    /// clockwise from 3 o'clock).
    fn arc(&self, x: f32, y: f32, r: f32, (a0, a1): (f32, f32), width: f32) {
        let n = ((a1 - a0).abs() * r / 2.0).ceil().max(4.0) as usize;
        let points: Vec<(f32, f32)> = (0..=n)
            .map(|i| {
                let a = a0 + (a1 - a0) * i as f32 / n as f32;
                (x + r * a.cos(), y + r * a.sin())
            })
            .collect();
        self.line(&points, width);
    }

    fn dot(&self, x: f32, y: f32, r: f32) {
        self.painter
            .circle_filled(self.p(x, y), self.w(r), self.color);
    }
}

/// The small triangle at an icon's bottom right that marks a menu.
fn menu_mark(pen: &Pen) {
    pen.poly(&[(10.0, 15.5), (17.5, 15.5), (13.75, 19.0)]);
}

/// Paints `icon` centered on `center`. `background` is the color behind it,
/// used for the gear's hole.
pub fn paint(painter: &Painter, center: Pos2, icon: Icon, color: Color32, background: Color32) {
    paint_scaled(painter, center, icon, color, background, 1.0);
}

/// [`paint`] at `scale` times the traced size.
pub fn paint_scaled(
    painter: &Painter,
    center: Pos2,
    icon: Icon,
    color: Color32,
    background: Color32,
    scale: f32,
) {
    let pen = Pen {
        painter,
        center,
        color,
        scale,
    };
    match icon {
        Icon::Home => {
            pen.profile(&[(-14.5, 0.0, 0.0), (2.0, -16.5, 16.5), (4.5, -14.5, 14.5)]);
            pen.rect(-12.0, 2.0, -4.0, 14.5);
            pen.rect(4.0, 2.0, 12.0, 14.5);
        }
        Icon::Move => {
            pen.rect(-1.0, -9.0, 1.0, 9.0);
            pen.rect(-15.0, -1.0, 15.0, 1.0);
            pen.poly(&[(0.0, -15.0), (5.0, -9.0), (-5.0, -9.0)]);
            pen.poly(&[(0.0, 15.0), (-5.0, 9.0), (5.0, 9.0)]);
            pen.poly(&[(-15.5, 0.0), (-9.0, -5.0), (-9.0, 5.0)]);
            pen.poly(&[(15.5, 0.0), (9.0, 5.0), (9.0, -5.0)]);
        }
        Icon::Caret => pen.line(&[(-6.0, -3.0), (0.0, 3.0), (6.0, -3.0)], 2.2),
        Icon::GroupExpanded => pen.line(&[(-7.0, -4.0), (0.0, 4.0), (7.0, -4.0)], 3.0),
        Icon::GroupCollapsed => pen.line(&[(-4.0, -7.0), (4.0, 0.0), (-4.0, 7.0)], 3.0),
        Icon::Folder => {
            // Traced around (2163, 1310) at 2x: the tab's outline, the top
            // edge of the front, and the filled front
            pen.rect(-14.0, -11.0, 0.0, -9.0);
            pen.rect(-14.0, -11.0, -12.0, 0.0);
            pen.rect(-4.0, -9.0, -2.0, -7.0);
            pen.rect(-4.0, -7.0, 13.0, -5.0);
            pen.rect(11.0, -5.0, 13.0, 0.0);
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(-14.0, 0.0), pen.p(13.0, 11.0)),
                pt(0.5),
                color,
            );
        }
        Icon::DialogChevron => pen.line(&[(-8.5, -4.5), (0.0, 4.0), (8.5, -4.5)], 3.5),
        Icon::Share => {
            // The tray, open at the top around the arrow
            pen.line(
                &[
                    (-8.0, -5.0),
                    (-14.0, -5.0),
                    (-14.0, 15.0),
                    (14.0, 15.0),
                    (14.0, -5.0),
                    (8.0, -5.0),
                ],
                4.0,
            );
            pen.rect(-2.0, -9.5, 2.0, 2.0);
            pen.poly(&[(0.0, -17.5), (8.0, -9.5), (-8.0, -9.5)]);
        }
        Icon::Bell => {
            pen.rect(-2.0, -17.0, 2.0, -13.0);
            pen.profile(&[
                (-13.0, -5.0, 5.0),
                (-10.0, -9.0, 9.0),
                (-5.0, -9.5, 9.5),
                (0.0, -10.0, 10.0),
                (4.0, -11.0, 11.0),
                (8.0, -14.0, 14.0),
                (11.0, -14.0, 14.0),
            ]);
            pen.profile(&[(13.0, -4.0, 4.0), (15.0, -3.0, 3.0), (16.5, -1.0, 1.0)]);
        }
        Icon::Search => {
            pen.ring(-2.5, -2.25, 10.5, 3.0);
            pen.round_line((5.0, 6.0), (12.0, 12.0), 4.5);
        }
        Icon::Lightbulb => {
            pen.ring(-0.5, -0.5, 7.5, 3.0);
            pen.line(&[(-4.5, 6.0), (-4.0, 10.0)], 3.0);
            pen.line(&[(3.5, 6.0), (3.0, 10.0)], 3.0);
            pen.profile(&[(10.0, -5.0, 4.0), (14.5, -5.0, 4.0), (16.5, -2.5, 1.5)]);
            pen.line(&[(-0.5, -16.5), (-0.5, -11.5)], 3.0);
            pen.line(&[(-12.0, -10.5), (-9.5, -8.0)], 3.0);
            pen.line(&[(11.0, -10.5), (8.5, -8.0)], 3.0);
            pen.line(&[(-17.0, 0.0), (-11.5, 0.0)], 3.0);
            pen.line(&[(10.5, 0.0), (16.0, 0.0)], 3.0);
        }
        Icon::Workspace => {
            pen.rect(-16.0, -13.0, 16.0, -9.0);
            pen.rect(-16.0, -9.0, -14.0, 13.0);
            pen.rect(14.0, -9.0, 16.0, 13.0);
            pen.rect(-16.0, 11.0, 16.0, 13.0);
            // The panel strip: three stacked cells
            for k in 0..4 {
                let y = -7.0 + 4.0 * k as f32;
                pen.rect(-12.0, y, -6.0, y + 2.0);
            }
            pen.rect(-12.0, -7.0, -10.0, 7.0);
            pen.rect(-8.0, -7.0, -6.0, 7.0);
        }
        Icon::AlignLeft => {
            pen.rect(-12.5, -15.0, -10.5, 15.0);
            pen.rect(-8.5, -9.0, 5.5, -1.0);
            pen.rect(-8.5, 3.0, 12.5, 11.0);
        }
        Icon::AlignHorizontalCenter => {
            pen.rect(-1.0, -15.0, 1.0, 15.0);
            pen.rect(-7.0, -10.0, 7.0, -2.0);
            pen.rect(-11.0, 2.0, 10.0, 10.0);
        }
        Icon::AlignRight => {
            pen.rect(10.5, -15.0, 12.5, 15.0);
            pen.rect(-5.5, -9.0, 8.5, -1.0);
            pen.rect(-12.5, 3.0, 8.5, 11.0);
        }
        Icon::DistributeVertically => {
            pen.rect(-16.0, -10.0, 16.0, -8.0);
            pen.rect(-10.0, -4.0, 10.0, 4.0);
            pen.rect(-16.0, 8.0, 16.0, 10.0);
        }
        Icon::AlignTop => {
            pen.rect(-14.0, -12.5, 14.0, -10.5);
            pen.rect(-10.0, -8.5, -2.0, 12.5);
            pen.rect(2.0, -8.5, 10.0, 5.5);
        }
        Icon::AlignVerticalCenter => {
            pen.rect(-14.0, -1.0, 14.0, 1.0);
            pen.rect(-10.0, -11.0, -2.0, 10.0);
            pen.rect(2.0, -7.0, 10.0, 7.0);
        }
        Icon::AlignBottom => {
            pen.rect(-14.0, 10.5, 14.0, 12.5);
            pen.rect(-10.0, -12.5, -2.0, 8.5);
            pen.rect(2.0, -5.5, 10.0, 8.5);
        }
        Icon::DistributeHorizontally => {
            pen.rect(-10.0, -16.0, -8.0, 16.0);
            pen.rect(8.0, -16.0, 10.0, 16.0);
            pen.rect(-4.0, -10.0, 4.0, 10.0);
        }
        Icon::More => {
            for x in [-12.0, 0.0, 12.0] {
                pen.dot(x, 0.0, 4.0);
            }
        }
        Icon::CollapseToolbar => {
            for left in [-6.5, 1.5] {
                let rows = [
                    (-5.0, 0.0, 1.0),
                    (-3.0, 0.0, 3.0),
                    (-1.0, 2.0, 5.0),
                    (1.0, 2.0, 5.0),
                    (3.0, 0.0, 3.0),
                    (5.0, 0.0, 1.0),
                ]
                .map(|(y, l, r)| (y, left + l, left + r));
                pen.profile(&rows);
            }
        }
        Icon::CollapseRight | Icon::CollapseLeft => {
            let flip = if icon == Icon::CollapseLeft {
                -1.0
            } else {
                1.0
            };
            for dx in [-8.0, 0.0] {
                pen.line(
                    &[
                        (flip * (dx + 2.5), -4.0),
                        (flip * (dx + 5.5), 0.0),
                        (flip * (dx + 2.5), 4.0),
                    ],
                    1.3,
                );
            }
        }
        Icon::History => {
            // Traced around (2013, 195): two outlined squares over a filled
            // one, and an arrow curving from the top right down to the left
            for y in [-17.0, -5.0] {
                pen.rect(-14.0, y, -4.0, y + 2.0);
                pen.rect(-14.0, y + 8.0, -4.0, y + 10.0);
                pen.rect(-14.0, y, -12.0, y + 10.0);
                pen.rect(-6.0, y, -4.0, y + 10.0);
            }
            pen.rect(-14.0, 7.0, -4.0, 17.0);
            let arc: Vec<(f32, f32)> = (0..=12)
                .map(|k| {
                    let a = (-40.0 + 135.0 * k as f32 / 12.0).to_radians();
                    (11.0 * a.cos(), 1.0 + 11.0 * a.sin())
                })
                .collect();
            pen.line(&arc, 4.0);
            pen.poly(&[(2.0, -14.0), (13.5, -14.0), (2.0, -2.5)]);
        }
        Icon::Comments => {
            // Traced around (2013, 251)
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(-15.0, -15.0), pen.p(15.0, 6.0)),
                pt(1.0),
                pen.color,
            );
            pen.poly(&[(-10.0, 5.0), (0.0, 5.0), (-9.5, 14.5)]);
        }
        Icon::FilterPixel => {
            pen.painter.rect_stroke(
                Rect::from_min_max(pen.p(-12.0, -10.0), pen.p(12.0, 10.0)),
                0,
                Stroke::new(pen.w(2.0), color),
                egui::StrokeKind::Inside,
            );
            pen.poly(&[(-6.5, 6.0), (-3.5, -3.5), (3.0, 6.0)]);
            pen.poly(&[(-1.0, 6.0), (6.0, -2.5), (10.5, 6.0)]);
        }
        Icon::FilterAdjustment | Icon::NewAdjustment => {
            pen.ring(0.0, 0.0, 9.0, 2.0);
            let half: Vec<(f32, f32)> = (0..=12)
                .map(|k| {
                    let a = (-45.0 + 180.0 * k as f32 / 12.0f32).to_radians();
                    (9.0 * a.cos(), 9.0 * a.sin())
                })
                .collect();
            pen.poly(&half);
            if icon == Icon::NewAdjustment {
                pen.poly(&[(-1.0, 13.5), (7.0, 13.5), (3.0, 17.0)]);
            }
        }
        Icon::FilterType => {
            pen.rect(-9.0, -9.5, 9.0, -6.5);
            pen.rect(-9.0, -6.5, -6.0, -3.5);
            pen.rect(6.0, -6.5, 9.0, -3.5);
            pen.rect(-1.5, -6.5, 1.5, 7.5);
            pen.rect(-5.5, 7.5, 5.5, 10.0);
        }
        Icon::FilterShape => {
            pen.rect(-5.0, -9.0, 5.0, -7.0);
            pen.rect(-5.0, 7.0, 5.0, 9.0);
            pen.rect(-9.0, -5.0, -7.0, 5.0);
            pen.rect(7.0, -5.0, 9.0, 5.0);
            for (x, y) in [(-8.0, -8.0), (8.0, -8.0), (-8.0, 8.0), (8.0, 8.0)] {
                pen.painter.rect_stroke(
                    Rect::from_center_size(pen.p(x, y), Vec2::splat(pen.w(6.0))),
                    0,
                    Stroke::new(pen.w(2.0), color),
                    egui::StrokeKind::Inside,
                );
            }
        }
        Icon::FilterSmartObject => {
            pen.line(
                &[
                    (-6.0, 0.0),
                    (-6.0, -9.0),
                    (4.5, -9.0),
                    (8.0, -5.5),
                    (8.0, 9.0),
                    (1.0, 9.0),
                ],
                2.0,
            );
            pen.line(&[(1.0, -9.0), (1.0, -4.0), (8.0, -4.0)], 2.0);
            pen.rect(-11.0, 0.0, 1.0, 12.0);
        }
        Icon::LockTransparent => {
            pen.painter.rect_stroke(
                Rect::from_min_max(pen.p(-11.0, -11.0), pen.p(11.0, 11.0)),
                0,
                Stroke::new(pen.w(2.0), color),
                egui::StrokeKind::Inside,
            );
            pen.rect(-3.0, -9.0, 3.0, -3.0);
            pen.rect(-9.0, -3.0, -3.0, 3.0);
            pen.rect(3.0, -3.0, 9.0, 3.0);
            pen.rect(-3.0, 3.0, 3.0, 9.0);
        }
        Icon::LockPixels | Icon::FooterBrush => {
            pen.round_line((12.0, -11.0), (4.0, 0.0), 5.0);
            pen.poly(&[
                (-12.0, 13.0),
                (-10.0, 5.0),
                (-5.0, 2.0),
                (1.0, 4.0),
                (1.0, 8.0),
                (-4.0, 12.0),
            ]);
        }
        Icon::LockPosition => {
            pen.rect(-1.0, -7.5, 1.0, 7.5);
            pen.rect(-8.0, -1.0, 8.0, 1.0);
            pen.poly(&[(0.0, -12.5), (5.0, -7.5), (-5.0, -7.5)]);
            pen.poly(&[(0.0, 12.5), (-5.0, 7.5), (5.0, 7.5)]);
            pen.poly(&[(-15.0, 0.0), (-8.0, -5.0), (-8.0, 5.0)]);
            pen.poly(&[(15.0, 0.0), (8.0, 5.0), (8.0, -5.0)]);
        }
        Icon::LockArtboards => {
            pen.line(
                &[
                    (6.0, -7.0),
                    (-11.0, -7.0),
                    (-11.0, 7.0),
                    (9.0, 7.0),
                    (9.0, 0.0),
                ],
                2.0,
            );
            pen.poly(&[
                (2.0, -8.0),
                (6.0, -8.0),
                (10.0, -4.0),
                (10.0, 0.0),
                (2.0, 0.0),
            ]);
            for (x0, y0, x1, y1) in [
                (-16.0, -8.0, -13.0, -6.0),
                (11.0, -8.0, 14.0, -6.0),
                (-16.0, 6.0, -13.0, 8.0),
                (11.0, 6.0, 14.0, 8.0),
                (-10.0, -12.0, -8.0, -9.0),
                (8.0, -12.0, 10.0, -9.0),
                (-10.0, 10.0, -8.0, 13.0),
                (8.0, 10.0, 10.0, 13.0),
            ] {
                pen.rect(x0, y0, x1, y1);
            }
        }
        Icon::LockAll => {
            let arc: Vec<(f32, f32)> = (0..=12)
                .map(|k| {
                    let a = (180.0 + 180.0 * k as f32 / 12.0f32).to_radians();
                    (5.5 * a.cos(), -6.0 + 5.5 * a.sin())
                })
                .collect();
            pen.line(&arc, 2.5);
            pen.rect(-6.75, -6.0, -4.25, -4.0);
            pen.rect(4.25, -6.0, 6.75, -4.0);
            pen.rect(-10.0, -4.0, 10.0, 12.0);
            Pen {
                color: background,
                ..pen
            }
            .rect(-2.0, 1.0, 2.0, 5.0);
        }
        Icon::Eye => {
            // Two arcs meeting at pointed corners, and a small pupil
            let arc = |sign: f32| -> Vec<(f32, f32)> {
                (0..=12)
                    .map(|k| {
                        let a = (-53.13 + 106.26 * k as f32 / 12.0f32).to_radians();
                        (12.5 * a.sin(), sign * (7.5 - 12.5 * a.cos()))
                    })
                    .collect()
            };
            pen.line(&arc(1.0), 4.0);
            pen.line(&arc(-1.0), 4.0);
            pen.dot(1.5, -1.5, 1.5);
        }
        Icon::LayerLock | Icon::LayerLockFull => {
            // Traced from Photoshop 2026 at 2x (one unit is a device pixel)
            let arc: Vec<(f32, f32)> = (0..=12)
                .map(|k| {
                    let a = (180.0 + 180.0 * k as f32 / 12.0f32).to_radians();
                    (-1.0 + 5.0 * a.cos(), -5.5 + 5.0 * a.sin())
                })
                .collect();
            pen.line(&arc, 2.5);
            pen.rect(-7.25, -5.5, -4.75, -4.0);
            pen.rect(2.75, -5.5, 5.25, -4.0);
            if icon == Icon::LayerLockFull {
                pen.rect(-11.0, -4.0, 9.0, 12.0);
                // The keyhole, 1 px above the hollow lock's dot
                pen.painter.rect_filled(
                    Rect::from_min_max(pen.p(-3.25, 1.0), pen.p(1.25, 6.0)),
                    pen.w(2.25),
                    background,
                );
            } else {
                pen.painter.rect_stroke(
                    Rect::from_min_max(pen.p(-11.0, -4.0), pen.p(9.0, 12.0)),
                    0,
                    Stroke::new(pen.w(2.0), color),
                    egui::StrokeKind::Inside,
                );
                // A round dot
                pen.painter.rect_filled(
                    Rect::from_min_max(pen.p(-4.0, 2.0), pen.p(2.0, 7.0)),
                    pen.w(2.5),
                    color,
                );
            }
        }
        Icon::LinkLayers => {
            for (cx, from) in [(-7.0f32, 40.0f32), (7.0, -140.0)] {
                let arc: Vec<(f32, f32)> = (0..=14)
                    .map(|k| {
                        let a = (from + 280.0 * k as f32 / 14.0).to_radians();
                        (cx + 6.0 * a.cos(), 6.0 * a.sin())
                    })
                    .collect();
                pen.line(&arc, 2.0);
            }
            pen.rect(-4.0, -1.0, 4.0, 1.0);
        }
        Icon::LayerStyle => {
            pen.line(
                &[(7.0, -11.0), (3.5, -11.0), (1.0, -8.0), (-4.0, 12.0)],
                2.0,
            );
            pen.line(&[(-6.0, -4.0), (2.0, -4.0)], 2.0);
            pen.line(&[(2.0, -4.0), (10.0, 8.0)], 2.0);
            pen.line(&[(10.0, -4.0), (2.0, 8.0)], 2.0);
            pen.poly(&[(20.0, 13.0), (27.0, 13.0), (23.5, 16.5)]);
        }
        Icon::LayerMask => {
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(-12.5, -9.5), pen.p(12.5, 9.5)),
                pt(0.5),
                color,
            );
            Pen {
                color: background,
                ..pen
            }
            .dot(0.0, 0.0, 7.0);
        }
        Icon::NewGroup => {
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(-11.0, -10.5), pen.p(0.0, -7.0)),
                pt(0.5),
                color,
            );
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(-12.0, -5.5), pen.p(12.0, 10.0)),
                pt(0.5),
                color,
            );
        }
        Icon::NewLayer => {
            pen.painter.rect_stroke(
                Rect::from_min_max(pen.p(-11.5, -11.5), pen.p(11.5, 11.5)),
                0,
                Stroke::new(pen.w(2.0), color),
                egui::StrokeKind::Inside,
            );
            pen.rect(-6.0, -1.0, 6.0, 1.0);
            pen.rect(-1.0, -6.0, 1.0, 6.0);
        }
        Icon::DeleteLayer => {
            pen.rect(-5.0, -12.0, 5.0, -10.0);
            pen.rect(-11.0, -8.0, 11.0, -6.0);
            pen.rect(-8.0, -6.0, -6.0, 12.0);
            pen.rect(6.0, -6.0, 8.0, 12.0);
            pen.rect(-8.0, 10.0, 8.0, 12.0);
            pen.rect(-3.0, -3.0, -1.0, 8.0);
            pen.rect(1.0, -3.0, 3.0, 8.0);
        }
        Icon::Straighten => {
            // A level: the bubble's dots above, two blocks and a cup
            for (x, y) in [
                (0.0, -11.5),
                (-6.0, -9.5),
                (6.0, -9.5),
                (-10.0, -5.5),
                (10.0, -5.5),
            ] {
                pen.rect(x - 1.0, y - 1.0, x + 1.0, y + 1.0);
            }
            pen.rect(-17.0, -0.5, -11.0, 12.5);
            pen.rect(11.0, -0.5, 17.0, 12.5);
            pen.rect(-9.0, -0.5, 9.0, 12.5);
            Pen {
                color: background,
                ..pen
            }
            .dot(0.0, -0.5, 6.5);
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(-5.0, -0.5), pen.p(5.0, 3.5)),
                pen.w(2.0),
                color,
            );
        }
        Icon::CropOverlay => {
            // A 3 × 3 grid with crop marks, and a menu triangle
            for x in [-11.5f32, -3.5, 4.5, 11.5] {
                pen.rect(x - 1.0, -12.5, x + 1.0, 12.5);
            }
            for y in [-11.5f32, -3.5, 4.5, 11.5] {
                pen.rect(-12.5, y - 1.0, 12.5, y + 1.0);
            }
            for x in [-6.0f32, 6.0] {
                pen.rect(x - 1.0, -16.5, x + 1.0, -12.5);
                pen.rect(x - 1.0, 12.5, x + 1.0, 16.5);
            }
            for y in [-6.0f32, 6.0] {
                pen.rect(-16.5, y - 1.0, -12.5, y + 1.0);
                pen.rect(12.5, y - 1.0, 16.5, y + 1.0);
            }
            pen.poly(&[(10.5, 15.5), (18.0, 15.5), (14.25, 19.0)]);
        }
        Icon::Info => {
            pen.ring(0.0, 0.0, 11.0, 2.5);
            pen.dot(0.0, -5.0, 1.75);
            pen.rect(-1.5, -1.0, 1.5, 7.5);
        }
        Icon::CropReset => {
            // A turning arrow over a bar
            let arc: Vec<(f32, f32)> = (0..=16)
                .map(|k| {
                    let a = (170.0 + 230.0 * k as f32 / 16.0f32).to_radians();
                    (1.0 + 8.5 * a.cos(), -0.5 + 8.5 * a.sin())
                })
                .collect();
            pen.line(&arc, 4.5);
            pen.poly(&[(-16.5, 1.5), (-1.5, 1.5), (-9.0, 10.0)]);
            pen.rect(-16.5, 15.0, 16.5, 17.0);
        }
        Icon::CropCancel => {
            pen.ring(0.0, 0.0, 13.0, 4.0);
            pen.line(&[(-9.0, -9.0), (9.0, 9.0)], 4.0);
        }
        Icon::TargetedHand => {
            // Traced from Photoshop 2026's Hue/Saturation at 2x
            pen.poly(&[(-13.0, -8.75), (-9.5, -10.5), (-9.5, -7.0)]);
            pen.poly(&[(10.5, -8.75), (7.0, -10.5), (7.0, -7.0)]);
            pen.round_line((-1.0, -9.0), (-1.0, 4.0), 3.5);
            pen.round_line((-8.0, -0.5), (-1.0, 10.0), 4.0);
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(-3.5, 0.0), pen.p(11.0, 15.5)),
                pen.w(5.5),
                pen.color,
            );
        }
        Icon::Eyedropper
        | Icon::EyedropperPlus
        | Icon::EyedropperMinus
        | Icon::EyedropperBlack
        | Icon::EyedropperGray
        | Icon::EyedropperWhite => {
            // A hollow tube from the lower left, a collar across it and
            // the bulb at the upper right
            let tube = [(-9.5, 9.5), (3.0, -3.0)];
            pen.round_line(tube[0], tube[1], 5.0);
            {
                let inner = Pen {
                    color: background,
                    ..pen
                };
                inner.round_line((-9.5, 9.5), (2.5, -2.5), 2.4);
                let liquid = match icon {
                    Icon::EyedropperBlack => Some(Color32::BLACK),
                    Icon::EyedropperGray => Some(Color32::from_gray(0xa0)),
                    Icon::EyedropperWhite => Some(Color32::WHITE),
                    _ => None,
                };
                if let Some(liquid) = liquid {
                    let fill = Pen {
                        color: liquid,
                        ..pen
                    };
                    fill.round_line((-9.5, 9.5), (-4.0, 4.0), 2.4);
                }
            }
            pen.round_line((0.5, -8.0), (7.5, -1.0), 4.0);
            pen.round_line((4.0, -4.5), (10.5, -11.0), 5.5);
            pen.dot(12.5, -13.0, 3.5);
            match icon {
                Icon::EyedropperPlus => {
                    pen.rect(4.4, 5.9, 14.4, 7.9);
                    pen.rect(8.4, 1.9, 10.4, 11.9);
                }
                Icon::EyedropperMinus => pen.rect(4.4, 5.5, 14.4, 7.5),
                _ => {}
            }
        }
        Icon::CurvePoints => {
            let pts = [(-16.0, 6.25), (-6.25, -7.5), (7.5, 8.75), (17.0, -6.25)];
            let mut curve = Vec::new();
            for k in 0..=24 {
                let t = k as f32 / 24.0;
                let x = -16.0 + 33.0 * t;
                curve.push((
                    x,
                    -7.5 * (std::f32::consts::PI * (x + 1.0) / 14.0).sin() * 0.95 + 0.6,
                ));
            }
            pen.line(&curve, 1.5);
            for (x, y) in pts {
                pen.dot(x, y, 1.8);
            }
        }
        Icon::Pencil => {
            let outline = [
                (-13.75, 13.75),
                (-11.0, 5.5),
                (9.5, -15.0),
                (15.0, -9.5),
                (-5.5, 11.0),
                (-13.75, 13.75),
            ];
            pen.line(&outline, 1.5);
            pen.line(&[(6.0, -11.5), (11.5, -6.0)], 1.5);
            pen.line(&[(-11.0, 5.5), (-5.5, 11.0)], 1.5);
        }
        Icon::TargetedHandVertical => {
            pen.poly(&[(-8.0, -9.5), (-11.0, -5.5), (-5.0, -5.5)]);
            pen.poly(&[(-8.0, 9.5), (-5.0, 5.5), (-11.0, 5.5)]);
            for (y, x0) in [(-4.0, -3.0), (0.0, -6.0), (4.0, -3.0)] {
                pen.round_line((x0, y), (4.0, y), 2.4);
            }
            pen.dot(7.0, 0.0, 6.5);
            pen.round_line((2.0, -6.5), (8.0, -6.5), 2.0);
        }
        Icon::GridQuarters => {
            pen.line(
                &[
                    (-13.0, -13.0),
                    (13.0, -13.0),
                    (13.0, 13.0),
                    (-13.0, 13.0),
                    (-13.0, -13.0),
                ],
                1.5,
            );
            pen.line(&[(0.0, -13.0), (0.0, 13.0)], 1.5);
            pen.line(&[(-13.0, 0.0), (13.0, 0.0)], 1.5);
        }
        Icon::GridTenths => {
            for k in 0..=4 {
                let v = -12.5 + 6.25 * k as f32;
                pen.line(&[(v, -12.5), (v, 12.5)], 1.2);
                pen.line(&[(-12.5, v), (12.5, v)], 1.2);
            }
        }
        Icon::PresetMenu => {
            for y in [-8.0, -0.25, 7.75] {
                pen.rect(-12.0, y - 1.25, 10.0, y + 1.25);
            }
            pen.poly(&[(19.5, 8.5), (19.5, 15.5), (12.0, 15.5)]);
        }
        Icon::PenPressure => {
            pen.arc(-3.5, 0.5, 13.0, (-0.23, 4.62), 1.8);
            pen.arc(-3.5, 0.5, 7.0, (0.35, 4.4), 1.8);
            pen.round_line((11.0, -16.0), (-2.0, -2.5), 6.5);
        }
        Icon::Refresh => {
            pen.arc(-1.0, 0.0, 10.5, (3.25, 5.45), 4.2);
            pen.poly(&[(13.0, -12.5), (13.0, -2.0), (2.5, -2.0)]);
            pen.arc(1.0, 0.0, 10.5, (0.1, 2.3), 4.2);
            pen.poly(&[(-14.0, 2.5), (-3.5, 2.5), (-14.0, 12.5)]);
        }
        Icon::ObjectFinder => {
            pen.line(
                &[
                    (-13.0, -12.5),
                    (13.0, -12.5),
                    (13.0, 12.5),
                    (-13.0, 12.5),
                    (-13.0, -12.5),
                ],
                2.0,
            );
            pen.rect(-10.0, -9.5, -2.5, -2.0);
            pen.rect(2.0, -5.5, 9.5, 2.0);
            pen.rect(-8.0, 2.5, -0.5, 10.0);
        }
        Icon::Feedback => {
            for k in 0..7 {
                pen.rect(-16.0 + 4.0 * k as f32, -15.5, -14.0 + 4.0 * k as f32, -13.5);
            }
            for k in 1..7 {
                pen.rect(-16.0, -15.5 + 4.0 * k as f32, -14.0, -13.5 + 4.0 * k as f32);
            }
            for k in 1..3 {
                pen.rect(8.0, -15.5 + 4.0 * k as f32, 10.0, -13.5 + 4.0 * k as f32);
            }
            pen.rect(-12.0, 8.5, -10.0, 10.5);
            pen.rect(-6.0, -3.5, 16.0, 12.0);
            pen.poly(&[(-2.0, 12.0), (4.0, 12.0), (-2.0, 17.5)]);
            let hole = Pen {
                color: background,
                ..pen
            };
            for x in [-2.5, 4.5, 11.5] {
                hole.dot(x, 4.5, 2.0);
            }
        }
        Icon::QuickNew | Icon::QuickAdd | Icon::QuickSubtract => {
            for k in 0..12 {
                let a = k as f32 * std::f32::consts::TAU / 12.0;
                let (x, y) = (-5.0 + 13.0 * a.cos(), 1.5 + 13.0 * a.sin());
                if !(0.0..=1.4).contains(&a) {
                    pen.rect(x - 1.0, y - 1.0, x + 1.0, y + 1.0);
                }
            }
            pen.round_line((14.5, -12.5), (5.5, -1.0), 5.0);
            pen.poly(&[
                (2.0, -0.5),
                (6.0, 0.5),
                (7.5, 4.0),
                (5.0, 8.5),
                (-1.0, 11.0),
                (-5.0, 11.5),
                (-3.0, 8.0),
                (-2.0, 3.0),
            ]);
            match icon {
                Icon::QuickAdd => {
                    pen.rect(5.0, -16.0, 7.0, -8.0);
                    pen.rect(2.0, -13.0, 10.0, -11.0);
                }
                Icon::QuickSubtract => pen.rect(-1.0, -14.5, 8.0, -12.5),
                _ => {}
            }
        }
        Icon::Angle => {
            pen.line(&[(-12.0, 7.0), (6.5, -9.5)], 2.6);
            pen.rect(-12.5, 6.5, 13.5, 8.5);
            pen.arc(-12.0, 7.5, 13.0, (-0.95, 0.0), 1.8);
        }
        Icon::BrushPanel | Icon::CloneSourcePanel => {
            pen.rect(-17.0, -16.5, -1.0, -12.0);
            pen.rect(-17.0, -12.0, 17.0, 16.0);
            let hole = Pen {
                color: background,
                ..pen
            };
            if icon == Icon::BrushPanel {
                hole.round_line((7.5, -8.0), (-1.0, 2.5), 3.5);
                hole.poly(&[
                    (-4.0, 3.0),
                    (-1.0, 5.0),
                    (-2.0, 9.0),
                    (-6.0, 11.5),
                    (-11.0, 12.5),
                    (-9.5, 8.0),
                    (-8.0, 4.5),
                ]);
            } else {
                hole.dot(-1.0, -5.0, 4.0);
                hole.rect(-2.5, -1.5, 0.5, 5.0);
                hole.rect(-9.0, 5.0, 7.5, 9.5);
                hole.rect(-7.0, 10.5, 5.5, 12.0);
            }
        }
        Icon::OpacityPressure => {
            pen.arc(-3.5, 0.5, 13.0, (-0.23, 4.62), 1.8);
            let faint = Pen {
                color: pen.color.gamma_multiply(0.45),
                ..pen
            };
            for k in 0..5 {
                let d = -12.0 + 6.0 * k as f32;
                faint.line(&[(-14.0 + d, 10.0), (-2.0 + d, -2.0)], 1.4);
            }
            pen.round_line((11.0, -16.0), (-2.0, -2.5), 6.5);
        }
        Icon::Airbrush => {
            let spray = Pen {
                color: pen.color.gamma_multiply(0.55),
                ..pen
            };
            spray.poly(&[
                (-6.0, -10.0),
                (-0.5, -11.0),
                (-5.0, -5.0),
                (-7.0, 2.0),
                (-5.0, 10.0),
                (-1.5, 13.0),
                (-9.0, 13.5),
                (-14.0, 7.0),
                (-15.0, -1.0),
                (-12.0, -7.0),
            ]);
            pen.round_line((12.5, -14.0), (1.0, -1.0), 7.0);
            pen.line(&[(-1.0, 1.0), (-10.0, 11.0)], 2.0);
            pen.line(&[(1.5, 4.0), (6.0, 9.0)], 2.0);
            pen.line(&[(-4.0, 13.0), (4.0, 7.5), (11.0, 14.5)], 2.0);
        }
        Icon::Symmetry => {
            pen.arc(-6.5, -6.0, 7.0, (1.2, 5.1), 2.0);
            pen.arc(6.5, -6.0, 7.0, (-1.95, 1.95), 2.0);
            pen.arc(-6.0, 9.5, 6.5, (1.4, 4.7), 2.0);
            pen.arc(6.0, 9.5, 6.5, (-1.55, 1.75), 2.0);
            for y in [-15.0, -11.0, -3.0, 5.0, 9.0, 13.0, 17.0] {
                pen.rect(-1.0, y, 1.0, y + 2.0);
            }
            pen.poly(&[(14.0, 17.0), (19.0, 17.0), (19.0, 12.0)]);
        }
        Icon::IgnoreAdjustments => {
            pen.ring(2.0, 3.0, 9.5, 1.6);
            pen.line(&[(-13.0, -11.5), (11.5, 13.0)], 1.6);
            let fill = Pen {
                color: pen.color.gamma_multiply(0.7),
                ..pen
            };
            fill.poly(&[(3.0, -6.0), (11.0, 2.0), (6.0, 9.0), (-5.0, -2.0)]);
        }
        Icon::SampleContinuous | Icon::SampleOnce | Icon::SampleSwatch => {
            // The eyedropper, top right
            pen.round_line((14.0, -12.5), (10.5, -9.0), 5.0);
            pen.poly(&[(4.5, -13.0), (12.5, -5.0), (9.5, -2.0), (1.5, -10.0)]);
            pen.line(&[(5.5, -6.0), (-6.0, 6.0)], 2.0);
            pen.line(&[(8.5, -3.0), (-3.0, 9.0)], 2.0);
            match icon {
                Icon::SampleContinuous => {
                    let fade = Pen {
                        color: pen.color.gamma_multiply(0.5),
                        ..pen
                    };
                    pen.rect(-15.0, 9.0, -7.0, 15.0);
                    fade.rect(-7.0, 9.0, 5.0, 15.0);
                }
                Icon::SampleOnce => {
                    pen.ring(-9.0, 9.0, 6.5, 1.8);
                    pen.rect(-10.0, 0.0, -8.0, 18.0);
                    pen.rect(-18.0, 8.0, 0.0, 10.0);
                }
                _ => {
                    pen.rect(-13.0, -2.0, 1.0, 12.0);
                    let hole = Pen {
                        color: background,
                        ..pen
                    };
                    hole.rect(-11.0, 0.0, -1.0, 10.0);
                    pen.rect(-11.0, 4.0, -5.0, 10.0);
                }
            }
        }
        Icon::MixerLoad | Icon::MixerClean => {
            pen.round_line((13.0, -14.0), (2.0, 0.0), 4.0);
            pen.poly(&[
                (-1.0, -1.0),
                (3.0, 2.0),
                (1.0, 7.0),
                (-5.0, 11.0),
                (-11.0, 12.5),
                (-8.0, 7.0),
                (-6.0, 2.0),
            ]);
            if icon == Icon::MixerLoad {
                pen.dot(-9.0, -6.0, 4.0);
                pen.poly(&[(-9.0, -15.0), (-5.2, -7.0), (-12.8, -7.0)]);
            } else {
                pen.line(&[(-14.0, -14.0), (14.0, 14.0)], 2.2);
            }
        }
        Icon::GradientLinear
        | Icon::GradientRadial
        | Icon::GradientAngle
        | Icon::GradientReflected
        | Icon::GradientDiamond => {
            // Gray from dark (0) to light (1), measured on Photoshop's
            let ramp = |x: f32, y: f32| -> f32 {
                let t = match icon {
                    Icon::GradientLinear => (x + 12.0) / 24.0,
                    Icon::GradientRadial => 1.0 - (x * x + y * y).sqrt() / 15.0,
                    Icon::GradientAngle => {
                        (y.atan2(x) + std::f32::consts::PI) / std::f32::consts::TAU
                    }
                    Icon::GradientReflected => 1.0 - y.abs() / 12.0,
                    _ => 1.0 - (x.abs() + y.abs()) / 20.0,
                };
                t.clamp(0.0, 1.0)
            };
            let mut mesh = egui::Mesh::default();
            let n = 12;
            for j in 0..=n {
                for i in 0..=n {
                    let (x, y) = (
                        -12.0 + 24.0 * i as f32 / n as f32,
                        -12.0 + 24.0 * j as f32 / n as f32,
                    );
                    let g = (40.0 + 196.0 * ramp(x, y)) as u8;
                    mesh.colored_vertex(pen.p(x, y), Color32::from_gray(g));
                }
            }
            for j in 0..n {
                for i in 0..n {
                    let k = (j * (n + 1) + i) as u32;
                    let w = n as u32 + 1;
                    mesh.add_triangle(k, k + 1, k + w + 1);
                    mesh.add_triangle(k, k + w + 1, k + w);
                }
            }
            pen.painter.add(Shape::mesh(mesh));
            pen.line(
                &[
                    (-13.0, -13.0),
                    (13.0, -13.0),
                    (13.0, 13.0),
                    (-13.0, 13.0),
                    (-13.0, -13.0),
                ],
                2.0,
            );
        }
        Icon::NotesPanel => {
            pen.rect(-14.0, -14.0, 14.0, 14.0);
            let hole = Pen {
                color: background,
                ..pen
            };
            hole.rect(-12.0, -6.0, 12.0, 12.0);
            for y in [-2.0, 3.0, 8.0] {
                pen.rect(-8.0, y, 8.0, y + 2.0);
            }
        }
        Icon::PathOperations => {
            pen.rect(-13.0, -13.0, 5.0, 5.0);
            pen.rect(-6.0, -6.0, 12.0, 12.0);
            let hole = Pen {
                color: background,
                ..pen
            };
            hole.rect(-5.0, -5.0, 4.0, 4.0);
            menu_mark(&pen);
        }
        Icon::PathCombine => {
            pen.rect(-9.0, -9.0, 9.0, 9.0);
            menu_mark(&pen);
        }
        Icon::PathAlignment => {
            pen.rect(-12.0, -15.0, -10.0, 14.0);
            pen.rect(-8.0, -9.0, 6.0, -2.0);
            pen.rect(-8.0, 3.0, 13.0, 10.0);
            menu_mark(&pen);
        }
        Icon::PathArrangement => {
            pen.line(
                &[
                    (5.0, -16.0),
                    (17.0, -10.0),
                    (5.0, -4.0),
                    (-7.0, -10.0),
                    (5.0, -16.0),
                ],
                2.0,
            );
            pen.poly(&[(5.0, -6.0), (17.0, 0.0), (5.0, 6.0), (-7.0, 0.0)]);
            pen.line(&[(-6.0, 4.0), (5.0, 10.0), (17.0, 4.0)], 2.0);
            pen.line(&[(-6.0, 9.0), (5.0, 15.0), (17.0, 9.0)], 2.0);
            pen.rect(-14.5, -5.0, -12.5, 7.0);
            pen.poly(&[(-13.5, -11.0), (-9.0, -4.0), (-18.0, -4.0)]);
            menu_mark(&pen);
        }
        Icon::GearMenu => {
            paint_scaled(
                pen.painter,
                pen.center,
                Icon::Gear,
                pen.color,
                background,
                pen.scale,
            );
            menu_mark(&pen);
        }
        Icon::TextOrientation => {
            let dim = Pen {
                color: pen.color.gamma_multiply(0.6),
                ..pen
            };
            dim.rect(-7.0, -14.0, 12.0, -11.0);
            dim.rect(1.0, -14.0, 4.0, 4.0);
            dim.rect(-3.0, 3.0, 8.0, 5.0);
            dim.rect(-14.0, -14.0, -12.0, 6.0);
            dim.poly(&[(-18.0, 4.0), (-8.0, 4.0), (-13.0, 10.0)]);
            dim.rect(-7.0, 11.0, 13.0, 13.0);
            dim.poly(&[(11.0, 7.0), (11.0, 17.0), (16.0, 12.0)]);
        }
        Icon::FontSize => {
            pen.rect(-8.0, -13.0, 16.0, -10.0);
            pen.rect(-8.0, -10.0, -5.0, -6.0);
            pen.rect(13.0, -10.0, 16.0, -6.0);
            pen.rect(2.0, -13.0, 6.0, 12.0);
            pen.rect(-3.0, 10.0, 11.0, 13.0);
            pen.rect(-16.0, 0.0, -4.0, 2.0);
            pen.rect(-11.0, 0.0, -9.0, 12.0);
            pen.rect(-14.0, 11.0, -6.0, 13.0);
        }
        Icon::TextLeft | Icon::TextCenter | Icon::TextRight => {
            for (k, y) in [-10.0, -5.0, 0.0, 5.0, 10.0].into_iter().enumerate() {
                let w = if k % 2 == 0 { 28.0 } else { 20.0 };
                let x0 = match icon {
                    Icon::TextLeft => -14.0,
                    Icon::TextCenter => -w / 2.0,
                    _ => 14.0 - w,
                };
                pen.rect(x0, y - 1.0, x0 + w, y + 1.0);
            }
        }
        Icon::WarpText => {
            pen.rect(-17.0, -14.0, 11.0, -10.0);
            pen.rect(-17.0, -10.0, -13.0, -7.0);
            pen.rect(7.0, -10.0, 11.0, -7.0);
            pen.rect(-5.0, -14.0, -1.0, 13.0);
            pen.rect(-9.0, 10.0, 3.0, 13.0);
            pen.poly(&[
                (13.0, -3.0),
                (17.0, 0.0),
                (15.0, 10.0),
                (10.0, 12.0),
                (8.0, 9.0),
            ]);
        }
        Icon::Text3d => {
            let dim = Pen {
                color: pen.color.gamma_multiply(0.55),
                ..pen
            };
            dim.line(&[(-1.0, -16.0), (-1.0, 5.0)], 2.0);
            dim.line(&[(-9.0, -10.0), (6.0, -16.0)], 2.0);
            dim.line(
                &[
                    (-17.0, 16.0),
                    (-11.0, 10.0),
                    (-4.0, 7.0),
                    (2.0, 7.0),
                    (9.0, 10.0),
                    (15.0, 16.0),
                ],
                2.0,
            );
        }
        Icon::CharacterPanels => {
            pen.rect(-16.0, -14.0, 16.0, 18.0);
            let hole = Pen {
                color: background,
                ..pen
            };
            for y in [0.0, 6.0, 12.0] {
                hole.rect(-11.0, y, -9.0, y + 2.0);
                hole.rect(-6.0, y, 11.0, y + 2.0);
            }
            hole.rect(0.0, -14.0, 16.0, -9.0);
            pen.rect(1.0, -14.0, 16.0, -11.0);
        }
        Icon::Link => {
            pen.line(
                &[
                    (-2.0, -4.0),
                    (-10.0, -4.0),
                    (-13.0, -1.0),
                    (-13.0, 1.0),
                    (-10.0, 4.0),
                    (-2.0, 4.0),
                ],
                2.0,
            );
            pen.line(
                &[
                    (2.0, -4.0),
                    (10.0, -4.0),
                    (13.0, -1.0),
                    (13.0, 1.0),
                    (10.0, 4.0),
                    (2.0, 4.0),
                ],
                2.0,
            );
            pen.rect(-6.0, -1.0, 6.0, 1.0);
        }
        Icon::CornerRadius => pen.arc(9.0, 12.0, 20.0, (3.15, 4.71), 2.4),
        Icon::PolygonSides => {
            pen.line(
                &[
                    (-8.0, -15.0),
                    (8.0, -15.0),
                    (16.0, 0.0),
                    (8.0, 15.0),
                    (-8.0, 15.0),
                    (-16.0, 0.0),
                    (-8.0, -15.0),
                ],
                2.0,
            );
            pen.rect(-5.0, -8.0, -3.0, 8.0);
            pen.rect(3.0, -8.0, 5.0, 8.0);
            pen.rect(-8.0, -4.0, 8.0, -2.0);
            pen.rect(-8.0, 2.0, 8.0, 4.0);
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            pen.ring(-3.0, -3.0, 10.0, 2.2);
            pen.round_line((4.5, 4.5), (13.0, 13.0), 4.0);
            pen.rect(-8.0, -4.0, 2.0, -2.0);
            if icon == Icon::ZoomIn {
                pen.rect(-4.0, -8.0, -2.0, 2.0);
            }
        }
        Icon::TextTop | Icon::TextMiddle | Icon::TextBottom => {
            for (k, x) in [-12.0, -8.0, -4.0, 0.0, 4.0, 8.0, 12.0]
                .into_iter()
                .enumerate()
            {
                let h = [22.0, 28.0, 26.0, 28.0, 20.0, 26.0, 24.0][k];
                let y0 = match icon {
                    Icon::TextTop => -14.0,
                    Icon::TextMiddle => -h / 2.0,
                    _ => 14.0 - h,
                };
                pen.rect(x - 1.0, y0, x + 1.0, y0 + h);
            }
        }
        Icon::ArrangeFront | Icon::ArrangeForward | Icon::ArrangeBackward | Icon::ArrangeBack => {
            let filled = match icon {
                Icon::ArrangeFront => 0,
                Icon::ArrangeForward => 1,
                Icon::ArrangeBackward => 2,
                _ => 3,
            };
            for k in (0..4).rev() {
                let y = -11.0 + 6.0 * k as f32;
                let d = [(4.0, y - 5.0), (15.0, y), (4.0, y + 5.0), (-7.0, y)];
                if k == filled {
                    pen.poly(&d);
                } else {
                    let hole = Pen {
                        color: background,
                        ..pen
                    };
                    hole.poly(&d);
                    pen.line(&[d[0], d[1], d[2], d[3], d[0]], 1.5);
                }
            }
            let up = filled < 2;
            // The arrow: a shaft and a head pointing up or down
            let (tip, dir, y0, y1): (f32, f32, f32, f32) = if up {
                (-3.0, 1.0, 0.0, 7.0)
            } else {
                (7.0, -1.0, -3.0, 4.0)
            };
            pen.rect(-13.75, y0, -12.25, y1);
            pen.poly(&[
                (-13.0, tip),
                (-9.0, tip + 4.0 * dir),
                (-17.0, tip + 4.0 * dir),
            ]);
        }
        Icon::ArtboardPortrait | Icon::ArtboardLandscape | Icon::AddArtboard => {
            let (w, h) = if icon == Icon::ArtboardLandscape {
                (12.0, 9.0)
            } else {
                (9.0, 12.0)
            };
            let (cx, cy) = if icon == Icon::AddArtboard {
                (2.0, 2.0)
            } else {
                (0.0, 0.0)
            };
            let fold = 4.0;
            pen.line(
                &[
                    (cx + w - fold, cy - h),
                    (cx - w, cy - h),
                    (cx - w, cy + h),
                    (cx + w, cy + h),
                    (cx + w, cy - h + fold),
                    (cx + w - fold, cy - h),
                ],
                1.5,
            );
            pen.poly(&[
                (cx + w - fold, cy - h),
                (cx + w, cy - h + fold),
                (cx + w - fold, cy - h + fold),
            ]);
            if icon == Icon::AddArtboard {
                pen.rect(cx - 0.75, cy - 4.0, cx + 0.75, cy + 4.0);
                pen.rect(cx - 4.0, cy - 0.75, cx + 4.0, cy + 0.75);
                pen.rect(-12.75, -16.0, -11.25, -12.0);
                pen.rect(-16.0, -12.75, -12.0, -11.25);
            } else {
                // Tick marks off each corner
                for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let (x, y) = (w * sx, h * sy);
                    pen.rect(x - 0.75, y + 3.0 * sy, x + 0.75, y + 5.0 * sy);
                    pen.rect(x + 3.0 * sx, y - 0.75, x + 5.0 * sx, y + 0.75);
                }
            }
        }
        Icon::FrameRect
        | Icon::FrameEllipse
        | Icon::FrameTriangle
        | Icon::FrameHexagon
        | Icon::FrameCustom => {
            let shade = Pen {
                color: Color32::from_gray(0x6f),
                ..pen
            };
            let outline: Vec<(f32, f32)> = match icon {
                Icon::FrameRect => vec![(-13.0, -13.0), (13.0, -13.0), (13.0, 13.0), (-13.0, 13.0)],
                Icon::FrameEllipse => (0..32)
                    .map(|i| {
                        let a = i as f32 / 32.0 * std::f32::consts::TAU;
                        (14.0 * a.cos(), 14.0 * a.sin())
                    })
                    .collect(),
                Icon::FrameTriangle => vec![(0.0, -14.0), (16.0, 14.0), (-16.0, 14.0)],
                Icon::FrameHexagon => vec![
                    (-7.0, -13.0),
                    (7.0, -13.0),
                    (15.0, 0.0),
                    (7.0, 13.0),
                    (-7.0, 13.0),
                    (-15.0, 0.0),
                ],
                _ => (0..40)
                    .map(|i| {
                        let a = i as f32 / 40.0 * std::f32::consts::TAU;
                        let r = 10.0 + 4.0 * (4.0 * a + 0.4).sin();
                        (r * a.cos(), r * a.sin())
                    })
                    .collect(),
            };
            if icon == Icon::FrameCustom {
                for w in outline.windows(2) {
                    shade.poly(&[(0.0, 0.0), w[0], w[1]]);
                }
                shade.poly(&[(0.0, 0.0), outline[outline.len() - 1], outline[0]]);
            } else {
                shade.poly(&outline);
            }
            let mut ring = outline.clone();
            ring.push(outline[0]);
            pen.line(&ring, 1.5);
            let (a, b) = match icon {
                Icon::FrameTriangle => (8.0, 6.0),
                Icon::FrameRect => (13.0, 13.0),
                _ => (9.5, 9.5),
            };
            let oy = if icon == Icon::FrameTriangle {
                6.0
            } else {
                0.0
            };
            pen.line(&[(-a, oy - b), (a, oy + b)], 1.5);
            pen.line(&[(a, oy - b), (-a, oy + b)], 1.5);
        }
        Icon::StrokeCenter => {
            pen.line(
                &[
                    (-8.0, -8.0),
                    (8.0, -8.0),
                    (8.0, 8.0),
                    (-8.0, 8.0),
                    (-8.0, -8.0),
                ],
                3.0,
            );
            for (x, y) in [(-10.0, -10.0), (10.0, -10.0), (-10.0, 10.0), (10.0, 10.0)] {
                pen.rect(x - 2.0, y - 2.0, x + 2.0, y + 2.0);
            }
        }
        Icon::BrushAdd | Icon::BrushSubtract => {
            // Traced at 2x: the handle, its ferrule and the bristles
            pen.poly(&[(11.0, -16.0), (15.0, -13.0), (1.0, 3.0), (-3.0, -1.0)]);
            pen.poly(&[(-4.0, 0.0), (0.0, 4.0), (-2.0, 6.0), (-6.0, 2.0)]);
            pen.poly(&[
                (-6.0, 3.0),
                (-2.0, 7.0),
                (-4.0, 12.0),
                (-9.0, 15.0),
                (-15.0, 15.0),
                (-12.0, 10.0),
                (-11.0, 5.0),
            ]);
            pen.rect(8.0, 7.0, 21.0, 9.5);
            if icon == Icon::BrushAdd {
                pen.rect(13.25, 1.0, 15.75, 14.0);
            }
        }
        Icon::Swap => {
            pen.rect(-7.5, -7.0, 4.0, -5.0);
            pen.poly(&[(3.5, -12.5), (14.5, -6.0), (3.5, 0.5)]);
            pen.rect(-3.0, 5.0, 7.5, 7.0);
            pen.poly(&[(-2.5, -0.5), (-2.5, 12.5), (-13.5, 6.0)]);
        }
        Icon::TransformPreset => {
            for (x, y) in [
                (-6.5, -10.0),
                (3.5, -10.0),
                (13.5, -10.0),
                (13.5, 0.0),
                (-6.5, 10.0),
                (3.5, 10.0),
                (13.5, 10.0),
            ] {
                pen.rect(x - 2.0, y - 2.0, x + 2.0, y + 2.0);
            }
            pen.rect(-4.5, -10.5, 11.5, -9.5);
            pen.rect(-4.5, 9.5, 11.5, 10.5);
            pen.rect(13.0, -8.0, 14.0, 8.0);
            pen.rect(-7.0, -8.0, -6.0, -6.0);
            pen.rect(-7.0, 6.0, -6.0, 8.0);
            pen.poly(&[(-16.0, -10.0), (-3.5, 1.5), (-11.0, 2.0)]);
            pen.poly(&[(-16.0, -10.0), (-11.0, 2.0), (-16.0, 6.5)]);
        }
        Icon::WarpToggle => {
            pen.ring(0.0, -6.0, 11.5, 2.5);
            pen.rect(-1.0, -17.0, 1.0, 3.0);
            let inner: Vec<(f32, f32)> = (0..=12)
                .map(|k| {
                    let a = (20.0 + 140.0 * k as f32 / 12.0f32).to_radians();
                    (8.5 * a.cos(), -7.0 + 8.0 * a.sin())
                })
                .collect();
            pen.line(&inner, 2.5);
            let bottom: Vec<(f32, f32)> = (0..=16)
                .map(|k| {
                    let a = (200.0 + 140.0 * k as f32 / 16.0f32).to_radians();
                    (16.0 * a.cos(), 22.0 + 15.0 * a.sin())
                })
                .collect();
            pen.line(&bottom, 2.5);
            pen.rect(-18.0, 12.0, -14.0, 16.0);
            pen.rect(14.0, 12.0, 18.0, 16.0);
        }
        Icon::CropCommit => {
            pen.line(&[(-14.0, -0.5), (-5.5, 9.0), (13.0, -12.5)], 4.5);
        }
        Icon::Gear => {
            pen.dot(0.0, 0.0, 10.0);
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                let (s, c) = a.sin_cos();
                let at = |r: f32, t: f32| (c * r - s * t, s * r + c * t);
                pen.poly(&[at(8.0, -2.5), at(13.0, -2.0), at(13.0, 2.0), at(8.0, 2.5)]);
            }
            Pen {
                color: background,
                ..pen
            }
            .dot(0.0, 0.0, 4.0);
            Pen {
                color: Color32::WHITE,
                ..pen
            }
            .poly(&[(10.0, 15.5), (18.0, 15.5), (14.0, 19.5)]);
        }
    }
}
