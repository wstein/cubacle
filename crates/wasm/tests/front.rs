//! ABI 3 `cubacle_front_solve`, called the way a WebAssembly host calls it: a
//! versioned text request in linear memory, a status byte and a text answer.
//!
//! The 80 cases are those of `tests/fixtures/front_goal_calendar.tsv` (the
//! research data of wstein/cubistry#484). The replay reads sticker identities
//! from the colors of the replayed cube with its own piece tables, not from the
//! solver's goal or tables.

use cubacle::puzzle::{rotation_moves, CenterTwists, Face, Facelets, PuzzleKind};
use cubacle_wasm::{
    cubacle_alloc, cubacle_free, cubacle_front_solve, cubacle_result_len, FRONT_STATUS_BAD_INPUT,
    FRONT_STATUS_NO_ROUTE, FRONT_STATUS_OK, FRONT_STATUS_SEARCH_BUDGET, FRONT_STATUS_TABLE_BUDGET,
    HEADER,
};

const FIXTURE: &str = include_str!("../../../tests/fixtures/front_goal_calendar.tsv");

const CORNERS: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];
const EDGES: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];

struct Case {
    print: String,
    step: String,
    expected: usize,
    center_up: bool,
    facelets: String,
    twists: String,
    accepted: String,
}

fn cases() -> Vec<Case> {
    FIXTURE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|line| {
            let c: Vec<&str> = line.split('\t').collect();
            Case {
                print: c[0].to_owned(),
                step: c[1].to_owned(),
                expected: c[3].parse().unwrap(),
                center_up: !c[4].split('|').nth(4).unwrap().is_empty(),
                facelets: c[5].to_owned(),
                twists: c[6].to_owned(),
                accepted: c[7].to_owned(),
            }
        })
        .collect()
}

/// A request: version, the case, and the limits (0 = default of each).
fn request(case: &Case, limits: [u64; 4]) -> String {
    request_v("1", case, limits)
}

fn request_v(version: &str, case: &Case, limits: [u64; 4]) -> String {
    format!(
        "{version}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        case.facelets,
        case.twists,
        case.accepted,
        u8::from(case.center_up),
        limits[0],
        limits[1],
        limits[2],
        limits[3]
    )
}

/// Runs `cubacle_front_solve` and returns the status byte and the text.
fn call(input: &str) -> (u8, String) {
    let bytes = input.as_bytes();
    let pointer = cubacle_alloc(bytes.len());
    // SAFETY: `pointer` was allocated with `bytes.len()` bytes just above.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
    // SAFETY: `pointer` and its length describe the buffer filled above.
    let result = unsafe { cubacle_front_solve(pointer, bytes.len()) };
    // SAFETY: the same allocation, released once.
    unsafe { cubacle_free(pointer, bytes.len()) };
    // SAFETY: `result` was just returned by `cubacle_front_solve`.
    let total = unsafe { cubacle_result_len(result) };
    // SAFETY: `result` points to `total` initialized bytes owned by the facade.
    let out = unsafe { std::slice::from_raw_parts(result, total) }.to_vec();
    // SAFETY: `result` came from the facade and is released once.
    unsafe { cubacle_free(result, total) };
    (out[4], String::from_utf8(out[HEADER..].to_vec()).unwrap())
}

/// The home facelet of the sticker shown at facelet `at`, read from colors.
fn identity_at(state: &Facelets, at: usize) -> usize {
    let color = |f: usize| "URFDLB".find(state.stickers()[f].letter()).unwrap();
    if at % 9 == 4 {
        return color(at) * 9 + 4;
    }
    let shown = color(at);
    for slot in CORNERS {
        if slot.contains(&at) {
            let mut seen: Vec<usize> = slot.iter().map(|&f| color(f)).collect();
            seen.sort_unstable();
            let piece = CORNERS.iter().find(|p| {
                let mut own: Vec<usize> = p.iter().map(|&f| f / 9).collect();
                own.sort_unstable();
                own == seen
            });
            return *piece.unwrap().iter().find(|&&f| f / 9 == shown).unwrap();
        }
    }
    for slot in EDGES {
        if slot.contains(&at) {
            let seen = [color(slot[0]), color(slot[1])];
            let piece = EDGES.iter().find(|p| {
                (p[0] / 9, p[1] / 9) == (seen[0], seen[1])
                    || (p[1] / 9, p[0] / 9) == (seen[0], seen[1])
            });
            return *piece.unwrap().iter().find(|&&f| f / 9 == shown).unwrap();
        }
    }
    unreachable!("every facelet is on a corner, an edge or a center")
}

/// The fields of an ok answer: moves, rotation, nodes, table milliseconds.
struct Answer {
    moves: String,
    rotation: String,
    nodes: u64,
    tables_ms: f64,
}

fn parse_ok(text: &str) -> Answer {
    let f: Vec<&str> = text.split('\t').collect();
    assert_eq!(f.len(), 5, "{text:?}");
    assert_eq!(f[0], "1", "answer version: {text:?}");
    Answer {
        moves: f[1].to_owned(),
        rotation: f[2].to_owned(),
        nodes: f[3].parse().unwrap(),
        tables_ms: f[4].parse().unwrap(),
    }
}

/// Replays the answer on the case: front goal met after the moves and the rotation.
fn replay(case: &Case, a: &Answer) -> Result<(), String> {
    let start = Facelets::parse(&case.facelets).unwrap();
    let moves = a.moves.parse().map_err(|e| format!("moves: {e}"))?;
    let end = start.apply(&moves).unwrap();
    let rotation = rotation_moves(&a.rotation).map_err(|e| format!("rotation: {e}"))?;
    let turned = end.apply(&rotation).unwrap();
    for (i, class) in case.accepted.split(' ').enumerate() {
        let identity = identity_at(&turned, 18 + i);
        if !class
            .split(',')
            .any(|n| n.parse::<usize>().unwrap() == identity)
        {
            return Err(format!("position {i} shows {identity}, accepted {class}"));
        }
    }
    if case.center_up {
        let twists: Vec<u8> = case.twists.split(',').map(|t| t.parse().unwrap()).collect();
        let twists = CenterTwists::new(PuzzleKind::Cube3, twists.try_into().unwrap()).unwrap();
        let tracked = twists.track(&moves).unwrap();
        if !tracked.frame.is_empty() {
            return Err("face turns moved the centers".into());
        }
        let seen = tracked.twists.rotated(&a.rotation).unwrap();
        if seen.twist(Face::F) != 0 {
            return Err("center picture not upright".into());
        }
    }
    Ok(())
}

#[test]
fn all_80_cases_give_the_484_optimum_and_replay() {
    let all = cases();
    assert_eq!(all.len(), 80);
    for case in &all {
        let (status, text) = call(&request(case, [0; 4]));
        assert_eq!(
            status, FRONT_STATUS_OK,
            "{} {}: {text}",
            case.print, case.step
        );
        let a = parse_ok(&text);
        assert_eq!(
            a.moves.split_whitespace().count(),
            case.expected,
            "{} {}: {text}",
            case.print,
            case.step
        );
        assert!(
            a.rotation.split_whitespace().count() <= 2,
            "{} {}: rotation {:?}",
            case.print,
            case.step,
            a.rotation
        );
        if let Err(why) = replay(case, &a) {
            panic!("{} {}: replay failed: {why}", case.print, case.step);
        }
    }
}

#[test]
fn the_answer_is_deterministic_except_the_table_time() {
    let case = &cases()[0];
    let first = parse_ok(&call(&request(case, [0; 4])).1);
    let second = parse_ok(&call(&request(case, [0; 4])).1);
    assert_eq!(first.moves, second.moves);
    assert_eq!(first.rotation, second.rotation);
    assert_eq!(first.nodes, second.nodes);
    assert!(first.tables_ms > 0.0 || second.tables_ms >= 0.0);
}

#[test]
fn an_unknown_version_is_refused_as_bad_input() {
    let case = &cases()[0];
    for version in ["2", "0", "", "x"] {
        let (status, text) = call(&request_v(version, case, [0; 4]));
        assert_eq!(status, FRONT_STATUS_BAD_INPUT, "version {version:?}");
        assert!(text.contains("version"), "{text}");
    }
}

#[test]
fn a_wrong_field_count_is_bad_input() {
    let case = &cases()[0];
    let short = request(case, [0; 4])
        .rsplit_once('\t')
        .unwrap()
        .0
        .to_owned();
    let (status, text) = call(&short);
    assert_eq!(status, FRONT_STATUS_BAD_INPUT, "{text}");
    assert!(text.contains("nine tab"), "{text}");
}

#[test]
fn a_truncated_accepted_list_is_bad_input() {
    let mut case = cases().remove(0);
    case.accepted = case
        .accepted
        .split(' ')
        .take(8)
        .collect::<Vec<_>>()
        .join(" ");
    let (status, text) = call(&request(&case, [0; 4]));
    assert_eq!(status, FRONT_STATUS_BAD_INPUT, "{text}");
    assert!(text.contains("nine"), "{text}");
}

#[test]
fn an_accepted_list_cut_to_one_sticker_changes_the_answer() {
    // Only the first accepted sticker per position: a smaller goal, so a route at
    // least as long, and for this case a different one.
    let mut case = cases().remove(0);
    let narrowed: Vec<&str> = case
        .accepted
        .split(' ')
        .map(|g| g.split(',').next().unwrap())
        .collect();
    let full = parse_ok(&call(&request(&case, [0; 4])).1);
    case.accepted = narrowed.join(" ");
    let (status, text) = call(&request(&case, [0; 4]));
    // Either no route, or a route that is not shorter.
    if status == FRONT_STATUS_OK {
        let a = parse_ok(&text);
        assert!(a.moves.split_whitespace().count() >= full.moves.split_whitespace().count());
        assert_ne!(
            (a.moves.clone(), a.rotation.clone()),
            (full.moves, full.rotation)
        );
    } else {
        assert_eq!(status, FRONT_STATUS_NO_ROUTE, "{text}");
    }
}

#[test]
fn center_up_is_honored() {
    // Find a case whose answer differs between center up and not: the replay with
    // the true flag must pass, and with the flag flipped the answer must fail the
    // true replay in at least one case.
    let mut differs = 0;
    for case in cases() {
        let mut flipped = Case {
            center_up: !case.center_up,
            ..clone(&case)
        };
        let a = call(&request(&flipped, [0; 4]));
        if a.0 != FRONT_STATUS_OK {
            continue;
        }
        let a = parse_ok(&a.1);
        flipped.center_up = case.center_up;
        if replay(&flipped, &a).is_err() {
            differs += 1;
        }
    }
    assert!(differs >= 1, "no case separates center up from free");
}

fn clone(c: &Case) -> Case {
    Case {
        print: c.print.clone(),
        step: c.step.clone(),
        expected: c.expected,
        center_up: c.center_up,
        facelets: c.facelets.clone(),
        twists: c.twists.clone(),
        accepted: c.accepted.clone(),
    }
}

#[test]
fn the_node_cap_is_a_limit() {
    let case = &cases()[0];
    let (status, text) = call(&request(case, [0, 0, 10, 0]));
    assert_eq!(status, FRONT_STATUS_SEARCH_BUDGET, "{text}");
    assert_eq!(text, "1\tnodes=11");
}

#[test]
fn the_table_budget_is_a_limit() {
    let case = &cases()[0];
    let (status, text) = call(&request(case, [1, 0, 0, 0]));
    assert_eq!(status, FRONT_STATUS_TABLE_BUDGET, "{text}");
    assert!(text.starts_with("1\tpermille="), "{text}");
}

#[test]
fn the_move_limit_is_a_limit() {
    let case = &cases()[0];
    let (status, text) = call(&request(case, [0, 0, 0, 1]));
    assert_eq!(status, FRONT_STATUS_NO_ROUTE, "{text}");
}

#[test]
fn limits_that_suffice_do_not_change_the_answer() {
    let case = &cases()[0];
    let free = parse_ok(&call(&request(case, [0; 4])).1);
    let (status, text) = call(&request(case, [60_000, 60_000, 50_000_000, 21]));
    assert_eq!(status, FRONT_STATUS_OK, "{text}");
    let bounded = parse_ok(&text);
    assert_eq!(
        (bounded.moves, bounded.rotation),
        (free.moves, free.rotation)
    );
}

#[test]
fn not_utf8_and_bad_fields_are_bad_input() {
    let case = &cases()[0];
    for broken in [
        "1\tzz\t0,0,0,0,0,0\t1 2 3 4 5 6 7 8 9\t0\t0\t0\t0\t0".to_owned(),
        request(case, [0; 4]).replacen(&case.twists, "1,2", 1),
        request(case, [0; 4]).replacen(&format!("\t{}\t", u8::from(case.center_up)), "\t2\t", 1),
        request(case, [0; 4]).replacen('\t', "\tq", 1),
    ] {
        let (status, text) = call(&broken);
        assert_eq!(status, FRONT_STATUS_BAD_INPUT, "{broken}: {text}");
    }
}
