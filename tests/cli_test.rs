use cubacle::puzzle::{Facelets, PuzzleKind};
use std::process::Command;

const SOLUTION: &str = "B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R";

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn every_notation_of_one_cube_gets_the_same_solution() {
    let notations: [&[&str]; 5] = [
        &["A4ib8aejxj4Y"],
        &["FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL"],
        &[
            "FUFUUFFFF",
            "UUUURRURU",
            "RRRFFRRFR",
            "BDBBDDBBB",
            "DLDDLLDDD",
            "LBLBBLLLL",
        ],
        &["GWGWWGGGGWWWWRRWRWRRRGGRRGRBYBBYYBBBYOYYOOYYYOBOBBOOOO"],
        &[
            "GWGWWGGGG",
            "WWWWRRWRW",
            "RRRGGRRGR",
            "BYBBYYBBB",
            "YOYYOOYYY",
            "OBOBBOOOO",
        ],
    ];
    for cube in notations {
        let mut args = vec!["solve"];
        args.extend(cube);
        let (ok, solution, error) = cubacle(&args);
        assert!(ok, "{cube:?}: {error}");
        assert_eq!(solution, SOLUTION, "{cube:?}");
    }
}

#[test]
fn convert_reads_the_japanese_scheme() {
    let japanese = "WWWWWWWWWRRRRRRRRRGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY";
    let solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    assert_eq!(cubacle(&["convert", japanese]).1, solved);
    assert_eq!(
        cubacle(&["convert", "--scheme", "japanese-bl-wt", japanese]).1,
        solved
    );
    let (ok, _, error) = cubacle(&["convert", "--scheme", "western-boy", japanese]);
    assert!(!ok);
    assert!(error.contains("centres"), "{error}");
}

#[test]
fn unsupported_sizes_say_so() {
    let solved_4x4: String = "URFDLB".chars().flat_map(|c| [c; 16]).collect();
    let (ok, _, error) = cubacle(&["solve", &solved_4x4]);
    assert!(!ok);
    assert!(error.contains("no engine for the 4x4x4"), "{error}");
}

/// Corner sticker positions of the 3x3 in facelet order, per corner slot.
const CORNERS_3X3: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];

/// The 2x2 facelet index of a 3x3 corner facelet.
fn thinned(facelet: usize) -> usize {
    facelet / 9 * 4 + facelet % 9 / 3 / 2 * 2 + facelet % 3 / 2
}

/// How one move rearranges the 24 stickers of a 2x2, derived from the 3x3
/// engine: `result[destination] = source`.
fn sticker_perm(mv: &str) -> [usize; 24] {
    let cube3 = |moves: &str| {
        Facelets::solved(PuzzleKind::Cube3)
            .apply(&moves.parse().unwrap())
            .unwrap()
            .to_string()
    };
    let solved: Vec<char> = cube3("").chars().collect();
    let moved: Vec<char> = cube3(mv).chars().collect();
    let mut perm: [usize; 24] = std::array::from_fn(|i| i);
    for slot in CORNERS_3X3 {
        let mut colours: Vec<char> = slot.iter().map(|&f| moved[f]).collect();
        colours.sort();
        let home = CORNERS_3X3
            .iter()
            .find(|home| {
                let mut c: Vec<char> = home.iter().map(|&f| solved[f]).collect();
                c.sort();
                c == colours
            })
            .unwrap();
        for &dest in &slot {
            let src = home.iter().find(|&&f| solved[f] == moved[dest]).unwrap();
            perm[thinned(dest)] = thinned(*src);
        }
    }
    perm
}

/// Replays a solution on 2x2 facelets and reports whether every face ended
/// up one colour.
fn solves_2x2(facelets: &str, solution: &str) -> bool {
    let mut state: Vec<char> = facelets.chars().collect();
    for mv in solution.split_whitespace() {
        let perm = sticker_perm(mv);
        state = perm.iter().map(|&src| state[src]).collect();
    }
    state
        .chunks(4)
        .all(|face| face.iter().all(|&c| c == face[0]))
}

#[test]
fn a_2x2_is_solved_from_every_notation() {
    // The same cube as an Orbit64 token, as facelets and as colours.
    let token = "ASj79";
    let facelets = "LFDRFLDBFURLBBLUURUDDFRB";
    assert_eq!(cubacle(&["convert", token]).1, facelets);
    let colours = facelets
        .chars()
        .map(|c| match c {
            'U' => 'W',
            'R' => 'R',
            'F' => 'G',
            'D' => 'Y',
            'L' => 'O',
            _ => 'B',
        })
        .collect::<String>();

    let mut lengths = Vec::new();
    for args in [
        vec!["solve", token],
        vec!["solve", facelets],
        vec!["solve", "--scheme", "western-boy", &colours],
    ] {
        let (ok, solution, error) = cubacle(&args);
        assert!(ok, "{args:?}: {error}");
        assert!(solves_2x2(facelets, &solution), "{args:?}: {solution}");
        lengths.push(solution.split_whitespace().count());
    }
    assert!(lengths.iter().all(|&n| n == lengths[0] && n > 0));
}

#[test]
fn scrambled_and_rotated_2x2_cubes_get_optimal_solutions() {
    // (moves applied to a solved 2x2, optimal solution length)
    for (scramble, length) in [
        ("R", 1),
        ("R U", 2),
        ("R U F'", 3),
        ("D L2 B'", 3),
        ("R L'", 0),
        ("R L' U D' F2 B2", 0),
        ("U R2 F' D", 4),
    ] {
        let mut state: Vec<char> = "UUUURRRRFFFFDDDDLLLLBBBB".chars().collect();
        for mv in scramble.split_whitespace() {
            state = sticker_perm(mv).iter().map(|&src| state[src]).collect();
        }
        let facelets: String = state.into_iter().collect();
        let (ok, solution, error) = cubacle(&["solve", &facelets]);
        assert!(ok, "{scramble}: {error}");
        assert!(solves_2x2(&facelets, &solution), "{scramble}: {solution}");
        assert_eq!(solution.split_whitespace().count(), length, "{scramble}");
    }
}

#[test]
fn a_2x2_respects_the_move_limit() {
    let (ok, _, error) = cubacle(&["solve", "-m", "2", "ASj79"]);
    assert!(!ok);
    assert!(error.contains("move limit"), "{error}");
    let (ok, _, error) = cubacle(&["solve", "-m", "11", "-b", "10000", "ASj79"]);
    assert!(ok, "{error}");
}

#[test]
fn an_impossible_2x2_is_rejected() {
    // One corner twisted in place: stickers U,R,F of URF become R,F,U.
    let solved = "UUUURRRRFFFFDDDDLLLLBBBB";
    let mut chars: Vec<char> = solved.chars().collect();
    let slot = [3, 4, 9];
    chars.swap(slot[0], slot[1]);
    chars.swap(slot[1], slot[2]);
    let twisted: String = chars.into_iter().collect();
    let (ok, _, error) = cubacle(&["solve", &twisted]);
    assert!(!ok);
    assert!(error.contains("impossible"), "{error}");
}

#[test]
fn a_tight_move_limit_fails_quickly() {
    let (ok, _, error) = cubacle(&["solve", "-m", "3", "A4ib8aejxj4Y"]);
    assert!(!ok);
    assert!(error.contains("move limit"), "{error}");
}

#[test]
fn a_scramble_that_moves_the_centres_is_reported_not_mis_solved() {
    let (ok, _, error) = cubacle(&["solve", "--scramble", "Rw U"]);
    assert!(!ok);
    assert!(error.contains("centres"), "{error}");
    assert!(!error.contains("internal error"), "{error}");
}
