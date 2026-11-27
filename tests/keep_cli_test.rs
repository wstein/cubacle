use cubacle::metrics::{count, Metric};
use cubacle::notation::parse_cube;
use cubacle::puzzle::{MoveSequence, PuzzleKind};
use std::process::{Command, Output};

const CUBE: &str = "nfb3sIV3UzMJrpAzyyp53BTN5MDza09nzxMJFyj0yA";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .unwrap()
}

fn solution(keep: Option<&str>, expected: usize) -> String {
    let mut args = vec!["solve", CUBE, "--max-time", "0"];
    if let Some(value) = keep {
        args.extend(["--keep", value]);
    }
    let out = run(&args);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    let moves: MoveSequence = text.trim().parse().unwrap();
    assert_eq!(
        count(&moves, PuzzleKind::Cube5, Metric::Obtm),
        expected,
        "keep={keep:?}"
    );
    assert!(parse_cube(CUBE, None)
        .unwrap()
        .apply(&moves)
        .unwrap()
        .is_solved());
    text
}

#[test]
fn keep_controls_reduction_and_solutions_replay() {
    let default = solution(None, 71);
    assert_eq!(default, solution(Some("200,500,500,500"), 71));
    let thousand = solution(Some("1000"), 70);
    assert_eq!(thousand, solution(Some("1000,500,500,500"), 70));
    solution(Some("20000"), 40);
    solution(Some("200,5000,5000,5000"), 71);
}

#[test]
fn keep_rejects_invalid_values_and_other_sizes() {
    for value in [
        "0",
        "20001",
        "no",
        "1,2",
        "1,2,3,4,5,6",
        "1,,3,4,1",
        "1,0,3,4,1",
        "1,x,3,4,1",
        "1,2,3,4,2",
    ] {
        let out = run(&["solve", CUBE, "--keep", value]);
        assert!(!out.status.success(), "{value}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("--keep"));
    }
    // five values, also with a last 1: the phase 5 quota comes from --results
    for value in ["1,2,3,4,1", "200,500,500,500,1"] {
        let out = run(&["solve", CUBE, "--keep", value]);
        assert!(!out.status.success(), "{value}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("--keep") && err.contains("--results"), "{err}");
    }
    for options in [
        vec!["--upright-centers"],
        vec!["--fast"],
        vec!["--now", CUBE],
        vec!["--within=2"],
    ] {
        let mut args = vec!["solve", CUBE, "--keep", "1000"];
        args.extend(options);
        let out = run(&args);
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("--keep"));
    }
    for size in ["2", "3", "4", "6", "7"] {
        let out = run(&["solve", "-s", "R", "--size", size, "--keep", "1000"]);
        assert!(!out.status.success(), "{size}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("--keep"));
    }
}
