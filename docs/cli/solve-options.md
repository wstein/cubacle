---
title: "Solve options"
description: "Within, move counts, known patterns, restricted move sets, listing solutions and failures."
weight: 2
---

# Solve options

### Phases of about 20 moves

A phase is where a person checks the cube, so the engines of the 4x4x4 to 7x7x7 report phases of 10 to 20 moves where they can. The phases are named after what can be seen at their end: the 4x4x4 has `centers staged`, `centers sorted, wings split`, `centers solved, wings paired`, `3x3 stage`; the 5x5x5 `centers on their axes`, `wings sorted`, `centers solved, wings paired`, `3x3 stage`; the 6x6x6 `LR centers on L/R`, `UD/FB centers on their axes`, `inner centers solved`, `inner wings paired`, `outer wings paired, centers solved`, `3x3 stage`; the 7x7x7 the same with `centers solved`, `middle wings paired`, `outer wings paired`. A stage over 20 moves is cut at the ends of the engine's own steps into numbered parts such as `centers solved, wings paired (1/2)`. The borders are for grouping only: a move may sit on the wrong side of one, and no answer is longer for it. Measured on 20 random cubes per size (`cubacle random`, then `solve --group phases`), phase lengths in moves:

| Cube | Before: phases per solution, min / median / max | After: phases per solution, min / median / max |
| --- | --- | --- |
| 4x4x4 | 4, 4 / 9.5 / 21 | 4, 4 / 9.5 / 21 |
| 5x5x5 | 4, 4 / 19 / 28 | 5, 5 / 14 / 21 |
| 6x6x6 | 3 to 6, 15 / 20 / 97 | 7 to 11, 6 / 14 / 21 |
| 7x7x7 | 1 to 6, 20 / 32 / 190 | 13 to 16, 7 / 12 / 21 |

The phases per solution of the "before" runs vary because a short stage end used to be placed wrongly and all moves fell into one phase; the 21 moves are the 3x3 stage. The moves of the answers did not change.

#### --every N

`--every N` cuts the answer after every N moves instead, whatever the solver phases are: ` · ` between the parts, and with `--icons` a heading `Phase 1: moves (N moves, ...)` per part. It replaces the solver phases and combines with `--group axis` (a unit never crosses a cut). On a 4x4x4:

```sh
cubacle solve --size 4 -s "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw" --every 10
```

```text
B' L' F' R Bw Dw' B F' R' Dw2 · L' Rw2 Fw' D' L' R' Fw2 U' Bw2 R2 · L' F2 Dw2 R U Rw2 U B U L2 · B2 D2 F' U F2 L2 D2 R B2 L · F2 R2 U2 L' B' L
```

#### --inverse

`--inverse` prints the solution inverted: the moves that build the start cube from the solved cube (reversed, each turned the other way). With `--to` they lead from the target to the start. The solver runs as always; the `moves` line and the metrics on standard error stay those of the solution, the answer on standard output is the inverse. Everything that cuts the answer is mirrored with it: the phase list is reversed and each phase is named `undo phase N: <name>` after the phase it undoes (so `-v` and the pictures show `undo phase 4: 3x3 stage` first), `--every N` cuts the inverted moves, and `--optimize ranges` compresses after inverting. With `--icons` the first picture is the cube the solution ends in (the solved cube, or the target), and the target state line is the start cube. `--inverse` is refused with `--now`, `--all` and `--count`, which print other things than one solution.

```sh
cubacle solve -s "R U F D2 L' B R2 U' F2 D L2 B' U2 R F"
cubacle solve -s "R U F D2 L' B R2 U' F2 D L2 B' U2 R F" --inverse
cubacle solve --size 4 -s "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw" --inverse --every 10
```

```text
F' R' U2 B L2 D' F2 U R2 B' L D2 F' U' R'
R U F D2 L' B R2 U' F2 D L2 B' U2 R F
L' B L U2 R2 F2 L' B2 R' D2 · L2 F2 U' F D2 B2 L2 U' B' U' · Rw2 U' R' Dw2 F2 L R2 Bw2 U Fw2 · R L D Fw Rw2 L Dw2 R F B' · Dw Bw' R' F L B
```

The inverse of a solution of a scrambled cube is, on the 3x3x3, the scramble itself when the solver found no shorter way; the REST field `inverse` is the same text (see the [REST API](../api.md)).

#### --json

`--json` prints the answer as one line of JSON on standard output, the object the [REST API](../api.md) sends for `POST /v1/solve`: `kind`, `solution`, `inverse`, `length`, `metrics` (all seven counts), `compressed`, `compressedLength` and `compressedMetrics` with `--optimize ranges`, and `elapsedMicros`. After `elapsedMicros` come two keys that appear only when there is something to say: `phases`, an array of `{"name", "moves"}` objects, one per phase the engine reports (the 4x4x4 to 7x7x7 engines; the empty phases are in it with `"moves":0`; the 2x2x2 and 3x3x3 engines report none, so the key is absent), and `patternHint`, the lowercase name of the known pattern the start cube matches (for example `"superflip"`), only with `-v`, the same opt-in as the `pattern:` line. The server and the command write it with the same function of the library (`cubacle::report`), so key names, key order and escaping are the same. Nothing else goes to standard output; the lines on standard error (`moves N`, `metrics:`, and with `-v` the statistics) stay as they are. Errors are text on standard error with the usual exit codes, not JSON.

**Changed in 0.2.0:** format 2 is the default. After every existing key it appends
`formatVersion` (2), `toolVersion`, `gitCommit` (the release source revision, or null),
`frame`, `proven`, and `lowerBound`. `--json-format 1` preserves the original body byte for
byte; `--json-format 2` selects the default explicitly. The selector needs `--json`, and an
unknown version fails before solving (exit status 1). The corresponding REST selector is
`"formatVersion": 1` or `2` in the request body; the endpoint remains `/v1/solve`.

`frame` describes the goal actually searched: the ordinary 2x2 table solver uses
`"any-orientation"`, while `--within` uses `"fixed"` even on a 2x2. `proven` means the
returned path is shortest **in HTM for that goal and frame**. A requested `--optimal` or
`--within` flag alone does not prove the returned answer: a budget-limited incumbent or
`--within --then-solve` fallback carries its actual result's evidence. `lowerBound` is an
established HTM bound, equal to the HTM length for a proven answer, or null when absent.
Proofs in slices/blocks alphabets are not promoted to HTM proofs. Existing partial-target
routes that refuse `--json` continue to refuse it. There is no general exact-physical
optimality claim for a solution extended by a center fix-up or another composed route.

<!-- example:json-fast -->
```sh
cubacle solve -s "R U" --fast --max-moves 21 --json
```

```text
moves 13
metrics: htm 13, qtm 18, stm 13
{"kind":"3x3x3","solution":"U R F2 R F2 R' L D2 R' D2 L' U2 R'","inverse":"R U2 L D2 R D2 L' R F2 R' F2 R' U'","length":13,"metrics":{"htm":13,"qtm":18,"stm":13,"etm":13,"obtm":13,"btm":13,"atm":12},"elapsedMicros":0,"formatVersion":2,"toolVersion":"0.1.0","gitCommit":null,"frame":"fixed","proven":false,"lowerBound":null}
```
<!-- /example -->

The first two lines are standard error. The server, asked for the equivalent request, answers the same bytes:

```sh
curl -s -XPOST localhost:8080/v1/solve -H 'Content-Type: application/json' \
  -d '{"scramble":"R U","maxMoves":21}'
```

```json
{"kind":"3x3x3","solution":"U R F2 R F2 R' L D2 R' D2 L' U2 R'","inverse":"R U2 L D2 R D2 L' R F2 R' F2 R' U'","length":13,"metrics":{"htm":13,"qtm":18,"stm":13,"etm":13,"obtm":13,"btm":13,"atm":12},"elapsedMicros":0,"formatVersion":2,"toolVersion":"0.1.0","gitCommit":null,"frame":"fixed","proven":false,"lowerBound":null}
```

Byte equality holds only for equivalent requests, and `elapsedMicros` differs from run to run (the test suite masks it, and these pages show it as 0). `--json` uses the defaults of the command line, which are not those of the server: a plain 3x3x3 `solve` makes 3000 probes for a shorter answer and limits it to 21 moves, while the server makes no extra probes and sets no move limit. `--fast --max-moves 21` is the server's `{"maxMoves":21}`. Without `--fast` the same cube gives a shorter solution:

<!-- example:json-default -->
```sh
cubacle solve -s "R U" --json
```

```text
{"kind":"3x3x3","solution":"U' R'","inverse":"R U","length":2,"metrics":{"htm":2,"qtm":2,"stm":2,"etm":2,"obtm":2,"btm":2,"atm":2},"elapsedMicros":0,"formatVersion":2,"toolVersion":"0.1.0","gitCommit":null,"frame":"fixed","proven":false,"lowerBound":null}
```
<!-- /example -->

The 4x4x4 and larger cubes also differ: the command wraps the engine in the pattern shortcut and the server does not. `inverse` is always in the body, so `--json` with `--inverse` is refused (`error: --json always carries the inverse; do not combine it with --inverse`, exit status 1); `--now`, `--all`, `--count`, `--group`, `--every` and `--icons` are refused too, because they print something other than one solution. `canon --format json` is another shape and is unchanged. New keys of later versions are appended after the existing ones.

### A nearly solved cube: --within

`--within K` is for a cube that is only slightly scrambled: it finds an optimal route of at most K moves (K from 1 to 7; write `--within=5`, plain `--within` means 6). The search starts at the cube and at the target and meets in the middle on the colors of the stickers, so it needs no relative state and works for every size from the 2x2x2 to the 7x7x7 and for every target: the solved cube, `--to` as a move expression or as a state text, also a state with mixed center colors (made with slice moves). Every route is replayed color by color before it is printed.

```sh
cubacle solve -s "R U2 F' D L2 B" --within=6
cubacle solve -s "R U F2" --to "R U2 F' D" --within=4
cubacle solve -s "2R 2U' F R" --size 4 --to "2R 2U'" --within=3 --slices
```

```text
moves 6
metrics: htm 6, qtm 8, stm 6
B' L2 D' F U2 R'
```

```text
moves 4
metrics: htm 4, qtm 5, stm 4
F2 U F' D
```

```text
moves 2
metrics: obtm 2, qtm 2, stm 2, btm 2
R' F'
```

The answer is the shortest in an alphabet, and the message names it. The default is the alphabet the normal solver writes: the six face turns, from the 4x4x4 the outer blocks (`Rw`, `3Rw`, up to half the cube and three layers, from either side), counted in HTM (OBTM from the 4x4x4, the same rule). `--slices` adds the single inner layers (`2R`, `M`), counted in BTM, one per block. `--blocks` adds every block of two and three outer layers from either side and the layers 2-3 (written `3Rw R'`), and counts the blocks, a block of layers 2-3 once; the `moves N` line counts plain moves, so such a route can show a larger `moves N` than K. A larger alphabet finds shorter routes and searches more:

```sh
cubacle solve -s "2R 2U 2F" --size 4 --within=3
cubacle solve -s "2R 2U 2F" --size 4 --within=3 --slices
```

```text
error: no route of at most 3 moves in the outer-block alphabet (obtm)
```

```text
moves 3
metrics: obtm 6, qtm 6, stm 3, btm 3
3B 3D 3L
```

If nothing is found, stdout stays empty and the status is 2. Before it searches, `--within` counts the stickers that differ between the cube and the target in each orbit (corners, each kind of edge or center piece) and divides by the most stickers one move of the alphabet displaces in that orbit; a bound above K ends the run at once. The same bound proves a target unreachable when a move never touches an orbit (the centers of the 3x3x3 with face turns). The proof is in `src/meet/alphabet.rs` and a test checks the bound against the search.

`--then-solve` (it needs `--within`) goes on with the normal solver, with the same options as without `--within`, when no route of at most K moves exists:

```sh
cubacle solve -s "R U2 F' D L2 B R' U F2" --within=4 --then-solve
```

```text
no route within 4 moves; solved with the normal solver
moves 9
metrics: htm 9, qtm 12, stm 9
F2 U' R B' L2 D' F U2 R'
```

(the first line is on stderr). `--max-time MS` (default 60000, 0 for none) and `--max-memory MB` (default 1000) bound the search; when one ends it, stderr says `error: time limit` or `error: memory limit` and the status is 3. `--within` works with `--notation`, `--icons`, `--group`, `--optimize` and `-v`. It does not combine with `--now`, `--optimal`, `--moves`, `--count`, `--all` (one route is listed) and, without `--then-solve`, with the options of the normal solver (`-m`, `-b`, `--probes`, `--improve`, `--parallel`, `--fast`); each is refused by name. The 2x2x2 route ends on the exact solved cube, not on a whole-cube rotation of it.

Exit codes of `solve`: 0 success; 1 input error or refused option; 2 no route (or an impossible state); 3 a time or memory limit ended the search.

On the 3x3x3 the search is faster than `--optimal` for up to seven moves (6 moves: 7 ms against 0.47 s, because the large tables of `--optimal` have to be loaded) and gives the same length, so it is used there too. Measured with the release build on a busy Apple-silicon machine (load average 40 to 80, so read the times as upper bounds), a cube K moves from solved (a hit; the optimum was K) and one K+1 moves away (none: the search runs to the end). Seconds, then peak memory (RSS) in MB:

| size | K=4 | K=5 | K=6 | K=7 |
| --- | --- | --- | --- | --- |
| 2x2x2 | 0.01 s, 7 MB | 0.00 s, 7 MB | 0.01 s, 7 MB | 0.01 s, 8 MB |
| 3x3x3 | 0.01 s, 7 MB | 0.02 s, 7 MB | 0.04 s, 7 MB | 0.02 s, 8 MB |
| 4x4x4 | 0.01 s, 7 MB | 0.01 s, 8 MB | 0.01 s, 8 MB | 0.15 s, 21 MB |
| 5x5x5 | 0.01 s, 7 MB | 0.01 s, 8 MB | 0.01 s, 8 MB | 0.26 s, 21 MB |
| 6x6x6 | 0.01 s, 7 MB | 0.03 s, 9 MB | 0.06 s, 9 MB | 3.1 s, 78 MB |
| 7x7x7 | 0.01 s, 7 MB | 0.03 s, 9 MB | 0.07 s, 9 MB | 4.0 s, 78 MB |

With `--blocks` (the largest alphabet, 45 moves on the 3x3x3 and 117 on the 7x7x7):

| size | K=4 | K=5 | K=6 | K=7 |
| --- | --- | --- | --- | --- |
| 3x3x3 | 0.01 s, 7 MB | 0.01 s, 8 MB | 0.02 s, 8 MB | 0.19 s, 41 MB |
| 4x4x4 | 0.01 s, 7 MB | 0.04 s, 13 MB | 0.2 s, 13 MB | 4.2 s, 375 MB |
| 5x5x5 | 0.01 s, 8 MB | 0.07 s, 17 MB | 0.32 s, 17 MB | 16.8 s, 836 MB |
| 6x6x6 | 0.01 s, 8 MB | 0.12 s, 21 MB | 1.2 s, 21 MB | 57 s, 21 MB |
| 7x7x7 | 0.04 s, 8 MB | 0.45 s, 24 MB | 1.1 s, 24 MB | 26 s, 24 MB |

A "none" run takes about as long as a hit of the same K. The stored side is capped by `--max-memory`; on the 6x6x6 and 7x7x7 with `--blocks` the cap keeps it three moves deep, so K=7 walks four moves and takes up to a minute (the default limit is 60 s: raise `--max-time` there).

### Move counts

`solve` writes `moves N` and one more line to standard error, the counts in the common move metrics (`htm qtm stm` up to the 3x3x3, `obtm qtm stm btm` from the 4x4x4). Standard output is unchanged. The page [Move metrics](../metrics.md) defines each count, says what the solvers minimize (the move count of their own alphabet, not QTM or STM) and lists where else the line appears.

```sh
cubacle solve -s "R U R' U' F2 D L2 B'"
```

```text
moves 8
metrics: htm 8, qtm 10, stm 8
B L2 D' F2 U R U' R'
```

(The first two lines are standard error, the last one standard output.) `moves` prints the same line for the sequence it is given, counted on the words as typed, and `--metrics` on `moves` and `convert` prints all seven counts instead:

```sh
cubacle moves --metrics "R2 L' M x"
cubacle moves --expand quarter "R2 U' F2"
cubacle moves --size 5 --optimize ranges "3Rw R'"
cubacle convert --metrics -s "R U2"
```

```text
R2 L' 2L Rw L'
metrics: htm 4, qtm 5, stm 3, etm 4, obtm 4, btm 3, atm 1
R R U' F F
metrics: htm 5, qtm 5, stm 5
2-3Rw
metrics: obtm 2, qtm 2, stm 2, btm 1
metrics: htm 2, qtm 3, stm 2, etm 2, obtm 2, btm 2, atm 2
FUUFUUFUULLLRRRRRRUBBFFDFFDDDBDDBDDBRRRLLLLLLFFDUBBUBB
```

`moves --expand quarter` writes every half turn as two quarter turns (QTM form; exact, no search) and `moves --optimize ranges` writes block turns as layer ranges, as `solve --optimize ranges` does. They do not combine, and `convert` has no move output to rewrite.

### Known patterns

From the 4x4x4 up, `solve` also tries to write the cube as a product of at most four pattern macros: a checkerboard, six spot, pons asinorum, tetris or cube-in-a-cube algorithm, turned on layers 1 to 3 from each face and seen from any of the 24 orientations. It answers with that product when it has fewer moves than the reduction engine's solution, and otherwise with the engine's solution, unchanged. `-v` prints `known patterns: N moves instead of M` when the product won. The search is exhaustive over those products and finished in under two seconds for a 7x7x7 on a loaded machine (a unit test that builds the macros and searches took 1.6 s); that is added to every solve of a 4x4x4 or bigger. Cubes built from other patterns, or from more than four macros, get the engine's answer. Measured moves, engine alone against with the shortcut:

| Cube | Engine | With known patterns |
| --- | --- | --- |
| 7x7x7, 15-move cube in a cube on the two outer layers (`2Fw 2Lw ...`) | 76 | 15 |
| 7x7x7, the same on single layer 3 (`3F 3L ...`) | 171 | 15 |
| 7x7x7 checkerboard (`R2 3R2 5R2 L2 U2 ...`, 12 moves) | 18 | 12 |
| 5x5x5 checkerboard (`R2 3R2 L2 U2 3U2 D2 F2 3F2 B2`) | 12 | 6 |
| 7x7x7 built from a checkerboard and three six spots | 172 | 38 |
| random 4x4x4 to 7x7x7 (40-move scrambles) | 42, 71, 136, 191 | same |

### Pattern-aware solving

`solve --pattern-aware` also looks the cube up in the pattern library (the built-in patterns and the files that `cubacle.toml` imports, see [Patterns](../patterns.md); `--config FILE` names another config) in every turned and mirrored position. When the cube is a pattern of the library and the library's route has strictly fewer moves than the solver's answer, that route is the answer. The solver still runs, so the answer is never longer, and without the flag nothing changes. Standard error says when the route won; standard output stays the plain solution. The 2x2x2 solver is optimal, so the flag does nothing there; it does not combine with `--moves`, `--optimal`, `--all`, `--count` or `--now`.

```sh
cubacle solve --size 3 -s "U' R2 U' F2 D' R2 U' B F U2 L' B2 R2 F2 D' U F' U2 L' R'"
cubacle solve --pattern-aware --size 3 -s "U' R2 U' F2 D' R2 U' B F U2 L' B2 R2 F2 D' U F' U2 L' R'"
```

```text
moves 21
metrics: htm 21, qtm 30, stm 21
R L F U2 R2 U' D' F2 R' F B U L2 B2 D2 R2 D' L2 D B2 D

moves 20
metrics: htm 20, qtm 28, stm 20
pattern route: cu:superflip Superflip (library route, 1 move shorter)
R L U2 F U' D F2 R2 B2 L U2 F' B' U R2 D F2 U R2 U
```

A 4x4x4 whose centers were cycled (`U' 3Fw F' 3Uw' U 3Fw' F 3Uw`) goes from 12 moves to 8, by the six-spot route (`pattern route: cu:sixspot Six spots (library route, 4 moves shorter)`). The lookup is one hash lookup in a map built once per run from the generators of the library under the 48 symmetries; the built-in library takes about 0.3 ms to index on a 3x3x3 (`-v` prints the time). A route keeps to the alphabet of the solver: face turns on the 3x3x3, outer blocks (`Rw`) from the 4x4x4 up, single inner layers only with `--slices`. With `--to`, the relative state and its inverse are looked up; when the relative state cannot be made for a size, the normal route is used.

### Restricted move sets

`--moves SET` makes the solution use only the moves of a set. A set is a list in any notation the program reads (taken literally: `"R U U2"` has no `R'`), generators in angle brackets (`"<R,U>"` is every turn of R and U), or a name: `htm` (no restriction, the default; the output is the same as without the option), `qtm` (quarter turns of the faces), `half-turns` and `quarter-slices`.

```sh
cubacle solve --moves half-turns -s "R2 U2 F2 D2 L2 B2 U2 R2"
cubacle solve --moves "<R,U>" -s "R U R' U' R U2 R' U R U' R U R' U2 R"
cubacle solve --moves "R U U2" -s "R' U"
```

```text
moves 8
R2 U2 B2 R2 U2 B2 D2 L2
moves 12
R U' R' U R' U R2 U2 R U' R' U2
moves 3
U U2 R
```

On the 3x3x3 the answer is the shortest in the set (for `qtm` it is the default solution written in quarter turns, not proven shortest). A cube the set cannot solve is reported at once, not as a timeout:

```text
error: not solvable with half-turns: a piece sits in a place these moves never bring it to
```

`-m` and `-b` still apply (`error: no solution of at most 3 moves in <R,U> ...`). `--max-memory MB` (default 1500; for `--optimal` see above) bounds the tables. `--moves` cannot be combined with `--improve`, `--parallel`, `--probes` or `--slices`. Measured on 20 random states per set (random 60-move walks inside the group for the first two; random cubes for `htm`): `<R,U>` 0.30 s mean, 0.31 s at most, 15.8 moves on average (13 to 17); `half-turns` 0.22 s, 10.9 moves (10 to 14); `htm` 0.04 s, 20.6 moves (20 to 21).

On a 4x4x4 to 7x7x7 the cube is searched as a product of pattern macros whose moves are all in the set (see [Patterns](../patterns.md)):

```sh
cubacle solve --size 5 --moves half-turns -s "R2 3R2 L2 U2 3U2 D2 F2 3F2 B2"
cubacle solve --size 5 --moves R -s "R2 L2"
```

```text
moves 6
2B2 2F2 2D2 2U2 2L2 2R2
error: the moves "R" cannot be searched: no pattern macro is made only of the allowed moves; the set cannot be searched as pattern macros
```

A cube that is no such product gets `no product of pattern macros made of half-turns solves this cube; that does not mean the cube cannot be solved with them`, never "unsolvable". The 2x2x2 does not take `--moves`.

### Listing solutions

`--all` lists every solution of the shortest length (3x3x3 only; with `--moves`, within the set). Of moves that commute only one order is listed, `--raw` lists every order. `--count N` stops after N solutions. `--depth D` also lists the longer solutions up to D moves, shortest first. Solutions go to standard output as they are found; `moves N` headers and a summary go to standard error.

```sh
cubacle solve --all --raw -s "R L"
cubacle solve --count 3 --depth 8 --moves half-turns -s "R2 U2 F2"
```

```text
moves 2
R' L'
L' R'
solutions 2 (shortest 2 moves; 50 nodes)
moves 3
F2 U2 R2
moves 7
U2 R2 L2 D2 B2 D2 L2
D2 R2 L2 U2 B2 D2 L2
solutions 3 (shortest 3 moves; 241 nodes)
stopped after 3 solutions
```

The search checks the time budget (`-b`) at every node; when it runs out the list so far is kept and the command fails with `the list is incomplete`. Without `--moves` only a small table bounds the search, so lists are practical for short scrambles; a random cube of 20 moves is not. `--depth` and `--raw` need `--all` or `--count`. The REST server does not list solutions yet.

### Failure

`solve` prints a message to standard error and exits with a non-zero status when the cube is impossible, the limit is too tight, or the time budget runs out. For example:

```sh
cubacle solve -s "R U F B L D R U" --max-moves 2
```

```text
error: no solution within the move limit (the search ended without a solution (time budget or move limit reached))
```

Exit status of `cubacle` on a command that fails:

| Status | Meaning |
| --- | --- |
| 0 | Done. |
| 1 | An error: input that is wrong or an impossible cube, a move limit that cannot be met, a face center off its face. |
| 2 | The command line is wrong (the usage error of `clap`, for example `--max-time` with `-b`). |
| 3 | A time limit was reached. Nothing is printed on standard output. |

The 4x4x4 to 7x7x7 have a time limit of 120 s without `--max-time`. Measured on a loaded 12-core machine (cached tables), three random scrambles each took: 3x3x3 25 to 50 ms, 4x4x4 25 to 30 ms, 5x5x5 60 to 140 ms (wall 1 to 4 s with the tables loaded), 6x6x6 1.1 to 5.6 s, 7x7x7 3.8 to 13 s (wall up to 18 s), so the limit is about seven times the slowest. When it is reached:

```sh
cubacle solve --size 7 --max-time 800 --scramble "R U2 Fw' L2 Uw B' D Rw2 F U' 3Bw L' D2 R2 3Uw F2 3Rw B"
echo $?
```

```text
preparing the 4x4x4 and 5x5x5 tables for the wings (the center tables are built on first use)
error: time limit of 0.8 s reached on the 7x7x7 (raise it with --max-time MS)
3
```

The time to read or build the tables is not counted (`prepare`); `--max-time 0` removes the limit.

The 6x6x6 and 7x7x7 solve by reduction; see [Engines and limits](../engines/_index.md).

### Impossible states

Every command that reads a cube state checks first that turns can reach it, for every size from the 2x2x2 to the 7x7x7, and says why not in the same words on every size: a twisted corner, a flipped edge, two swapped pieces (parity), a sticker combination no piece has (this includes corners that are the mirror image of real corners), a wrong center. `solve` (also `--to` and `--now`), `convert` and `patterns find` stop with a non-zero exit status. A hint follows where the repair is certain. A twisted corner on a 3x3x3:

```sh
cubacle solve UUUUUUUURFRRRRRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

```text
error: the cube state is impossible (the corners are invalid: one corner is twisted on its own)
hint: turn one corner back
```

The same state on a 2x2x2 (`UUURFRRRFUFFDDDDLLLLBBBB`) gives exactly this text. Two swapped edges:

```text
error: the cube state is impossible (the cube is invalid: two pieces are swapped (parity error))
hint: swap two edges
```

A cube written as face letters with wrong color counts says which color has too many or too few stickers:

```text
error: not a cube (color R appears 10 times, expected 9) and not a move expression for a 3x3x3 (character 0: invalid move "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBR")
hint: color R has 1 sticker too many, color B has 1 sticker too few
```

`patterns identify` and `patterns cycles` do not fail: patterns on illegally assembled cubes are legitimate. They print a warning on standard error, leave standard output as it is, and go on:

```text
warning: the cube state is impossible (the corners are invalid: one corner is twisted on its own)
hint: turn one corner back
```

`moves --from` and `facelets` take move expressions, so their states are always possible. The REST server answers an impossible start with `422 impossible_cube` and the same text.

### Face centers off their faces

A middle-slice turn (`2R` on a 3x3x3, `3R` on a 5x5x5, `2-6Rw` on a 7x7x7) moves the centers of the faces. The cube is reachable, and `solve` takes the color of each face from its center: it names the colors after the centers, solves that cube, and prints moves that make every face of your cube one color, the color of its center. A note on standard error says so; standard output holds the moves only:

```sh
cubacle solve --scramble "R U 2R2 F 2U D'"
```

```text
note: the face centers are not on their own faces; the solution makes every face the color of its center
moves 6
metrics: htm 6, qtm 8, stm 6
U L' F2 B2 D' F'
```

The cube after these moves is not the facelet string of a solved cube (`U` on the up face, and so on) but every face is one color, which is what a solved cube is. A cube with moved centers whose faces already are one color is `moves 0`: `cubacle solve -s x` on a 3x3x3 is a rotation, nothing to turn. The 5x5x5 pattern `centers-cycled` is solved the same way:

```sh
cubacle solve --size 5 --scramble "2-4Uw 2-4Rw' 2-4Uw' 2-4Rw"
```

```text
note: the face centers are not on their own faces; the solution makes every face the color of its center
moves 8
metrics: obtm 8, qtm 8, stm 8, btm 8
U D' R L' F B' U D'
```

The even sizes (2x2x2, 4x4x4, 6x6x6) have no fixed center piece and need none of this. Two things stay errors. Centers that no turn can reach (two swapped) are an impossible cube. A solve to a target with `--to` still needs the centers on their own faces, with exit status 1 and nothing on standard output:

```sh
cubacle solve --size 7 --scramble "2-6Rw" --to R
```

```text
error: the face centers are not on their own faces (a middle-slice turn moved them); this solver needs the centers of each face in place
hint: turn the cube so the centers match, or undo the slice turn
```

The REST server solves such a cube the same way (the answer leaves every face one color); with a `target` it answers `422 centers_off_face` with the same first sentence.
