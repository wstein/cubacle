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
  engines/
    three/        the 3x3x3 two-phase engine (Min2PhaseSolver)
    two/          the optimal 2x2x2 engine (PocketSolver)
  pool.rs         worker pool, private to engines/three
  main.rs         the cubacle command line
crates/server/    the cubacle-server HTTP backend
```

## Dependency rules

```
puzzle  <-  notation  <-  engines  <-  apps (CLI, server)
   ^            ^
   +--- generate
```

- `puzzle` depends on nothing else in the crate.
- `notation` and `generate` depend only on `puzzle`.
- An engine depends on `puzzle` (and, for the 2x2x2, on the Orbit64 corner
  tables in `notation`). Engines never use each other's internals: they meet
  the world only through the `Solver` trait and `puzzle` types.
- Applications (the CLI and the server) sit on top and may use everything.
- `pool` is an implementation detail of `engines::three`.

`tests/architecture_test.rs` reads the sources and fails on a `crate::` path
that breaks these rules (unit tests are exempt, so an engine's tests may
cross-check against `generate` or `Facelets::apply`).

## Visibility

Items are private, or `pub(crate)` when a sibling module needs them. Only what
a future crate would export is `pub`: the `puzzle`, `notation` and `generate`
APIs, `Min2PhaseSolver` and `PocketSolver`. Inside `engines::three` the
sub-modules (`cubie`, `tables`, `coordinates`, `search`, `engine`) are private
and talk to each other with `pub(super)` items.

## Shared state

The 3x3x3 pruning tables belong to a `Min2PhaseSolver` instance, behind an
`Arc<OnceLock<..>>`. `Min2PhaseSolver::new()` starts a table set of its own,
clones share it, `prepare()` builds it ahead of the first solve, and
`Min2PhaseSolver::shared()` returns a process-wide default for callers with
nothing to inject. The server builds one solver at startup and clones it into
every solve. The 2x2x2 distance table is still a process-wide `OnceLock` in
`engines::two`.

## Future reduction engines (4x4x4 and up)

A reduction engine solves a big cube by reducing it to a 3x3x3 and handing
that to a 3x3x3 solver. It must receive that solver, never reach for one:

```rust
pub struct Reduction4 {
    three: Arc<dyn Solver + Send + Sync>, // or a generic S: Solver
}
```

It depends on the 3x3x3 engine only through the `Solver` trait and `puzzle`
types, and never through a global, so the engine can be tested with a stub
solver and moved to its own crate without a dependency on `engines::three`.
