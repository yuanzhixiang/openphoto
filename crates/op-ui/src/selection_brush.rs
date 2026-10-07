//! The Selection Brush tool (Photoshop 2026): painting the selection, shown
//! as a colored overlay while the tool is current.

use egui::{Color32, Pos2, Rect, Ui};
use op_core::paint::BrushTip;
use op_core::selection_brush::SelectionStroke;

use crate::document_view::{to_doc, to_screen};
use crate::state::{DocState, PaintOptions};

/// The gear menu's Overlay Option colors, in Photoshop's order (their
/// swatches as Photoshop draws them).
pub const OVERLAY_COLORS: [(&str, [u8; 3]); 13] = [
    ("Blue", [0x42, 0x7f, 0xe4]),
    ("Green", [0x61, 0xb6, 0x93]),
    ("Chartreuse", [0xa1, 0xdc, 0x60]),
    ("Yellow", [0xf4, 0xdb, 0x49]),
    ("Orange", [0xec, 0xa8, 0x54]),
    ("Red", [0xe6, 0x76, 0x77]),
    // The overlay Photoshop paints with its default Magenta, measured on
    // the canvas (the swatch itself is #d2559b)
    ("Magenta", [0xe5, 0x61, 0xab]),
    ("Purple", [0x8a, 0x59, 0xd3]),
    ("Indigo", [0x68, 0x67, 0xe4]),
    ("Fuchsia", [0xd4, 0x6d, 0xe8]),
    ("Seafoam", [0x63, 0xcc, 0xd3]),
    ("White", [0xf4, 0xf5, 0xf4]),
    ("Black", [0x00, 0x00, 0x00]),
];
/// Magenta.
pub const DEFAULT_OVERLAY: usize = 6;
/// The overlay covers fully selected pixels at half strength.
const OVERLAY_OPACITY: f32 = 0.5;

/// Photoshop's alert when the tool is used with no layer selected.
pub const NO_LAYER: &str = "Can't use the tool because no layers are selected.";

/// Press and drag to paint the selection: Add (or Subtract, also while Alt
/// is held) with the tool's brush at its Opacity. One "Selection Brush"
/// history state per stroke. Returns Photoshop's alert when no layer is
/// selected.
pub fn input(
    ui: &Ui,
    response: &egui::Response,
    state: &mut DocState,
    opts: PaintOptions,
    subtract: bool,
    ppp: f32,
) -> Option<String> {
    let pointer = ui
        .input(|i| i.pointer.interact_pos())
        .map(|p| to_doc(state, p, ppp));
    if ui.input(|i| i.pointer.primary_pressed()) && response.hovered() {
        if state.doc.active_layer.is_none() {
            return Some(NO_LAYER.to_owned());
        }
        let alt = ui.input(|i| i.modifiers.alt);
        let tip = BrushTip {
            diameter: opts.size,
            hardness: opts.hardness,
            aliased: false,
            square: false,
            angle: 0.0,
            roundness: 1.0,
            spacing: 0.25,
        };
        let (w, h) = (state.doc.width, state.doc.height);
        let mut stroke = SelectionStroke::new(
            w,
            h,
            state.doc.selection(),
            tip,
            opts.opacity,
            subtract != alt,
        );
        if let Some(p) = pointer {
            stroke.add_point(p.x, p.y);
        }
        state.doc.set_selection(stroke.selection());
        state.selection_stroke = Some(stroke);
    }
    let stroke = state.selection_stroke.as_mut()?;
    if let Some(p) = pointer {
        stroke.add_point(p.x, p.y);
    }
    let selection = stroke.selection();
    let done = !ui.input(|i| i.pointer.primary_down());
    state.doc.set_selection(selection);
    if done {
        state.selection_stroke = None;
        state.record("Selection Brush");
    } else {
        ui.ctx().request_repaint();
    }
    None
}

/// The selection as Photoshop shows it with this tool: the overlay color
/// at half strength over selected pixels (in proportion to how selected
/// they are), instead of marching ants. The texture is rebuilt when the
/// selection or the color changes.
pub fn draw_overlay(ui: &Ui, state: &mut DocState, clip: Rect, color: [u8; 3], ppp: f32) {
    let Some(selection) = state.doc.selection() else {
        return;
    };
    let key = (state.doc.selection_revision(), color);
    let stale = state.overlay.as_ref().is_none_or(|(k, _)| *k != key);
    if stale {
        let (w, h) = (selection.width() as usize, selection.height() as usize);
        let mut pixels = Vec::with_capacity(w * h);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let a = selection.get(x, y) as f32 / 255.0 * OVERLAY_OPACITY;
                pixels.push(Color32::from_rgba_unmultiplied(
                    color[0],
                    color[1],
                    color[2],
                    (a * 255.0).round() as u8,
                ));
            }
        }
        let image = egui::ColorImage::new([w, h], pixels);
        let texture = ui.ctx().load_texture(
            "selection-brush-overlay",
            image,
            egui::TextureOptions::NEAREST,
        );
        state.overlay = Some((key, texture));
    }
    let Some((_, texture)) = &state.overlay else {
        return;
    };
    // The image's corners mapped one by one, so the overlay turns and
    // flips with the view
    let (w, h) = (state.doc.width as f32, state.doc.height as f32);
    let corners = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]
        .map(|(x, y)| to_screen(state, Pos2::new(x, y), ppp));
    let uvs = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let mut mesh = egui::Mesh::with_texture(texture.id());
    for (p, (u, v)) in corners.iter().zip(uvs) {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: *p,
            uv: Pos2::new(u, v),
            color: Color32::WHITE,
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    ui.painter().with_clip_rect(clip).add(mesh);
}
