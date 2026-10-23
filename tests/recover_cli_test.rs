use std::process::Command;

const ORIGINAL: &str = "R U2 F' D L2 B R' U' F2 D2 L B'";
/// The original cube after `B L' U2 B2 U2 B2 D2`: step 5 (`D'`) was skipped.
const NOW: &str = "FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL";

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn a_skipped_move_is_named_and_the_route_is_printed() {
    let (ok, out, _) = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW]);
    assert!(ok);
    assert!(out.contains("step 5 (D') was skipped"), "{out}");
    assert!(out.contains("You had done 8 of 21 steps"), "{out}");
    assert!(out.contains("Moves now: D2 B2 D'"), "{out}");
    assert!(
        out.contains("Then continue with step 7 (B2) of the printed solution."),
        "{out}"
    );
}

#[test]
fn a_given_solution_is_followed_and_the_shorter_route_wins() {
    let solution = "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'";
    let (ok, out, _) = cubacle(&[
        "recover",
        "-s",
        ORIGINAL,
        "--now",
        NOW,
        "--solution",
        solution,
        "--strays",
        "0",
        "--depth",
        "0",
        "--one-mistake",
    ]);
    assert!(ok);
    // Undoing 3 moves costs 20 in all, solving fresh 18: fresh wins.
    assert!(out.contains("Recommended: solve fresh"), "{out}");
    let moves = out.lines().find_map(|l| l.strip_prefix("Moves now: "));
    assert_eq!(moves.map(|m| m.split(' ').count()), Some(18), "{out}");
}

#[test]
fn fresh_skips_the_search() {
    let (ok, out, _) = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW, "--fresh"]);
    assert!(ok);
    assert_eq!(out.split(' ').count(), 18, "{out}");
}

#[test]
fn a_cube_on_the_path_says_where() {
    // The original cube after its first four solution moves, `B L' U2 B2`.
    let (_, now, _) = cubacle(&["facelets", &format!("{ORIGINAL} B L' U2 B2")]);
    let (ok, out, _) = cubacle(&["recover", "-s", ORIGINAL, "--now", &now]);
    assert!(ok);
    assert!(out.contains("you are after step 4 of 21"), "{out}");
    assert!(out.contains("Continue with step 5 (D')."), "{out}");
}

#[test]
fn bad_input_is_an_error() {
    let (ok, _, err) = cubacle(&["recover", "-s", ORIGINAL, "--now", "UUUU"]);
    assert!(!ok);
    assert!(err.starts_with("error:"), "{err}");
    let (ok, _, err) = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW, "--solution", "R U"]);
    assert!(!ok);
    assert!(err.contains("does not solve"), "{err}");
}
