use std::cell::Cell;
use std::time::Duration;

use super::*;
use crate::notation::parse_alphabet;

fn solver(set: &str) -> SubsetSolver {
    SubsetSolver::new(parse_alphabet(set, PuzzleKind::Cube3).unwrap()).unwrap()
}

fn cube(moves: &str) -> Facelets {
    let sequence: MoveSequence = moves.parse().unwrap();
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&sequence)
        .unwrap()
}

const FACES: &str = "U U2 U' R R2 R' F F2 F' D D2 D' L L2 L' B B2 B'";

/// Every solution listed for `scramble`, as text, with the listing report.
fn list(set: &str, scramble: &str, request: EnumerateRequest) -> (Vec<String>, Listing) {
    let state = cube(scramble);
    let mut found = Vec::new();
    let listing = solver(set)
        .enumerate(
            &state,
            &request,
            &SolveOptions::default(),
            &mut |solution| {
                assert!(state.apply(solution).unwrap().is_solved());
                found.push(solution.to_string());
                true
            },
            &|| false,
        )
        .expect("a listing");
    (found, listing)
}

fn shortest_only() -> EnumerateRequest {
    EnumerateRequest {
        longest: None,
        count: None,
        raw: false,
    }
}

#[test]
fn a_unique_shortest_solution_is_listed_once() {
    let (found, listing) = list("<R,U>", "R U", shortest_only());
    assert_eq!(found, ["U' R'"]);
    assert_eq!(listing.shortest, 2);
    assert_eq!(listing.end, EnumerateEnd::Complete);
}

#[test]
fn commuting_moves_are_listed_in_one_order_unless_raw_is_asked_for() {
    let (canonical, _) = list(FACES, "R L", shortest_only());
    assert_eq!(canonical.len(), 1, "{canonical:?}");
    let raw = EnumerateRequest {
        raw: true,
        ..shortest_only()
    };
    let (every, _) = list(FACES, "R L", raw);
    assert_eq!(every.len(), 2, "{every:?}");
}

#[test]
fn opposite_faces_and_their_slice_commute_too() {
    // U D and the E slice share an axis: of the orders only one is listed.
    let (found, _) = list("U D U' D' 2U", "U D", shortest_only());
    assert_eq!(found.len(), 1, "{found:?}");
}

#[test]
fn longer_solutions_follow_the_shortest_in_order_of_length() {
    let request = EnumerateRequest {
        longest: Some(8),
        ..shortest_only()
    };
    let (found, listing) = list(FACES, "R U F D' L B2", request);
    let lengths: Vec<usize> = found.iter().map(|s| s.split(' ').count()).collect();
    assert_eq!(lengths[0], listing.shortest);
    assert!(lengths.windows(2).all(|w| w[0] <= w[1]), "{lengths:?}");
    assert!(*lengths.last().unwrap() <= 8);
    let unique: std::collections::HashSet<&String> = found.iter().collect();
    assert_eq!(unique.len(), found.len(), "no solution is listed twice");
    assert_eq!(listing.found, found.len());
    // No padding: two turns of one face in a row are always one turn or none.
    for line in &found {
        let faces: Vec<char> = line.split(' ').filter_map(|m| m.chars().next()).collect();
        assert!(faces.windows(2).all(|w| w[0] != w[1]), "{line}");
    }
}

#[test]
fn a_count_stops_the_listing() {
    let request = EnumerateRequest {
        longest: Some(8),
        count: Some(3),
        raw: false,
    };
    let (found, listing) = list(FACES, "R2 U2 F2", request);
    assert_eq!(found.len(), 3);
    assert_eq!(listing.end, EnumerateEnd::CountReached);
}

#[test]
fn the_caller_can_stop_the_search_at_any_node() {
    let state = cube("R U R' F2 D L'");
    let calls = Cell::new(0u64);
    let result = solver(FACES).enumerate(
        &state,
        &EnumerateRequest {
            longest: Some(9),
            count: None,
            raw: false,
        },
        &SolveOptions::default(),
        &mut |_| true,
        &|| {
            calls.set(calls.get() + 1);
            calls.get() > 2_000
        },
    );
    // The shortest length is found first; the listing, or that search, was cut.
    match result {
        Ok(listing) => assert_eq!(listing.end, EnumerateEnd::Cancelled),
        Err(failed) => assert!(failed.outcome.is_err()),
    }
    assert!(calls.get() <= 2_001, "the check runs at every node");
}

#[test]
fn a_cube_outside_the_group_gets_no_listing() {
    let state = cube("R");
    let result = solver("half-turns").enumerate(
        &state,
        &shortest_only(),
        &SolveOptions::default(),
        &mut |_| true,
        &|| false,
    );
    let failed = result.expect_err("not solvable");
    assert!(failed.message.unwrap().contains("not solvable"));
}

#[test]
fn a_spent_budget_ends_the_listing_with_a_failure() {
    let state = cube("R U R' U' R U2 R' U R U'");
    let options = SolveOptions::default().with_time_budget(Duration::ZERO);
    let result =
        solver("<R,U>").enumerate(&state, &shortest_only(), &options, &mut |_| true, &|| false);
    assert_eq!(
        result.expect_err("no time").outcome,
        Err(Failure::BudgetExhausted)
    );
}

#[test]
fn the_solved_cube_has_the_empty_solution() {
    let (found, listing) = list("<R,U>", "", shortest_only());
    assert_eq!(found, [""]);
    assert_eq!(listing.shortest, 0);
}
