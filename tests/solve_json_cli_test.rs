//! `solve --json` writes the body the REST server sends for an equivalent
//! request. `crates/server/tests/solve_json_test.rs` compares the server with
//! the same file, `tests/solve_json_body.txt` (`elapsedMicros` masked to 0).
//!
//! Equivalent: the CLI default adds 3000 probes for a shorter answer and the
//! server default does not, so `--fast` stands for the server's plain search;
//! `--max-moves 21` is `{"maxMoves":21}`.

use std::process::{Command, Output};

const BODY: &str = include_str!("solve_json_body.txt");

fn cubacle(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs")
}

/// Replaces the number after `"elapsedMicros":` by 0.
fn masked(body: &str) -> String {
    let key = "\"elapsedMicros\":";
    let at = body.find(key).expect("elapsedMicros") + key.len();
    let digits = body[at..].bytes().take_while(u8::is_ascii_digit).count();
    format!("{}0{}", &body[..at], &body[at + digits..])
}

#[test]
fn json_is_the_rest_body_of_the_equivalent_request() {
    let output = cubacle(&[
        "solve",
        "-s",
        "R U",
        "--fast",
        "--max-moves",
        "21",
        "--json",
        "--json-format",
        "1",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.ends_with("}\n"), "one line: {stdout:?}");
    assert_eq!(masked(stdout.trim_end()), BODY.trim_end());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("moves 13\n"), "{stderr}");
}

#[test]
fn json_with_ranges_adds_the_compressed_keys_before_the_time() {
    let output = cubacle(&[
        "solve",
        "-s",
        "R U",
        "--fast",
        "--max-moves",
        "21",
        "--optimize",
        "ranges",
        "--json",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let at = |key: &str| {
        stdout
            .find(key)
            .unwrap_or_else(|| panic!("{key} in {stdout}"))
    };
    assert!(at("\"metrics\"") < at("\"compressed\":"));
    assert!(at("\"compressed\":") < at("\"compressedLength\""));
    assert!(at("\"compressedLength\"") < at("\"compressedMetrics\""));
    assert!(at("\"compressedMetrics\"") < at("\"elapsedMicros\""));
}

#[test]
fn json_with_inverse_is_refused() {
    let output = cubacle(&["solve", "-s", "R U", "--json", "--inverse"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("--json always carries the inverse"),
        "{stderr}"
    );
}

#[test]
fn json_is_refused_with_text_layouts() {
    for other in ["--group=phases", "--every=5", "--icons"] {
        let output = cubacle(&["solve", "-s", "R U", "--json", other]);
        assert!(!output.status.success(), "{other}");
        assert!(output.stdout.is_empty(), "{other}");
    }
}

#[test]
fn json_of_a_staged_engine_ends_with_the_phases() {
    let output = cubacle(&[
        "solve",
        "-s",
        "R U Rw Uw",
        "--size",
        "4",
        "--fast",
        "--json",
        "--json-format",
        "1",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        masked(stdout.trim_end()),
        include_str!("solve_json_body_four.txt").trim_end()
    );
}

#[test]
fn json_names_a_known_pattern_only_with_verbose() {
    let superflip = "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2";
    let hinted = cubacle(&[
        "solve",
        "-s",
        superflip,
        "--fast",
        "--json",
        "--json-format",
        "1",
        "-v",
    ]);
    assert!(hinted.status.success());
    let stdout = String::from_utf8(hinted.stdout).unwrap();
    assert!(
        stdout.ends_with(",\"patternHint\":\"superflip\"}\n"),
        "{stdout}"
    );
    let plain = cubacle(&["solve", "-s", superflip, "--fast", "--json"]);
    let stdout = String::from_utf8(plain.stdout).unwrap();
    assert!(!stdout.contains("patternHint"), "{stdout}");
}
