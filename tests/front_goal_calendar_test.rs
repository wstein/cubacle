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

use cubacle::engines::three::{
    solve_front_goal, solve_front_goal_observed, FrontFailure, FrontGoal, FrontLimits, FrontTables,
};
use cubacle::puzzle::{rotation_moves, rotation_words, CenterTwists, Facelets, PuzzleKind};

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
            "{} step {}: length differs from the wstein/cubistrywstein/cubistry#484 optimum",
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
        Err(FrontFailure::BudgetExhausted { nodes }) => assert_eq!(nodes, 11),
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

/// A cube that is solved but for `moves`, with its (home) center twists.
fn scrambled(moves: &str) -> (Facelets, CenterTwists) {
    let start = Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves.parse().unwrap())
        .unwrap();
    (start, CenterTwists::new(PuzzleKind::Cube3, [0; 6]).unwrap())
}

/// The goal "the front shows its own stickers".
fn own_front() -> FrontGoal {
    let lists: Vec<[u8; 1]> = (18..27).map(|i| [i]).collect();
    let classes: Vec<&[u8]> = lists.iter().map(|l| &l[..]).collect();
    FrontGoal::new(&classes, false).unwrap()
}

fn invalid_text(result: Result<cubacle::engines::three::FrontRoute, FrontFailure>) -> String {
    match result {
        Err(FrontFailure::InvalidInput(text)) => text,
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

#[test]
fn a_time_limit_too_large_to_add_means_no_time_limit() {
    let (start, twists) = scrambled("R2");
    let limits = FrontLimits::default().with_time(Duration::MAX);
    let route = solve_front_goal(&start, &twists, &own_front(), &limits).unwrap();
    assert_eq!(route.moves.len(), 1);
}

#[test]
fn twists_that_do_not_match_the_corner_parity_are_invalid() {
    let (start, _) = scrambled("R2");
    let odd = CenterTwists::new(PuzzleKind::Cube3, [1, 0, 0, 0, 0, 0]).unwrap();
    let text = invalid_text(solve_front_goal(
        &start,
        &odd,
        &own_front(),
        &FrontLimits::default(),
    ));
    assert!(text.contains("center twists"), "{text}");
}

#[test]
fn a_start_of_another_kind_is_invalid() {
    let (_, twists) = scrambled("R2");
    let two = Facelets::solved(PuzzleKind::Cube2);
    let text = invalid_text(solve_front_goal(
        &two,
        &twists,
        &own_front(),
        &FrontLimits::default(),
    ));
    assert!(text.contains("3x3x3"), "{text}");
}

#[test]
fn an_impossible_cube_is_invalid() {
    let (solved, twists) = scrambled("");
    // One flipped edge: the stickers of facelets 5 and 10 swapped.
    let mut letters: Vec<char> = solved.to_string().chars().collect();
    letters.swap(5, 10);
    let flipped = Facelets::parse(&letters.iter().collect::<String>()).unwrap();
    let text = invalid_text(solve_front_goal(
        &flipped,
        &twists,
        &own_front(),
        &FrontLimits::default(),
    ));
    assert!(text.contains("impossible"), "{text}");
}

#[test]
fn a_goal_needs_exactly_nine_lists() {
    let one: &[u8] = &[24];
    assert!(FrontGoal::new(&[one; 8], false).is_err());
    assert!(FrontGoal::new(&[one; 10], false).is_err());
    assert!(FrontGoal::new(&[], false).is_err());
}

#[test]
fn a_goal_no_state_meets_is_no_route() {
    // Every position accepts only the center of the front: impossible.
    let center: &[u8] = &[22];
    let goal = FrontGoal::new(&[center; 9], false).unwrap();
    let (start, twists) = scrambled("R2");
    assert!(matches!(
        solve_front_goal(&start, &twists, &goal, &FrontLimits::default()),
        Err(FrontFailure::NoRoute(_))
    ));
}

#[test]
fn the_time_limit_is_read_every_65536_nodes() {
    // A search that ends within 65,536 nodes never looks at the clock.
    let (start, twists) = scrambled("R2");
    let zero = FrontLimits::default().with_time(Duration::ZERO);
    assert!(solve_front_goal(&start, &twists, &own_front(), &zero).is_ok());
    // One that needs more (de-a step 2 needs 166,681) stops at the first reading.
    let case = &cases("de-a")[1];
    assert_eq!(case.step, 2);
    match solve_front_goal(&case.start, &case.twists, &goal_of(case), &zero) {
        Err(FrontFailure::BudgetExhausted { nodes }) => assert_eq!(nodes, 65_536),
        other => panic!("expected BudgetExhausted, got {other:?}"),
    }
}

#[test]
fn of_several_fitting_rotations_the_first_in_rotation_words_order_is_reported() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let turned = |word: &str| solved.apply(&rotation_moves(word).unwrap()).unwrap();
    // Each position accepts its own sticker and the one a turn about y shows.
    let y = turned("y");
    let lists: Vec<[u8; 2]> = (0..9)
        .map(|i| [(18 + i) as u8, identity_at(&y, 18 + i) as u8])
        .collect();
    let classes: Vec<&[u8]> = lists.iter().map(|l| &l[..]).collect();
    let goal = FrontGoal::new(&classes, false).unwrap();
    let fitting: Vec<String> = rotation_words()
        .into_iter()
        .filter(|w| {
            let shown = turned(w);
            (0..9).all(|i| lists[i].contains(&(identity_at(&shown, 18 + i) as u8)))
        })
        .collect();
    assert!(fitting.len() >= 2, "{fitting:?}");
    let (start, twists) = scrambled("");
    let route = solve_front_goal(&start, &twists, &goal, &FrontLimits::default()).unwrap();
    assert!(route.moves.is_empty());
    assert_eq!(route.rotation, fitting[0], "all fitting: {fitting:?}");
}

#[test]
fn the_default_table_set_is_the_full_one() {
    assert_eq!(
        FrontLimits::default(),
        FrontLimits::default().with_tables(FrontTables::Full)
    );
    assert_ne!(
        FrontLimits::default(),
        FrontLimits::default().with_tables(FrontTables::Groups9)
    );
}

#[test]
fn two_four_and_nine_group_tables_find_the_484_optimum_and_replay() {
    for tables in [
        FrontTables::Groups2,
        FrontTables::Groups4,
        FrontTables::Groups9,
    ] {
        let limits = FrontLimits::default()
            .with_tables(tables)
            .with_time(Duration::from_secs(120));
        for print in ["de-a", "en-3-zero"] {
            for case in cases(print).iter().take(4) {
                let goal = goal_of(case);
                let route = solve_front_goal(&case.start, &case.twists, &goal, &limits)
                    .unwrap_or_else(|f| panic!("{tables:?} {print} step {}: {f:?}", case.step));
                assert_eq!(
                    route.moves.len(),
                    case.expected,
                    "{tables:?} {print} step {}",
                    case.step
                );
                assert!(
                    reaches_front_goal(case, &route),
                    "{tables:?} {print} step {}",
                    case.step
                );
            }
        }
    }
}

#[test]
fn more_group_tables_visit_fewer_nodes() {
    let case = &cases("de-a")[0];
    let goal = goal_of(case);
    let nodes = |tables| {
        let limits = FrontLimits::default().with_tables(tables);
        solve_front_goal(&case.start, &case.twists, &goal, &limits)
            .unwrap()
            .nodes
    };
    let (two, nine) = (nodes(FrontTables::Groups2), nodes(FrontTables::Groups9));
    assert!(nine < two, "nine tables {nine} nodes, two tables {two}");
}

#[test]
fn a_table_budget_of_zero_ends_the_build_not_the_search() {
    let case = &cases("de-a")[0];
    let goal = goal_of(case);
    for tables in [FrontTables::Full, FrontTables::Groups9] {
        let limits = FrontLimits::default()
            .with_tables(tables)
            .with_table_time(Duration::ZERO);
        match solve_front_goal(&case.start, &case.twists, &goal, &limits) {
            Err(FrontFailure::BuildBudgetExhausted { permille }) => assert!(permille < 1000),
            other => panic!("{tables:?}: expected BuildBudgetExhausted, got {other:?}"),
        }
    }
}

#[test]
fn the_table_budget_does_not_limit_the_search() {
    let case = &cases("de-a")[0];
    let goal = goal_of(case);
    let limits = FrontLimits::default()
        .with_tables(FrontTables::Groups9)
        .with_table_time(Duration::from_secs(120))
        .with_max_nodes(10);
    assert!(matches!(
        solve_front_goal(&case.start, &case.twists, &goal, &limits),
        Err(FrontFailure::BudgetExhausted { .. })
    ));
}

#[test]
fn the_build_reports_progress_upward_and_ends_at_1000() {
    let case = &cases("de-a")[0];
    let goal = goal_of(case);
    let limits = FrontLimits::default().with_tables(FrontTables::Groups9);
    let mut seen: Vec<u16> = Vec::new();
    let mut observer = |p: u16| {
        seen.push(p);
        true
    };
    solve_front_goal_observed(&case.start, &case.twists, &goal, &limits, &mut observer).unwrap();
    assert!(seen.windows(2).all(|w| w[0] < w[1]), "{seen:?}");
    assert_eq!(seen.last(), Some(&1000), "{seen:?}");
    assert!(seen.len() >= 9, "one report per table at least: {seen:?}");
}

#[test]
fn a_cancel_stops_inside_the_first_of_the_two_full_tables() {
    let case = &cases("de-a")[0];
    let goal = goal_of(case);
    let mut calls = 0usize;
    let mut observer = |_: u16| {
        calls += 1;
        calls < 20
    };
    let result = solve_front_goal_observed(
        &case.start,
        &case.twists,
        &goal,
        &FrontLimits::default(),
        &mut observer,
    );
    match result {
        Err(FrontFailure::Cancelled { permille }) => {
            assert!(
                permille < 500,
                "stopped in the second table or later: {permille}"
            )
        }
        other => panic!("expected Cancelled, got {other:?}"),
    }
    assert_eq!(calls, 20);
}
