//! `patterns identify` prints the canonical Orbit64 token, and the patterns
//! commands read Orbit64 tokens wherever they read a cube.

use std::process::Command;

use cubacle::canonical::canonical_text;

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
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn token_of(size: &str, moves: &str) -> String {
    let (ok, out, _) = cubacle(&["convert", "--size", size, "--to", "token", moves]);
    assert!(ok);
    out.trim().to_owned()
}

#[test]
fn identify_prints_the_canonical_token_below_the_pattern_lines() {
    for (size, moves) in [
        ("2", "R U"),
        ("3", "R U R' U'"),
        ("4", "Rw"),
        ("7", "R U Rw"),
    ] {
        let (ok, out, _) = cubacle(&["patterns", "identify", "--size", size, moves]);
        assert!(ok);
        let expected = canonical_text(&token_of(size, moves)).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        let at = lines
            .iter()
            .position(|l| l.starts_with("canonical: "))
            .expect("a canonical line");
        assert_eq!(lines[at], format!("canonical: {expected}"));
        assert!(lines[at - 1].starts_with("pattern:"), "{out}");
        assert!(lines[at + 1].starts_with("patternness:"), "{out}");
    }
}

#[test]
fn identify_with_several_matches_puts_the_line_below_the_last_one() {
    let (ok, out, _) = cubacle(&["patterns", "identify", "R2 L2 U2 D2 F2 B2"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with("canonical: "))
        .unwrap();
    assert!(
        lines[..at].iter().all(|l| l.starts_with("pattern:")),
        "{out}"
    );
    assert!(lines[at + 1].starts_with("patternness:"), "{out}");
}

#[test]
fn an_impossible_state_keeps_its_warning_and_has_no_canonical_line() {
    // one corner twisted in place
    let twisted = "UUUUUUUUU RRRRRRRRR FFFFFFFFF DDDDDDDDD LLLLLLLLL BBBBBBBBB";
    let swapped =
        twisted
            .replacen("UUUUUUUUU", "UUUUUUUUR", 1)
            .replacen("RRRRRRRRR", "URRRRRRRR", 1);
    let (ok, out, err) = cubacle(&["patterns", "identify", &swapped]);
    assert!(ok, "{err}");
    assert!(err.contains("impossible"), "{err}");
    assert!(!out.contains("canonical:"), "{out}");
}

#[test]
fn orbit64_tokens_are_cubes_for_identify_find_and_cycles() {
    for (size, moves) in [
        ("2", "R U"),
        ("3", "R U R' U'"),
        ("5", "Rw U"),
        ("7", "R U Rw"),
    ] {
        let token = token_of(size, moves);
        // no --size: the size comes from the token
        let (ok, out, err) = cubacle(&["patterns", "identify", &token]);
        assert!(ok, "{err}");
        assert!(out.contains("canonical: "), "{out}");
        let (ok, _, err) = cubacle(&["patterns", "cycles", &token]);
        assert!(ok, "{err}");
        let (ok, out, err) = cubacle(&["patterns", "find", &token]);
        assert!(ok, "{err}");
        assert!(!out.trim().is_empty());
    }
}

#[test]
fn a_bad_token_is_an_error_that_says_so() {
    let (ok, _, err) = cubacle(&["patterns", "identify", "AAAAAAAAAAAAAAAAAAAAAAAAAA"]);
    assert!(!ok);
    assert!(err.contains("not a cube"), "{err}");
}
