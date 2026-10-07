//! Tool definitions: every Photoshop 2026 tool, how they are grouped in the
//! toolbar, their names and single-key shortcuts.

/// Every tool in Photoshop's toolbar, including the ones hidden in groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tool {
    Move,
    Artboard,
    RectangularMarquee,
    EllipticalMarquee,
    SingleRowMarquee,
    SingleColumnMarquee,
    Lasso,
    PolygonalLasso,
    MagneticLasso,
    /// Photoshop 2026's Selection Brush (paints a selection).
    SelectionBrush,
    ObjectSelection,
    QuickSelection,
    MagicWand,
    Crop,
    PerspectiveCrop,
    Slice,
    SliceSelect,
    Frame,
    Eyedropper,
    ColorSampler,
    Ruler,
    Note,
    Count,
    SpotHealingBrush,
    Remove,
    HealingBrush,
    Patch,
    ContentAwareMove,
    RedEye,
    Brush,
    Pencil,
    ColorReplacement,
    MixerBrush,
    CloneStamp,
    PatternStamp,
    HistoryBrush,
    ArtHistoryBrush,
    Eraser,
    BackgroundEraser,
    MagicEraser,
    Gradient,
    PaintBucket,
    Blur,
    Sharpen,
    Smudge,
    /// Photoshop 2025+'s Adjustment Brush (paints an adjustment).
    AdjustmentBrush,
    Dodge,
    Burn,
    Sponge,
    Pen,
    FreeformPen,
    CurvaturePen,
    AddAnchorPoint,
    DeleteAnchorPoint,
    ConvertPoint,
    HorizontalType,
    VerticalType,
    VerticalTypeMask,
    HorizontalTypeMask,
    PathSelection,
    DirectSelection,
    Rectangle,
    Ellipse,
    Triangle,
    Polygon,
    Line,
    CustomShape,
    Hand,
    RotateView,
    Zoom,
}

use Tool::*;

/// The toolbar's slots from top to bottom; each slot is a group of tools that
/// share a button, in the order of Photoshop's flyout menu.
pub const TOOLBAR: &[&[Tool]] = &[
    &[Move, Artboard],
    &[
        RectangularMarquee,
        EllipticalMarquee,
        SingleRowMarquee,
        SingleColumnMarquee,
    ],
    // Photoshop 2026 lists the Selection Brush first, though the Lasso is
    // the one the slot shows at first
    &[SelectionBrush, Lasso, PolygonalLasso, MagneticLasso],
    &[ObjectSelection, QuickSelection, MagicWand],
    &[Crop, PerspectiveCrop, Slice, SliceSelect],
    &[Frame],
    &[Eyedropper, ColorSampler, Ruler, Note, Count],
    &[
        SpotHealingBrush,
        Remove,
        HealingBrush,
        Patch,
        ContentAwareMove,
        RedEye,
    ],
    &[Brush, Pencil, ColorReplacement, MixerBrush],
    &[CloneStamp, PatternStamp],
    &[HistoryBrush, ArtHistoryBrush],
    &[Eraser, BackgroundEraser, MagicEraser],
    &[Gradient, PaintBucket],
    &[Blur, Sharpen, Smudge],
    &[AdjustmentBrush],
    &[Dodge, Burn, Sponge],
    &[
        Pen,
        FreeformPen,
        CurvaturePen,
        AddAnchorPoint,
        DeleteAnchorPoint,
        ConvertPoint,
    ],
    &[
        HorizontalType,
        VerticalType,
        VerticalTypeMask,
        HorizontalTypeMask,
    ],
    &[PathSelection, DirectSelection],
    &[Rectangle, Ellipse, Triangle, Polygon, Line, CustomShape],
    &[Hand, RotateView],
    &[Zoom],
];

/// The tool a toolbar slot shows before another is picked: the group's
/// first, except the Lasso in its group (as in Photoshop 2026).
pub fn default_in_group(group: &[Tool]) -> Tool {
    if group.contains(&Lasso) {
        Lasso
    } else {
        group[0]
    }
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Move => "Move Tool",
            Artboard => "Artboard Tool",
            RectangularMarquee => "Rectangular Marquee Tool",
            EllipticalMarquee => "Elliptical Marquee Tool",
            SingleRowMarquee => "Single Row Marquee Tool",
            SingleColumnMarquee => "Single Column Marquee Tool",
            Lasso => "Lasso Tool",
            PolygonalLasso => "Polygonal Lasso Tool",
            MagneticLasso => "Magnetic Lasso Tool",
            SelectionBrush => "Selection Brush Tool",
            ObjectSelection => "Object Selection Tool",
            QuickSelection => "Quick Selection Tool",
            MagicWand => "Magic Wand Tool",
            Crop => "Crop Tool",
            PerspectiveCrop => "Perspective Crop Tool",
            Slice => "Slice Tool",
            SliceSelect => "Slice Select Tool",
            Frame => "Frame Tool",
            Eyedropper => "Eyedropper Tool",
            ColorSampler => "Color Sampler Tool",
            Ruler => "Ruler Tool",
            Note => "Note Tool",
            Count => "Count Tool",
            SpotHealingBrush => "Spot Healing Brush Tool",
            Remove => "Remove Tool",
            HealingBrush => "Healing Brush Tool",
            Patch => "Patch Tool",
            ContentAwareMove => "Content-Aware Move Tool",
            RedEye => "Red Eye Tool",
            Brush => "Brush Tool",
            Pencil => "Pencil Tool",
            ColorReplacement => "Color Replacement Tool",
            MixerBrush => "Mixer Brush Tool",
            CloneStamp => "Clone Stamp Tool",
            PatternStamp => "Pattern Stamp Tool",
            HistoryBrush => "History Brush Tool",
            ArtHistoryBrush => "Art History Brush Tool",
            Eraser => "Eraser Tool",
            BackgroundEraser => "Background Eraser Tool",
            MagicEraser => "Magic Eraser Tool",
            Gradient => "Gradient Tool",
            PaintBucket => "Paint Bucket Tool",
            Blur => "Blur Tool",
            Sharpen => "Sharpen Tool",
            Smudge => "Smudge Tool",
            AdjustmentBrush => "Adjustment Brush Tool",
            Dodge => "Dodge Tool",
            Burn => "Burn Tool",
            Sponge => "Sponge Tool",
            Pen => "Pen Tool",
            FreeformPen => "Freeform Pen Tool",
            CurvaturePen => "Curvature Pen Tool",
            AddAnchorPoint => "Add Anchor Point Tool",
            DeleteAnchorPoint => "Delete Anchor Point Tool",
            ConvertPoint => "Convert Point Tool",
            HorizontalType => "Horizontal Type Tool",
            VerticalType => "Vertical Type Tool",
            VerticalTypeMask => "Vertical Type Mask Tool",
            HorizontalTypeMask => "Horizontal Type Mask Tool",
            PathSelection => "Path Selection Tool",
            DirectSelection => "Direct Selection Tool",
            Rectangle => "Rectangle Tool",
            Ellipse => "Ellipse Tool",
            Triangle => "Triangle Tool",
            Polygon => "Polygon Tool",
            Line => "Line Tool",
            CustomShape => "Custom Shape Tool",
            Hand => "Hand Tool",
            RotateView => "Rotate View Tool",
            Zoom => "Zoom Tool",
        }
    }

    /// Single-key shortcut, as in Photoshop. Tools in a group share the
    /// group's key (Shift+key cycles through them); a few have none.
    pub fn shortcut(self) -> Option<char> {
        Some(match self {
            Move | Artboard => 'V',
            RectangularMarquee | EllipticalMarquee => 'M',
            SelectionBrush | Lasso | PolygonalLasso | MagneticLasso => 'L',
            ObjectSelection | QuickSelection | MagicWand => 'W',
            Crop | PerspectiveCrop | Slice | SliceSelect => 'C',
            Frame => 'K',
            Eyedropper | ColorSampler | Ruler | Note | Count => 'I',
            SpotHealingBrush | Remove | HealingBrush | Patch | ContentAwareMove | RedEye => 'J',
            Brush | Pencil | ColorReplacement | MixerBrush => 'B',
            CloneStamp | PatternStamp => 'S',
            HistoryBrush | ArtHistoryBrush => 'Y',
            Eraser | BackgroundEraser | MagicEraser => 'E',
            Gradient | PaintBucket => 'G',
            Dodge | Burn | Sponge => 'O',
            Pen | FreeformPen | CurvaturePen => 'P',
            HorizontalType | VerticalType | VerticalTypeMask | HorizontalTypeMask => 'T',
            PathSelection | DirectSelection => 'A',
            Rectangle | Ellipse | Triangle | Polygon | Line | CustomShape => 'U',
            Hand => 'H',
            RotateView => 'R',
            Zoom => 'Z',
            SingleRowMarquee | SingleColumnMarquee | Blur | Sharpen | Smudge | AddAnchorPoint
            | DeleteAnchorPoint | ConvertPoint | AdjustmentBrush => return None,
        })
    }

    /// Index of the toolbar slot this tool lives in.
    pub fn slot(self) -> usize {
        TOOLBAR
            .iter()
            .position(|group| group.contains(&self))
            .expect("every tool is in the toolbar")
    }
}

/// The tool a key press selects. `current` is the tool shown in each slot
/// (Photoshop remembers the last tool used per group); with `shift`, the key
/// cycles to the next tool in the group that has the same shortcut.
pub fn tool_for_key(
    key: char,
    shift: bool,
    active: Tool,
    current: impl Fn(usize) -> Tool,
) -> Option<Tool> {
    let key = key.to_ascii_uppercase();
    // Rotate View shares the Hand slot but has its own key
    let slot = TOOLBAR
        .iter()
        .position(|group| group.iter().any(|t| t.shortcut() == Some(key)))?;
    let group = TOOLBAR[slot];
    let shown = current(slot);
    if group.iter().filter(|t| t.shortcut() == Some(key)).count() == 1 {
        return group.iter().copied().find(|t| t.shortcut() == Some(key));
    }
    if !shift || active.slot() != slot || shown.shortcut() != Some(key) {
        return Some(if shown.shortcut() == Some(key) {
            shown
        } else {
            group.iter().copied().find(|t| t.shortcut() == Some(key))?
        });
    }
    let i = group.iter().position(|t| *t == shown)?;
    (1..=group.len())
        .map(|k| group[(i + k) % group.len()])
        .find(|t| t.shortcut() == Some(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first(slot: usize) -> Tool {
        TOOLBAR[slot][0]
    }

    #[test]
    fn every_tool_appears_once() {
        let all: Vec<Tool> = TOOLBAR.iter().flat_map(|g| g.iter().copied()).collect();
        let mut dedup = all.clone();
        dedup.sort_by_key(|t| *t as usize);
        dedup.dedup();
        assert_eq!(all.len(), dedup.len());
        assert_eq!(all.len(), 70);
        assert_eq!(TOOLBAR.len(), 22);
    }

    #[test]
    fn keys_select_the_group_and_shift_cycles() {
        assert_eq!(
            tool_for_key('m', false, Move, first),
            Some(RectangularMarquee)
        );
        // Shift+M from the rectangular marquee goes to the elliptical one
        assert_eq!(
            tool_for_key('M', true, RectangularMarquee, first),
            Some(EllipticalMarquee)
        );
        // ...and back, skipping the single row/column tools (no shortcut)
        let shown = |slot: usize| {
            if slot == 1 {
                EllipticalMarquee
            } else {
                first(slot)
            }
        };
        assert_eq!(
            tool_for_key('M', true, EllipticalMarquee, shown),
            Some(RectangularMarquee)
        );
        // Without Shift the group's current tool is chosen
        assert_eq!(
            tool_for_key('m', false, Move, shown),
            Some(EllipticalMarquee)
        );
        // Shift+key from another group just selects the group
        assert_eq!(
            tool_for_key('M', true, Brush, first),
            Some(RectangularMarquee)
        );
        // Keys unique within their slot
        assert_eq!(tool_for_key('r', false, Move, first), Some(RotateView));
        assert_eq!(tool_for_key('h', true, RotateView, first), Some(Hand));
        assert_eq!(tool_for_key('q', false, Move, first), None);
    }
}
