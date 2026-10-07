//! Compares filters with Photoshop's results on the same test patterns.
//! The patterns are made and filtered in Photoshop by a script; set
//! OPENPHOTO_FILTER_DIR to the folder of its PNGs. Ignored by default.

use op_core::filter::{self, Filter};
use op_core::{Color, Document};

/// The 64 × 64 test pattern: a background color with rectangles.
fn pattern(bg: [u8; 3], rects: &[(u32, u32, u32, u32, [u8; 3])]) -> Document {
    let mut doc = Document::new_with_background("t", 64, 64, Color::from_rgba8([bg[0], bg[1], bg[2], 255]));
    let id = doc.active_layer.unwrap();
    let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
    for &(x0, y0, x1, y1, c) in rects {
        for y in y0..y1 {
            for x in x0..x1 {
                image.set_pixel(x, y, [c[0], c[1], c[2], 255]);
            }
        }
    }
    doc
}

/// The differences from Photoshop's file `name`: the largest and how many
/// channel values differ by more than one level.
fn compare(name: &str, doc: &Document) -> Option<(u8, usize)> {
    let dir = std::env::var("OPENPHOTO_FILTER_DIR").ok()?;
    let ps = image::open(format!("{dir}/{name}.png")).ok()?.into_rgb8();
    let ours = doc.composite_rgba8();
    let mut worst = 0u8;
    let mut off = 0;
    for (i, p) in ps.pixels().enumerate() {
        for c in 0..3 {
            let d = p.0[c].abs_diff(ours[i * 4 + c]);
            worst = worst.max(d);
            if d > 1 {
                off += 1;
            }
        }
    }
    Some((worst, off))
}

fn run(name: &str, bg: [u8; 3], rects: &[(u32, u32, u32, u32, [u8; 3])], f: Filter) {
    let mut doc = pattern(bg, rects);
    filter::apply(&mut doc, f, [255; 3]).unwrap();
    // Ours beside Photoshop's, for a closer look
    if let Ok(dir) = std::env::var("OPENPHOTO_FILTER_DIR") {
        let px = doc.composite_rgba8();
        let _ = image::RgbaImage::from_raw(64, 64, px)
            .map(|i| i.save(format!("{dir}/ours_{name}.png")));
    }
    match compare(name, &doc) {
        Some((worst, off)) => println!("{name}: worst {worst}, {off} values off by more than 1"),
        None => println!("{name}: no Photoshop file"),
    }
}

#[test]
#[ignore]
fn minimum_and_maximum_round() {
    const W: [u8; 3] = [255; 3];
    const B: [u8; 3] = [0; 3];
    for r in [1, 2, 3, 4, 5, 6, 8, 10] {
        let radius = r as f32;
        run(&format!("max_round_{r}"), B, &[(32, 32, 33, 33, W)], Filter::Maximum { radius, round: true });
        run(&format!("min_round_{r}"), W, &[(32, 32, 33, 33, B)], Filter::Minimum { radius, round: true });
    }
    // Mixed colors on gray: each neighbour pulled in by its coverage
    let mix = [
        (10, 10, 20, 20, [200, 60, 30]),
        (18, 14, 40, 30, [30, 180, 90]),
        (44, 40, 50, 56, [240, 240, 240]),
        (30, 44, 36, 46, [0, 0, 0]),
    ];
    let gray = [100; 3];
    run("max_round_mix3", gray, &mix, Filter::Maximum { radius: 3.0, round: true });
    run("min_round_mix3", gray, &mix, Filter::Minimum { radius: 3.0, round: true });
    run("max_round_mix7", gray, &mix, Filter::Maximum { radius: 7.0, round: true });
}
