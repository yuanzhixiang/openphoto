// Canvas: checkerboard transparency + document image + pixel grid at high zoom.
// All coordinates are physical pixels (framebuffer coordinates), independent
// of the viewport egui sets.

struct Uniforms {
    // xy: screen position of the document's top-left; z: zoom (physical px per
    // document px); w: checkerboard cell size
    origin_zoom: vec4<f32>,
    // xy: document size; z: max mip level; w: pixel grid enabled
    doc: vec4<f32>,
    // The crop shield: box center and half size (document px), its color
    // (sRGB) and angle (clockwise radians), opacity and whether it's on
    shield_box: vec4<f32>,
    shield_color: vec4<f32>,
    shield_params: vec4<f32>,
    // The image turned: pivot (document px), angle (clockwise radians), on
    rotation: vec4<f32>,
    // The view mirrored left to right: x: axis (document px), y: on
    mirror: vec4<f32>,
};

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // A single triangle covering the whole viewport
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let local = frag.xy - u.origin_zoom.xy;
    let zoom = u.origin_zoom.z;
    // The view's (upright) point, then the image point shown there
    var b = local / zoom;
    if u.mirror.y > 0.5 {
        b.x = 2.0 * u.mirror.x - b.x;
    }
    var d = b;
    if u.rotation.w > 0.5 {
        let a = u.rotation.z;
        let q = b - u.rotation.xy;
        d = u.rotation.xy + vec2<f32>(q.x * cos(a) - q.y * sin(a), q.x * sin(a) + q.y * cos(a));
    }
    if d.x < 0.0 || d.y < 0.0 || d.x >= u.doc.x || d.y >= u.doc.y {
        discard;
    }

    let cell = vec2<i32>(floor(local / u.origin_zoom.w));
    let checker = select(1.0, 0.8, ((cell.x + cell.y) & 1) == 1);

    // The texture may have been downscaled to fit the GPU limit, so map proportionally
    let dims = vec2<f32>(textureDimensions(tex, 0));
    let uv = d / u.doc.xy;
    var c: vec4<f32>;
    if zoom >= 1.0 {
        c = textureLoad(tex, vec2<i32>(floor(uv * dims)), 0);
    } else {
        let lod = clamp(log2(dims.x / u.doc.x / zoom), 0.0, u.doc.z);
        c = textureSampleLevel(tex, samp, uv, lod);
    }

    // The texture is premultiplied
    var rgb = c.rgb + vec3<f32>(checker) * (1.0 - c.a);

    // The crop shield, mixed in linear light like Photoshop's
    if u.shield_params.y > 0.5 {
        let a = u.shield_color.w;
        let p = b - u.shield_box.xy;
        // Into the box's own axes (it turns clockwise by a)
        let q = vec2<f32>(p.x * cos(a) + p.y * sin(a), -p.x * sin(a) + p.y * cos(a));
        if abs(q.x) > u.shield_box.z || abs(q.y) > u.shield_box.w {
            let lin = mix(to_linear(rgb), to_linear(u.shield_color.rgb), u.shield_params.x);
            rgb = to_srgb(lin);
        }
    }

    if u.doc.w > 0.5 && zoom >= 6.0 {
        // The screen pixel whose left (top) edge lies within one screen
        // pixel after an image pixel's edge draws the line: one pixel wide
        // whether or not the image is aligned to screen pixels
        let f = fract(d - vec2<f32>(0.5 / zoom)) * zoom;
        if min(f.x, f.y) < 1.0 {
            rgb = mix(rgb, vec3<f32>(0.55), 0.35);
        }
    }
    return vec4<f32>(rgb, 1.0);
}
