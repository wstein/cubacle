//! A cube whose corners are the mirror image of a real cube's is impossible
//! (a corner permutation of a real cube is a rotation of the solved corners),
//! on every size, for the validity check and for `solve`.

use std::process::Command;

use cubacle::notation::check;
use cubacle::puzzle::{Facelets, PuzzleKind};

/// Facelets of an `n`-cube whose eight corners all show their two side
/// colors exchanged: every corner is the mirror image of a real corner, the
/// edges and centers are solved.
fn mirrored_corners(n: usize) -> String {
    let mut s: Vec<char> = "URFDLB"
        .chars()
        .flat_map(|c| std::iter::repeat_n(c, n * n))
        .collect();
    let (r, f, l, b) = (n * n, 2 * n * n, 4 * n * n, 5 * n * n);
    let (last, row) = (n * n - 1, n * (n - 1));
    for (x, y) in [
        (r, f + n - 1),
        (f, l + n - 1),
        (l, b + n - 1),
        (b, r + n - 1),
        (f + last, r + row),
        (l + last, f + row),
        (b + last, l + row),
        (r + last, b + row),
    ] {
        s.swap(x, y);
    }
    s.into_iter().collect()
}

#[test]
fn the_validity_check_rejects_mirrored_corners() {
    for kind in PuzzleKind::ALL {
        let text = mirrored_corners(kind.size());
        let cube = Facelets::parse(&text).unwrap();
        assert!(check(&cube).is_err(), "{kind:?}");
    }
}

#[test]
fn solve_rejects_mirrored_corners_on_every_size() {
    for n in 2..=7 {
        let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
            .args(["solve", "--size", &n.to_string(), &mirrored_corners(n)])
            .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
            .env(
                "CUBACLE_CACHE_DIR",
                concat!(env!("CARGO_TARGET_TMPDIR"), "/five-tables"),
            )
            .output()
            .expect("cubacle runs");
        let err = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{n}: {err}");
        assert!(
            err.starts_with("error: the cube state is impossible ("),
            "{n}: {err}"
        );
    }
}

/// Each engine, used directly (not through the CLI's check), refuses the
/// mirror-image corners as invalid input and never answers with moves.
#[test]
fn every_engine_refuses_mirrored_corners() {
    use std::sync::Arc;

    use cubacle::engines::dwalton::DwaltonSolver;
    use cubacle::engines::five::FiveSolver;
    use cubacle::engines::four::FourSolver;
    use cubacle::engines::three::Min2PhaseSolver;
    use cubacle::engines::two::PocketSolver;
    use cubacle::puzzle::{Failure, SolveOptions, Solver};

    let three = Arc::new(Min2PhaseSolver::shared());
    let dwalton = |kind| {
        DwaltonSolver::new(
            kind,
            three.clone(),
            Arc::new(FourSolver::new(three.clone())),
            Arc::new(FiveSolver::new(three.clone())),
        )
    };
    let solvers: Vec<(usize, Box<dyn Solver>)> = vec![
        (2, Box::new(PocketSolver::new())),
        (3, Box::new(Min2PhaseSolver::shared())),
        (4, Box::new(FourSolver::new(three.clone()))),
        (5, Box::new(FiveSolver::new(three.clone()))),
        (6, Box::new(dwalton(PuzzleKind::Cube6))),
        (7, Box::new(dwalton(PuzzleKind::Cube7))),
    ];
    let options = SolveOptions {
        time_budget: Some(std::time::Duration::from_secs(20)),
        ..SolveOptions::default()
    };
    for (n, solver) in solvers {
        let cube = Facelets::parse(&mirrored_corners(n)).unwrap();
        let result = solver.solve(&cube, &options);
        assert_eq!(result.outcome, Err(Failure::InvalidInput), "{n}");
        if n == 5 {
            // refused at once by the color reading, not after a reduction
            let message = result.message.expect("a message");
            assert!(message.starts_with("the corners"), "{message}");
        }
    }
}
