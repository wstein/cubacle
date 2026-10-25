//! `solve --pattern-aware`: a library route only when it is strictly shorter.
//! Only the built-in library is used (a config that names nothing else).

use cubacle::patterns;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use std::process::Command;

const CONFIG: &str = concat!(env!("CARGO_TARGET_TMPDIR"), "/pattern-aware.toml");

struct Run {
    ok: bool,
    stdout: String,
    stderr: String,
}

impl Run {
    fn moves(&self) -> usize {
        self.stderr
            .lines()
            .find_map(|l| l.strip_prefix("moves "))
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("no moves line in {:?}", self.stderr))
    }

    fn pattern(&self) -> Option<&str> {
        self.stderr.lines().find(|l| l.starts_with("pattern: "))
    }
}

fn cubacle(args: &[&str]) -> Run {
    std::fs::write(CONFIG, "[patterns]\nfiles = [\"builtin\"]\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs");
    Run {
        ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

fn aware(args: &[&str]) -> Run {
    let mut all = vec!["solve", "--pattern-aware", "--config", CONFIG];
    all.extend(args);
    cubacle(&all)
}

fn plain(args: &[&str]) -> Run {
    let mut all = vec!["solve"];
    all.extend(args);
    cubacle(&all)
}

/// The shortest plain generator of the pattern on the size.
fn generator(id: &str, size: usize) -> String {
    patterns::find(id)
        .unwrap()
        .generators()
        .iter()
        .filter(|g| g.size == size)
        .filter_map(|g| g.sequence().ok())
        .min_by_key(MoveSequence::len)
        .unwrap()
        .to_string()
}

fn solves(size: usize, scramble: &str, answer: &str) -> bool {
    let kind = PuzzleKind::of_size(size).unwrap();
    let cube = Facelets::solved(kind)
        .apply(&scramble.parse().unwrap())
        .unwrap();
    cube.apply(&answer.parse().unwrap()).unwrap().is_solved()
}

#[test]
fn the_superflip_route_is_shorter_and_solves_the_cube() {
    let scramble = generator("superflip", 3);
    let without = plain(&["--size", "3", "-s", &scramble]);
    let with = aware(&["--size", "3", "-s", &scramble]);
    assert!(without.ok && with.ok, "{}", with.stderr);
    assert_eq!(without.moves(), 21);
    assert_eq!(with.moves(), 20);
    assert!(solves(3, &scramble, &with.stdout));
    let line = with.pattern().expect("the route is marked");
    assert!(line.contains("Superflip") && line.contains("1 move shorter"));
    assert_eq!(without.pattern(), None);
}

#[test]
fn never_longer_and_every_answer_solves_for_every_builtin_pattern() {
    for id in ["checkerboard", "six-spots", "four-spots", "cube-in-cube"] {
        let scramble = generator(id, 3);
        let without = plain(&["--size", "3", "-s", &scramble]);
        let with = aware(&["--size", "3", "-s", &scramble]);
        assert!(with.moves() <= without.moves(), "{id}");
        assert!(solves(3, &scramble, &with.stdout), "{id}");
        if with.pattern().is_none() {
            assert_eq!(with.stdout, without.stdout, "{id}");
        }
    }
}

#[test]
fn without_the_flag_the_answer_is_what_it_was() {
    let scramble = generator("superflip", 3);
    let without = plain(&["--size", "3", "-s", &scramble]);
    assert_eq!(without.moves(), 21);
    assert_eq!(
        without.stdout,
        "R L F U2 R2 U' D' F2 R' F B U L2 B2 D2 R2 D' L2 D B2 D"
    );
}

#[test]
fn a_random_scramble_gets_the_same_answer_with_and_without_the_flag() {
    let scramble = "R U2 F' L D2 B R' U F2 L' D B2 U' R2 F D' L2 B' U R";
    let without = plain(&["--size", "3", "-s", scramble]);
    let with = aware(&["--size", "3", "-s", scramble]);
    assert_eq!(with.stdout, without.stdout);
    assert_eq!(with.moves(), without.moves());
    assert_eq!(with.pattern(), None);
}

#[test]
fn the_2x2x2_is_not_touched() {
    let without = plain(&["--size", "2", "-s", "R U R' U' F2"]);
    let with = aware(&["--size", "2", "-s", "R U R' U' F2"]);
    assert!(with.ok);
    assert_eq!(with.stdout, without.stdout);
    assert_eq!(with.pattern(), None);
}

#[test]
fn a_4x4x4_pattern_is_never_longer() {
    let scramble = generator("superflip", 4);
    let without = plain(&["--size", "4", "-s", &scramble]);
    let with = aware(&["--size", "4", "-s", &scramble]);
    assert!(with.moves() <= without.moves());
    assert!(solves(4, &scramble, &with.stdout));
    assert!(with.pattern().is_some());
}

#[test]
fn with_a_target_the_relative_state_is_looked_up() {
    let scramble = format!("U {}", generator("superflip", 3));
    let without = plain(&["--size", "3", "--to", "U", "-s", &scramble]);
    let with = aware(&["--size", "3", "--to", "U", "-s", &scramble]);
    assert!(with.ok, "{}", with.stderr);
    assert!(with.moves() < without.moves());
    assert!(with.pattern().is_some());
    // the answer takes the start to the target
    let start = Facelets::solved(PuzzleKind::Cube3)
        .apply(&scramble.parse().unwrap())
        .unwrap();
    let target = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"U".parse().unwrap())
        .unwrap();
    assert_eq!(start.apply(&with.stdout.parse().unwrap()).unwrap(), target);
}

#[test]
fn the_flag_does_not_combine_with_a_restricted_move_set() {
    let run = aware(&["--size", "3", "--moves", "htm", "-s", "R U"]);
    assert!(!run.ok);
}
