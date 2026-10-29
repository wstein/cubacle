use std::time::Instant;

use crate::puzzle::frame::{Print, Projection, Sticker};

use super::coordinates::Coordinates;
use super::*;

fn print() -> Print {
    Print::new(
        (0..8u8)
            .flat_map(|body| {
                (0..3).map(move |axis| {
                    let p =
                        std::array::from_fn(|i| if body & (1 << (2 - i)) == 0 { -1 } else { 1 });
                    let mut n = [0; 3];
                    n[axis] = p[axis];
                    Sticker {
                        id: 3 * body + axis as u8,
                        cubie: body,
                        p,
                        n,
                        up: if axis == 1 { [0, 0, 1] } else { [0, 1, 0] },
                        image: Some(format!("{body}:{axis}")),
                    }
                })
            })
            .collect(),
    )
    .unwrap()
}

fn cell(ids: Vec<u8>) -> Cell {
    Cell {
        p: [1, 1, 1],
        n: [0, 0, 1],
        accept: vec![Rule {
            accept: Accept::Ids(ids),
            up: Some([0, 1, 0]),
        }],
    }
}

#[test]
fn invalid_display_inputs_fail_instead_of_becoming_unsatisfiable() {
    let p = print();
    for ids in [vec![], vec![24], vec![2, 2]] {
        assert!(matches!(
            FrameGoal::new(&p, vec![cell(ids)]),
            Err(FixedError::InvalidGoal(_))
        ));
    }
    let c = cell(vec![2]);
    assert!(FrameGoal::new(&p, vec![c.clone(), c.clone()]).is_err());
    let mut bad = c.clone();
    bad.p = [0, 1, 1];
    assert!(FrameGoal::new(&p, vec![bad]).is_err());
    let mut bad = c.clone();
    bad.n = [0, 0, -1];
    assert!(FrameGoal::new(&p, vec![bad]).is_err());
    let mut bad = c.clone();
    bad.accept[0].up = Some([1, 1, 0]);
    assert!(FrameGoal::new(&p, vec![bad]).is_err());
    let mut bad = c;
    bad.accept[0].accept = Accept::Image("missing".into());
    assert!(FrameGoal::new(&p, vec![bad]).is_err());
}

#[test]
fn impossible_upright_rules_are_reported_and_never_produce_a_table() {
    let p = print();
    let mut c = cell(vec![2]);
    c.accept[0].up = Some([0, 0, 1]);
    let goal = FrameGoal::new(&p, vec![c]).unwrap();
    assert_eq!(goal.dead_rules(), &[[0, 0]]);
    let table = GoalTable::build(goal, BuildLimits::default()).unwrap();
    assert_eq!(table.goal_keys(), 0);
    assert!(table.histogram().is_empty());
    assert!(
        table.distances().is_none() && table.sha256().is_none() && table.eccentricity().is_none()
    );
    assert_eq!(
        table.distance(&p.reference()),
        Err(FixedError::Unsatisfiable)
    );
    assert_eq!(
        table.shortest_path(&p.reference()),
        Err(FixedError::Unsatisfiable)
    );
}

#[test]
fn empty_goal_has_one_zero_key_and_no_move_is_needed() {
    let p = print();
    let goal = FrameGoal::new(&p, Vec::new()).unwrap();
    let table = GoalTable::build(goal, BuildLimits::default()).unwrap();
    assert!(table.goal().projection().bodies().is_empty());
    assert_eq!(table.goal_keys(), 1);
    assert_eq!(table.histogram(), &[1]);
    assert_eq!(table.distances(), Some(&[0][..]));
    assert_eq!(table.eccentricity(), Some(0));
    assert_eq!(
        table.sha256(),
        Some("6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d")
    );
    let source = p
        .apply_moves(&p.reference(), &"U D' R2".parse().unwrap())
        .unwrap();
    let answer = table.shortest_path(&source).unwrap();
    assert_eq!(answer.distance, 0);
    assert!(answer.path.is_empty());
    assert_eq!(answer.reached, source);
    assert!(table.distance_key(1).is_err());
}

#[test]
fn canonical_cells_rules_and_unicode_have_one_ascii_normal_form() {
    let mut stickers = print().printing().to_vec();
    stickers[2].image = Some("\n😀é/\"\\".into());
    let p = Print::new(stickers).unwrap();
    let mut a = cell(vec![12, 2]);
    let r = Rule {
        accept: Accept::Image("\n😀é/\"\\".into()),
        up: None,
    };
    a.accept.extend([r.clone(), r.clone()]);
    let mut b = cell(vec![0]);
    b.p = [-1, -1, 1];
    let left = FrameGoal::new(&p, vec![a.clone(), b.clone()]).unwrap();
    a.accept.reverse();
    let right = FrameGoal::new(&p, vec![b, a]).unwrap();
    assert_eq!(left.canonical_display(), right.canonical_display());
    assert_eq!(left.dead_rules(), right.dead_rules());
    assert!(left.canonical_display().is_ascii());
    assert!(left
        .canonical_display()
        .contains(r#"\u000a\ud83d\ude00\u00e9/\"\\"#));
    assert_eq!(left.cells()[1].accept.len(), 2);
    assert!(matches!(&left.cells()[1].accept[0].accept,Accept::Ids(ids) if ids==&[2,12]));
}

#[test]
fn allocation_free_coordinates_agree_with_the_public_contract() {
    let p = print();
    for bodies in [
        vec![],
        vec![1, 3, 6],
        vec![1, 3, 6, 7],
        (0..7).collect(),
        (0..8).collect(),
    ] {
        let projection = Projection::new(bodies).unwrap();
        let coordinates = Coordinates::new(&p, &projection);
        for key in [0, projection.keys() / 2, projection.keys() - 1] {
            let full = coordinates.decode(key);
            assert_eq!(coordinates.encode(&full), key);
            let expected = projection.decode(&p, key).unwrap();
            assert_eq!(
                projection
                    .bodies()
                    .iter()
                    .map(|b| full[*b as usize])
                    .collect::<Vec<_>>(),
                expected
            );
            let mut partial = [None; 8];
            for body in projection.bodies() {
                partial[*body as usize] = Some(full[*body as usize]);
            }
            let state = p.complete(partial).unwrap();
            for (index, mv) in crate::puzzle::frame::moves().into_iter().enumerate() {
                assert_eq!(
                    coordinates.successor(&full, index),
                    projection.encode(&p.apply(&state, mv).unwrap())
                );
            }
        }
    }
}

#[test]
fn memory_and_expired_deadline_fail_before_returning_partial_answers() {
    let p = print();
    let goal = FrameGoal::new(&p, vec![cell(vec![23])]).unwrap();
    assert!(matches!(
        GoalTable::build(
            goal.clone(),
            BuildLimits {
                max_memory_bytes: 0,
                deadline: None
            }
        ),
        Err(FixedError::OverLimit)
    ));
    assert!(matches!(
        GoalTable::build(
            goal.clone(),
            BuildLimits {
                max_memory_bytes: usize::MAX,
                deadline: Some(Instant::now())
            }
        ),
        Err(FixedError::Deadline)
    ));
    let table = GoalTable::build(goal.clone(), BuildLimits::default()).unwrap();
    assert!(matches!(
        GoalTable::build(
            goal.clone(),
            BuildLimits {
                max_memory_bytes: table.build_memory_bytes() - 1,
                deadline: None
            }
        ),
        Err(FixedError::OverLimit)
    ));
    // Measure and retry the same clone allocation shape: an original canonical String
    // can retain spare capacity that its clone does not, and the budget charges capacities.
    assert!(GoalTable::build(
        goal.clone(),
        BuildLimits {
            max_memory_bytes: table.build_memory_bytes(),
            deadline: None
        }
    )
    .is_ok());
}

#[test]
fn locally_live_rules_can_still_be_jointly_unsatisfiable() {
    let p = print();
    let mut a = cell(vec![0]);
    a.accept[0].up = None;
    let mut b = cell(vec![1]);
    b.accept[0].up = None;
    b.p = [-1, -1, 1];
    let goal = FrameGoal::new(&p, vec![a, b]).unwrap();
    assert!(goal.dead_rules().is_empty());
    let table = GoalTable::build(goal, BuildLimits::default()).unwrap();
    assert_eq!(table.goal_keys(), 0);
    assert!(table.distances().is_none());
}

#[test]
fn accepting_more_stickers_never_increases_an_exact_distance() {
    let p = print();
    let mut narrow_cell = cell(vec![0]);
    narrow_cell.accept[0].up = None;
    let mut wide_cell = narrow_cell.clone();
    wide_cell.accept[0].accept = Accept::Ids(vec![0, 1]);
    let narrow = GoalTable::build(
        FrameGoal::new(&p, vec![narrow_cell]).unwrap(),
        BuildLimits::default(),
    )
    .unwrap();
    let wide = GoalTable::build(
        FrameGoal::new(&p, vec![wide_cell]).unwrap(),
        BuildLimits::default(),
    )
    .unwrap();
    for key in 0..narrow.goal().projection().keys() {
        assert!(wide.distance_key(key).unwrap() <= narrow.distance_key(key).unwrap());
    }
}
