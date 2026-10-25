//! `solve --to TARGET --now NOW`: the way back leads to the target.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The facelets of the solved cube of `size` after `moves`.
fn facelets(size: &str, moves: &str) -> String {
    let (ok, out, err) = cubacle(&["convert", "--size", size, "-s", moves]);
    assert!(ok, "{err}");
    out.trim().to_owned()
}

/// The words after `label` on the line that starts with it.
fn after<'a>(text: &'a str, label: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|line| line.strip_prefix(label))
        .map(str::trim)
}

/// The route of `solve --to`, a wrong copy of it with move `skip` (0-based)
/// left out and `extra` appended, and the answer of `--now`.
fn run(size: &str, scramble: &str, target: &str, skip: usize, extra: &str, more: &[&str]) {
    let (ok, route, err) = cubacle(&["solve", "--size", size, "-s", scramble, "--to", target]);
    assert!(ok, "{err}");
    let route: Vec<&str> = route.split_whitespace().collect();
    let mut done: Vec<&str> = route.clone();
    if skip < done.len() {
        done.remove(skip);
    }
    if !extra.is_empty() {
        done.push(extra);
    }
    let walked = format!("{scramble} {}", done.join(" "));
    let now = facelets(size, &walked);
    let mut args = vec!["solve", "--size", size, "-s", scramble, "--to", target];
    args.extend_from_slice(&["--now", &now]);
    args.extend_from_slice(more);
    let (ok, out, err) = cubacle(&args);
    assert!(ok, "{err}");
    assert!(
        out.contains("mistake at move") || out.contains("was skipped") || out.contains("extra"),
        "{out}"
    );
    // The moves printed, and the rest of the route when the way back rejoins it.
    let mut moves = after(&out, "Moves now:")
        .or_else(|| after(&out, "Fresh solution:"))
        .unwrap_or("")
        .to_owned();
    if let Some(line) = out
        .lines()
        .find(|l| l.starts_with("Then continue with move "))
    {
        let number: usize = line["Then continue with move ".len()..]
            .split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .expect("a move number");
        moves.push(' ');
        moves.push_str(&route[number - 1..].join(" "));
    }
    // A 2x2x2 may end turned as a whole.
    if size != "2" {
        let end = facelets(size, &format!("{walked} {moves}"));
        assert_eq!(end, facelets(size, target), "{out}");
    }
}

#[test]
fn a_skipped_move_towards_a_target_3x3() {
    run(
        "3",
        "R U2 F' D L2 B R' U' F2 D2 L B'",
        "R2 U2 F2 D",
        2,
        "",
        &[],
    );
}

#[test]
fn an_extra_move_at_the_end_towards_a_target_3x3() {
    run(
        "3",
        "R U2 F' D L2 B R' U' F2 D2 L B'",
        "R2 U2 F2 D",
        99,
        "U",
        &[],
    );
}

#[test]
fn recover_options_keep_their_meaning_with_a_target() {
    run(
        "3",
        "R U2 F' D L2 B R' U' F2 D2 L B'",
        "R2 U2 F2 D",
        2,
        "",
        &[
            "--one-mistake",
            "--at-move",
            "9",
            "--sure-until",
            "1",
            "--depth",
            "4",
        ],
    );
}

#[test]
fn a_skipped_move_towards_a_target_2x2() {
    run("2", "R U2 F' R2 U F", "R2 U2", 1, "", &[]);
}

#[test]
fn a_skipped_move_towards_a_target_4x4() {
    run("4", "R U 2R 2U' F Rw' D2 Lw", "R2 U2 F D'", 3, "", &[]);
}

#[test]
fn an_extra_move_at_the_end_towards_a_target_4x4() {
    run("4", "R U 2R 2U' F Rw' D2 Lw", "R2 U2 F D'", 99, "U", &[]);
}

#[test]
fn a_skipped_move_towards_a_target_5x5() {
    run("5", "R U 2R 2U' F Rw' D2", "R2 U2", 3, "", &[]);
}

#[test]
fn the_fresh_solution_leads_to_the_target() {
    let scramble = "R U2 F' D L2 B R' U' F2 D2 L B'";
    let now = facelets("3", &format!("{scramble} R U2 F2"));
    let (ok, out, err) = cubacle(&[
        "solve", "-s", scramble, "--to", "D2 F", "--now", &now, "--fresh",
    ]);
    assert!(ok, "{err}");
    let end = facelets("3", &format!("{scramble} R U2 F2 {}", out.trim()));
    assert_eq!(end, facelets("3", "D2 F"));
}

#[test]
fn a_solution_that_misses_the_target_is_refused() {
    let scramble = "R U2 F' D L2 B R' U' F2 D2 L B'";
    let now = facelets("3", &format!("{scramble} R"));
    let (ok, _, err) = cubacle(&[
        "solve",
        "-s",
        scramble,
        "--to",
        "D2 F",
        "--now",
        &now,
        "--solution",
        "R",
    ]);
    assert!(!ok);
    assert!(
        err.contains("does not lead the original cube to the target"),
        "{err}"
    );
}

#[test]
fn a_target_with_open_stickers_is_refused_with_now() {
    let scramble = "R U2 F' D";
    let now = facelets("3", scramble);
    let open = format!("X{}", &facelets("3", "")[1..]);
    let (ok, _, err) = cubacle(&["solve", "-s", scramble, "--to", &open, "--now", &now]);
    assert!(!ok);
    assert!(err.contains("without open stickers"), "{err}");
}

#[test]
fn the_other_conflicts_with_now_remain() {
    let (ok, _, err) = cubacle(&[
        "solve",
        "-s",
        "R U",
        "--to",
        "R",
        "--now",
        &facelets("3", "R U"),
        "--fast",
    ]);
    assert!(!ok);
    assert!(err.contains("cannot be used with '--fast'"), "{err}");
}
