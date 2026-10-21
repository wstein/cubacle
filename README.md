# cubacle

A Rubik's cube solver and generator in Rust, built as a backend for cube explorers. The name is a play on *cubicle*: a small room for working on cubes.

Today the 3×3×3 is solved with an optimized implementation of the two-phase algorithm (derived from [min2phase](https://github.com/cs0x7f/min2phase) by Chen Shuang). The `cubacle::api` module describes cubes of every size from 2×2×2 to 7×7×7 behind one `Solver` trait; engines for the other sizes are being added.

Licensed under AGPL-3.0-or-later; see [`NOTICE.md`](NOTICE.md) for the MIT-licensed origin of the 3×3×3 core.

The solver accepts a cube in facelet order and returns a move sequence. It also provides helpers for creating random cubes, parsing scrambles, and applying moves.

## Command line

```
cubacle solve A4ib8aejxj4Y                       # Orbit64 token
cubacle solve FUFUUFFFF UUUURRURU RRRFFRRFR BDBBDDBBB DLDDLLDDD LBLBBLLLL
cubacle solve GWGWWGGGG WWWWRRWRW RRRGGRRGR BYBBYYBBB YOYYOOYYY OBOBBOOOO
cubacle solve --scramble "R U R' U'" --max-moves 20 --budget-ms 500 --verbose
cubacle convert --scheme japanese-bl-wt WWWW RRRR GGGG BBBB OOOO YYYY
cubacle random | cubacle solve -
cubacle scramble -n 25
cubacle facelets "R U R' U'"
```

A cube may be written as an [Orbit64](https://github.com/wstein/flix-orbit64) state token, as `URFDLB` facelets, or as sticker colours `W R G Y O B`. Facelets and colours may be compact or spaced, in any grouping, and need no quoting. Colours are read from the fixed centres of an odd cube, so western and Japanese cubes need no flag. A cube without centres (the 2×2×2) uses `--scheme`:

| Scheme | U R F D L B | Opposite pairs |
| --- | --- | --- |
| `western-boy` (default) | W R G Y O B | white–yellow, green–blue, red–orange |
| `japanese-bl-wt` | W R G B O Y | white–blue, green–yellow, red–orange |

Any six letters in `URFDLB` order also work, e.g. `--scheme WRGYOB`.

`solve` prints the solution and checks by replay that it solves the cube. It exits with a non-zero status when the state is impossible, the limit is too tight or the time budget runs out. Orbit64 tokens decode for the 2×2×2 and 3×3×3; only the 3×3×3 has an engine so far.

## Usage

```rust
use cubacle::{apply_moves, from_moves, solve};

let cube = from_moves("R U R' U'").expect("valid scramble");
let solution = solve(&cube, 20);
let solved = apply_moves(&cube, &solution).expect("valid solution");

assert_eq!(
    solved,
    "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
);
```

All text inputs are `&str`; callers can pass string literals, slices, or `&String` values without allocating a new string.

### Facelet order

Facelets must be supplied in this order:

```text
U1U2...U9R1R2...R9F1...F9D1...D9L1...L9B1...B9
```

The solved cube is:

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

`solve(facelet, maxl)` returns a solution of at most `maxl` moves, or an `Error <code>` string when the input is invalid or no solution is found within the requested limit. A limit of 20 or 21 moves is recommended.

## API overview

- `solve(facelet: &str, maxl: u8) -> String`: solve a valid facelet representation.
- `solve_with_stats(facelet: &str, maxl: u8) -> SolveStats`: solve with search counters and a `SolveOutcome`.
- `solve_with_budget(facelet: &str, maxl: u8, budget: Duration) -> SolveStats`: solve with a wall-clock search budget.
- `solve_with_minimum_time(facelet: &str, maxl: u8, minimum: Duration, maximum: Duration) -> SolveStats`: retain and improve a solution until the minimum search time, subject to a hard maximum.
- `solve_pattern(from_facelet: &str, to_facelet: &str, maxl: u8) -> SolveStats`: find moves that transform one valid cube state into another.
- `solve_batch(facelets: &[T], maxl: u8) -> Vec<SolveStats>` where `T: AsRef<str> + Sync`: solve independent cubes concurrently while preserving input order.
- `solve_parallel`, `solve_parallel_with_budget`, and `solve_parallel_with_minimum_time`: experimental six-orientation single-cube searches with deterministic result selection.
- `solve_pattern_parallel(from_facelet, to_facelet, maxl) -> SolveStats`: experimental parallel goal-state solving.
- `from_moves(cube_moves: &str) -> Option<String>`: apply a move sequence to a solved cube.
- `apply_moves(facelet: &str, cube_moves: &str) -> Option<String>`: apply a move sequence to a valid cube.
- `random_cube() -> String` and `random_moves(n_moves) -> String`: generate valid random inputs.

The solver initializes its shared move and pruning tables on first use. For a batch of cubes, keep the process alive and call `solve` repeatedly so this one-time cost is amortized.

`SolveStats::outcome` distinguishes a normal solution, a deadline reached with a retained solution, a deadline reached without a solution, and invalid or unsolved input. For compatibility, `SolveStats::timed_out` remains `true` only for `DeadlineNoSolution` (`Error 8`).

### Goal-state solving

Use `solve_pattern` when the desired result is an arbitrary valid cube state rather than the solved cube:

```rust
use cubacle::{apply_moves, from_moves, solve_pattern};

let from = from_moves("R U R' U'").expect("valid source");
let to = from_moves("F2 D L2").expect("valid target");
let result = solve_pattern(&from, &to, 20);

assert_eq!(apply_moves(&from, &result.solution), Some(to));
```

### Batch throughput

`solve_batch` reuses a process-wide worker pool, avoiding per-solve thread creation. Set `CUBACLE_THREADS` to a positive worker count before its first use to override the default of `available_parallelism()`. The pool has no shutdown operation and lives for the process lifetime. It is optimized for many independent cubes, not lower latency for a single cube.

```sh
CUBACLE_THREADS=4 cargo run --release --example bulk_compare -- \
  examples/scrambles.csv examples/scramble-report.csv 4 - 100 100
```

The `solve_parallel*` APIs split one solve across the six URF orientations, complete every orientation search, and select the first valid result in fixed URF order. Thus unbounded parallel solves are deterministic for a fixed cube, move limit, solver version, and build. They use the same pool, so set `CUBACLE_THREADS` before their first use. Wall-clock budgets still have the same timing sensitivity as the serial budget APIs. This is an experimental latency mode: it can perform more total IDA* work than serial search. Validate it on the target machine before adopting it.

## Comparing a scramble batch

`examples/bulk_compare.rs` compares the solutions in a scramble CSV with solutions generated by this crate at a 19-move limit and a 100 ms minimum and maximum search time per cube. After finding an initial solution, it continues searching for a shorter one until the minimum is reached, and never exceeds the hard maximum. The input must use the header and column order in `examples/scrambles.csv`.

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

The summary distinguishes an exact move sequence, any same-length solution (including exact sequences), shorter solutions, longer solutions, errors, and timeouts. `SolveStats::timed_out == true` indicates `Error 8`: no solution was found before the hard limit. It is distinct from ordinary input errors.

To retry only `longer` rows from an existing report, use `-` for no row limit and pass `longer` after the minimum and maximum search times:

```sh
RUSTFLAGS='-C target-cpu=native' cargo run --release --example bulk_compare -- \
  examples/scramble-report-100ms.csv examples/scramble-report-longer-200ms.csv \
  12 - 200 500 longer
```

Unbounded solves are deterministic for a fixed cube, move limit, solver version, and build. Time-bounded solves are intentionally not deterministic: CPU scheduling and the point at which the search observes the time limit can affect the best result found.

`target-cpu=native` enables CPU features available on the host, including Apple M2 systems. Do not use that flag for binaries intended to run on other CPU models.

## Benchmarking

The validating benchmark warms the solver tables, solves a fixed corpus repeatedly, and checks every returned sequence by applying it to the cube. It reports average, median, and p95 solver time, phase-two probes per second, and phase-one expansion/pruning counters, including the `break` versus `continue` split for each phase-one pruning estimate. The argument is the number of passes over the corpus:

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
cargo clippy --release --all-targets -- -D warnings
cargo test --release --all-targets
```

See `examples/main.rs` for a runnable demonstration.
