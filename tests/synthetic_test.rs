//! The synthetic face generator: determinism, properties, and golden hashes
//! of this implementation (docs/specs/synthetic-face.md).
//!
//! The golden SHA-256 values pin the output of the current implementation.
//! Change them only by an explicit decision to change the photos.

use cubacle::synthetic::{render_face, Geometry, Logo, Options, Preset, Rgb};
use sha2::{Digest, Sha256};

fn grid(rows: &[&str]) -> Vec<Vec<char>> {
    rows.iter().map(|row| row.chars().collect()).collect()
}

fn sample() -> Vec<Vec<char>> {
    grid(&["WYO", "RGB", "BWR"])
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn render(preset: Preset, options: &Options) -> Vec<u8> {
    render_face(&sample(), preset, 7, 96, options).unwrap()
}

fn pixel(data: &[u8], size: usize, x: usize, y: usize) -> [u8; 4] {
    let i = (y * size + x) * 4;
    [data[i], data[i + 1], data[i + 2], data[i + 3]]
}

const GOLDEN_PRESETS: [(Preset, &str); 8] = [
    (
        Preset::Clean,
        "4d9b503091836ea63f2c2ed9ec34c091add6ac6b7fe092c2ff0d54c26adb0573",
    ),
    (
        Preset::Scatter,
        "3f709f97c4056c0de5456bc1bd12a2ed6422c1faa12a4ae159c14a9ed7540fc1",
    ),
    (
        Preset::Gradient,
        "f23736bc5cfb77c753810668d2acab5922ccc4f579d555e2564f7e6412ad11a3",
    ),
    (
        Preset::Shadow,
        "0a1b8fc051b2375e82cc9456b756b4ccd777acc2fe7720d0d1217e50cd588463",
    ),
    (
        Preset::DimWarm,
        "1f98a53f8b019a607bb07774676d8de1c969278c3f3ee4fcb827d7a5648e48fa",
    ),
    (
        Preset::Glare,
        "fcdfd41ee09a6601e137457afa6a99fc1ea5546a6359438121541f29a896bf3e",
    ),
    (
        Preset::Noise,
        "cd65835c69c5430e00bcf4f975eaffcd54b5e4042479ff82b57cf945b63d5a1f",
    ),
    (
        Preset::BadCamera,
        "90b48574998549e2b7d34acabe6dc2a295f08c920f66172d35a670a24f0c3a64",
    ),
];

#[test]
fn golden_hash_per_preset() {
    for (preset, expected) in GOLDEN_PRESETS {
        let hash = sha(&render(preset, &Options::default()));
        assert_eq!(hash, expected, "preset {preset}: {hash}");
    }
}

fn option_cases() -> Vec<(&'static str, Options, &'static str)> {
    vec![
        (
            "lightness",
            Options {
                lightness: Some(3.0),
                ..Options::default()
            },
            "2f2ea3b6573f940fc7a70bfc56928b8f439e3ac9d190c9af1a908b41dbfc0722",
        ),
        (
            "palette",
            Options {
                palette: [('W', Rgb::new(200.0, 40.0, 40.0))].into(),
                ..Options::default()
            },
            "f78cd40f2a239332ebb5cd46b73e5296a2807eaaf1c3f00a57997e084c5709f1",
        ),
        (
            "logo",
            Options {
                logo: Some(Logo {
                    row: 0,
                    col: 0,
                    color: 'B',
                    size: Some(0.5),
                }),
                ..Options::default()
            },
            "5496a1c073d3c35ccc3c54c3af272f0966421d511903c3d50af287d242e2e314",
        ),
        (
            "noise",
            Options {
                noise: Some(12.0),
                ..Options::default()
            },
            "b98ba789f869397a27a9a6c1159d8548f3fd778a9b6b658f12f11258a0ee4037",
        ),
        (
            "blur",
            Options {
                blur: Some(3.0),
                ..Options::default()
            },
            "fd8958b1bca27c15f2c7805e40e157f449063294b204536b0fab431a8f2a5395",
        ),
        (
            "brightness",
            Options {
                brightness: Some(0.5),
                ..Options::default()
            },
            "29326d83609e0e3dbab31c35de35e5eed86b38b65a2fe6ddaf0cfae81cb33bda",
        ),
        (
            "geometry",
            Options {
                geometry: Geometry {
                    margin: Some(0.06),
                    gap: Some(0.2),
                    radius: Some(0.3),
                },
                ..Options::default()
            },
            "ef7156ed98cf3f21912f2ff3316cda2f028888389d2bb4ec7b5df22f510f5c57",
        ),
    ]
}

#[test]
fn golden_hash_per_option() {
    for (name, options, expected) in option_cases() {
        let hash = sha(&render(Preset::Clean, &options));
        assert_eq!(hash, expected, "option {name}: {hash}");
    }
}

#[test]
fn same_inputs_same_bytes() {
    for preset in Preset::ALL {
        assert_eq!(
            render(preset, &Options::default()),
            render(preset, &Options::default())
        );
    }
}

#[test]
fn different_seeds_differ_for_every_random_preset() {
    for preset in [
        Preset::Scatter,
        Preset::DimWarm,
        Preset::Glare,
        Preset::Noise,
        Preset::BadCamera,
    ] {
        let a = render_face(&sample(), preset, 1, 96, &Options::default()).unwrap();
        let b = render_face(&sample(), preset, 2, 96, &Options::default()).unwrap();
        assert_ne!(a, b, "{preset}");
    }
}

#[test]
fn output_is_opaque_rgba_of_the_right_length() {
    for preset in Preset::ALL {
        let data = render(preset, &Options::default());
        assert_eq!(data.len(), 96 * 96 * 4);
        assert!(data.chunks(4).all(|p| p[3] == 255), "{preset}");
    }
}

#[test]
fn corner_is_backdrop_and_center_is_the_sticker() {
    let data = render(Preset::Clean, &Options::default());
    let corner = pixel(&data, 96, 1, 1);
    assert!(corner[0] < 30 && corner[1] < 30 && corner[2] < 40);
    // The middle sticker of the sample is green.
    let [r, g, b, _] = pixel(&data, 96, 48, 48);
    assert!(g > r && g > b, "{r},{g},{b}");
}

#[test]
fn the_label_is_drawn_in_the_bottom_margin() {
    let data = render_face(&sample(), Preset::Clean, 7, 256, &Options::default()).unwrap();
    let lit = (240..256)
        .flat_map(|y| (0..256).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(&data, 256, x, y)[0] > 120)
        .count();
    assert!(lit > 100, "{lit} lit pixels");
}

#[test]
fn unknown_preset_and_bad_input_are_errors() {
    assert_eq!(
        "real".parse::<Preset>().unwrap_err().to_string(),
        "Unknown synthetic preset: real"
    );
    let ragged = grid(&["WY", "R"]);
    assert!(render_face(&ragged, Preset::Clean, 1, 64, &Options::default()).is_err());
    assert!(render_face(&[], Preset::Clean, 1, 64, &Options::default()).is_err());
    assert!(render_face(&sample(), Preset::Clean, 1, 0, &Options::default()).is_err());
}

/// The nearest of the six palette colors to a pixel.
fn nearest(p: [u8; 4]) -> char {
    const PALETTE: [(char, [f64; 3]); 6] = [
        ('W', [232.0, 232.0, 224.0]),
        ('Y', [238.0, 206.0, 44.0]),
        ('O', [238.0, 122.0, 34.0]),
        ('R', [190.0, 36.0, 46.0]),
        ('G', [34.0, 150.0, 74.0]),
        ('B', [34.0, 84.0, 192.0]),
    ];
    let distance = |c: &[f64; 3]| {
        (0..3)
            .map(|i| (c[i] - f64::from(p[i])).powi(2))
            .sum::<f64>()
    };
    PALETTE
        .iter()
        .min_by(|a, b| distance(&a.1).total_cmp(&distance(&b.1)))
        .unwrap()
        .0
}

#[test]
fn clean_stickers_read_back_to_the_grid_for_2x2_to_7x7() {
    let letters = ['W', 'Y', 'O', 'R', 'G', 'B'];
    let size = 420;
    for n in 2..=7usize {
        let g: Vec<Vec<char>> = (0..n)
            .map(|row| {
                (0..n)
                    .map(|col| letters[(row * 5 + col * 3 + n) % 6])
                    .collect()
            })
            .collect();
        let data = render_face(&g, Preset::Clean, 42, size, &Options::default()).unwrap();
        // The margin is round(0.035 * 420) = 15 pixels; sample each center.
        let inner = size as f64 - 30.0;
        let cell = inner / n as f64;
        for (row, line) in g.iter().enumerate() {
            for (col, &letter) in line.iter().enumerate() {
                let x = (15.0 + (col as f64 + 0.5) * cell) as usize;
                let y = (15.0 + (row as f64 + 0.5) * cell) as usize;
                assert_eq!(
                    nearest(pixel(&data, size, x, y)),
                    letter,
                    "{n}x{n} {row},{col}"
                );
            }
        }
    }
}
