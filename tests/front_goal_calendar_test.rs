//! The front goal of the 3x3 calendar through the public front-goal API.
//!
//! The 80 cases (20 steps from 2027-01-01 for each of four prints) and their
//! expected lengths come from the research data of wstein/cubistry#484, commit
//! dcd60a2203d30d9bc79ad8c454e355930c07c5aa; see the header of
//! `tests/fixtures/front_goal_calendar.tsv`. Run with `--release`: each case
//! builds about 44 MB of pruning tables.
//!
//! The replay reads sticker identities from the colors of the replayed cube
//! with its own piece tables, not from the solver's goal or tables.

use std::time::Duration;

use cubacle::engines::three::{solve_front_goal, FrontFailure, FrontGoal, FrontLimits};
use cubacle::puzzle::{rotation_moves, CenterTwists, Facelets, PuzzleKind};

const FIXTURE: &str = include_str!("fixtures/front_goal_calendar.tsv");

/// Home facelets of the corner slots and edge slots (U R F D L B, nine each).
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
    step: usize,
    expected: usize,
    cells: Vec<String>,
    start: Facelets,
    twists: CenterTwists,
    accepted: Vec<Vec<u8>>,
}

fn cases(print: &str) -> Vec<Case> {
    FIXTURE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|line| {
            let c: Vec<&str> = line.split('\t').collect();
            let twists: Vec<u8> = c[6].split(',').map(|t| t.parse().unwrap()).collect();
            Case {
                print: c[0].to_owned(),
                step: c[1].parse().unwrap(),
                expected: c[3].parse().unwrap(),
                cells: c[4].split('|').map(str::to_owned).collect(),
                start: Facelets::parse(c[5]).unwrap(),
                twists: CenterTwists::new(PuzzleKind::Cube3, twists.try_into().unwrap()).unwrap(),
                accepted: c[7]
                    .split(' ')
                    .map(|g| g.split(',').map(|n| n.parse().unwrap()).collect())
                    .collect(),
            }
        })
        .filter(|case| case.print == print)
        .collect()
}

fn goal_of(case: &Case) -> FrontGoal {
    let classes: Vec<&[u8]> = case.accepted.iter().map(Vec::as_slice).collect();
    // The center up rule holds when the image on the center is not blank.
    FrontGoal::new(&classes, !case.cells[4].is_empty()).unwrap()
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

/// Whether `start` turned by `moves` and then by `rotation` shows the goal on the front.
fn reaches_front_goal(case: &Case, route: &cubacle::engines::three::FrontRoute) -> bool {
    let end = case.start.apply(&route.moves).unwrap();
    let turned = end
        .apply(&rotation_moves(&route.rotation).unwrap())
        .unwrap();
    for (i, class) in case.accepted.iter().enumerate() {
        let identity = identity_at(&turned, 18 + i);
        if !class.iter().any(|&a| usize::from(a) == identity) {
            return false;
        }
    }
    if !case.cells[4].is_empty() {
        // Face turns keep the centers home; the reported rotation carries the twists along.
        let tracked = case.twists.track(&route.moves).unwrap();
        assert!(tracked.frame.is_empty(), "face turns keep the centers home");
        let seen = tracked.twists.rotated(&route.rotation).unwrap();
        if seen.twist(cubacle::puzzle::Face::F) != 0 {
            return false;
        }
    }
    true
}

fn check_print(print: &str) {
    let cases = cases(print);
    assert_eq!(cases.len(), 20, "{print}");
    let limits = FrontLimits::default().with_time(Duration::from_secs(120));
    for case in &cases {
        let goal = goal_of(case);
        let route = solve_front_goal(&case.start, &case.twists, &goal, &limits)
            .unwrap_or_else(|f| panic!("{} step {}: {f:?}", case.print, case.step));
        eprintln!(
            "{} step {}: {} moves, rotation {:?}, {} nodes, tables {:?}",
            case.print,
            case.step,
            route.moves.len(),
            route.rotation,
            route.nodes,
            route.tables
        );
        assert_eq!(
            route.moves.len(),
            case.expected,
            "{} step {}: length differs from the #484 optimum",
            case.print,
            case.step
        );
        assert!(
            reaches_front_goal(case, &route),
            "{} step {}: the replayed route misses the front goal after rotation {:?}",
            case.print,
            case.step,
            route.rotation
        );
    }
}

#[test]
fn de_a_matches_the_484_optimum_and_replays() {
    check_print("de-a");
}

#[test]
fn de_a_unpadded_matches_the_484_optimum_and_replays() {
    check_print("de-a-unpadded");
}

#[test]
fn en_3_zero_matches_the_484_optimum_and_replays() {
    check_print("en-3-zero");
}

#[test]
fn en_3_unpadded_matches_the_484_optimum_and_replays() {
    check_print("en-3-unpadded");
}

#[test]
fn a_tiny_node_budget_ends_in_budget_exhausted() {
    let case = &cases("de-a")[0];
    let goal = goal_of(case);
    let limits = FrontLimits::default().with_max_nodes(10);
    match solve_front_goal(&case.start, &case.twists, &goal, &limits) {
        Err(FrontFailure::BudgetExhausted { nodes }) => assert!(nodes >= 10, "{nodes}"),
        other => panic!("expected BudgetExhausted, got {other:?}"),
    }
}

#[test]
fn a_node_budget_one_short_of_the_need_fails_and_the_need_succeeds() {
    let case = &cases("en-3-unpadded")[0];
    let goal = goal_of(case);
    let free = FrontLimits::default().with_time(Duration::from_secs(120));
    let need = solve_front_goal(&case.start, &case.twists, &goal, &free)
        .unwrap()
        .nodes;
    let exact = FrontLimits::default().with_max_nodes(need);
    assert!(solve_front_goal(&case.start, &case.twists, &goal, &exact).is_ok());
    let short = FrontLimits::default().with_max_nodes(need - 1);
    assert!(matches!(
        solve_front_goal(&case.start, &case.twists, &goal, &short),
        Err(FrontFailure::BudgetExhausted { .. })
    ));
}

#[test]
fn an_invalid_goal_is_refused() {
    let one: &[u8] = &[24];
    let empty: &[u8] = &[];
    assert!(FrontGoal::new(&[empty; 9], false).is_err());
    assert!(FrontGoal::new(&[&[54u8][..]; 9], false).is_err());
    assert!(FrontGoal::new(&[one; 9], false).is_ok());
}
