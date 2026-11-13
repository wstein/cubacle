//! Specification of the twist condition of a target and the start twist of a solve.

use super::{
    parse_twist_masks, twist_masks_text, CenterTwists, Face, Facelets, PuzzleKind, SolveOptions,
    Target, TargetError, TwistError, FREE_TWIST,
};

fn twists(t: [u8; 6]) -> CenterTwists {
    CenterTwists::new(PuzzleKind::Cube3, t).unwrap()
}

#[test]
fn a_twist_condition_has_alternatives_and_open_centers() {
    let masks = parse_twist_masks("0,X,2|3,?,1,X").unwrap();
    assert_eq!(masks, [1, 15, 12, 15, 2, 15]);
    assert_eq!(twist_masks_text(&masks), "0,X,2|3,X,1,X");
    assert_eq!(parse_twist_masks("X,X,X,X,X,X").unwrap(), FREE_TWIST);
}

#[test]
fn a_twist_condition_refuses_junk() {
    for bad in [
        "",
        "0,1",
        "0,1,2,3,4,0",
        "0,1,2,3,0,0,0",
        "0||1,X,X,X,X,X",
        "a,X,X,X,X,X",
    ] {
        assert!(
            matches!(parse_twist_masks(bad), Err(TwistError::Syntax(_))),
            "{bad:?}"
        );
    }
}

#[test]
fn a_target_is_free_of_twist_conditions_until_one_is_set() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let target = Target::complete(&solved);
    assert_eq!(target.twist_masks(), &FREE_TWIST);
    assert!(!target.has_twist_condition());
    assert!(target.twist_met(&twists([3, 1, 2, 0, 1, 2])));
    // a twist condition makes the target a different target
    let picky = target
        .clone()
        .with_twist(parse_twist_masks("0,X,X,X,X,X").unwrap())
        .unwrap();
    assert_ne!(picky, target);
    assert!(picky.has_twist_condition());
}

#[test]
fn a_twist_condition_is_met_when_every_center_shows_an_accepted_twist() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let target = Target::complete(&solved)
        .with_twist(parse_twist_masks("0,X,2|3,X,X,1").unwrap())
        .unwrap();
    assert!(target.twist_met(&twists([0, 3, 2, 1, 1, 1])));
    assert!(target.twist_met(&twists([0, 0, 3, 0, 0, 1])));
    assert!(!target.twist_met(&twists([1, 3, 2, 1, 1, 1])), "U");
    assert!(!target.twist_met(&twists([0, 3, 1, 1, 1, 1])), "F");
    assert!(!target.twist_met(&twists([0, 3, 2, 1, 1, 0])), "B");
}

#[test]
fn masked_targets_keep_a_free_twist_condition() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let target = Target::masked(&solved, |position| position < 9);
    assert!(!target.has_twist_condition());
}

#[test]
fn a_start_twist_implies_the_exact_search() {
    let options = SolveOptions::default();
    assert_eq!(options.start_twist, None);
    assert!(!options.exact);
    let start = twists([0, 1, 2, 0, 3, 0]);
    let options = options.with_start_twist(start);
    assert_eq!(options.start_twist, Some(start));
    assert!(options.exact);
}

#[test]
fn every_way_to_make_a_target_starts_free_of_twist_conditions() {
    let from_sets = Target::from_sets(vec![0b11_1111; 54]).unwrap();
    assert_eq!(from_sets.twist_masks(), &FREE_TWIST);
    assert!(!from_sets.has_twist_condition());
    let solved = Facelets::solved(PuzzleKind::Cube3);
    assert_eq!(Target::complete(&solved).twist_masks(), &FREE_TWIST);
    assert_eq!(
        Target::masked(&solved, |position| position < 9).twist_masks(),
        &FREE_TWIST
    );
}

#[test]
fn a_twist_mask_keeps_its_low_four_bits() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let target = Target::complete(&solved)
        .with_twist([0xF1, 0xF2, 0xF4, 0xF8, 0x1F, 0xFF])
        .unwrap();
    assert_eq!(target.twist_masks(), &[1, 2, 4, 8, 15, 15]);
}

#[test]
fn a_center_that_accepts_no_twist_is_an_error() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let target = Target::complete(&solved);
    for masks in [[0, 15, 15, 15, 15, 15], [15, 15, 15, 15, 15, 0xF0]] {
        let error = target.clone().with_twist(masks).unwrap_err();
        let face = if masks[0] == 0 { Face::U } else { Face::B };
        assert_eq!(error, TargetError::EmptyTwistSet { face });
        assert!(error.to_string().contains(face.letter()), "{error}");
    }
    // the text form never writes an empty item, so it always parses back
    let ok = target.with_twist([1, 15, 5, 8, 15, 15]).unwrap();
    let text = twist_masks_text(ok.twist_masks());
    assert_eq!(parse_twist_masks(&text).unwrap(), *ok.twist_masks());
}
