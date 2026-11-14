//! Tests of the four-cubie group tables and of the gate that stops a build.

use std::collections::HashSet;

use super::super::cubie::Cubie;
use super::exact_tests::cases;
use super::gate::{Gate, Stop};
use super::goal::Goal;
use super::groups::{plan_groups, GroupTable, Orbit};
use super::model::{Pos, Turns, TURNS};
use super::tables::Built;

fn table_of(goal: &Goal, turns: &Turns, tracked: &[(Orbit, u8); 4]) -> GroupTable {
    match GroupTable::build(goal, turns, tracked, &mut Gate::none()) {
        Ok(Built::Table(t)) => t,
        _ => panic!("a table was expected"),
    }
}

fn pos_of(case: &super::exact_tests::Case) -> Pos {
    let mut cube = Cubie::new();
    assert_eq!(cube.set_from_facelet(&case.start.to_string()), 0);
    Pos::from_cubie(&cube)
}

#[test]
fn the_plan_has_two_four_or_nine_groups_of_four_distinct_cubies() {
    let case = &cases()[0];
    let goal = Goal::new(&case.views);
    for (count, expected) in [(2, 2), (4, 4), (9, 9)] {
        let plan = plan_groups(&goal, count);
        assert_eq!(plan.len(), expected);
        for group in &plan {
            let set: HashSet<_> = group.iter().map(|&(o, p)| (o as u8, p)).collect();
            assert_eq!(set.len(), 4, "{group:?}");
        }
    }
    let nine = plan_groups(&goal, 9);
    assert!(nine[0].iter().all(|&(o, _)| o == Orbit::Corner));
    assert!(nine[2].iter().all(|&(o, _)| o == Orbit::Edge));
    // the two and four group sets are prefixes of the choices, not other tables
    let two = plan_groups(&goal, 2);
    assert_eq!(two[0], nine[0]);
    assert_eq!(two[1], nine[2]);
    let four = plan_groups(&goal, 4);
    assert_eq!(four[..], [nine[0], nine[2], nine[3], nine[5]]);
}

#[test]
fn no_group_bound_exceeds_the_optimum_and_the_nine_are_not_trivial() {
    let turns = Turns::new();
    let mut strongest = 0;
    for case in cases() {
        let goal = Goal::new(&case.views);
        let pos = pos_of(&case);
        for tracked in plan_groups(&goal, 9) {
            let table = table_of(&goal, &turns, &tracked);
            let h = usize::from(table.bound(table.index(&pos)));
            assert!(
                h <= case.expected,
                "bound {h} above the optimum {}",
                case.expected
            );
            strongest = strongest.max(h);
        }
    }
    assert!(
        strongest >= 4,
        "the tables prune nothing: best root bound {strongest}"
    );
}

#[test]
fn a_group_bound_drops_by_at_most_one_per_move() {
    let turns = Turns::new();
    let case = &cases()[0];
    let goal = Goal::new(&case.views);
    let tracked = plan_groups(&goal, 9)[3];
    let table = table_of(&goal, &turns, &tracked);
    let mut pos = pos_of(case);
    let mut at = table.index(&pos);
    for step in 0..60 {
        let m = (step * 7 + step / 3) % TURNS;
        let next = pos.step(&turns, m);
        let next_at = table.step(at, m);
        assert_eq!(
            next_at,
            table.index(&next),
            "step {step}: the coordinate step and the index disagree"
        );
        let (a, b) = (table.bound(at), table.bound(next_at));
        assert!(
            a <= b + 1 && b <= a + 1,
            "bounds {a} and {b} differ by more than one move"
        );
        pos = next;
        at = next_at;
    }
}

#[test]
fn a_goal_met_at_the_start_has_bound_zero_in_every_group() {
    let turns = Turns::new();
    let case = &cases()[0];
    let goal = Goal::new(&case.views);
    let mut cube = Cubie::new();
    assert_eq!(cube.set_from_facelet(&case.start.to_string()), 0);
    // a state that meets the goal: the end of the optimum route of the full search
    let route = super::exact::solve_goal(
        crate::puzzle::Instant::now(),
        &cube,
        &goal,
        &crate::puzzle::SolveOptions::default(),
    )
    .outcome
    .expect("a route");
    let end = case.start.apply(&route).unwrap();
    let mut end_cube = Cubie::new();
    assert_eq!(end_cube.set_from_facelet(&end.to_string()), 0);
    let end_pos = Pos::from_cubie(&end_cube);
    assert!(goal.is_met(&end_pos));
    for tracked in plan_groups(&goal, 9) {
        let table = table_of(&goal, &turns, &tracked);
        assert_eq!(table.bound(table.index(&end_pos)), 0);
    }
}

#[test]
fn a_gate_with_no_time_stops_at_once_and_a_cancel_stops_with_the_progress_so_far() {
    let mut gate = Gate::new(Some(crate::puzzle::Instant::now()), None);
    gate.stage(0, 1);
    assert_eq!(gate.check(10), Err(Stop::Budget(10)));

    let mut seen = Vec::new();
    let mut cancel = |permille: u16| {
        seen.push(permille);
        seen.len() < 3
    };
    let mut gate = Gate::new(None, Some(&mut cancel));
    gate.stage(1, 2);
    assert_eq!(gate.check(100), Ok(()));
    assert_eq!(gate.check(200), Ok(()));
    assert_eq!(gate.check(300), Err(Stop::Cancelled(650)));
    assert_eq!(seen, [550, 600, 650]);
}

#[test]
fn a_gate_reports_each_step_forward_once_and_never_backward() {
    let mut seen = Vec::new();
    let mut observer = |permille: u16| {
        seen.push(permille);
        true
    };
    let mut gate = Gate::new(None, Some(&mut observer));
    gate.stage(0, 2);
    for p in [0, 0, 100, 100, 50, 400, 1000] {
        gate.check(p).unwrap();
    }
    gate.stage(1, 2);
    for p in [0, 500, 1000] {
        gate.check(p).unwrap();
    }
    gate.finish();
    assert_eq!(seen, [0, 50, 200, 500, 750, 1000]);
}
