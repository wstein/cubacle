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
        let mut args = vec!["solve", "--fast"];
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
fn a_random_6x6_and_7x7_are_generated_and_solved() {
    for (size, kind) in [("6", PuzzleKind::Cube6), ("7", PuzzleKind::Cube7)] {
        let (ok, facelets, error) = cubacle(&["random", "--size", size]);
        assert!(ok, "{error}");
        let cube = Facelets::parse(&facelets).unwrap();
        assert_eq!(cube.kind(), kind);
        let (ok, solution, error) = cubacle(&["solve", &facelets]);
        assert!(ok, "{error}");
        assert!(solved_by(&cube, &solution), "{solution}");
    }
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
fn a_scramble_that_moves_the_centers_is_solved_by_color() {
    let (ok, solution, error) = cubacle(&["solve", "--scramble", "Rw U"]);
    assert!(ok, "{error}");
    assert!(error.contains("note: the face centers"), "{error}");
    let start = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"Rw U".parse().unwrap())
        .unwrap();
    let moves = solution.parse().expect("the solution is move notation");
    assert!(cubacle::puzzle::solves_by_color(&start, &moves));
    // With a target the engine still needs the centers in place.
    let (ok, _, error) = cubacle(&["solve", "--scramble", "Rw U", "--to", "R"]);
    assert!(!ok);
    assert!(error.contains("centers"), "{error}");
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
    // The middle layer moves the face centers; with a target no engine solves that.
    let (ok, _, error) = cubacle(&["solve", "--size", "5", "--scramble", "3R U", "--to", "R"]);
    assert!(!ok);
    assert!(error.contains("centers"), "{error}");
    let (ok, _, error) = cubacle(&["solve", "--size", "5", "-m", "5", "-s", "2R U 2F' R"]);
    assert!(!ok);
    assert!(error.contains("move limit"), "{error}");
    let (ok, _, error) = cubacle(&["solve", "--size", "9", "-s", "R"]);
    assert!(!ok);
    assert!(error.contains("size"), "{error}");
}

#[test]
fn moves_icons_text_on_a_pipe_has_no_escape_sequences() {
    let (ok, out, _) = cubacle(&["moves", "R", "U'", "--icons", "--format", "text"]);
    assert!(ok);
    assert!(!out.contains('\x1b'));
    assert!(out.starts_with("R \u{2191}"));
    assert!(out.contains("U' \u{2192}"));
    let (ok, auto, _) = cubacle(&["moves", "R", "U'", "--icons"]);
    assert!(ok);
    assert!(!auto.contains('\x1b'));
    assert!(auto.starts_with("move 1 of 2: R\nR \u{2191}"), "{auto}");
    assert!(auto.contains("move 2 of 2: U'\nU' \u{2192}"), "{auto}");
}

#[test]
fn moves_icons_svg_goes_to_stdout_and_graphics_are_refused_on_a_pipe() {
    let (ok, out, _) = cubacle(&["moves", "Rw2", "--size", "4", "--icons", "--format", "svg"]);
    assert!(ok);
    assert!(out.starts_with("<svg ") && out.contains("<title>Rw2</title>"));
    let (ok, out, err) = cubacle(&["moves", "R", "--icons", "--format", "kitty"]);
    assert!(!ok);
    assert!(!out.contains('\x1b'));
    assert!(err.contains("terminal"), "{err}");
    let (ok, _, err) = cubacle(&["moves", "R", "U", "--icons", "--format", "svg"]);
    assert!(!ok && err.contains("one icon"), "{err}");
    let (ok, _, err) = cubacle(&["moves", "3R", "--icons"]);
    assert!(!ok && err.contains("does not exist"), "{err}");
}

#[test]
fn moves_icons_write_graphics_to_a_file() {
    let dir = std::env::temp_dir().join(format!("cubacle-icon-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let kitty = dir.join("r.kitty");
    let (ok, _, err) = cubacle(&[
        "moves",
        "R",
        "--icons",
        "--format",
        "kitty",
        "--out",
        kitty.to_str().unwrap(),
    ]);
    assert!(ok, "{err}");
    let text = std::fs::read_to_string(&kitty).unwrap();
    // Erase six rows, step back up, draw without moving the cursor, step down.
    let head = "\x1b[2K\n".repeat(6) + "\x1b[6A\x1b_Ga=T,f=32,s=100,v=96,q=2,r=6,C=1,m=1;";
    assert!(text.starts_with(&head), "{:?}", &text[..80]);
    assert!(
        !text.contains(",c="),
        "only r is given so the aspect ratio holds"
    );
    assert!(text.contains("\x1b_Gm=0;"));
    assert!(text.ends_with("\x1b\\\x1b[6B\r"));
    let clear = dir.join("opaque.kitty");
    let (ok, _, err) = cubacle(&[
        "moves",
        "R",
        "--icons",
        "--format",
        "kitty",
        "--opaque",
        "--out",
        clear.to_str().unwrap(),
    ]);
    assert!(ok, "{err}");
    assert_ne!(
        std::fs::read_to_string(&clear).unwrap(),
        text,
        "--opaque changes the picture"
    );
    let rgba = dir.join("r.rgba");
    let (ok, _, err) = cubacle(&[
        "moves",
        "R",
        "--icons",
        "--icon-size",
        "32",
        "--format",
        "rgba",
        "--out",
        rgba.to_str().unwrap(),
    ]);
    assert!(ok, "{err}");
    assert_eq!(std::fs::metadata(&rgba).unwrap().len(), 33 * 32 * 4);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn solve_icons_on_a_pipe_is_plain_text_and_plain_solve_is_unchanged() {
    let (ok, plain, _) = cubacle(&["solve", "-s", "R U"]);
    assert!(ok);
    assert!(!plain.contains("move"));
    let (ok, icons, _) = cubacle(&["solve", "-s", "R U", "--icons"]);
    assert!(ok);
    assert!(!icons.contains('\x1b'));
    assert!(icons.contains("move 1 of "));
    assert!(icons.trim_end().ends_with(&plain));
}

#[test]
fn solve_icons_draws_the_start_state_before_move_one() {
    let (ok, icons, _) = cubacle(&["solve", "-s", "R U F", "--icons"]);
    assert!(ok);
    let lines: Vec<&str> = icons.lines().collect();
    assert!(lines[0].starts_with("Start state: "));
    // no heading between the token line and the picture
    assert!(lines[1].starts_with("state, folded view"), "{}", lines[1]);
    let step = icons.find("move 1 of ").unwrap();
    assert!(icons.find("state, folded view").unwrap() < step);
    let (_, urf, _) = cubacle(&["solve", "-s", "R U F", "--icons", "--views", "urf"]);
    assert!(
        urf.lines()
            .nth(1)
            .unwrap()
            .starts_with("state, seen from URF"),
        "{urf}"
    );
    // moves has no start state
    let (_, moves, _) = cubacle(&["moves", "R", "--icons", "--format", "text"]);
    assert!(!moves.contains("Start state"));
}

#[test]
fn solve_icons_prints_the_start_token_and_a_token_after_each_phase() {
    let (ok, start, _) = cubacle(&["convert", "--to", "token", "-s", "R U F"]);
    assert!(ok);
    let (ok, icons, _) = cubacle(&["solve", "-s", "R U F", "--icons"]);
    assert!(ok);
    let first = icons.lines().next().unwrap();
    assert_eq!(first, format!("Start state: {}", start.trim_end()));
    let (_, solved, _) = cubacle(&["convert", "--to", "token", "-s", ""]);
    let last = icons
        .lines()
        .rfind(|l| l.starts_with("End state: "))
        .unwrap();
    assert_eq!(last, format!("End state: {}", solved.trim_end()));
    let (_, plain, _) = cubacle(&["solve", "-s", "R U F"]);
    assert!(!plain.contains("token") && !plain.contains("Start"));
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
fn a_solved_7x7_token_is_read_and_solved() {
    let token = "A".repeat(90);
    assert!(cubacle(&["convert", &token]).0);
    let (ok, solution, error) = cubacle(&["solve", &token]);
    assert!(ok, "{error}");
    assert!(solution.trim().is_empty(), "{solution}");
}

#[test]
fn an_impossible_token_is_an_error() {
    let (ok, _, error) = cubacle(&["convert", &"_".repeat(43)]);
    assert!(!ok);
    assert!(error.contains("out of range"), "{error}");
}

#[test]
fn moves_icons_draw_each_move_on_its_own_start_cube_unless_cumulative() {
    let (_, both, _) = cubacle(&["moves", "R", "U", "--icons", "--format", "text"]);
    let (_, second, _) = cubacle(&["moves", "U", "--icons", "--format", "text"]);
    assert!(
        both.ends_with(&second),
        "U must be drawn on the solved cube"
    );
    let (_, chained, _) = cubacle(&[
        "moves",
        "R",
        "U",
        "--icons",
        "--format",
        "text",
        "--cumulative",
    ]);
    assert_ne!(chained, both);
    assert!(!chained.ends_with(&second));
    // --from sets the start cube of every icon.
    let (_, from, _) = cubacle(&[
        "moves", "U", "U", "--icons", "--from", "R", "--format", "text",
    ]);
    let (_, one, _) = cubacle(&["moves", "U", "--icons", "--from", "R", "--format", "text"]);
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
    let count = solution.split_whitespace().count();
    let mut lines = stderr.lines();
    assert_eq!(lines.next(), Some(format!("moves {count}").as_str()));
    let metrics = lines.next().expect("a metrics line");
    assert!(
        metrics.starts_with(&format!("metrics: htm {count}, qtm ")),
        "{metrics}"
    );
    assert_eq!(lines.next(), None);
}

#[test]
fn moves_icons_background_sets_the_notation_colour_and_opaque_paints_it() {
    let svg = |extra: &[&str]| {
        let mut args = vec!["moves", "R", "--icons", "--format", "svg"];
        args.extend_from_slice(extra);
        let (ok, out, err) = cubacle(&args);
        assert!(ok, "{err}");
        out
    };
    let (dark, light) = (svg(&[]), svg(&["--background", "light"]));
    assert_eq!(dark, svg(&["--background", "dark"]), "dark is the default");
    assert!(dark.contains("#f4f4f4") && !dark.contains("<rect"));
    assert!(light.contains("#14161a") && !light.contains("#f4f4f4"));
    assert!(
        svg(&["--opaque"]).contains("<rect width=\"114.00\" height=\"109.20\" fill=\"#20242b\"/>")
    );
    assert!(svg(&["--opaque", "--background", "light"]).contains("fill=\"#ffffff\"/>"));
    let (ok, _, err) = cubacle(&["moves", "R", "--icons", "--background", "grey"]);
    assert!(!ok && err.contains("invalid value"), "{err}");
}

#[test]
fn solve_icons_takes_a_background_and_plain_solve_does_not() {
    let (ok, out, err) = cubacle(&["solve", "-s", "R U", "--icons", "--background", "light"]);
    assert!(ok, "{err}");
    assert!(out.contains("move 1 of "));
    let (ok, _, err) = cubacle(&["solve", "-s", "R U", "--background", "light"]);
    assert!(!ok && err.contains("--icons"), "{err}");
}

#[test]
fn solve_icons_takes_columns_that_a_pipe_does_not_need() {
    let (ok, out, err) = cubacle(&["solve", "-s", "R U", "--icons", "--columns", "3"]);
    assert!(ok, "{err}");
    assert!(out.contains("move 1 of ") && !out.contains('\x1b'));
    for bad in ["0", "x"] {
        let (ok, _, err) = cubacle(&["solve", "-s", "R U", "--icons", "--columns", bad]);
        assert!(!ok && err.contains("invalid value"), "{err}");
    }
    let (ok, _, err) = cubacle(&["solve", "-s", "R U", "--columns", "3"]);
    assert!(!ok && err.contains("--icons"), "{err}");
}

#[test]
fn verbose_names_the_phases_on_stderr_and_stdout_stays_the_move_string() {
    let (ok, plain, _) = cubacle(&["solve", "-s", "R U F' D2 L B R2 U' F2 D"]);
    assert!(ok);
    let (ok, out, err) = cubacle(&["solve", "-s", "R U F' D2 L B R2 U' F2 D", "-v"]);
    assert!(ok, "{err}");
    assert_eq!(out, plain);
    let count: usize = err
        .lines()
        .find_map(|l| l.strip_prefix("moves "))
        .and_then(|n| n.parse().ok())
        .expect("a moves line");
    let phases: Vec<&str> = err.lines().filter(|l| l.starts_with("phase ")).collect();
    assert!(!phases.is_empty(), "{err}");
    assert!(phases[0].starts_with("phase 1: "), "{phases:?}");
    // the phases cover the solution to its last move; an empty one says so
    let last = phases
        .iter()
        .rev()
        .find(|l| !l.ends_with("nothing to do"))
        .expect("a phase with moves");
    assert!(last.ends_with(&format!("-{count}")), "{phases:?}");
}

#[test]
fn a_4x4_verbose_solve_reports_its_reduction() {
    let (ok, _, err) = cubacle(&["solve", "--size", "4", "-s", "R U Rw Fw2 U' 2R B", "-v"]);
    assert!(ok, "{err}");
    let phases: Vec<&str> = err.lines().filter(|l| l.starts_with("phase ")).collect();
    assert!(phases.iter().any(|l| l.contains("centres")), "{phases:?}");
    assert!(phases.last().unwrap().contains("3x3 stage"), "{phases:?}");
    // the engine's own stages: four, numbered in order, no `edges` phase
    assert_eq!(phases.len(), 4, "{phases:?}");
    let names = [
        "centres staged",
        "centres sorted and wings split",
        "centres and wings paired",
        "3x3 stage",
    ];
    for (i, (line, name)) in phases.iter().zip(names).enumerate() {
        assert!(
            line.starts_with(&format!("phase {}: {name}, ", i + 1)),
            "{phases:?}"
        );
    }
    assert!(!err.contains("edges"), "{err}");
}

#[test]
fn a_4x4_icons_solve_on_a_pipe_is_still_plain_text() {
    let args = ["solve", "--size", "4", "-s", "R U Rw Fw2 U' 2R B"];
    let (ok, plain, _) = cubacle(&args);
    assert!(ok);
    let mut with = args.to_vec();
    with.extend(["--icons", "-v"]);
    let (ok, icons, err) = cubacle(&with);
    assert!(ok, "{err}");
    assert!(
        !icons.contains('\x1b') && !icons.contains("Phase "),
        "{icons}"
    );
    assert!(icons.contains("move 1 of "));
    assert!(icons.trim_end().ends_with(&plain));
}

#[test]
fn icons_of_larger_cubes_are_drawn_larger() {
    let bytes = |cube: &str| {
        Command::new(env!("CARGO_BIN_EXE_cubacle"))
            .args(["moves", "--icons", "--format", "rgba", "--size", cube, "U"])
            .output()
            .expect("cubacle runs")
            .stdout
            .len()
    };
    // height x width (the canvas is 114 x 109.2) x 4 bytes
    assert_eq!(bytes("3"), 96 * 100 * 4);
    assert_eq!(bytes("7"), 224 * 234 * 4);
}

#[test]
fn graphics_flag_and_variable_choose_the_pictures() {
    let run = |args: &[&str], var: Option<&str>| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cubacle"));
        cmd.args(args).env_remove("CUBACLE_GRAPHICS");
        if let Some(v) = var {
            cmd.env("CUBACLE_GRAPHICS", v);
        }
        cmd.output().expect("cubacle runs")
    };
    let text = |o: &std::process::Output| String::from_utf8_lossy(&o.stdout).into_owned();
    let base = ["moves", "R", "--icons"];
    let flag = |g: &'static str| [base.as_slice(), &["--graphics", g]].concat();
    // a pipe: plain unless the choice is explicit
    assert!(!text(&run(&base, None)).contains('\x1b'));
    assert!(text(&run(&flag("ansi"), None)).contains("\x1b[48;2;"));
    assert!(text(&run(&flag("kitty"), None)).contains("\x1b_G"));
    // the variable works, and the flag wins over it
    assert!(text(&run(&base, Some("ansi"))).contains("\x1b[48;2;"));
    assert!(!text(&run(&flag("plain"), Some("ansi"))).contains('\x1b'));
    // an invalid value names the accepted ones
    let out = run(&base, Some("bogus"));
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("auto, kitty, ansi, plain"), "{err}");
}

#[test]
fn plain_solve_is_a_counted_improvement_and_fast_is_the_first_answer() {
    let cube = "A4ib8aejxj4Y";
    let (ok, fast, error) = cubacle(&["solve", "--fast", cube]);
    assert!(ok, "{error}");
    assert_eq!(fast, SOLUTION);
    let (ok, first, error) = cubacle(&["solve", cube]);
    assert!(ok, "{error}");
    let (_, second, _) = cubacle(&["solve", cube]);
    assert_eq!(first, second, "plain solve is deterministic");
    assert!(first.split_whitespace().count() <= fast.split_whitespace().count());
    let (_, probes, _) = cubacle(&["solve", "--probes", "3000", cube]);
    assert_eq!(first, probes, "the default is --probes 3000");
    let (ok, capped, error) = cubacle(&["solve", "-m", "21", cube]);
    assert!(ok, "{error}");
    assert_eq!(first, capped, "-m keeps the default improvement");
}

#[test]
fn state_pictures_have_no_headline_and_sit_in_the_rows() {
    let (ok, out, _) = cubacle(&[
        "solve",
        "-s",
        "R U F' D2 L B",
        "--icons",
        "--graphics",
        "kitty",
        "--columns",
        "4",
    ]);
    assert!(ok);
    assert!(!out.contains("compare it with your cube"), "{out:?}");
    assert!(!out.lines().any(|l| l == "Start state"), "{out:?}");
    // the start picture on rows of its own, two rows of moves, the end picture
    assert_eq!(out.matches("\x1b_Ga=T").count(), 4, "{out:?}");
    // the end state token comes above the end picture, which is last
    let end = out.find("End state: ").unwrap();
    assert!(end < out.rfind("\x1b_Ga=T").unwrap());
    assert!(!out.contains("State after phase"), "{out:?}");
}
