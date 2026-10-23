# Architecture

cubacle is one crate organised by role. Every module below `src/` is written
so that it could become its own workspace crate by moving the directory and
adding a `Cargo.toml`; nothing needs to be untangled first.

```
src/
  lib.rs          module list and crate docs only
  puzzle/         PuzzleKind, Facelets/Face, Move/MoveSequence, the Solver trait,
                  SolveOptions/SolveResult/Failure, Facelets::apply
  notation/       Orbit64, colour schemes, input detection and parsing
  generate/       random states and scrambles
  phases/         the phases of a solution, from the engine's stages or the cube state
  ranges/         post-solve compression of axis runs into layer ranges (2-3r)
  render/         move icons (SVG, RGBA, Kitty graphics, text) from puzzle types
  recover/         where a cube went wrong in a printed solution, and the way back
                  (sticker permutations along the solution; the fresh solution is passed in)
  reduction/      crate-private parts shared by reduction engines (4x4x4,
                  5x5x5): combinatorics, 4-bit pruning tables with a parallel
                  builder and an on-disk cache, thread helpers, the ordered
                  parallel iterative-deepening search, the first-results
                  helper for deterministic parallel searches
  engines/
    three/        the 3x3x3 two-phase engine (Min2PhaseSolver), with its
                  private worker pool in pool.rs
    two/          the optimal 2x2x2 engine (PocketSolver)
    five/         the 5x5x5 reduction engine (FiveSolver), finishing with an
                  injected 3x3x3 Solver
    four/         the 4x4x4 reduction engine (FourSolver), finishing with an
                  injected 3x3x3 Solver
  main.rs         the cubacle command line
crates/server/    the cubacle-server HTTP backend
```

## Dependency rules

```
puzzle  <-  notation  <-  engines  <-  apps (CLI, server)
   ^            ^              ^
   +--- generate               |
   +--- phases                 |
   +--- render                 |
   +--- recover                 |
   +--- reduction  ------------+   (used by the reduction engines only)
```

- `puzzle` depends on nothing else in the crate.
- `ranges` depends on `puzzle` and `notation` (it writes its moves in a `Notation`).
- `notation`, `generate`, `phases`, `recover` and `render` depend only on `puzzle`. `render` is pure
  (no I/O, no global state) and never names `notation`: the CLI maps a colour
  scheme to a `render::Palette`. Terminal detection lives in the CLI.
- `reduction` depends only on `puzzle`, knows no cube, and is crate-private.
- The 4x4x4 and 5x5x5 engines depend on `puzzle` and `reduction`; the 3x3x3 solver they
  finish with is injected as an `Arc<dyn Solver + Send + Sync>`.
- An engine depends on `puzzle` (and, for the 2x2x2, on the Orbit64 corner
  tables in `notation`). Engines never use each other's internals: they meet
  the world only through the `Solver` trait and `puzzle` types.
- Applications (the CLI and the server) sit on top and may use everything.
- The worker pool lives in `engines/three/pool.rs` and is private to that engine.

`tests/architecture_test.rs` reads the sources and fails on a `crate::` path
that breaks these rules (unit tests are exempt, so an engine's tests may
cross-check against `generate` or `Facelets::apply`).

## Visibility

Items are private, or `pub(crate)` when a sibling module needs them. Only what
a future crate would export is `pub`: the `puzzle`, `notation` and `generate`
APIs, the `render` API (`icon`, `Action`, `Style`, `Icon`), `Min2PhaseSolver`, `PocketSolver` and `FiveSolver`. Inside `engines::three` the
sub-modules (`consts`, `coordinates`, `cubie`, `engine`, `pool`, `search`, `solver`, `stats`, `tables`) are private
and talk to each other with `pub(super)` items.

## Shared state

The 3x3x3 pruning tables belong to a `Min2PhaseSolver` instance, behind an
`Arc<OnceLock<..>>`. `Min2PhaseSolver::new()` starts a table set of its own,
clones share it, `prepare()` builds it ahead of the first solve, and
`Min2PhaseSolver::shared()` returns a process-wide default for callers with
nothing to inject. The server builds one solver at startup and clones it into
every solve. The 2x2x2 distance table is still a process-wide `OnceLock` in
`engines::two`. A `FiveSolver` keeps its five-phase pruning tables (237 MB)
behind an `Arc<OnceLock<..>>` in the same way, read from or written to the
cache directory (`CUBACLE_CACHE_DIR`, else `$XDG_CACHE_HOME/cubacle`, else
`~/.cache/cubacle`); the server builds one around a clone of
its 3x3x3 solver.

## Reduction engines

`engines::five` is the first, `engines::four` the second. A reduction engine solves a big cube by reducing it to a 3x3x3 and handing
that to a 3x3x3 solver. It must receive that solver, never reach for one:

```rust
pub struct FiveSolver {
    three: Arc<dyn Solver + Send + Sync>, // or a generic S: Solver
}
```

It depends on the 3x3x3 engine only through the `Solver` trait and `puzzle`
types, and never through a global, so the engine can be tested with a stub
solver and moved to its own crate without a dependency on `engines::three`.

`FourSolver` also takes the number of probes it asks the 3x3x3 solver for (`SolveOptions::min_probes`, 500 by
default, counted rather than timed so the answer stays reproducible). Its searches run on several threads and
depend on neither the thread count nor the schedule: `reduction::par::first_results` joins the results of
the tasks in the order a single thread would find them.
