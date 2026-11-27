//! The state formatter and `convert --to token2`: round trips through the
//! Orbit64 V2 library, agreement with V1 where the formats share tokens, and
//! the states a format cannot show.

use cubacle::generate::random_state;
use cubacle::notation::orbit64;
use cubacle::notation::orbit64::v2::{self, Model, Pose};
use cubacle::notation::{state_text, token_v2, StateFormat};
use cubacle::puzzle::{Facelets, PuzzleKind};
use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .env_remove("CUBACLE_CONFIG")
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn ok(args: &[&str]) -> String {
    let (success, out, err) = cubacle(args);
    assert!(success, "{args:?}: {err}");
    out
}

/// The facelets the token shows, decoded by the V2 library.
fn decoded(token: &str) -> String {
    v2::facelets::decode(token).expect("a V2 token").1
}

const SIZES: [usize; 6] = [2, 3, 4, 5, 6, 7];

#[test]
fn a_color_framed_token_decodes_to_the_facelets_it_was_made_from() {
    for kind in PuzzleKind::ALL {
        for _ in 0..4 {
            let cube = random_state(kind).expect("random state");
            let token = token_v2(&cube, Model::Color, Pose::Framed).unwrap();
            assert_eq!(decoded(&token), face_fields(kind.size(), &cube), "{kind:?}");
        }
    }
}

/// The V2 facelet text writes six face fields separated by one space.
fn face_fields(n: usize, cube: &Facelets) -> String {
    let text = cube.to_string();
    text.as_bytes()
        .chunks(n * n)
        .map(|field| std::str::from_utf8(field).unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_token_of_a_cube_in_another_orientation_keeps_the_orientation() {
    for size in SIZES {
        let n = size.to_string();
        // a whole-cube rotation after turns: the framed token must give back
        // the rotated stickers, not the canonical ones
        let expression = "R U x y2 z'";
        let facelets = ok(&["convert", "--size", &n, expression]);
        let token = ok(&["convert", "--size", &n, expression, "--to", "token2"]);
        let spaced: Vec<&str> = facelets
            .as_bytes()
            .chunks(size * size)
            .map(|field| std::str::from_utf8(field).unwrap())
            .collect();
        assert_eq!(decoded(&token), spaced.join(" "), "size {size}");
    }
}

#[test]
fn the_state_token_is_the_framed_color_v2_token() {
    for kind in PuzzleKind::ALL {
        for _ in 0..3 {
            let cube = random_state(kind).unwrap();
            let token = orbit64::encode(&cube).unwrap();
            assert_eq!(token_v2(&cube, Model::Color, Pose::Framed).unwrap(), token);
        }
    }
}

#[test]
fn the_5x5x5_framed_token_is_42_characters_and_reads_back() {
    let cube = random_state(PuzzleKind::Cube5).unwrap();
    let token = orbit64::encode(&cube).unwrap();
    assert_eq!(token, token_v2(&cube, Model::Color, Pose::Framed).unwrap());
    assert_eq!(token.len(), 42);
    assert!(orbit64::decode(&"A".repeat(43)).is_err());
    assert_eq!(decoded(&token), face_fields(5, &cube));
}

#[test]
fn the_canonical_pose_gives_one_token_for_cubes_that_differ_by_a_rotation() {
    for size in ["3", "4"] {
        let plain = ok(&[
            "convert",
            "--size",
            size,
            "R U R' F",
            "--to",
            "token2",
            "--pose",
            "canonical",
        ]);
        let turned = ok(&[
            "convert",
            "--size",
            size,
            "R U R' F x y",
            "--to",
            "token2",
            "--pose",
            "canonical",
        ]);
        assert_eq!(plain, turned, "size {size}");
        let framed = ok(&["convert", "--size", size, "R U R' F", "--to", "token2"]);
        let framed_turned = ok(&["convert", "--size", size, "R U R' F x y", "--to", "token2"]);
        assert_ne!(framed, framed_turned, "size {size}");
    }
}

#[test]
fn the_formatter_writes_every_format() {
    let cube = random_state(PuzzleKind::Cube3).unwrap();
    let text = |format| state_text(&cube, format, None).unwrap();
    assert_eq!(text(StateFormat::None), None);
    assert_eq!(text(StateFormat::Facelets), Some(cube.to_string()));
    assert_eq!(
        text(StateFormat::Orbit64),
        Some(orbit64::encode(&cube).unwrap())
    );
    assert_eq!(
        text(StateFormat::Orbit64V2),
        Some(token_v2(&cube, Model::Color, Pose::Framed).unwrap())
    );
    let colors = text(StateFormat::Color).unwrap();
    assert_eq!(colors.len(), 54);
    assert!(colors.chars().all(|c| "WRGYOB".contains(c)));
    for name in ["orbit64v2", "orbit64", "facelets", "color", "none"] {
        assert_eq!(name.parse::<StateFormat>().unwrap().to_string(), name);
    }
    assert!("v2".parse::<StateFormat>().is_err());
}

#[test]
fn a_picture_token_of_the_solved_3x3x3_reads_as_the_solved_picture_cube() {
    let cube = Facelets::solved(PuzzleKind::Cube3);
    for pose in [Pose::Canonical, Pose::Framed] {
        let token = token_v2(&cube, Model::Picture, pose).unwrap();
        let (entry, state) = v2::decode(&token).unwrap();
        assert_eq!(
            (entry.n, entry.model, entry.pose),
            (3, Model::Picture, pose)
        );
        let (colors, twists) = v2::facelets::to_picture_parts(3, pose, &state).unwrap();
        assert_eq!(colors, cube);
        assert!(twists.is_untwisted());
    }
    assert_eq!(
        ok(&["convert", "--size", "3", "U U'", "--to", "token2", "--model", "picture"]),
        token_v2(&cube, Model::Picture, Pose::Framed).unwrap()
    );
}

#[test]
fn a_picture_token_that_colors_cannot_determine_is_an_error_with_the_reason() {
    let five = random_state(PuzzleKind::Cube5).unwrap();
    let why = token_v2(&five, Model::Picture, Pose::Framed).unwrap_err();
    assert!(why.contains("5x5x5") && why.contains("picture"), "{why}");
    let two = Facelets::solved(PuzzleKind::Cube2);
    assert!(token_v2(&two, Model::Picture, Pose::Framed)
        .unwrap_err()
        .contains("2x2x2"));
    // one quarter turn is an odd corner permutation: turn 0 on every center fails
    let (success, _, err) = cubacle(&[
        "convert", "--size", "3", "R", "--to", "token2", "--model", "picture",
    ]);
    assert!(!success);
    assert!(err.contains("odd corner permutation"), "{err}");
}

#[test]
fn model_and_pose_need_token2() {
    let (success, _, err) = cubacle(&["convert", "--size", "3", "R", "--model", "picture"]);
    assert!(!success);
    assert!(err.contains("--to token2"), "{err}");
    let (success, _, err) = cubacle(&[
        "convert",
        "--size",
        "3",
        "R",
        "--to",
        "token",
        "--pose",
        "canonical",
    ]);
    assert!(!success);
    assert!(err.contains("--to token2"), "{err}");
}

#[test]
fn token2_is_the_default_color_framed_token_for_the_reference_scrambles() {
    // Vectors written from the V2 library (v2::facelets::encode of the
    // facelets of the scramble), recorded here as literals.
    for (size, scramble, token) in REFERENCE {
        let n = size.to_string();
        assert_eq!(
            ok(&["convert", "--size", &n, scramble, "--to", "token2"]),
            *token,
            "{size} {scramble}"
        );
    }
}

const REFERENCE: &[(usize, &str, &str)] = &[
    (2, "R U R' F", "DBqzR"),
    (3, "R U R' U'", "HYKC9_HL1gAA"),
    (3, "R U x y2", "AwtB4B1fWQAG"),
    (4, "R U Rw F", "BHjaL7hOcxS6-MAeub6GYss2B5k"),
    (5, "R U Rw 3Rw' F x", "Jx93AAeuXDzNpiFevFGqGquAJRbGFTWfFpLhZ9fjMQ"),
    (6, "R U Rw 3Rw' F", "A3DZCufOaL8X2ds_Dl7hUb4pQc_RxaIYa6ZQKE-YTiz1nyjuH3Y38j8YNek-Kq8EVU"),
    (7, "R U Rw 3Rw' F z", "EyU924LQGFKDRWABhjbvSPYl0QNF_W8deVNXLb0AdnlbbrpTXaXssqxs5cdZL1_keYcbb0s2rDQy_7L6Sp4g4Da-mg"),
];
