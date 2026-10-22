---
title: "4x4x4 port plan"
description: "Plan to port the TPR-4x4x4-Solver to Rust: licence, module map, 4x4 specifics, multi-core, benchmark and commits."
weight: 90
---

# 4x4x4 port plan

Status: design only, no engine code. This page is for contributors. It is the
plan for the 4x4x4 engine and needs the owner's approval before any code lands.

Numbers are labelled. **Claim** is what the reference README says. **Measured**
is a run on 2026-10-22 on an Apple M2 Pro (12 cores, 16 GB), Java 21, one JVM
thread. **Estimate** is reasoning, not a run. **Unverified** is stated as such.

## Decision

The 4x4x4 engine is a port to Rust of
[`cs0x7f/TPR-4x4x4-Solver`](https://github.com/cs0x7f/TPR-4x4x4-Solver) (Java,
Chen Shuang), which is Tsai's 8-step reduction with steps 3 and 4 merged and a
3x3x3 solver for the rest. The 3x3x3 part (the bundled min2phase jar) is not
ported: it is replaced by the injected `Solver` trait, as the 5x5x5 engine does.

Why a port and not an own search: the owner's decision of 2026-10-22, and the
measurement below. The reference averages 44.6 moves; the greedy 3-cycle method
of the first 5x5x5 engine (median 412 moves there) is dropped for the
4x4x4; a few hundred moves on a 4x4x4 is the estimate, not a measurement.

## 1. Licence

The reference has no `LICENSE` file. Its `README.md` carries two sections, both
by Chen Shuang, quoted here as read in the clone:

- "License GPLv3: Copyright (C) 2023 Shuang Chen ... either version 3 of the
  License, or (at your option) any later version."
- "License MIT: Copyright (c) 2023 Chen Shuang. Permission is hereby granted,
  free of charge, to any person obtaining a copy of this software ... The above
  copyright notice and this permission notice shall be included in all copies or
  substantial portions of the Software."

Two source files (`Search.java`, `MainProgram.java`) carry a 2014 header
"Copyright (C) 2014 Shuang Chen" with the GPL-3.0-or-later text only. The other
files have no header. The 2014 headers and the 2023 README do not contradict
each other (the README offers the same GPL plus MIT), but they are not
reconciled either; this is the same situation as the min2phase core (see
`NOTICE.md`).

Decision: take the **MIT option**, the same grant the 3x3x3 core already retains
in `LICENSE-MIT`. The combined work stays AGPL-3.0-or-later. Even if only the
GPL-3.0 header applied, GPL-3.0 code may be combined with AGPL-3.0 code
(GPLv3 section 13), so the plan stays valid; it is an open question for the
owner whether to ask the author for a `LICENSE` file (below).

Steps, in this order, before any ported code:

1. `NOTICE.md` gets a row: "4x4 reduction engine (`src/engines/four/`) |
   cs0x7f/TPR-4x4x4-Solver | MIT or GPL-3.0-or-later (README) | MIT taken;
   AGPL-3.0-or-later for this work, MIT notice retained", and the MIT text with
   the copyright line "Copyright (c) 2023 Chen Shuang" is quoted there.
2. Every ported Rust file starts with a header naming the Java file it
   derives from, the upstream copyright line, and "ported to Rust, modified;
   AGPL-3.0-or-later with the MIT notice of the upstream retained".
3. Not ported: `MainProgram.java` (GUI), `test.java`, `solver.java` (command line
   drivers; we have our own CLI), and the `twophase.jar` classes. The reference's
   Tsai algorithm description is public; the link is kept in the module docs.

## 2. The reference, as read

Source: `src/*.java`, about 73 KB in 14 files (the GUI is 13 KB of it), read for this plan. Java static
tables and a mutable global `inited` flag are the main structural problem.

| Phase | Java classes | What it does | Moves used | Tables |
| --- | --- | --- | --- | --- |
| 1 | `Center1` | Put the 8 centres of one colour axis on the two opposite faces of that axis. Tries the three axes. | 27 of 36 (U R F D L B and the wide u r f; d l b follow by symmetry) | 735,471 raw states, 15,582 after 48-fold symmetry; move table 15,582 x 36, prune table 15,582 |
| 2 | `Center2` | Sort the remaining centres towards the arrangement phase 3 can finish (the 16 up/down and front/back centres by axis class, the 8 right/left ones by exact colour); tracks the parity bit of the edges. The exact goal is confirmed when the file is ported. | 23 of 28 (18 face turns, u2, r r2 r', f2) | 6,435 x 70 states, prune table 450,450 bytes; move tables 6,435 x 28 and 70 x 28 |
| 3 | `Center3`, `Edge3` | Solve the centres and bring the 24 wings into the reduced state, tracking the parity of edges and corners (the `Edge3` coordinate is the relative arrangement of the two wing halves; zero is the reduced state, to be confirmed when ported). | 17 of 20 (U D F B quarter turns, R2 L2, u2 r2 f2) | centres 29,400 states x 20 moves; edges 1,538 x 20,160 = 31,006,080 symmetry-reduced states in 2 bits (7.75 MB) |
| 4 | `Search` + min2phase jar | 3x3x3 on the corners and the paired edges. | outer turns | jar tables (`twophase.data`, 0.96 MB) |

`Search.doSearch` is a candidate pipeline: phase 1 enumerates up to 10,000
solutions (`PHASE1_SOLUTIONS`) and keeps the best 500 in a priority queue
(`PHASE2_ATTEMPTS`) by length plus the phase 2 pruning value; phase 2 keeps up to
100 results (`PHASE2_SOLUTIONS`) that pass `checkEdge`; phase 3 tries up to 100
candidates (`PHASE3_ATTEMPTS`) at growing total length; the first success is
taken, then min2phase is called with `solution(facelet, 21, 1000000, 500, 0)`
(at most 21 moves, at least 500 probes, so it improves the first solution).
`FullCube.getMoveString` maps the moves back through the symmetry group and
appends **a final whole-cube rotation** (`x`, `y2`, `x y2`, ...). Everything is
deterministic: two runs of the same cubes gave identical output (measured on 20).

### Measured against claimed

Method, outside the repo (nothing of the reference is copied into the repo):
build the reference with its own `make.sh` in a temporary directory; a small
Java harness (own code, public API only) generates N cubes with the reference's
public `Tools.randomCube(Random)` and solves them in one JVM after
`Search.init()`; a throwaway Rust verifier depends on `cubacle` and replays every
answer with `Facelets::apply`. A final rotation is replayed as wide turns
(`x` = `Rw Lw'`, `y` = `Uw Dw'`, `z` = `Fw Bw'`), and the cube must then equal
`Facelets::solved(Cube4)` exactly.

| Quantity | Claim (README, 2011 i7-2670QM) | Measured (M2 Pro) |
| --- | --- | --- |
| Mean length | 44.39 moves, 2000 cubes | README command `test 2000` (seed 42): `test 2000` (seed 42) prints 45.05 (it counts the rotation characters as moves) and 128.9 ms mean; counting moves without the rotation from its output: mean 44.57, median 45, p90 46, min 38, max 48 (not replay-verified). Own run, 1000 cubes (seed 1), rotation not counted: mean 44.59, median 45, p90 46, min 40, max 47. With the rotation turned into wide turns: mean 47.71, p90 50 |
| Replay with `Facelets::apply` | not claimed | 1000 of 1000 solved; the facelet layout of the reference (index `face*16 + row*4 + col`, faces `URFDLB`) equals ours; wide moves `Uw` mean two layers, as in our notation |
| Mean time per cube | 250 ms | 131 ms mean, 112 ms median, 223 ms p90, 694 ms max over 1000 cubes (warm JVM). The first 20 cubes of a cold JVM average 314 ms |
| Phase split of the time | not claimed | phase 1 6.7 ms, phase 2 60.1 ms, phase 3 52.1 ms, 3x3x3 11.7 ms (1000 cubes, own timers in a throwaway copy) |
| Phase split of the length | not claimed | 6.75 + 5.81 + 12.40 + 19.60 = 44.56 (300 cubes, mean) |
| Initialisation | 6 to 7 s | `Search.init()` 0.81 s (five runs 805 to 825 ms; 2.37 s on the very first, cold run). Command line, cold JVM, no data files: 1.23 s wall; with the data files: 0.87 s |
| Memory | at most 30 MB with min2phase | runs with `-Xmx24m`; resident set 93 to 100 MB (JVM overhead) and 167 MB at default heap; the table sizes below are the number that matters |
| Table file | about 20 MB, md5 `0ee2f10d...` | `threephase.data` 9,995,328 bytes (2,243,808 + 7,751,520 as computed), `twophase.data` 964,596 bytes; md5 `208740d4...` differs from the README, unexplained, irrelevant to the port |

The Orbit64 reference vector for 4x4x4 (`BJStkD4cayBhsWj6eHgDZLTP1th`) is a
valid cube for the reference: solved in 45 moves, replay ok.

Not measured: Java memory with a profiler; per-cube time on a cold CPU; any
other solver.

## 3. Module map

New directory `src/engines/four/`, public item `FourSolver` only. Every file
below 500 lines, items `pub(crate)` or private, no `static mut`, no global state:
the Java statics become fields of one immutable `Tables` value behind
`Arc<OnceLock<Tables>>`, built by `prepare()` or on first use, shared by clones.
Per-search buffers (`move1..3`, `tempe`, the candidate queues) live in a
`Searcher` created per call, so concurrent solves share nothing mutable.

| Rust file | From Java | Content |
| --- | --- | --- |
| `four/mod.rs` | | docs, `pub use solver::FourSolver` |
| `four/solver.rs` | `Search.solution`, `solver.java` | `FourSolver::new(Arc<dyn Solver + Send + Sync>)`, `Solver` impl, budget and limit accounting, replay check |
| `four/moves.rs` | `Moves` | 36 move ids (U R F D L B, then u r f d l b wide of two layers, three powers), conversion to `puzzle::Move`, the redundancy tables `ckmv`, `skipAxis` for the three move sets |
| `four/cube.rs` | `CornerCube`, `EdgeCube`, `CenterCube`, `FullCube` | cubie state (8 corners, 24 wings, 24 centres), moves, lazy move buffer |
| `four/facelets.rs` | `FullCube(byte[])`, `toFacelet`, `to333Facelet` | read and validate a `Facelets`, build the 3x3x3 `Facelets` for the hand-off, rotation to wide turns |
| `four/sym.rs` | `Center1.initSym`, `symmult`, `symmove`, `syminv`, `finish`, `rot2str` | the 48 symmetries and their tables |
| `four/center1.rs` | `Center1` | coordinate, symmetry reduction, move and prune tables |
| `four/center2.rs` | `Center2` | coordinate, move and prune tables |
| `four/center3.rs` | `Center3` | coordinate, move and prune tables |
| `four/edge3.rs`, `four/edge3_prune.rs` | `Edge3` | wing coordinate, move/rotation tables, 2-bit prune table and its builder |
| `four/phase1.rs`, `phase2.rs`, `phase3.rs` | `Search.search1/2/3`, `init2/3` | the IDA* searches, written without recursion over mutable statics |
| `four/candidates.rs` | `PriorityQueue`, `FullCube.compareTo` | bounded best-N list, ordering rules |
| `four/handoff.rs` | tail of `doSearch` | 3x3x3 call through the trait, tail verification |

Reference sizes (Java, for the 500-line budget): `Edge3` 11.9 KB, `Center1` 7.2 KB,
`Search` 10.7 KB, `FullCube` 8.8 KB; each splits cleanly.

### Tables, build time, memory (Rust)

Estimate from the array sizes in the reference, not a measurement: `ctsmv` 2.24 MB,
`eprun` 7.75 MB, `Center3.ctmove` 1.18 MB, `Center2.ctmv` 0.36 MB, `ctprun`
0.45 MB, the rest well under 1 MB: about 12 MB plus the injected 3x3x3 tables.
Build time: Java needs 0.81 s with min2phase; the Rust port is expected in the
same range single-threaded and well below it with the parallel builder
(section 6). To be measured in the first table commit.

Cold start for the CLI: no disk cache. The Java solver writes `twophase.data`
and `threephase.data` into the working directory; a library or a stateless
server must not write files implicitly. The measured saving of the cache is
0.36 s of 1.23 s, so the first port builds in memory. A cache is a later,
opt-in feature if the Rust build turns out slower than the Java one.

## 4. The 4x4x4 specifics

### Reading a facelet string

`Facelets` is already the right input: 96 stickers, `URFDLB`, row-major, which is
the reference's own layout (replay verified, above). There are no fixed centres,
so a facelet string does not name its colour scheme. The reference assumes the
standard lettering (the letters are the colours, `U` and `D` opposite, `R` and
`L`, `F` and `B`, with the usual handedness) and reads corners relative to `U`/`D`.
The port keeps that assumption and validates instead of trusting it (the
reference has no validation and can throw `ArrayIndexOutOfBoundsException`, a
known item in its README TODO):

- every wing found exactly once among the 24 ordered colour pairs, every corner
  exactly once with an orientation, otherwise `Failure::InvalidInput`;
- a lettering that is a valid cube but not the standard scheme (for example a
  mirrored one) is also rejected as `InvalidInput` with the message that the
  colours do not follow the `URFDLB` scheme. The other engines also refuse strings that are not a cube of the standard scheme. Cubes written in `W R G Y O B` are mapped to
  letters by the colour scheme, as today; for an even cube the scheme cannot be
  read from the centres, so the existing fallback (`boy` unless `--scheme` is
  given) applies and the CLI already documents it.

### What "solved" means, and the rotation

Without fixed centres a cube can be solved in any of the 24 orientations. The
reference solves the centres up to rotation and then prints the rotation that
brings the cube to the standard orientation. Decision: the contract of
`SolveResult` stays as it is (the solution turns the given cube into
`Facelets::solved(Cube4)`, no whole-cube rotations in a solution). The final
rotation is therefore appended as at most four wide turns (`x` = `Rw Lw'`,
`y` = `Uw Dw'`, `z` = `Fw Bw'`, composed for the others). Measured cost on 1000
cubes: +3.1 moves on average (44.59 to 47.71). `Facelets::is_solved`, the CLI
replay check and the server `debug_assert` need no new rule; the CLI replay
check only has to include `Cube4` in its size list (`main.rs`, now `Cube3 | Cube5`).

The alternative is to define a 4x4x4 as solved when all faces are uniform (any
orientation), which saves those 3.1 moves but adds a second meaning of "solved"
to the API. It is an open question for the owner. The benchmark reports both
lengths so either answer can be judged against the reference.

A saving that keeps the strict contract is possible later: among the phase 3
candidates of equal total length prefer the one with the cheapest final
rotation. Unverified, to be measured after parity with the reference.

### Parity

The reference has no parity algorithms. Parity is part of the coordinates:

- `Center2` has a parity bit (index `rl * 2 + parity`); the table `pmv` marks
  the moves that flip it (within the phase 2 move set, the quarter turns of
  the wide `r` and `l`, which move an inner slice; an inner-slice quarter turn is
  what changes the permutation parity of the wings);
- `Center3` combines the centre parity with the parity of the edge pairing and
  the corner parity (`eparity ^ corner.getParity()`), and its goal state
  requires that bit to be 0. A search that ends there ends in a cube whose
  corners and paired edges form a solvable 3x3x3;
- `Search.init3` rejects phase 2 results whose wings are not in the expected
  halves (`EdgeCube.checkEdge`: one wing of every pair among the first 12).

Consequence for the port: both parity cases of a 4x4x4 (a single flipped paired
edge, and two swapped edges or corners) can occur in the input and are all
solved by the search itself; no fixed algorithms are needed, so nothing has to
be verified by labelled replay except the move tables. Evidence: 1000 random
cubes, whose wing and corner parities are independent coin flips by
construction, were all solved and replay-verified (measured; the four classes
were not counted separately). The port must keep the loops bounded: the reference runs
`do { ... } while (length12 == 100)` and can spin on an unsolvable input; the
port returns `Failure::Exhausted` at length 100.

Which cases are impossible: a corner twist that does not sum to 0 modulo 3 and a
colour count other than four centres per colour. The permutation parities of
corners and wings are independent, because centres of equal colour are
interchangeable.

### The hand-off to the 3x3x3

`to333Facelet` needs solved centres (it throws "Unsolved Center" otherwise) and
writes a 54-sticker string: edges from the paired wings, centres from the four
centre stickers, corners. In the port this is `Facelets::from_stickers` with the
stickers of the 3x3x3 positions, handed to the injected solver through the trait
with `max_moves` 21 as in the reference, then verified exactly as the 5x5x5
engine verifies its tail (replay on the reduced cube, then on the original).
A stub solver is used in the unit tests, as for the 5x5x5.

The 3x3x3 stage decides 19.60 of the 44.56 moves in the reference because it uses
a probe minimum of 500. Our 3x3x3 trait call stops at the first solution. Measured
on 500 random 3x3x3 states: first solution mean 20.84 moves in 1.2 ms; asking
the trait again with one move less until it fails (20 ms budget per step) gives
19.49 moves in 25 ms per cube. So without a change the port loses about 1.3
moves, and the tightening loop is time-based, hence not deterministic. Preparatory
proposal (needs approval, it touches the 3x3x3 search options): a probe-count
minimum in `SolveOptions` that `Min2PhaseSolver` honours, mirroring min2phase's
`probeMin`. Without it the acceptance threshold in section 8 cannot be met.

### Random states and validity

`generate::random_state(Cube4)`: corner permutation uniform over 8! with a twist
vector summing to 0 modulo 3; the 24 wings a uniform permutation of the 24
ordered pairs; the centre colours a uniform arrangement of 4 x 6 colours. All
independent, no parity coupling. Reason, measured: the number of such colour
states is 8! 3^7 24! (24!/(4!)^6) = 1.776e47 = 156.96 bits; divided by 24 it is
the well-known 7.401e45 positions, and 156.96 bits is the figure of the Orbit64
format for the 4x4x4. The reference's own generator (`FullCube(Random)`) does
exactly this, and all 1000 states it produced were solved. Also needed:
`generate::scramble` must treat `Cube4` like `Cube5` (two layer depths).

### Orbit64 for 4x4x4

`FORMAT.md` (flix-orbit64): the state is coordinates in `Orbit.layout(n)` order
plus a frame; on even cubes the frame is required to be canonical (rank 0)
because the movable centres already fix the pose; width 27 characters for 156.96
bits; one reference vector. Feasible in principle: our facelet convention equals
the format's, and the vector decodes to a cube the reference solves. Unverified:
the slot numbering of the 24 wings and 24 centres and the ranking of the centre
multiset permutation (the document defines the order only through the 6x6x6 and
7x7x7 extension rule and the source of `Orbit.layout`, which was not read), with a
single vector as the only evidence, and the existing caveat that tokens do not
prove the producer's numbering. Decision: decoding is a separate commit after the
engine, accepted only if the one vector decodes to its facelets; more vectors
from the format's author are requested. Until then a 4x4x4 token stays
`Orbit64Error::Unsupported(4)`; the server maps it to `token_unsupported` once the
engine exists (it maps to `no_engine` today).

## 5. Interfaces

- `FourSolver::new(Arc<dyn Solver + Send + Sync>)`, `Clone`, `Debug`, `Solver`
  impl with `kind() == Cube4` and `prepare()` building the tables and preparing
  the injected solver. Optional `with_threads(n)` (section 6).
- CLI: `cubacle solve` dispatches `Cube4` to `FourSolver`; `scramble`, `random`
  and `facelets` accept `--size 4`; the default move limit is none for the 4x4x4;
  the replay check includes `Cube4`; wide moves in the output are valid notation.
- Server: `engine_kind` accepts 4; `AppState` gets `four: FourSolver::new(three)`
  built at startup and prepared; sizes 6 and 7 stay `501 no_engine`; `maxMoves`
  uses the large bound of the 5x5x5. One 4x4x4 solve takes one thread by default
  in the server (requests already run in parallel); a setting raises it.
- Documentation to update with the engine: `docs/engines.md`, `docs/cli.md`,
  `docs/api.md`, `ARCHITECTURE.md`, `NOTICE.md`.

## 6. Multi-core

Where the reference time goes (measured): 90% is phases 2 and 3, which are loops
over independent candidates. Parallelising them keeps the answer identical to
the sequential one if the merge follows the sequential order. Design:

| Part | Unit of work | Merge rule that keeps the sequential result |
| --- | --- | --- |
| Phase 1 (5% of time) | each axis, then each first move, per length | concatenate the per-task solution lists in axis order, cut at 10,000 |
| Phase 2 (46%) | one candidate of the 500 per length | concatenate in candidate order, cut at 100 |
| Phase 3 (40%) | one candidate of the 100 per length | the lowest candidate index that succeeds wins; later tasks are cancelled |
| 3x3x3 (9%) | the injected solver | through the trait only a single-threaded solve is available; the 3x3x3 engine's parallel entry points are not on the trait |
| Table build | the Edge3 breadth-first scan over 2-bit entries | per-thread chunks of the scan; entries are set with an idempotent atomic AND of the 2-bit field, so concurrent duplicate writes are harmless and the table is bit-identical for every thread count |
| Many cubes | one cube per task | `solve_batch`-style, results in input order |

Thread control: an explicit `with_threads(n)` on the solver, default from
`CUBACLE_THREADS` (same variable and meaning as the 3x3x3 pool: positive integer,
otherwise available parallelism). Workers are `std::thread::scope` threads per
solve with an atomic work counter, so there is no global pool (the 3x3x3 pool in
`engines/three/pool.rs` is private to that engine and must not be imported; moving
it to a shared module is an option if per-solve spawning measures slower).
Deterministic: the solution is identical for every thread count unless a time
budget expires. Not deterministic by nature: anything under a time budget.
Estimate by Amdahl with 90% parallel work: 4 threads about 3x, 8 threads about
4.7x; to be measured, reported as speedup of mean and p90 latency for 1, 2, 4, 8
and 12 threads on the same cubes, plus a test that 1 and 4 threads produce the
same 20 solutions.

## 7. Shared framework with the 5x5x5 port

Another author ports `cs0x7f/cube555` (GPL-3.0 only in its README, five phases,
a `PhaseSearch` base class and `Coord`/`SymCoord`/`RawCoord` helpers in
`Util.java`; README claims 51 to 52 moves, 1 to 2 s, tables about 240 MB, licence
GPLv3 in the README, whether it also offers MIT was not checked). Its sources
were not read for this plan; only the file list and the README. Proposal, to be
aligned through the coordinator: a module `src/engines/reduction/` for code that
is written once, from scratch in Rust, and owned by neither port:

| Shared (`engines/reduction`) | Used by 4x4 as | Notes |
| --- | --- | --- |
| `combinatorics`: binomials, subset and permutation ranks, parity | `Cnk`, `set8Perm`, `Util.parity`, the coordinate code | pure functions, unit tests |
| `prune`: byte-depth and 2-bit mod-3 tables, serial and parallel breadth-first builders | `createPrun` of Center1/2/3 and Edge3 | the atomic AND builder lives here |
| `candidates`: bounded best-N list with a defined tie order | the `PriorityQueue` of phase 1 | tie order documented and tested |
| `parallel`: thread count setting, ordered "first success by index" and "ordered concat" helpers | section 6 | no global pool |
| `handoff`: injected 3x3x3 call, budget and limit accounting, tail verification, replay | `four/handoff.rs`, and the existing `five/solver.rs` logic | extracted from `five`, which today has the only copy |

Not shared: the phase coordinates and move tables (cube-size specific), the
cube model, anything copied from either Java code base. Licensing consequence:
the shared module contains no ported code, so it carries no third-party notice;
each port keeps its own upstream licence in its own directory (TPR: MIT option;
the cube555 licence is the other author's question, and keeping the shared
module free of ported code means it cannot depend on that answer).

The earlier idea of extracting the 5x5x5's move vocabulary and 3-cycle library
for the 4x4x4 is withdrawn: the port does not use 3-cycle libraries. The existing
`engines/five` stays unchanged and its golden test untouched until the 5x5x5 port
plan decides its fate. `tests/architecture_test.rs` gets a rule for
`engines/reduction` (may use `puzzle` only) and for `engines/four` (`puzzle`,
`engines::reduction`, itself); `five` may use `reduction` once it does.

## 8. Acceptance criteria and benchmark

Same cubes for both solvers. The harness lives outside the repo: the Java side
uses only the public API of the reference; the Rust side is an `examples/` binary
that can be committed later because it contains no reference code.

1. Cubes: N = 1000, produced by the reference's public `Tools.randomCube(Random(1))`
   and stored as facelet strings; 100 of them are also produced by our
   `generate::random_state(Cube4)` as a separate check.
2. Every answer of both solvers is replayed with `Facelets::apply` and must
   equal the solved cube; one failure fails the run.
3. Length (moves, rotation as wide turns reported separately): Java mean 44.59,
   median 45, p90 46, max 47 (measured above). Pass: Rust mean at most 44.7
   (reference plus 0.1), p90 at most 46, max at most 48; target: below the
   reference. Report mean, median, p90 and the histogram.
4. Phases 1 to 3 are compared bit for bit on 40 cubes against the reference
   (prefix of the answer of length `l1 + l2 + l3`, taken from an instrumented
   reference run). Pass: identical. This holds only if the candidate queue
   reproduces the Java tie order; if it cannot, the pass criterion falls back to
   equal mean lengths of phases 1 to 3 within 0.05 over 1000 cubes, and the
   queue's order is documented.
5. Time, one thread, release build, warm: median at most 1.5 x the Java median
   of 112 ms, p90 at most 1.5 x 223 ms. With 4 threads: mean latency at most 0.5 x
   the single-thread mean (estimate says 0.33).
6. Table build: at most 1.5 s single-threaded (Java: 0.81 s with min2phase);
   report threaded times. Memory: peak resident set of the 4x4x4 solve with its
   tables at most 64 MB plus the 3x3x3 solver (tables about 12 MB by estimate).
7. Thresholds are proposals for the owner, derived from the measurement above,
   not from the README claims.

## 9. Tests (CI-fit)

- Unit: coordinate set/get round trips, the 48-symmetry tables (group laws), move
  tables against a cubie simulation on random states, prune tables against the
  reference's own depth counts for Edge3 (1, 4, 16, 55, 324, 1922, ... measured
  from the reference output) and for the centre tables; ranks and parity.
- Golden: `tests/fixtures/four.txt` in the format of `five.txt` (about 40 cubes
  with length and hash of our output, plus a few cubes that must be refused) and
  a second file with the phase 1 to 3 prefix of the reference for the bit-identical
  check. A change of solver output needs an explicit decision, as for the others.
- Orientation: the solved cube, the solved cube rotated by `x`, `y` and `z` (built
  with wide turns), a cube whose centres are exchanged. Parity: states built from the
  reduced state by one wing pair flipped, two edge pairs swapped, both. Failure:
  wrong size, a lettering that is not the standard scheme, a duplicated wing, a
  twisted corner (3x3x3 stage refuses), a stub that fails or lies (replay catches
  it), tight `max_moves`, zero time budget.
- No test asserts a time. The 1000-cube comparison and the speedup runs are
  `#[ignore]` or an example binary; CI runs the unit tests, the golden test and one
  end-to-end solve through the CLI. All tests share one table set through a
  `OnceLock`, as the 5x5x5 tests do.
- Determinism: 20 cubes with 1 and 4 threads give identical answers.
- Existing gates unchanged: `cargo fmt --check`, clippy with `-D warnings`,
  `cargo test --workspace --release`; the 5x5x5 golden test stays untouched.

## 10. Commits and who can build what

Atomic commits, each building and passing the gates. `[A]`, `[B]`, `[C]` mark
three streams that can run in parallel once commit 4 is in.

1. `docs(notice): record the TPR-4x4x4-Solver port` (row and MIT text; before code).
2. `feat(reduction): add combinatorics helpers` (ranks, binomials, parity).
3. `feat(reduction): add prune tables and breadth-first builders` (serial).
4. `feat(four): add the cube model, moves and facelet reading` (validation, the
   hand-off facelets, rotation to wide turns). Rules in `architecture_test` in the
   same commit as the directories.
5. `[A]` `feat(four): add the symmetry tables`, then `feat(four): add the phase 1
   centre tables`, `phase 2`, `phase 3 centres`.
6. `[B]` `feat(four): add the wing coordinate and its prune table` (Edge3).
7. `[A+B]` `feat(four): add the candidate pipeline of phases 1 to 3` with the
   bit-for-bit fixture.
8. `feat(four): add FourSolver and the 3x3x3 hand-off` with stub tests.
9. `[C]` `feat(generate): add 4x4x4 random states and scrambles`.
10. `[C]` `feat(cli): solve and generate 4x4x4 cubes`.
11. `[C]` `feat(server): serve 4x4x4 solves`.
12. `test: add the 4x4x4 golden record and the parity, orientation and failure cases`.
13. `perf(four): evaluate candidates on several threads` with the determinism test.
14. `perf(reduction): build prune tables on several threads`.
15. `docs: describe the 4x4x4 engine, CLI and API`; notes.
16. optional `feat(notation): decode 4x4x4 Orbit64 tokens`.
17. optional preparatory, on its own: probe-count minimum in `SolveOptions`
    (touches the 3x3x3 engine; needs approval; should come before 12).

Streams: A is the three centre phases and symmetry (pure coordinate work), B the
wing table and its builder, C the application layer (generate, CLI, server) which
needs only the `FourSolver` signature from commit 8. A benchmark harness and
fixtures generation (outside the repo) can be done by a fourth person at any time.

## 11. Risks

| Risk | Mitigation |
| --- | --- |
| Licence headers: GPL-only 2014 headers in two files next to the dual README | MIT option recorded, GPL-3.0 combination with AGPL-3.0 allowed anyway; ask for a `LICENSE` file |
| Java quirks (statics, unvalidated input, unbounded loops) copied by a literal port | validation and bounded loops are explicit requirements and tests |
| 3x3x3 tail 1.3 moves longer without a probe minimum | preparatory commit 17, or the threshold is missed |
| Bit-identical phases 1 to 3 depend on heap tie order | fallback criterion in section 8 |
| Table build slower in Rust than Java, hurting the CLI and CI | parallel builder, `OnceLock` shared in tests, measured in commit 3 |
| Solutions contain wide moves (`Rw`) and, when the contract is kept, a trailing rotation as wide turns | documented in the CLI and API pages; clients that only know single layer turns are affected |
| Server latency under load with several threads per solve | one thread per solve by default |
| Orbit64 4x4x4 mapping wrong with one vector | separate commit, acceptance by the vector only, more vectors requested |
| Two ports diverge on shared code | single owner for `engines/reduction`, interface agreed before commit 2 |

## 12. Open questions for the owner

1. Contract for the final rotation: strict (append up to four wide turns, +3.1
   moves) or "solved in any orientation" with a second meaning of solved.
2. Approve the preparatory probe-count minimum in `SolveOptions` and the 3x3x3
   engine, or accept about 1.3 moves more.
3. Ask Chen Shuang for a `LICENSE` file or accept the README as the grant.
4. Bit-identical phases 1 to 3 as the acceptance test, or distribution only.
5. No on-disk table cache in the first version.
6. Server default of one thread per 4x4x4 solve.
7. The thresholds in section 8.
8. Order with the 5x5x5 port and the owner of `engines/reduction`.
9. More Orbit64 4x4x4 reference vectors from the format's author.

## What was not verified

Rust table sizes, build time and speed (no code exists); the speedup of the
parallel design; the exact meaning of the `Edge3` coordinate beyond what the
replay shows (to be confirmed when the file is ported); Orbit64 slot order for
the 4x4x4; behaviour of the reference on invalid or non-standard input (read
from the code, not run); the README claim of about 20 MB of tables (the data
files total 10.96 MB); the md5 in the README.
