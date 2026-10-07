//! Compares filters with Photoshop's results on the same test patterns.
//! The patterns are made and filtered in Photoshop by a script; set
//! OPENPHOTO_FILTER_DIR to the folder of its PNGs. Ignored by default.

use op_core::filter::{self, Filter};
use op_core::{Color, Document};

/// The 64 × 64 test pattern: a background color with rectangles.
fn pattern(bg: [u8; 3], rects: &[(u32, u32, u32, u32, [u8; 3])]) -> Document {
    let mut doc =
        Document::new_with_background("t", 64, 64, Color::from_rgba8([bg[0], bg[1], bg[2], 255]));
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
        run(
            &format!("max_round_{r}"),
            B,
            &[(32, 32, 33, 33, W)],
            Filter::Maximum {
                radius,
                round: true,
            },
        );
        run(
            &format!("min_round_{r}"),
            W,
            &[(32, 32, 33, 33, B)],
            Filter::Minimum {
                radius,
                round: true,
            },
        );
    }
    // Mixed colors on gray: each neighbour pulled in by its coverage
    let mix = [
        (10, 10, 20, 20, [200, 60, 30]),
        (18, 14, 40, 30, [30, 180, 90]),
        (44, 40, 50, 56, [240, 240, 240]),
        (30, 44, 36, 46, [0, 0, 0]),
    ];
    let gray = [100; 3];
    run(
        "max_round_mix3",
        gray,
        &mix,
        Filter::Maximum {
            radius: 3.0,
            round: true,
        },
    );
    run(
        "min_round_mix3",
        gray,
        &mix,
        Filter::Minimum {
            radius: 3.0,
            round: true,
        },
    );
    run(
        "max_round_mix7",
        gray,
        &mix,
        Filter::Maximum {
            radius: 7.0,
            round: true,
        },
    );
}

/// The mixed pattern: rectangles of several colors and a white dot on gray.
const MIX: [(u32, u32, u32, u32, [u8; 3]); 5] = [
    (10, 10, 20, 20, [200, 60, 30]),
    (18, 14, 40, 30, [30, 180, 90]),
    (44, 40, 50, 56, [240, 240, 240]),
    (30, 44, 36, 46, [0, 0, 0]),
    (32, 32, 33, 33, [255, 255, 255]),
];

#[test]
#[ignore]
fn motion_blur_and_emboss_angles() {
    for angle in [0, 30, 45, 60, 90, -45, 15] {
        run(
            &format!("motion_{angle}_10"),
            [100; 3],
            &MIX,
            Filter::MotionBlur {
                angle,
                distance: 10,
            },
        );
    }
    for angle in [45, 135, 30, -60, 90, 0] {
        run(
            &format!("emboss_{angle}_3_100"),
            [100; 3],
            &MIX,
            Filter::Emboss {
                angle,
                height: 3,
                amount: 100,
            },
        );
    }
}

/// One pixel, raised in red and lowered in green, on mid gray: the filter's
/// impulse response.
const DOT: [(u32, u32, u32, u32, [u8; 3]); 1] = [(32, 32, 33, 33, [255, 0, 128])];

#[test]
#[ignore]
fn emboss_impulses() {
    for (angle, height) in [(45, 3), (30, 3), (20, 3), (-60, 3), (30, 5)] {
        run(
            &format!("imp_emboss_{angle}_{height}_100"),
            [128; 3],
            &DOT,
            Filter::Emboss {
                angle,
                height,
                amount: 100,
            },
        );
    }
}

#[test]
#[ignore]
fn motion_blur_impulses() {
    for (angle, distance) in [
        (10, 10),
        (15, 10),
        (30, 10),
        (45, 10),
        (60, 10),
        (45, 5),
        (30, 3),
        (30, 5),
        (30, 7),
        (45, 3),
        (20, 20),
        (45, 20),
        (5, 10),
        (80, 10),
        (30, 20),
        (30, 2),
        (30, 1),
        (-45, 10),
        (-30, 10),
        (-10, 10),
        (-60, 10),
        (-45, 5),
        (-30, 3),
        (-5, 10),
        (-80, 10),
        (1, 10),
        (89, 10),
        (-45, 20),
    ] {
        run(
            &format!("imp_motion_{angle}_{distance}"),
            [128; 3],
            &DOT,
            Filter::MotionBlur { angle, distance },
        );
    }
}

/// Add Noise is random in Photoshop, so its spread is compared: the
/// standard deviation of red on flat 50% gray, 256 × 256, ours against
/// Photoshop's.
#[test]
#[ignore]
fn add_noise_strength() {
    let Ok(dir) = std::env::var("OPENPHOTO_FILTER_DIR") else {
        return;
    };
    let std_dev = |px: &mut dyn Iterator<Item = u8>| {
        let v: Vec<f64> = px.map(|p| p as f64 - 128.0).collect();
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    };
    for amount in [5, 10, 25, 50, 100] {
        for (gaussian, kind) in [(false, "u"), (true, "g")] {
            let name = format!("noise_{kind}_{amount}");
            let Ok(ps) = image::open(format!("{dir}/{name}.png")) else {
                println!("{name}: no Photoshop file");
                continue;
            };
            let ps = std_dev(&mut ps.into_rgb8().pixels().map(|p| p.0[0]));
            let mut doc = Document::new_with_background(
                "t",
                256,
                256,
                Color::from_rgba8([128, 128, 128, 255]),
            );
            let f = Filter::AddNoise {
                amount: amount as f32,
                gaussian,
                monochromatic: false,
            };
            filter::apply(&mut doc, f, [255; 3]).unwrap();
            let ours = std_dev(&mut doc.composite_rgba8().chunks(4).map(|p| p[0]));
            println!("{name}: standard deviation {ours:.1}, Photoshop {ps:.1}");
        }
    }
}

/// Wind is random in Photoshop: this writes ours for the same 256 × 256
/// patterns (a white line at x 40 on dark gray; gray steps on the left
/// half, one 32-row band each) so their streaks can be counted side by
/// side (`photoshop_filters.md`).
#[test]
#[ignore]
fn wind_patterns() {
    let Ok(dir) = std::env::var("OPENPHOTO_FILTER_DIR") else {
        return;
    };
    let make = |rects: &[(u32, u32, u32, u32, u8)]| {
        let mut doc =
            Document::new_with_background("t", 256, 256, Color::from_rgba8([40, 40, 40, 255]));
        let id = doc.active_layer.unwrap();
        let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
        for &(x0, y0, x1, y1, v) in rects {
            for y in y0..y1 {
                for x in x0..x1 {
                    image.set_pixel(x, y, [v, v, v, 255]);
                }
            }
        }
        doc
    };
    let steps: Vec<_> = [60, 80, 100, 120, 140, 180, 220, 255]
        .iter()
        .enumerate()
        .map(|(i, &v)| (0, i as u32 * 32, 128, i as u32 * 32 + 32, v))
        .collect();
    for (k, method) in [
        filter::WindMethod::Wind,
        filter::WindMethod::Blast,
        filter::WindMethod::Stagger,
    ]
    .into_iter()
    .enumerate()
    {
        for (name, rects) in [
            ("line", vec![(40, 0, 41, 256, 255)]),
            ("steps", steps.clone()),
        ] {
            let mut doc = make(&rects);
            let f = Filter::Wind {
                method,
                from_left: true,
            };
            filter::apply(&mut doc, f, [255; 3]).unwrap();
            let px = doc.composite_rgba8();
            image::RgbaImage::from_raw(256, 256, px)
                .unwrap()
                .save(format!("{dir}/ours_wind_{name}_{k}_R.png"))
                .unwrap();
        }
    }
}
