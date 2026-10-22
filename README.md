# cubacle

A Rubik's cube solver and generator in Rust, built as a backend for cube explorers. The name is a play on *cubicle*: a small room for working on cubes.

Today the 2×2×2 is solved optimally and the 3×3×3 is solved with an optimized implementation of the two-phase algorithm (derived from [min2phase](https://github.com/cs0x7f/min2phase) by Chen Shuang). The `cubacle::puzzle` module describes cubes of every size from 2×2×2 to 7×7×7 behind one `Solver` trait; engines for the other sizes are being added. [ARCHITECTURE.md](ARCHITECTURE.md) explains how the crate is laid out and which module may depend on which.

Licensed under AGPL-3.0-or-later; see [`NOTICE.md`](NOTICE.md) for the MIT-licensed origin of the 3×3×3 core.

A solver accepts a cube as typed facelets and returns a typed move sequence. The crate also generates random states and scrambles, parses scrambles and applies them to a cube.

## Command line

```
cubacle solve A4ib8aejxj4Y                       # Orbit64 token
cubacle solve FUFUUFFFF UUUURRURU RRRFFRRFR BDBBDDBBB DLDDLLDDD LBLBBLLLL
cubacle solve GWGWWGGGG WWWWRRWRW RRRGGRRGR BYBBYYBBB YOYYOOYYY OBOBBOOOO
cubacle solve --scramble "R U R' U'" --max-moves 20 --budget-ms 500 --verbose
cubacle convert --scheme jp WWWW RRRR GGGG BBBB OOOO YYYY
cubacle random | cubacle solve -
cubacle scramble -n 25
cubacle facelets "R U R' U'"
```

A scramble is standard move notation with whitespace between the moves (`R U R' U'`, `F2`, `B'`). The aliases `R1` (`R`), `R3` (`R'`), and `R2'` or `R'2` (`R2`) are accepted too, as in the WCA export form; unseparated runs such as `RUR'` and a lone `-` are rejected.

A cube may be written as an [Orbit64](https://github.com/wstein/flix-orbit64) state token, as `URFDLB` facelets, or as sticker colours `W R G Y O B`. Facelets and colours may be compact or spaced, in any grouping, and need no quoting. Colours are read from the fixed centres of an odd cube, so western and Japanese cubes need no flag. A cube without centres (the 2×2×2) uses `--scheme`:

| Scheme | U R F D L B | Opposite pairs |
| --- | --- | --- |
| `boy` (default) | W R G Y O B | white–yellow, green–blue, red–orange |
| `jp` | W R G B O Y | white–blue, green–yellow, red–orange |

Any six letters in `URFDLB` order also work, e.g. `--scheme WRGYOB`.

`solve` handles the 2×2×2 and the 3×3×3; larger sizes have no engine yet. It prints the solution, and for the 3×3×3 checks by replay that it solves the cube. It exits with a non-zero status when the state is impossible, the limit is too tight or the time budget runs out. Orbit64 tokens decode for the 2×2×2 and 3×3×3.

The 2×2×2 solver (`PocketSolver`) keeps one corner fixed and searches an exact distance table of all 3,674,160 states, so its solutions are optimal in the half-turn metric (U, R and F turns by 90° or 180°). The table is built on first use in well under a second in a release build. A cube that is solved in any orientation needs no moves. A solve with a ready table takes microseconds and always meets `--budget-ms`; the first solve in a process includes the one-off table build, which a budget cannot interrupt and which counts against it. `Solver::prepare()` does the build ahead of time.

## Usage

```rust
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions, Solver};

let scramble = "R U R' U'".parse().expect("valid scramble");
let cube = Facelets::solved(PuzzleKind::Cube3).apply(&scramble).expect("fits a 3x3");

let solver = Min2PhaseSolver::shared();
let result = solver.solve(&cube, &SolveOptions::default().with_max_moves(20));
let solution = result.solution().expect("solvable");

assert!(cube.apply(solution).expect("fits").is_solved());
```

Failures are values, not strings: `result.outcome` is `Err(Failure::InvalidInput)` for an impossible cube, `Err(Failure::BudgetExhausted)` when the time budget ran out and `Err(Failure::Exhausted)` when no solution exists within the move limit; `result.message` carries a sentence for people.

### Facelet order

`Facelets::parse` reads, and `Facelets` prints, stickers in this order:

```text
U1U2...U9R1R2...R9F1...F9D1...D9L1...L9B1...B9
```

The solved 3×3×3 is:

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

A limit of 20 or 21 moves is recommended for the 3×3×3 (`SolveOptions::with_max_moves`; the default is 21).

## API overview

- `puzzle`: `PuzzleKind`, `Facelets` (with `apply(&MoveSequence)`), `Move` and `MoveSequence` (parse with `str::parse`), the `Solver` trait, `SolveOptions`, `SolveResult` and `Failure`.
- `notation`: `parse_cube` (Orbit64 tokens, facelets and colours), `ColourScheme`, the `orbit64` codec.
- `generate`: `random_state(kind)` (a uniform 2×2×2 or 3×3×3 state) and `scramble(kind, length)`.
- `engines::three::Min2PhaseSolver`: the 3×3×3 `Solver`, plus
  - `solve_pattern(&from, &to, &options)`: moves that transform one cube state into another.
  - `solve_batch(&cubes, &options)`: independent cubes solved concurrently, results in input order.
  - `solve_parallel`, `solve_parallel_with_minimum_time` and `solve_pattern_parallel`: experimental six-orientation single-cube searches.
  - `solve_with_minimum_time(&cube, &options, minimum)`: keep improving the solution until the minimum search time, within the hard maximum set by `options.time_budget`.
- `engines::two::PocketSolver`: the optimal 2×2×2 `Solver`.

A `Min2PhaseSolver` owns its move and pruning tables, built on first use or by `prepare()`. Clones share them: create one solver at startup and pass clones to whoever needs one. `Min2PhaseSolver::new()` starts a table set of its own, and `Min2PhaseSolver::shared()` returns a process-wide default for code with nothing to inject.

A time budget that expires after a solution was found still returns that solution; only a deadline reached with no solution is `Failure::BudgetExhausted`.

### Goal-state solving

Use `solve_pattern` when the desired result is an arbitrary valid cube state rather than the solved cube:

```rust
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, PuzzleKind, SolveOptions};

let solved = Facelets::solved(PuzzleKind::Cube3);
let from = solved.apply(&"R U R' U'".parse().unwrap()).unwrap();
let to = solved.apply(&"F2 D L2".parse().unwrap()).unwrap();

let options = SolveOptions::default().with_max_moves(20);
let result = Min2PhaseSolver::shared().solve_pattern(&from, &to, &options);

assert_eq!(from.apply(result.solution().unwrap()).unwrap(), to);
```

### Batch throughput

`Min2PhaseSolver::solve_batch` reuses a process-wide worker pool, avoiding per-solve thread creation. Set `CUBACLE_THREADS` to a positive worker count before its first use to override the default of `available_parallelism()`. The pool has no shutdown operation and lives for the process lifetime. It is optimized for many independent cubes, not lower latency for a single cube.

```sh
CUBACLE_THREADS=4 cargo run --release --example bulk_compare -- \
  examples/scrambles.csv examples/scramble-report.csv 4 - 100 100
```

The `solve_parallel*` methods split one solve across the six URF orientations, complete every orientation search, and select the first valid result in fixed URF order. The orientation searches share a best-length bound, so which valid solution comes back can vary between runs; it is always a solution within the move limit. They use the same pool, so set `CUBACLE_THREADS` before their first use. Wall-clock budgets still have the same timing sensitivity as the serial budget APIs. This is an experimental latency mode: it can perform more total IDA* work than serial search. Validate it on the target machine before adopting it.

## Comparing a scramble batch

`examples/bulk_compare.rs` compares the solutions in a scramble CSV with solutions generated by this crate at a 19-move limit and a 100 ms minimum and maximum search time per cube. After finding an initial solution, it continues searching for a shorter one until the minimum is reached, and never exceeds the hard maximum. The input CSV is not part of the repository (`examples/*.csv` is ignored by git); supply your own with the header `competitionId,evntId,roundTypeId,group,isExtra,number,scramble,moves,solution`.

```sh
RUSTFLAGS='-C target-cpu=native' cargo run --release --example bulk_compare -- \
  examples/scrambles.csv examples/scramble-report-100ms.csv
```

The command uses one worker per available logical CPU by default and shows progress, elapsed time, and an ETA. Its live progress line also reports completed classifications, timeouts, accumulated phase-two probes, and average solve time. To choose a worker count, row limit, minimum search time, and maximum search time in milliseconds, pass optional third through sixth arguments:

```sh
cargo run --release --example bulk_compare -- \
  examples/scrambles.csv examples/scramble-report-200-500ms.csv 12 100 200 500
```

The command prints a summary and writes every row to the report CSV. The summary reports actual worker count, wall-clock batch duration, and the sum of per-job solve times. Each row includes the classification, expected and actual length, phase-two probes, elapsed microseconds, and `timed_out` status. A timed-out row has no solution before the hard limit; a row with a solution is returned after the minimum search time or the hard maximum, whichever comes first. Reference moves using `1` and `3` suffixes are normalized to the solver's bare-turn and apostrophe notation before comparison.

The summary distinguishes an exact move sequence, any same-length solution (including exact sequences), shorter solutions, longer solutions, errors, and timeouts. A timeout is `Failure::BudgetExhausted`: no solution was found before the hard limit. It is distinct from ordinary input errors.

To retry only `longer` rows from an existing report, use `-` for no row limit and pass `longer` after the minimum and maximum search times:

```sh
RUSTFLAGS='-C target-cpu=native' cargo run --release --example bulk_compare -- \
  examples/scramble-report-100ms.csv examples/scramble-report-longer-200ms.csv \
  12 - 200 500 longer
```

Unbounded serial solves are deterministic for a fixed cube, move limit, solver version, and build; parallel solves are not guaranteed to be. Time-bounded solves are intentionally not deterministic: CPU scheduling and the point at which the search observes the time limit can affect the best result found.

`target-cpu=native` enables CPU features available on the host, including Apple M2 systems. Do not use that flag for binaries intended to run on other CPU models.

## Benchmarking

The validating benchmark warms the solver tables, solves a fixed corpus repeatedly, and checks every returned sequence by applying it to the cube. It reports average, median, and p95 solver time and phase-two probes per second. The argument is the number of passes over the corpus:

```sh
RUSTFLAGS='-C target-cpu=native' cargo run --release --example benchmark -- 100
```

To benchmark the experimental single-cube parallel path with the same fixed corpus, add `--parallel`; compare its p50 and p95 against a serial run on the same warm machine:

```sh
RUSTFLAGS='-C target-cpu=native' cargo run --release --example benchmark -- 200 --parallel
```

For a parallel throughput comparison, run the same bulk sample with one worker and then with your desired worker count. The report and summary make probe totals and result classifications directly comparable.

## Development

Run the project quality checks with:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
```

`tests/golden_test.rs` replays `tests/fixtures/golden.txt`, a record of the exact solutions and probe counts of both engines for 554 fixed cubes; a change that alters search behaviour fails it. `tests/architecture_test.rs` enforces the module dependency rules.

## HTTP server

`crates/server` holds `cubacle-server`, a stateless REST backend (solve, convert, scramble, random) for the 2x2x2 and 3x3x3. See [crates/server/README.md](crates/server/README.md).
