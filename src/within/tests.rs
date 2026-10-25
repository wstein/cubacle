use super::*;

fn scrambled(kind: PuzzleKind, text: &str) -> Facelets {
    Facelets::solved(kind)
        .apply(&text.parse().unwrap())
        .unwrap()
}

fn limits(k: usize) -> Limits {
    Limits {
        max_moves: k,
        ..Limits::default()
    }
}

/// A scramble of `len` random alphabet moves from a fixed seed.
fn random_scramble(kind: PuzzleKind, kit: Kit, seed: &mut u64, len: usize) -> Facelets {
    let al = Alphabet::new(kind, kit);
    let mut state = colours(&Facelets::solved(kind));
    let mut last = None;
    for _ in 0..len {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        let k = (*seed >> 11) as usize % al.len();
        if !al.may_follow(last, k) {
            continue;
        }
        last = Some(k as u8);
        state = meet::permuted(&state, &al.perms[k]);
    }
    Facelets::from_stickers(
        state
            .iter()
            .map(|&c| crate::puzzle::Face::ALL[c as usize])
            .collect(),
    )
    .unwrap()
}

#[test]
fn the_route_back_is_found_and_replays_on_every_size() {
    for (kind, text) in [
        (PuzzleKind::Cube2, "R U F'"),
        (PuzzleKind::Cube3, "R U R' U'"),
        (PuzzleKind::Cube4, "R Uw F'"),
        (PuzzleKind::Cube5, "Rw U 2R'"),
        (PuzzleKind::Cube6, "R U"),
        (PuzzleKind::Cube7, "R U"),
    ] {
        let cube = scrambled(kind, text);
        let slices = text.contains('2');
        let kit = if slices { Kit::Slices } else { Kit::Outer };
        let report = search(&cube, &Facelets::solved(kind), kit, &limits(6));
        let Outcome::Found(route) = report.outcome else {
            panic!("{kind:?}: {:?}", report.outcome);
        };
        assert!(route.len() <= text.split(' ').count(), "{kind:?} {route}");
        assert!(cube.apply(&route).unwrap().is_solved());
    }
}

#[test]
fn a_slice_needs_the_slice_alphabet() {
    // Each slice is two outer blocks (`Rw R'`), so three of them are out of
    // reach of three outer moves.
    let cube = scrambled(PuzzleKind::Cube4, "2R 2U 2F");
    let solved = Facelets::solved(PuzzleKind::Cube4);
    let plain = search(&cube, &solved, Kit::Outer, &limits(3));
    assert!(!matches!(plain.outcome, Outcome::Found(_)), "{plain:?}");
    let with = search(&cube, &solved, Kit::Slices, &limits(3));
    let Outcome::Found(route) = with.outcome else {
        panic!("{with:?}");
    };
    assert_eq!(route.len(), 3);
    let blocks = search(&cube, &solved, Kit::Blocks, &limits(3));
    assert!(matches!(blocks.outcome, Outcome::Found(_)), "{blocks:?}");
}

#[test]
fn a_target_with_mixed_centres_is_fine() {
    for kind in [PuzzleKind::Cube4, PuzzleKind::Cube7] {
        let target = scrambled(kind, "2R 2U'");
        let start = target.apply(&"F R".parse().unwrap()).unwrap();
        let report = search(&start, &target, Kit::Slices, &limits(4));
        let Outcome::Found(route) = report.outcome else {
            panic!("{kind:?}: {:?}", report.outcome);
        };
        assert!(route.len() <= 2, "{route}");
        assert_eq!(start.apply(&route).unwrap(), target);
    }
}

#[test]
fn a_rotated_solved_cube_is_not_solved() {
    // Rw Lw' is a whole-cube rotation: the stickers differ, so the route is
    // not empty even though the cube looks solved up to colours.
    let cube = scrambled(PuzzleKind::Cube4, "Rw Lw'");
    let report = search(
        &cube,
        &Facelets::solved(PuzzleKind::Cube4),
        Kit::Outer,
        &limits(4),
    );
    let Outcome::Found(route) = report.outcome else {
        panic!("{:?}", report.outcome);
    };
    assert_eq!(route.len(), 2);
}

#[test]
fn the_optimum_of_the_solved_cube_is_nothing() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let report = search(&solved, &solved, Kit::Outer, &limits(3));
    assert_eq!(report.outcome, Outcome::Found(MoveSequence::default()));
}

#[test]
fn a_known_distance_is_exact() {
    // The superflip-free case: two commuting half turns need two moves.
    let cube = scrambled(PuzzleKind::Cube3, "R2 L2");
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let report = search(&cube, &solved, Kit::Outer, &limits(5));
    match report.outcome {
        Outcome::Found(route) => assert_eq!(route.len(), 2),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        search(&cube, &solved, Kit::Outer, &limits(1)).outcome,
        Outcome::Excluded | Outcome::NotFound
    ));
}

/// The lower bound is never above the true distance, and a search that skips
/// the bound agrees with every exclusion.
#[test]
fn the_lower_bound_never_excludes_wrongly() {
    let mut seed = 0x1234_5678_9ABC_DEF1u64;
    for (kind, kit, max_len, max_k) in [
        (PuzzleKind::Cube2, Kit::Outer, 5, 5),
        (PuzzleKind::Cube3, Kit::Outer, 5, 5),
        (PuzzleKind::Cube3, Kit::Blocks, 4, 4),
        (PuzzleKind::Cube4, Kit::Outer, 5, 5),
        (PuzzleKind::Cube4, Kit::Blocks, 4, 4),
        (PuzzleKind::Cube5, Kit::Slices, 4, 4),
    ] {
        let solved = Facelets::solved(kind);
        for len in 1..=max_len {
            for _ in 0..6 {
                let cube = random_scramble(kind, kit, &mut seed, len);
                let bound = lower_bound(&cube, &solved, kit);
                let truth = (0..=max_k).find(|&k| {
                    matches!(
                        run(&cube, &solved, kit, &limits(k), false).outcome,
                        Outcome::Found(_)
                    )
                });
                if let (Some(bound), Some(truth)) = (bound, truth) {
                    assert!(bound <= truth, "{kind:?} {kit:?}: {bound} > {truth}");
                }
                if bound.is_none() {
                    assert_eq!(truth, None, "{kind:?} {kit:?} unreachable but found");
                }
                for k in 1..=max_k {
                    let excluded = bound.is_none_or(|b| b > k);
                    let found = matches!(
                        run(&cube, &solved, kit, &limits(k), false).outcome,
                        Outcome::Found(_)
                    );
                    assert!(!(excluded && found), "{kind:?} {kit:?} len {len} k {k}");
                }
            }
        }
    }
}

#[test]
fn a_tiny_memory_cap_is_a_memory_limit() {
    let cube = scrambled(PuzzleKind::Cube3, "R U");
    let report = search(
        &cube,
        &Facelets::solved(PuzzleKind::Cube3),
        Kit::Outer,
        &Limits {
            max_memory_mb: 0,
            ..limits(4)
        },
    );
    assert_eq!(report.outcome, Outcome::Limit(Limit::Memory));
}

#[test]
fn no_time_is_a_time_limit() {
    let cube = scrambled(PuzzleKind::Cube5, "R U F D L B");
    let report = search(
        &cube,
        &Facelets::solved(PuzzleKind::Cube5),
        Kit::Blocks,
        &Limits {
            max_time: Duration::ZERO,
            ..limits(7)
        },
    );
    assert!(
        matches!(
            report.outcome,
            Outcome::Limit(Limit::Time) | Outcome::Excluded
        ),
        "{report:?}"
    );
}
