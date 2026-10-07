use std::collections::HashMap;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

/// Composite image to display (tightly packed straight RGBA8).
pub struct CanvasImage {
    /// Texture slot key, one per document.
    pub key: u64,
    /// Content revision; the texture is re-uploaded when it changes.
    pub revision: u64,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// View transform.
#[derive(Clone, Copy, Debug)]
pub struct CanvasView {
    /// Screen position of the document's top-left corner, in points.
    pub origin: [f32; 2],
    /// Zoom: **physical pixels** per document pixel.
    /// As in Photoshop, 100% means one image pixel per screen pixel.
    pub zoom: f32,
    pub pixel_grid: bool,
    /// The Crop tool's shield over the image outside the crop box.
    pub shield: Option<Shield>,
    /// The image turned about a document point by an angle (clockwise
    /// radians): the Crop tool shows the image turned under an upright box.
    /// A screen point maps to the document point turned by the angle about
    /// the pivot.
    pub rotation: Option<([f32; 2], f32)>,
    /// View › Flip Horizontal: the view mirrored about this vertical line
    /// (document x), before the rotation is applied.
    pub mirror: Option<f32>,
}

/// The crop shield: everything outside a (possibly turned) box is mixed
/// with `color` at `opacity`, in linear light as Photoshop does it (75% of
/// the canvas gray over white gives 141, not 94).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shield {
    /// The box's center and half size, in document pixels.
    pub center: [f32; 2],
    pub half: [f32; 2],
    /// Clockwise, in radians.
    pub angle: f32,
    /// sRGB, 0..1.
    pub color: [f32; 3],
    pub opacity: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    origin_zoom: [f32; 4],
    doc: [f32; 4],
    shield_box: [f32; 4],
    shield_color: [f32; 4],
    shield_params: [f32; 4],
    rotation: [f32; 4],
    mirror: [f32; 4],
}

struct Slot {
    revision: u64,
    mip_count: u32,
    texture: wgpu::Texture,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

struct CanvasResources {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    slots: HashMap<u64, Slot>,
}

/// Creates the pipeline and registers it in egui-wgpu's callback resources.
/// Call once at startup.
pub fn install(render_state: &egui_wgpu::RenderState) {
    let device = &render_state.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("canvas"),
        source: wgpu::ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("canvas"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("canvas"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("canvas"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(render_state.target_format.into())],
        }),
        multiview_mask: None,
        cache: None,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("canvas"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    render_state
        .renderer
        .write()
        .callback_resources
        .insert(CanvasResources {
            pipeline,
            layout,
            sampler,
            slots: HashMap::new(),
        });
}

/// Builds an egui paint command that draws the canvas within `rect`.
pub fn paint_callback(
    rect: egui::Rect,
    image: Arc<CanvasImage>,
    view: CanvasView,
) -> egui::PaintCallback {
    egui_wgpu::Callback::new_paint_callback(rect, CanvasCallback { image, view })
}

struct CanvasCallback {
    image: Arc<CanvasImage>,
    view: CanvasView,
}

impl CallbackTrait for CanvasCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let res: &mut CanvasResources = resources.get_mut().expect("canvas renderer installed");
        let img = &self.image;

        let stale = res
            .slots
            .get(&img.key)
            .is_none_or(|s| s.revision != img.revision);
        if stale {
            let max_dim = device.limits().max_texture_dimension_2d;
            let levels = build_mips(img, max_dim);
            // Same size: write into the texture the slot has (creating a
            // new one each time a layer is dragged is too slow)
            let reuse = res.slots.get_mut(&img.key).filter(|slot| {
                let size = slot.texture.size();
                levels
                    .first()
                    .is_some_and(|l| (l.width, l.height) == (size.width, size.height))
                    && slot.mip_count as usize == levels.len()
            });
            match reuse {
                Some(slot) => {
                    upload(queue, &slot.texture, &levels);
                    slot.revision = img.revision;
                }
                None => {
                    let slot = create_slot(device, queue, res, img.revision, levels);
                    res.slots.insert(img.key, slot);
                }
            }
        }
        let slot = &res.slots[&img.key];

        let ppp = screen.pixels_per_point;
        let u = Uniforms {
            origin_zoom: [
                self.view.origin[0] * ppp,
                self.view.origin[1] * ppp,
                self.view.zoom,
                (8.0 * ppp).round(),
            ],
            doc: [
                img.width as f32,
                img.height as f32,
                (slot.mip_count - 1) as f32,
                self.view.pixel_grid as u32 as f32,
            ],
            shield_box: self.view.shield.map_or([0.0; 4], |s| {
                [s.center[0], s.center[1], s.half[0], s.half[1]]
            }),
            shield_color: self
                .view
                .shield
                .map_or([0.0; 4], |s| [s.color[0], s.color[1], s.color[2], s.angle]),
            shield_params: [
                self.view.shield.map_or(0.0, |s| s.opacity),
                self.view.shield.is_some() as u32 as f32,
                0.0,
                0.0,
            ],
            rotation: self
                .view
                .rotation
                .map_or([0.0; 4], |(p, a)| [p[0], p[1], a, 1.0]),
            mirror: self.view.mirror.map_or([0.0; 4], |x| [x, 1.0, 0.0, 0.0]),
        };
        queue.write_buffer(&slot.uniforms, 0, bytemuck::bytes_of(&u));
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let res: &CanvasResources = resources.get().expect("canvas renderer installed");
        let Some(slot) = res.slots.get(&self.image.key) else {
            return;
        };
        pass.set_pipeline(&res.pipeline);
        pass.set_bind_group(0, &slot.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn create_slot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    res: &CanvasResources,
    revision: u64,
    mut levels: Vec<MipLevel>,
) -> Slot {
    if levels.is_empty() {
        levels.push(MipLevel {
            width: 1,
            height: 1,
            data: vec![0; 4],
        });
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("canvas"),
        size: wgpu::Extent3d {
            width: levels[0].width,
            height: levels[0].height,
            depth_or_array_layers: 1,
        },
        mip_level_count: levels.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Not an sRGB format: Photoshop blends and displays in gamma-encoded space
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    upload(queue, &texture, &levels);

    let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("canvas uniforms"),
        size: size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let view = texture.create_view(&Default::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("canvas"),
        layout: &res.layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&res.sampler),
            },
        ],
    });

    Slot {
        revision,
        mip_count: levels.len() as u32,
        texture,
        uniforms,
        bind_group,
    }
}

/// Writes every mip level into `texture`.
fn upload(queue: &wgpu::Queue, texture: &wgpu::Texture, levels: &[MipLevel]) {
    for (i, level) in levels.iter().enumerate() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: i as u32,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &level.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(level.width * 4),
                rows_per_image: Some(level.height),
            },
            wgpu::Extent3d {
                width: level.width,
                height: level.height,
                depth_or_array_layers: 1,
            },
        );
    }
}

struct MipLevel {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

/// Premultiplies, then builds mips with a 2×2 box filter. Levels larger than
/// the GPU texture limit are skipped.
fn build_mips(img: &CanvasImage, max_dim: u32) -> Vec<MipLevel> {
    if img.width == 0 || img.height == 0 {
        return Vec::new();
    }
    let mut premul = vec![0u8; img.pixels.len()];
    in_bands(&mut premul, img.width as usize * 4, |y0, band| {
        let src = &img.pixels[y0 * img.width as usize * 4..][..band.len()];
        for (d, p) in band
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(src.as_chunks::<4>().0)
        {
            let a = p[3] as u32;
            let m = |c: u8| ((c as u32 * a + 127) / 255) as u8;
            *d = [m(p[0]), m(p[1]), m(p[2]), p[3]];
        }
    });

    let mut levels = Vec::new();
    let mut cur = MipLevel {
        width: img.width,
        height: img.height,
        data: premul,
    };
    loop {
        let next = (cur.width > 1 || cur.height > 1).then(|| downsample(&cur));
        if cur.width <= max_dim && cur.height <= max_dim {
            levels.push(cur);
        }
        match next {
            Some(n) => cur = n,
            None => break,
        }
    }
    levels
}

/// Runs `f(first_row, rows)` over bands of `rows` (rows of `row_len`
/// bytes) on separate threads.
fn in_bands(data: &mut [u8], row_len: usize, f: impl Fn(usize, &mut [u8]) + Sync) {
    let rows = data.len() / row_len.max(1);
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let band = rows.div_ceil(threads).max(1);
    if rows < 64 || threads == 1 {
        f(0, data);
        return;
    }
    std::thread::scope(|scope| {
        for (i, chunk) in data.chunks_mut(band * row_len).enumerate() {
            let f = &f;
            scope.spawn(move || f(i * band, chunk));
        }
    });
}

fn downsample(src: &MipLevel) -> MipLevel {
    let (w, h) = ((src.width / 2).max(1), (src.height / 2).max(1));
    let mut data = vec![0u8; (w * h * 4) as usize];
    let sw = src.width as usize;
    in_bands(&mut data, w as usize * 4, |first, band| {
        for (r, row) in band.chunks_mut(w as usize * 4).enumerate() {
            let y = first + r;
            let y0 = (y * 2).min(src.height as usize - 1);
            let y1 = (y * 2 + 1).min(src.height as usize - 1);
            for x in 0..w as usize {
                let x0 = (x * 2).min(sw - 1);
                let x1 = (x * 2 + 1).min(sw - 1);
                for c in 0..4 {
                    let s = |xx: usize, yy: usize| src.data[(yy * sw + xx) * 4 + c] as u32;
                    let sum = s(x0, y0) + s(x1, y0) + s(x0, y1) + s(x1, y1);
                    row[x * 4 + c] = ((sum + 2) / 4) as u8;
                }
            }
        }
    });
    MipLevel {
        width: w,
        height: h,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mip chain as the spec describes it, one pixel at a time.
    fn reference(img: &CanvasImage) -> Vec<MipLevel> {
        let premul: Vec<u8> = img
            .pixels
            .chunks(4)
            .flat_map(|p| {
                let a = p[3] as u32;
                let m = |c: u8| ((c as u32 * a + 127) / 255) as u8;
                [m(p[0]), m(p[1]), m(p[2]), p[3]]
            })
            .collect();
        let mut levels = vec![MipLevel {
            width: img.width,
            height: img.height,
            data: premul,
        }];
        while levels.last().is_some_and(|l| l.width > 1 || l.height > 1) {
            let src = levels.last().unwrap();
            let (w, h) = ((src.width / 2).max(1), (src.height / 2).max(1));
            let (sw, sh) = (src.width as usize, src.height as usize);
            let mut data = Vec::new();
            for y in 0..h as usize {
                for x in 0..w as usize {
                    for c in 0..4 {
                        let s = |xx: usize, yy: usize| {
                            src.data[(yy.min(sh - 1) * sw + xx.min(sw - 1)) * 4 + c] as u32
                        };
                        let sum = s(2 * x, 2 * y)
                            + s(2 * x + 1, 2 * y)
                            + s(2 * x, 2 * y + 1)
                            + s(2 * x + 1, 2 * y + 1);
                        data.push(((sum + 2) / 4) as u8);
                    }
                }
            }
            levels.push(MipLevel {
                width: w,
                height: h,
                data,
            });
        }
        levels
    }

    #[test]
    fn mips_built_in_bands_match_the_reference() {
        // Odd sizes, tall enough to split into bands across threads
        for (w, h) in [(131u32, 197u32), (1, 300), (300, 1), (7, 5)] {
            let pixels: Vec<u8> = (0..w * h * 4)
                .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
                .collect();
            let img = CanvasImage {
                key: 1,
                revision: 1,
                width: w,
                height: h,
                pixels,
            };
            let got = build_mips(&img, 16384);
            let want = reference(&img);
            assert_eq!(got.len(), want.len(), "{w}x{h}");
            for (g, r) in got.iter().zip(&want) {
                assert_eq!((g.width, g.height), (r.width, r.height));
                assert!(g.data == r.data, "{w}x{h} level {}x{}", g.width, g.height);
            }
        }
    }

    #[test]
    fn levels_over_the_texture_limit_are_skipped() {
        let img = CanvasImage {
            key: 1,
            revision: 1,
            width: 64,
            height: 32,
            pixels: vec![255; 64 * 32 * 4],
        };
        let levels = build_mips(&img, 16);
        assert_eq!((levels[0].width, levels[0].height), (16, 8));
    }
}
