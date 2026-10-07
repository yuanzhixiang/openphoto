//! The adjustment dialogs' preset files: the gear menu's Save Preset...,
//! Load Preset... and Delete Current Preset, and the saved presets listed
//! in the Preset pop-up. The files are Photoshop's own formats (decoded
//! from the presets Photoshop 2026 ships), so presets move both ways.

use std::path::{Path, PathBuf};

use crate::native_popup::{self, Entry};

/// One dialog's kind of preset file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kind {
    /// The folder under Presets (as Photoshop names it).
    pub folder: &'static str,
    pub ext: &'static str,
}

pub const LEVELS: Kind = Kind {
    folder: "Levels",
    ext: "alv",
};
pub const CURVES: Kind = Kind {
    folder: "Curves",
    ext: "acv",
};
pub const HUE_SATURATION: Kind = Kind {
    folder: "Hue and Saturation",
    ext: "ahu",
};
pub const CHANNEL_MIXER: Kind = Kind {
    folder: "Channel Mixer",
    ext: "cha",
};
pub const BLACK_WHITE: Kind = Kind {
    folder: "Black and White",
    ext: "blw",
};
pub const EXPOSURE: Kind = Kind {
    folder: "Exposure",
    ext: "eap",
};

impl Kind {
    /// Where saved presets live:
    /// `~/Library/Application Support/OpenPhoto/Presets/<folder>`.
    pub fn dir(self) -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(
            PathBuf::from(home)
                .join("Library/Application Support/OpenPhoto/Presets")
                .join(self.folder),
        )
    }

    /// The saved presets (name without extension, path), by name.
    pub fn saved(self) -> Vec<(String, PathBuf)> {
        self.dir().map_or_else(Vec::new, |d| self.saved_in(&d))
    }

    pub fn saved_in(self, dir: &Path) -> Vec<(String, PathBuf)> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut out: Vec<(String, PathBuf)> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case(self.ext))
            })
            .filter_map(|p| Some((p.file_stem()?.to_string_lossy().into_owned(), p)))
            .collect();
        out.sort_by_key(|a| a.0.to_lowercase());
        out
    }

    /// Save Preset...: asks where (the presets folder to start with) and
    /// writes `bytes` there. Returns the preset's name.
    pub fn save(self, bytes: &[u8]) -> Option<String> {
        let mut dialog = rfd::FileDialog::new()
            .add_filter(self.folder, &[self.ext])
            .set_file_name(format!("Untitled.{}", self.ext));
        if let Some(dir) = self.dir() {
            let _ = std::fs::create_dir_all(&dir);
            dialog = dialog.set_directory(dir);
        }
        let mut path = dialog.save_file()?;
        if path.extension().is_none() {
            path.set_extension(self.ext);
        }
        std::fs::write(&path, bytes).ok()?;
        Some(path.file_stem()?.to_string_lossy().into_owned())
    }

    /// Load Preset...: a preset file anywhere. Returns its name and bytes.
    pub fn load(self) -> Option<(String, Vec<u8>)> {
        let mut dialog = rfd::FileDialog::new().add_filter(self.folder, &[self.ext]);
        if let Some(dir) = self.dir() {
            dialog = dialog.set_directory(dir);
        }
        let path = dialog.pick_file()?;
        let bytes = std::fs::read(&path).ok()?;
        Some((path.file_stem()?.to_string_lossy().into_owned(), bytes))
    }
}

/// What the gear menu asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearPick {
    Save,
    Load,
    Delete,
}

/// The gear's menu (a native pop-up) under `response`: Save Preset...,
/// Load Preset... and Delete Current Preset (only for a saved preset).
pub fn gear_menu(
    ui: &mut egui::Ui,
    response: &egui::Response,
    can_delete: bool,
) -> Option<GearPick> {
    let entries = [
        Entry::item("Save Preset...", false),
        Entry::item("Load Preset...", false),
        Entry::item("Delete Current Preset", false).enabled(can_delete),
    ];
    match native_popup::dropdown(ui, response, response.id.with("gear"), &entries)? {
        0 => Some(GearPick::Save),
        1 => Some(GearPick::Load),
        2 => Some(GearPick::Delete),
        _ => None,
    }
}

/// A kind's saved-preset state: the preset loaded or saved last, as its
/// name and its file's bytes. The dialog shows its name while its values
/// still encode to those bytes. Kept per kind for the session, as only one
/// dialog of a kind is open at a time.
#[derive(Clone, Debug, Default)]
struct Current {
    name: Option<String>,
    bytes: Vec<u8>,
}

static CURRENT: std::sync::Mutex<Vec<(&str, Current)>> = std::sync::Mutex::new(Vec::new());

fn current(kind: Kind) -> Current {
    CURRENT
        .lock()
        .ok()
        .and_then(|c| {
            c.iter()
                .find(|(k, _)| *k == kind.folder)
                .map(|(_, c)| c.clone())
        })
        .unwrap_or_default()
}

fn set_current(kind: Kind, name: Option<String>, bytes: Vec<u8>) {
    if let Ok(mut c) = CURRENT.lock() {
        c.retain(|(k, _)| *k != kind.folder);
        c.push((kind.folder, Current { name, bytes }));
    }
}

/// The saved preset's name while the dialog's values (`encoded`) are
/// still that preset's.
pub fn shown(kind: Kind, encoded: Option<&[u8]>) -> Option<String> {
    let c = current(kind);
    c.name
        .filter(|_| encoded.is_some_and(|e| e == c.bytes.as_slice()))
}

/// Adds the saved presets to a Preset pop-up's entries (after a
/// separator, the one named `shown` checked); returns them, in order, for
/// [`picked_saved`].
pub fn add_saved(entries: &mut Vec<Entry>, kind: Kind, shown: &str) -> Vec<(String, PathBuf)> {
    let saved = kind.saved();
    if !saved.is_empty() {
        entries.push(Entry::Separator);
        for (name, _) in &saved {
            entries.push(Entry::item(name.clone(), name == shown));
        }
    }
    saved
}

/// The saved preset a Preset pop-up pick `k` chose, when the pop-up had
/// `before` entries ahead of [`add_saved`]'s (whose separator comes
/// first): its file's bytes, and it becomes the current preset.
pub fn picked_saved(
    k: usize,
    before: usize,
    saved: &[(String, PathBuf)],
    kind: Kind,
) -> Option<Vec<u8>> {
    load_saved(kind, saved.get(k.checked_sub(before + 1)?)?)
}

/// A saved preset (name, path) read; it becomes the current preset.
pub fn load_saved(kind: Kind, (name, path): &(String, PathBuf)) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).ok()?;
    set_current(kind, Some(name.clone()), bytes.clone());
    Some(bytes)
}

/// The gear at `rect`: its menu (Save Preset..., Load Preset..., Delete
/// Current Preset, the last only while a saved preset is shown), and the
/// pick carried out. Returns a loaded preset's bytes for the dialog to
/// decode. `encoded` is None when the values can't be saved (an invalid
/// field).
pub fn gear(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: &str,
    kind: Kind,
    encoded: Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let response = ui.interact(rect, ui.id().with(id), egui::Sense::click());
    let can_delete = shown(kind, encoded.as_deref()).is_some();
    match gear_menu(ui, &response, can_delete)? {
        GearPick::Save => {
            let encoded = encoded?;
            let name = kind.save(&encoded)?;
            set_current(kind, Some(name), encoded);
            None
        }
        GearPick::Load => {
            let (name, bytes) = kind.load()?;
            set_current(kind, Some(name), bytes.clone());
            Some(bytes)
        }
        GearPick::Delete => {
            let name = current(kind).name?;
            if let Some((_, path)) = kind.saved().into_iter().find(|(n, _)| *n == name) {
                let _ = std::fs::remove_file(path);
            }
            set_current(kind, None, Vec::new());
            None
        }
    }
}

// ---- Photoshop's formats ----

fn put16(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&(v.clamp(i16::MIN as i32, u16::MAX as i32) as u16).to_be_bytes());
}

fn get16(b: &[u8], at: usize) -> Option<i32> {
    Some(i16::from_be_bytes([*b.get(at)?, *b.get(at + 1)?]) as i32)
}

fn getu16(b: &[u8], at: usize) -> Option<usize> {
    Some(u16::from_be_bytes([*b.get(at)?, *b.get(at + 1)?]) as usize)
}

/// Levels (.alv): version 2, 29 records of input black, input white,
/// output black, output white and gamma × 100 (composite, red, green,
/// blue, then unused ones), then "Lvls", version 3, the total count (62)
/// and the remaining 33 records. `levels` holds, per channel, input black,
/// gamma, input white, output black and output white, as the dialog does.
pub fn encode_levels(levels: &[[f32; 5]; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    put16(&mut out, 2);
    let record = |out: &mut Vec<u8>, l: Option<&[f32; 5]>| {
        let [ib, g, iw, ob, ow] = l.copied().unwrap_or([0.0, 1.0, 255.0, 0.0, 255.0]);
        for v in [ib, iw, ob, ow, g * 100.0] {
            put16(out, v.round() as i32);
        }
    };
    for k in 0..29 {
        record(&mut out, levels.get(k));
    }
    out.extend_from_slice(b"Lvls");
    put16(&mut out, 3);
    put16(&mut out, 62);
    for _ in 29..62 {
        record(&mut out, None);
    }
    out
}

pub fn decode_levels(b: &[u8]) -> Option<[[f32; 5]; 4]> {
    if !matches!(get16(b, 0)?, 1 | 2) {
        return None;
    }
    let mut out = [[0.0; 5]; 4];
    for (k, l) in out.iter_mut().enumerate() {
        let at = 2 + 10 * k;
        let v: Vec<i32> = (0..5)
            .map(|i| get16(b, at + 2 * i))
            .collect::<Option<_>>()?;
        *l = [
            v[0] as f32,
            v[4] as f32 / 100.0,
            v[1] as f32,
            v[2] as f32,
            v[3] as f32,
        ];
    }
    Some(out)
}

/// Curves (.acv): version 4, five curves (composite, red, green, blue and
/// an unused one), each its point count and (output, input) pairs.
/// `curves` are (input, output) points per channel.
pub fn encode_curves(curves: &[Vec<(f32, f32)>; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    put16(&mut out, 4);
    put16(&mut out, 5);
    let identity = vec![(0.0, 0.0), (255.0, 255.0)];
    for c in curves.iter().chain(std::iter::once(&identity)) {
        put16(&mut out, c.len() as i32);
        for &(i, o) in c {
            put16(&mut out, o.round() as i32);
            put16(&mut out, i.round() as i32);
        }
    }
    out
}

pub fn decode_curves(b: &[u8]) -> Option<[Vec<(f32, f32)>; 4]> {
    if !matches!(get16(b, 0)?, 1 | 4) {
        return None;
    }
    let count = getu16(b, 2)?;
    let mut at = 4;
    let mut out: [Vec<(f32, f32)>; 4] = Default::default();
    for curve in out.iter_mut().take(count) {
        let n = getu16(b, at)?;
        at += 2;
        let mut points = Vec::with_capacity(n);
        for _ in 0..n {
            let (o, i) = (get16(b, at)?, get16(b, at + 2)?);
            points.push((i as f32, o as f32));
            at += 4;
        }
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        *curve = points;
    }
    for c in out.iter_mut().filter(|c| c.len() < 2) {
        *c = vec![(0.0, 0.0), (255.0, 255.0)];
    }
    Some(out)
}

/// Hue/Saturation's settings as the file holds them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HueSaturation {
    pub colorize: bool,
    /// Colorize's hue (0–360), saturation and lightness.
    pub colorize_values: [i32; 3],
    /// Master, then Reds … Magentas: hue, saturation, lightness.
    pub values: [[i32; 3]; 7],
    /// The six ranges' bounds (degrees).
    pub bounds: [[i32; 4]; 6],
}

/// Hue/Saturation (.ahu): version 2, a Colorize byte and a pad, Colorize's
/// hue (−180–180), saturation and lightness, Master's three values, then
/// each range's four bounds and three values.
pub fn encode_hue_saturation(h: &HueSaturation) -> Vec<u8> {
    let mut out = Vec::new();
    put16(&mut out, 2);
    out.push(h.colorize as u8);
    out.push(0);
    let [ch, cs, cl] = h.colorize_values;
    for v in [if ch > 180 { ch - 360 } else { ch }, cs, cl] {
        put16(&mut out, v);
    }
    for v in h.values[0] {
        put16(&mut out, v);
    }
    for k in 0..6 {
        for v in h.bounds[k] {
            put16(&mut out, v);
        }
        for v in h.values[k + 1] {
            put16(&mut out, v);
        }
    }
    out
}

pub fn decode_hue_saturation(b: &[u8]) -> Option<HueSaturation> {
    if get16(b, 0)? != 2 {
        return None;
    }
    let colorize = *b.get(2)? != 0;
    let g = |k: usize| get16(b, 4 + 2 * k);
    let ch = g(0)?;
    let mut values = [[0; 3]; 7];
    values[0] = [g(3)?, g(4)?, g(5)?];
    let mut bounds = [[0; 4]; 6];
    for k in 0..6 {
        let at = 6 + 7 * k;
        bounds[k] = [g(at)?, g(at + 1)?, g(at + 2)?, g(at + 3)?];
        values[k + 1] = [g(at + 4)?, g(at + 5)?, g(at + 6)?];
    }
    Some(HueSaturation {
        colorize,
        colorize_values: [ch.rem_euclid(360), g(1)?, g(2)?],
        values,
        bounds,
    })
}

/// Channel Mixer (.cha): version 1, Monochrome, then four output channels
/// of four source amounts and a constant (RGB uses three of each). With
/// Monochrome the gray's mix is the first channel. `rows` are red, green,
/// blue and constant per output; `gray` likewise.
pub fn encode_channel_mixer(rows: &[[i32; 4]; 3], gray: &[i32; 4], monochrome: bool) -> Vec<u8> {
    let mut out = Vec::new();
    put16(&mut out, 1);
    put16(&mut out, monochrome as i32);
    let row = |out: &mut Vec<u8>, r: &[i32; 4]| {
        for v in [r[0], r[1], r[2], 0, r[3]] {
            put16(out, v);
        }
    };
    if monochrome {
        row(&mut out, gray);
        for _ in 0..3 {
            row(&mut out, &[0; 4]);
        }
    } else {
        for r in rows {
            row(&mut out, r);
        }
        row(&mut out, &[0; 4]);
    }
    out
}

/// (rows, gray, monochrome); a monochrome file's rows are the defaults.
pub fn decode_channel_mixer(b: &[u8]) -> Option<([[i32; 4]; 3], [i32; 4], bool)> {
    if get16(b, 0)? != 1 {
        return None;
    }
    let monochrome = get16(b, 2)? != 0;
    let row = |k: usize| -> Option<[i32; 4]> {
        let at = 4 + 10 * k;
        Some([
            get16(b, at)?,
            get16(b, at + 2)?,
            get16(b, at + 4)?,
            get16(b, at + 8)?,
        ])
    };
    if monochrome {
        let identity = [[100, 0, 0, 0], [0, 100, 0, 0], [0, 0, 100, 0]];
        Some((identity, row(0)?, true))
    } else {
        Some(([row(0)?, row(1)?, row(2)?], [40, 40, 20, 0], false))
    }
}

/// Exposure (.eap): version 1, then exposure, offset and gamma as
/// big-endian 32-bit floats.
pub fn encode_exposure(v: [f32; 3]) -> Vec<u8> {
    let mut out = vec![0, 1];
    for x in v {
        out.extend_from_slice(&x.to_be_bytes());
    }
    out
}

pub fn decode_exposure(b: &[u8]) -> Option<[f32; 3]> {
    if get16(b, 0)? != 1 {
        return None;
    }
    let f = |k: usize| -> Option<f32> {
        let s = b.get(2 + 4 * k..6 + 4 * k)?;
        Some(f32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    };
    Some([f(0)?, f(1)?, f(2)?])
}

/// Black & White (.blw): an action descriptor (version 16): the six
/// weights as longs (Rd, Yllw, Grn, Cyn, Bl, Mgnt), useTint, the tint
/// color as an RGBC object of doubles, bwPresetKind 3 and an empty
/// blackAndWhitePresetFileName.
pub fn encode_black_white(weights: [i32; 6], tint: bool, color: [u8; 3]) -> Vec<u8> {
    fn id(out: &mut Vec<u8>, key: &str) {
        if key.len() == 4 {
            out.extend_from_slice(&0u32.to_be_bytes());
        } else {
            out.extend_from_slice(&(key.len() as u32).to_be_bytes());
        }
        out.extend_from_slice(key.as_bytes());
    }
    fn header(out: &mut Vec<u8>, class: &str, count: u32) {
        // An empty Unicode name (one null character), the class, the count
        out.extend_from_slice(&1u32.to_be_bytes());
        out.extend_from_slice(&[0, 0]);
        id(out, class);
        out.extend_from_slice(&count.to_be_bytes());
    }
    let mut out = 16u32.to_be_bytes().to_vec();
    header(&mut out, "null", 10);
    for (key, w) in ["Rd  ", "Yllw", "Grn ", "Cyn ", "Bl  ", "Mgnt"]
        .iter()
        .zip(weights)
    {
        id(&mut out, key);
        out.extend_from_slice(b"long");
        out.extend_from_slice(&w.to_be_bytes());
    }
    id(&mut out, "useTint");
    out.extend_from_slice(b"bool");
    out.push(tint as u8);
    id(&mut out, "tintColor");
    out.extend_from_slice(b"Objc");
    header(&mut out, "RGBC", 3);
    for (key, c) in ["Rd  ", "Grn ", "Bl  "].iter().zip(color) {
        id(&mut out, key);
        out.extend_from_slice(b"doub");
        out.extend_from_slice(&(c as f64).to_be_bytes());
    }
    id(&mut out, "bwPresetKind");
    out.extend_from_slice(b"long");
    out.extend_from_slice(&3i32.to_be_bytes());
    id(&mut out, "blackAndWhitePresetFileName");
    out.extend_from_slice(b"TEXT");
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

/// (weights, use tint, tint color), read by key wherever they are.
pub fn decode_black_white(b: &[u8]) -> Option<([i32; 6], bool, [u8; 3])> {
    if b.get(0..4)? != 16u32.to_be_bytes() {
        return None;
    }
    let find = |key: &[u8], from: usize| -> Option<usize> {
        b.get(from..)?
            .windows(key.len())
            .position(|w| w == key)
            .map(|p| p + from + key.len())
    };
    let long = |at: usize| -> Option<i32> {
        Some(i32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
    };
    let mut weights = [0; 6];
    for (w, key) in weights.iter_mut().zip([
        b"Rd  long",
        b"Yllwlong",
        b"Grn long",
        b"Cyn long",
        b"Bl  long",
        b"Mgntlong",
    ]) {
        *w = long(find(key, 0)?)?;
    }
    let tint = *b.get(find(b"useTintbool", 0)?)? != 0;
    let objc = find(b"tintColorObjc", 0)?;
    let double = |key: &[u8]| -> Option<u8> {
        let at = find(key, objc)?;
        let v = f64::from_be_bytes(b.get(at..at + 8)?.try_into().ok()?);
        Some(v.round().clamp(0.0, 255.0) as u8)
    };
    let color = [
        double(b"Rd  doub")?,
        double(b"Grn doub")?,
        double(b"Bl  doub")?,
    ];
    Some((weights, tint, color))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Photoshop's shipped presets, when Photoshop is installed.
    fn photoshop(path: &str) -> Option<Vec<u8>> {
        std::fs::read(format!("/Applications/Adobe Photoshop 2026/Presets/{path}")).ok()
    }

    #[test]
    fn formats_round_trip() {
        let mut levels = [[0.0, 1.0, 255.0, 0.0, 255.0]; 4];
        levels[0] = [15.0, 1.2, 240.0, 5.0, 250.0];
        let b = encode_levels(&levels);
        assert_eq!(b.len(), 630);
        assert_eq!(decode_levels(&b), Some(levels));

        let curves: [Vec<(f32, f32)>; 4] = [
            vec![(0.0, 0.0), (130.0, 101.0), (255.0, 255.0)],
            vec![(0.0, 10.0), (255.0, 255.0)],
            vec![(0.0, 0.0), (255.0, 245.0)],
            vec![(0.0, 0.0), (255.0, 255.0)],
        ];
        assert_eq!(decode_curves(&encode_curves(&curves)), Some(curves));

        let mut hs = HueSaturation {
            colorize: true,
            colorize_values: [215, 25, 0],
            values: [[0; 3]; 7],
            bounds: [[315, 345, 15, 45]; 6],
        };
        hs.values[3] = [10, -20, 5];
        let b = encode_hue_saturation(&hs);
        assert_eq!(b.len(), 100);
        assert_eq!(decode_hue_saturation(&b), Some(hs));

        let rows = [[80, 10, 10, 0], [0, 100, 0, 5], [0, 0, 100, -5]];
        let b = encode_channel_mixer(&rows, &[40, 40, 20, 0], false);
        assert_eq!(b.len(), 44);
        assert_eq!(
            decode_channel_mixer(&b),
            Some((rows, [40, 40, 20, 0], false))
        );

        assert_eq!(
            decode_exposure(&encode_exposure([-1.0, 0.1, 1.2])),
            Some([-1.0, 0.1, 1.2])
        );

        let b = encode_black_white([0, 0, 0, 110, 110, 110], true, [225, 211, 179]);
        assert_eq!(
            decode_black_white(&b),
            Some(([0, 0, 0, 110, 110, 110], true, [225, 211, 179]))
        );
    }

    #[test]
    fn photoshops_own_presets_read() {
        // Skipped where Photoshop isn't installed
        if let Some(b) = photoshop("Levels/Darker.alv") {
            assert_eq!(
                decode_levels(&b).unwrap()[0],
                [15.0, 1.0, 255.0, 0.0, 255.0]
            );
            // Ours is byte for byte the same layout
            assert_eq!(encode_levels(&decode_levels(&b).unwrap()), b);
        }
        if let Some(b) = photoshop("Curves/Darker (RGB).acv") {
            let c = decode_curves(&b).unwrap();
            assert_eq!(c[0], vec![(0.0, 0.0), (130.0, 101.0), (255.0, 255.0)]);
            assert_eq!(encode_curves(&c), b);
        }
        if let Some(b) = photoshop("Hue and Saturation/Cyanotype.ahu") {
            let h = decode_hue_saturation(&b).unwrap();
            assert!(h.colorize);
            assert_eq!(h.colorize_values, [215, 25, 0]);
            assert_eq!(encode_hue_saturation(&h), b);
        }
        if let Some(b) = photoshop("Channel Mixer/Black & White with Blue Filter (RGB).cha") {
            let (_, gray, mono) = decode_channel_mixer(&b).unwrap();
            assert!(mono);
            assert_eq!(gray, [0, 0, 100, 0]);
        }
        if let Some(b) = photoshop("Exposure/Minus 1.0.eap") {
            assert_eq!(decode_exposure(&b), Some([-1.0, 0.0, 1.0]));
            assert_eq!(encode_exposure([-1.0, 0.0, 1.0]), b);
        }
        if let Some(b) = photoshop("Black and White/Blue Filter.blw") {
            let (w, tint, color) = decode_black_white(&b).unwrap();
            assert_eq!(w, [0, 0, 0, 110, 110, 110]);
            assert!(!tint);
            assert_eq!(color, [225, 211, 179]);
            assert_eq!(encode_black_white(w, tint, color).len(), b.len());
        }
    }

    #[test]
    fn saved_presets_are_listed_by_name() {
        let dir = std::env::temp_dir().join(format!("op-presets-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["b.alv", "A.alv", "c.acv"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        let names: Vec<String> = LEVELS.saved_in(&dir).into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, ["A", "b"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
