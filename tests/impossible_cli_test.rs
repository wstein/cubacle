//! Commands that read a cube state refuse an impossible one, with the case
//! named and a hint where the repair is certain.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        // never the user's real configuration
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "CUBACLE_CACHE_DIR",
            concat!(env!("CARGO_TARGET_TMPDIR"), "/impossible-tables"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

fn solved(n: usize) -> Vec<char> {
    "URFDLB"
        .chars()
        .flat_map(|c| std::iter::repeat_n(c, n * n))
        .collect()
}

/// The URF corner twisted in place.
fn twisted(n: usize) -> String {
    let mut s = solved(n);
    let (u, r, f) = (n * n - 1, n * n, 2 * n * n + n - 1);
    (s[u], s[r], s[f]) = (s[r], s[f], s[u]);
    s.into_iter().collect()
}

/// The UF and UB middle edges exchanged.
fn swapped_edges(n: usize) -> String {
    let mut s = solved(n);
    let m = n / 2;
    s.swap((n - 1) * n + m, m);
    s.swap(2 * n * n + m, 5 * n * n + m);
    s.into_iter().collect()
}

const TWISTED: &str = "error: the cube state is impossible \
(the corners are invalid: one corner is twisted on its own)\nhint: turn one corner back";

#[test]
fn solve_names_a_twisted_corner_alike_on_every_size() {
    for n in 2..=7 {
        let size = n.to_string();
        let (ok, out, err) = cubacle(&["solve", "--size", &size, &twisted(n)]);
        assert!(!ok, "{n}");
        assert_eq!(out, "", "{n}");
        assert_eq!(err, TWISTED, "{n}");
    }
}

#[test]
fn parity_and_color_counts_are_named() {
    let (ok, _, err) = cubacle(&["solve", &swapped_edges(3)]);
    assert!(!ok);
    assert_eq!(
        err,
        "error: the cube state is impossible (the cube is invalid: two pieces are swapped \
(parity error))\nhint: swap two edges"
    );
    let mut counts = solved(3);
    counts[0] = 'R';
    let text: String = counts.into_iter().collect();
    let (ok, _, err) = cubacle(&["solve", &text]);
    assert!(!ok);
    assert!(
        err.ends_with("\nhint: color U has 1 sticker too few, color R has 1 sticker too many"),
        "{err}"
    );
}

#[test]
fn convert_and_patterns_find_run_the_check() {
    for args in [
        vec!["convert", "--size", "4", &twisted(4)],
        vec!["patterns", "find", "--size", "4", &twisted(4)],
    ] {
        let (ok, out, err) = cubacle(&args);
        assert!(!ok, "{args:?}");
        assert_eq!(out, "", "{args:?}");
        assert_eq!(err, TWISTED, "{args:?}");
    }
}

#[test]
fn solve_now_checks_the_cube_as_it_is() {
    let (ok, _, err) = cubacle(&["solve", "R U", "--now", &twisted(3)]);
    assert!(!ok);
    assert_eq!(err, TWISTED.replace("error: ", "error: --now: "));
}

#[test]
fn possible_states_are_unchanged() {
    for size in ["2", "3", "4", "5", "6", "7"] {
        let (ok, expected, _) = cubacle(&["facelets", "--size", size, "R U R' U' F"]);
        assert!(ok);
        let (ok, out, err) = cubacle(&["convert", "--size", size, &expected]);
        assert!(ok, "{err}");
        assert_eq!(out, expected);
        assert_eq!(err, "");
    }
}

#[test]
fn identify_and_cycles_warn_and_go_on() {
    for sub in ["identify", "cycles"] {
        for n in [2usize, 3, 4] {
            let size = n.to_string();
            let (ok, out, err) = cubacle(&["patterns", sub, "--size", &size, &twisted(n)]);
            assert!(ok, "{sub} {n}: {err}");
            assert!(!out.is_empty(), "{sub} {n}");
            // (other warnings, about the pattern sources, may come first)
            assert!(
                err.ends_with(
                    "warning: the cube state is impossible (the corners are invalid: one corner \
is twisted on its own)\nhint: turn one corner back"
                ),
                "{sub} {n}: {err}"
            );
            // a possible state gets no such warning
            let (ok, _, err) = cubacle(&["patterns", sub, "--size", &size, "R U"]);
            assert!(ok);
            assert!(!err.contains("state is impossible"), "{sub} {n}: {err}");
        }
    }
}
