use cubacle::puzzle::{Facelets, PuzzleKind};
use std::process::Command;

const SOLUTION: &str = "B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R";

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        // never the user's real table cache
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
    assert_eq!(cubacle(&["convert", "--scheme", "jp", japanese]).1, solved);
    let (ok, _, error) = cubacle(&["convert", "--scheme", "boy", japanese]);
    assert!(!ok);
    assert!(error.contains("centres"), "{error}");
}

#[test]
fn unsupported_sizes_say_so() {
    let solved_6x6: String = "URFDLB".chars().flat_map(|c| [c; 36]).collect();
    assert_eq!(solved_6x6.len(), 216);
    let (ok, _, error) = cubacle(&["solve", &solved_6x6]);
    assert!(!ok);
    assert!(error.contains("no engine for the 6x6x6"), "{error}");
}

#[test]
fn a_4x4_scramble_is_solved_and_the_solution_replays() {
    let scramble = "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw";
    let cube = Facelets::solved(PuzzleKind::Cube4)
        .apply(&scramble.parse().unwrap())
        .unwrap();
    let (ok, solution, error) = cubacle(&["solve", "--size", "4", "--scramble", scramble]);
    assert!(ok, "{error}");
    assert!(solved_by(&cube, &solution), "{solution}");
    // The same cube as typed facelets gets the same solution.
    let (ok, again, error) = cubacle(&["solve", &cube.to_string()]);
    assert!(ok, "{error}");
    assert_eq!(again, solution);
}

#[test]
fn a_random_4x4_is_generated_and_solved() {
    let (ok, facelets, error) = cubacle(&["random", "--size", "4"]);
    assert!(ok, "{error}");
    let cube = Facelets::parse(&facelets).unwrap();
    assert_eq!(cube.kind(), PuzzleKind::Cube4);
    let (ok, solution, error) = cubacle(&["solve", &facelets]);
    assert!(ok, "{error}");
    assert!(solved_by(&cube, &solution));
}

#[test]
fn a_4x4_failure_is_reported_and_never_replayed_wrongly() {
    let (ok, _, error) = cubacle(&[
        "solve",
        "--size",
        "4",
        "-m",
        "30",
        "-s",
        "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw",
    ]);
    assert!(!ok);
    assert!(error.contains("move limit"), "{error}");
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
        vec!["solve", "--scheme", "boy", &colours],
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

fn solved_by(facelets: &Facelets, solution: &str) -> bool {
    let moves = solution.parse().expect("the solution is move notation");
    facelets.apply(&moves).is_ok_and(|end| end.is_solved())
}

#[test]
fn a_5x5_scramble_is_solved_and_the_solution_replays() {
    let scramble = "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F";
    let cube = Facelets::solved(PuzzleKind::Cube5)
        .apply(&scramble.parse().unwrap())
        .unwrap();
    let (ok, solution, error) = cubacle(&["solve", "--size", "5", "--scramble", scramble]);
    assert!(ok, "{error}");
    assert!(solved_by(&cube, &solution), "{solution}");
    // The same cube as typed facelets gets the same solution.
    let (ok, again, error) = cubacle(&["solve", &cube.to_string()]);
    assert!(ok, "{error}");
    assert_eq!(again, solution);
}

#[test]
fn a_random_5x5_is_generated_and_solved() {
    let (ok, facelets, error) = cubacle(&["random", "--size", "5"]);
    assert!(ok, "{error}");
    let cube = Facelets::parse(&facelets).unwrap();
    assert_eq!(cube.kind(), PuzzleKind::Cube5);
    let (ok, solution, error) = cubacle(&["solve", &facelets]);
    assert!(ok, "{error}");
    assert!(solved_by(&cube, &solution));
}

#[test]
fn a_5x5_scramble_is_printed_for_the_requested_size() {
    let (ok, text, error) = cubacle(&["scramble", "--size", "5", "-n", "40"]);
    assert!(ok, "{error}");
    let moves: cubacle::puzzle::MoveSequence = text.parse().unwrap();
    assert_eq!(moves.len(), 40);
    assert!(moves.moves().iter().all(|m| m.fits(PuzzleKind::Cube5)));
}

#[test]
fn a_5x5_failure_is_reported_and_never_replayed_wrongly() {
    // The middle layer moves the face centres, which no engine solves.
    let (ok, _, error) = cubacle(&["solve", "--size", "5", "--scramble", "3R U"]);
    assert!(!ok);
    assert!(error.contains("centres"), "{error}");
    let (ok, _, error) = cubacle(&["solve", "--size", "5", "-m", "5", "-s", "2R U 2F' R"]);
    assert!(!ok);
    assert!(error.contains("move limit"), "{error}");
    let (ok, _, error) = cubacle(&["solve", "--size", "9", "-s", "R"]);
    assert!(!ok);
    assert!(error.contains("size"), "{error}");
}

#[test]
fn icon_text_on_a_pipe_has_no_escape_sequences() {
    let (ok, out, _) = cubacle(&["icon", "R", "U'", "--format", "text"]);
    assert!(ok);
    assert!(!out.contains('\x1b'));
    assert!(out.starts_with("R \u{2191}"));
    assert!(out.contains("U' \u{2192}"));
    let (ok, auto, _) = cubacle(&["icon", "R", "U'"]);
    assert!(ok);
    assert_eq!(auto, out);
}

#[test]
fn icon_svg_goes_to_stdout_and_graphics_are_refused_on_a_pipe() {
    let (ok, out, _) = cubacle(&["icon", "Rw2", "--cube", "4", "--format", "svg"]);
    assert!(ok);
    assert!(out.starts_with("<svg ") && out.contains("<title>Rw2</title>"));
    let (ok, out, err) = cubacle(&["icon", "R", "--format", "kitty"]);
    assert!(!ok);
    assert!(!out.contains('\x1b'));
    assert!(err.contains("terminal"), "{err}");
    let (ok, _, err) = cubacle(&["icon", "R", "U", "--format", "svg"]);
    assert!(!ok && err.contains("one icon"), "{err}");
    let (ok, _, err) = cubacle(&["icon", "3R"]);
    assert!(!ok && err.contains("does not exist"), "{err}");
}

#[test]
fn icon_writes_graphics_to_a_file() {
    let dir = std::env::temp_dir().join(format!("cubacle-icon-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let kitty = dir.join("r.kitty");
    let (ok, _, err) = cubacle(&[
        "icon",
        "R",
        "--format",
        "kitty",
        "--out",
        kitty.to_str().unwrap(),
    ]);
    assert!(ok, "{err}");
    let text = std::fs::read_to_string(&kitty).unwrap();
    // Erase six rows, step back up, draw without moving the cursor, step down.
    let head = "\x1b[2K\n".repeat(6) + "\x1b[6A\x1b_Ga=T,f=32,s=144,v=96,q=2,r=6,C=1,m=1;";
    assert!(text.starts_with(&head), "{:?}", &text[..80]);
    assert!(
        !text.contains(",c="),
        "only r is given so the aspect ratio holds"
    );
    assert!(text.contains("\x1b_Gm=0;"));
    assert!(text.ends_with("\x1b\\\x1b[6B\r"));
    let clear = dir.join("clear.kitty");
    let (ok, _, err) = cubacle(&[
        "icon",
        "R",
        "--format",
        "kitty",
        "--transparent",
        "--out",
        clear.to_str().unwrap(),
    ]);
    assert!(ok, "{err}");
    assert_ne!(
        std::fs::read_to_string(&clear).unwrap(),
        text,
        "opaque by default"
    );
    let rgba = dir.join("r.rgba");
    let (ok, _, err) = cubacle(&[
        "icon",
        "R",
        "--size",
        "32",
        "--format",
        "rgba",
        "--out",
        rgba.to_str().unwrap(),
    ]);
    assert!(ok, "{err}");
    assert_eq!(std::fs::metadata(&rgba).unwrap().len(), 48 * 32 * 4);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn solve_icons_on_a_pipe_is_plain_text_and_plain_solve_is_unchanged() {
    let (ok, plain, _) = cubacle(&["solve", "-s", "R U"]);
    assert!(ok);
    assert!(!plain.contains("step"));
    let (ok, icons, _) = cubacle(&["solve", "-s", "R U", "--icons"]);
    assert!(ok);
    assert!(!icons.contains('\x1b'));
    assert!(icons.contains("step 1 of "));
    assert!(icons.trim_end().ends_with(&plain));
}

#[test]
fn convert_writes_orbit64_tokens_for_every_size() {
    for (size, width) in [(2, 5), (3, 12), (4, 27), (5, 43), (6, 66), (7, 90)] {
        let scramble = if size < 4 {
            "R U F' R2 U'"
        } else {
            "R U 2R 2U' F"
        };
        let size = size.to_string();
        let (ok, facelets, error) = cubacle(&["facelets", scramble, "--size", &size]);
        assert!(ok, "{error}");
        let (ok, token, error) = cubacle(&["convert", "--to", "token", &facelets]);
        assert!(ok, "{size}: {error}");
        assert_eq!(token.trim().len(), width, "{size}");
        // The token is read back by its length alone.
        assert_eq!(cubacle(&["convert", token.trim()]).1, facelets, "{size}");
    }
}

#[test]
fn a_token_without_an_engine_is_read_and_then_refused_by_solve() {
    let token = "A".repeat(90);
    assert!(cubacle(&["convert", &token]).0);
    let (ok, _, error) = cubacle(&["solve", &token]);
    assert!(!ok);
    assert!(error.contains("no engine for the 7x7x7"), "{error}");
}

#[test]
fn an_impossible_token_is_an_error() {
    let (ok, _, error) = cubacle(&["convert", &"_".repeat(43)]);
    assert!(!ok);
    assert!(error.contains("out of range"), "{error}");
}

#[test]
fn icon_draws_each_move_on_its_own_start_cube_unless_cumulative() {
    let (_, both, _) = cubacle(&["icon", "R", "U", "--format", "text"]);
    let (_, second, _) = cubacle(&["icon", "U", "--format", "text"]);
    assert!(
        both.ends_with(&second),
        "U must be drawn on the solved cube"
    );
    let (_, chained, _) = cubacle(&["icon", "R", "U", "--format", "text", "--cumulative"]);
    assert_ne!(chained, both);
    assert!(!chained.ends_with(&second));
    // --from sets the start cube of every icon.
    let (_, from, _) = cubacle(&["icon", "U", "U", "--from", "R", "--format", "text"]);
    let (_, one, _) = cubacle(&["icon", "U", "--from", "R", "--format", "text"]);
    assert_eq!(from, format!("{one}\n\n{one}"));
}

/// The token of a 4x4x4 from a user report; its answer must replay with and
/// without `--slices`, use no inner layer by default and be no longer with them.
#[test]
fn a_4x4_token_is_solved_with_and_without_slices() {
    let token = "Bs6Qrv1Dp0motqndUv7rAuyA9Kb";
    let cube = cubacle::notation::orbit64::decode(token).expect("a 4x4x4 token");
    let run = |extra: &[&str]| {
        let mut args = vec!["solve", token];
        args.extend_from_slice(extra);
        let (ok, out, err) = cubacle(&args);
        assert!(ok, "{err}");
        assert!(solved_by(&cube, &out), "{out}");
        out
    };
    let plain = run(&[]);
    let sliced = run(&["--slices"]);
    let inner = |text: &str| {
        text.split_whitespace()
            .any(|m| m.starts_with(|c: char| c.is_ascii_digit()) && !m.contains('w'))
    };
    assert!(!inner(&plain), "{plain}");
    assert!(sliced.split_whitespace().count() <= plain.split_whitespace().count());
}

#[test]
fn solve_always_reports_the_move_count() {
    let (ok, solution, stderr) = cubacle(&["solve", "A4ib8aejxj4Y"]);
    assert!(ok);
    assert_eq!(
        stderr,
        format!("moves {}", solution.split_whitespace().count())
    );
}

#[test]
fn icons_of_larger_cubes_are_drawn_larger() {
    let bytes = |cube: &str| {
        Command::new(env!("CARGO_BIN_EXE_cubacle"))
            .args(["icon", "--format", "rgba", "--cube", cube, "U"])
            .output()
            .expect("cubacle runs")
            .stdout
            .len()
    };
    // height x 1.5 height x 4 bytes
    assert_eq!(bytes("3"), 96 * 144 * 4);
    assert_eq!(bytes("7"), 224 * 336 * 4);
}
