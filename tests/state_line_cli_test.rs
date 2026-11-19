//! The state line on stderr (`state: <format> <value>`, `target: ...`) of the
//! commands that start from or aim at a cube, and `--state-format`.

use std::io::Write;
use std::process::{Command, Stdio};

use cubacle::notation::orbit64::v2;

struct Out {
    ok: bool,
    stdout: String,
    stderr: String,
}

fn command(args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cubacle"));
    command
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .env_remove("CUBACLE_CONFIG");
    command
}

fn run(args: &[&str]) -> Out {
    let output = command(args).output().expect("cubacle runs");
    Out {
        ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

fn ok(args: &[&str]) -> Out {
    let out = run(args);
    assert!(out.ok, "{args:?}: {}", out.stderr);
    out
}

/// The lines of stderr that start with `label:`.
fn lines<'a>(out: &'a Out, label: &str) -> Vec<&'a str> {
    out.stderr
        .lines()
        .filter(|line| line.starts_with(&format!("{label}: ")))
        .collect()
}

/// The V2 token of `state: orbit64v2 <token>`.
fn token_of(line: &str) -> &str {
    line.strip_prefix("state: orbit64v2 ")
        .or_else(|| line.strip_prefix("target: orbit64v2 "))
        .unwrap_or_else(|| panic!("not a V2 line: {line}"))
}

/// The facelets a V2 token shows, without the field separators.
fn facelets_of(token: &str) -> String {
    v2::facelets::decode(token)
        .expect("a V2 token")
        .1
        .replace(' ', "")
}

/// A scramble with a whole-cube rotation, so that the frame matters.
const SCRAMBLE: &str = "R U x y2 F'";

#[test]
fn every_command_with_a_cube_prints_a_readable_state_line() {
    let cube = ok(&["convert", "--size", "3", SCRAMBLE]).stdout;
    let short = ok(&["convert", "--size", "3", "R U F'"]).stdout;
    let checks: Vec<(Vec<&str>, &str, &str)> = vec![
        (vec!["facelets", "--size", "3", SCRAMBLE], "state", &cube),
        (
            vec!["solve", "--size", "3", "-s", SCRAMBLE, "--fast"],
            "state",
            &cube,
        ),
        (vec!["moves", "--size", "3", SCRAMBLE], "target", &cube),
        (
            vec!["patterns", "find", "--size", "3", "-s", "R U F'"],
            "target",
            &short,
        ),
    ];
    for (args, label, expected) in checks {
        let out = ok(&args);
        let found = lines(&out, label);
        assert_eq!(found.len(), 1, "{args:?}: {}", out.stderr);
        assert_eq!(facelets_of(token_of(found[0])), *expected, "{args:?}");
    }
}

#[test]
fn scramble_and_random_print_the_state_of_the_cube_they_made() {
    let out = ok(&["random", "--size", "5"]);
    let found = lines(&out, "state");
    assert_eq!(found.len(), 1);
    assert_eq!(facelets_of(token_of(found[0])), out.stdout);
    let out = ok(&["scramble", "--size", "4", "-n", "12"]);
    let found = lines(&out, "state");
    assert_eq!(found.len(), 1);
    let again = ok(&["convert", "--size", "4", &out.stdout]).stdout;
    assert_eq!(facelets_of(token_of(found[0])), again);
}

#[test]
fn moves_prints_the_start_and_the_target() {
    let out = ok(&["moves", "--size", "3", "R U"]);
    assert_eq!(
        lines(&out, "state"),
        ["state: orbit64v2 AAAAAAAAAAAA"],
        "{}",
        out.stderr
    );
    let target = ok(&["convert", "--size", "3", "R U", "--to", "token2"]).stdout;
    assert_eq!(
        lines(&out, "target"),
        [format!("target: orbit64v2 {target}")]
    );
    // --from changes the start
    let out = ok(&[
        "moves", "--size", "3", "--icons", "--format", "text", "--from", "F", "R",
    ]);
    let start = ok(&["convert", "--size", "3", "F", "--to", "token2"]).stdout;
    assert_eq!(lines(&out, "state"), [format!("state: orbit64v2 {start}")]);
}

#[test]
fn solve_prints_the_start_and_the_target() {
    let out = ok(&["solve", "--size", "3", "-s", "R U", "--to", "R", "--fast"]);
    let start = ok(&["convert", "--size", "3", "R U", "--to", "token2"]).stdout;
    let target = ok(&["convert", "--size", "3", "R", "--to", "token2"]).stdout;
    assert_eq!(lines(&out, "state"), [format!("state: orbit64v2 {start}")]);
    assert_eq!(
        lines(&out, "target"),
        [format!("target: orbit64v2 {target}")]
    );
}

#[test]
fn a_target_with_open_stickers_is_unavailable_and_the_exit_code_stays() {
    // superflip-free test target: the cross of the down face, the rest open
    let out = ok(&[
        "solve", "--size", "3", "-s", "R U", "--to", "cross", "--fast",
    ]);
    assert_eq!(
        lines(&out, "target"),
        ["target: unavailable: open stickers: the target is a set of states, not one cube"],
        "{}",
        out.stderr
    );
    assert_eq!(lines(&out, "state").len(), 1);
}

#[test]
fn the_state_format_picks_the_text_and_none_leaves_the_line_out() {
    let token = ok(&["convert", "--size", "3", SCRAMBLE, "--to", "token"]).stdout;
    let facelets = ok(&["convert", "--size", "3", SCRAMBLE]).stdout;
    let colors = ok(&["convert", "--size", "3", SCRAMBLE, "--to", "color"]).stdout;
    for (format, value) in [
        ("orbit64", token.as_str()),
        ("facelets", facelets.as_str()),
        ("color", colors.as_str()),
    ] {
        let out = ok(&[
            "facelets",
            "--size",
            "3",
            SCRAMBLE,
            "--state-format",
            format,
        ]);
        assert_eq!(lines(&out, "state"), [format!("state: {format} {value}")]);
    }
    let out = ok(&[
        "facelets",
        "--size",
        "3",
        SCRAMBLE,
        "--state-format",
        "none",
    ]);
    assert!(!out.stderr.contains("state:"), "{}", out.stderr);
    let out = ok(&["moves", "R U", "--state-format", "none"]);
    assert!(!out.stderr.contains("state:") && !out.stderr.contains("target:"));
    let bad = run(&["facelets", "R", "--state-format", "v3"]);
    assert!(!bad.ok);
    assert!(bad.stderr.contains("orbit64v2"), "{}", bad.stderr);
}

#[test]
fn standard_output_does_not_depend_on_the_state_line() {
    for args in [
        vec!["facelets", "--size", "4", "R U Rw"],
        vec!["moves", "R U R' U'"],
        vec!["solve", "--size", "3", "-s", "R U R'", "--fast"],
        vec!["solve", "--size", "2", "-s", "R U R'"],
    ] {
        let with_line = ok(&args);
        let mut quiet = args.clone();
        quiet.extend(["--state-format", "none"]);
        let without = ok(&quiet);
        assert_eq!(with_line.stdout, without.stdout, "{args:?}");
        assert!(!with_line.stdout.contains("state:"));
        assert!(!without.stderr.contains("state:"));
        assert!(with_line.stderr.contains("state:"), "{args:?}");
    }
}

#[test]
fn a_scramble_can_be_piped_into_solve() {
    let mut scramble = command(&["scramble", "--size", "3", "-n", "10"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut piped = scramble.stdout.take().unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(&mut piped, &mut text).unwrap();
    let scramble_out = scramble.wait_with_output().unwrap();
    assert!(String::from_utf8_lossy(&scramble_out.stderr).contains("state: orbit64v2 "));
    let mut solve = command(&["solve", "-", "--fast"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    solve
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
    let output = solve.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("state:") && !stdout.trim().is_empty());
}

#[test]
fn recover_prints_the_original_the_current_cube_and_the_target() {
    let now_cube = ok(&["convert", "--size", "3", "R U F R"]).stdout;
    let out = ok(&["solve", "--size", "3", "-s", "R U F", "--now", &now_cube]);
    assert_eq!(lines(&out, "state").len(), 1, "{}", out.stderr);
    assert_eq!(lines(&out, "now").len(), 1);
    let original = ok(&["convert", "--size", "3", "R U F", "--to", "token2"]).stdout;
    let now = ok(&["convert", "--size", "3", "R U F R", "--to", "token2"]).stdout;
    assert_eq!(
        lines(&out, "state"),
        [format!("state: orbit64v2 {original}")]
    );
    assert_eq!(lines(&out, "now"), [format!("now: orbit64v2 {now}")]);
}

#[test]
fn a_picture_start_has_no_color_state() {
    let solved = ok(&[
        "convert", "--size", "3", "U U'", "--to", "token2", "--model", "picture",
    ])
    .stdout;
    let out = ok(&[
        "moves",
        "--icons",
        "--format",
        "text",
        "--picture",
        &solved,
        "R",
    ]);
    assert_eq!(
        lines(&out, "state"),
        ["state: unavailable: --picture starts from a picture cube"],
        "{}",
        out.stderr
    );
    assert!(out.ok);
}

#[test]
fn photo_prints_the_state_of_the_cube_it_photographs() {
    let dir = std::env::temp_dir().join(format!("cubacle-state-photo-{}", std::process::id()));
    let out = ok(&[
        "photo",
        "--size",
        "3",
        "-s",
        "R U",
        "--pixels",
        "16",
        "--sheet",
        "--out",
        dir.to_str().unwrap(),
    ]);
    let start = ok(&["convert", "--size", "3", "R U", "--to", "token2"]).stdout;
    assert_eq!(lines(&out, "state"), [format!("state: orbit64v2 {start}")]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn commands_without_a_cube_print_no_state_line() {
    for args in [
        vec!["order", "R U"],
        vec!["convert", "--size", "3", "R"],
        vec!["canon", "--size", "3", "-s", "R"],
    ] {
        let out = ok(&args);
        assert!(
            !out.stderr.contains("state:") && !out.stderr.contains("target:"),
            "{args:?}"
        );
    }
}

#[test]
fn none_also_silences_unavailable_lines() {
    let out = ok(&[
        "solve",
        "--size",
        "3",
        "-s",
        "R U",
        "--to",
        "cross",
        "--fast",
        "--state-format",
        "none",
    ]);
    assert!(
        !out.stderr.contains("target:") && !out.stderr.contains("state:"),
        "{}",
        out.stderr
    );
}

#[test]
#[cfg(target_os = "linux")]
fn an_unwritable_stderr_does_not_panic() {
    let status = command(&["random", "--size", "3"])
        .stderr(
            std::fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .stdout(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "{status}");
}
