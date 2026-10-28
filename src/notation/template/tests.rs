use super::*;
use crate::puzzle::{Face, Facelets};

fn identity() -> ColorScheme {
    "URFDLB".parse().unwrap()
}

/// Six faces joined by " / "; `face` gets the face number 1..=6.
fn grid(face: impl Fn(usize) -> String) -> String {
    (1..=6).map(face).collect::<Vec<_>>().join(" / ")
}

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    let moves = moves.parse().unwrap();
    Facelets::solved(kind).apply(&moves).unwrap()
}

/// Every face is one placeholder `a1`..`a6`.
fn uniform(n: usize) -> String {
    grid(|k| vec![format!("a{k}"); n * n].join(" "))
}

/// The ring schema of the 5x5x5 example: corners a, edges b, crosses c, and a
/// fixed center; `centers` are the center letters of the six faces.
fn rings(centers: [char; 6]) -> String {
    grid(|k| {
        let (a, b, c) = (format!("a{k}"), format!("b{k}"), format!("c{k}"));
        let center = centers[k - 1].to_string();
        let rows = [
            [&a, &b, &b, &b, &a],
            [&b, &b, &c, &b, &b],
            [&b, &c, &center, &c, &b],
            [&b, &b, &c, &b, &b],
            [&a, &b, &b, &b, &a],
        ];
        rows.iter()
            .flat_map(|row| row.iter().map(|s| s.to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    })
}

fn parse(schema: &str, guard: &str, kind: PuzzleKind) -> Template {
    Template::parse(schema, guard, kind, &identity())
        .unwrap_or_else(|e| panic!("{schema:?} {guard:?}: {e}"))
}

#[test]
fn solved_cubes_fit_one_variable_per_face_and_turned_cubes_do_not() {
    for kind in [
        PuzzleKind::Cube2,
        PuzzleKind::Cube3,
        PuzzleKind::Cube4,
        PuzzleKind::Cube5,
    ] {
        let template = parse(&uniform(kind.size()), "", kind);
        assert_eq!(template.variables().len(), 6);
        assert!(template.matches(&Facelets::solved(kind)), "{kind:?}");
        assert!(!template.matches(&cube(kind, "R U'")), "{kind:?}");
    }
}

#[test]
fn guards_decide_between_assignments() {
    let kind = PuzzleKind::Cube3;
    let solved = Facelets::solved(kind);
    let schema = uniform(3);
    assert!(parse(&schema, "unique(a*)", kind).matches(&solved));
    assert!(parse(&schema, "count(a1, a2, a3) == 3", kind).matches(&solved));
    assert!(!parse(&schema, "equal(a1, a2)", kind).matches(&solved));
    assert!(parse(&schema, "a1 != a2 && a1 < a2", kind).matches(&solved));
    assert!(!parse(&schema, "a1 == a2 || not unique(a*)", kind).matches(&solved));
}

#[test]
fn fixed_colors_and_dont_cares() {
    let kind = PuzzleKind::Cube3;
    // Only the six centers are fixed, so every outer turn keeps the match.
    let centers = grid(|k| {
        let mut face = ["."; 9];
        face[4] = ["U", "R", "F", "D", "L", "B"][k - 1];
        face.join(" ")
    });
    let template = parse(&centers, "", kind);
    assert!(template.matches(&Facelets::solved(kind)));
    assert!(template.matches(&cube(kind, "R U F2 L' D B2")));
    // A different scheme reads the same letters as different faces.
    let western: ColorScheme = "western".parse().unwrap();
    assert!(Template::parse(&centers, "", kind, &western).is_err());
    let mixed = centers.replace("U", "R");
    assert!(!parse(&mixed, "", kind).matches(&Facelets::solved(kind)));
}

#[test]
fn center_blocks_survive_outer_turns_on_big_cubes() {
    for kind in [PuzzleKind::Cube4, PuzzleKind::Cube5] {
        let n = kind.size();
        // The inner (n-2)x(n-2) block of each face is one placeholder.
        let schema = grid(|k| {
            (0..n * n)
                .map(|i| {
                    let (row, col) = (i / n, i % n);
                    if (1..n - 1).contains(&row) && (1..n - 1).contains(&col) {
                        format!("a{k}")
                    } else {
                        ".".to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        });
        let template = parse(&schema, "unique(a*)", kind);
        assert!(template.matches(&Facelets::solved(kind)));
        assert!(template.matches(&cube(kind, "R U' F2 L2 B D'")));
        assert!(!template.matches(&cube(kind, "Rw U")), "{kind:?}");
    }
}

#[test]
fn a_cube_of_another_size_never_matches() {
    let template = parse(&uniform(3), "", PuzzleKind::Cube3);
    assert!(!template.matches(&Facelets::solved(PuzzleKind::Cube4)));
}

#[test]
fn owner_example_ring_schema() {
    let kind = PuzzleKind::Cube5;
    let schema = rings(['U', 'R', 'F', 'D', 'L', 'B']);
    let unique = parse(&schema, "unique(a*, b*, c*)", kind);
    assert_eq!(unique.variables().len(), 18);
    let solved = Facelets::solved(kind);
    // On a solved cube every ring of a face has the face's color.
    assert!(!unique.matches(&solved));
    assert!(parse(&schema, "count(a*, b*, c*) == 1", kind).matches(&solved));

    // Face f: a in color f, b in f+1, c in f+2, the center in f.
    let stickers = |swap: bool| {
        let mut out = Vec::new();
        for f in 0..6 {
            for i in 0..25 {
                let (row, col) = (i / 5, i % 5);
                let color = match (row.min(4 - row), col.min(4 - col)) {
                    (0, 0) => f,
                    (2, 2) => f,
                    (1, 2) | (2, 1) => f + 2,
                    _ => f + 1,
                };
                out.push(Face::ALL[color % 6]);
            }
        }
        if swap {
            out.swap(0, 1);
        }
        Facelets::from_stickers(out).unwrap()
    };
    assert!(unique.matches(&stickers(false)));
    assert!(!unique.matches(&stickers(true)));
}

#[test]
fn errors_come_from_the_part_that_is_wrong() {
    let kind = PuzzleKind::Cube2;
    let error = |schema: &str, guard: &str| {
        Template::parse(schema, guard, kind, &identity())
            .unwrap_err()
            .to_string()
    };
    assert_eq!(
        error(&uniform(2), "unique(q1)"),
        "line 1, column 8: variable \"q1\" does not occur in the schema"
    );
    assert_eq!(
        error("a a a a / b b b b", ""),
        "line 1, column 18: expected 6 faces, found 2"
    );
}

/// A small deterministic generator, so that the test needs no dependency.
struct Lcg(u64);

impl Lcg {
    fn step(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, n: usize) -> usize {
        (self.step() % n as u64) as usize
    }
}

#[test]
fn no_input_makes_the_parser_panic() {
    let mut rng = Lcg(7);
    let alphabet: Vec<char> = "abc123ABUZ.? ,/\n\t()!&|=<>*uniqeacotwhr_é€"
        .chars()
        .collect();
    let pieces = [
        "unique(", "equal(", "count(", "a*", "b*", "a1", "a2", ", ", ")", " && ", " || ", "!",
        " == ", " != ", " <= ", "3", "(", "not ", " AND ", " or ", "\n",
    ];
    let valid_names: Vec<String> = (1..=6).map(|k| format!("a{k}")).collect();
    for round in 0..20_000 {
        let len = rng.below(80);
        let noise: String = (0..len)
            .map(|_| alphabet[rng.below(alphabet.len())])
            .collect();
        let bytes: Vec<u8> = (0..len).map(|_| rng.step() as u8).collect();
        let lossy = String::from_utf8_lossy(&bytes).into_owned();
        let glued: String = (0..rng.below(14))
            .map(|_| pieces[rng.below(pieces.len())])
            .collect();
        for text in [&noise, &lossy, &glued] {
            let _ = Template::parse(text, text, PuzzleKind::Cube2, &identity());
            let _ = Template::parse(&uniform(2), text, PuzzleKind::Cube2, &identity());
            let _ = Guard::parse(text, &valid_names);
        }
        if round % 4 == 0 {
            // Real-looking grids with a random number of stickers.
            let stickers = rng.below(30);
            let face = vec!["a"; stickers].join(" ");
            let _ = Template::parse(
                &vec![face; 6].join(" / "),
                "",
                PuzzleKind::Cube3,
                &identity(),
            );
        }
    }
}

#[test]
fn huge_inputs_are_refused_not_expanded() {
    let schema = "a ".repeat(2_000_000);
    let error = Template::parse(&schema, "", PuzzleKind::Cube2, &identity()).unwrap_err();
    assert!(error.message().contains("more than 4 stickers"), "{error}");
}
