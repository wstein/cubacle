//! `cubacle photo`: PNG files of the six faces or one contact sheet.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cubacle-photo-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn photo(out: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .arg("photo")
        .args(args)
        .arg("--out")
        .arg(out)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .output()
        .unwrap()
}

/// Width and height from the IHDR chunk, after checking the signature.
fn dimensions(path: &Path) -> (u32, u32) {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(bytes[..8], SIGNATURE, "{}", path.display());
    assert_eq!(&bytes[12..16], b"IHDR");
    (
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
    )
}

#[test]
fn writes_one_png_per_face() {
    let out = dir("faces");
    let output = photo(&out, &["--scramble", "R U", "--pixels", "64"]);
    assert!(output.status.success(), "{output:?}");
    for face in ["U", "R", "F", "D", "L", "B"] {
        assert_eq!(dimensions(&out.join(format!("{face}.png"))), (64, 64));
    }
    let listed = String::from_utf8(output.stdout).unwrap();
    assert_eq!(listed.lines().count(), 6);
    assert!(listed.lines().next().unwrap().ends_with("U.png"));
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn writes_a_contact_sheet() {
    let out = dir("sheet");
    let output = photo(&out, &["--sheet", "--pixels", "32", "--scramble", "R U"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(dimensions(&out.join("sheet.png")), (128, 96));
    assert!(!out.join("U.png").exists());
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn the_same_seed_gives_identical_files_and_another_seed_differs() {
    let (a, b, c) = (dir("seed-a"), dir("seed-b"), dir("seed-c"));
    let args = |seed: &'static str| {
        [
            "--scramble",
            "R U F",
            "--pixels",
            "48",
            "--preset",
            "noise",
            "--seed",
            seed,
        ]
    };
    assert!(photo(&a, &args("5")).status.success());
    assert!(photo(&b, &args("5")).status.success());
    assert!(photo(&c, &args("6")).status.success());
    for face in ["U", "R", "F", "D", "L", "B"] {
        let name = format!("{face}.png");
        let first = std::fs::read(a.join(&name)).unwrap();
        assert_eq!(first, std::fs::read(b.join(&name)).unwrap(), "{name}");
        assert_ne!(first, std::fs::read(c.join(&name)).unwrap(), "{name}");
    }
    for d in [a, b, c] {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[test]
fn the_default_seed_is_fixed() {
    let (a, b) = (dir("default-a"), dir("default-b"));
    let args = ["--scramble", "R", "--pixels", "48", "--preset", "scatter"];
    assert!(photo(&a, &args).status.success());
    assert!(photo(&b, &args).status.success());
    assert_eq!(
        std::fs::read(a.join("F.png")).unwrap(),
        std::fs::read(b.join("F.png")).unwrap()
    );
    for d in [a, b] {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[test]
fn every_cube_size_from_2_to_7_works() {
    for n in 2..=7 {
        let out = dir(&format!("size{n}"));
        let output = photo(
            &out,
            &[
                "--scramble",
                "R",
                "--size",
                &n.to_string(),
                "--pixels",
                "64",
            ],
        );
        assert!(output.status.success(), "{n}: {output:?}");
        assert_eq!(dimensions(&out.join("R.png")), (64, 64));
        let _ = std::fs::remove_dir_all(&out);
    }
}

#[test]
fn takes_facelets_and_reports_a_bad_preset() {
    let out = dir("facelets");
    let solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = photo(&out, &[solved, "--pixels", "32"]);
    assert!(output.status.success(), "{output:?}");
    let bad = photo(&out, &[solved, "--preset", "real"]);
    assert!(!bad.status.success());
    let message = String::from_utf8(bad.stderr).unwrap();
    assert!(
        message.contains("Unknown synthetic preset: real"),
        "{message}"
    );
    let _ = std::fs::remove_dir_all(&out);
}
