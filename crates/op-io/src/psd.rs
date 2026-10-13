//! Photoshop documents (.psd): 8-bit RGB with pixel layers.
//!
//! Written per Adobe's "Photoshop File Formats Specification". Layers keep
//! their name, visibility, opacity, fill opacity and blend mode; channel
//! data is PackBits (RLE) compressed, and the merged image is stored too so
//! other applications can show the file.

use std::io::{Cursor, Read};

use op_core::{BlendMode, Color, Document, Layer, LayerKind, TiledImage};

use crate::IoError;

fn invalid(what: &str) -> IoError {
    IoError::Psd(what.to_string())
}

const BLEND_KEYS: &[(BlendMode, &[u8; 4])] = &[
    (BlendMode::Normal, b"norm"),
    (BlendMode::PassThrough, b"pass"),
    (BlendMode::Dissolve, b"diss"),
    (BlendMode::Darken, b"dark"),
    (BlendMode::Multiply, b"mul "),
    (BlendMode::ColorBurn, b"idiv"),
    (BlendMode::LinearBurn, b"lbrn"),
    (BlendMode::DarkerColor, b"dkCl"),
    (BlendMode::Lighten, b"lite"),
    (BlendMode::Screen, b"scrn"),
    (BlendMode::ColorDodge, b"div "),
    (BlendMode::LinearDodge, b"lddg"),
    (BlendMode::LighterColor, b"lgCl"),
    (BlendMode::Overlay, b"over"),
    (BlendMode::SoftLight, b"sLit"),
    (BlendMode::HardLight, b"hLit"),
    (BlendMode::VividLight, b"vLit"),
    (BlendMode::LinearLight, b"lLit"),
    (BlendMode::PinLight, b"pLit"),
    (BlendMode::HardMix, b"hMix"),
    (BlendMode::Difference, b"diff"),
    (BlendMode::Exclusion, b"smud"),
    (BlendMode::Subtract, b"fsub"),
    (BlendMode::Divide, b"fdiv"),
    (BlendMode::Hue, b"hue "),
    (BlendMode::Saturation, b"sat "),
    (BlendMode::Color, b"colr"),
    (BlendMode::Luminosity, b"lum "),
];

// Photoshop's version-16 artb descriptor. Kept separate from pixel records.
fn descriptor_id(out: &mut Writer, id: &str) {
    out.u32(if id.len() == 4 { 0 } else { id.len() as u32 });
    out.bytes(id.as_bytes());
}
fn descriptor_header(out: &mut Writer, class: &str, count: u32) {
    out.u32(1);
    out.u16(0); // empty terminated Unicode name
    descriptor_id(out, class);
    out.u32(count);
}
fn descriptor_number(out: &mut Writer, key: &str, value: f64) {
    descriptor_id(out, key);
    out.bytes(b"doub");
    out.bytes(&value.to_be_bytes());
}
fn write_artboard_descriptor(out: &mut Writer, b: op_core::artboard::Artboard) {
    out.bytes(b"8BIMartb");
    let size = out.length_slot();
    out.u32(16);
    descriptor_header(out, "artboard", 5);
    descriptor_id(out, "artboardRect");
    out.bytes(b"Objc");
    descriptor_header(out, "classFloatRect", 4);
    for (key, value) in [
        ("Top ", b.y),
        ("Left", b.x),
        ("Btom", b.y + b.height as i64),
        ("Rght", b.x + b.width as i64),
    ] {
        descriptor_number(out, key, value as f64);
    }
    descriptor_id(out, "guideIndeces");
    out.bytes(b"VlLs");
    out.u32(0);
    descriptor_id(out, "artboardPresetName");
    out.bytes(b"TEXT");
    out.u32(1);
    out.u16(0);
    descriptor_id(out, "Clr ");
    out.bytes(b"Objc");
    descriptor_header(out, "RGBC", 3);
    let [r, g, blue] = b.background.unwrap_or([255; 3]);
    for (key, value) in [("Rd  ", r), ("Grn ", g), ("Bl  ", blue)] {
        descriptor_number(out, key, value as f64);
    }
    descriptor_id(out, "artboardBackgroundType");
    out.bytes(b"long");
    out.u32(match b.background {
        None => 3,
        Some([255, 255, 255]) => 1,
        Some([0, 0, 0]) => 2,
        Some(_) => 4,
    });
    out.close(size, 2);
}

fn write_artboard_document_descriptor(out: &mut Writer, count: u32) {
    out.bytes(b"8BIMartd");
    let size = out.length_slot();
    out.u32(16);
    descriptor_header(out, "null", 9);
    descriptor_id(out, "Cnt ");
    out.bytes(b"long");
    out.u32(count);
    for key in ["autoExpandOffset", "origin"] {
        descriptor_id(out, key);
        out.bytes(b"Objc");
        descriptor_header(out, "Pnt ", 2);
        descriptor_number(out, "Hrzn", 0.0);
        descriptor_number(out, "Vrtc", 0.0);
    }
    for key in [
        "autoExpandEnabled",
        "autoNestEnabled",
        "autoPositionEnabled",
        "shrinkwrapOnSaveEnabled",
    ] {
        descriptor_id(out, key);
        out.bytes(b"bool");
        out.u8(1);
    }
    descriptor_id(out, "docDefaultNewArtboardBackgroundColor");
    out.bytes(b"Objc");
    descriptor_header(out, "RGBC", 3);
    for key in ["Rd  ", "Grn ", "Bl  "] {
        descriptor_number(out, key, 255.0);
    }
    descriptor_id(out, "docDefaultNewArtboardBackgroundType");
    out.bytes(b"long");
    out.u32(1);
    out.close(size, 2);
}

#[derive(Debug)]
enum DescriptorValue {
    Number(f64),
    Object(Vec<(String, DescriptorValue)>),
    Other,
}
fn read_descriptor_id(r: &mut Reader<'_>) -> Result<String, IoError> {
    let n = r.u32()? as usize;
    let n = if n == 0 { 4 } else { n };
    if n > 4096 {
        return Err(invalid("descriptor ID is too long"));
    }
    Ok(String::from_utf8_lossy(r.bytes(n)?).into_owned())
}
fn skip_descriptor_text(r: &mut Reader<'_>) -> Result<(), IoError> {
    let n = r.u32()? as usize;
    if n > 1_000_000 {
        return Err(invalid("descriptor text is too long"));
    }
    r.bytes(n * 2)?;
    Ok(())
}
fn read_descriptor(r: &mut Reader<'_>, depth: u8) -> Result<DescriptorValue, IoError> {
    if depth > 8 {
        return Err(invalid("descriptor nesting is too deep"));
    }
    skip_descriptor_text(r)?;
    read_descriptor_id(r)?;
    let count = r.u32()?;
    if count > 4096 {
        return Err(invalid("too many descriptor fields"));
    }
    let mut fields = Vec::new();
    for _ in 0..count {
        let key = read_descriptor_id(r)?;
        let kind = r.take::<4>()?;
        fields.push((key, read_descriptor_value(r, kind, depth + 1)?));
    }
    Ok(DescriptorValue::Object(fields))
}
fn read_descriptor_value(
    r: &mut Reader<'_>,
    kind: [u8; 4],
    depth: u8,
) -> Result<DescriptorValue, IoError> {
    if depth > 8 {
        return Err(invalid("descriptor nesting is too deep"));
    }
    Ok(match &kind {
        b"Objc" | b"GlbO" => read_descriptor(r, depth)?,
        b"long" => DescriptorValue::Number(r.i32()? as f64),
        b"doub" => DescriptorValue::Number(f64::from_be_bytes(r.take()?)),
        b"UntF" => {
            r.take::<4>()?;
            DescriptorValue::Number(f64::from_be_bytes(r.take()?))
        }
        b"bool" => {
            r.u8()?;
            DescriptorValue::Other
        }
        b"TEXT" => {
            skip_descriptor_text(r)?;
            DescriptorValue::Other
        }
        b"enum" => {
            read_descriptor_id(r)?;
            read_descriptor_id(r)?;
            DescriptorValue::Other
        }
        b"VlLs" => {
            let count = r.u32()?;
            if count > 4096 {
                return Err(invalid("descriptor list is too long"));
            }
            for _ in 0..count {
                let kind = r.take::<4>()?;
                read_descriptor_value(r, kind, depth + 1)?;
            }
            DescriptorValue::Other
        }
        b"tdta" => {
            let n = r.u32()? as usize;
            r.bytes(n)?;
            DescriptorValue::Other
        }
        _ => return Err(invalid("unsupported artboard descriptor value")),
    })
}
fn read_artboard_descriptor(data: &[u8]) -> Result<Option<op_core::artboard::Artboard>, IoError> {
    let mut r = Reader(Cursor::new(data));
    if r.u32()? != 16 {
        return Err(invalid("unsupported artboard descriptor version"));
    }
    let value = read_descriptor(&mut r, 0)?;
    let DescriptorValue::Object(fields) = value else {
        return Ok(None);
    };
    let find = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);
    let Some(DescriptorValue::Object(rect)) = find("artboardRect") else {
        return Ok(None);
    };
    let number = |items: &[(String, DescriptorValue)], key: &str| {
        items.iter().find_map(|(k, v)| match v {
            DescriptorValue::Number(n) if k == key && n.is_finite() => Some(*n),
            _ => None,
        })
    };
    let (Some(top), Some(left), Some(bottom), Some(right)) = (
        number(rect, "Top "),
        number(rect, "Left"),
        number(rect, "Btom"),
        number(rect, "Rght"),
    ) else {
        return Ok(None);
    };
    let background = match find("artboardBackgroundType") {
        Some(DescriptorValue::Number(n)) if *n == 3.0 => None,
        Some(DescriptorValue::Number(n)) if *n == 2.0 => Some([0; 3]),
        Some(DescriptorValue::Number(n)) if *n == 1.0 => Some([255; 3]),
        _ => match find("Clr ") {
            Some(DescriptorValue::Object(color)) => Some(
                [
                    number(color, "Rd  ").unwrap_or(255.0),
                    number(color, "Grn ").unwrap_or(255.0),
                    number(color, "Bl  ").unwrap_or(255.0),
                ]
                .map(|v| v.round().clamp(0.0, 255.0) as u8),
            ),
            _ => Some([255; 3]),
        },
    };
    let board = op_core::artboard::Artboard {
        x: left.round() as i64,
        y: top.round() as i64,
        width: (right - left).round().max(0.0) as u32,
        height: (bottom - top).round().max(0.0) as u32,
        background,
    };
    Ok(board.valid().then_some(board))
}

// ---- PackBits ----

fn packbits(row: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < row.len() {
        // A run of at least 3 equal bytes is worth encoding as a repeat
        let mut run = 1;
        while i + run < row.len() && run < 128 && row[i + run] == row[i] {
            run += 1;
        }
        if run >= 3 {
            out.push((257 - run) as u8);
            out.push(row[i]);
            i += run;
            continue;
        }
        let start = i;
        while i < row.len() && i - start < 128 {
            if i + 2 < row.len() && row[i] == row[i + 1] && row[i] == row[i + 2] {
                break;
            }
            i += 1;
        }
        out.push((i - start - 1) as u8);
        out.extend_from_slice(&row[start..i]);
    }
}

fn unpackbits(data: &[u8], len: usize) -> Result<Vec<u8>, IoError> {
    let mut out = Vec::with_capacity(len);
    let mut i = 0;
    while out.len() < len {
        let n = *data.get(i).ok_or_else(|| invalid("truncated RLE data"))? as i8;
        i += 1;
        if n >= 0 {
            let count = n as usize + 1;
            let bytes = data
                .get(i..i + count)
                .ok_or_else(|| invalid("truncated RLE data"))?;
            out.extend_from_slice(bytes);
            i += count;
        } else if n != -128 {
            let count = 1 - n as isize;
            let byte = *data.get(i).ok_or_else(|| invalid("truncated RLE data"))?;
            out.extend(std::iter::repeat_n(byte, count as usize));
            i += 1;
        }
    }
    out.truncate(len);
    Ok(out)
}

/// One channel (`rows` × `cols` bytes) RLE-compressed: the per-row byte
/// counts, then the rows.
fn rle_channel(plane: &[u8], rows: usize, cols: usize) -> (Vec<u16>, Vec<u8>) {
    let mut counts = Vec::with_capacity(rows);
    let mut data = Vec::new();
    for r in 0..rows {
        let before = data.len();
        packbits(&plane[r * cols..(r + 1) * cols], &mut data);
        counts.push((data.len() - before) as u16);
    }
    (counts, data)
}

fn slice_text(out: &mut Writer, text: &str) {
    let units: Vec<_> = text.encode_utf16().collect();
    out.u32(units.len() as u32);
    for unit in units {
        out.u16(unit);
    }
}
fn read_slice_text(r: &mut Reader) -> Result<String, IoError> {
    let n = r.u32()? as usize;
    if n > r.remaining() / 2 {
        return Err(invalid("invalid slice text length"));
    }
    let units = (0..n).map(|_| r.u16()).collect::<Result<Vec<_>, _>>()?;
    Ok(String::from_utf16_lossy(&units))
}
// Standard named path image resources, as specified by Adobe's PSD format.
pub(crate) fn validate_paths(doc: &Document) -> Result<(), IoError> {
    if doc.paths.len() > 998 {
        return Err(invalid("too many saved paths"));
    }
    for path in &doc.paths {
        if !path.name.is_ascii() || path.name.len() > 255 {
            return Err(invalid("path name must be ASCII and at most 255 bytes"));
        }
        for sub in &path.subpaths {
            if sub.knots.len() > u16::MAX as usize {
                return Err(invalid("too many Bezier knots"));
            }
            for knot in &sub.knots {
                for (x, y) in [knot.incoming, knot.anchor, knot.outgoing] {
                    if !x.is_finite()
                        || !y.is_finite()
                        || !(-16.0..16.0).contains(&(x / doc.width as f32))
                        || !(-16.0..16.0).contains(&(y / doc.height as f32))
                    {
                        return Err(invalid("path point exceeds PSD coordinate range"));
                    }
                }
            }
        }
    }
    Ok(())
}
fn write_paths(out: &mut Writer, doc: &Document) {
    for (index, path) in doc.paths.iter().enumerate() {
        out.bytes(b"8BIM");
        out.u16(2000 + index as u16);
        let name = path.name.as_bytes();
        out.u8(name.len() as u8);
        out.bytes(name);
        if (name.len() + 1) % 2 == 1 {
            out.u8(0);
        }
        let size = out.length_slot();
        out.u16(6);
        out.bytes(&[0; 24]);
        for sub in &path.subpaths {
            out.u16(if sub.closed { 0 } else { 3 });
            out.u16(sub.knots.len() as u16);
            out.bytes(&[0; 22]);
            for knot in &sub.knots {
                out.u16(match (sub.closed, knot.linked) {
                    (true, true) => 1,
                    (true, false) => 2,
                    (false, true) => 4,
                    (false, false) => 5,
                });
                for (x, y) in [knot.incoming, knot.anchor, knot.outgoing] {
                    out.i32((y as f64 / doc.height as f64 * 16777216.).round() as i32);
                    out.i32((x as f64 / doc.width as f64 * 16777216.).round() as i32);
                }
            }
        }
        out.close(size, 1);
    }
}
fn read_path(data: &[u8], name: String, w: u32, h: u32) -> Result<op_core::path::Path, IoError> {
    if !data.len().is_multiple_of(26) {
        return Err(invalid("invalid path record size"));
    }
    let mut r = Reader(Cursor::new(data));
    let mut path = op_core::path::Path {
        name,
        subpaths: vec![],
    };
    let mut remaining = 0usize;
    while r.remaining() > 0 {
        let selector = r.u16()?;
        match selector {
            0 | 3 => {
                if remaining != 0 {
                    return Err(invalid("short path subpath"));
                }
                remaining = r.u16()? as usize;
                r.bytes(22)?;
                path.subpaths.push(op_core::path::Subpath {
                    knots: vec![],
                    closed: selector == 0,
                });
            }
            1 | 2 | 4 | 5 => {
                let Some(sub) = path.subpaths.last_mut() else {
                    return Err(invalid("path knot without subpath"));
                };
                if remaining == 0 || sub.closed != (selector <= 2) {
                    return Err(invalid("invalid path knot selector"));
                }
                let mut point = || -> Result<(f32, f32), IoError> {
                    let y = r.i32()? as f64 / 16777216. * h as f64;
                    let x = r.i32()? as f64 / 16777216. * w as f64;
                    Ok((x as f32, y as f32))
                };
                sub.knots.push(op_core::path::Knot {
                    incoming: point()?,
                    anchor: point()?,
                    outgoing: point()?,
                    linked: selector == 1 || selector == 4,
                });
                remaining -= 1;
            }
            6 | 7 => {
                r.bytes(24)?;
            }
            8 => {
                if r.u16()? != 0 {
                    return Err(invalid("inverted path fill is unsupported"));
                }
                r.bytes(22)?;
            }
            _ => return Err(invalid("unknown path record")),
        }
    }
    if remaining != 0 {
        return Err(invalid("short path subpath"));
    }
    Ok(path)
}
fn write_counts(out: &mut Writer, doc: &Document) {
    if doc.count_groups.is_empty() {
        return;
    }
    out.bytes(b"8BIM");
    out.u16(4001);
    out.bytes(&[0, 0]);
    let size = out.length_slot();
    out.bytes(b"OPCT");
    out.u32(1);
    out.u32(doc.count_groups.len() as u32);
    for g in &doc.count_groups {
        out.u32(g.id);
        out.u8(u8::from(g.visible));
        out.bytes(&g.color);
        out.u32(g.marker);
        out.u32(g.label);
        slice_text(out, &g.name);
        out.u32(g.points.len() as u32);
        for &(x, y) in &g.points {
            out.u32(x);
            out.u32(y);
        }
    }
    out.close(size, 1);
    if (out.0.len() - size) % 2 == 1 {
        out.u8(0);
    }
}
fn read_counts(data: &[u8]) -> Result<Vec<op_core::annotation::CountGroup>, IoError> {
    let mut r = Reader(Cursor::new(data));
    if r.remaining() < 12 || &r.take::<4>()? != b"OPCT" || r.u32()? != 1 {
        return Ok(Vec::new());
    }
    let n = r.u32()? as usize;
    if n > r.remaining() / 24 {
        return Err(invalid("invalid count group length"));
    }
    let mut groups = Vec::new();
    for _ in 0..n {
        let id = r.u32()?;
        let visible = r.u8()? != 0;
        let color = r.take::<3>()?;
        let marker = r.u32()?.clamp(1, 100);
        let label = r.u32()?.clamp(1, 100);
        let name = read_slice_text(&mut r)?;
        let n = r.u32()? as usize;
        if n > r.remaining() / 8 {
            return Err(invalid("invalid count point length"));
        }
        let points = (0..n)
            .map(|_| Ok((r.u32()?, r.u32()?)))
            .collect::<Result<Vec<_>, IoError>>()?;
        groups.push(op_core::annotation::CountGroup {
            id,
            name,
            visible,
            color,
            marker,
            label,
            points,
        });
    }
    Ok(groups)
}
fn write_notes(out: &mut Writer, doc: &Document) {
    if doc.notes.is_empty() {
        return;
    }
    out.bytes(b"8BIM");
    out.u16(4000);
    out.bytes(&[0, 0]);
    let size = out.length_slot();
    out.bytes(b"OPNT");
    out.u32(1);
    out.u32(doc.notes.len() as u32);
    for n in &doc.notes {
        out.u32(n.id);
        out.u32(n.x);
        out.u32(n.y);
        out.bytes(&n.color);
        out.u8(0);
        slice_text(out, &n.author);
        slice_text(out, &n.text);
    }
    out.close(size, 1);
    if (out.0.len() - size) % 2 == 1 {
        out.u8(0);
    }
}
fn read_notes(data: &[u8]) -> Result<Vec<op_core::annotation::Note>, IoError> {
    let mut r = Reader(Cursor::new(data));
    if r.remaining() < 12 || &r.take::<4>()? != b"OPNT" || r.u32()? != 1 {
        return Ok(Vec::new());
    }
    let count = r.u32()? as usize;
    if count > r.remaining() / 24 {
        return Err(invalid("invalid note count"));
    }
    let mut notes = Vec::new();
    for _ in 0..count {
        let id = r.u32()?;
        let x = r.u32()?;
        let y = r.u32()?;
        let color = r.take::<3>()?;
        r.u8()?;
        let author = read_slice_text(&mut r)?;
        let text = read_slice_text(&mut r)?;
        notes.push(op_core::annotation::Note {
            id,
            x,
            y,
            color,
            author,
            text,
        });
    }
    Ok(notes)
}
fn write_slices(out: &mut Writer, doc: &Document) {
    if doc.slices.is_empty() {
        return;
    }
    out.bytes(b"8BIM");
    out.u16(1050);
    out.bytes(&[0, 0]);
    let length = out.length_slot();
    out.u32(6);
    for v in [0, 0, doc.height, doc.width] {
        out.u32(v);
    }
    slice_text(out, &doc.title);
    out.u32(doc.slices.len() as u32);
    for s in &doc.slices {
        out.u32(s.id as u32);
        out.u32(0);
        out.u32(2);
        slice_text(out, &s.name);
        out.u32(0);
        let (x0, y0, x1, y1) = s.bounds;
        for v in [x0, y0, x1, y1] {
            out.u32(v);
        }
        for text in [&s.url, &s.target, "", &s.alt] {
            slice_text(out, text);
        }
        out.u8(0);
        slice_text(out, "");
        out.u32(0);
        out.u32(0);
        out.bytes(&[0, 0, 0, 0]);
    }
    out.close(length, 1);
    if (out.0.len() - length) % 2 == 1 {
        out.u8(0);
    }
}
fn read_slices(bytes: &[u8]) -> Result<Vec<op_core::slice::Slice>, IoError> {
    let mut r = Reader(Cursor::new(bytes));
    if r.u32()? != 6 {
        return Ok(Vec::new());
    }
    for _ in 0..4 {
        r.u32()?;
    }
    read_slice_text(&mut r)?;
    let count = r.u32()? as usize;
    if count > r.remaining() / 69 {
        return Err(invalid("invalid slice count"));
    }
    let mut slices = Vec::new();
    for _ in 0..count {
        let id = r.u32()? as u64;
        r.u32()?;
        let origin = r.u32()?;
        if origin == 1 {
            r.u32()?;
        }
        let name = read_slice_text(&mut r)?;
        r.u32()?;
        let bounds = (r.u32()?, r.u32()?, r.u32()?, r.u32()?);
        let url = read_slice_text(&mut r)?;
        let target = read_slice_text(&mut r)?;
        read_slice_text(&mut r)?;
        let alt = read_slice_text(&mut r)?;
        r.u8()?;
        read_slice_text(&mut r)?;
        r.u32()?;
        r.u32()?;
        r.take::<4>()?;
        if origin == 2 && bounds.0 < bounds.2 && bounds.1 < bounds.3 {
            slices.push(op_core::slice::Slice {
                id,
                name,
                bounds,
                url,
                target,
                alt,
            });
        }
    }
    Ok(slices)
}

// ---- Writing ----

struct Writer(Vec<u8>);

impl Writer {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }
    fn i16(&mut self, v: i16) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }
    /// Writes a placeholder length and returns where to patch it.
    fn length_slot(&mut self) -> usize {
        self.u32(0);
        self.0.len()
    }
    /// Pads to a multiple of `align` from `start`, then patches the length.
    fn close(&mut self, start: usize, align: usize) {
        while !(self.0.len() - start).is_multiple_of(align) {
            self.0.push(0);
        }
        let len = (self.0.len() - start) as u32;
        self.0[start - 4..start].copy_from_slice(&len.to_be_bytes());
    }
}

/// What the layer records list, bottom to top: layers, and the divider
/// that opens each group (below its layers).
/// Resource 1026's numbers in record order: 1, 2, ... per link set (in
/// order of first appearance), 0 for unlinked layers and dividers.
fn link_numbers(doc: &Document) -> Vec<u16> {
    let mut sets: Vec<u32> = Vec::new();
    records(doc)
        .iter()
        .map(|record| match record {
            Record::Layer(l) if op_core::link::is_linked(doc, l.id) => {
                let link = l.link.expect("linked layers have a number");
                let k = sets.iter().position(|&s| s == link).unwrap_or_else(|| {
                    sets.push(link);
                    sets.len() - 1
                });
                k as u16 + 1
            }
            _ => 0,
        })
        .collect()
}

enum Record<'a> {
    Layer(&'a op_core::Layer),
    Divider,
}

/// The layer records in PSD order: before a group's bottom-most layer (or
/// the group itself when empty) comes its divider; for nested groups the
/// outer group's divider comes first.
fn records(doc: &Document) -> Vec<Record<'_>> {
    let index = |id| doc.layers.iter().position(|l| l.id == id);
    let depth = |id: op_core::LayerId| {
        let mut d = 0;
        let mut parent = doc.layer(id).and_then(|l| l.parent);
        while let Some(p) = parent {
            d += 1;
            parent = doc.layer(p).and_then(|l| l.parent);
        }
        d
    };
    // Each group's first record position, with its depth
    let mut starts: Vec<(usize, usize)> = doc
        .layers
        .iter()
        .filter(|g| g.is_group())
        .map(|g| {
            let first = doc
                .descendants(g.id)
                .into_iter()
                .filter_map(index)
                .chain(index(g.id))
                .min()
                .expect("the group itself");
            (first, depth(g.id))
        })
        .collect();
    starts.sort();
    let mut out = Vec::new();
    for (i, layer) in doc.layers.iter().enumerate() {
        for _ in starts.iter().filter(|(first, _)| *first == i) {
            out.push(Record::Divider);
        }
        out.push(Record::Layer(layer));
    }
    out
}

fn blend_key(mode: BlendMode) -> &'static [u8; 4] {
    for &(m, key) in BLEND_KEYS {
        if m == mode {
            return key;
        }
    }
    b"norm"
}

/// The four planes (alpha, red, green, blue) of a region of a layer, in
/// canvas coordinates (it can reach outside the canvas).
fn planes(image: &TiledImage, (x0, y0, x1, y1): (i64, i64, i64, i64)) -> [Vec<u8>; 4] {
    let n = ((x1 - x0) * (y1 - y0)) as usize;
    let mut p: [Vec<u8>; 4] = std::array::from_fn(|_| Vec::with_capacity(n));
    for y in y0..y1 {
        for x in x0..x1 {
            let [r, g, b, a] = image.pixel_at(x, y);
            p[0].push(a);
            p[1].push(r);
            p[2].push(g);
            p[3].push(b);
        }
    }
    p
}

pub fn write(doc: &Document) -> Vec<u8> {
    let (w, h) = (doc.width, doc.height);
    let mut out = Writer(Vec::new());
    // Only a background: Photoshop stores no layers, just the merged image
    let only_background = doc.layers.len() == 1 && doc.layers[0].is_background;
    let merged = doc.composite_rgba8();
    let opaque = merged.as_chunks::<4>().0.iter().all(|p| p[3] == 255);
    let merged_channels: u16 = if opaque { 3 } else { 4 };

    // Header
    out.bytes(b"8BPS");
    out.u16(1);
    out.bytes(&[0; 6]);
    out.u16(merged_channels);
    out.u32(h);
    out.u32(w);
    out.u16(8);
    out.u16(3); // RGB

    // Color mode data
    out.u32(0);

    // Image resources: resolution (ResolutionInfo, 1005)
    let resources = out.length_slot();
    out.bytes(b"8BIM");
    out.u16(1005);
    out.bytes(&[0, 0]); // empty name, padded to even
    out.u32(16);
    let fixed = (doc.resolution * 65536.0).round() as u32;
    for _ in 0..2 {
        out.u32(fixed);
        out.u16(1); // pixels per inch
        out.u16(1); // display unit: inches
    }
    // Linked layers (Layer Group Information, 1026): a u16 per layer record
    // (dividers too), bottom to top; layers sharing a number are linked
    let links = link_numbers(doc);
    if links.iter().any(|&n| n != 0) {
        out.bytes(b"8BIM");
        out.u16(1026);
        out.bytes(&[0, 0]);
        out.u32(links.len() as u32 * 2);
        links.iter().for_each(|&n| out.u16(n));
    }
    // Guides (Grid and Guides Information, 1032): positions in 1/32 px,
    // 0 for a vertical guide and 1 for a horizontal one
    if !doc.guides.is_empty() {
        out.bytes(b"8BIM");
        out.u16(1032);
        out.bytes(&[0, 0]);
        let size = 16 + doc.guides.len() as u32 * 5;
        out.u32(size);
        out.u32(1); // version
        out.u32(576); // grid cycles (Photoshop's defaults)
        out.u32(576);
        out.u32(doc.guides.len() as u32);
        for g in &doc.guides {
            out.u32((g.position * 32.0).round() as i32 as u32);
            out.bytes(&[if g.vertical { 0 } else { 1 }]);
        }
        if size % 2 == 1 {
            out.bytes(&[0]);
        }
    }
    write_slices(&mut out, doc);
    write_notes(&mut out, doc);
    write_counts(&mut out, doc);
    write_paths(&mut out, doc);
    out.close(resources, 2);

    // Layer and mask information
    let layer_and_mask = out.length_slot();
    if !only_background {
        let layer_info = out.length_slot();
        let records = records(doc);
        // Negative: the merged image's alpha holds the merged transparency
        let count = records.len() as i16;
        out.i16(if opaque { count } else { -count });
        let mut channel_data: Vec<Vec<(u16, Vec<u8>)>> = Vec::new();
        let divider = op_core::Layer::group(op_core::LayerId(0), "</Layer group>");
        for record in &records {
            let (layer, section) = match record {
                Record::Layer(layer) => (*layer, None),
                // Bottom of a group: an empty record named "</Layer group>"
                Record::Divider => (&divider, Some(3u32)),
            };
            let section = section.or(match layer.kind {
                LayerKind::Group { collapsed } => Some(if collapsed { 2 } else { 1 }),
                LayerKind::Raster(_) => None,
            });
            // The record covers the layer's non-transparent pixels,
            // including any outside the canvas (Photoshop keeps them too)
            let empty = TiledImage::new(0, 0);
            let image = layer.image().unwrap_or(&empty);
            let bounds = image.content_bounds();
            let (x0, y0, x1, y1) = bounds.unwrap_or((0, 0, 0, 0));
            out.i32(y0 as i32);
            out.i32(x0 as i32);
            out.i32(y1 as i32);
            out.i32(x1 as i32);
            // Photoshop stores the background without an alpha channel
            let ids: &[i16] = if layer.is_background {
                &[0, 1, 2]
            } else {
                &[-1, 0, 1, 2]
            };
            out.u16(ids.len() as u16 + u16::from(layer.mask.is_some()));
            let (rows, cols) = ((y1 - y0) as usize, (x1 - x0) as usize);
            let mut channels = Vec::new();
            let plane_data = bounds.map(|b| planes(image, b));
            for &id in ids {
                let k = (id + 1) as usize;
                let mut data = Writer(Vec::new());
                data.u16(1); // RLE
                if let Some(p) = &plane_data {
                    let (counts, bytes) = rle_channel(&p[k], rows, cols);
                    counts.iter().for_each(|&c| data.u16(c));
                    data.bytes(&bytes);
                }
                out.i16(id);
                out.u32(data.0.len() as u32);
                channels.push((id as u16, data.0));
            }
            // The layer mask (channel -2) over the whole canvas
            if let Some(mask) = &layer.mask {
                let plane: Vec<u8> = (0..h)
                    .flat_map(|y| (0..w).map(move |x| (x, y)))
                    .map(|(x, y)| mask.value(x, y))
                    .collect();
                let mut data = Writer(Vec::new());
                data.u16(1);
                let (counts, bytes) = rle_channel(&plane, h as usize, w as usize);
                counts.iter().for_each(|&c| data.u16(c));
                data.bytes(&bytes);
                out.i16(-2);
                out.u32(data.0.len() as u32);
                channels.push((-2i16 as u16, data.0));
            }
            channel_data.push(channels);

            out.bytes(b"8BIM");
            out.bytes(blend_key(layer.blend_mode));
            out.u8((layer.opacity * 255.0).round() as u8);
            out.u8(0); // clipping: base
            // Bit 3: bit 4 is meaningful (Photoshop 5 and later)
            let mut flags = 0b1000u8;
            if layer.is_background || layer.transparency_locked() {
                flags |= 1; // transparency protected
            }
            if !layer.visible {
                flags |= 2;
            }
            out.u8(flags);
            out.u8(0);
            let extra = out.length_slot();
            match &layer.mask {
                Some(mask) => {
                    // Layer mask data: its rectangle (the canvas), default
                    // color outside it, flags (bit 1: disabled), padding
                    out.u32(20);
                    out.i32(0);
                    out.i32(0);
                    out.i32(h as i32);
                    out.i32(w as i32);
                    out.u8(255);
                    out.u8(if mask.enabled { 0 } else { 2 });
                    out.bytes(&[0, 0]);
                }
                None => out.u32(0),
            }
            out.u32(0); // no blending ranges
            // Pascal name (MacRoman; non-ASCII becomes '?'), padded to 4
            let ascii: Vec<u8> = layer
                .name
                .chars()
                .take(255)
                .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
                .collect();
            let name_start = out.0.len();
            out.u8(ascii.len() as u8);
            out.bytes(&ascii);
            while !(out.0.len() - name_start).is_multiple_of(4) {
                out.u8(0);
            }
            // Unicode name
            out.bytes(b"8BIMluni");
            let luni = out.length_slot();
            let utf16: Vec<u16> = layer.name.encode_utf16().collect();
            out.u32(utf16.len() as u32);
            utf16.iter().for_each(|&c| out.u16(c));
            out.close(luni, 4);
            // The background's name comes from Photoshop ("bgnd")
            if layer.is_background {
                out.bytes(b"8BIMlnsr");
                out.u32(4);
                out.bytes(b"bgnd");
            }
            // The color label: its index, then 6 bytes of zeros
            if layer.color != op_core::LayerColor::None {
                out.bytes(b"8BIMlclr");
                out.u32(8);
                out.u16(layer.color.psd_index());
                out.bytes(&[0; 6]);
            }
            // The locks (`lspf`, as Photoshop 2026 writes them): 1
            // transparency, 2 pixels, 4 position, 8 auto-nesting, bit 31 all
            let locks = lock_bits(layer);
            if !layer.is_background && locks != 0 {
                out.bytes(b"8BIMlspf");
                out.u32(4);
                out.u32(locks);
            }
            if let Some(board) = layer.artboard {
                write_artboard_descriptor(&mut out, board);
            }
            if let Some(frame) = layer.frame {
                out.bytes(b"8BIMopfr");
                out.u32(24);
                out.u32(1);
                out.u32(u32::from(frame.elliptical));
                let (x0, y0, x1, y1) = frame.bounds;
                for v in [x0, y0, x1, y1] {
                    out.u32(v);
                }
            }
            // OpenPhoto mask-link extension; Adobe mask-link encoding is not verified.
            if let Some(mask) = &layer.mask {
                out.bytes(b"8BIMopml");
                out.u32(4);
                out.u32(u32::from(mask.linked));
            }
            // Fill opacity
            out.bytes(b"8BIMiOpa");
            out.u32(4);
            out.u8((layer.fill * 255.0).round() as u8);
            out.bytes(&[0; 3]);
            // Groups: the section type (1 open, 2 closed, 3 the bottom
            // divider), and for a folder its blend mode
            if let Some(kind) = section {
                out.bytes(b"8BIMlsct");
                if kind == 3 {
                    out.u32(4);
                    out.u32(3);
                } else {
                    out.u32(12);
                    out.u32(kind);
                    out.bytes(b"8BIM");
                    out.bytes(blend_key(layer.blend_mode));
                }
            }
            out.close(extra, 2);
        }
        for channels in channel_data {
            for (_, data) in channels {
                out.bytes(&data);
            }
        }
        out.close(layer_info, 2);
        out.u32(0); // global layer mask info
        let board_count = doc.layers.iter().filter(|l| l.artboard.is_some()).count() as u32;
        if board_count > 0 {
            write_artboard_document_descriptor(&mut out, board_count);
        }
    }
    out.close(layer_and_mask, 2);

    // Merged image: RLE, planes in order R, G, B (, A)
    out.u16(1);
    let (rows, cols) = (h as usize, w as usize);
    let mut all_counts = Vec::new();
    let mut all_data = Vec::new();
    for c in [0usize, 1, 2, 3].into_iter().take(merged_channels as usize) {
        let plane: Vec<u8> = merged.chunks(4).map(|p| p[c]).collect();
        let (counts, data) = rle_channel(&plane, rows, cols);
        all_counts.extend(counts);
        all_data.extend(data);
    }
    all_counts.iter().for_each(|&c| out.u16(c));
    out.bytes(&all_data);
    out.0
}

// ---- Reading ----

struct Reader<'a>(Cursor<&'a [u8]>);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], IoError> {
        let mut b = [0; N];
        self.0
            .read_exact(&mut b)
            .map_err(|_| invalid("unexpected end of file"))?;
        Ok(b)
    }
    fn u8(&mut self) -> Result<u8, IoError> {
        Ok(self.take::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, IoError> {
        Ok(u16::from_be_bytes(self.take()?))
    }
    fn i16(&mut self) -> Result<i16, IoError> {
        Ok(i16::from_be_bytes(self.take()?))
    }
    fn u32(&mut self) -> Result<u32, IoError> {
        Ok(u32::from_be_bytes(self.take()?))
    }
    fn i32(&mut self) -> Result<i32, IoError> {
        Ok(i32::from_be_bytes(self.take()?))
    }
    /// The bytes left after the current position.
    fn remaining(&self) -> usize {
        self.0.get_ref().len().saturating_sub(self.pos() as usize)
    }

    fn pos(&self) -> u64 {
        self.0.position()
    }
    fn seek(&mut self, pos: u64) {
        self.0.set_position(pos);
    }
    fn bytes(&mut self, n: usize) -> Result<&[u8], IoError> {
        let start = self.pos() as usize;
        let data = self.0.get_ref();
        let slice = data
            .get(start..start + n)
            .ok_or_else(|| invalid("unexpected end of file"))?;
        self.seek((start + n) as u64);
        Ok(slice)
    }
}

/// Decodes one channel stored with its own compression field.
fn read_channel(r: &mut Reader, len: usize, rows: usize, cols: usize) -> Result<Vec<u8>, IoError> {
    let end = r.pos() + len as u64;
    let compression = if len >= 2 { r.u16()? } else { 0 };
    let plane = match compression {
        0 => r.bytes(rows * cols)?.to_vec(),
        1 => {
            let mut counts = Vec::with_capacity(rows);
            for _ in 0..rows {
                counts.push(r.u16()? as usize);
            }
            let mut plane = Vec::with_capacity(rows * cols);
            for count in counts {
                let data = r.bytes(count)?;
                plane.extend(unpackbits(data, cols)?);
            }
            plane
        }
        2 | 3 => {
            let data = r.bytes(len.saturating_sub(2))?;
            unzip(data, rows * cols, (compression == 3).then_some(cols))?
        }
        _ => return Err(invalid("unknown channel compression")),
    };
    r.seek(end);
    Ok(plane)
}

/// ZIP channel data: a zlib stream of `size` bytes; with prediction
/// (`Some(cols)`) each row holds differences from the byte before.
fn unzip(data: &[u8], size: usize, prediction: Option<usize>) -> Result<Vec<u8>, IoError> {
    use std::io::Read;
    let mut out = Vec::with_capacity(size);
    flate2::read::ZlibDecoder::new(data)
        .take(size as u64)
        .read_to_end(&mut out)
        .map_err(|_| invalid("damaged ZIP data"))?;
    if out.len() < size {
        return Err(invalid("damaged ZIP data"));
    }
    if let Some(cols) = prediction.filter(|&c| c > 0) {
        for row in out.chunks_mut(cols) {
            for i in 1..row.len() {
                row[i] = row[i].wrapping_add(row[i - 1]);
            }
        }
    }
    Ok(out)
}

fn lock_bits(layer: &Layer) -> u32 {
    let mut bits = 0;
    for (on, bit) in [
        (layer.lock_transparency, 1),
        (layer.lock_pixels, 2),
        (layer.lock_position, 4),
        (layer.lock_nesting, 8),
        (layer.lock_all, 1 << 31),
    ] {
        if on {
            bits |= bit;
        }
    }
    bits
}

struct LayerRecord {
    /// The color label's index (`lclr`).
    label: u16,
    /// The lock bits (`lspf`), when saved.
    locks: Option<u32>,
    mask_linked: bool,
    artboard: Option<op_core::artboard::Artboard>,
    frame: Option<op_core::frame::Frame>,
    rect: (i32, i32, i32, i32),
    channels: Vec<(i16, usize)>,
    blend: [u8; 4],
    opacity: u8,
    flags: u8,
    name: String,
    fill: u8,
    /// The `lsct` section type of a group record (1 open, 2 closed, 3 divider).
    section: Option<u32>,
    mask: Option<MaskRecord>,
}

/// A layer mask's record: its rectangle (top, left, bottom, right), the
/// value outside it, and whether it is on.
struct MaskRecord {
    rect: (i32, i32, i32, i32),
    default: u8,
    enabled: bool,
}

pub fn read(data: &[u8], title: String) -> Result<Document, IoError> {
    let mut r = Reader(Cursor::new(data));
    if &r.take::<4>()? != b"8BPS" {
        return Err(invalid("not a Photoshop file"));
    }
    if r.u16()? != 1 {
        return Err(invalid("large documents (PSB) are not supported"));
    }
    r.take::<6>()?;
    let merged_channels = r.u16()? as usize;
    let h = r.u32()?;
    let w = r.u32()?;
    let depth = r.u16()?;
    let mode = r.u16()?;
    if depth != 8 || mode != 3 {
        return Err(invalid("only 8-bit RGB documents are supported"));
    }
    let skip = r.u32()? as u64;
    r.seek(r.pos() + skip);

    // Image resources: pick up the resolution
    let resources_len = r.u32()? as u64;
    let resources_end = r.pos() + resources_len;
    let mut resolution = 72.0;
    let mut links: Vec<u16> = Vec::new();
    let mut guides: Vec<op_core::Guide> = Vec::new();
    let mut count_groups = Vec::new();
    let mut notes = Vec::new();
    let mut slices = Vec::new();
    let mut paths = Vec::new();
    while r.pos() + 12 <= resources_end {
        if &r.take::<4>()? != b"8BIM" {
            break;
        }
        let id = r.u16()?;
        let name_len = r.u8()? as u64;
        let name = String::from_utf8_lossy(r.bytes(name_len as usize)?).into_owned();
        if (name_len + 1) % 2 == 1 {
            r.u8()?;
        }
        let size = r.u32()? as u64;
        let start = r.pos();
        if id == 1005 && size >= 4 {
            resolution = r.u32()? as f32 / 65536.0;
        }
        if id == 1026 {
            links = (0..size / 2).map(|_| r.u16()).collect::<Result<_, _>>()?;
        }
        if id == 1032 && size >= 16 {
            let _version = r.u32()?;
            let _grid = (r.u32()?, r.u32()?);
            let count = r.u32()? as u64;
            for _ in 0..count.min((size - 16) / 5) {
                let at = r.u32()? as i32 as f32 / 32.0;
                let direction = r.u8()?;
                guides.push(op_core::Guide {
                    vertical: direction == 0,
                    position: at,
                    color: None,
                });
            }
        }
        if (2000..=2997).contains(&id) || id == 1025 {
            r.seek(start);
            paths.push(read_path(r.bytes(size as usize)?, name, w, h)?);
        }
        if id == 4001 {
            r.seek(start);
            count_groups = read_counts(r.bytes(size as usize)?)?;
        }
        if id == 4000 {
            r.seek(start);
            notes = read_notes(r.bytes(size as usize)?)?;
        }
        if id == 1050 {
            r.seek(start);
            slices = read_slices(r.bytes(size as usize)?)?;
        }
        r.seek(start + size.next_multiple_of(2));
    }
    r.seek(resources_end);

    // Layers
    let lm_len = r.u32()? as u64;
    let lm_end = r.pos() + lm_len;
    let mut layers: Vec<Layer> = Vec::new();
    // Its placeholder layer is replaced below
    let mut doc = Document::new_with_background(title, w, h, Color::from_rgba8([0; 4]));
    doc.resolution = resolution;
    doc.guides = guides.clone();
    doc.slices = slices.clone();
    doc.notes = notes.clone();
    doc.count_groups = count_groups.clone();
    doc.paths = paths.clone();
    doc.active_path = (!doc.paths.is_empty()).then_some(0);
    if lm_len > 0 {
        let info_len = r.u32()? as u64;
        if info_len > 0 {
            let count = r.i16()?.unsigned_abs() as usize;
            let mut records = Vec::with_capacity(count);
            for _ in 0..count {
                let (top, left, bottom, right) = (r.i32()?, r.i32()?, r.i32()?, r.i32()?);
                let n = r.u16()? as usize;
                let mut channels = Vec::with_capacity(n);
                for _ in 0..n {
                    channels.push((r.i16()?, r.u32()? as usize));
                }
                if &r.take::<4>()? != b"8BIM" {
                    return Err(invalid("bad layer record"));
                }
                let blend = r.take::<4>()?;
                let opacity = r.u8()?;
                r.u8()?; // clipping
                let flags = r.u8()?;
                r.u8()?;
                let extra_len = r.u32()? as u64;
                let extra_end = r.pos() + extra_len;
                let mask_len = r.u32()? as u64;
                let mask_end = r.pos() + mask_len;
                let mask = if mask_len >= 18 {
                    let rect = (r.i32()?, r.i32()?, r.i32()?, r.i32()?);
                    let default = r.u8()?;
                    let flags = r.u8()?;
                    Some(MaskRecord {
                        rect,
                        default,
                        enabled: flags & 2 == 0,
                    })
                } else {
                    None
                };
                r.seek(mask_end);
                let ranges_len = r.u32()? as u64;
                r.seek(r.pos() + ranges_len);
                let name_start = r.pos();
                let name_len = r.u8()? as usize;
                let mut name = String::from_utf8_lossy(r.bytes(name_len)?).into_owned();
                r.seek(name_start + ((name_len as u64 + 1).next_multiple_of(4)));
                let mut fill = 255;
                // Group records: 1 open folder, 2 closed folder, 3 the
                // divider at the group's bottom (0: a normal layer)
                let mut section = None;
                let mut label = 0u16;
                let mut locks = None;
                let mut mask_linked = true;
                let mut artboard = None;
                let mut frame = None;
                while r.pos() + 12 <= extra_end {
                    let sig = r.take::<4>()?;
                    if &sig != b"8BIM" && &sig != b"8B64" {
                        break;
                    }
                    let key = r.take::<4>()?;
                    let len = r.u32()? as u64;
                    let start = r.pos();
                    match &key {
                        b"luni" => {
                            let chars = r.u32()? as usize;
                            let mut units = Vec::with_capacity(chars);
                            for _ in 0..chars {
                                units.push(r.u16()?);
                            }
                            name = String::from_utf16_lossy(&units);
                        }
                        b"iOpa" => fill = r.u8()?,
                        b"lclr" => label = r.u16()?,
                        b"lspf" => locks = Some(r.u32()?),
                        b"opfr" if len == 24 => {
                            let version = r.u32()?;
                            let elliptical = r.u32()? != 0;
                            let f = op_core::frame::Frame {
                                bounds: (r.u32()?, r.u32()?, r.u32()?, r.u32()?),
                                elliptical,
                            };
                            if version == 1 && f.valid(w, h) {
                                frame = Some(f);
                            }
                        }
                        b"opml" => mask_linked = r.u32()? != 0,
                        b"artb" => artboard = read_artboard_descriptor(r.bytes(len as usize)?)?,
                        b"opab" if len >= 20 => {
                            let x = r.i32()? as i64;
                            let y = r.i32()? as i64;
                            let width = r.u32()?;
                            let height = r.u32()?;
                            let background = r.take::<4>()?;
                            let b = op_core::artboard::Artboard {
                                x,
                                y,
                                width,
                                height,
                                background: (background[3] != 0).then_some([
                                    background[0],
                                    background[1],
                                    background[2],
                                ]),
                            };
                            if b.valid() {
                                artboard = Some(b);
                            }
                        }
                        b"lsct" | b"lsdk" => section = Some(r.u32()?),
                        _ => {}
                    }
                    r.seek(start + len);
                }
                r.seek(extra_end);
                records.push(LayerRecord {
                    rect: (top, left, bottom, right),
                    channels,
                    blend,
                    opacity,
                    flags,
                    name,
                    fill,
                    section,
                    mask,
                    label,
                    locks,
                    mask_linked,
                    artboard,
                    frame,
                });
            }
            let mut open_groups: Vec<usize> = Vec::new();
            for (i, rec) in records.iter().enumerate() {
                let (top, left, bottom, right) = rec.rect;
                let rows = (bottom - top).max(0) as usize;
                let cols = (right - left).max(0) as usize;
                let mut planes: [Option<Vec<u8>>; 4] = Default::default();
                let mut mask_plane = None;
                for &(id, len) in &rec.channels {
                    let slot = match id {
                        -1 => 0,
                        0..=2 => id as usize + 1,
                        -2 if rec.mask.is_some() => {
                            let (t, l, b, rt) = rec.mask.as_ref().expect("checked").rect;
                            let (mr, mc) = ((b - t).max(0) as usize, (rt - l).max(0) as usize);
                            mask_plane = Some(read_channel(&mut r, len, mr, mc)?);
                            continue;
                        }
                        // Vector and real user masks (-3): skipped
                        _ => {
                            r.seek(r.pos() + len as u64);
                            continue;
                        }
                    };
                    planes[slot] = Some(read_channel(&mut r, len, rows, cols)?);
                }
                // A group's bottom: its layers follow until its folder record
                if rec.section == Some(3) {
                    open_groups.push(layers.len());
                    continue;
                }
                let mut image = TiledImage::new(w, h);
                // Pixels outside the canvas are kept on the layer
                for y in 0..rows {
                    let dy = top as i64 + y as i64;
                    for x in 0..cols {
                        let dx = left as i64 + x as i64;
                        let k = y * cols + x;
                        let get = |s: usize, d: u8| planes[s].as_ref().map_or(d, |p| p[k]);
                        let px = [get(1, 0), get(2, 0), get(3, 0), get(0, 255)];
                        if px[3] > 0 {
                            image.set_pixel_at(dx, dy, px);
                        }
                    }
                }

                let mut layer = Layer::raster(doc.new_layer_id(), rec.name.clone(), image);
                if let (Some(m), Some(plane)) = (&rec.mask, &mask_plane) {
                    let (t, l, b, rt) = m.rect;
                    let cols = (rt - l).max(0) as i64;
                    let mut mask = op_core::LayerMask::from_values(w, h, |x, y| {
                        let (mx, my) = (x as i64 - l as i64, y as i64 - t as i64);
                        if mx < 0 || my < 0 || mx >= cols || my >= (b - t) as i64 {
                            m.default
                        } else {
                            plane[(my * cols + mx) as usize]
                        }
                    });
                    mask.enabled = m.enabled;
                    mask.linked = rec.mask_linked;
                    layer.mask = Some(mask);
                }
                layer.artboard = rec.artboard;
                layer.frame = rec.frame;
                layer.visible = rec.flags & 2 == 0;
                layer.opacity = rec.opacity as f32 / 255.0;
                layer.fill = rec.fill as f32 / 255.0;
                layer.color = op_core::LayerColor::from_psd_index(rec.label);
                layer.link = links.get(i).copied().filter(|&n| n != 0).map(u32::from);
                layer.blend_mode = BLEND_KEYS
                    .iter()
                    .find(|(_, k)| **k == rec.blend)
                    .map_or(BlendMode::Normal, |(m, _)| *m);
                // Photoshop's background: the bottom layer, transparency
                // protected and named "Background"
                // (no alpha channel, as Photoshop writes it, or named
                // "Background" with transparency protected)
                let no_alpha = !rec.channels.iter().any(|&(id, _)| id == -1);
                if i == 0 && rec.flags & 1 != 0 && (no_alpha || rec.name == "Background") {
                    layer.is_background = true;
                    // The background ends at the canvas
                    if let Some(image) = layer.image_mut() {
                        *image = image.clipped();
                    }
                } else {
                    // Photoshop also sets the record's "transparency
                    // protected" bit for locked pixels: `lspf` decides
                    // when it's there
                    let bits = rec.locks.unwrap_or(u32::from(rec.flags & 1));
                    layer.lock_transparency = bits & 1 != 0;
                    layer.lock_pixels = bits & 2 != 0;
                    layer.lock_position = bits & 4 != 0;
                    layer.lock_nesting = bits & 8 != 0;
                    layer.lock_all = bits & (1 << 31) != 0;
                }
                if let Some(kind @ (1 | 2)) = rec.section {
                    // The folder: the layers since its divider are in it
                    layer.kind = LayerKind::Group {
                        collapsed: kind == 2,
                    };
                    layer.lock_transparency = false;
                    let start = open_groups.pop().unwrap_or(0);
                    for child in &mut layers[start..] {
                        if child.parent.is_none() {
                            child.parent = Some(layer.id);
                        }
                    }
                }
                layers.push(layer);
            }
        }
    }
    r.seek(lm_end);

    if layers.is_empty() {
        // No layers: the merged image is the document
        let compression = r.u16()?;
        let (rows, cols) = (h as usize, w as usize);
        let channels = merged_channels.min(4);
        let mut planes = Vec::with_capacity(channels);
        match compression {
            0 => {
                for _ in 0..channels {
                    planes.push(r.bytes(rows * cols)?.to_vec());
                }
            }
            1 => {
                let mut counts = Vec::with_capacity(rows * merged_channels);
                for _ in 0..rows * merged_channels {
                    counts.push(r.u16()? as usize);
                }
                for c in 0..channels {
                    let mut plane = Vec::with_capacity(rows * cols);
                    for &count in &counts[c * rows..(c + 1) * rows] {
                        plane.extend(unpackbits(r.bytes(count)?, cols)?);
                    }
                    planes.push(plane);
                }
            }
            2 | 3 => {
                // One stream for all the channels
                let rest = r.bytes(r.remaining())?;
                let all = unzip(
                    rest,
                    rows * cols * merged_channels,
                    (compression == 3).then_some(cols),
                )?;
                for c in 0..channels {
                    planes.push(all[c * rows * cols..(c + 1) * rows * cols].to_vec());
                }
            }
            _ => return Err(invalid("unknown image data compression")),
        }
        let mut rgba = Vec::with_capacity(rows * cols * 4);
        for k in 0..rows * cols {
            for plane in &planes[..3] {
                rgba.push(plane[k]);
            }
            rgba.push(planes.get(3).map_or(255, |a| a[k]));
        }
        let mut doc = Document::from_rgba8(doc.title.clone(), w, h, &rgba);
        doc.resolution = resolution;
        doc.guides = guides;
        doc.slices = slices;
        doc.notes = notes;
        doc.count_groups = count_groups;
        doc.paths = paths;
        doc.active_path = (!doc.paths.is_empty()).then_some(0);
        return Ok(doc);
    }

    doc.active_layer = layers.last().map(|l| l.id);
    doc.layers = layers;
    doc.mark_dirty();
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_groups_round_trip_styles_visibility_and_points() {
        let mut d = Document::new_with_background("Counts", 32, 32, Color::WHITE);
        op_core::annotation::create_count_group(&mut d);
        let g = &mut d.count_groups[0];
        g.name = "计数 ✨".into();
        g.points = vec![(4, 8), (12, 20)];
        g.visible = false;
        g.color = [100, 200, 50];
        g.marker = 4;
        g.label = 12;
        let back = read(&write(&d), "Counts".into()).unwrap();
        assert_eq!(back.count_groups, d.count_groups);
        assert_eq!(back.composite_rgba8(), d.composite_rgba8());
    }

    #[test]
    fn notes_round_trip_unicode_text_and_positions() {
        let mut d = Document::new_with_background("Notes", 32, 32, Color::WHITE);
        op_core::annotation::create_note(&mut d, 5, 8, "作者");
        d.notes[0].text = "First line\n第二行 ✨".into();
        d.notes[0].color = [255, 200, 0];
        let back = read(&write(&d), "Notes".into()).unwrap();
        assert_eq!(back.notes, d.notes);
        assert_eq!(back.composite_rgba8(), d.composite_rgba8());
    }

    #[test]
    fn frame_geometry_mask_and_content_round_trip() {
        let mut doc = Document::new_with_background("Frame", 32, 32, Color::WHITE);
        let f = op_core::frame::Frame {
            bounds: (4, 4, 28, 28),
            elliptical: true,
        };
        let id = op_core::frame::create(&mut doc, f).unwrap();
        assert_eq!(doc.descendants(id).len(), 1);
        let back = read(&write(&doc), "Frame".into()).unwrap();
        assert_eq!(back.layers.iter().find_map(|l| l.frame), Some(f));
        assert_eq!(back.composite_rgba8(), doc.composite_rgba8());
    }

    #[test]
    fn slices_round_trip_unicode_and_metadata() {
        let mut doc = Document::new_with_background("Slices", 20, 20, Color::WHITE);
        let id = op_core::slice::create(&mut doc, (2, 3, 12, 13)).unwrap();
        let s = doc.slices.iter_mut().find(|s| s.id == id).unwrap();
        s.name = "测试 ✨".into();
        s.url = "https://example.com/photo".into();
        s.target = "_blank".into();
        s.alt = "Image".into();
        assert_eq!(
            read(&write(&doc), "read".into()).unwrap().slices,
            doc.slices
        );
        assert!(read_slices(&[0, 0, 0, 6]).is_err());
    }

    #[test]
    fn guides_round_trip() {
        let mut doc = Document::new_with_background("g", 50, 40, Color::WHITE);
        doc.guides = vec![
            op_core::Guide {
                vertical: true,
                position: 12.5,
                color: None,
            },
            op_core::Guide {
                vertical: false,
                position: 30.0,
                color: None,
            },
        ];
        let back = read(&write(&doc), "g".into()).unwrap();
        assert_eq!(back.guides, doc.guides);
    }

    #[test]
    fn zip_channels_with_and_without_prediction() {
        use std::io::Write;
        let plane: Vec<u8> = (0..12).map(|i| (i * 20) as u8).collect();
        let zip = |data: &[u8]| {
            let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            e.write_all(data).unwrap();
            e.finish().unwrap()
        };
        // Compression 2: the plane itself
        let mut bytes = 2u16.to_be_bytes().to_vec();
        bytes.extend(zip(&plane));
        let mut r = Reader(std::io::Cursor::new(&bytes[..]));
        assert_eq!(read_channel(&mut r, bytes.len(), 3, 4).unwrap(), plane);
        // Compression 3: each row's differences
        let deltas: Vec<u8> = plane
            .chunks(4)
            .flat_map(|row| {
                let mut d = vec![row[0]];
                d.extend(row.windows(2).map(|w| w[1].wrapping_sub(w[0])));
                d
            })
            .collect();
        let mut bytes = 3u16.to_be_bytes().to_vec();
        bytes.extend(zip(&deltas));
        let mut r = Reader(std::io::Cursor::new(&bytes[..]));
        assert_eq!(read_channel(&mut r, bytes.len(), 3, 4).unwrap(), plane);
        // A damaged stream is an error, not a panic
        let mut r = Reader(std::io::Cursor::new(&[0u8, 2, 1, 2, 3][..]));
        assert!(read_channel(&mut r, 5, 3, 4).is_err());
    }

    #[test]
    fn packbits_round_trip() {
        let row: Vec<u8> = [vec![7; 200], (0..=255).collect(), vec![1, 1, 2, 2, 2, 2]].concat();
        let mut packed = Vec::new();
        packbits(&row, &mut packed);
        assert!(packed.len() < row.len());
        assert_eq!(unpackbits(&packed, row.len()).unwrap(), row);
    }

    #[test]
    fn layers_round_trip() {
        let mut doc = Document::new_with_background("t.psd", 6, 4, Color::WHITE);
        let mut image = TiledImage::new(6, 4);
        image.set_pixel(2, 1, [255, 0, 0, 128]);
        image.set_pixel(4, 3, [0, 0, 255, 255]);
        let mut layer = Layer::raster(doc.new_layer_id(), "Ünïcode layer", image);
        layer.opacity = 0.5;
        layer.fill = 0.25;
        layer.blend_mode = BlendMode::Multiply;
        layer.visible = false;
        doc.insert_above_active(layer);
        doc.resolution = 300.0;

        let back = read(&write(&doc), "t.psd".into()).unwrap();
        assert_eq!((back.width, back.height, back.resolution), (6, 4, 300.0));
        assert_eq!(back.layers.len(), 2);
        assert!(back.layers[0].is_background);
        let l = &back.layers[1];
        assert_eq!(l.name, "Ünïcode layer");
        assert_eq!((l.opacity * 255.0).round(), 128.0);
        assert_eq!((l.fill * 255.0).round(), 64.0);
        assert_eq!(l.blend_mode, BlendMode::Multiply);
        assert!(!l.visible);
        let image = l.image().unwrap();
        assert_eq!(image.pixel(2, 1), [255, 0, 0, 128]);
        assert_eq!(image.pixel(4, 3), [0, 0, 255, 255]);
        assert_eq!(image.pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(back.active_layer, Some(l.id));
    }

    #[test]
    fn masks_round_trip() {
        let mut doc = Document::new_with_background("t.psd", 4, 3, Color::WHITE);
        let image = TiledImage::filled(4, 3, [255, 0, 0, 255]);
        let mut layer = Layer::raster(doc.new_layer_id(), "Masked", image);
        let mut mask = op_core::LayerMask::from_values(4, 3, |x, _| if x < 2 { 0 } else { 255 });
        mask.enabled = false;
        mask.linked = false;
        layer.mask = Some(mask);
        doc.insert_above_active(layer);
        let back = read(&write(&doc), "t.psd".into()).unwrap();
        let mask = back.layers[1].mask.as_ref().unwrap();
        assert!(!mask.enabled);
        assert!(!mask.linked);
        assert_eq!((mask.value(0, 1), mask.value(3, 2)), (0, 255));
    }

    #[test]
    fn a_lone_background_is_stored_as_the_merged_image() {
        let mut doc = Document::new_with_background("t.psd", 3, 2, Color::WHITE);
        let id = doc.active_layer.unwrap();
        let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
        image.set_pixel(1, 1, [10, 20, 30, 255]);
        let back = read(&write(&doc), "t.psd".into()).unwrap();
        assert_eq!(back.layers.len(), 1);
        assert!(back.layers[0].is_background);
        assert_eq!(&back.composite_rgba8()[16..20], [10, 20, 30, 255]);
    }

    #[test]
    fn rejects_other_files() {
        assert!(read(b"GIF89a....", "x".into()).is_err());
    }
}

#[cfg(test)]
mod photoshop_check {
    use super::*;

    /// Writes `target/psd-check/ours.psd` for opening in Photoshop.
    #[test]
    #[ignore]
    fn write_sample() {
        let mut doc = Document::new_with_background("ours.psd", 64, 48, Color::WHITE);
        let mut image = TiledImage::new(64, 48);
        // Reaches 26 px past the right edge and 5 px above the top
        for y in -5..30 {
            for x in 20..90 {
                image.set_pixel_at(x, y, [220, 30, 30, 255]);
            }
        }
        let mut layer = Layer::raster(doc.new_layer_id(), "Red Box", image);
        layer.opacity = 0.6;
        layer.blend_mode = BlendMode::Multiply;
        // A mask hiding the left of x 40
        let mut mask = op_core::LayerMask::filled(64, 48, 255);
        for y in 0..48 {
            for x in 0..40 {
                mask.image.set_pixel(x, y, [0, 0, 0, 255]);
            }
        }
        layer.mask = Some(mask);
        doc.insert_above_active(layer);
        // A blue box under a disabled mask that would hide it all
        let mut blue = TiledImage::new(64, 48);
        for y in 34..44 {
            for x in 4..60 {
                blue.set_pixel(x, y, [30, 30, 220, 255]);
            }
        }
        let mut layer = Layer::raster(doc.new_layer_id(), "Blue Bar", blue);
        let mut off = op_core::LayerMask::filled(64, 48, 0);
        off.enabled = false;
        layer.mask = Some(off);
        doc.insert_above_active(layer);
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/psd-check");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ours.psd"), write(&doc)).unwrap();
    }

    /// Reads a file Photoshop saved, given as OPENPHOTO_PSD.
    #[test]
    #[ignore]
    fn read_photoshop_file() {
        let path = std::env::var("OPENPHOTO_PSD").expect("OPENPHOTO_PSD");
        let doc = read(&std::fs::read(&path).unwrap(), "t".into()).unwrap();
        for l in &doc.layers {
            eprintln!(
                "{} bg={} visible={} opacity={} color={:?} group={} parent={:?}",
                l.name,
                l.is_background,
                l.visible,
                l.opacity,
                l.color,
                l.is_group(),
                l.parent.and_then(|p| doc.layer(p)).map(|p| p.name.clone())
            );
        }
        assert!(doc.layers[0].is_background);
    }

    #[test]
    fn pixels_outside_the_canvas_round_trip() {
        let mut doc = Document::new_with_background("t", 4, 3, Color::WHITE);
        let mut image = TiledImage::new(4, 3);
        image.set_pixel_at(-3, 1, [255, 0, 0, 255]);
        image.set_pixel_at(2, 1, [0, 255, 0, 255]);
        image.set_pixel_at(6, 5, [0, 0, 255, 255]);
        let id = doc.new_layer_id();
        doc.layers
            .push(op_core::Layer::raster(id, "Layer 1", image));
        let back = read(&write(&doc), "t.psd".into()).unwrap();
        let image = back.layers[1].image().unwrap();
        assert_eq!(image.content_bounds(), Some((-3, 1, 7, 6)));
        assert_eq!(image.pixel_at(-3, 1), [255, 0, 0, 255]);
        assert_eq!(image.pixel_at(6, 5), [0, 0, 255, 255]);
        assert_eq!(image.pixel(2, 1), [0, 255, 0, 255]);
    }

    #[test]
    fn color_labels_round_trip() {
        let mut doc = Document::new_with_background("t", 4, 3, Color::WHITE);
        let id = doc.new_layer_id();
        let mut layer = op_core::Layer::raster(id, "Layer 1", TiledImage::new(4, 3));
        layer.color = op_core::LayerColor::Violet;
        doc.layers.push(layer);
        let back = read(&write(&doc), "t.psd".into()).unwrap();
        assert_eq!(back.layers[0].color, op_core::LayerColor::None);
        assert_eq!(back.layers[1].color, op_core::LayerColor::Violet);
    }

    #[test]
    fn locks_round_trip() {
        let mut doc = Document::new_with_background("t", 4, 3, Color::WHITE);
        let all = op_core::Locks {
            transparency: true,
            pixels: false,
            position: true,
            nesting: true,
            all: true,
        };
        for locks in [op_core::Locks::default(), all] {
            let id = doc.new_layer_id();
            let mut layer = op_core::Layer::raster(id, "L", TiledImage::new(4, 3));
            layer.set_locks(locks);
            doc.layers.push(layer);
        }
        let back = read(&write(&doc), "t.psd".into()).unwrap();
        assert!(back.layers[0].is_background);
        assert_eq!(back.layers[1].locks(), op_core::Locks::default());
        assert_eq!(back.layers[2].locks(), all);
    }

    /// Saved by Photoshop 2026: one layer per lock, set in Photoshop.
    #[test]
    fn reads_photoshop_locks() {
        let bytes = include_bytes!("../fixtures/photoshop_locks.psd");
        let doc = read(bytes, "locks.psd".into()).unwrap();
        let locks = |name: &str| {
            let layer = doc.layers.iter().find(|l| l.name == name).unwrap();
            let l = layer.locks();
            (l.transparency, l.pixels, l.position, l.nesting, l.all)
        };
        assert_eq!(locks("tp"), (true, false, false, false, false));
        assert_eq!(locks("px"), (false, true, false, false, false));
        assert_eq!(locks("pos"), (false, false, true, false, false));
        assert_eq!(locks("nest"), (false, false, false, true, false));
        assert_eq!(locks("all"), (false, false, false, false, true));
        assert!(doc.layers[0].is_background);
    }

    /// Saved by Photoshop 2026: A and B linked, C and the background
    /// linked, D not linked.
    #[test]
    fn reads_photoshop_links() {
        let bytes = include_bytes!("../fixtures/photoshop_links.psd");
        let doc = read(bytes, "links.psd".into()).unwrap();
        let id = |name: &str| doc.layers.iter().find(|l| l.name == name).unwrap().id;
        let linked = |name: &str| {
            let mut names: Vec<String> = op_core::link::linked_with(&doc, id(name))
                .into_iter()
                .map(|i| doc.layer(i).unwrap().name.clone())
                .collect();
            names.sort();
            names
        };
        assert_eq!(linked("A"), ["B"]);
        assert_eq!(linked("C"), ["Background"]);
        assert!(linked("D").is_empty());
        // Written back with the same layers linked (numbered afresh)
        let again = read(&write(&doc), "t.psd".into()).unwrap();
        let sets = |d: &Document| {
            let index = |id| d.layers.iter().position(|l| l.id == id);
            d.layers
                .iter()
                .map(|l| {
                    let mut v: Vec<usize> = op_core::link::linked_with(d, l.id)
                        .into_iter()
                        .filter_map(index)
                        .collect();
                    v.sort();
                    v
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(sets(&again), sets(&doc));
    }

    #[test]
    fn groups_round_trip() {
        // Background, then Outer { Inner { a }, b }, then c on top
        let mut doc = Document::new_with_background("t", 4, 3, Color::WHITE);
        let ids: Vec<op_core::LayerId> = (0..5).map(|_| doc.new_layer_id()).collect();
        let (outer, inner, a, b, c) = (ids[0], ids[1], ids[2], ids[3], ids[4]);
        let pixel = |id, name: &str, parent| {
            let mut image = TiledImage::new(4, 3);
            image.set_pixel(1, 1, [9, 9, 9, 255]);
            let mut l = op_core::Layer::raster(id, name, image);
            l.parent = parent;
            l
        };
        let la = pixel(a, "a", Some(inner));
        let lb = pixel(b, "b", Some(outer));
        let lc = pixel(c, "c", None);
        let mut g_inner = op_core::Layer::group(inner, "Inner");
        g_inner.parent = Some(outer);
        if let LayerKind::Group { collapsed } = &mut g_inner.kind {
            *collapsed = true;
        }
        let mut g_outer = op_core::Layer::group(outer, "Outer");
        g_outer.opacity = 0.5;
        doc.layers.extend([la, g_inner, lb, g_outer, lc]);
        let back = read(&write(&doc), "t.psd".into()).unwrap();
        let names: Vec<&str> = back.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Background", "a", "Inner", "b", "Outer", "c"]);
        let by = |n: &str| back.layers.iter().find(|l| l.name == n).unwrap();
        assert_eq!(by("a").parent, Some(by("Inner").id));
        assert_eq!(by("Inner").parent, Some(by("Outer").id));
        assert_eq!(by("b").parent, Some(by("Outer").id));
        assert_eq!(by("Outer").parent, None);
        assert_eq!(by("c").parent, None);
        assert!(matches!(
            by("Inner").kind,
            LayerKind::Group { collapsed: true }
        ));
        assert!(matches!(
            by("Outer").kind,
            LayerKind::Group { collapsed: false }
        ));
        assert_eq!(by("Outer").blend_mode, BlendMode::PassThrough);
        assert!((by("Outer").opacity - 0.5).abs() < 0.01);
    }

    /// Writes target/psd-check/groups.psd to open in Photoshop.
    #[test]
    #[ignore]
    fn write_group_sample() {
        let mut doc = Document::new_with_background("groups.psd", 64, 48, Color::WHITE);
        let (outer, inner, a, b) = (
            doc.new_layer_id(),
            doc.new_layer_id(),
            doc.new_layer_id(),
            doc.new_layer_id(),
        );
        let mut image = TiledImage::new(64, 48);
        for y in 10..30 {
            for x in 10..30 {
                image.set_pixel(x, y, [220, 30, 30, 255]);
            }
        }
        let mut la = op_core::Layer::raster(a, "Red", image);
        la.parent = Some(inner);
        let mut lb = op_core::Layer::raster(b, "Empty", TiledImage::new(64, 48));
        lb.parent = Some(outer);
        let mut gi = op_core::Layer::group(inner, "Inner");
        gi.parent = Some(outer);
        let mut go = op_core::Layer::group(outer, "Outer");
        go.opacity = 0.6;
        doc.layers.extend([la, gi, lb, go]);
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/psd-check");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("groups.psd"), write(&doc)).unwrap();
    }
}

#[cfg(test)]
mod artboard_tests {
    use super::*;
    #[test]
    fn reads_artboards_from_independent_agpsd_fixture() {
        let bytes = include_bytes!("../fixtures/artboards_agpsd_31.psd");
        let doc = read(bytes, "independent".into()).unwrap();
        let board = |name: &str| {
            doc.layers
                .iter()
                .find(|l| l.name == name)
                .unwrap()
                .artboard
                .unwrap()
        };
        assert_eq!(board("White").background, Some([255; 3]));
        assert_eq!(board("Black").background, Some([0; 3]));
        assert_eq!(board("Transparent").background, None);
        assert_eq!(board("Custom").background, Some([20, 30, 40]));
        let pixels = doc.composite_rgba8();
        assert_eq!(&pixels[(2 * 40) * 4..(2 * 40) * 4 + 4], &[255; 4]);
        assert_eq!(pixels[(2 * 40 + 20) * 4 + 3], 0);
        assert_eq!(
            &pixels[(2 * 40 + 30) * 4..(2 * 40 + 30) * 4 + 4],
            &[20, 30, 40, 255]
        );
    }
    #[test]
    fn rejects_truncated_and_excessively_nested_artboard_descriptors() {
        assert!(read_artboard_descriptor(&[0, 0, 0, 16]).is_err());
        let mut out = Writer(Vec::new());
        out.u32(16);
        for _ in 0..10 {
            descriptor_header(&mut out, "artboard", 1);
            descriptor_id(&mut out, "nested");
            out.bytes(b"Objc");
        }
        assert!(read_artboard_descriptor(&out.0).is_err());
    }

    #[test]
    fn artboard_metadata_children_and_composite_round_trip() {
        let mut doc = Document::new_with_background("t", 16, 16, op_core::Color::WHITE);
        let child = doc.layers[0].id;
        let b = op_core::artboard::Artboard {
            x: 2,
            y: 3,
            width: 10,
            height: 8,
            background: Some([0; 3]),
        };
        let id = op_core::artboard::create(&mut doc, b, true).unwrap();
        assert_eq!(doc.layer(child).unwrap().parent, Some(id));
        let bytes = write(&doc);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/artboard-parity-check.psd");
        std::fs::write(path, &bytes).unwrap();
        assert!(bytes.windows(4).any(|window| window == b"artb"));
        let restored = read(&bytes, "round trip".into()).unwrap();
        let group = restored.layers.last().unwrap();
        assert_eq!(group.artboard, Some(b));
        assert_eq!(restored.layers[0].parent, Some(group.id));
        assert_eq!(restored.composite_rgba8(), doc.composite_rgba8());
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;
    use op_core::path::{Knot, Path, Subpath};
    #[test]
    fn standard_paths_preserve_cubic_controls_and_closed_flags() {
        let mut doc = Document::new_with_background("Paths", 128, 64, Color::WHITE);
        doc.paths.push(Path {
            name: "Outline".into(),
            subpaths: vec![
                Subpath {
                    closed: true,
                    knots: vec![
                        Knot {
                            incoming: (-4., 4.),
                            anchor: (20., 30.),
                            outgoing: (44., 56.),
                            linked: true,
                        },
                        Knot::corner((100., 50.)),
                    ],
                },
                Subpath {
                    closed: false,
                    knots: vec![Knot::corner((0., 0.)), Knot::corner((128., 64.))],
                },
            ],
        });
        validate_paths(&doc).unwrap();
        let data = write(&doc);
        let marker = data.windows(6).position(|b| b == b"8BIM\x07\xd0").unwrap();
        // Independent checks of Adobe record selector and normalized y-first coordinates.
        let start = marker + 6 + 8 + 4; // Pascal Outline occupies eight bytes.
        assert_eq!(&data[start..start + 2], &6u16.to_be_bytes());
        assert_eq!(&data[start + 26..start + 30], &[0, 0, 0, 2]);
        assert_eq!(&data[start + 52..start + 54], &1u16.to_be_bytes());
        assert_eq!(&data[start + 54..start + 58], &(1048576i32).to_be_bytes());
        let back = read(&data, "Back".into()).unwrap();
        assert_eq!(back.paths, doc.paths);
        assert_eq!(back.composite_rgba8(), doc.composite_rgba8());
        assert!(read_path(&[0; 25], "bad".into(), 128, 64).is_err());
        doc.paths[0].subpaths[0].knots[0].anchor.0 = f32::NAN;
        assert!(validate_paths(&doc).is_err());
    }
}
