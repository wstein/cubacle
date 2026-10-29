use super::geometry::{axis, position, Rotation};
use super::*;
use crate::puzzle::MoveSequence;

fn print() -> Print {
    let stickers = (0..8)
        .flat_map(|body| {
            (0..3).map(move |a| {
                let p = position(body);
                let mut n = [0; 3];
                n[a] = p[a];
                Sticker {
                    id: body * 3 + a as u8,
                    cubie: body,
                    p,
                    n,
                    up: if a == 1 { [0, 0, 1] } else { [0, 1, 0] },
                    image: Some(format!("{body}:{a}")),
                }
            })
        })
        .collect();
    Print::new(stickers).unwrap()
}

#[test]
fn proper_rotations_and_all_body_pose_indices_are_bijective() {
    let p = print();
    let all = Rotation::all();
    assert_eq!(all.len(), 24);
    for body in 0..8 {
        let mut seen = [false; 24];
        for r in &all {
            let q = r.apply(position(body));
            let pose = 3 * geometry::position_rank(q).unwrap() + r.twist_digit();
            assert!(!seen[pose as usize]);
            seen[pose as usize] = true;
            let mut partial = [None; 8];
            partial[body as usize] = Some(pose);
            let state = p.complete(partial).unwrap();
            assert_eq!(
                p.state_from_stickers(p.stickers(&state).unwrap().to_vec())
                    .unwrap(),
                state
            );
        }
        assert!(seen.into_iter().all(|v| v));
    }
}

#[test]
fn every_htm_move_preserves_rigid_identity_and_has_the_expected_order() {
    let p = print();
    let home = p.reference();
    assert_eq!(home.poses(), &[0, 3, 6, 9, 12, 15, 18, 21]);
    for mv in moves() {
        let moved = p.apply(&home, mv).unwrap();
        assert_ne!(moved, home);
        let stickers = p.stickers(&moved).unwrap();
        for (before, after) in p.printing().iter().zip(&stickers) {
            assert_eq!(
                (before.id, before.cubie, &before.image),
                (after.id, after.cubie, &after.image)
            );
            assert!(axis(after.up).is_some());
            assert_eq!(geometry::dot(after.n, after.up), 0);
        }
        let inverse = MoveSequence::new(vec![mv]).inverse();
        assert_eq!(p.apply_moves(&moved, &inverse).unwrap(), home);
        let count = if mv.turn == Turn::Half { 2 } else { 4 };
        assert_eq!(
            p.apply_moves(&home, &MoveSequence::new(vec![mv; count]))
                .unwrap(),
            home
        );
    }
    assert!(p
        .apply(&home, Move::new(Face::R, 2, false, Turn::Clockwise))
        .is_err());
    assert!(p
        .apply(&home, Move::new(Face::R, 1, true, Turn::Clockwise))
        .is_err());
}

#[test]
fn exact_frames_and_chained_transitions_keep_all_artwork_orientations() {
    let p = print();
    let home = p.reference();
    let rotate: MoveSequence = "U D'".parse().unwrap();
    let turned = p.apply_moves(&home, &rotate).unwrap();
    assert_ne!(turned, home);
    assert_ne!(p.stickers(&turned).unwrap(), p.stickers(&home).unwrap());
    let ab: MoveSequence = "R U2 F' D B2".parse().unwrap();
    let bc: MoveSequence = "L' F U B D2".parse().unwrap();
    let b = p.apply_moves(&home, &ab).unwrap();
    let c = p.apply_moves(&b, &bc).unwrap();
    let back = p
        .apply_moves(&p.apply_moves(&c, &bc.inverse()).unwrap(), &ab.inverse())
        .unwrap();
    assert_eq!(back, home);
    assert_eq!(p.stickers(&back).unwrap(), *p.printing());
}

#[test]
fn invalid_prints_fail_before_a_state_is_created() {
    let p = print();
    for change in 0..6 {
        let mut bad = p.printing().to_vec();
        match change {
            0 => {
                bad.pop();
            }
            1 => bad[1].id = bad[0].id,
            2 => bad[0].p = [0, -1, -1],
            3 => bad[0].up = bad[0].n,
            4 => bad[0].cubie = 1,
            _ => bad[0].n = bad[1].n,
        }
        assert!(Print::new(bad).is_err(), "case {change}");
    }
}

#[test]
fn unreachable_reflected_or_individually_changed_stickers_are_rejected() {
    let p = print();
    let mut poses = *p.reference().poses();
    poses[0] += 1;
    assert!(p.state_from_poses(poses).is_err());
    poses[0] = poses[1];
    assert!(p.state_from_poses(poses).is_err());
    for change in 0..4 {
        let mut bad = p.printing().to_vec();
        match change {
            0 => bad[0].up = [0, 0, 1],
            1 => bad[0].cubie = 1,
            2 => bad[0].image = None,
            _ => {
                // Reflection exchanges two outward axes, while leaving the third fixed.
                for s in &mut bad[..3] {
                    s.n.swap(0, 1);
                    s.up.swap(0, 1);
                }
            }
        }
        assert!(p.state_from_stickers(bad).is_err(), "case {change}");
    }
}

#[test]
fn dense_projection_keys_round_trip_and_complete_the_eighth_pose() {
    let p = print();
    for (bodies, count) in [
        (vec![], 1),
        (vec![1, 5, 7], 9072),
        (vec![1, 3, 5, 7], 136080),
    ] {
        let projection = Projection::new(bodies).unwrap();
        assert_eq!(projection.keys(), count);
        for key in 0..count {
            let poses = projection.decode(&p, key).unwrap();
            assert_eq!(projection.encode_poses(&p, &poses).unwrap(), key);
        }
        assert!(projection.decode(&p, count).is_err());
    }
    for k in [7, 8] {
        let projection = Projection::new((0..k).collect()).unwrap();
        assert_eq!(projection.keys(), 88179840);
        for key in [0, 1, 100, 114083, 19999999, 88179839] {
            let poses = projection.decode(&p, key).unwrap();
            assert_eq!(projection.encode_poses(&p, &poses).unwrap(), key);
        }
    }
    assert!(Projection::new(vec![0, 0]).is_err());
    assert!(Projection::new(vec![8]).is_err());
}

#[test]
fn each_random_transition_has_one_eighth_corner_completion() {
    let p = print();
    let projection = Projection::new((0..8).collect()).unwrap();
    let mut state = p.reference();
    let mut random = 42u64;
    for _ in 0..1000 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        state = p.apply(&state, moves()[(random % 18) as usize]).unwrap();
        let key = projection.encode(&state);
        assert_eq!(projection.decode(&p, key).unwrap(), state.poses());
        assert_eq!(
            p.state_from_stickers(p.stickers(&state).unwrap().to_vec())
                .unwrap(),
            state
        );
    }
}

#[test]
fn real_blank_print_matches_the_pinned_counterexample_pose_encoding() {
    let data: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/fixed_frame/blank.print.json"
    ))
    .unwrap();
    let vector = |v: &serde_json::Value| std::array::from_fn(|i| v[i].as_i64().unwrap() as i8);
    let stickers = data
        .as_array()
        .unwrap()
        .iter()
        .map(|s| Sticker {
            id: s["id"].as_u64().unwrap() as u8,
            cubie: s["cubie"].as_u64().unwrap() as u8,
            p: vector(&s["p"]),
            n: vector(&s["n"]),
            up: vector(&s["up"]),
            image: s["image"].as_str().map(str::to_owned),
        })
        .collect();
    let p = Print::new(stickers).unwrap();
    let state = p
        .apply_moves(&p.reference(), &"U D2 B D'".parse().unwrap())
        .unwrap();
    assert_eq!(state.poses(), &[5, 19, 7, 12, 2, 17, 23, 11]);
    assert_eq!(
        Projection::new(vec![1, 3, 6, 7]).unwrap().encode(&state),
        114083
    );
}

#[test]
fn right_clockwise_turn_has_the_specified_signed_world_direction() {
    let p = print();
    let moved = p.apply(&p.reference(), "R".parse().unwrap()).unwrap();
    let stickers = p.stickers(&moved).unwrap();
    let s = &stickers[21];
    assert_eq!(s.p, [1, 1, -1]);
    assert_eq!(s.n, [1, 0, 0]);
    assert_eq!(s.up, [0, 0, -1]);
}

#[test]
fn eighth_pose_decoding_is_independent_of_reference_body_order() {
    let mut stickers = print().printing().to_vec();
    for s in &mut stickers {
        s.cubie = 7 - s.cubie;
    }
    let p = Print::new(stickers).unwrap();
    assert_eq!(p.reference().poses(), &[21, 18, 15, 12, 9, 6, 3, 0]);
    let projection = Projection::new((0..8).collect()).unwrap();
    let mut state = p.reference();
    for i in 0..100 {
        state = p.apply(&state, moves()[i % 18]).unwrap();
        assert_eq!(
            projection.decode(&p, projection.encode(&state)).unwrap(),
            state.poses()
        );
    }
}
