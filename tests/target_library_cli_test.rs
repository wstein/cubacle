//! `solve --to NAME`: a library pattern as the target.

use cubacle::patterns;
use cubacle::puzzle::{reaches, Facelets, MoveSequence, PuzzleKind};
use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        // the built-in library only: never the developer's config or home
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .env_remove("CUBACLE_CONFIG")
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn generator_state(name: &str) -> Facelets {
    let generator = patterns::find(name)
        .unwrap()
        .generator(3, None)
        .unwrap()
        .sequence()
        .unwrap();
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&generator)
        .unwrap()
}

#[test]
fn a_library_name_is_reached_exactly_with_any_spelling() {
    let target = generator_state("six-spots");
    let start = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"R U F2 D' L".parse().unwrap())
        .unwrap();
    for name in ["six-spots", "Six Spots", "SIX_SPOTS", "6 spots"] {
        let (ok, text, error) = cubacle(&["solve", "R U F2 D' L", "--to", name]);
        assert!(ok, "{name}: {error}");
        let moves: MoveSequence = text.lines().last().unwrap().parse().unwrap();
        assert!(reaches(&start, &moves, &target), "{name}: {text}");
    }
}

#[test]
fn an_unknown_name_lists_close_names_and_the_list_command() {
    let (ok, _, error) = cubacle(&["solve", "R U", "--to", "six-spotz"]);
    assert!(!ok);
    assert!(error.contains("--to"), "{error}");
    assert!(error.contains("six-spots"), "{error}");
    assert!(error.contains("cubacle patterns list"), "{error}");
}

#[test]
fn a_name_that_is_no_pattern_of_the_size_is_refused() {
    let (ok, _, error) = cubacle(&["solve", "--size", "2", "R U", "--to", "six-spots"]);
    assert!(!ok);
    assert!(error.contains("2x2x2"), "{error}");
}

#[test]
fn the_fixed_names_and_expressions_keep_their_meaning() {
    let (ok, _, _) = cubacle(&["solve", "R U", "--to", "superflip"]);
    assert!(ok);
    let (ok, _, _) = cubacle(&["solve", "R U", "--to", "F2 B2"]);
    assert!(ok);
}
