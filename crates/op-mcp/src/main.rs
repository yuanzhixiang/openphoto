//! OpenPhoto MCP server.
//!
//! The MCP interface exposes exactly two tools:
//! - `search_tools`: keyword search over the internal operation catalog.
//!   Returns matching tool names along with their full definitions.
//! - `call_tool`: invokes one internal tool by name with arguments.
//!
//! The internal catalog wraps real `op-core` / `op-io` operations
//! (documents, adjustments, filters, layers, selections, canvas,
//! painting, history). Nothing is reimplemented: every handler calls
//! the same functions the application itself uses.
//!
//! Protocol: MCP over stdio, newline-delimited JSON-RPC 2.0.

use std::collections::HashMap;
use std::io::{BufRead, Write as IoWrite};
use std::path::Path;

use op_core::adjust::{Adjustment, HUE_RANGES, HueRange, HueSaturation, Levels};
use op_core::filter::{Filter, OffsetFill, SpherizeMode, WindMethod};
use op_core::gradient::{GradientKind, GradientOptions, Method as GradientMethod};
use op_core::{Anchor, BlendMode, Color, Document, History, Layer, LayerId, Selection, TiledImage};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

struct ManagedDoc {
    doc: Document,
    history: History,
}

struct AppState {
    docs: HashMap<String, ManagedDoc>,
    active: Option<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            docs: HashMap::new(),
            active: None,
        }
    }

    fn insert(&mut self, doc: Document, history_name: &str) -> String {
        let id = doc.id.0.to_string();
        let history = History::new(&doc, history_name);
        self.docs.insert(id.clone(), ManagedDoc { doc, history });
        self.active = Some(id.clone());
        id
    }

    /// Resolve the target document id from args (`doc`) or the active doc.
    fn resolve_id(&self, args: &Value) -> Result<String, String> {
        if let Some(s) = args.get("doc").and_then(|v| v.as_str()) {
            if self.docs.contains_key(s) {
                return Ok(s.to_string());
            }
            return Err(format!("unknown document '{s}'"));
        }
        self.active
            .clone()
            .ok_or_else(|| "no open document; use new_document or open_image first".to_string())
    }

    fn get(&self, args: &Value) -> Result<&ManagedDoc, String> {
        let id = self.resolve_id(args)?;
        self.docs
            .get(&id)
            .ok_or_else(|| format!("unknown document '{id}'"))
    }

    fn get_mut(&mut self, args: &Value) -> Result<(String, &mut ManagedDoc), String> {
        let id = self.resolve_id(args)?;
        let m = self
            .docs
            .get_mut(&id)
            .ok_or_else(|| format!("unknown document '{id}'"))?;
        Ok((id, m))
    }
}

// ---------------------------------------------------------------------------
// Argument helpers
// ---------------------------------------------------------------------------

fn req_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing required string parameter '{key}'"))
}

fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_f64(args: &Value, key: &str, default: f64) -> Result<f64, String> {
    match args.get(key) {
        None => Ok(default),
        Some(v) => v
            .as_f64()
            .ok_or_else(|| format!("parameter '{key}' must be a number")),
    }
}

fn get_u32(args: &Value, key: &str, default: u32) -> Result<u32, String> {
    match args.get(key) {
        None => Ok(default),
        Some(v) => {
            if let Some(n) = v.as_u64() {
                u32::try_from(n).map_err(|_| format!("parameter '{key}' out of range"))
            } else if let Some(n) = v.as_i64() {
                u32::try_from(n).map_err(|_| format!("parameter '{key}' out of range"))
            } else if let Some(f) = v.as_f64() {
                if f >= 0.0 && f <= u32::MAX as f64 {
                    Ok(f as u32)
                } else {
                    Err(format!("parameter '{key}' out of range"))
                }
            } else {
                Err(format!("parameter '{key}' must be a number"))
            }
        }
    }
}

fn req_u32(args: &Value, key: &str) -> Result<u32, String> {
    match args.get(key) {
        None => Err(format!("missing required parameter '{key}'")),
        Some(_) => get_u32(args, key, 0),
    }
}

fn get_i32(args: &Value, key: &str, default: i32) -> Result<i32, String> {
    match args.get(key) {
        None => Ok(default),
        Some(v) => {
            if let Some(n) = v.as_i64() {
                i32::try_from(n).map_err(|_| format!("parameter '{key}' out of range"))
            } else if let Some(n) = v.as_u64() {
                i32::try_from(n).map_err(|_| format!("parameter '{key}' out of range"))
            } else if let Some(f) = v.as_f64() {
                Ok(f as i32)
            } else {
                Err(format!("parameter '{key}' must be a number"))
            }
        }
    }
}

/// The largest width or height a document may have, as in the app
/// (`op-ui` dialogs' `MAX_DIMENSION`).
const MAX_DIMENSION: u32 = 30_000;

/// Fails unless `v` is within `lo..=hi` (the range the app's dialog for
/// that parameter allows).
fn check_range<T: PartialOrd + std::fmt::Display>(
    key: &str,
    v: T,
    lo: T,
    hi: T,
) -> Result<T, String> {
    if v < lo || v > hi {
        return Err(format!("{key} must be {lo}-{hi}"));
    }
    Ok(v)
}

fn check_size(width: u32, height: u32) -> Result<(), String> {
    if !(1..=MAX_DIMENSION).contains(&width) || !(1..=MAX_DIMENSION).contains(&height) {
        return Err(format!("width and height must be 1-{MAX_DIMENSION}"));
    }
    Ok(())
}

fn get_u8(args: &Value, key: &str, default: u8) -> Result<u8, String> {
    let n = get_u32(args, key, default as u32)?;
    u8::try_from(n).map_err(|_| format!("parameter '{key}' must be 0-255"))
}

#[allow(dead_code)]
fn req_u8(args: &Value, key: &str) -> Result<u8, String> {
    let n = req_u32(args, key)?;
    u8::try_from(n).map_err(|_| format!("parameter '{key}' must be 0-255"))
}

fn get_f32(args: &Value, key: &str, default: f32) -> Result<f32, String> {
    Ok(get_f64(args, key, default as f64)? as f32)
}

fn get_bool(args: &Value, key: &str, default: bool) -> Result<bool, String> {
    match args.get(key) {
        None => Ok(default),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| format!("parameter '{key}' must be boolean")),
    }
}

fn num_array(v: &Value, len: usize, key: &str) -> Result<Vec<f64>, String> {
    let a = v
        .as_array()
        .ok_or_else(|| format!("parameter '{key}' must be an array of {len} numbers"))?;
    if a.len() != len {
        return Err(format!("parameter '{key}' must have exactly {len} numbers"));
    }
    a.iter()
        .map(|x| {
            x.as_f64()
                .ok_or_else(|| format!("parameter '{key}' must be numbers"))
        })
        .collect()
}

fn req_rgb(args: &Value) -> Result<[u8; 3], String> {
    opt_rgb(args, "color")?
        .ok_or_else(|| "missing required parameter 'color' ([r,g,b])".to_string())
}

fn opt_rgb(args: &Value, key: &str) -> Result<Option<[u8; 3]>, String> {
    match args.get(key) {
        None => {
            // Convenience: r/g/b keys.
            let (r, g, b) = (args.get("r"), args.get("g"), args.get("b"));
            if r.is_none() && g.is_none() && b.is_none() {
                return Ok(None);
            }
            Ok(Some([
                get_u8(args, "r", 0)?,
                get_u8(args, "g", 0)?,
                get_u8(args, "b", 0)?,
            ]))
        }
        Some(Value::Null) => Ok(None),
        Some(v) => {
            let n = num_array(v, 3, key)?;
            Ok(Some([
                n[0].clamp(0.0, 255.0) as u8,
                n[1].clamp(0.0, 255.0) as u8,
                n[2].clamp(0.0, 255.0) as u8,
            ]))
        }
    }
}

fn req_rgb_key(args: &Value, key: &str) -> Result<[u8; 3], String> {
    match args.get(key) {
        None => Err(format!("missing required parameter '{key}' ([r,g,b])")),
        Some(v) => {
            let n = num_array(v, 3, key)?;
            Ok([
                n[0].clamp(0.0, 255.0) as u8,
                n[1].clamp(0.0, 255.0) as u8,
                n[2].clamp(0.0, 255.0) as u8,
            ])
        }
    }
}

fn opt_i32_array(args: &Value, key: &str, len: usize, default: i32) -> Result<Vec<i32>, String> {
    match args.get(key) {
        None => Ok(vec![default; len]),
        Some(v) => {
            let a = v
                .as_array()
                .ok_or_else(|| format!("parameter '{key}' must be an array of {len} integers"))?;
            if a.len() != len {
                return Err(format!(
                    "parameter '{key}' must have exactly {len} integers"
                ));
            }
            a.iter()
                .map(|x| {
                    x.as_i64()
                        .and_then(|n| i32::try_from(n).ok())
                        .or_else(|| x.as_f64().map(|f| f as i32))
                        .ok_or_else(|| format!("parameter '{key}' must be integers"))
                })
                .collect()
        }
    }
}

fn parse_layer_id(args: &Value, doc: &Document) -> Result<LayerId, String> {
    if let Some(v) = args.get("layer") {
        let n = v
            .as_u64()
            .ok_or("parameter 'layer' must be a layer id number")?;
        let id = LayerId(n);
        if doc.layer(id).is_none() {
            return Err(format!("unknown layer {n}"));
        }
        return Ok(id);
    }
    doc.active_layer
        .ok_or_else(|| "no active layer".to_string())
}

// ---------------------------------------------------------------------------
// Enum parsing
// ---------------------------------------------------------------------------

fn parse_blend_mode(s: &str) -> Result<BlendMode, String> {
    let k: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    let m = match k.as_str() {
        "normal" => BlendMode::Normal,
        "passthrough" => BlendMode::PassThrough,
        "dissolve" => BlendMode::Dissolve,
        "darken" => BlendMode::Darken,
        "multiply" => BlendMode::Multiply,
        "colorburn" => BlendMode::ColorBurn,
        "linearburn" => BlendMode::LinearBurn,
        "darkercolor" => BlendMode::DarkerColor,
        "lighten" => BlendMode::Lighten,
        "screen" => BlendMode::Screen,
        "colordodge" => BlendMode::ColorDodge,
        "lineardodge" | "linearadd" | "add" => BlendMode::LinearDodge,
        "lightercolor" => BlendMode::LighterColor,
        "overlay" => BlendMode::Overlay,
        "softlight" => BlendMode::SoftLight,
        "hardlight" => BlendMode::HardLight,
        "vividlight" => BlendMode::VividLight,
        "linearlight" => BlendMode::LinearLight,
        "pinlight" => BlendMode::PinLight,
        "hardmix" => BlendMode::HardMix,
        "difference" => BlendMode::Difference,
        "exclusion" => BlendMode::Exclusion,
        "subtract" => BlendMode::Subtract,
        "divide" => BlendMode::Divide,
        "hue" => BlendMode::Hue,
        "saturation" => BlendMode::Saturation,
        "color" => BlendMode::Color,
        "luminosity" => BlendMode::Luminosity,
        _ => return Err(format!("unknown blend mode '{s}'")),
    };
    Ok(m)
}

fn blend_label(m: BlendMode) -> &'static str {
    m.label()
}

fn parse_orientation(s: &str) -> Result<op_core::image_ops::Orientation, String> {
    use op_core::image_ops::Orientation as O;
    let k: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    match k.as_str() {
        "rotate180" | "rot180" | "180" => Ok(O::Rotate180),
        "rotate90cw" | "rotate90clockwise" | "rot90cw" | "90cw" | "clockwise" => {
            Ok(O::Rotate90Clockwise)
        }
        "rotate90ccw" | "rotate90counterclockwise" | "rot90ccw" | "90ccw" | "counterclockwise" => {
            Ok(O::Rotate90CounterClockwise)
        }
        "fliphorizontal" | "fliph" | "horizontal" => Ok(O::FlipHorizontal),
        "flipvertical" | "flipv" | "vertical" => Ok(O::FlipVertical),
        _ => Err("unknown orientation; use rotate_180, rotate_90_cw, rotate_90_ccw, flip_horizontal, flip_vertical".to_string()),
    }
}

fn parse_resample(s: &str) -> Result<op_core::image_ops::Resample, String> {
    use op_core::image_ops::Resample as R;
    let k = s.to_lowercase();
    if k.contains("nearest") {
        Ok(R::NearestNeighbor)
    } else if k.contains("bilinear") {
        Ok(R::Bilinear)
    } else if k.contains("bicubicsharper") || k.contains("sharper") {
        Ok(R::BicubicSharper)
    } else if k.contains("bicubicsmoother") || k.contains("smoother") {
        Ok(R::BicubicSmoother)
    } else if k.contains("bicubic") {
        Ok(R::Bicubic)
    } else if k.contains("preserve") && k.contains('2') {
        Ok(R::PreserveDetails2)
    } else if k.contains("preserve") {
        Ok(R::PreserveDetails)
    } else if k.contains("auto") {
        Ok(R::Automatic)
    } else {
        Err("unknown resample method; use automatic, nearest_neighbor, bilinear, bicubic, bicubic_smoother, bicubic_sharper".to_string())
    }
}

fn parse_trim_basis(s: &str) -> Result<op_core::image_ops::TrimBasis, String> {
    use op_core::image_ops::TrimBasis as T;
    let k = s.to_lowercase();
    if k.contains("transparent") {
        Ok(T::Transparent)
    } else if k.contains("topleft") || k.contains("top_left") {
        Ok(T::TopLeftColor)
    } else if k.contains("bottomright") || k.contains("bottom_right") {
        Ok(T::BottomRightColor)
    } else {
        Err("unknown trim basis; use transparent, top_left, bottom_right".to_string())
    }
}

fn parse_arrange(s: &str) -> Result<op_core::layer_ops::Arrange, String> {
    use op_core::layer_ops::Arrange as A;
    let k: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    match k.as_str() {
        "bringtofront" | "front" => Ok(A::BringToFront),
        "bringforward" | "forward" => Ok(A::BringForward),
        "sendbackward" | "backward" => Ok(A::SendBackward),
        "sendtoback" | "back" => Ok(A::SendToBack),
        _ => Err(
            "unknown arrange; use bring_to_front, bring_forward, send_backward, send_to_back"
                .to_string(),
        ),
    }
}

fn parse_align(s: &str) -> Result<op_core::align::Align, String> {
    use op_core::align::Align as A;
    let k: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    match k.as_str() {
        "top" | "topedges" => Ok(A::Top),
        "verticalcenter" | "verticalcenters" | "vcenter" => Ok(A::VerticalCenter),
        "bottom" | "bottomedges" => Ok(A::Bottom),
        "left" | "leftedges" => Ok(A::Left),
        "horizontalcenter" | "horizontalcenters" | "hcenter" => Ok(A::HorizontalCenter),
        "right" | "rightedges" => Ok(A::Right),
        _ => Err(
            "unknown align; use top, bottom, left, right, vertical_center, horizontal_center"
                .to_string(),
        ),
    }
}

fn parse_distribute(s: &str) -> Result<op_core::align::Distribute, String> {
    use op_core::align::Distribute as D;
    let k: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    match k.as_str() {
        "top" => Ok(D::Top),
        "verticalcenter" | "vcenter" => Ok(D::VerticalCenter),
        "bottom" => Ok(D::Bottom),
        "left" => Ok(D::Left),
        "horizontalcenter" | "hcenter" => Ok(D::HorizontalCenter),
        "right" => Ok(D::Right),
        "horizontally" => Ok(D::Horizontally),
        "vertically" => Ok(D::Vertically),
        _ => Err("unknown distribute; use top, bottom, left, right, vertical_center, horizontal_center, horizontally, vertically".to_string()),
    }
}

fn parse_offset_fill(s: &str) -> Result<OffsetFill, String> {
    let k = s.to_lowercase();
    if k.contains("wrap") {
        Ok(OffsetFill::Wrap)
    } else if k.contains("repeat") || k.contains("edge") {
        Ok(OffsetFill::RepeatEdges)
    } else if k.contains("background") {
        Ok(OffsetFill::Background)
    } else {
        Err("unknown offset fill; use background, repeat_edges, wrap".to_string())
    }
}

fn parse_wind_method(s: &str) -> Result<WindMethod, String> {
    let k = s.to_lowercase();
    if k.contains("blast") {
        Ok(WindMethod::Blast)
    } else if k.contains("stagger") {
        Ok(WindMethod::Stagger)
    } else if k.contains("wind") {
        Ok(WindMethod::Wind)
    } else {
        Err("unknown wind method; use wind, blast, stagger".to_string())
    }
}

fn parse_spherize_mode(s: &str) -> Result<SpherizeMode, String> {
    let k = s.to_lowercase();
    if k.contains("horizontal") {
        Ok(SpherizeMode::HorizontalOnly)
    } else if k.contains("vertical") {
        Ok(SpherizeMode::VerticalOnly)
    } else {
        Ok(SpherizeMode::Normal)
    }
}

fn parse_gradient_kind(s: &str) -> Result<GradientKind, String> {
    let k = s.to_lowercase();
    if k.contains("radial") {
        Ok(GradientKind::Radial)
    } else if k.contains("angle") {
        Ok(GradientKind::Angle)
    } else if k.contains("reflect") {
        Ok(GradientKind::Reflected)
    } else if k.contains("diamond") {
        Ok(GradientKind::Diamond)
    } else {
        Ok(GradientKind::Linear)
    }
}

fn parse_gradient_method(s: &str) -> Result<GradientMethod, String> {
    let k = s.to_lowercase();
    if k.contains("perceptual") {
        Ok(GradientMethod::Perceptual)
    } else if k.contains("linear") {
        Ok(GradientMethod::Linear)
    } else if k.contains("classic") {
        Ok(GradientMethod::Classic)
    } else {
        Ok(GradientMethod::Smooth)
    }
}

fn parse_new_mask(s: &str) -> Result<op_core::layer_ops::NewMask, String> {
    use op_core::layer_ops::NewMask as M;
    let k: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    match k.as_str() {
        "revealall" => Ok(M::RevealAll),
        "hideall" => Ok(M::HideAll),
        "revealselection" => Ok(M::RevealSelection),
        "hideselection" => Ok(M::HideSelection),
        _ => Err(
            "unknown mask kind; use reveal_all, hide_all, reveal_selection, hide_selection"
                .to_string(),
        ),
    }
}

fn bg_color(args: &Value) -> Result<[u8; 3], String> {
    if let Some(v) = args.get("background") {
        let n = num_array(v, 3, "background")?;
        return Ok([
            n[0].clamp(0.0, 255.0) as u8,
            n[1].clamp(0.0, 255.0) as u8,
            n[2].clamp(0.0, 255.0) as u8,
        ]);
    }
    Ok(opt_rgb(args, "background_color")?.unwrap_or([255, 255, 255]))
}

fn bg_as_color(args: &Value) -> Result<Color, String> {
    let [r, g, b] = bg_color(args)?;
    Ok(Color::from_rgba8([r, g, b, 255]))
}

// ---------------------------------------------------------------------------
// Summaries
// ---------------------------------------------------------------------------

fn layer_json(doc: &Document, l: &Layer) -> Value {
    json!({
        "id": l.id.0,
        "name": l.name,
        "visible": l.visible,
        "shown": doc.is_shown(l.id),
        "opacity": l.opacity,
        "fill": l.fill,
        "blend_mode": blend_label(l.blend_mode),
        "is_background": l.is_background,
        "is_group": l.is_group(),
        "has_mask": l.mask.is_some(),
        "parent": l.parent.map(|p| p.0),
        "active": doc.active_layer == Some(l.id),
    })
}

fn doc_info(id: &str, m: &ManagedDoc) -> Value {
    let layers: Vec<Value> = m.doc.layers.iter().map(|l| layer_json(&m.doc, l)).collect();
    json!({
        "doc": id,
        "title": m.doc.title,
        "width": m.doc.width,
        "height": m.doc.height,
        "active_layer": m.doc.active_layer.map(|a| a.0),
        "has_selection": m.doc.selection().is_some(),
        "layer_count": m.doc.layers.len(),
        "layers": layers,
    })
}

fn ok_mut(id: &str, m: &ManagedDoc, extra: Value) -> Value {
    let mut base = json!({
        "doc": id,
        "width": m.doc.width,
        "height": m.doc.height,
        "active_layer": m.doc.active_layer.map(|a| a.0),
        "revision": m.doc.revision(),
    });
    if let (Value::Object(map), Value::Object(base_map)) = (extra, &mut base) {
        for (k, v) in map {
            base_map.insert(k, v);
        }
    }
    base
}

// ---------------------------------------------------------------------------
// Document / file handlers
// ---------------------------------------------------------------------------

fn h_new_document(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let title = opt_str(a, "title").unwrap_or_else(|| "Untitled".to_string());
    let width = get_u32(a, "width", 800)?;
    let height = get_u32(a, "height", 600)?;
    check_size(width, height)?;
    let fill = opt_rgb(a, "fill")?.unwrap_or([255, 255, 255]);
    let color = Color::from_rgba8([fill[0], fill[1], fill[2], 255]);
    let doc = Document::new_with_background(title, width, height, color);
    let id = s.insert(doc, "New");
    let m = s.docs.get(&id).expect("just inserted");
    Ok(json!({"doc": id, "title": m.doc.title, "width": m.doc.width, "height": m.doc.height}))
}

fn h_open_image(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let path = req_str(a, "path")?;
    let doc = op_io::open(Path::new(&path)).map_err(|e| e.to_string())?;
    let (w, h, title) = (doc.width, doc.height, doc.title.clone());
    let id = s.insert(doc, "Open");
    Ok(json!({"doc": id, "title": title, "width": w, "height": h}))
}

fn h_save_image(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let path = req_str(a, "path")?;
    op_io::save(&m.doc, Path::new(&path)).map_err(|e| e.to_string())?;
    Ok(json!({"doc": id, "path": path, "saved": true}))
}

fn h_export_composite(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let path = req_str(a, "path")?;
    op_io::export_composite(&m.doc, Path::new(&path)).map_err(|e| e.to_string())?;
    Ok(json!({"doc": id, "path": path, "exported": true}))
}

fn h_list_documents(s: &mut AppState, _a: &Value) -> Result<Value, String> {
    let docs: Vec<Value> = s
        .docs
        .iter()
        .map(|(id, m)| {
            json!({
                "doc": id,
                "title": m.doc.title,
                "width": m.doc.width,
                "height": m.doc.height,
                "layers": m.doc.layers.len(),
                "active": s.active.as_deref() == Some(id),
            })
        })
        .collect();
    Ok(json!({"documents": docs, "active": s.active}))
}

fn h_get_document_info(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let id = s.resolve_id(a)?;
    let m = s
        .docs
        .get(&id)
        .ok_or_else(|| format!("unknown document '{id}'"))?;
    Ok(doc_info(&id, m))
}

fn h_close_document(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let id = s.resolve_id(a)?;
    if s.docs.remove(&id).is_none() {
        return Err(format!("unknown document '{id}'"));
    }
    if s.active.as_deref() == Some(&id) {
        s.active = s.docs.keys().next().cloned();
    }
    Ok(json!({"closed": id, "active": s.active}))
}

fn h_set_active_document(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let id = req_str(a, "doc")?;
    if !s.docs.contains_key(&id) {
        return Err(format!("unknown document '{id}'"));
    }
    s.active = Some(id.clone());
    Ok(json!({"active": id}))
}

fn h_get_pixel(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let m = s.get(a)?;
    let x = req_u32(a, "x")?;
    let y = req_u32(a, "y")?;
    if x >= m.doc.width || y >= m.doc.height {
        return Err(format!(
            "pixel ({x},{y}) outside {}x{}",
            m.doc.width, m.doc.height
        ));
    }
    let composite = get_bool(a, "composite", true)?;
    let px = if composite {
        let buf = m.doc.composite_rgba8();
        let i = ((y * m.doc.width + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    } else {
        let id = parse_layer_id(a, &m.doc)?;
        let layer = m.doc.layer(id).ok_or("unknown layer".to_string())?;
        let img = layer.image().ok_or("layer is a group".to_string())?;
        img.pixel(x, y)
    };
    Ok(json!({"r": px[0], "g": px[1], "b": px[2], "a": px[3]}))
}

fn h_get_histogram(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let m = s.get(a)?;
    let h = op_core::adjust::rgb_histograms(&m.doc);
    Ok(json!({"r": h[0].to_vec(), "g": h[1].to_vec(), "b": h[2].to_vec()}))
}

// ---------------------------------------------------------------------------
// Adjustment handlers
// ---------------------------------------------------------------------------

fn apply_adjust(s: &mut AppState, a: &Value, adj: Adjustment, name: &str) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    op_core::adjust::apply(&mut m.doc, adj).map_err(|e| e.message(name))?;
    m.history.record(&m.doc, name);
    Ok(ok_mut(&id, m, json!({"applied": name})))
}

fn h_adjust_invert(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_adjust(s, a, Adjustment::Invert, "Invert")
}
fn h_adjust_desaturate(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_adjust(s, a, Adjustment::Desaturate, "Desaturate")
}
fn h_adjust_threshold(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let level = get_u8(a, "level", 128)?.max(1);
    apply_adjust(s, a, Adjustment::Threshold(level), "Threshold")
}
fn h_adjust_posterize(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let levels = get_u8(a, "levels", 4)?.clamp(2, 255);
    apply_adjust(s, a, Adjustment::Posterize(levels), "Posterize")
}
fn h_adjust_equalize(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_adjust(s, a, Adjustment::Equalize, "Equalize")
}
fn h_adjust_equalize_entire(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_adjust(s, a, Adjustment::EqualizeEntireImage, "Equalize")
}
fn h_adjust_auto_tone(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_adjust(s, a, Adjustment::AutoTone, "Auto Tone")
}
fn h_adjust_auto_contrast(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_adjust(s, a, Adjustment::AutoContrast, "Auto Contrast")
}
fn h_adjust_auto_color(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_adjust(s, a, Adjustment::AutoColor, "Auto Color")
}

fn parse_levels_obj(v: Option<&Value>) -> Result<Levels, String> {
    let mut l = Levels::IDENTITY;
    if let Some(o) = v {
        if let Some(n) = o.get("input_black").and_then(|x| x.as_u64()) {
            l.input_black = u8::try_from(n).map_err(|_| "input_black must be 0-255")?;
        }
        if let Some(n) = o.get("input_white").and_then(|x| x.as_u64()) {
            l.input_white = u8::try_from(n).map_err(|_| "input_white must be 0-255")?;
        }
        if let Some(n) = o.get("gamma").and_then(|x| x.as_f64()) {
            if !(0.01..=9.99).contains(&n) {
                return Err("gamma must be 0.01-9.99".to_string());
            }
            l.gamma = n as f32;
        }
        if let Some(n) = o.get("output_black").and_then(|x| x.as_u64()) {
            l.output_black = u8::try_from(n).map_err(|_| "output_black must be 0-255")?;
        }
        if let Some(n) = o.get("output_white").and_then(|x| x.as_u64()) {
            l.output_white = u8::try_from(n).map_err(|_| "output_white must be 0-255")?;
        }
    }
    Ok(l)
}

fn h_adjust_levels(s: &mut AppState, a: &Value) -> Result<Value, String> {
    // Composite params may be top-level or nested under "composite".
    let composite_src = a.get("composite").unwrap_or(a);
    let mut comp = Levels::IDENTITY;
    if let Some(n) = composite_src.get("input_black").and_then(|x| x.as_u64()) {
        comp.input_black = u8::try_from(n).map_err(|_| "input_black must be 0-255")?;
    }
    if let Some(n) = composite_src.get("input_white").and_then(|x| x.as_u64()) {
        comp.input_white = u8::try_from(n).map_err(|_| "input_white must be 0-255")?;
    }
    if let Some(n) = composite_src.get("gamma").and_then(|x| x.as_f64()) {
        if !(0.01..=9.99).contains(&n) {
            return Err("gamma must be 0.01-9.99".to_string());
        }
        comp.gamma = n as f32;
    }
    if let Some(n) = composite_src.get("output_black").and_then(|x| x.as_u64()) {
        comp.output_black = u8::try_from(n).map_err(|_| "output_black must be 0-255")?;
    }
    if let Some(n) = composite_src.get("output_white").and_then(|x| x.as_u64()) {
        comp.output_white = u8::try_from(n).map_err(|_| "output_white must be 0-255")?;
    }
    let r = parse_levels_obj(a.get("red"))?;
    let g = parse_levels_obj(a.get("green"))?;
    let b = parse_levels_obj(a.get("blue"))?;
    apply_adjust(s, a, Adjustment::Levels([comp, r, g, b]), "Levels")
}

fn h_adjust_hue_saturation(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let hue = get_i32(a, "hue", 0)?;
    let sat = get_i32(a, "saturation", 0)?;
    let light = get_i32(a, "lightness", 0)?;
    if !(-180..=180).contains(&hue) {
        return Err("hue must be -180-180".to_string());
    }
    if !(-100..=100).contains(&sat) || !(-100..=100).contains(&light) {
        return Err("saturation and lightness must be -100-100".to_string());
    }
    let colorize = get_bool(a, "colorize", false)?;
    let mut hs = HueSaturation::master(hue, sat, light);
    hs.colorize = colorize;
    if let Some(ranges) = a.get("ranges").and_then(|v| v.as_array()) {
        if ranges.len() != 6 {
            return Err("ranges must have exactly 6 entries".to_string());
        }
        for (i, r) in ranges.iter().enumerate() {
            hs.ranges[i] = HueRange {
                bounds: HUE_RANGES[i],
                hue: r
                    .get("hue")
                    .and_then(|x| x.as_i64())
                    .unwrap_or(0)
                    .clamp(-180, 180) as i32,
                saturation: r
                    .get("saturation")
                    .and_then(|x| x.as_i64())
                    .unwrap_or(0)
                    .clamp(-100, 100) as i32,
                lightness: r
                    .get("lightness")
                    .and_then(|x| x.as_i64())
                    .unwrap_or(0)
                    .clamp(-100, 100) as i32,
            };
        }
    }
    apply_adjust(s, a, Adjustment::HueSaturation(hs), "Hue/Saturation")
}

fn h_adjust_exposure(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let exposure = get_f32(a, "exposure", 0.0)?;
    let offset = get_f32(a, "offset", 0.0)?;
    let gamma = get_f32(a, "gamma", 1.0)?;
    if !(-20.0..=20.0).contains(&exposure) {
        return Err("exposure must be -20-20".to_string());
    }
    if !(-0.5..=0.5).contains(&offset) {
        return Err("offset must be -0.5-0.5".to_string());
    }
    if !(0.01..=9.99).contains(&gamma) {
        return Err("gamma must be 0.01-9.99".to_string());
    }
    apply_adjust(
        s,
        a,
        Adjustment::Exposure {
            exposure,
            offset,
            gamma,
        },
        "Exposure",
    )
}

fn h_adjust_brightness_contrast(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let brightness = get_i32(a, "brightness", 0)?;
    let contrast = get_i32(a, "contrast", 0)?;
    let legacy = get_bool(a, "legacy", false)?;
    if !(-150..=150).contains(&brightness) {
        return Err("brightness must be -150-150".to_string());
    }
    if !(-50..=100).contains(&contrast) {
        return Err("contrast must be -50-100".to_string());
    }
    apply_adjust(
        s,
        a,
        Adjustment::BrightnessContrast {
            brightness,
            contrast,
            legacy,
        },
        "Brightness/Contrast",
    )
}

fn h_adjust_color_balance(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let shadows = opt_i32_array(a, "shadows", 3, 0)?;
    let midtones = opt_i32_array(a, "midtones", 3, 0)?;
    let highlights = opt_i32_array(a, "highlights", 3, 0)?;
    for v in shadows
        .iter()
        .chain(midtones.iter())
        .chain(highlights.iter())
    {
        if !(-100..=100).contains(v) {
            return Err("color balance shifts must be -100-100".to_string());
        }
    }
    let preserve = get_bool(a, "preserve_luminosity", true)?;
    apply_adjust(
        s,
        a,
        Adjustment::ColorBalance {
            shadows: [shadows[0], shadows[1], shadows[2]],
            midtones: [midtones[0], midtones[1], midtones[2]],
            highlights: [highlights[0], highlights[1], highlights[2]],
            preserve_luminosity: preserve,
        },
        "Color Balance",
    )
}

fn h_adjust_black_white(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let w = opt_i32_array(a, "weights", 6, 40)?;
    for v in &w {
        if !(-200..=300).contains(v) {
            return Err("black & white weights must be -200-300".to_string());
        }
    }
    let tint = opt_rgb(a, "tint")?;
    apply_adjust(
        s,
        a,
        Adjustment::BlackWhite {
            weights: [w[0], w[1], w[2], w[3], w[4], w[5]],
            tint,
        },
        "Black & White",
    )
}

fn h_adjust_vibrance(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let vibrance = get_i32(a, "vibrance", 0)?;
    let saturation = get_i32(a, "saturation", 0)?;
    if !(-100..=100).contains(&vibrance) || !(-100..=100).contains(&saturation) {
        return Err("vibrance and saturation must be -100-100".to_string());
    }
    apply_adjust(
        s,
        a,
        Adjustment::Vibrance {
            vibrance,
            saturation,
        },
        "Vibrance",
    )
}

fn h_adjust_photo_filter(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let color = opt_rgb(a, "color")?.unwrap_or([255, 128, 0]);
    let density = get_u8(a, "density", 25)?;
    if density > 100 {
        return Err("density must be 0-100".to_string());
    }
    let preserve = get_bool(a, "preserve_luminosity", true)?;
    apply_adjust(
        s,
        a,
        Adjustment::PhotoFilter {
            color,
            density,
            preserve_luminosity: preserve,
        },
        "Photo Filter",
    )
}

fn h_adjust_gradient_map(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let from = req_rgb_key(a, "from")?;
    let to = req_rgb_key(a, "to")?;
    let method = parse_gradient_method(opt_str(a, "method").as_deref().unwrap_or("smooth"))?;
    apply_adjust(
        s,
        a,
        Adjustment::GradientMap { from, to, method },
        "Gradient Map",
    )
}

fn parse_curve_points(v: Option<&Value>, key: &str) -> Result<Vec<(u8, u8)>, String> {
    match v {
        None => Ok(Vec::new()),
        Some(arr) => {
            let list = arr
                .as_array()
                .ok_or_else(|| format!("'{key}' must be an array of [in,out] pairs"))?;
            if list.len() > 16 {
                return Err(format!("'{key}' holds at most 16 points"));
            }
            list.iter()
                .map(|p| {
                    let pair = p
                        .as_array()
                        .ok_or_else(|| format!("'{key}' entries must be [in,out]"))?;
                    if pair.len() != 2 {
                        return Err(format!("'{key}' entries must be [in,out]"));
                    }
                    let x = pair[0]
                        .as_u64()
                        .ok_or_else(|| format!("'{key}' values must be 0-255"))?;
                    let y = pair[1]
                        .as_u64()
                        .ok_or_else(|| format!("'{key}' values must be 0-255"))?;
                    Ok((
                        u8::try_from(x).map_err(|_| format!("'{key}' values must be 0-255"))?,
                        u8::try_from(y).map_err(|_| format!("'{key}' values must be 0-255"))?,
                    ))
                })
                .collect()
        }
    }
}

fn h_adjust_curves(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let c = parse_curve_points(a.get("points"), "points")?;
    let r = parse_curve_points(a.get("red"), "red")?;
    let g = parse_curve_points(a.get("green"), "green")?;
    let b = parse_curve_points(a.get("blue"), "blue")?;
    if c.is_empty() && r.is_empty() && g.is_empty() && b.is_empty() {
        return Err("provide at least one curve point list".to_string());
    }
    let adj = Adjustment::curves_per_channel([&c, &r, &g, &b]);
    apply_adjust(s, a, adj, "Curves")
}

fn h_adjust_channel_mixer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let mut rows = [[0i32; 4]; 3];
    rows[0] = [100, 0, 0, 0];
    rows[1] = [0, 100, 0, 0];
    rows[2] = [0, 0, 100, 0];
    if let Some(v) = a.get("rows").and_then(|x| x.as_array()) {
        if v.len() != 3 {
            return Err("'rows' must have exactly 3 rows".to_string());
        }
        for (i, row) in v.iter().enumerate() {
            let r = num_array(row, 4, "rows")?;
            for (j, n) in r.iter().enumerate() {
                if !(-200.0..=200.0).contains(n) {
                    return Err("'rows' values must be -200-200".to_string());
                }
                rows[i][j] = *n as i32;
            }
        }
    }
    let monochrome = get_bool(a, "monochrome", false)?;
    apply_adjust(
        s,
        a,
        Adjustment::ChannelMixer { rows, monochrome },
        "Channel Mixer",
    )
}

fn h_adjust_selective_color(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let mut colors = [[0i32; 4]; 9];
    if let Some(v) = a.get("colors").and_then(|x| x.as_array()) {
        if v.len() != 9 {
            return Err("'colors' must have exactly 9 entries (reds, yellows, greens, cyans, blues, magentas, whites, neutrals, blacks)".to_string());
        }
        for (i, row) in v.iter().enumerate() {
            let r = num_array(row, 4, "colors")?;
            for (j, n) in r.iter().enumerate() {
                if !(-100.0..=100.0).contains(n) {
                    return Err("'colors' values must be -100-100".to_string());
                }
                colors[i][j] = *n as i32;
            }
        }
    }
    let absolute = get_bool(a, "absolute", false)?;
    apply_adjust(
        s,
        a,
        Adjustment::SelectiveColor { colors, absolute },
        "Selective Color",
    )
}

// ---------------------------------------------------------------------------
// Filter handlers
// ---------------------------------------------------------------------------

fn apply_filter(s: &mut AppState, a: &Value, f: Filter, name: &str) -> Result<Value, String> {
    let bg = bg_color(a)?;
    let (id, m) = s.get_mut(a)?;
    op_core::filter::apply(&mut m.doc, f, bg).map_err(|e| e.message(name))?;
    m.history.record(&m.doc, name);
    Ok(ok_mut(&id, m, json!({"applied": name})))
}

fn h_filter_gaussian_blur(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = check_range("radius", get_f32(a, "radius", 1.0)?, 0.1, 1000.0)?;
    apply_filter(s, a, Filter::GaussianBlur { radius }, "Gaussian Blur")
}
fn h_filter_box_blur(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = check_range("radius", get_u32(a, "radius", 1)?, 1, 2000)?;
    apply_filter(s, a, Filter::BoxBlur { radius }, "Box Blur")
}
fn h_filter_average(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::Average, "Average")
}
fn h_filter_unsharp_mask(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let amount = check_range("amount", get_f32(a, "amount", 100.0)?, 1.0, 500.0)?;
    let radius = check_range("radius", get_f32(a, "radius", 1.0)?, 0.1, 1000.0)?;
    let threshold = get_u8(a, "threshold", 0)?;
    apply_filter(
        s,
        a,
        Filter::UnsharpMask {
            amount,
            radius,
            threshold,
        },
        "Unsharp Mask",
    )
}
fn h_filter_add_noise(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let amount = check_range("amount", get_f32(a, "amount", 10.0)?, 0.1, 400.0)?;
    let gaussian = get_bool(a, "gaussian", false)?;
    let monochromatic = get_bool(a, "monochromatic", false)?;
    apply_filter(
        s,
        a,
        Filter::AddNoise {
            amount,
            gaussian,
            monochromatic,
        },
        "Add Noise",
    )
}
fn h_filter_median(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = check_range("radius", get_u32(a, "radius", 1)?, 1, 500)?;
    apply_filter(s, a, Filter::Median { radius }, "Median")
}
fn h_filter_minimum(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = check_range("radius", get_f32(a, "radius", 1.0)?, 0.2, 500.0)?;
    let round = get_bool(a, "round", false)?;
    apply_filter(s, a, Filter::Minimum { radius, round }, "Minimum")
}
fn h_filter_maximum(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = check_range("radius", get_f32(a, "radius", 1.0)?, 0.2, 500.0)?;
    let round = get_bool(a, "round", false)?;
    apply_filter(s, a, Filter::Maximum { radius, round }, "Maximum")
}
fn h_filter_blur(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::Blur, "Blur")
}
fn h_filter_blur_more(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::BlurMore, "Blur More")
}
fn h_filter_sharpen(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::Sharpen, "Sharpen")
}
fn h_filter_sharpen_more(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::SharpenMore, "Sharpen More")
}
fn h_filter_sharpen_edges(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::SharpenEdges, "Sharpen Edges")
}
fn h_filter_despeckle(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::Despeckle, "Despeckle")
}
fn h_filter_find_edges(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::FindEdges, "Find Edges")
}
fn h_filter_motion_blur(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let angle = check_range("angle", get_i32(a, "angle", 0)?, -360, 360)?;
    let distance = check_range("distance", get_u32(a, "distance", 10)?, 1, 2000)?;
    apply_filter(s, a, Filter::MotionBlur { angle, distance }, "Motion Blur")
}
fn h_filter_emboss(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let angle = get_i32(a, "angle", 135)?;
    let height = get_u32(a, "height", 3)?.clamp(1, 100);
    let amount = get_u32(a, "amount", 100)?.clamp(1, 500);
    apply_filter(
        s,
        a,
        Filter::Emboss {
            angle,
            height,
            amount,
        },
        "Emboss",
    )
}
fn h_filter_fragment(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::Fragment, "Fragment")
}
fn h_filter_custom(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let k = a
        .get("kernel")
        .ok_or("missing required parameter 'kernel' (25 numbers)")?;
    let n = num_array(k, 25, "kernel")?;
    let mut kernel = [0i16; 25];
    for (i, v) in n.iter().enumerate() {
        if !(-999.0..=999.0).contains(v) {
            return Err("kernel values must be -999-999".to_string());
        }
        kernel[i] = *v as i16;
    }
    let scale = get_i32(a, "scale", 1)? as i16;
    let offset = get_i32(a, "offset", 0)? as i16;
    apply_filter(
        s,
        a,
        Filter::Custom {
            kernel,
            scale,
            offset,
        },
        "Custom",
    )
}
fn h_filter_surface_blur(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = get_u32(a, "radius", 5)?.clamp(1, 100);
    let threshold = get_u8(a, "threshold", 15)?.max(2);
    apply_filter(
        s,
        a,
        Filter::SurfaceBlur { radius, threshold },
        "Surface Blur",
    )
}
fn h_filter_dust_scratches(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = get_u32(a, "radius", 4)?.clamp(1, 500);
    let threshold = get_u8(a, "threshold", 0)?;
    apply_filter(
        s,
        a,
        Filter::DustAndScratches { radius, threshold },
        "Dust & Scratches",
    )
}
fn h_filter_trace_contour(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let level = get_u8(a, "level", 128)?;
    let upper = get_bool(a, "upper", true)?;
    apply_filter(s, a, Filter::TraceContour { level, upper }, "Trace Contour")
}
fn h_filter_wind(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let method = parse_wind_method(opt_str(a, "method").as_deref().unwrap_or("wind"))?;
    let from_left = get_bool(a, "from_left", true)?;
    apply_filter(s, a, Filter::Wind { method, from_left }, "Wind")
}
fn h_filter_twirl(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let angle = get_i32(a, "angle", 100)?;
    if !(-999..=999).contains(&angle) {
        return Err("angle must be -999-999".to_string());
    }
    apply_filter(s, a, Filter::Twirl { angle }, "Twirl")
}
fn h_filter_pinch(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let amount = get_i32(a, "amount", 50)?;
    if !(-100..=100).contains(&amount) {
        return Err("amount must be -100-100".to_string());
    }
    apply_filter(s, a, Filter::Pinch { amount }, "Pinch")
}
fn h_filter_spherize(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let amount = get_i32(a, "amount", 50)?;
    if !(-100..=100).contains(&amount) {
        return Err("amount must be -100-100".to_string());
    }
    let mode = parse_spherize_mode(opt_str(a, "mode").as_deref().unwrap_or("normal"))?;
    apply_filter(s, a, Filter::Spherize { amount, mode }, "Spherize")
}
fn h_filter_polar(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let to_polar = get_bool(a, "to_polar", true)?;
    apply_filter(
        s,
        a,
        Filter::PolarCoordinates { to_polar },
        "Polar Coordinates",
    )
}
fn h_filter_high_pass(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = check_range("radius", get_f32(a, "radius", 10.0)?, 0.1, 1000.0)?;
    apply_filter(s, a, Filter::HighPass { radius }, "High Pass")
}
fn h_filter_offset(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let dx = check_range("dx", get_i32(a, "dx", 0)?, -30_000, 30_000)?;
    let dy = check_range("dy", get_i32(a, "dy", 0)?, -30_000, 30_000)?;
    let fill = parse_offset_fill(opt_str(a, "fill").as_deref().unwrap_or("background"))?;
    apply_filter(s, a, Filter::Offset { dx, dy, fill }, "Offset")
}
fn h_filter_mosaic(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let cell = check_range("cell", get_u32(a, "cell", 8)?, 2, 200)?;
    apply_filter(s, a, Filter::Mosaic { cell }, "Mosaic")
}
fn h_filter_solarize(s: &mut AppState, a: &Value) -> Result<Value, String> {
    apply_filter(s, a, Filter::Solarize, "Solarize")
}

// ---------------------------------------------------------------------------
// Layer handlers
// ---------------------------------------------------------------------------

fn h_list_layers(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let id = s.resolve_id(a)?;
    let m = s
        .docs
        .get(&id)
        .ok_or_else(|| format!("unknown document '{id}'"))?;
    let layers: Vec<Value> = m.doc.layers.iter().map(|l| layer_json(&m.doc, l)).collect();
    Ok(json!({"doc": id, "layers": layers}))
}

fn h_set_active_layer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let lid = parse_layer_id(a, &m.doc)?;
    m.doc.select_layer(lid);
    Ok(json!({"doc": id, "active_layer": lid.0}))
}

fn h_new_raster_layer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let (w, h) = (m.doc.width, m.doc.height);
    let name = opt_str(a, "name").unwrap_or_else(|| m.doc.next_layer_name());
    let image = match opt_rgb(a, "fill")? {
        Some([r, g, b]) => TiledImage::filled(w, h, [r, g, b, 255]),
        None => TiledImage::new(w, h),
    };
    let lid = m.doc.new_layer_id();
    m.doc
        .insert_above_active(Layer::raster(lid, name.clone(), image));
    m.history.record(&m.doc, "New Layer");
    Ok(json!({"doc": id, "layer": lid.0, "name": name}))
}

fn h_new_group(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let name = opt_str(a, "name").unwrap_or_else(|| m.doc.next_group_name());
    let lid = m.doc.new_layer_id();
    m.doc.insert_above_active(Layer::group(lid, name.clone()));
    m.history.record(&m.doc, "New Group");
    Ok(json!({"doc": id, "layer": lid.0, "name": name}))
}

fn h_duplicate_layer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    match op_core::layer_ops::duplicate(&mut m.doc) {
        Some(lid) => {
            m.history.record(&m.doc, "Duplicate Layer");
            Ok(json!({"doc": id, "layer": lid.0}))
        }
        None => Err("no active layer to duplicate".to_string()),
    }
}

fn h_delete_selected_layers(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !op_core::layer_ops::delete_selected(&mut m.doc) {
        return Err("nothing to delete".to_string());
    }
    m.history.record(&m.doc, "Delete Layers");
    Ok(ok_mut(&id, m, json!({"deleted": true})))
}

fn h_rename_layer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let lid = parse_layer_id(a, &m.doc)?;
    let name = req_str(a, "name")?;
    if !op_core::layer_ops::rename(&mut m.doc, lid, &name) {
        return Err("rename failed".to_string());
    }
    m.history.record(&m.doc, "Rename Layer");
    Ok(json!({"doc": id, "layer": lid.0, "name": name}))
}

fn h_set_layer_visibility(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let lid = parse_layer_id(a, &m.doc)?;
    let visible = get_bool(a, "visible", true)?;
    let layer = m.doc.layer_mut(lid).ok_or("unknown layer".to_string())?;
    layer.visible = visible;
    m.doc.mark_dirty();
    m.history
        .record(&m.doc, if visible { "Show Layer" } else { "Hide Layer" });
    Ok(json!({"doc": id, "layer": lid.0, "visible": visible}))
}

fn h_set_layer_opacity(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let lid = parse_layer_id(a, &m.doc)?;
    let opacity = get_f64(a, "opacity", 1.0)?;
    if !(0.0..=1.0).contains(&opacity) {
        return Err("opacity must be 0-1".to_string());
    }
    let layer = m.doc.layer_mut(lid).ok_or("unknown layer".to_string())?;
    layer.opacity = opacity as f32;
    m.doc.mark_dirty();
    m.history.record(&m.doc, "Layer Opacity");
    Ok(json!({"doc": id, "layer": lid.0, "opacity": opacity}))
}

fn h_set_layer_fill(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let lid = parse_layer_id(a, &m.doc)?;
    let fill = get_f64(a, "fill", 1.0)?;
    if !(0.0..=1.0).contains(&fill) {
        return Err("fill must be 0-1".to_string());
    }
    let layer = m.doc.layer_mut(lid).ok_or("unknown layer".to_string())?;
    layer.fill = fill as f32;
    m.doc.mark_dirty();
    m.history.record(&m.doc, "Layer Fill");
    Ok(json!({"doc": id, "layer": lid.0, "fill": fill}))
}

fn h_set_layer_blend_mode(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let lid = parse_layer_id(a, &m.doc)?;
    let mode = parse_blend_mode(&req_str(a, "mode")?)?;
    let layer = m.doc.layer_mut(lid).ok_or("unknown layer".to_string())?;
    layer.blend_mode = mode;
    m.doc.mark_dirty();
    m.history.record(&m.doc, "Blend Mode");
    Ok(json!({"doc": id, "layer": lid.0, "blend_mode": blend_label(mode)}))
}

fn h_arrange_layer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let arrange = parse_arrange(&req_str(a, "arrange")?)?;
    let (id, m) = s.get_mut(a)?;
    let moved = op_core::layer_ops::arrange(&mut m.doc, arrange);
    if moved {
        m.history.record(&m.doc, "Arrange");
    }
    Ok(json!({"doc": id, "moved": moved}))
}

fn h_merge_down(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !op_core::layer_ops::merge_down(&mut m.doc) {
        return Err("cannot merge down".to_string());
    }
    m.history.record(&m.doc, "Merge Down");
    Ok(ok_mut(&id, m, json!({"merged": true})))
}

fn h_merge_visible(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !op_core::layer_ops::merge_visible(&mut m.doc) {
        return Err("cannot merge visible".to_string());
    }
    m.history.record(&m.doc, "Merge Visible");
    Ok(ok_mut(&id, m, json!({"merged": true})))
}

fn h_flatten_image(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    op_core::layer_ops::flatten(&mut m.doc);
    m.history.record(&m.doc, "Flatten Image");
    Ok(ok_mut(&id, m, json!({"flattened": true})))
}

fn h_group_selected(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    match op_core::layer_ops::group_selected(&mut m.doc) {
        Some(gid) => {
            m.history.record(&m.doc, "Group Layers");
            Ok(json!({"doc": id, "group": gid.0}))
        }
        None => Err("cannot group the selected layers".to_string()),
    }
}

fn h_ungroup(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !op_core::layer_ops::ungroup(&mut m.doc) {
        return Err("active layer is not a group".to_string());
    }
    m.history.record(&m.doc, "Ungroup");
    Ok(ok_mut(&id, m, json!({"ungrouped": true})))
}

fn h_add_layer_mask(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let kind = parse_new_mask(opt_str(a, "kind").as_deref().unwrap_or("reveal_all"))?;
    let (id, m) = s.get_mut(a)?;
    if !op_core::layer_ops::add_mask(&mut m.doc, kind) {
        return Err("cannot add a mask here".to_string());
    }
    m.history.record(&m.doc, "Add Mask");
    Ok(json!({"doc": id, "mask_added": true}))
}

fn h_convert_background(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !op_core::layer_ops::layer_from_background(&mut m.doc) {
        return Err("no background layer to convert".to_string());
    }
    m.history.record(&m.doc, "Layer From Background");
    Ok(ok_mut(&id, m, json!({"converted": true})))
}

fn h_align_layers(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let how = parse_align(&req_str(a, "how")?)?;
    let (id, m) = s.get_mut(a)?;
    if !op_core::align::align(&mut m.doc, how) {
        return Err("align did nothing (select two or more layers)".to_string());
    }
    m.history.record(&m.doc, how.name());
    Ok(json!({"doc": id, "aligned": how.label()}))
}

fn h_distribute_layers(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let how = parse_distribute(&req_str(a, "how")?)?;
    let (id, m) = s.get_mut(a)?;
    if !op_core::align::distribute(&mut m.doc, how) {
        return Err("distribute did nothing (select three or more layers)".to_string());
    }
    m.history.record(&m.doc, "Distribute");
    Ok(json!({"doc": id, "distributed": true}))
}

fn h_layer_via_copy(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    match op_core::layer_ops::via_copy(&mut m.doc) {
        Ok(lid) => {
            m.history.record(&m.doc, "Layer Via Copy");
            Ok(json!({"doc": id, "layer": lid.0}))
        }
        Err(e) => Err(e.message("Layer Via Copy")),
    }
}

fn h_layer_via_cut(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let bg = bg_color(a)?;
    let (id, m) = s.get_mut(a)?;
    match op_core::layer_ops::via_cut(&mut m.doc, bg) {
        Ok(lid) => {
            m.history.record(&m.doc, "Layer Via Cut");
            Ok(json!({"doc": id, "layer": lid.0}))
        }
        Err(e) => Err(e.message("Layer Via Cut")),
    }
}

// ---------------------------------------------------------------------------
// Selection handlers: each change is a history state, named as the app
// names it (Photoshop's History panel lists selections too)
// ---------------------------------------------------------------------------

fn h_select_all(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    let (w, h) = (m.doc.width, m.doc.height);
    m.doc.set_selection(Some(Selection::all(w, h)));
    m.history.record(&m.doc, "Select All");
    Ok(json!({"doc": id, "selection": "all"}))
}

fn h_clear_selection(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if m.doc.selection().is_none() {
        return Ok(json!({"doc": id, "selection": "none"}));
    }
    m.doc.set_selection(None);
    m.history.record(&m.doc, "Deselect");
    Ok(json!({"doc": id, "selection": "none"}))
}

fn h_reselect(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !m.doc.reselect() {
        return Err("nothing to reselect".to_string());
    }
    m.history.record(&m.doc, "Reselect");
    Ok(json!({"doc": id, "reselected": true}))
}

fn h_select_rect(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let x0 = get_f64(a, "x0", 0.0)?;
    let y0 = get_f64(a, "y0", 0.0)?;
    let x1 = get_f64(a, "x1", 0.0)?;
    let y1 = get_f64(a, "y1", 0.0)?;
    let (id, m) = s.get_mut(a)?;
    let (w, h) = (m.doc.width, m.doc.height);
    let rect = op_core::selection::Rect::new(x0 as f32, y0 as f32, x1 as f32, y1 as f32);
    m.doc.set_selection(Some(Selection::rect(w, h, rect)));
    m.history.record(&m.doc, "Rectangular Marquee");
    Ok(json!({"doc": id, "selection": "rect"}))
}

fn h_select_ellipse(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let x0 = get_f64(a, "x0", 0.0)?;
    let y0 = get_f64(a, "y0", 0.0)?;
    let x1 = get_f64(a, "x1", 0.0)?;
    let y1 = get_f64(a, "y1", 0.0)?;
    let aa = get_bool(a, "anti_alias", true)?;
    let (id, m) = s.get_mut(a)?;
    let (w, h) = (m.doc.width, m.doc.height);
    let rect = op_core::selection::Rect::new(x0 as f32, y0 as f32, x1 as f32, y1 as f32);
    m.doc
        .set_selection(Some(Selection::ellipse(w, h, rect, aa)));
    m.history.record(&m.doc, "Elliptical Marquee");
    Ok(json!({"doc": id, "selection": "ellipse"}))
}

fn h_feather_selection(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let radius = check_range("radius", get_f32(a, "radius", 1.0)?, 0.1, 1000.0)?;
    let (id, m) = s.get_mut(a)?;
    let cur = m
        .doc
        .selection()
        .cloned()
        .ok_or("no selection".to_string())?;
    m.doc.set_selection(Some(cur.feather(radius)));
    m.history.record(&m.doc, "Feather");
    Ok(json!({"doc": id, "feathered": radius}))
}

// ---------------------------------------------------------------------------
// Canvas handlers
// ---------------------------------------------------------------------------

fn h_rotate_flip(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let o = parse_orientation(&req_str(a, "orientation")?)?;
    let name = o.history_name().to_string();
    let (id, m) = s.get_mut(a)?;
    op_core::image_ops::reorient(&mut m.doc, o);
    m.history.record(&m.doc, name.clone());
    Ok(ok_mut(&id, m, json!({"applied": name})))
}

fn h_rotate_arbitrary(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let degrees = get_f32(a, "degrees", 0.0)?;
    let bg = bg_as_color(a)?;
    let (id, m) = s.get_mut(a)?;
    if !op_core::image_ops::rotate_arbitrary(&mut m.doc, degrees, bg) {
        return Err("rotation is a multiple of 360 degrees".to_string());
    }
    m.history.record(&m.doc, "Rotate Canvas");
    Ok(ok_mut(&id, m, json!({"applied": "Rotate Canvas"})))
}

fn h_resize_canvas(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let width = req_u32(a, "width")?;
    let height = req_u32(a, "height")?;
    check_size(width, height)?;
    let ax = get_u32(a, "anchor_x", 1)?;
    let ay = get_u32(a, "anchor_y", 1)?;
    if ax > 2 || ay > 2 {
        return Err("anchor_x and anchor_y must be 0, 1 or 2".to_string());
    }
    let fill = bg_as_color(a)?;
    let (id, m) = s.get_mut(a)?;
    m.doc.resize_canvas(
        width,
        height,
        Anchor {
            x: ax as u8,
            y: ay as u8,
        },
        fill,
    );
    m.history.record(&m.doc, "Canvas Size");
    Ok(ok_mut(&id, m, json!({"applied": "Canvas Size"})))
}

fn h_crop(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let x0 = req_u32(a, "x0")?;
    let y0 = req_u32(a, "y0")?;
    let x1 = req_u32(a, "x1")?;
    let y1 = req_u32(a, "y1")?;
    let (id, m) = s.get_mut(a)?;
    if !(x0 < x1 && y0 < y1 && x1 <= m.doc.width && y1 <= m.doc.height) {
        return Err("crop box must satisfy x0<x1<=width and y0<y1<=height".to_string());
    }
    op_core::image_ops::crop(&mut m.doc, x0, y0, x1, y1);
    m.history.record(&m.doc, "Crop");
    Ok(ok_mut(&id, m, json!({"applied": "Crop"})))
}

fn h_crop_to_selection(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !op_core::image_ops::crop_to_selection(&mut m.doc) {
        return Err("no selection to crop to".to_string());
    }
    m.history.record(&m.doc, "Crop");
    Ok(ok_mut(&id, m, json!({"applied": "Crop"})))
}

fn h_resize_image(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let width = req_u32(a, "width")?;
    let height = req_u32(a, "height")?;
    check_size(width, height)?;
    let method = parse_resample(opt_str(a, "method").as_deref().unwrap_or("automatic"))?;
    let (id, m) = s.get_mut(a)?;
    op_core::image_ops::resize(&mut m.doc, width, height, method);
    m.history.record(&m.doc, "Image Size");
    Ok(ok_mut(&id, m, json!({"applied": "Image Size"})))
}

fn h_trim(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let basis = parse_trim_basis(opt_str(a, "basis").as_deref().unwrap_or("transparent"))?;
    let sides = op_core::image_ops::TrimSides {
        top: get_bool(a, "top", true)?,
        left: get_bool(a, "left", true)?,
        bottom: get_bool(a, "bottom", true)?,
        right: get_bool(a, "right", true)?,
    };
    let (id, m) = s.get_mut(a)?;
    if !op_core::image_ops::trim(&mut m.doc, basis, sides) {
        return Ok(ok_mut(&id, m, json!({"trimmed": false})));
    }
    m.history.record(&m.doc, "Trim");
    Ok(ok_mut(&id, m, json!({"trimmed": true})))
}

fn h_reveal_all(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let bg = bg_as_color(a)?;
    let (id, m) = s.get_mut(a)?;
    if !op_core::image_ops::reveal_all(&mut m.doc, bg) {
        return Ok(ok_mut(&id, m, json!({"revealed": false})));
    }
    m.history.record(&m.doc, "Reveal All");
    Ok(ok_mut(&id, m, json!({"revealed": true})))
}

// ---------------------------------------------------------------------------
// Edit handlers
// ---------------------------------------------------------------------------

fn h_fill_solid(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let color = req_rgb(a)?;
    let opacity = get_f64(a, "opacity", 1.0)?;
    if !(0.0..=1.0).contains(&opacity) {
        return Err("opacity must be 0-1".to_string());
    }
    let mode = parse_blend_mode(opt_str(a, "mode").as_deref().unwrap_or("normal"))?;
    let preserve = get_bool(a, "preserve_transparency", false)?;
    let opts = op_core::fill::FillOptions {
        mode,
        opacity: opacity as f32,
        preserve_transparency: preserve,
    };
    let (id, m) = s.get_mut(a)?;
    op_core::fill::fill(&mut m.doc, color, opts).map_err(|e| e.message("Fill"))?;
    m.history.record(&m.doc, "Fill");
    Ok(ok_mut(&id, m, json!({"applied": "Fill"})))
}

fn h_clear_layer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let bg = bg_color(a)?;
    let (id, m) = s.get_mut(a)?;
    op_core::fill::clear(&mut m.doc, bg).map_err(|e| e.message("Clear"))?;
    m.history.record(&m.doc, "Clear");
    Ok(ok_mut(&id, m, json!({"applied": "Clear"})))
}

fn h_paint_bucket(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let x = req_u32(a, "x")?;
    let y = req_u32(a, "y")?;
    let color = req_rgb(a)?;
    let tolerance = get_u8(a, "tolerance", 32)?;
    let anti_alias = get_bool(a, "anti_alias", true)?;
    let contiguous = get_bool(a, "contiguous", true)?;
    let all_layers = get_bool(a, "all_layers", false)?;
    let opts = op_core::fill::BucketOptions {
        fill: op_core::fill::FillOptions::default(),
        tolerance,
        anti_alias,
        contiguous,
        all_layers,
    };
    let (id, m) = s.get_mut(a)?;
    match op_core::fill::bucket(&mut m.doc, x, y, color, opts) {
        Ok(false) => Err("click is outside the document".to_string()),
        Ok(true) => {
            m.history.record(&m.doc, "Paint Bucket");
            Ok(ok_mut(&id, m, json!({"applied": "Paint Bucket"})))
        }
        Err(e) => Err(e.message("Paint Bucket")),
    }
}

fn h_gradient_fill(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let x0 = get_f64(a, "x0", 0.0)? as f32;
    let y0 = get_f64(a, "y0", 0.0)? as f32;
    let x1 = get_f64(a, "x1", 100.0)? as f32;
    let y1 = get_f64(a, "y1", 100.0)? as f32;
    let from = req_rgb_key(a, "from")?;
    let to = req_rgb_key(a, "to")?;
    let kind = parse_gradient_kind(opt_str(a, "kind").as_deref().unwrap_or("linear"))?;
    // The gradient tool has no interpolation method (only Gradient Map
    // does), so asking for one is an error rather than silently ignored
    if a.get("method").is_some() {
        return Err(
            "gradient_fill has no 'method'; it is an option of adjust_gradient_map".to_string(),
        );
    }
    let opacity = get_f64(a, "opacity", 1.0)?;
    if !(0.0..=1.0).contains(&opacity) {
        return Err("opacity must be 0-1".to_string());
    }
    let reverse = get_bool(a, "reverse", false)?;
    let opts = GradientOptions {
        kind,
        mode: BlendMode::Normal,
        opacity: opacity as f32,
        reverse,
    };
    let (id, m) = s.get_mut(a)?;
    op_core::gradient::gradient(&mut m.doc, (x0, y0), (x1, y1), (from, to), opts)
        .map_err(|e| e.message("Gradient"))?;
    m.history.record(&m.doc, "Gradient");
    Ok(ok_mut(&id, m, json!({"applied": "Gradient"})))
}

fn h_move_active_layer(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let dx = get_i32(a, "dx", 0)? as i64;
    let dy = get_i32(a, "dy", 0)? as i64;
    let bg = bg_color(a)?;
    let (id, m) = s.get_mut(a)?;
    let mv = op_core::move_tool::Move::begin(&m.doc, bg).map_err(|e| e.message().to_string())?;
    mv.apply(&mut m.doc, dx, dy);
    m.history.record(&m.doc, "Move");
    Ok(ok_mut(&id, m, json!({"moved": [dx, dy]})))
}

// ---------------------------------------------------------------------------
// History handlers
// ---------------------------------------------------------------------------

fn h_undo(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !m.history.undo(&mut m.doc) {
        return Err("nothing to undo".to_string());
    }
    Ok(ok_mut(&id, m, json!({"undone": true})))
}

fn h_redo(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let (id, m) = s.get_mut(a)?;
    if !m.history.redo(&mut m.doc) {
        return Err("nothing to redo".to_string());
    }
    Ok(ok_mut(&id, m, json!({"redone": true})))
}

fn h_list_history(s: &mut AppState, a: &Value) -> Result<Value, String> {
    let id = s.resolve_id(a)?;
    let m = s
        .docs
        .get(&id)
        .ok_or_else(|| format!("unknown document '{id}'"))?;
    let cur = m.history.current();
    let states: Vec<Value> = m
        .history
        .states()
        .iter()
        .enumerate()
        .map(|(i, st)| json!({"index": i, "name": st.name, "current": i == cur}))
        .collect();
    Ok(json!({"doc": id, "states": states}))
}

// ---------------------------------------------------------------------------
// Internal tool registry + keyword search
// ---------------------------------------------------------------------------

type Handler = fn(&mut AppState, &Value) -> Result<Value, String>;

struct ToolDef {
    name: &'static str,
    description: &'static str,
    keywords: &'static [&'static str],
    schema: Value,
    run: Handler,
}

fn doc_param() -> Value {
    json!({"type": "string", "description": "Document id (defaults to the active document)"})
}

fn schema(props: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": props,
        "required": required,
        "additionalProperties": true
    })
}

fn all_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "new_document",
            description: "File > New: create a blank document with a background layer.",
            keywords: &[
                "new",
                "create",
                "blank",
                "document",
                "file",
                "canvas",
                "background",
            ],
            schema: schema(
                json!({"title": {"type": "string"}, "width": {"type": "integer", "minimum": 1, "maximum": 30000}, "height": {"type": "integer", "minimum": 1, "maximum": 30000}, "fill": {"type": "array", "items": {"type": "integer"}, "minItems": 3, "maxItems": 3}, "doc": doc_param()}),
                &["width", "height"],
            ),
            run: h_new_document,
        },
        ToolDef {
            name: "open_image",
            description: "File > Open: open a PSD, PNG, JPEG, WebP, TIFF, BMP or GIF file.",
            keywords: &[
                "open", "load", "read", "file", "psd", "png", "jpeg", "import",
            ],
            schema: schema(
                json!({"path": {"type": "string", "description": "Filesystem path to open"}}),
                &["path"],
            ),
            run: h_open_image,
        },
        ToolDef {
            name: "save_image",
            description: "File > Save: save the document (.psd keeps layers, other extensions save the composite).",
            keywords: &["save", "write", "file", "psd", "store"],
            schema: schema(
                json!({"doc": doc_param(), "path": {"type": "string"}}),
                &["path"],
            ),
            run: h_save_image,
        },
        ToolDef {
            name: "export_composite",
            description: "Export the flattened composite to PNG/JPEG/etc. chosen by file extension.",
            keywords: &[
                "export",
                "composite",
                "flatten",
                "png",
                "jpeg",
                "render",
                "output",
            ],
            schema: schema(
                json!({"doc": doc_param(), "path": {"type": "string"}}),
                &["path"],
            ),
            run: h_export_composite,
        },
        ToolDef {
            name: "list_documents",
            description: "List all open documents and which one is active.",
            keywords: &["list", "documents", "open", "active"],
            schema: schema(json!({}), &[]),
            run: h_list_documents,
        },
        ToolDef {
            name: "get_document_info",
            description: "Document details: size, layers with ids/names/opacity/blend modes, selection state.",
            keywords: &["info", "document", "details", "layers", "size", "inspect"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_get_document_info,
        },
        ToolDef {
            name: "close_document",
            description: "Close a document and drop it from memory.",
            keywords: &["close", "document", "remove", "drop"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_close_document,
        },
        ToolDef {
            name: "set_active_document",
            description: "Make a document the active one used when 'doc' is omitted.",
            keywords: &["active", "document", "switch", "select"],
            schema: schema(json!({"doc": {"type": "string"}}), &["doc"]),
            run: h_set_active_document,
        },
        ToolDef {
            name: "get_pixel",
            description: "Read one pixel (composite or a single layer) as r/g/b/a.",
            keywords: &["pixel", "color", "sample", "eyedropper", "probe", "rgba"],
            schema: schema(
                json!({"doc": doc_param(), "x": {"type": "integer"}, "y": {"type": "integer"}, "composite": {"type": "boolean", "default": true}, "layer": {"type": "integer"}}),
                &["x", "y"],
            ),
            run: h_get_pixel,
        },
        ToolDef {
            name: "get_histogram",
            description: "RGB channel histograms of the active layer (Levels/Curves data).",
            keywords: &["histogram", "levels", "curves", "channels", "statistics"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_get_histogram,
        },
        // Adjustments
        ToolDef {
            name: "adjust_invert",
            description: "Image > Adjustments > Invert: flip every channel.",
            keywords: &["invert", "negative", "adjustment"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_adjust_invert,
        },
        ToolDef {
            name: "adjust_desaturate",
            description: "Image > Adjustments > Desaturate: grayscale by HSL lightness.",
            keywords: &[
                "desaturate",
                "grayscale",
                "black",
                "white",
                "gray",
                "adjustment",
            ],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_adjust_desaturate,
        },
        ToolDef {
            name: "adjust_threshold",
            description: "Threshold: pixels at/above level become white, rest black (1-255).",
            keywords: &["threshold", "black", "white", "posterize", "adjustment"],
            schema: schema(
                json!({"doc": doc_param(), "level": {"type": "integer", "minimum": 1, "maximum": 255, "default": 128}}),
                &[],
            ),
            run: h_adjust_threshold,
        },
        ToolDef {
            name: "adjust_posterize",
            description: "Posterize: reduce tonal levels per channel (2-255).",
            keywords: &["posterize", "levels", "tones", "adjustment"],
            schema: schema(
                json!({"doc": doc_param(), "levels": {"type": "integer", "minimum": 2, "maximum": 255, "default": 4}}),
                &[],
            ),
            run: h_adjust_posterize,
        },
        ToolDef {
            name: "adjust_equalize",
            description: "Equalize: spread brightness values over the selected area.",
            keywords: &["equalize", "histogram", "contrast", "adjustment"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_adjust_equalize,
        },
        ToolDef {
            name: "adjust_equalize_entire_image",
            description: "Equalize entire image from the selection histogram.",
            keywords: &["equalize", "histogram", "entire", "adjustment"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_adjust_equalize_entire,
        },
        ToolDef {
            name: "adjust_levels",
            description: "Levels: per-channel input black/white, gamma, output black/white.",
            keywords: &[
                "levels",
                "gamma",
                "tones",
                "black",
                "white",
                "contrast",
                "adjustment",
            ],
            schema: schema(
                json!({"doc": doc_param(), "input_black": {"type": "integer"}, "input_white": {"type": "integer"}, "gamma": {"type": "number"}, "output_black": {"type": "integer"}, "output_white": {"type": "integer"}, "red": {"type": "object"}, "green": {"type": "object"}, "blue": {"type": "object"}}),
                &[],
            ),
            run: h_adjust_levels,
        },
        ToolDef {
            name: "adjust_hue_saturation",
            description: "Hue/Saturation with master hue/saturation/lightness, optional six color ranges and colorize.",
            keywords: &[
                "hue",
                "saturation",
                "colorize",
                "vibrance",
                "color",
                "adjustment",
            ],
            schema: schema(
                json!({"doc": doc_param(), "hue": {"type": "integer"}, "saturation": {"type": "integer"}, "lightness": {"type": "integer"}, "colorize": {"type": "boolean"}, "ranges": {"type": "array"}}),
                &[],
            ),
            run: h_adjust_hue_saturation,
        },
        ToolDef {
            name: "adjust_exposure",
            description: "Exposure in stops, offset and gamma in linear light.",
            keywords: &[
                "exposure",
                "offset",
                "gamma",
                "brightness",
                "stops",
                "adjustment",
            ],
            schema: schema(
                json!({"doc": doc_param(), "exposure": {"type": "number"}, "offset": {"type": "number"}, "gamma": {"type": "number"}}),
                &[],
            ),
            run: h_adjust_exposure,
        },
        ToolDef {
            name: "adjust_brightness_contrast",
            description: "Brightness/Contrast with optional legacy mode.",
            keywords: &["brightness", "contrast", "legacy", "adjustment"],
            schema: schema(
                json!({"doc": doc_param(), "brightness": {"type": "integer"}, "contrast": {"type": "integer"}, "legacy": {"type": "boolean"}}),
                &[],
            ),
            run: h_adjust_brightness_contrast,
        },
        ToolDef {
            name: "adjust_color_balance",
            description: "Color Balance: cyan-red, magenta-green, yellow-blue shifts for shadows/midtones/highlights.",
            keywords: &[
                "color",
                "balance",
                "cyan",
                "magenta",
                "yellow",
                "shadows",
                "highlights",
                "adjustment",
            ],
            schema: schema(
                json!({"doc": doc_param(), "shadows": {"type": "array"}, "midtones": {"type": "array"}, "highlights": {"type": "array"}, "preserve_luminosity": {"type": "boolean"}}),
                &[],
            ),
            run: h_adjust_color_balance,
        },
        ToolDef {
            name: "adjust_black_white",
            description: "Black & White conversion with per-hue weights and optional tint.",
            keywords: &[
                "black",
                "white",
                "grayscale",
                "tint",
                "monochrome",
                "adjustment",
            ],
            schema: schema(
                json!({"doc": doc_param(), "weights": {"type": "array"}, "tint": {"type": "array"}}),
                &[],
            ),
            run: h_adjust_black_white,
        },
        ToolDef {
            name: "adjust_vibrance",
            description: "Vibrance (muted colors first) and saturation.",
            keywords: &["vibrance", "saturation", "color", "adjustment"],
            schema: schema(
                json!({"doc": doc_param(), "vibrance": {"type": "integer"}, "saturation": {"type": "integer"}}),
                &[],
            ),
            run: h_adjust_vibrance,
        },
        ToolDef {
            name: "adjust_photo_filter",
            description: "Photo Filter: multiply a color at a density percent.",
            keywords: &[
                "photo",
                "filter",
                "warming",
                "cooling",
                "color",
                "adjustment",
            ],
            schema: schema(
                json!({"doc": doc_param(), "color": {"type": "array"}, "density": {"type": "integer"}, "preserve_luminosity": {"type": "boolean"}}),
                &[],
            ),
            run: h_adjust_photo_filter,
        },
        ToolDef {
            name: "adjust_gradient_map",
            description: "Gradient Map: luminosity mapped from one color to another.",
            keywords: &["gradient", "map", "duotone", "luminosity", "adjustment"],
            schema: schema(
                json!({"doc": doc_param(), "from": {"type": "array"}, "to": {"type": "array"}, "method": {"type": "string"}}),
                &["from", "to"],
            ),
            run: h_adjust_gradient_map,
        },
        ToolDef {
            name: "adjust_auto_tone",
            description: "Auto Tone: stretch each channel ignoring extreme 0.1%.",
            keywords: &["auto", "tone", "contrast", "adjustment"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_adjust_auto_tone,
        },
        ToolDef {
            name: "adjust_auto_contrast",
            description: "Auto Contrast: stretch all channels together.",
            keywords: &["auto", "contrast", "adjustment"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_adjust_auto_contrast,
        },
        ToolDef {
            name: "adjust_auto_color",
            description: "Auto Color: per-channel stretch like Auto Tone.",
            keywords: &["auto", "color", "adjustment"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_adjust_auto_color,
        },
        ToolDef {
            name: "adjust_curves",
            description: "Curves: up to 16 (in,out) points for composite and R/G/B channels.",
            keywords: &["curves", "spline", "tones", "contrast", "adjustment"],
            schema: schema(
                json!({"doc": doc_param(), "points": {"type": "array"}, "red": {"type": "array"}, "green": {"type": "array"}, "blue": {"type": "array"}}),
                &[],
            ),
            run: h_adjust_curves,
        },
        ToolDef {
            name: "adjust_channel_mixer",
            description: "Channel Mixer: each output channel from R/G/B percents plus constant; optional monochrome.",
            keywords: &["channel", "mixer", "monochrome", "rgb", "adjustment"],
            schema: schema(
                json!({"doc": doc_param(), "rows": {"type": "array"}, "monochrome": {"type": "boolean"}}),
                &[],
            ),
            run: h_adjust_channel_mixer,
        },
        ToolDef {
            name: "adjust_selective_color",
            description: "Selective Color: CMYK sliders for 9 color ranges, relative or absolute.",
            keywords: &[
                "selective",
                "color",
                "cmyk",
                "reds",
                "yellows",
                "neutrals",
                "adjustment",
            ],
            schema: schema(
                json!({"doc": doc_param(), "colors": {"type": "array"}, "absolute": {"type": "boolean"}}),
                &[],
            ),
            run: h_adjust_selective_color,
        },
        // Filters
        ToolDef {
            name: "filter_gaussian_blur",
            description: "Gaussian Blur with radius in pixels.",
            keywords: &["blur", "gaussian", "soften", "filter", "smooth"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "number", "default": 1.0}}),
                &[],
            ),
            run: h_filter_gaussian_blur,
        },
        ToolDef {
            name: "filter_box_blur",
            description: "Box Blur: average of a square.",
            keywords: &["blur", "box", "average", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "integer", "default": 1}}),
                &[],
            ),
            run: h_filter_box_blur,
        },
        ToolDef {
            name: "filter_average",
            description: "Average: fill the selection with its mean color.",
            keywords: &["average", "blur", "mean", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_average,
        },
        ToolDef {
            name: "filter_unsharp_mask",
            description: "Unsharp Mask sharpening: amount percent, radius, threshold.",
            keywords: &["sharpen", "unsharp", "mask", "clarity", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "amount": {"type": "number"}, "radius": {"type": "number"}, "threshold": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_unsharp_mask,
        },
        ToolDef {
            name: "filter_add_noise",
            description: "Add Noise: uniform or gaussian, optionally monochromatic.",
            keywords: &["noise", "grain", "add", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "amount": {"type": "number"}, "gaussian": {"type": "boolean"}, "monochromatic": {"type": "boolean"}}),
                &[],
            ),
            run: h_filter_add_noise,
        },
        ToolDef {
            name: "filter_median",
            description: "Median noise reduction over a square.",
            keywords: &["median", "noise", "dust", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_median,
        },
        ToolDef {
            name: "filter_minimum",
            description: "Minimum: shrink bright areas (square or round).",
            keywords: &["minimum", "erode", "shrink", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "number"}, "round": {"type": "boolean"}}),
                &[],
            ),
            run: h_filter_minimum,
        },
        ToolDef {
            name: "filter_maximum",
            description: "Maximum: spread bright areas (square or round).",
            keywords: &["maximum", "dilate", "spread", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "number"}, "round": {"type": "boolean"}}),
                &[],
            ),
            run: h_filter_maximum,
        },
        ToolDef {
            name: "filter_blur",
            description: "Blur: light 3x3 blur.",
            keywords: &["blur", "soften", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_blur,
        },
        ToolDef {
            name: "filter_blur_more",
            description: "Blur More: stronger 3x3 blur.",
            keywords: &["blur", "more", "soften", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_blur_more,
        },
        ToolDef {
            name: "filter_sharpen",
            description: "Sharpen.",
            keywords: &["sharpen", "crisp", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_sharpen,
        },
        ToolDef {
            name: "filter_sharpen_more",
            description: "Sharpen More.",
            keywords: &["sharpen", "more", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_sharpen_more,
        },
        ToolDef {
            name: "filter_sharpen_edges",
            description: "Sharpen Edges: sharpen only where edges are.",
            keywords: &["sharpen", "edges", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_sharpen_edges,
        },
        ToolDef {
            name: "filter_despeckle",
            description: "Despeckle: blur held back at edges.",
            keywords: &["despeckle", "noise", "smooth", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_despeckle,
        },
        ToolDef {
            name: "filter_find_edges",
            description: "Find Edges: white where flat, darker on edges.",
            keywords: &["edges", "find", "sketch", "filter", "stylize"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_find_edges,
        },
        ToolDef {
            name: "filter_motion_blur",
            description: "Motion Blur along an angle.",
            keywords: &["motion", "blur", "speed", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "angle": {"type": "integer"}, "distance": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_motion_blur,
        },
        ToolDef {
            name: "filter_emboss",
            description: "Emboss with angle, height and amount.",
            keywords: &["emboss", "relief", "stylize", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "angle": {"type": "integer"}, "height": {"type": "integer"}, "amount": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_emboss,
        },
        ToolDef {
            name: "filter_fragment",
            description: "Fragment: four offset copies averaged.",
            keywords: &["fragment", "pixelate", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_fragment,
        },
        ToolDef {
            name: "filter_custom",
            description: "Custom 5x5 kernel with scale and offset.",
            keywords: &["custom", "kernel", "convolution", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "kernel": {"type": "array"}, "scale": {"type": "integer"}, "offset": {"type": "integer"}}),
                &["kernel"],
            ),
            run: h_filter_custom,
        },
        ToolDef {
            name: "filter_surface_blur",
            description: "Surface Blur preserving edges (radius, threshold).",
            keywords: &["surface", "blur", "smooth", "skin", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "integer"}, "threshold": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_surface_blur,
        },
        ToolDef {
            name: "filter_dust_scratches",
            description: "Dust & Scratches: median where pixels differ past a threshold.",
            keywords: &["dust", "scratches", "retouch", "median", "filter", "noise"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "integer"}, "threshold": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_dust_scratches,
        },
        ToolDef {
            name: "filter_trace_contour",
            description: "Trace Contour: black lines where values cross a level.",
            keywords: &["trace", "contour", "edges", "stylize", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "level": {"type": "integer"}, "upper": {"type": "boolean"}}),
                &[],
            ),
            run: h_filter_trace_contour,
        },
        ToolDef {
            name: "filter_wind",
            description: "Wind: streaks blown left or right (wind, blast, stagger).",
            keywords: &["wind", "blast", "streak", "motion", "stylize", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "method": {"type": "string"}, "from_left": {"type": "boolean"}}),
                &[],
            ),
            run: h_filter_wind,
        },
        ToolDef {
            name: "filter_twirl",
            description: "Twirl distortion fading from the center.",
            keywords: &["twirl", "swirl", "distort", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "angle": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_twirl,
        },
        ToolDef {
            name: "filter_pinch",
            description: "Pinch in or out (-100 to 100).",
            keywords: &["pinch", "bulge", "distort", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "amount": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_pinch,
        },
        ToolDef {
            name: "filter_spherize",
            description: "Spherize like a 3D sphere (normal, horizontal, vertical).",
            keywords: &["spherize", "sphere", "fisheye", "distort", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "amount": {"type": "integer"}, "mode": {"type": "string"}}),
                &[],
            ),
            run: h_filter_spherize,
        },
        ToolDef {
            name: "filter_polar_coordinates",
            description: "Polar Coordinates: rectangular to polar or back.",
            keywords: &["polar", "coordinates", "rectangular", "distort", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "to_polar": {"type": "boolean"}}),
                &[],
            ),
            run: h_filter_polar,
        },
        ToolDef {
            name: "filter_high_pass",
            description: "High Pass: image minus its blur, around gray.",
            keywords: &["high", "pass", "sharpen", "edges", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "number"}}),
                &[],
            ),
            run: h_filter_high_pass,
        },
        ToolDef {
            name: "filter_offset",
            description: "Offset: shift the layer with background, repeat-edges or wrap fill.",
            keywords: &["offset", "shift", "move", "tile", "wrap", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "dx": {"type": "integer"}, "dy": {"type": "integer"}, "fill": {"type": "string"}}),
                &[],
            ),
            run: h_filter_offset,
        },
        ToolDef {
            name: "filter_mosaic",
            description: "Mosaic: squares filled with their average.",
            keywords: &["mosaic", "pixelate", "squares", "filter"],
            schema: schema(
                json!({"doc": doc_param(), "cell": {"type": "integer"}}),
                &[],
            ),
            run: h_filter_mosaic,
        },
        ToolDef {
            name: "filter_solarize",
            description: "Solarize: values above 127 are inverted.",
            keywords: &["solarize", "invert", "sabattier", "stylize", "filter"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_filter_solarize,
        },
        // Layers
        ToolDef {
            name: "list_layers",
            description: "List layers bottom to top with ids, names, opacity, blend modes.",
            keywords: &["list", "layers", "stack", "inspect"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_list_layers,
        },
        ToolDef {
            name: "set_active_layer",
            description: "Select a layer by id (makes it active).",
            keywords: &["active", "select", "layer", "current"],
            schema: schema(
                json!({"doc": doc_param(), "layer": {"type": "integer"}}),
                &["layer"],
            ),
            run: h_set_active_layer,
        },
        ToolDef {
            name: "new_raster_layer",
            description: "Create a transparent (or solid-filled) raster layer above the active one.",
            keywords: &["new", "layer", "raster", "create", "transparent"],
            schema: schema(
                json!({"doc": doc_param(), "name": {"type": "string"}, "fill": {"type": "array"}}),
                &[],
            ),
            run: h_new_raster_layer,
        },
        ToolDef {
            name: "new_group",
            description: "Create a layer group.",
            keywords: &["new", "group", "folder", "layers"],
            schema: schema(json!({"doc": doc_param(), "name": {"type": "string"}}), &[]),
            run: h_new_group,
        },
        ToolDef {
            name: "duplicate_layer",
            description: "Duplicate the active layer (or group with contents).",
            keywords: &["duplicate", "copy", "clone", "layer"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_duplicate_layer,
        },
        ToolDef {
            name: "delete_selected_layers",
            description: "Delete the selected layers.",
            keywords: &["delete", "remove", "layer"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_delete_selected_layers,
        },
        ToolDef {
            name: "rename_layer",
            description: "Rename a layer.",
            keywords: &["rename", "name", "layer"],
            schema: schema(
                json!({"doc": doc_param(), "layer": {"type": "integer"}, "name": {"type": "string"}}),
                &["name"],
            ),
            run: h_rename_layer,
        },
        ToolDef {
            name: "set_layer_visibility",
            description: "Show or hide a layer.",
            keywords: &["visibility", "visible", "hide", "show", "eye", "layer"],
            schema: schema(
                json!({"doc": doc_param(), "layer": {"type": "integer"}, "visible": {"type": "boolean"}}),
                &["visible"],
            ),
            run: h_set_layer_visibility,
        },
        ToolDef {
            name: "set_layer_opacity",
            description: "Set layer opacity 0-1.",
            keywords: &["opacity", "transparent", "fade", "layer"],
            schema: schema(
                json!({"doc": doc_param(), "layer": {"type": "integer"}, "opacity": {"type": "number"}}),
                &["opacity"],
            ),
            run: h_set_layer_opacity,
        },
        ToolDef {
            name: "set_layer_fill",
            description: "Set layer fill opacity 0-1.",
            keywords: &["fill", "opacity", "layer"],
            schema: schema(
                json!({"doc": doc_param(), "layer": {"type": "integer"}, "fill": {"type": "number"}}),
                &["fill"],
            ),
            run: h_set_layer_fill,
        },
        ToolDef {
            name: "set_layer_blend_mode",
            description: "Set blend mode (normal, multiply, screen, overlay, ...).",
            keywords: &["blend", "mode", "multiply", "screen", "overlay", "layer"],
            schema: schema(
                json!({"doc": doc_param(), "layer": {"type": "integer"}, "mode": {"type": "string"}}),
                &["mode"],
            ),
            run: h_set_layer_blend_mode,
        },
        ToolDef {
            name: "arrange_layer",
            description: "Arrange: bring to front, forward, backward, to back.",
            keywords: &[
                "arrange", "order", "front", "back", "forward", "backward", "layer", "stack",
            ],
            schema: schema(
                json!({"doc": doc_param(), "arrange": {"type": "string"}}),
                &["arrange"],
            ),
            run: h_arrange_layer,
        },
        ToolDef {
            name: "merge_down",
            description: "Merge the active layer down.",
            keywords: &["merge", "down", "combine", "flatten", "layer"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_merge_down,
        },
        ToolDef {
            name: "merge_visible",
            description: "Merge all visible layers.",
            keywords: &["merge", "visible", "flatten", "combine", "layer"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_merge_visible,
        },
        ToolDef {
            name: "flatten_image",
            description: "Flatten the whole image to one background.",
            keywords: &["flatten", "merge", "background", "layer"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_flatten_image,
        },
        ToolDef {
            name: "group_selected_layers",
            description: "Group the selected layers.",
            keywords: &["group", "folder", "layers", "combine"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_group_selected,
        },
        ToolDef {
            name: "ungroup_layers",
            description: "Ungroup the active group.",
            keywords: &["ungroup", "folder", "layers", "expand"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_ungroup,
        },
        ToolDef {
            name: "add_layer_mask",
            description: "Add a layer mask (reveal/hide all or from selection).",
            keywords: &["mask", "layer", "reveal", "hide", "selection"],
            schema: schema(json!({"doc": doc_param(), "kind": {"type": "string"}}), &[]),
            run: h_add_layer_mask,
        },
        ToolDef {
            name: "convert_background_to_layer",
            description: "Convert the background into a regular layer.",
            keywords: &["background", "convert", "unlock", "layer"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_convert_background,
        },
        ToolDef {
            name: "align_layers",
            description: "Align selected layers (top, bottom, left, right, centers).",
            keywords: &[
                "align", "top", "bottom", "left", "right", "center", "layers",
            ],
            schema: schema(
                json!({"doc": doc_param(), "how": {"type": "string"}}),
                &["how"],
            ),
            run: h_align_layers,
        },
        ToolDef {
            name: "distribute_layers",
            description: "Distribute selected layers evenly.",
            keywords: &["distribute", "space", "evenly", "layers", "align"],
            schema: schema(
                json!({"doc": doc_param(), "how": {"type": "string"}}),
                &["how"],
            ),
            run: h_distribute_layers,
        },
        ToolDef {
            name: "layer_via_copy",
            description: "Layer Via Copy: selected pixels to a new layer.",
            keywords: &["via", "copy", "layer", "selection", "duplicate"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_layer_via_copy,
        },
        ToolDef {
            name: "layer_via_cut",
            description: "Layer Via Cut: selected pixels cut to a new layer.",
            keywords: &["via", "cut", "layer", "selection", "move"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_layer_via_cut,
        },
        // Selection
        ToolDef {
            name: "select_all",
            description: "Select > All.",
            keywords: &["select", "all", "selection", "marquee"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_select_all,
        },
        ToolDef {
            name: "clear_selection",
            description: "Select > Deselect.",
            keywords: &["deselect", "clear", "selection", "none"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_clear_selection,
        },
        ToolDef {
            name: "reselect_selection",
            description: "Select > Reselect the last selection.",
            keywords: &["reselect", "restore", "selection"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_reselect,
        },
        ToolDef {
            name: "select_rect",
            description: "Rectangular marquee selection.",
            keywords: &["rect", "rectangle", "marquee", "select", "selection"],
            schema: schema(
                json!({"doc": doc_param(), "x0": {"type": "number"}, "y0": {"type": "number"}, "x1": {"type": "number"}, "y1": {"type": "number"}}),
                &["x0", "y0", "x1", "y1"],
            ),
            run: h_select_rect,
        },
        ToolDef {
            name: "select_ellipse",
            description: "Elliptical marquee selection.",
            keywords: &[
                "ellipse",
                "elliptical",
                "circle",
                "marquee",
                "select",
                "selection",
            ],
            schema: schema(
                json!({"doc": doc_param(), "x0": {"type": "number"}, "y0": {"type": "number"}, "x1": {"type": "number"}, "y1": {"type": "number"}, "anti_alias": {"type": "boolean"}}),
                &["x0", "y0", "x1", "y1"],
            ),
            run: h_select_ellipse,
        },
        ToolDef {
            name: "feather_selection",
            description: "Feather (soften) the selection edge.",
            keywords: &["feather", "soften", "selection", "blur", "edge"],
            schema: schema(
                json!({"doc": doc_param(), "radius": {"type": "number"}}),
                &[],
            ),
            run: h_feather_selection,
        },
        // Canvas
        ToolDef {
            name: "rotate_flip_canvas",
            description: "Rotate 180/90 or flip horizontal/vertical.",
            keywords: &["rotate", "flip", "canvas", "orientation", "mirror", "image"],
            schema: schema(
                json!({"doc": doc_param(), "orientation": {"type": "string", "description": "rotate_180, rotate_90_cw, rotate_90_ccw, flip_horizontal, flip_vertical"}}),
                &["orientation"],
            ),
            run: h_rotate_flip,
        },
        ToolDef {
            name: "rotate_canvas_arbitrary",
            description: "Rotate canvas by degrees about its center.",
            keywords: &["rotate", "arbitrary", "angle", "canvas", "straighten"],
            schema: schema(
                json!({"doc": doc_param(), "degrees": {"type": "number"}}),
                &["degrees"],
            ),
            run: h_rotate_arbitrary,
        },
        ToolDef {
            name: "resize_canvas",
            description: "Image > Canvas Size with anchor and fill color.",
            keywords: &["canvas", "size", "resize", "extend", "anchor", "crop"],
            schema: schema(
                json!({"doc": doc_param(), "width": {"type": "integer"}, "height": {"type": "integer"}, "anchor_x": {"type": "integer"}, "anchor_y": {"type": "integer"}}),
                &["width", "height"],
            ),
            run: h_resize_canvas,
        },
        ToolDef {
            name: "crop_image",
            description: "Crop to a pixel box.",
            keywords: &["crop", "trim", "cut", "canvas"],
            schema: schema(
                json!({"doc": doc_param(), "x0": {"type": "integer"}, "y0": {"type": "integer"}, "x1": {"type": "integer"}, "y1": {"type": "integer"}}),
                &["x0", "y0", "x1", "y1"],
            ),
            run: h_crop,
        },
        ToolDef {
            name: "crop_to_selection",
            description: "Image > Crop to the selection bounds.",
            keywords: &["crop", "selection", "trim", "canvas"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_crop_to_selection,
        },
        ToolDef {
            name: "resize_image",
            description: "Image > Image Size resampling.",
            keywords: &["image", "size", "resize", "resample", "scale", "resolution"],
            schema: schema(
                json!({"doc": doc_param(), "width": {"type": "integer"}, "height": {"type": "integer"}, "method": {"type": "string"}}),
                &["width", "height"],
            ),
            run: h_resize_image,
        },
        ToolDef {
            name: "trim_image",
            description: "Image > Trim transparent or solid borders.",
            keywords: &["trim", "crop", "transparent", "border", "canvas"],
            schema: schema(
                json!({"doc": doc_param(), "basis": {"type": "string"}, "top": {"type": "boolean"}, "left": {"type": "boolean"}, "bottom": {"type": "boolean"}, "right": {"type": "boolean"}}),
                &[],
            ),
            run: h_trim,
        },
        ToolDef {
            name: "reveal_all",
            description: "Image > Reveal All: grow canvas to every pixel.",
            keywords: &["reveal", "all", "canvas", "grow", "bounds"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_reveal_all,
        },
        // Edit
        ToolDef {
            name: "fill_solid",
            description: "Edit > Fill with a solid color, blend mode and opacity.",
            keywords: &["fill", "paint", "color", "bucket", "background", "edit"],
            schema: schema(
                json!({"doc": doc_param(), "color": {"type": "array", "items": {"type": "integer"}, "minItems": 3, "maxItems": 3}, "opacity": {"type": "number"}, "mode": {"type": "string"}, "preserve_transparency": {"type": "boolean"}}),
                &["color"],
            ),
            run: h_fill_solid,
        },
        ToolDef {
            name: "clear_layer",
            description: "Edit > Clear (erase selection; background fills instead).",
            keywords: &["clear", "erase", "delete", "transparent", "edit"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_clear_layer,
        },
        ToolDef {
            name: "paint_bucket",
            description: "Paint Bucket fill from a clicked point with tolerance.",
            keywords: &["bucket", "paint", "fill", "tolerance", "flood", "tool"],
            schema: schema(
                json!({"doc": doc_param(), "x": {"type": "integer"}, "y": {"type": "integer"}, "color": {"type": "array", "items": {"type": "integer"}, "minItems": 3, "maxItems": 3}, "tolerance": {"type": "integer"}, "anti_alias": {"type": "boolean"}, "contiguous": {"type": "boolean"}, "all_layers": {"type": "boolean"}}),
                &["x", "y", "color"],
            ),
            run: h_paint_bucket,
        },
        ToolDef {
            name: "gradient_fill",
            description: "Gradient tool drag between two points and colors.",
            keywords: &["gradient", "blend", "fade", "linear", "radial", "tool"],
            schema: schema(
                json!({"doc": doc_param(), "x0": {"type": "number"}, "y0": {"type": "number"}, "x1": {"type": "number"}, "y1": {"type": "number"}, "from": {"type": "array"}, "to": {"type": "array"}, "kind": {"type": "string", "description": "linear, radial, angle, reflected, diamond"}, "opacity": {"type": "number"}, "reverse": {"type": "boolean"}}),
                &["from", "to"],
            ),
            run: h_gradient_fill,
        },
        ToolDef {
            name: "move_active_layer",
            description: "Move tool: shift layers or selected pixels by dx/dy.",
            keywords: &["move", "shift", "nudge", "translate", "tool", "layer"],
            schema: schema(
                json!({"doc": doc_param(), "dx": {"type": "integer"}, "dy": {"type": "integer"}}),
                &[],
            ),
            run: h_move_active_layer,
        },
        // History
        ToolDef {
            name: "history_undo",
            description: "Edit > Undo.",
            keywords: &["undo", "history", "revert", "back"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_undo,
        },
        ToolDef {
            name: "history_redo",
            description: "Edit > Redo.",
            keywords: &["redo", "history", "forward", "reapply"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_redo,
        },
        ToolDef {
            name: "list_history",
            description: "History panel: list named states and the current one.",
            keywords: &["history", "states", "undo", "list", "panel"],
            schema: schema(json!({"doc": doc_param()}), &[]),
            run: h_list_history,
        },
    ]
}

fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

fn search_catalog(query: &str, limit: usize) -> Value {
    let tools = all_tools();
    let q = query.trim().to_lowercase();
    if q.is_empty() || q == "*" || q == "all" || q == "list" || q == "list all" {
        let items: Vec<Value> = tools
            .iter()
            .take(limit)
            .map(|t| json!({"name": t.name, "description": t.description, "inputSchema": t.schema}))
            .collect();
        return json!({"query": query, "count": items.len(), "total_tools": tools.len(), "tools": items});
    }
    let qnorm: String = q.chars().filter(|c| c.is_alphanumeric()).collect();
    let qtokens = tokenize(&q);
    let mut scored: Vec<(i64, &ToolDef)> = Vec::new();
    for t in &tools {
        let name_norm: String = t.name.chars().filter(|c| c.is_alphanumeric()).collect();
        let desc_lower = t.description.to_lowercase();
        let hay_tokens: Vec<String> = tokenize(t.name)
            .into_iter()
            .chain(t.keywords.iter().map(|k| k.to_string()))
            .chain(tokenize(t.description))
            .collect();
        let mut score: i64 = 0;
        if !qnorm.is_empty() {
            if name_norm == qnorm {
                score += 200;
            } else if name_norm.contains(&qnorm as &str) {
                score += 100;
            }
            if desc_lower.contains(&q as &str) {
                score += 20;
            }
        }
        for qt in &qtokens {
            if qt.len() < 3 {
                continue;
            }
            let mut best = 0;
            for ht in &hay_tokens {
                if ht.len() < 3 {
                    continue;
                }
                if ht == qt {
                    best = best.max(15);
                } else if ht.contains(qt as &str) {
                    best = best.max(7);
                } else if ht.len() >= 5 && (qt as &str).contains(ht as &str) {
                    // Query word is a longer form of a catalog word
                    // (e.g. "blurring" vs "blur"); ht.len() >= 5 keeps
                    // common short words ("per", "tin", "ion") from
                    // matching inside unrelated query words.
                    best = best.max(5);
                }
            }
            score += best;
            if t.keywords.iter().any(|k| *k == qt) {
                score += 10;
            }
        }
        if score > 0 {
            scored.push((score, t));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(b.1.name)));
    let items: Vec<Value> = scored
        .into_iter()
        .take(limit)
        .map(
            |(_, t)| json!({"name": t.name, "description": t.description, "inputSchema": t.schema}),
        )
        .collect();
    json!({"query": query, "count": items.len(), "total_tools": tools.len(), "tools": items})
}

fn call_catalog(s: &mut AppState, name: &str, args: &Value) -> Result<Value, String> {
    let tools = all_tools();
    let def = tools.iter().find(|t| t.name == name);
    match def {
        Some(t) => (t.run)(s, args),
        None => Err(format!(
            "unknown tool '{name}'; use search_tools to discover available tool names"
        )),
    }
}

// ---------------------------------------------------------------------------
// MCP protocol (JSON-RPC 2.0 over stdio, newline-delimited)
// ---------------------------------------------------------------------------

const SEARCH_TOOL_NAME: &str = "search_tools";
const CALL_TOOL_NAME: &str = "call_tool";

fn search_tool_def() -> Value {
    json!({
        "name": SEARCH_TOOL_NAME,
        "description": "Keyword search over all OpenPhoto operations. Returns matching tool names with their full definitions (description + JSON input schema). Call this first to discover how to do something, then run the operation with call_tool. Query examples: 'blur', 'sharpen noise', 'resize canvas', 'layer opacity', 'hue saturation'.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Keywords, e.g. 'gaussian blur', 'export png', 'merge layers'"},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50, "default": 10}
            },
            "required": ["query"],
            "additionalProperties": false
        }
    })
}

fn call_tool_def() -> Value {
    json!({
        "name": CALL_TOOL_NAME,
        "description": "Run one OpenPhoto operation found via search_tools. 'name' is the internal tool name (e.g. 'filter_gaussian_blur'); 'arguments' is its parameter object, matching the inputSchema returned by search_tools. 'doc' selects the document and defaults to the active one.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Internal tool name from search_tools"},
                "arguments": {"type": "object", "description": "Parameters for that tool", "default": {}}
            },
            "required": ["name"],
            "additionalProperties": false
        }
    })
}

fn ok(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn err(id: &Value, code: i64, message: String) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn text_result(payload: &Value, is_error: bool) -> Value {
    let text = serde_json::to_string_pretty(payload).unwrap_or_else(|_| payload.to_string());
    json!({"content": [{"type": "text", "text": text}], "isError": is_error})
}

fn handle_message(state: &mut AppState, msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(|m| m.as_str())?;
    let has_id = msg.get("id").is_some_and(|v| !v.is_null());
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    // Notifications (no id) get no response.
    if method.starts_with("notifications/") {
        return None;
    }
    match method {
        "initialize" => {
            let requested = msg
                .get("params")
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");
            Some(ok(
                &id,
                json!({
                    "protocolVersion": requested,
                    "capabilities": {"tools": {"listChanged": false}},
                    "serverInfo": {"name": "openphoto-mcp", "version": env!("CARGO_PKG_VERSION")},
                    "instructions": "OpenPhoto gateway. Only two tools are exposed: search_tools (keyword search over ~100 image operations; returns full definitions) and call_tool (run one operation by name). Workflow: 1) search_tools with keywords, 2) call_tool with the chosen name + arguments. Documents: open_image/new_document first; omit 'doc' to use the active document."
                }),
            ))
        }
        "ping" => Some(ok(&id, json!({}))),
        "tools/list" => Some(ok(
            &id,
            json!({"tools": [search_tool_def(), call_tool_def()]}),
        )),
        "tools/call" => {
            if !has_id {
                return None;
            }
            let params = msg.get("params").cloned().unwrap_or(json!({}));
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match name {
                SEARCH_TOOL_NAME => {
                    let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
                    let limit = args
                        .get("limit")
                        .and_then(|l| l.as_u64())
                        .unwrap_or(10)
                        .clamp(1, 50) as usize;
                    let payload = search_catalog(query, limit);
                    Some(ok(&id, text_result(&payload, false)))
                }
                CALL_TOOL_NAME => {
                    let inner = args.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    if inner.is_empty() {
                        return Some(ok(
                            &id,
                            text_result(
                                &json!({"error": "missing 'name' of the tool to call"}),
                                true,
                            ),
                        ));
                    }
                    let inner_args = args.get("arguments").cloned().unwrap_or(json!({}));
                    let args_obj = if inner_args.is_null() {
                        json!({})
                    } else {
                        inner_args
                    };
                    match call_catalog(state, inner, &args_obj) {
                        Ok(v) => Some(ok(&id, text_result(&v, false))),
                        Err(e) => Some(ok(&id, text_result(&json!({"error": e}), true))),
                    }
                }
                _ => Some(err(&id, -32601, format!("unknown tool '{name}'"))),
            }
        }
        _ => {
            if has_id {
                Some(err(&id, -32601, format!("method not found: {method}")))
            } else {
                None
            }
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut state = AppState::new();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let resp = json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": format!("parse error: {e}")}});
                if let Err(ioe) = writeln!(out, "{}", resp) {
                    eprintln!("op-mcp write error: {ioe}");
                    break;
                }
                let _ = out.flush();
                continue;
            }
        };
        match handle_message(&mut state, &msg) {
            Some(resp) => {
                if let Err(ioe) = writeln!(out, "{resp}") {
                    eprintln!("op-mcp write error: {ioe}");
                    break;
                }
                let _ = out.flush();
            }
            None => {
                let _ = out.flush();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn args(pairs: &[(&str, Value)]) -> Value {
        let mut m = serde_json::Map::new();
        for (k, v) in pairs {
            m.insert(k.to_string(), v.clone());
        }
        Value::Object(m)
    }

    #[test]
    fn search_finds_blur_and_returns_definitions() {
        let v = search_catalog("gaussian blur", 5);
        let tools = v.get("tools").and_then(|t| t.as_array()).unwrap();
        assert!(!tools.is_empty());
        assert!(
            tools
                .iter()
                .any(|t| t.get("name").and_then(|n| n.as_str()) == Some("filter_gaussian_blur"))
        );
        let first = &tools[0];
        assert!(first.get("inputSchema").is_some());
        assert!(first.get("description").is_some());
    }

    #[test]
    fn search_empty_query_lists_tools() {
        let v = search_catalog("", 5);
        assert_eq!(v.get("tools").and_then(|t| t.as_array()).unwrap().len(), 5);
    }

    #[test]
    fn search_unknown_query_returns_empty() {
        let v = search_catalog("zzz_no_such_operation_qqq", 5);
        assert_eq!(v.get("count").and_then(|c| c.as_u64()), Some(0));
    }

    #[test]
    fn new_invert_and_pixel_roundtrip() {
        let mut s = AppState::new();
        let id = {
            let v = h_new_document(&mut s, &args(&[("width", json!(4)), ("height", json!(4))]))
                .unwrap();
            v.get("doc").and_then(|d| d.as_str()).unwrap().to_string()
        };
        let a = args(&[("doc", json!(id))]);
        h_adjust_invert(&mut s, &a).unwrap();
        let px = h_get_pixel(
            &mut s,
            &args(&[("doc", json!(id)), ("x", json!(0)), ("y", json!(0))]),
        )
        .unwrap();
        // White background inverted is black.
        assert_eq!(px.get("r").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(px.get("a").and_then(|v| v.as_u64()), Some(255));
    }

    #[test]
    fn gaussian_blur_and_undo_redo() {
        let mut s = AppState::new();
        h_new_document(&mut s, &args(&[("width", json!(8)), ("height", json!(8))])).unwrap();
        let doc = s.active.clone().unwrap();
        let a = args(&[("doc", json!(doc))]);
        h_filter_gaussian_blur(
            &mut s,
            &args(&[("doc", json!(doc)), ("radius", json!(1.0))]),
        )
        .unwrap();
        h_undo(&mut s, &a).unwrap();
        h_redo(&mut s, &a).unwrap();
        let h = h_list_history(&mut s, &a).unwrap();
        assert!(h.get("states").and_then(|v| v.as_array()).unwrap().len() >= 2);
    }

    #[test]
    fn mcp_initialize_list_call_roundtrip() {
        let mut s = AppState::new();
        let init = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05"}});
        let resp = handle_message(&mut s, &init).unwrap();
        assert_eq!(
            resp.get("result")
                .and_then(|r| r.get("protocolVersion"))
                .and_then(|v| v.as_str()),
            Some("2024-11-05")
        );

        let list = json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"});
        let resp = handle_message(&mut s, &list).unwrap();
        let tools = resp
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .unwrap();
        assert_eq!(tools.len(), 2);
        assert!(
            tools
                .iter()
                .any(|t| t.get("name").and_then(|n| n.as_str()) == Some("search_tools"))
        );
        assert!(
            tools
                .iter()
                .any(|t| t.get("name").and_then(|n| n.as_str()) == Some("call_tool"))
        );

        // search via MCP
        let search = json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "search_tools", "arguments": {"query": "merge layers", "limit": 5}}});
        let resp = handle_message(&mut s, &search).unwrap();
        let text = resp
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get(0))
            .and_then(|t| t.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();
        assert!(text.contains("merge"));

        // unknown internal tool is an MCP-level success with isError
        let bad = json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "call_tool", "arguments": {"name": "nope_not_a_tool"}}});
        let resp = handle_message(&mut s, &bad).unwrap();
        assert_eq!(
            resp.get("result")
                .and_then(|r| r.get("isError"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn canvas_and_layer_ops_roundtrip() {
        let mut s = AppState::new();
        h_new_document(
            &mut s,
            &args(&[("width", json!(16)), ("height", json!(16))]),
        )
        .unwrap();
        let doc = s.active.clone().unwrap();
        let d = || args(&[("doc", json!(doc))]);
        h_new_raster_layer(&mut s, &d()).unwrap();
        h_fill_solid(
            &mut s,
            &args(&[("doc", json!(doc)), ("color", json!([255, 0, 0]))]),
        )
        .unwrap();
        h_resize_canvas(
            &mut s,
            &args(&[
                ("doc", json!(doc)),
                ("width", json!(20)),
                ("height", json!(20)),
            ]),
        )
        .unwrap();
        let info = h_get_document_info(&mut s, &d()).unwrap();
        assert_eq!(info.get("width").and_then(|v| v.as_u64()), Some(20));
        h_merge_visible(&mut s, &d()).unwrap();
        h_export_composite(
            &mut s,
            &args(&[
                ("doc", json!(doc)),
                (
                    "path",
                    json!(format!("/tmp/op-mcp-test-{}.png", std::process::id())),
                ),
            ]),
        )
        .unwrap();
    }

    #[test]
    fn selections_are_history_states() {
        let mut s = AppState::new();
        h_new_document(
            &mut s,
            &args(&[("width", json!(16)), ("height", json!(16))]),
        )
        .unwrap();
        let d = args(&[]);
        h_filter_gaussian_blur(&mut s, &args(&[("radius", json!(1.0))])).unwrap();
        h_select_rect(
            &mut s,
            &args(&[
                ("x0", json!(2)),
                ("y0", json!(2)),
                ("x1", json!(8)),
                ("y1", json!(8)),
            ]),
        )
        .unwrap();
        // Undo takes back the selection only; the blur stays
        h_undo(&mut s, &d).unwrap();
        let h = h_list_history(&mut s, &d).unwrap();
        let states = h.get("states").and_then(|v| v.as_array()).unwrap();
        let names: Vec<&str> = states
            .iter()
            .map(|st| st.get("name").and_then(|n| n.as_str()).unwrap())
            .collect();
        assert_eq!(names, ["New", "Gaussian Blur", "Rectangular Marquee"]);
        let current = states
            .iter()
            .position(|st| st.get("current").and_then(|c| c.as_bool()) == Some(true));
        assert_eq!(current, Some(1));
        let info = h_get_document_info(&mut s, &d).unwrap();
        assert_eq!(
            info.get("has_selection").and_then(|v| v.as_bool()),
            Some(false)
        );
        // Deselecting nothing adds no state
        h_clear_selection(&mut s, &d).unwrap();
        h_select_all(&mut s, &d).unwrap();
        h_clear_selection(&mut s, &d).unwrap();
        let h = h_list_history(&mut s, &d).unwrap();
        let last = h
            .get("states")
            .and_then(|v| v.as_array())
            .unwrap()
            .last()
            .cloned()
            .unwrap();
        assert_eq!(last.get("name").and_then(|n| n.as_str()), Some("Deselect"));
    }

    #[test]
    fn sizes_and_filter_parameters_are_limited() {
        let mut s = AppState::new();
        h_new_document(&mut s, &args(&[("width", json!(8)), ("height", json!(8))])).unwrap();
        let big = args(&[("width", json!(200_000)), ("height", json!(200_000))]);
        assert!(h_resize_image(&mut s, &big).is_err());
        assert!(h_resize_canvas(&mut s, &big).is_err());
        assert!(h_new_document(&mut s, &big).is_err());
        assert!(h_filter_median(&mut s, &args(&[("radius", json!(100_000))])).is_err());
        assert!(h_filter_box_blur(&mut s, &args(&[("radius", json!(5000))])).is_err());
        assert!(h_filter_motion_blur(&mut s, &args(&[("distance", json!(5000))])).is_err());
        assert!(h_filter_maximum(&mut s, &args(&[("radius", json!(600.0))])).is_err());
        assert!(h_filter_mosaic(&mut s, &args(&[("cell", json!(1))])).is_err());
        // In range still works; nothing failed above was recorded
        h_filter_median(&mut s, &args(&[("radius", json!(1))])).unwrap();
        let h = h_list_history(&mut s, &args(&[])).unwrap();
        assert_eq!(h.get("states").and_then(|v| v.as_array()).unwrap().len(), 2);
    }

    #[test]
    fn schemas_list_what_handlers_require() {
        // Every parameter a handler needs is in its schema's "required"
        for (name, needed) in [("fill_solid", "color"), ("paint_bucket", "color")] {
            let tools = all_tools();
            let t = tools.iter().find(|t| t.name == name).unwrap();
            let required = t.schema.get("required").and_then(|r| r.as_array()).unwrap();
            assert!(
                required.iter().any(|r| r.as_str() == Some(needed)),
                "{name}"
            );
        }
        let mut s = AppState::new();
        h_new_document(&mut s, &args(&[("width", json!(8)), ("height", json!(8))])).unwrap();
        // The gradient tool rejects a method instead of ignoring it
        let a = args(&[("from", json!([0, 0, 0])), ("to", json!([255, 255, 255]))]);
        h_gradient_fill(&mut s, &a).unwrap();
        let mut b = a.clone();
        b["method"] = json!("linear");
        assert!(h_gradient_fill(&mut s, &b).is_err());
    }
}
