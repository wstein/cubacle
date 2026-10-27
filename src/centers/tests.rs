//! Unit tests that also run engines; the architecture scan skips the inline module.

#[cfg(test)]
mod cases {
    use std::sync::Arc;

    use crate::centers::*;
    use crate::engines::four::FourSolver;
    use crate::engines::three::Min2PhaseSolver;
    use crate::puzzle::{reaches, relative_by};

    fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
        Facelets::solved(kind)
            .apply(&moves.parse().unwrap())
            .unwrap()
    }

    fn four() -> FourSolver {
        FourSolver::new(Arc::new(Min2PhaseSolver::shared()))
    }

    #[test]
    fn solved_cubes_and_outer_turns_are_uniform() {
        for kind in PuzzleKind::ALL {
            assert!(uniform(&Facelets::solved(kind)));
        }
        assert!(uniform(&cube(PuzzleKind::Cube5, "R U2 F' D L2 B")));
    }

    #[test]
    fn an_inner_slice_makes_centers_mixed() {
        for kind in [PuzzleKind::Cube4, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            assert!(!uniform(&cube(kind, "2R")), "{kind:?}");
        }
    }

    #[test]
    fn uniform_targets_and_equal_states_need_no_plan() {
        let engine = four();
        let start = cube(PuzzleKind::Cube4, "R U");
        let options = SolveOptions::default();
        let outer = cube(PuzzleKind::Cube4, "R U F2");
        assert!(plan(&engine, &start, &outer, &options, None)
            .unwrap()
            .is_none());
        let mixed = cube(PuzzleKind::Cube4, "R U Rw");
        assert!(plan(&engine, &mixed, &mixed, &options, None)
            .unwrap()
            .is_none());
    }

    #[test]
    fn a_known_way_bounds_the_fix_up() {
        let engine = four();
        let start = Facelets::solved(PuzzleKind::Cube4);
        // a long target the search will not undo in fewer moves than it took
        let moves: MoveSequence = "2R 2U' 2F 2L2 2D 2B' 2R' 2U 2F2 2L".parse().unwrap();
        let target = start.apply(&moves).unwrap();
        let plan = plan(
            &engine,
            &start,
            &target,
            &SolveOptions::default(),
            Some(&moves),
        )
        .unwrap()
        .expect("mixed");
        assert!(plan.tail().len() <= moves.len());
        // a wrong hint changes nothing but the bound
        let other: MoveSequence = "R U".parse().unwrap();
        let plan = crate::centers::plan(
            &engine,
            &start,
            &target,
            &SolveOptions::default(),
            Some(&other),
        )
        .unwrap()
        .expect("mixed");
        assert!(!plan.tail().is_empty());
    }

    #[test]
    fn either_pairing_of_equal_pieces_reaches_the_goal() {
        let engine = four();
        let options = SolveOptions::default();
        for (start, target) in [
            ("", "2R 2U' 2F 2L2 2D 2B' 2R' 2U"),
            ("R U F2 D' L B", "Rw 2U Fw' 2L"),
        ] {
            let (start, target) = (
                cube(PuzzleKind::Cube4, start),
                cube(PuzzleKind::Cube4, target),
            );
            let plan = plan(&engine, &start, &target, &options, None)
                .unwrap()
                .expect("mixed");
            assert_ne!(plan.method(), Method::Direct);
            for reversed in [false, true] {
                let relative = relative_by(&start, plan.goal(), reversed).unwrap();
                let result = plan.complete(engine.solve(&relative, &options));
                let route = result.solution().expect("solved");
                assert!(reaches(&start, route, &target), "reversed {reversed}");
                assert_eq!(result.stages.last().map(|s| s.name), Some(STAGE_NAME));
                assert_eq!(result.stages.last().map(|s| s.end), Some(route.len()));
            }
        }
    }

    #[test]
    fn the_limit_covers_the_fix_up_and_the_budget_the_planning() {
        let engine = four();
        let start = cube(PuzzleKind::Cube4, "R U");
        let target = cube(PuzzleKind::Cube4, "2R 2U' 2F 2L2 2D 2B' 2R' 2U");
        let options = SolveOptions::default();
        let plan = plan(&engine, &start, &target, &options, None)
            .unwrap()
            .expect("mixed");
        let tail = plan.tail().len();
        let limited = plan.limit(&options.with_max_moves(30)).unwrap();
        assert_eq!(usize::from(limited.max_moves.unwrap()), 30 - tail);
        let refused = plan.limit(&options.with_max_moves(tail as u8)).unwrap_err();
        assert_eq!(refused.outcome, Err(Failure::Exhausted));
        assert!(refused.message.unwrap().contains("center fix-up"));
        let spent = options.with_time_budget(Duration::ZERO);
        assert_eq!(
            plan.limit(&spent).unwrap_err().outcome,
            Err(Failure::BudgetExhausted)
        );
    }

    #[test]
    fn a_target_no_cube_can_be_is_impossible() {
        // swap a corner-of-centers sticker with an edge-of-centers sticker of a
        // 5x5: the center orbits then hold the wrong colors
        let mut stickers = cube(PuzzleKind::Cube5, "R U").stickers().to_vec();
        let orbits = center_orbits(PuzzleKind::Cube5);
        let (x, t) = (orbits[0][0], orbits[1][8]);
        assert_ne!(stickers[x], stickers[t]);
        stickers.swap(x, t);
        let target = Facelets::from_stickers(stickers).unwrap();
        let start = cube(PuzzleKind::Cube5, "R");
        let engine = crate::engines::five::FiveSolver::new(Arc::new(Min2PhaseSolver::shared()));
        let error = plan(&engine, &start, &target, &SolveOptions::default(), None).unwrap_err();
        assert!(matches!(error, PlanError::Impossible(_)), "{error:?}");
    }

    #[test]
    fn a_long_piece_of_the_fix_up_is_cut_into_equal_parts() {
        let pieces = [
            ("a".to_string(), 7),
            ("b".to_string(), 45),
            ("c".to_string(), 0),
        ];
        let stages = tail_stages(&pieces);
        let want = [("a", 7), ("b (1/3)", 22), ("b (2/3)", 37), ("b (3/3)", 52)];
        assert_eq!(stages, want);
        assert!(stages.windows(2).all(|w| w[1].1 - w[0].1 <= PHASE_MOVES));
    }

    #[test]
    fn the_undoing_of_an_engine_prefix_keeps_its_stage_borders_reversed() {
        let stage = |name, end| SolveStage { name, end };
        let stages = [stage("one", 10), stage("two", 10), stage("three", 25)];
        let pieces = undo_pieces(&stages, 18);
        let want = [
            ("undo phase 3: three".to_string(), 8),
            ("undo phase 1: one".to_string(), 10),
        ];
        assert_eq!(pieces, want);
        // no stages: one piece
        assert_eq!(undo_pieces(&[], 5), [(STAGE_NAME.to_string(), 5)]);
    }

    #[test]
    fn close_states_get_a_direct_route_of_the_moves_between_them() {
        let engine = four();
        for (kind, moves) in [
            (PuzzleKind::Cube5, "2R 3U' 2F"),
            (PuzzleKind::Cube6, "2R 3U' 2F"),
            (PuzzleKind::Cube7, "2R 3U' 4F"),
            (PuzzleKind::Cube5, "2R2"),
        ] {
            let start = cube(kind, "R U F2 D' L B");
            let target = start.apply(&moves.parse().unwrap()).unwrap();
            let plan = plan(&engine, &start, &target, &SolveOptions::default(), None)
                .unwrap()
                .expect("mixed");
            assert_eq!(plan.method(), Method::Direct, "{kind:?}");
            assert!(plan.tail().len() <= 3, "{kind:?}: {}", plan.tail());
            assert_eq!(plan.goal(), &start);
            assert_eq!(start.apply(plan.tail()).unwrap(), target);
            assert_eq!(plan.tail_stages(), [(DIRECT_STAGE_NAME, plan.tail().len())]);
            // nothing to solve first: the stages are the route alone
            let result = plan.complete(SolveResult {
                outcome: Ok(MoveSequence::default()),
                message: None,
                metrics: SolveMetrics::default(),
                stages: Vec::new(),
            });
            assert_eq!(result.stages.len(), 1);
            assert_eq!(result.stages[0].name, DIRECT_STAGE_NAME);
        }
    }

    #[test]
    fn far_states_keep_the_fix_up_route() {
        let engine = four();
        let start = cube(PuzzleKind::Cube4, "R U F2 D' L B");
        let target = start
            .apply(&"2R 2U' 2F 2L2 2D 2B' 2R' 2U Rw".parse().unwrap())
            .unwrap();
        let plan = plan(&engine, &start, &target, &SolveOptions::default(), None)
            .unwrap()
            .expect("mixed");
        assert_ne!(plan.method(), Method::Direct);
        assert!(uniform(plan.goal()));
        assert_eq!(plan.tail_stages().last().map(|s| s.0), Some(STAGE_NAME));
    }

    #[test]
    fn the_engine_prefix_is_finished_by_a_shorter_beam_search() {
        let engine = crate::engines::five::FiveSolver::new(Arc::new(Min2PhaseSolver::shared()));
        let start = cube(
            PuzzleKind::Cube5,
            "R U F2 D' L B 2R 2U' 2F 2L2 2D' B2 2F 2U 2R' D2 F' 2B U2 2D",
        );
        let target = start
            .apply(&"2R 2U' 2F 2L2 2D 2B' 2R' 2U 2F 2R2 2D' 2L".parse().unwrap())
            .unwrap();
        let plan = plan(&engine, &start, &target, &SolveOptions::default(), None)
            .unwrap()
            .expect("mixed");
        assert_eq!(plan.method(), Method::Refined);
        // an engine prefix is about 50 moves
        assert!(plan.tail().len() < 45, "{}", plan.tail());
        assert!(uniform(plan.goal()));
        assert_eq!(plan.goal().apply(plan.tail()).unwrap(), target);
        assert_eq!(
            plan.tail_stages().last().map(|s| s.1),
            Some(plan.tail().len())
        );
    }
}
