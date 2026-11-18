//! `solve --results N`: N distinct solutions, the plain solution first.

use cubacle::notation::parse_cube;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind};
use std::process::Command;

struct Run {
    ok: bool,
    out: String,
    err: String,
}

fn cubacle(args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    Run {
        ok: output.status.success(),
        out: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        err: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

/// A cube of each size from 2 to 5: the size, the arguments that give the
/// cube, and the cube itself.
struct Sample {
    size: u8,
    input: Vec<&'static str>,
}

const FIVE: &str = "Ak_VhCkouy6cxCwk0oBN17-MS2FVAHOpnrxMJFyj0yA";

fn samples() -> Vec<Sample> {
    vec![
        Sample {
            size: 2,
            input: vec!["-s", "R U2 F' R U F2 R'", "--size", "2"],
        },
        Sample {
            size: 3,
            input: vec!["-s", "R U2 F' D L2 B U' R2 D2 F L' U B2 R' D' F2 L U2"],
        },
        Sample {
            size: 4,
            input: vec![
                "-s",
                "R U2 Rw' F D2 Uw B' L2 Fw U' R2 Bw L' D Rw2 U F' Lw B2 D' R Uw' F2 L",
                "--size",
                "4",
            ],
        },
        Sample {
            size: 5,
            input: vec![FIVE],
        },
    ]
}

fn solve(sample: &Sample, extra: &[&str]) -> Run {
    let mut args = vec!["solve"];
    args.extend(&sample.input);
    args.extend(extra);
    cubacle(&args)
}

/// The cube of a sample, from `facelets` or the token.
fn cube_of(sample: &Sample) -> Facelets {
    if sample.size == 5 {
        return parse_cube(FIVE, None).unwrap();
    }
    let mut args = vec!["facelets"];
    let scramble = sample.input[1];
    args.extend([
        "--size",
        if sample.size == 2 {
            "2"
        } else if sample.size == 3 {
            "3"
        } else {
            "4"
        },
    ]);
    args.push(scramble);
    let run = cubacle(&args);
    assert!(run.ok, "{}", run.err);
    parse_cube(&run.out, None).unwrap()
}

/// The three fields of a result line: rank, counts, moves.
fn fields(line: &str) -> (usize, &str, &str) {
    let mut parts = line.splitn(3, '\t');
    let rank = parts.next().unwrap().parse().unwrap();
    (rank, parts.next().unwrap(), parts.next().unwrap())
}

#[test]
fn one_result_is_the_plain_solve_in_every_size() {
    for sample in samples() {
        let plain = solve(&sample, &[]);
        let one = solve(&sample, &["--results", "1"]);
        assert!(plain.ok && one.ok, "{} {}", plain.err, one.err);
        assert_eq!(plain.out, one.out, "size {}", sample.size);
        assert_eq!(plain.err, one.err, "size {}", sample.size);
    }
}

#[test]
fn three_results_are_distinct_solve_the_cube_and_start_with_the_plain_one() {
    for sample in samples() {
        let plain = solve(&sample, &[]);
        let run = solve(&sample, &["--results", "3"]);
        assert!(run.ok, "size {}: {}", sample.size, run.err);
        let lines: Vec<&str> = run.out.lines().collect();
        assert_eq!(lines.len(), 3, "size {}: {}", sample.size, run.out);
        let cube = cube_of(&sample);
        let mut seen = Vec::new();
        for (at, line) in lines.iter().enumerate() {
            let (rank, counts, moves) = fields(line);
            assert_eq!(rank, at + 1);
            let sequence: MoveSequence = moves.parse().unwrap();
            // the counts are those of `moves --metrics`
            let size = sample.size.to_string();
            let counted = cubacle(&["moves", "--size", &size, "--metrics", moves]);
            assert_eq!(
                counted.err,
                format!("metrics: {counts}"),
                "size {}: {moves}",
                sample.size
            );
            assert!(
                cube.apply(&sequence).unwrap().is_solved(),
                "size {}: {moves}",
                sample.size
            );
            assert!(
                !seen.contains(&moves),
                "size {}: {moves} twice",
                sample.size
            );
            seen.push(moves);
        }
        assert_eq!(fields(lines[0]).2, plain.out, "size {}", sample.size);
    }
}

#[test]
fn json_is_an_array_of_reports_and_inverse_writes_each_solution_inverted() {
    let sample = &samples()[3];
    let json = solve(sample, &["--results", "3", "--json"]);
    assert!(json.ok, "{}", json.err);
    let value: serde_json::Value = serde_json::from_str(&json.out).unwrap();
    let reports = value.as_array().unwrap();
    assert_eq!(reports.len(), 3);
    let text = solve(sample, &["--results", "3"]);
    for (report, line) in reports.iter().zip(text.out.lines()) {
        assert_eq!(report["solution"].as_str().unwrap(), fields(line).2);
        let counts = fields(line).1;
        let obtm: u64 = counts
            .split(", ")
            .find_map(|part| part.strip_prefix("obtm "))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(report["metrics"]["obtm"], obtm);
    }
    let back = solve(sample, &["--results", "3", "--inverse"]);
    assert!(back.ok, "{}", back.err);
    let cube = cube_of(sample);
    let solved = Facelets::solved(PuzzleKind::Cube5);
    for (line, forward) in back.out.lines().zip(text.out.lines()) {
        let (_, counts, moves) = fields(line);
        assert_eq!(
            counts,
            fields(forward).1,
            "the counts stay those of the solution"
        );
        let inverse: MoveSequence = moves.parse().unwrap();
        assert_eq!(solved.apply(&inverse).unwrap(), cube);
    }
}

#[test]
fn fewer_distinct_solutions_than_asked_say_so() {
    let sample = &samples()[2];
    let run = solve(sample, &["--results", "100"]);
    assert!(run.ok, "{}", run.err);
    let count = run.out.lines().count();
    assert!(count < 100, "{count}");
    assert!(
        run.err.contains(&format!("found {count} of 100 solutions")),
        "{}",
        run.err
    );
}

#[test]
fn results_out_of_range_and_unsupported_combinations_name_the_option() {
    let three = &samples()[1];
    for value in ["0", "10001", "x"] {
        let run = solve(three, &["--results", value]);
        assert!(!run.ok, "{value}");
        assert!(run.err.contains("--results"), "{value}: {}", run.err);
    }
    for extra in [
        vec!["--icons"],
        vec!["--optimize", "ranges"],
        vec!["--group", "axis"],
        vec!["--every", "5"],
        vec!["--improve"],
        vec!["--moves", "R U"],
        vec!["--within=2"],
        vec!["--json", "--inverse"],
    ] {
        let mut args = vec!["--results", "3"];
        args.extend(&extra);
        let run = solve(three, &args);
        assert!(!run.ok, "{extra:?}");
        assert!(
            run.err.contains("--results") || extra.contains(&"--inverse"),
            "{extra:?}: {}",
            run.err
        );
    }
    // one result stays possible in every mode
    let run = solve(three, &["--results", "1", "--optimize", "ranges"]);
    assert!(run.ok, "{}", run.err);
    for size in ["6", "7"] {
        let run = cubacle(&["solve", "-s", "R", "--size", size, "--results", "3"]);
        assert!(!run.ok, "{size}");
        assert!(run.err.contains("--results"), "{size}: {}", run.err);
    }
}

/// The moves of a line with every run of turns about one axis sorted: two
/// lines that differ only in the order of commuting turns get the same key.
fn axis_key(moves: &str) -> Vec<String> {
    let axis = |token: &str| {
        let face = token.chars().find(char::is_ascii_uppercase).unwrap();
        "UD RL FB"
            .split(' ')
            .position(|pair| pair.contains(face))
            .unwrap()
    };
    let tokens: Vec<&str> = moves.split_whitespace().collect();
    let mut key = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let mut j = i;
        while j < tokens.len() && axis(tokens[j]) == axis(tokens[i]) {
            j += 1;
        }
        let mut run: Vec<&str> = tokens[i..j].to_vec();
        run.sort_unstable();
        key.extend(run.iter().map(|t| (*t).to_owned()));
        key.push("|".to_owned());
        i = j;
    }
    key
}

#[test]
fn no_two_results_differ_only_in_the_order_of_commuting_turns() {
    for (at, count) in [(3, "20"), (3, "100"), (1, "100"), (2, "100")] {
        let sample = &samples()[at];
        let run = solve(sample, &["--results", count]);
        assert!(run.ok, "size {}: {}", sample.size, run.err);
        if sample.size == 5 {
            // the list is filled up to N while duplicates are left out
            assert_eq!(run.out.lines().count(), count.parse::<usize>().unwrap());
        }
        let mut keys: Vec<Vec<String>> = Vec::new();
        for line in run.out.lines() {
            let key = axis_key(fields(line).2);
            assert!(
                !keys.contains(&key),
                "size {}: {line} repeats an earlier line up to commuting turns",
                sample.size
            );
            keys.push(key);
        }
    }
}

#[test]
fn the_limit_is_ten_thousand_and_the_message_names_it() {
    // the 2x2x2 has far fewer solutions than that: the list ends short, no error
    let two = &samples()[0];
    let run = solve(two, &["--results", "10000"]);
    assert!(run.ok, "{}", run.err);
    let count = run.out.lines().count();
    assert!(count > 1 && count < 10000, "{count}");
    assert!(
        run.err
            .contains(&format!("found {count} of 10000 solutions")),
        "{}",
        run.err
    );
    let run = solve(two, &["--results", "10001"]);
    assert!(!run.ok);
    assert!(
        run.err.contains("--results") && run.err.contains("10000"),
        "{}",
        run.err
    );
}

#[test]
fn a_closed_pipe_ends_the_list_without_an_error() {
    use std::io::Read;
    let three = &samples()[1];
    let mut args = vec!["solve"];
    args.extend(&three.input);
    args.extend(["--results", "50"]);
    let mut child = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // close the reading end before the list is written (the search takes longer)
    drop(child.stdout.take().unwrap());
    let mut err = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut err)
        .unwrap();
    let status = child.wait().unwrap();
    assert!(!err.contains("panicked"), "{err}");
    assert!(status.success(), "{status:?} {err}");
}
