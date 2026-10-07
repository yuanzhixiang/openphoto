use crate::tile::TiledImage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(pub u64);

/// All Photoshop layer blend modes, ordered and grouped as in the blend mode menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    /// Groups only: the group's layers blend straight into what's below,
    /// as if they weren't grouped.
    PassThrough,
    Dissolve,

    Darken,
    Multiply,
    ColorBurn,
    LinearBurn,
    DarkerColor,

    Lighten,
    Screen,
    ColorDodge,
    LinearDodge,
    LighterColor,

    Overlay,
    SoftLight,
    HardLight,
    VividLight,
    LinearLight,
    PinLight,
    HardMix,

    Difference,
    Exclusion,
    Subtract,
    Divide,

    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl BlendMode {
    /// Menu groups; a separator is drawn between groups.
    pub const GROUPS: &[&[Self]] = &[
        &[Self::Normal, Self::Dissolve],
        &[
            Self::Darken,
            Self::Multiply,
            Self::ColorBurn,
            Self::LinearBurn,
            Self::DarkerColor,
        ],
        &[
            Self::Lighten,
            Self::Screen,
            Self::ColorDodge,
            Self::LinearDodge,
            Self::LighterColor,
        ],
        &[
            Self::Overlay,
            Self::SoftLight,
            Self::HardLight,
            Self::VividLight,
            Self::LinearLight,
            Self::PinLight,
            Self::HardMix,
        ],
        &[
            Self::Difference,
            Self::Exclusion,
            Self::Subtract,
            Self::Divide,
        ],
        &[Self::Hue, Self::Saturation, Self::Color, Self::Luminosity],
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::PassThrough => "Pass Through",
            Self::Dissolve => "Dissolve",
            Self::Darken => "Darken",
            Self::Multiply => "Multiply",
            Self::ColorBurn => "Color Burn",
            Self::LinearBurn => "Linear Burn",
            Self::DarkerColor => "Darker Color",
            Self::Lighten => "Lighten",
            Self::Screen => "Screen",
            Self::ColorDodge => "Color Dodge",
            Self::LinearDodge => "Linear Dodge (Add)",
            Self::LighterColor => "Lighter Color",
            Self::Overlay => "Overlay",
            Self::SoftLight => "Soft Light",
            Self::HardLight => "Hard Light",
            Self::VividLight => "Vivid Light",
            Self::LinearLight => "Linear Light",
            Self::PinLight => "Pin Light",
            Self::HardMix => "Hard Mix",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Subtract => "Subtract",
            Self::Divide => "Divide",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
        }
    }
}

#[derive(Clone)]
pub enum LayerKind {
    Raster(TiledImage),
    /// A layer group (folder). Its layers follow it directly below in
    /// `Document::layers`, each with `parent` set to the group's id.
    Group {
        /// Collapsed in the Layers panel (its layers not listed).
        collapsed: bool,
    },
}

/// A layer mask: white shows the layer, black hides it, grays partly.
/// Stored as an opaque gray image (the value is in every color channel), so
/// a plain white or black mask shares one tile across the whole canvas.
#[derive(Clone)]
pub struct LayerMask {
    pub image: TiledImage,
    /// Layer > Layer Mask > Disable: a disabled mask is kept but ignored.
    pub enabled: bool,
}

impl LayerMask {
    /// A mask of one value: 255 reveals everything, 0 hides everything.
    pub fn filled(width: u32, height: u32, value: u8) -> Self {
        Self {
            image: TiledImage::filled(width, height, [value, value, value, 255]),
            enabled: true,
        }
    }

    /// A mask from per-pixel values (a selection's mask), row by row.
    pub fn from_values(width: u32, height: u32, values: impl Fn(u32, u32) -> u8) -> Self {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let v = values(x, y);
                pixels.extend_from_slice(&[v, v, v, 255]);
            }
        }
        Self {
            image: TiledImage::from_rgba8(width, height, &pixels),
            enabled: true,
        }
    }

    /// The mask's value at a pixel (0 outside the image).
    pub fn value(&self, x: u32, y: u32) -> u8 {
        self.image.pixel(x, y)[0]
    }
}

/// A layer's color label (Photoshop's "Color" in the New Layer dialog and
/// the Layers panel's context menu), shown behind the eye in the panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayerColor {
    #[default]
    None,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
    Gray,
}

impl LayerColor {
    pub const ALL: [Self; 8] = [
        Self::None,
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Violet,
        Self::Gray,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Red => "Red",
            Self::Orange => "Orange",
            Self::Yellow => "Yellow",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Violet => "Violet",
            Self::Gray => "Gray",
        }
    }

    /// The index Photoshop stores in a PSD's `lclr` block.
    pub fn psd_index(self) -> u16 {
        Self::ALL.iter().position(|&c| c == self).unwrap_or(0) as u16
    }

    pub fn from_psd_index(i: u16) -> Self {
        Self::ALL.get(i as usize).copied().unwrap_or_default()
    }
}

/// The color Photoshop fills a new layer with for "Fill with
/// \<mode\>-neutral color": the color that leaves the image unchanged in
/// that blend mode. `None` for modes without one (the option is greyed).
pub fn neutral_color(mode: BlendMode) -> Option<[u8; 3]> {
    use BlendMode::*;
    match mode {
        Overlay | SoftLight | HardLight | VividLight | LinearLight | PinLight => Some([128; 3]),
        Multiply | ColorBurn | LinearBurn | Darken | Divide => Some([255; 3]),
        Screen | ColorDodge | LinearDodge | Lighten | Difference | Exclusion | Subtract => {
            Some([0; 3])
        }
        _ => None,
    }
}

#[derive(Clone)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    /// Layer opacity, applied to the whole layer including layer styles.
    pub opacity: f32,
    /// Fill opacity, applied to the pixels only (not to layer styles).
    pub fill: f32,
    pub blend_mode: BlendMode,
    /// The "Background" layer: locked, opaque and always at the bottom.
    pub is_background: bool,
    pub lock_transparency: bool,
    pub lock_pixels: bool,
    pub lock_position: bool,
    /// Prevent auto-nesting into and out of artboards and frames.
    pub lock_nesting: bool,
    /// Lock all: a flag of its own, so turning it off brings back the
    /// individual locks underneath (as in Photoshop).
    pub lock_all: bool,
    pub kind: LayerKind,
    pub mask: Option<LayerMask>,
    pub color: LayerColor,
    /// The group the layer is in, if any.
    pub parent: Option<LayerId>,
    /// Layer > Link Layers: layers with the same number are linked (a
    /// number no other layer has links nothing; see `link`).
    pub link: Option<u32>,
    /// Shift-clicking the link icon: the link set aside for now (a red ×
    /// on the icon); the layer moves on its own until enabled again.
    pub link_disabled: bool,
}

impl Layer {
    pub fn raster(id: LayerId, name: impl Into<String>, image: TiledImage) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            opacity: 1.0,
            fill: 1.0,
            blend_mode: BlendMode::Normal,
            is_background: false,
            lock_transparency: false,
            lock_pixels: false,
            lock_position: false,
            lock_nesting: false,
            lock_all: false,
            kind: LayerKind::Raster(image),
            mask: None,
            color: LayerColor::None,
            parent: None,
            link: None,
            link_disabled: false,
        }
    }

    /// A layer group (folder) with the given name.
    pub fn group(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            kind: LayerKind::Group { collapsed: false },
            blend_mode: BlendMode::PassThrough,
            ..Self::raster(id, name, TiledImage::new(0, 0))
        }
    }

    pub fn is_group(&self) -> bool {
        matches!(self.kind, LayerKind::Group { .. })
    }

    /// The layer's pixels; `None` for a group.
    pub fn image(&self) -> Option<&TiledImage> {
        match &self.kind {
            LayerKind::Raster(image) => Some(image),
            LayerKind::Group { .. } => None,
        }
    }

    pub fn image_mut(&mut self) -> Option<&mut TiledImage> {
        match &mut self.kind {
            LayerKind::Raster(image) => Some(image),
            LayerKind::Group { .. } => None,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_background || self.pixels_locked() || self.position_locked()
    }

    /// Whether transparent pixels are protected (by its lock or Lock all).
    pub fn transparency_locked(&self) -> bool {
        self.lock_transparency || self.lock_all
    }

    /// Whether the pixels can't be edited (by their lock or Lock all).
    pub fn pixels_locked(&self) -> bool {
        self.lock_pixels || self.lock_all
    }

    /// Whether the layer can't move (by its lock or Lock all).
    pub fn position_locked(&self) -> bool {
        self.lock_position || self.lock_all
    }

    pub fn locks(&self) -> Locks {
        Locks {
            transparency: self.lock_transparency,
            pixels: self.lock_pixels,
            position: self.lock_position,
            nesting: self.lock_nesting,
            all: self.lock_all,
        }
    }

    pub fn set_locks(&mut self, locks: Locks) {
        self.lock_transparency = locks.transparency;
        self.lock_pixels = locks.pixels;
        self.lock_position = locks.position;
        self.lock_nesting = locks.nesting;
        self.lock_all = locks.all;
    }
}

/// A layer's lock flags, as the Lock Layers dialog and the Layers panel's
/// lock buttons set them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Locks {
    pub transparency: bool,
    pub pixels: bool,
    pub position: bool,
    pub nesting: bool,
    pub all: bool,
}
