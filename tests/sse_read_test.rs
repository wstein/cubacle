//! SSE (Superset ENG) tokens read on every size: each twist equals turning
//! its layers one at a time, and the per-size rules of the six pages hold.

use cubacle::notation::{parse_expression, parse_moves};
use cubacle::puzzle::{Facelets, PuzzleKind};

const FACES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

fn opposite(face: char) -> char {
    FACES[(FACES.iter().position(|&f| f == face).unwrap() + 3) % 6]
}

fn kind(size: usize) -> PuzzleKind {
    PuzzleKind::of_size(size).unwrap()
}

fn inverse(suffix: &str) -> &'static str {
    match suffix {
        "" => "'",
        "'" => "",
        _ => "2",
    }
}

/// Turning the positions (1 = the face's own layer) one at a time, all the
/// same way round the axis as seen from `face`.
fn singles(size: usize, face: char, positions: &[usize], suffix: &str) -> String {
    positions
        .iter()
        .map(|&p| {
            if p < size {
                let depth = if p == 1 { String::new() } else { p.to_string() };
                format!("{depth}{face}{suffix} ")
            } else {
                format!("{}{} ", opposite(face), inverse(suffix))
            }
        })
        .collect()
}

fn state(size: usize, moves: &str) -> Facelets {
    Facelets::solved(kind(size))
        .apply(&parse_moves(moves).unwrap())
        .unwrap()
}

/// The layers a token turns, by the pages' own wording; `None` when the page
/// does not have the twist on this size.
fn positions(size: usize, prefix: char, a: usize, b: Option<usize>) -> Option<Vec<usize>> {
    let n = size;
    match prefix {
        'T' => (a >= 1 && a < n).then(|| (1..=a).collect()),
        'N' => {
            let b = b.unwrap_or(a);
            (n >= 5 && a >= 1 && a <= b && b <= n && !(a == 1 && b == n)).then(|| (a..=b).collect())
        }
        'V' => (n >= 5 && a >= 1 && a + 1 < n).then(|| (2..=a + 1).collect()),
        'M' => (n >= 3 && a >= 1 && a + 2 <= n).then(|| {
            let from = (n - a) / 2 + 1;
            (from..from + a).collect()
        }),
        'W' => (n >= 4).then(|| (2..n).collect()),
        'S' => {
            let (from, to) = match b {
                Some(b) => (a, b),
                None => (1, a),
            };
            if from < 1 || from > to || to > n {
                return None;
            }
            let mut set: Vec<usize> = (from..=to).chain(n + 1 - to..=n + 1 - from).collect();
            set.sort_unstable();
            set.dedup();
            (set.len() < n).then_some(set)
        }
        _ => Some((1..=n).collect()),
    }
}

#[test]
fn every_sse_twist_turns_exactly_its_layers() {
    let mut checked = 0;
    for size in 2..=7usize {
        for face in FACES {
            for (suffix, sse) in [("", ""), ("'", "'"), ("'", "-"), ("2", "2")] {
                for prefix in "TNVMWSC".chars() {
                    // (written number, range end)
                    let mut forms: Vec<(Option<usize>, Option<usize>)> = vec![(None, None)];
                    forms.extend((1..=8).map(|a| (Some(a), None)));
                    if matches!(prefix, 'N' | 'S') {
                        for a in 1..=8 {
                            forms.extend((a..=8).map(|b| (Some(a), Some(b))));
                        }
                    }
                    for (number, end) in forms {
                        let word = format!(
                            "{prefix}{}{}{face}{sse}",
                            number.map(|a| a.to_string()).unwrap_or_default(),
                            end.map(|b| format!("-{b}")).unwrap_or_default()
                        );
                        let default = match prefix {
                            'S' | 'M' => 1,
                            _ => 2,
                        };
                        let a = number.unwrap_or(default);
                        let wanted = if matches!(prefix, 'W' | 'C') && number.is_some() {
                            None
                        } else {
                            positions(size, prefix, a, end)
                        };
                        let got = parse_expression(&word, kind(size));
                        match wanted {
                            Some(set) => {
                                let moves = got.unwrap_or_else(|e| panic!("{word} on {size}: {e}"));
                                let want = state(size, &singles(size, face, &set, suffix));
                                let have = Facelets::solved(kind(size)).apply(&moves).unwrap();
                                assert_eq!(have, want, "{word} on {size}");
                                checked += 1;
                            }
                            None => {
                                let error = got.expect_err(&format!("{word} on {size}"));
                                assert!(error.to_string().contains("character 0"), "{error}");
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(checked > 5_000, "{checked}");
}

/// `M`, `S`, `E`, `x`, `y`, `z` keep their WCA meaning next to SSE.
#[test]
fn the_wca_slices_and_rotations_do_not_clash() {
    for (wca, sse) in [
        ("M", "ML"),
        ("M'", "ML'"),
        ("M2", "ML2"),
        ("S", "MF"),
        ("E'", "MD'"),
        ("x", "CR"),
        ("y'", "CU'"),
        ("z2", "CF2"),
    ] {
        assert_eq!(
            parse_expression(wca, kind(3)).unwrap(),
            parse_expression(sse, kind(3)).unwrap(),
            "{wca} / {sse}"
        );
    }
}

#[test]
fn groups_repeat_and_mix_with_wca() {
    let sse = parse_expression("(R TU F-)3 NL N2-3R", kind(5)).unwrap();
    let wca = parse_expression("(R Uw F')3 2L 3Rw R'", kind(5)).unwrap();
    assert_eq!(sse, wca);
}

#[test]
fn errors_name_the_position_and_the_rule() {
    let cases = [
        ("TR", 2, "leave a layer"),
        ("T9R", 5, "leave a layer"),
        ("N2R", 4, "from the 5x5"),
        ("V2R", 4, "from the 5x5"),
        ("V4R", 5, "last layer"),
        ("WR", 3, "from the 4x4"),
        ("MR", 2, "1 to n-2"),
        ("M9R", 5, "1 to n-2"),
        ("SR", 2, "every layer"),
        ("S2R", 4, "every layer"),
        ("N9R", 5, "beyond"),
        ("N3-2R", 5, "inwards"),
        ("TR3", 5, "not a turn"),
        ("N2-4", 5, "face letter"),
        ("C2R", 5, "no number"),
        ("W2R", 5, "no number"),
        ("T2-3R", 5, "only N and S"),
    ];
    for (text, size, reason) in cases {
        let error = parse_expression(&format!("R {text}"), kind(size))
            .unwrap_err()
            .to_string();
        assert!(error.contains("character 2"), "{text}: {error}");
        assert!(error.contains(reason), "{text}: {error}");
    }
}

#[test]
fn size_free_reading_takes_the_size_independent_twists_only() {
    assert_eq!(
        parse_moves("TR N2-4U' V3F").unwrap().to_string(),
        "Rw 4Uw' U 4Fw F'"
    );
    for word in ["MR", "WR", "SR", "CR"] {
        let error = parse_moves(word).unwrap_err().to_string();
        assert!(error.contains("cube size"), "{word}: {error}");
    }
}
