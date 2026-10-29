//! Solve report versioning and evidence from the actual route, not requested flags.

use std::process::{Command, Output};

fn solve(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .arg("solve")
        .args(args)
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
        )
        .output()
        .expect("cubacle runs")
}

fn body(args: &[&str]) -> String {
    let result = solve(args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout)
        .unwrap()
        .trim_end()
        .to_owned()
}

fn evidence(text: &str, frame: &str, proven: bool, bound: Option<usize>) {
    assert!(text.contains("\"formatVersion\":2"), "{text}");
    assert!(
        text.contains(&format!(
            "\"toolVersion\":\"{}\"",
            env!("CARGO_PKG_VERSION")
        )),
        "{text}"
    );
    let tail = format!(
        "\"frame\":\"{frame}\",\"proven\":{proven},\"lowerBound\":{}}}",
        bound.map_or_else(|| "null".into(), |v| v.to_string())
    );
    assert!(text.ends_with(&tail), "{text}");
}

#[test]
fn default_is_two_and_one_preserves_the_original_body_prefix() {
    let first = body(&["R U", "--fast", "--json", "--json-format", "1"]);
    let second = body(&["R U", "--fast", "--json"]);
    let explicit = body(&["R U", "--fast", "--json", "--json-format", "2"]);
    let mask_time = |s: &str| {
        let at = s.find("\"elapsedMicros\":").unwrap() + "\"elapsedMicros\":".len();
        let digits = s[at..].bytes().take_while(u8::is_ascii_digit).count();
        format!("{}0{}", &s[..at], &s[at + digits..])
    };
    let tail = second.find(",\"formatVersion\":").unwrap();
    assert_eq!(
        mask_time(&first),
        mask_time(&format!("{}}}", &second[..tail]))
    );
    assert_eq!(mask_time(&second), mask_time(&explicit));
    assert!(!first.contains("formatVersion"));
    evidence(&second, "fixed", false, None);
}

#[test]
fn version_one_preserves_phases_and_pattern_hint_bytes_as_well() {
    for args in [
        &["R U Rw Uw", "--size", "4", "--fast"][..],
        &[
            "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2",
            "--fast",
            "-v",
        ][..],
    ] {
        let mut one_args = args.to_vec();
        one_args.extend_from_slice(&["--json", "--json-format", "1"]);
        let mut two_args = args.to_vec();
        two_args.push("--json");
        let one = body(&one_args);
        let two = body(&two_args);
        let tail = two.find(",\"formatVersion\":").unwrap();
        let mask = |s: &str| {
            let at = s.find("\"elapsedMicros\":").unwrap() + "\"elapsedMicros\":".len();
            let digits = s[at..].bytes().take_while(u8::is_ascii_digit).count();
            format!("{}0{}", &s[..at], &s[at + digits..])
        };
        assert_eq!(mask(&one), mask(&format!("{}}}", &two[..tail])));
    }
}

#[test]
fn quotient_table_and_exact_within_have_different_frames() {
    // A whole-cube y rotation: solved in the quotient, two turns in the fixed frame.
    let quotient = body(&["U D'", "--size", "2", "--json"]);
    assert!(quotient.contains("\"length\":0"), "{quotient}");
    evidence(&quotient, "any-orientation", true, Some(0));
    let exact = body(&["U D'", "--size", "2", "--within=2", "--json"]);
    assert!(exact.contains("\"length\":2"), "{exact}");
    evidence(&exact, "fixed", true, Some(2));
    let solved = cubacle::puzzle::Facelets::solved(cubacle::puzzle::PuzzleKind::Cube2);
    let source = solved.apply(&"U D'".parse().unwrap()).unwrap();
    assert_ne!(
        source, solved,
        "the fixed physical frame is genuinely different"
    );
    let word = exact
        .split("\"solution\":\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert_eq!(source.apply(&word.parse().unwrap()).unwrap(), solved);
}

#[test]
fn within_fallback_and_non_htm_search_do_not_claim_proof_from_flags() {
    let fallback = body(&["R U", "--within=1", "--then-solve", "--fast", "--json"]);
    evidence(&fallback, "fixed", false, None);
    let slices = body(&["R", "--within=1", "--slices", "--json"]);
    evidence(&slices, "fixed", false, None);
    let blocks = body(&["R", "--within=1", "--blocks", "--json"]);
    evidence(&blocks, "fixed", false, None);
}

#[test]
fn optimal_budget_incumbent_is_not_reported_proven() {
    let result = body(&[
        "R U F' D2 L B' R2 U' F2 D L2 B R' U2 F D' L' B2",
        "--optimal",
        "--nodes",
        "1",
        "--max-memory",
        "100",
        "--json",
    ]);
    assert!(result.contains("\"proven\":false"), "{result}");
    assert!(result.contains("\"frame\":\"fixed\""), "{result}");
    assert!(!result.contains("\"lowerBound\":null"), "{result}");
}

#[test]
fn version_selector_requires_json_and_rejects_unknown_versions() {
    for args in [
        &["R", "--json-format", "1"][..],
        &["R", "--json", "--json-format", "3"][..],
    ] {
        let result = solve(args);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
    }
}
