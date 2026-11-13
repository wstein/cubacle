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

/// The middle slice `slice` of the axis `axis` is the whole-cube rotation
/// `axis` followed by the two outer turns `outer` (R' L for 2R): checked on
/// the stickers, apart from the tracker.
fn assert_slice_is_rotation_and_outer_turns(slice: &str, axis: &str, outer: &str) {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let by_slice = solved.apply(&seq(slice)).unwrap();
    let mut moves = rotation_moves(axis).unwrap().moves().to_vec();
    moves.extend(seq(outer).moves().iter().copied());
    let by_parts = solved.apply(&MoveSequence::new(moves)).unwrap();
    assert_eq!(by_slice, by_parts, "{slice} is {axis} {outer}");
}

/// After the slice and the frame rotation every center is on its own face.
fn assert_centers_home_after(slice: &str, frame: &str) {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let end = solved
        .apply(&seq(slice))
        .unwrap()
        .apply(&rotation_moves(frame).unwrap())
        .unwrap();
    for face in Face::ALL {
        assert_eq!(
            end.stickers()[face.index() * 9 + 4],
            face,
            "{slice} then {frame}"
        );
    }
}

#[test]
fn a_middle_slice_turns_only_the_two_outer_centers_of_its_axis() {
    // 2R is x R' L, 2U is y U' D, 2F is z F' B: after the rotation back the
    // outer center of the turn direction gains 3 quarters and the opposite one 1.
    let cases = [
        ("2R", "x", "R' L", [0, 3, 0, 0, 1, 0]),
        ("2U", "y", "U' D", [3, 0, 0, 1, 0, 0]),
        ("2F", "z", "F' B", [0, 0, 3, 0, 0, 1]),
    ];
    for (slice, axis, outer, added) in cases {
        assert_slice_is_rotation_and_outer_turns(slice, axis, outer);
        for start in [[0u8; 6], [1, 2, 3, 0, 1, 2]] {
            let tracked = twists(start).track(&seq(slice)).unwrap();
            let want: [u8; 6] = std::array::from_fn(|g| (start[g] + added[g]) & 3);
            assert_eq!(array(&tracked.twists), want, "{slice} from {start:?}");
            assert!(!tracked.frame.is_empty(), "{slice}");
            assert_centers_home_after(slice, &tracked.frame);
        }
    }
}
