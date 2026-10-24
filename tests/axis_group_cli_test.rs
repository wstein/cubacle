//! `--group axis`: turns about one axis on disjoint layers are one hand unit.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
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

fn axis(size: &str, text: &str) -> String {
    let (ok, out, error) = cubacle(&["moves", "--size", size, "--group", "axis", text]);
    assert!(ok, "{text}: {error}");
    out
}

#[test]
fn moves_writes_dots_between_axial_units() {
    assert_eq!(axis("3", "U D' R L F B2 R U"), "U D' · R L · F B2 · R · U");
    assert_eq!(axis("3", "U D' U"), "U D' · U");
    assert_eq!(axis("3", "R"), "R");
}

#[test]
fn wide_moves_ranges_and_big_cubes() {
    assert_eq!(axis("4", "Rw Lw U"), "Rw Lw · U");
    assert_eq!(axis("4", "Rw R"), "Rw · R");
    assert_eq!(axis("7", "2-3Rw 2-3Lw U"), "3Rw R' 3Lw L' · U");
    assert_eq!(axis("5", "2R 2L' 3U"), "2R 2L' · 3U");
}

#[test]
fn rotations_join_the_run_of_their_axis() {
    assert_eq!(axis("3", "R x L"), "R Rw L' · L");
    assert_eq!(axis("3", "R y"), "R · Uw D'");
    assert_eq!(axis("4", "U Rw Lw' Uw"), "U · Rw Lw' · Uw");
}

#[test]
fn the_dotted_text_reads_back_as_the_same_moves() {
    let text = "U D' R 2-3Lw x F B' R2 U";
    let dotted = axis("5", text);
    assert!(dotted.contains(" · "));
    let (ok, flat, error) = cubacle(&["moves", "--size", "5", &dotted]);
    assert!(ok, "{error}");
    let (ok, want, _) = cubacle(&["moves", "--size", "5", text]);
    assert!(ok);
    assert_eq!(flat, want);
    // the marks in the input are ignored by axis
    let (ok, again, _) = cubacle(&["moves", "--size", "5", "--group", "axis", &dotted]);
    assert!(ok);
    assert_eq!(again, dotted);
}

#[test]
fn the_default_output_is_unchanged() {
    let (ok, plain, _) = cubacle(&["moves", "U D' R"]);
    assert!(ok);
    assert_eq!(plain, "U D' R");
    let (ok, _, error) = cubacle(&["moves", "--group", "none", "U D'"]);
    assert!(!ok);
    assert!(error.contains("--icons"), "{error}");
}

#[test]
fn icons_draw_the_axial_units() {
    let (ok, out, error) = cubacle(&[
        "moves",
        "--icons",
        "--graphics",
        "plain",
        "--group",
        "axis",
        "U D' R L F",
    ]);
    assert!(ok, "{error}");
    assert!(out.contains("group 1 of 3: U D'"), "{out}");
    assert!(out.contains("group 2 of 3: R L"), "{out}");
    assert!(out.contains("group 3 of 3: F"), "{out}");
}

const SCRAMBLE: &str = "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw";

#[test]
fn solve_axis_adds_dots_only_and_reads_back() {
    for notation in ["wca", "sign", "sse"] {
        let base = ["solve", "--size", "4", "--notation", notation];
        let (ok, plain, error) = cubacle(&[&base[..], &[SCRAMBLE]].concat());
        assert!(ok, "{error}");
        let (ok, dotted, error) = cubacle(&[&base[..], &["--group", "axis", SCRAMBLE]].concat());
        assert!(ok, "{error}");
        assert!(dotted.contains(" · "), "{dotted}");
        assert_eq!(dotted.replace(" · ", " "), plain, "{notation}");
        let (ok, flat, error) = cubacle(&["moves", "--size", "4", &dotted]);
        assert!(ok, "{dotted}: {error}");
        let (_, want, _) = cubacle(&["moves", "--size", "4", &plain]);
        assert_eq!(flat, want, "{notation}");
    }
}

#[test]
fn phases_and_axis_combine_with_phase_borders_first() {
    let size = ["solve", "--size", "4"];
    let run = |group: &str| {
        let (ok, out, error) = cubacle(&[&size[..], &["--group", group, SCRAMBLE]].concat());
        assert!(ok, "{error}");
        out
    };
    let phases = run("phases");
    let by_axis = run("axis");
    let both = run("phases,axis");
    assert_eq!(both.replace(" · ", " "), phases.replace(" · ", " "));
    // every phase border is a dot of the combination
    let count = |text: &str| text.matches(" · ").count();
    assert!(count(&both) >= count(&phases));
    assert!(count(&both) >= count(&by_axis));
    let ends: Vec<usize> = both
        .split(" · ")
        .scan(0usize, |seen, unit| {
            *seen += unit.split_whitespace().count();
            Some(*seen)
        })
        .collect();
    let mut seen = 0;
    for segment in phases.split(" · ") {
        seen += segment.split_whitespace().count();
        assert!(ends.contains(&seen), "{both}");
    }
}

#[test]
fn ranges_never_merge_across_an_axial_dot() {
    let (ok, packed, error) = cubacle(&[
        "solve",
        "--size",
        "4",
        "--group",
        "axis",
        "--optimise",
        "ranges",
        SCRAMBLE,
    ]);
    assert!(ok, "{error}");
    assert!(packed.contains(" · "), "{packed}");
    assert!(error.contains("compressed moves"), "{error}");
}
