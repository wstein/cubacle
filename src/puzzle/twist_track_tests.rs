//! Specification of the center twist tracking on `CenterTwists` (3x3x3).

use super::twist_track::{center_rotations, rotation_moves, rotation_words};
use super::{
    rotation_maps, CenterTwists, Face, Facelets, MoveSequence, PuzzleKind, Turn, TwistError,
};

fn seq(text: &str) -> MoveSequence {
    text.parse().unwrap()
}

fn upright() -> CenterTwists {
    CenterTwists::untwisted(PuzzleKind::Cube3).unwrap()
}

fn twists(t: [u8; 6]) -> CenterTwists {
    CenterTwists::new(PuzzleKind::Cube3, t).unwrap()
}

fn array(t: &CenterTwists) -> [u8; 6] {
    Face::ALL.map(|f| t.twist(f))
}

#[test]
fn a_face_move_adds_to_its_own_center_only() {
    for (i, face) in Face::ALL.into_iter().enumerate() {
        for (suffix, add) in [("", 1), ("2", 2), ("'", 3)] {
            let moves = seq(&format!("{}{suffix}", face.letter()));
            let tracked = upright().track(&moves).unwrap();
            let mut want = [0u8; 6];
            want[i] = add;
            assert_eq!(array(&tracked.twists), want, "{moves}");
            assert!(tracked.frame.is_empty());
        }
    }
}

#[test]
fn the_sum_parity_is_the_quarter_turn_parity_on_random_words() {
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for _ in 0..200 {
        let len = 1 + (next() % 20) as usize;
        let text: Vec<String> = (0..len)
            .map(|_| {
                let face = Face::ALL[(next() % 6) as usize].letter();
                let suffix = ["", "2", "'"][(next() % 3) as usize];
                format!("{face}{suffix}")
            })
            .collect();
        let moves = seq(&text.join(" "));
        let quarters: usize = moves
            .moves()
            .iter()
            .map(|m| if m.turn == Turn::Half { 2 } else { 1 })
            .sum();
        let tracked = upright().track(&moves).unwrap();
        assert_eq!(
            tracked.twists.sum_is_even(),
            quarters.is_multiple_of(2),
            "{moves}"
        );
    }
}

#[test]
fn a_move_followed_by_its_inverse_leaves_the_twists_alone() {
    let start = twists([1, 2, 3, 0, 1, 2]);
    let tracked = start.track(&seq("R U F2 F2 U' R'")).unwrap();
    assert_eq!(tracked.twists, start);
}

#[test]
fn the_24_rotations_agree_with_the_sticker_maps_on_the_centers() {
    let rotations = center_rotations();
    assert_eq!(rotations.len(), 24);
    assert_eq!(rotation_words().len(), 24);
    let solved = Facelets::solved(PuzzleKind::Cube3);
    for (map, rotation) in rotation_maps(PuzzleKind::Cube3).iter().zip(&rotations) {
        for g in 0..6 {
            assert_eq!(map[g * 9 + 4] / 9, rotation.dest[g], "{}", rotation.word);
        }
        let turned = solved
            .apply(&rotation_moves(&rotation.word).unwrap())
            .unwrap();
        let mut by_map = solved.stickers().to_vec();
        for (i, &to) in map.iter().enumerate() {
            by_map[to] = solved.stickers()[i];
        }
        assert_eq!(turned.stickers(), by_map.as_slice(), "{}", rotation.word);
    }
}

#[test]
fn a_rotation_effect_is_the_same_in_the_tracker_and_the_table() {
    let start = twists([1, 2, 0, 3, 1, 1]);
    for rotation in center_rotations() {
        let seen = array(&start.rotated(&rotation.word).unwrap());
        let before = array(&start);
        for g in 0..6 {
            assert_eq!(
                seen[rotation.dest[g]],
                (before[g] + rotation.add[g]) & 3,
                "{}",
                rotation.word
            );
        }
    }
}

#[test]
fn four_quarters_of_a_rotation_are_the_identity() {
    for axis in ["x", "y", "z"] {
        let start = twists([1, 2, 0, 3, 1, 1]);
        let word = format!("{axis} {axis} {axis} {axis}");
        assert_eq!(start.rotated(&word).unwrap(), start, "{word}");
    }
}

#[test]
fn a_whole_cube_rotation_is_turned_back_and_costs_no_move() {
    let tracked = upright().track(&rotation_moves("y").unwrap()).unwrap();
    assert!(!tracked.frame.is_empty());
    assert!(rotation_words().contains(&tracked.frame));
    assert_eq!(tracked.twists, upright());
    let there_and_back = upright().track(&rotation_moves("x x'").unwrap()).unwrap();
    assert_eq!(there_and_back.twists, upright());
    assert!(there_and_back.frame.is_empty());
}

#[test]
fn a_middle_slice_ends_in_a_home_frame() {
    let slice = upright().track(&seq("2R")).unwrap();
    assert!(!slice.frame.is_empty());
    assert!(rotation_words().contains(&slice.frame));
    let cancels = upright().track(&seq("L' 2R 2R'")).unwrap();
    assert_eq!(array(&cancels.twists), [0, 0, 0, 0, 3, 0]);
    assert!(cancels.frame.is_empty());
}

#[test]
fn the_text_form_round_trips_and_refuses_junk() {
    let t = CenterTwists::parse(PuzzleKind::Cube3, "0,1,2,0,3,0").unwrap();
    assert_eq!(array(&t), [0, 1, 2, 0, 3, 0]);
    assert_eq!(t.to_string(), "0,1,2,0,3,0");
    assert_eq!(
        CenterTwists::parse(PuzzleKind::Cube3, " 0, 1,2,0,3 ,0").unwrap(),
        t
    );
    for bad in [
        "",
        "0,1,2,0,3",
        "0,1,2,0,3,4",
        "a,b,c,d,e,f",
        "0,1,2,0,3,0,0",
    ] {
        assert!(
            CenterTwists::parse(PuzzleKind::Cube3, bad).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn only_the_3x3x3_is_tracked() {
    for kind in [PuzzleKind::Cube5, PuzzleKind::Cube7] {
        let start = CenterTwists::untwisted(kind).unwrap();
        assert!(matches!(
            start.track(&seq("R")),
            Err(TwistError::Unsupported(_))
        ));
    }
}

#[test]
fn the_convention_puts_the_top_of_u_to_the_back_and_of_d_to_the_front() {
    // x turns the cube like R: U goes to B and D to F, the back goes down and the
    // front goes up. The top of U (back) ends pointing down on B, the top of D
    // (front) ends pointing up on F.
    let x = center_rotations()
        .into_iter()
        .find(|r| r.word == "x")
        .expect("x is one of the 24 rotations");
    assert_eq!(x.dest[Face::U.index()], Face::B.index());
    assert_eq!(x.dest[Face::D.index()], Face::F.index());
    assert_eq!(x.add[Face::U.index()], 2);
    assert_eq!(x.add[Face::D.index()], 0);
}
