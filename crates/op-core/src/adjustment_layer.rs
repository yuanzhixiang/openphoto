//! Non-destructive, masked point adjustments over the composite below a layer.
use crate::{
    Document, Layer, LayerMask, TiledImage,
    adjust::{self, Adjustment},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Vibrance,
    BrightnessContrast,
    Exposure,
    HueSaturation,
    ColorBalance,
    BlackWhite,
}
impl Kind {
    pub const ALL: [Self; 6] = [
        Self::Vibrance,
        Self::BrightnessContrast,
        Self::Exposure,
        Self::HueSaturation,
        Self::ColorBalance,
        Self::BlackWhite,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Vibrance => "Vibrance",
            Self::BrightnessContrast => "Brightness/Contrast",
            Self::Exposure => "Exposure",
            Self::HueSaturation => "Hue/Saturation",
            Self::ColorBalance => "Color Balance",
            Self::BlackWhite => "Black & White",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub kind: Kind,
    pub values: [f32; 9],
}
impl Settings {
    pub fn new(kind: Kind) -> Self {
        let mut values = [0.; 9];
        match kind {
            Kind::Vibrance => values[0] = 20.,
            Kind::BrightnessContrast => values[0] = 20.,
            Kind::Exposure => {
                values[0] = 1.;
                values[2] = 1.;
            }
            Kind::HueSaturation => values[1] = 20.,
            Kind::ColorBalance => values[3] = 20.,
            Kind::BlackWhite => values[..6].copy_from_slice(&[40., 60., 40., 60., 20., 80.]),
        }
        Self { kind, values }
    }
    pub fn effect(self) -> Adjustment {
        let v = self.values;
        let int = |i: usize, lo: i32, hi: i32| {
            if v[i].is_finite() {
                (v[i].round() as i32).clamp(lo, hi)
            } else {
                0
            }
        };
        match self.kind {
            Kind::Vibrance => Adjustment::Vibrance {
                vibrance: int(0, -100, 100),
                saturation: int(1, -100, 100),
            },
            Kind::BrightnessContrast => Adjustment::BrightnessContrast {
                brightness: int(0, -150, 150),
                contrast: int(1, -50, 100),
                legacy: false,
            },
            Kind::Exposure => Adjustment::Exposure {
                exposure: v[0].clamp(-20., 20.),
                offset: v[1].clamp(-0.5, 0.5),
                gamma: v[2].clamp(0.01, 9.99),
            },
            Kind::HueSaturation => Adjustment::HueSaturation(adjust::HueSaturation::master(
                int(0, -180, 180),
                int(1, -100, 100),
                int(2, -100, 100),
            )),
            Kind::ColorBalance => Adjustment::ColorBalance {
                shadows: [int(0, -100, 100), int(1, -100, 100), int(2, -100, 100)],
                midtones: [int(3, -100, 100), int(4, -100, 100), int(5, -100, 100)],
                highlights: [int(6, -100, 100), int(7, -100, 100), int(8, -100, 100)],
                preserve_luminosity: true,
            },
            Kind::BlackWhite => Adjustment::BlackWhite {
                weights: std::array::from_fn(|i| int(i, -200, 300)),
                tint: None,
            },
        }
    }
    pub fn valid(self) -> bool {
        self.values.iter().all(|v| v.is_finite())
    }
}
pub fn create(doc: &mut Document, settings: Settings) -> crate::LayerId {
    let id = doc.new_layer_id();
    let mut layer = Layer::raster(
        id,
        settings.kind.name(),
        TiledImage::new(doc.width, doc.height),
    );
    layer.adjustment = Some(settings);
    layer.mask = Some(LayerMask::filled(doc.width, doc.height, 0));
    doc.insert_above_active(layer);
    doc.mask_target = true;
    id
}
pub fn map(effect: Adjustment, tables: &Option<[[u8; 256]; 3]>, px: [u8; 4]) -> [u8; 4] {
    if let Some(t) = tables {
        return [
            t[0][px[0] as usize],
            t[1][px[1] as usize],
            t[2][px[2] as usize],
            px[3],
        ];
    }
    match effect {
        Adjustment::HueSaturation(h) => h.apply(px),
        Adjustment::BlackWhite { .. } | Adjustment::Vibrance { .. } => {
            adjust::color_adjust(effect, px)
        }
        _ => px,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn masked_adjustments_remain_editable_and_follow_source_pixels() {
        let mut d = Document::new_with_background(
            "Adjustment",
            4,
            4,
            crate::Color::from_rgba8([80, 100, 120, 255]),
        );
        let source = d.layers[0].id;
        let before = d.composite_rgba8();
        let id = create(&mut d, Settings::new(Kind::BrightnessContrast));
        assert_eq!(d.composite_rgba8(), before);
        d.layer_mut(id)
            .unwrap()
            .mask
            .as_mut()
            .unwrap()
            .image
            .set_pixel(1, 1, [255; 4]);
        d.mark_dirty();
        assert_ne!(d.composite_rgba8()[20..24], before[20..24]);
        assert_eq!(
            d.layers[0].image().unwrap().pixel(1, 1),
            [80, 100, 120, 255]
        );
        d.layer_mut(source)
            .unwrap()
            .image_mut()
            .unwrap()
            .set_pixel(1, 1, [140, 140, 140, 255]);
        d.mark_dirty();
        let mapped = d.composite_rgba8();
        assert!(mapped[20] > 140);
        assert_eq!(&mapped[..4], &before[..4]);
        d.layer_mut(id).unwrap().adjustment.as_mut().unwrap().values[0] = -50.;
        d.mark_dirty();
        assert!(d.composite_rgba8()[20] < 140);
        d.mask_target = false;
        assert!(d.edit_target().is_none());
    }
}
