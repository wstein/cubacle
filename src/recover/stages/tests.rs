use super::*;

#[test]
fn the_window_ends_at_the_move_the_person_stopped_at() {
    let w = window(&Scope { at_move: 10 }, 0);
    // Move 10 is position 9; later moves are ignored.
    assert!(w.allows(0, 9, 10));
    assert!(!w.allows(0, 10, 10));
}

#[test]
fn a_mistake_may_shift_the_count_of_used_moves_by_the_slack() {
    let w = window(&Scope { at_move: 10 }, 0);
    assert!(w.allows(3, 3, 8));
    assert!(w.allows(3, 3, 12));
    assert!(!w.allows(3, 3, 7));
    assert!(!w.allows(3, 3, 13));
}

#[test]
fn a_window_near_the_start_does_not_underflow() {
    let w = window(&Scope { at_move: 1 }, 0);
    assert!(w.allows(0, 0, 0));
    assert!(!w.allows(0, 1, 1));
}

#[test]
fn the_first_stage_reaches_back_to_the_start() {
    assert_eq!(plan(&Scope { at_move: 5 }), [(Widening::Prefix, 0)]);
}
