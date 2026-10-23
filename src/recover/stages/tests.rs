use super::*;

fn scope(at_move: usize, sure_until: Option<usize>, phase_starts: &[usize]) -> Scope {
    Scope {
        at_move,
        sure_until,
        phase_starts: phase_starts.to_vec(),
    }
}

const PHASES: [usize; 4] = [0, 10, 20, 35];

#[test]
fn the_window_ends_at_the_move_the_person_stopped_at() {
    let w = window(&Scope::at(10), 0);
    // Move 10 is position 9; later moves are ignored.
    assert!(w.allows(0, 9, 10));
    assert!(!w.allows(0, 10, 10));
}

#[test]
fn a_mistake_may_shift_the_count_of_used_moves_by_the_slack() {
    let w = window(&Scope::at(10), 0);
    assert!(w.allows(3, 3, 8));
    assert!(w.allows(3, 3, 12));
    assert!(!w.allows(3, 3, 7));
    assert!(!w.allows(3, 3, 13));
}

#[test]
fn a_window_near_the_start_does_not_underflow() {
    let w = window(&Scope::at(1), 0);
    assert!(w.allows(0, 0, 0));
    assert!(!w.allows(0, 1, 1));
}

#[test]
fn a_window_starts_where_it_is_told_to() {
    let w = window(&Scope::at(40), 35);
    assert!(w.allows(35, 39, 40));
    assert!(!w.allows(34, 39, 40));
}

#[test]
fn the_phase_of_a_position_is_the_last_one_that_started() {
    assert_eq!(phase_index(&PHASES, 0), 0);
    assert_eq!(phase_index(&PHASES, 9), 0);
    assert_eq!(phase_index(&PHASES, 10), 1);
    assert_eq!(phase_index(&PHASES, 39), 3);
    assert_eq!(phase_index(&[], 5), 0);
}

#[test]
fn without_phases_the_only_window_is_the_prefix() {
    assert_eq!(plan(&Scope::at(5)), [(Widening::Prefix, 0)]);
}

#[test]
fn the_stages_widen_from_the_phase_of_the_stop_to_the_start() {
    assert_eq!(
        plan(&scope(40, None, &PHASES)),
        [
            (Widening::Phase, 35),
            (Widening::PreviousPhase, 20),
            (Widening::Prefix, 0)
        ]
    );
}

#[test]
fn a_stop_in_the_first_phase_has_no_previous_phase() {
    assert_eq!(
        plan(&scope(5, None, &PHASES)),
        [(Widening::Phase, 0)],
        "the prefix is the phase"
    );
}

#[test]
fn a_stop_in_the_second_phase_widens_to_the_first_and_stops() {
    assert_eq!(
        plan(&scope(12, None, &PHASES)),
        [(Widening::Phase, 10), (Widening::PreviousPhase, 0)]
    );
}

#[test]
fn the_move_the_person_is_sure_of_comes_first() {
    assert_eq!(
        plan(&scope(40, Some(37), &PHASES)),
        [
            (Widening::Sure, 37),
            (Widening::Phase, 35),
            (Widening::PreviousPhase, 20),
            (Widening::Prefix, 0)
        ]
    );
}

#[test]
fn a_sure_move_before_the_phase_makes_the_phase_stages_redundant() {
    assert_eq!(
        plan(&scope(40, Some(5), &PHASES)),
        [(Widening::Sure, 5), (Widening::Prefix, 0)]
    );
}

#[test]
fn a_sure_move_at_the_stop_is_no_hint() {
    assert_eq!(
        plan(&scope(40, Some(40), &PHASES)),
        plan(&scope(40, None, &PHASES))
    );
}
