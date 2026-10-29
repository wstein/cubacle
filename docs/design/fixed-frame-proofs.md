---
title: "Fixed-frame proofs"
description: "Design: a fixed-frame 2x2x2 mode with sticker identities and display goals, batch JSON Lines, and distance certificates that a checker sharing no code with cubacle can verify."
weight: 45
---

# Fixed-frame proofs

Status: design for the owner's review. Nothing on this page is built. The owner decided the
direction on 2026-10-29 (section 1.3) and the open questions of the first draft on the same day:
they are recorded as DECIDED D1 to D7 in section 14, and the sections that depend on them
follow the decisions. Marks used below:

- **[M]**: computed for this paper by throwaway scripts written from scratch (Python and C,
  not committed; see section 13). They share no code with cubacle or Cubistry.
- **[C]**: read from the code of `develop` at `c460ffa`.
- **[R]**: a number reported by Cubistry Research (branch `research/cube-display-feasibility`,
  commit `8a85fd8`). Every [R] number used for a golden test was reproduced [M].
- **[E]**: estimate, not measured.
- **[A]**: assumption that the implementation or the owner must confirm.

## 1. Problem

### 1.1 What the consumers need

The Cubistry display proofs (H/S/M counter, 00 to 99 counter, timers) use a 2x2x2 with a
fixed print of 24 immutable stickers on eight rigid corners. A *display* is a set of
specific stickers at specific front cells with an upright glyph: for the H/S/M counter, the
letter at upper right, tens at lower left, units at lower right, and a blank sticker or
nothing in particular at upper left. The proofs need:

1. **A fixed frame.** Front stays front. Two states that differ by a whole-cube rotation are
   different states, and a rotation is not a move.
2. **Sticker identity and orientation.** Which physical sticker is at a cell, and where its
   print's up direction points, not only its color.
3. **Legal outer-face turns only**, counted in HTM: 18 moves, each one move.
4. **Shortest paths to any matching display** from an actual full pose, where a display may
   be met by several sticker selections (duplicate digits, the three twists of a blank
   corner).
5. **Distances over the whole graph**: the eccentricity of a target set ("every pose reaches
   this display in at most 6"), also restricted to sources that meet a premise ("with the
   letter in place, at most 5").
6. **Period closure** of counting runs: replaying a run returns to an earlier full state.
7. **Proof data that can be checked.** The Cubistry verifiers rebuild their 9,072- and
   136,080-state graphs themselves, in independent Python. A cubacle answer is useful there
   only as a second, independent producer whose claims a checker can verify.

### 1.2 What cubacle does today [C]

- `PocketSolver` (`src/engines/two/mod.rs`) keeps the DBL corner fixed and turns only U, R and
  F: 7! x 3^6 = 3,674,160 states. A cube is first rotated as a whole until DBL is home. The
  answer is optimal **up to a whole-cube rotation**: a solved cube in any orientation needs no
  moves.
- Targets with open stickers (`src/engines/two/partial.rs`) are tested against all 24
  rotations (`rotation_maps`, `cared_lists`). A target "F face shows U" is met when *some* face
  shows U. This is the documented choice of the don't-care design (section 11 of
  [Don't-care targets](dont-care-targets.md)), and it is the wrong semantics for a display
  proof.
- A target is a map from position to a *set of colors* (`puzzle::Target`). Facelets and
  Orbit64 tokens are colors too ("Orbit64 reads colors so equal-colored pieces need no
  representative"). There is no sticker identity and no glyph orientation on the 2x2.
- The 3x3x3 target sets have identity views internally
  (`src/engines/three/partial/goal.rs`: a 54-bit set of sticker identities per position,
  identity = home position), but they are not exposed and do not apply to the 2x2.
- "Shortest" is a message on standard error (`shortest: proved by a search from the whole goal
  set of N states`). `solve --json` carries no `proven`, no lower bound, no version and no
  `formatVersion` field. There is no batch mode: every call builds the tables again.

The integration assessment of 2026-10-29 states the same gaps. Its claims about the code were
checked against the files named above; its run-time numbers were not repeated.

### 1.3 Owner decisions (final)

1. cubacle gets a fixed-frame 2x2 mode that can supply the proof data of the Cubistry
   display proofs.
2. It comes with a checkable optimality **certificate**: a file that an independent checker
   can verify, not a claim on standard error.
3. Other repositories pin only release tags with green CI. This work ships in a later tag
   (for example v0.2.0) once `develop` CI is green.
4. `--json` may be extended **additively and versioned** (new fields, a batch mode in JSON
   Lines with tables built once). Existing v1 consumers keep working.
5. MIT repositories use cubacle only as an external process in optional tests, never linked
   or embedded.

### 1.4 Goals and non-goals

Goals: exact fixed-frame semantics on the 2x2x2; identity-level display goals; exact distances,
eccentricities and runs; certificates; JSON Lines batch; deterministic, byte-reproducible
output.

Non-goals: no change to `solve`, `solve --to`, Orbit64 or any default output (byte-identical,
as the don't-care design requires); no print or layout search (that stays in Cubistry); no
claim about print optimality, whole-route optimality, intermediate animation frames or timing;
no 3x3x3 or larger mode in this design (section 2.8 is a note only); no checker inside cubacle
that would count as independent.

## 2. Semantics

### 2.1 Frame and moves

The world frame is the one of the Cubistry verifiers and of the proof-checkers contract
(`proof-checkers/SPEC.md` in Cubistry):

- Axes: `x` points to R, `y` to U, `z` to F. The frame is right-handed (`x` cross `y` = `z`).
- A corner position is a vector in {-1, +1}^3. A facelet is a pair (`p`, `n`) of a position and
  an outward axis normal with `p . n = 1`; there are exactly 24.
- A sticker frame is (`p`, `n`, `up`): `up` is a signed axis vector with `n . up = 0`, the
  direction of the top of the printed artwork.
- The 18 moves are the faces U, R, F, D, L, B, each as a quarter turn, a half turn and an
  inverse quarter turn. A quarter turn of the face with outward normal `a` rotates every sticker
  frame with `p . a = 1` by -90 degrees about `a` (clockwise seen from outside the face); the
  half turn is two quarter turns, the inverse quarter turn is +90 degrees. For R, U and F this is
  -90 degrees about +x, +y, +z; for L, D and B it is +90 degrees about +x, +y, +z. This agrees
  with the proof-checkers wording and with the integer Rodrigues turn of
  `research/hsm-counter/verify.py` [C, read in Cubistry Research].
- Metric: HTM, every one of the 18 moves counts 1. No whole-cube rotations, no slice or wide
  moves. Move text is WCA face letters with an optional `2` or `'`, separated by single spaces.
- **No rotation equivalence.** A state is the placement of all 24 sticker frames in this world
  frame. A whole-cube rotation of a state is a different state. Because the 2x2x2 has no
  centers, every whole-cube rotation is also reachable by moves (`y` is `U D'`), and then it
  costs those moves like any other change.
- **Canonical move order** (used for tie-breaking, section 8):
  `U U2 U' R R2 R' F F2 F' D D2 D' L L2 L' B B2 B'`, which is cubacle's face order URFDLB with
  clockwise, half and counter-clockwise turns, as the existing `search_move` orders them [C].

### 2.2 The print

A print is the reference state: 24 stickers, each with

| Field | Meaning |
| --- | --- |
| `id` | Sticker identity, 0 to 23, unique. |
| `cubie` | Body identity, 0 to 7; exactly three stickers per body. |
| `p`, `n`, `up` | The sticker's frame in the reference state. |
| `image` | Optional opaque string. cubacle compares it by exact equality and never interprets it. |

Validation, before anything else, fails closed: unique ids and bodies, the 24 facelets each
covered once, the three stickers of a body share one position and lie on its three facelets,
every `up` is a signed axis vector orthogonal to `n`. The three `up` vectors of a body are
independent of each other (the artwork's orientation is chosen per sticker). These are the
Group 1 rules of the proof-checkers contract, so a Cubistry `printing` array can be passed as
it is; extra fields such as `slot` are ignored but still parsed.

### 2.3 States

A body moves rigidly, so its placement is one of 24 proper rotations of its reference frames
(8 positions times 3 twists). A physical state is the placement of all eight bodies. The states
reachable from the print are 8! x 3^7 = 88,179,840 [M: a breadth-first search over all of them,
section 3]. Every state has a unique full sticker list; the corner twist sum is the only
invariant.

Input forms of a state:

- `{"moves": "U D2 B D'"}`: the word applied to the print.
- `{"stickers": [...]}`: the 24 current frames, same fields as the print (`id`, `p`, `n`, `up`;
  `cubie` and `image` must equal the print's). cubacle checks that every body is a proper
  rotation of its reference (a reflected or independently twisted sticker is refused) and that
  the state is reachable (twist invariant); an unreachable state is an error, never a distance.
- `{"poses": [8 integers]}`: the compact form of section 2.4.

Output always carries `poses`; with `"stickers": true` in the request it also carries the full
wire state in the proof-checkers form: an id-sorted array of
`[id, cubie, [p], [n], [up], image]`.

### 2.4 Pose index and keys (`pose24.v1`, `key.v1`)

These definitions are the contract between cubacle and a checker. They use only the frame and
the print, never cubacle's internal coordinates.

- **Position rank.** The rank of `p` in the lexicographic order of {-1, +1}^3:
  (-1,-1,-1) = 0, (-1,-1,+1) = 1, (-1,+1,-1) = 2, (-1,+1,+1) = 3, (+1,-1,-1) = 4, (+1,-1,+1) = 5,
  (+1,+1,-1) = 6, (+1,+1,+1) = 7.
- **Twist digit.** Each sticker of a body has a *reference axis*: the axis (0 = x, 1 = y,
  2 = z) of its normal in the print. The twist digit of a body is the reference axis of the one
  sticker of the body whose current normal is on the x axis (+x or -x).
- **Pose.** `pose(b) = 3 * positionRank + twistDigit`, 0 to 23. It is a bijection between the 24
  placements of a body and 0..23 [M]. In the reference state body `b` has pose
  `3 * positionRank(reference position of b)`.
- **Key of tracked bodies.** For tracked bodies `b_1 < b_2 < ... < b_k` (ascending body id) with
  positions `q_j` (distinct) and twist digits `t_j`: the position part is the Lehmer rank with
  radices 8, 7, ..., 8 - k + 1, where digit `c_j` counts the positions below `q_j` that
  `q_1 .. q_(j-1)` do not use, most significant first. Then
  `key = positionPart * 3^k + sum(t_j * 3^(k - j))`. The key space has
  `N_k = 8! / (8 - k)! * 3^k` entries.
- **All eight bodies.** For k = 8 the eighth body's pose follows from the other seven in every
  reachable state, so the key is the k = 7 key of the seven lowest tracked body ids and
  `N_8 = N_7 = 88,179,840`. A checker derives the eighth pose either from the twist invariant,
  which it must then verify on its own (for example, that the print and the 18 move images of
  it satisfy it), or by its own breadth-first search.

For k <= 7 every key is a reachable projected state: N_3 = 9,072, N_4 = 136,080 and
N_5 = 1,632,960 were reached by breadth-first search [M], and N_7 = 88,179,840 by the full
search [M]; N_6 = 14,696,640 follows from N_7 (every projection of a reachable state is a key,
and every key of k <= 7 bodies extends to a full state), not run separately.

### 2.5 Displays (goals)

A display is a list of cells. A cell is a facelet (`p`, `n`) and an `accept` list of rules; a
rule names stickers either by `ids` (a list of sticker ids) or by `image` (every sticker of the
print with that exact image string), and optionally an `up` vector.

A state **meets** a display when, for every cell, the sticker currently at that facelet is
named by some rule of the cell, and, if that rule has `up`, the sticker's current `up` equals
it. A rule without `up` accepts every orientation: this is the blank sticker, whose artwork
has no top. The cells must be distinct facelets; a display may constrain any of the 24
facelets, not only the front.

Consequences, all exact and all checked by the predicate, not by special cases:

- *Upright* is a filter, not a degree of freedom: when sticker `s` is at facelet (`p`, `n`), the
  body's placement and therefore `s`'s `up` are fully determined. A rule whose `up` can never
  occur at its cell can never be met; cubacle reports such rules in the goal record
  (`deadRules`) so that a misprinted display is visible, not silently ignored.
- Distinct physical bodies are automatic: one body cannot be at two positions.
- Duplicate glyphs, alternatives and blank corners are union semantics inside one goal set, so
  "shortest to ANY matching display" is one multi-source search, not a minimum over
  enumerated selections.
- An empty display (no cells) is met by every state.

The **tracked bodies** `T(D)` of a display are the bodies of all stickers its rules name. The
goal set is `G(D)`, the reachable states that meet `D`; it depends only on the poses of `T(D)`.

### 2.6 Distances

- `dist(s, D)`: the least number of moves from state `s` to a state that meets `D`; undefined
  when `G(D)` is empty (an *unsatisfiable* display).
- `ecc(D | P)`: the maximum of `dist(s, D)` over all reachable states `s` that meet the premise
  display `P` (the empty premise means all states). This is the "max distance to a target set"
  of the consumers: 6 for the blank print, 5 for the preferred intermediate print under the
  premise "letter in place" (section 10).
- `dist(s, t)` between two full states is `dist(s, D_t)` where `D_t` fixes all 24 facelets to
  the stickers of `t` with their `up`.

**Projection lemma.** Let `pi` map a state to the key of `T(D)`. Moves act on keys (the action
of a move on a body depends only on that body), `pi` commutes with moves, and
`G(D) = pi^-1(pi(G(D)))`. Then `dist(s, D)` equals the distance from `pi(s)` to `pi(G(D))` in
the key graph: a key path lifts to a state path of the same length by applying the same moves
to `s`, and a state path projects to a key path. So a breadth-first search over `N_k` keys gives
exact physical distances, for every hidden pose of the untracked bodies. This is the argument
the Cubistry reviews already use for their 9,072- and 136,080-state graphs; the certificate
makes it explicit and checkable.

### 2.7 Relation to the existing models

| Model | Identity | Frame | Where |
| --- | --- | --- | --- |
| `Facelets`, Orbit64 token | colors only | as given | everywhere; Orbit64 cannot hold identity or `X` |
| `Target` (slot to color set) | colors only | 2x2: any of 24 rotations | don't-care targets, `solve --to` |
| 3x3x3 target-set views | sticker identity = home position, per-position bit set | centers fix the frame; `--any-orientation` adds 24 views | `src/engines/three/partial/` |
| **Fixed-frame display** (new) | print sticker id with frame and `up` | fixed world frame, no rotations | `cubacle fixed-frame` |

The new goal is a separate type (working name `FrameGoal`), not a change of `Target`.
`Target` stays color-only, as decided ("Same-colored inner pieces are equal and there is no
strict mode"). A color target is the special case where each cell accepts all stickers of one
image with no `up`; cubacle does not convert between the two models. The identity rule is the
one of the 3x3x3 views (a sticker is named by where it sits in a reference state), with the print
in place of the solved cube and an orientation filter added. The don't-care design's open
question "a certificate (the proof as a replayable record)" for slice 2 is answered here for the
2x2x2.

Default behavior is untouched: `solve --size 2` keeps the rotation quotient and its bytes. A
later, separate option `solve --size 2 --fixed-frame` (fixed frame, color cube, solved target)
is possible but is not needed by the consumers and is not part of this plan.

### 2.8 Beyond the 2x2x2 (note only)

The ideas generalize, the sizes do not. On the 3x3x3 the centers fix the frame, so "fixed frame"
means center orientation (Z4 per center, already listed as slice 2 of the target sets) and
picture stickers on edges and corners. Exact multi-source tables exist only for small
projections (the cross: 190,080 keys); a full distance table is impossible. A lower-bound
certificate there would be a *consistent potential*: a projection table `h` with `h = 0` on the
projected goals and `h(x) <= h(x m) + 1` on its own graph, which a checker can verify locally, plus
a replayable path; when `h(s)` is below the path length, the gap needs a search record that is
too large to be a practical certificate. That stays out of scope.

## 3. State spaces and sizes

| Tracked bodies k | Keys N_k | Table (1 byte per key) | Example use |
| --- | ---: | ---: | --- |
| 3 | 9,072 [M] | 9 KB | H/S/M with free upper left (three cells) |
| 4 | 136,080 [M] | 133 KB | blank H/S/M, 00 to 99 counter, timers (four cells) |
| 5 | 1,632,960 [M] | 1.6 MB | a display whose duplicate glyphs touch five bodies |
| 6 | 14,696,640 | 14 MB | six bodies |
| 7 or 8 | 88,179,840 [M] | 84 MB | full state; state-to-state distance |

Measured histograms [M] (distance: number of keys):

- k = 3, one target (letter upper right, tens lower left, units lower right, all upright on F):
  0: 1, 1: 15, 2: 177, 3: 1,397, 4: 4,981, 5: 2,497, 6: 4. Maximum 6. Equal to the [R] histogram
  of `research/hsm-counter/REVIEW.md`.
- k = 3, sources with the letter already in place (378 keys): 0: 1, 1: 6, 2: 29, 3: 118,
  4: 180, 5: 44. Maximum 5. Equal to [R] of `periodic/REVIEW.md`.
- k = 4, blank corner upper left in any of its three twists plus three upright glyphs (3 goal
  keys): 0: 3, 1: 45, 2: 594, 3: 6,015, 4: 39,849, 5: 79,143, 6: 10,431. Maximum 6, equal to [R]
  of `blank/REVIEW.md`. Restricted to the 810 sources with blank and letter in place: 0: 3, 1: 9,
  2: 27, 3: 99, 4: 273, 5: 363, 6: 36, equal to [R].
- k = 4, one fully fixed four-cell target: 0: 1, 1: 15, 2: 198, 3: 2,030, 4: 15,396, 5: 61,768,
  6: 53,528, 7: 3,144. Maximum 7. (Not reported by Cubistry; a new golden value.)
- k = 5, one target: 0: 1, 1: 18, 2: 243, 3: 2,874, 4: 27,380, 5: 178,562, 6: 645,104,
  7: 717,990, 8: 60,788. Maximum 8.
- Full fixed frame (k = 7 key, from the reference state): 0: 1, 1: 18, 2: 243, 3: 2,874,
  4: 28,000, 5: 205,416, 6: 1,168,516, 7: 5,402,628, 8: 20,776,176, 9: 45,391,616,
  10: 15,154,409, 11: 49,943. All 88,179,840 keys reached; maximum 11, the same diameter as
  the rotation quotient. Table SHA-256 for a print whose body `b` sits at position rank `b` (true
  of the Cubistry H/S/M prints [M]):
  `caf69d83df6fd4c42a899cfd55c664bca87da01408a5aa4973c4eee6633825d9`.

## 4. Command line

One new subcommand; no existing command changes. The name `fixed-frame` and its three
subcommands `batch`, `distance` and `table` are decided (D1, section 14).

```text
cubacle fixed-frame batch    [--tables DIR] [--max-memory MB] [--certificate] < requests.jsonl > records.jsonl
cubacle fixed-frame distance --print PRINT.json --from STATE --display DISPLAY.json [--json]
cubacle fixed-frame table    --print PRINT.json --display DISPLAY.json [--premise DISPLAY.json] --out TABLE.bin
```

- `batch` is the real interface (section 5). It reads JSON Lines on standard input and writes
  JSON Lines on standard output, one record per line, in request order. Tables are built once
  per process: per-body move tables at start, goal tables on first use, cached by the canonical
  goal key within `--max-memory` (default 1500 MB, the project cap; the least recently used table
  is dropped first). `--tables DIR` writes the sidecar table of every goal (section 7.4) into
  `DIR` and names it in the goal record. `--certificate` adds the header and summary records
  (section 7) and refuses requests whose answers would not be proven.
- `distance` is one query for people: it prints `6` on standard error and the lex-least
  shortest word on standard output, as `solve` does; `--json` prints the one `distance` record
  instead. `--from` takes a move word, or `@FILE` for a state JSON.
- `table` writes the sidecar for one goal (and premise) and prints the goal record.
- Errors: text on standard error and exit status 1 for invalid input (print, state, display,
  JSON), 2 for an unsupported format version or puzzle, 3 for internal or I/O errors. In
  `batch` a bad request produces an `error` record and the run goes on, unless `--certificate`
  is set, which stops at the first error with exit status 1, so a certificate is never partial.
- Threads: one by default. Optional parallel goal builds must not change a byte of output.

## 5. Batch JSON Lines

Every line is one JSON object, UTF-8, no byte order mark, ended by `\n`. The schemas are
[fixed-frame-request.schema.json](../static/schema/v1/fixed-frame-request.schema.json) and
[fixed-frame-record.schema.json](../static/schema/v1/fixed-frame-record.schema.json) (draft,
schema family v1). Requests carry `"formatVersion": 1` on the first line only (the `print`
request); records carry `format` and `formatVersion` in the header.

### 5.1 Requests

| `op` | Fields | Answer |
| --- | --- | --- |
| `print` | `formatVersion`, `puzzle` (`"2x2x2"`), `printing` (section 2.2) | `header` record; must be the first line; a later `print` line starts a new context |
| `goal` | `goalId`, `display`, optional `premise` | `goal` record (counts, histogram, eccentricity, table hash) |
| `distance` | `id`, `from` (a state), `goalId` or `display`, optional `stickers` | `distance` record |
| `eccentricity` | `id`, `goalId`, `premise` (a display), optional `witness` | `eccentricity` record |
| `run` | `id`, `from`, `goals` (list of goal ids, cyclic), `maxSteps` | `run` record (section 7.7) |
| `sample` | `id`, `count`, `seed`, `goalId` | `count` `distance` records from seeded random states (section 8) |

### 5.2 Example

The blank H/S/M print (`research/hsm-counter/blank/witness.json`, SHA-256 `13eea2a1ad88561775158a70633b80e25d06f883e42d44aafb4e6808699be47b`),
source `U D2 B D'` (display blank/H/00), target blank/H/34. Values [M] by the scratch
implementation of this spec; the implementation PR must reproduce them byte for byte.

Requests (the `printing` array is shortened here):

```json
{"op":"print","formatVersion":1,"puzzle":"2x2x2","printing":[{"id":0,"cubie":0,"p":[-1,-1,-1],"n":[-1,0,0],"up":[0,1,0],"image":"0"}]}
{"op":"goal","goalId":"blank-H-34","display":{"cells":[{"p":[-1,1,1],"n":[0,0,1],"accept":[{"image":""}]},{"p":[1,1,1],"n":[0,0,1],"accept":[{"image":"H","up":[0,1,0]}]},{"p":[-1,-1,1],"n":[0,0,1],"accept":[{"image":"3","up":[0,1,0]}]},{"p":[1,-1,1],"n":[0,0,1],"accept":[{"image":"4","up":[0,1,0]}]}]}}
{"op":"distance","id":"cex","from":{"moves":"U D2 B D'"},"goalId":"blank-H-34"}
```

Records (the header is shortened):

```json
{"record":"header","format":"cubacle.fixed-frame","formatVersion":1,"toolVersion":"0.2.0","gitCommit":null,"puzzle":"2x2x2","frame":"x=R,y=U,z=F;quarter=-90@outward","metric":"htm","moveOrder":"U U2 U' R R2 R' F F2 F' D D2 D' L L2 L' B B2 B'","poseIndex":"pose24.v1","key":"key.v1","tableEncoding":"u8-dense-v1","printSha256":"..."}
{"record":"goal","goalId":"blank-H-34","trackedBodies":[1,3,6,7],"keys":136080,"goalKeys":3,"deadRules":[],"histogram":[3,45,594,6015,39849,79143,10431],"eccentricity":6,"tableSha256":"0a3f73e6d95731fe778d47d5a8bdd092d925a3514f2936f60557899d38d81c1c"}
{"record":"distance","id":"cex","goalId":"blank-H-34","from":{"poses":[5,19,7,12,2,17,23,11]},"key":114083,"distance":6,"proven":true,"path":"U B R2 B2 U' L","pathRule":"lex-least-shortest","reached":{"poses":[2,4,14,17,18,8,23,11]}}
```

The Cubistry review gives `R F' D F2 R' D` for the same pair; both words have six moves. The
canonical tie rule (section 8) picks `U B R2 B2 U' L`. A consumer compares lengths, or replays
either word; it never needs cubacle's word to equal its own.

### 5.3 Byte rules

Compact JSON, keys in the order of the schema's `properties`, no insignificant whitespace,
integers without fraction or exponent, strings escaped as in the proof-checkers wire rule
(ASCII output; `\uXXXX` in lowercase for control and non-ASCII code points). No time values in
records; `-v` prints timings on standard error only. Input is read strictly: duplicate keys,
NaN, numbers with a fraction or exponent where an integer is expected, unknown `op` or
`formatVersion` are errors (the bounds of the proof-checkers contract apply: 16 MiB per line,
depth 64, 128 bytes per numeric token). The reader is `serde_json` plus a duplicate-key check
(D2): the check is a `Visitor` that remembers the keys of every object, so a duplicate is an
error at any depth; unknown fields are errors in every object that the schema closes with
`additionalProperties: false`, and are parsed and ignored only where the schema leaves the object
open (a sticker, so that a Cubistry `slot` passes). What the dependency decision means is in
D2.

## 6. Versioning and the additive `solve --json` change

- **New formats** carry `format` (`cubacle.fixed-frame`) and `formatVersion` (1). They belong to
  schema family v1 and use `common.schema.json` for `formatVersion`. A later incompatible change
  is format version 2 with a new schema file; cubacle keeps reading version 1 requests for at
  least one release after that.
- **`solve --json` today** has no version field and no schema file. Its contract is "keys are
  appended, never reordered" (`docs/cli/solve-options.md`, `src/report/mod.rs`) [C].
- **Additive change, opt-in.** `solve --json-format 2` on the command line, and
  `"formatVersion": 2` in a `POST /v1/solve` request, append these keys after the existing ones:

  | Key | Type | Meaning |
  | --- | --- | --- |
  | `formatVersion` | 2 | Marks the extended body. |
  | `toolVersion` | string | The crate version, `0.2.0`. |
  | `gitCommit` | string or null | Set by the release build from the environment; null otherwise. No build script is added. |
  | `frame` | `"any-orientation"` or `"fixed"` | The 2x2x2 `solve` is `"any-orientation"`; fixed-frame records are `"fixed"`. Makes the rotation quotient visible to consumers. |
  | `proven` | Boolean | True only where the engine proves the answer shortest in HTM: 2x2x2 tables, `--optimal`, `--within`, exact partial routes; false for two-phase and reduction engines. |
  | `lowerBound` | integer or null | The proven lower bound: equal to `length` when `proven`; the depth searched out when a limit stopped an optimal search; null when nothing is proven. |

  Without the option the body is byte-identical to today, so every v1 consumer and every
  recorded example keeps working. A v1 reader that ignores unknown keys can also read a v2
  body, since all v1 keys keep their place and meaning. Making version 2 the default is a later
  owner decision (section 14).
- **Schema for the existing body.** The implementation adds `solve-result.schema.json` (v1)
  describing today's body, with the v2 keys as optional, so the contract has a file.
- **Pinning.** Consumers pin a release tag and the SHA-256 of the release tarball, record
  `toolVersion` from the header, and refuse an unknown `formatVersion`.

## 7. Certificate

### 7.1 Trust model

The checker trusts only itself: its JSON reader, its own integer move engine on sticker frames,
its own implementation of `pose24.v1` and `key.v1`, and SHA-256. It does **not** trust cubacle's
moves, tables, hashes, paths or counts. A certificate is accepted only if every claim in it is
verified; a check it cannot run is a failure, not a skip. A checker must share no code with
cubacle (no port, no generated tables, no linked library); it may read the same specification.

### 7.2 What a certificate is

A certificate is the batch output with `--certificate`: a JSON Lines file with a `header`
record, then the answer records, then a `summary` record, plus optional sidecar tables. The
header contains the complete print (not only its hash), the frame string, the move order and
the encodings, so the file states the whole problem on its own. The summary holds the record
count and `chainSha256`, the SHA-256 of all preceding lines as bytes (each with its `\n`).

| Claim | Evidence in the certificate | What the checker does |
| --- | --- | --- |
| `dist(s, D) <= d` | `path` of length `d` | Replays it on the full 24-sticker state of `s` with its own engine; checks the end meets `D` and equals `reached`. |
| `dist(s, D) >= d` | goal table of `D` (by hash or sidecar) and `key` | Verifies the table (7.4), computes the key of `s` itself, checks `table[key] == d`. |
| `ecc(D \| P) = e` | goal table, premise `P`, `witness` key and path | Verifies the table; maximum of the table over keys meeting `P` equals `e`; the witness meets `P` and has value `e`. |
| `G(D)` empty | `goalKeys: 0` | Evaluates the predicate on every key of `T(D)` itself. |
| Run closes with preperiod `a`, period `b` | the step words | Replays full states; state after step `a + b` equals state after step `a` with the same goal index; each step's length checked as above. |
| Histogram, counts | the numbers | Recomputed from the verified table. |

### 7.3 Upper bounds

A path is a move word in the canonical spelling. The checker parses it with its own parser
(exactly the 18 tokens, single spaces), applies it to the full sticker state, and checks the
display predicate on the result. This uses all 24 stickers, so hidden bodies are checked too.

### 7.4 Lower bounds: goal tables

The goal table of `D` is the array `t[0 .. N_k)` over the keys of `T(D)`, with
`t[key] = dist(key, pi(G(D)))`. Encoding `u8-dense-v1`: one byte per key in key order, values
0 to 254, no header; the value 255 does not occur in version 1 (an unsatisfiable display has no
table). `tableSha256` is the SHA-256 of these bytes. The hash is a function of the problem
only, so any correct implementation produces the same bytes.

A checker verifies a table in one of two modes; both are complete proofs.

**Mode R, recompute.** The checker runs its own multi-source breadth-first search over the keys
of `T(D)` from the keys that meet `D`, encodes the result as `u8-dense-v1` and compares the
SHA-256 with `tableSha256`, and the histogram. No sidecar is needed. This is what the Cubistry
verifiers do today; the certificate pins the exact claim.

**Mode L, local conditions.** The checker reads the sidecar, checks its length `N_k` and hash,
and then checks, for every key `x` and every move `m` (with `x m` computed by its own engine):

- L1: `t[x] = 0` if and only if `x` meets `D` (the predicate evaluated by the checker);
- L2: `t[x m] >= t[x] - 1`;
- L3: if `t[x] > 0`, some move `m` has `t[x m] = t[x] - 1`.

Theorem: L1 to L3 hold exactly when `t` is the true distance. Proof: by L1 and L2, along a
shortest path `x = x_0, ..., x_d` in `G` the values drop by at most 1 per step and end at 0, so
`t[x] <= d`. By L3, a descent from `x` reaches value 0 in `t[x]` steps, which is a goal by L1,
so `d <= t[x]`. The move set is closed under inverses, so L2 over all moves covers both
directions of every edge. Mode L needs no queue and no search; it is a linear scan with 18
lookups per key and can be split over threads or machines. Its cost is that of the search, so it
is offered for independence of method, not for speed.

Sidecar files are named `goal-<tableSha256>.u8` and are referenced by the goal record's
`tableSha256`; the file name carries no meaning beyond that.

### 7.5 Eccentricity and universal bounds

The consumers' universal claims become sets of eccentricity records. For example "after setup
of the letter, every same-letter transition takes at most five" on the preferred intermediate
print is, per target display `D` (180 of them), `ecc(D | P_letter) <= 5`, where the premise
`P_letter` puts the letter upright upper right. The induction step of the Cubistry proof
(every target meets the premise again) is a check of `G(D)` against `P`: the checker verifies
that every goal key also meets `P` (on the union of tracked bodies). cubacle reports it as
`premiseClosed: true`; the checker recomputes it.

### 7.6 What is not certified

Print optimality, global route optimality, the counting policy of the Cubistry generators,
intermediate frames of an animation, timing, and anything about other puzzles. A certificate
covers exactly its records.

### 7.7 Runs and period closure

A `run` request gives a start state and a cyclic list of goal ids. cubacle steps with the
lex-least shortest word to the next goal, until the pair (full state, goal index) repeats or
`maxSteps` is reached. The record lists every step (`goalId`, `distance`, `path`) and the
`preperiod` and `period`, or `closed: false`. cubacle's policy is its tie rule, not Cubistry's
policy 13 or 1, so its runs are not expected to equal Cubistry's runs; the universal bounds of
section 7.5 hold for every shortest policy, which is what the golden tests use.

### 7.8 The independent checker

The checker lives outside cubacle: in Cubistry Research (Python, standard library) and,
optionally, in the Cubistry proof-checkers (Flix, Julia), under their own licenses and their own
review. Contract:

1. Read the certificate under the strict JSON rules of section 5.3; reject an unknown `format`
   or `formatVersion`.
2. Validate the print (section 2.2) and recompute `printSha256` over the canonical print bytes.
3. Build its own move engine on sticker frames and its own `pose24.v1` and `key.v1`; self-test
   them: the 18 moves are permutations of the 24 facelets, `X X'` is the identity, a quarter
   turn has order 4, and the reference state has the poses `3 * positionRank`.
4. For each goal record: tracked bodies, `goalKeys`, `deadRules` and the table by mode R or L;
   then histogram and eccentricity.
5. For each distance, eccentricity and run record: the checks of section 7.2.
6. Recompute `chainSha256` and the record count.
7. Report `VALID` or `INVALID` with the first failing record, exit 0 or 1, as the proof-checkers
   interface does.

Expected effort [E]: about 400 lines of Python. Mode R for k <= 4 runs in seconds in CPython
(the 136,080-key searches of this paper took about 20 s each on a loaded shared machine [M]);
k = 5 took about 90 s [M]; k >= 6 and mode L on large tables need a compiled checker (Julia,
or the JVM for Flix).

## 8. Determinism and reproducibility

- Same `toolVersion`, same request bytes: the same output bytes, on every platform and thread
  count. A test runs the batch twice and with two thread counts and compares bytes.
- Paths: the **lex-least shortest word** in the canonical move order. It is found by greedy
  descent on the goal table, choosing at each step the first move in the order that lowers the
  value by one; every such move extends to a shortest word, so the greedy word is the
  lexicographically least one. The existing `walk` of the 2x2x2 engine already works this way
  [C]. A checker can reproduce the exact word, not only its length.
- Tables: a function of the problem, hashed in a defined encoding (7.4).
- Sampling: `sample` uses SplitMix64 seeded with `seed` (an unsigned 64-bit integer from the
  request), and draws a state as a uniformly random key of 8 bodies: a Lehmer digit for each
  position below 8 - j by `next() % (8 - j)`, then seven twist digits by `next() % 3`, the eighth
  derived. The modulo bias is accepted and documented: the draws are test inputs, not
  statistics. The seed is echoed in each record.
- No clock, no environment, no locale in any record. `toolVersion` and `gitCommit` are in the
  header only.
- The header carries `printSha256`: SHA-256 of the print re-serialized by the rules of 5.3 with
  the stickers in id order and only the fields of section 2.2.

## 9. Size and run-time budget

| Work | Estimate or measurement |
| --- | --- |
| Per-body move tables (8 x 24 x 18 bytes) | negligible |
| One goal table, k <= 4 | well under 10 ms [E] in release; the existing 3.67-million-state table builds in 64 ms [C, docs/engines/2x2.md] |
| k = 5 | about 30 ms [E] |
| k = 7 or 8 (full) | 1 to 4 s and 84 MB [E], scaled from the 64 ms build with 24 times the states and twice the moves; the C scratch program took 5 min 20 s of CPU on a heavily loaded two-core machine without any tuning [M] |
| A batch of the 180 H/S/M displays plus their eccentricities | under 2 s [E] |
| Memory | the tables in the cache plus one frontier; a full search keeps the layer in the table (scan per depth) so the peak stays near 90 MB [E] |
| Certificate size, 180 goals of k = 4 with sidecars | 180 x 133 KB = 24 MB of sidecars [E]; without sidecars (mode R) a few hundred KB of JSON Lines |

## 10. Test plan

### 10.1 Golden cases (from the Cubistry proofs, reproduced [M])

| Case | Expected |
| --- | --- |
| k = 3 single target, histogram | 1, 15, 177, 1397, 4981, 2497, 4; key count 9,072 |
| k = 3 letter-in-place premise | 378 sources; 1, 6, 29, 118, 180, 44; eccentricity 5 (preferred intermediate print, `comparison/candidates/intermediate.json`, SHA-256 `3dbaf98e8acd7ab8abb21a09bab1779a57edf14284d3287a69d669985e6bb933`) |
| k = 4 blank corner, three twists | 136,080 keys, 3 goal keys; maximum 6 (blank print, SHA-256 `13eea2a1ad88561775158a70633b80e25d06f883e42d44aafb4e6808699be47b`) |
| k = 4 blank premise | 810 sources; 3, 9, 27, 99, 273, 363, 36 |
| Blank counterexample | from `U D2 B D'` (blank/H/00) to blank/H/34: distance 6; key 114083 of bodies 1, 3, 6, 7; lex-least word `U B R2 B2 U' L`; Cubistry's `R F' D F2 R' D` also replays to the display; table SHA-256 `0a3f73e6d95731fe778d47d5a8bdd092d925a3514f2936f60557899d38d81c1c` |
| Full fixed frame from the reference | the histogram of section 3, diameter 11, SHA-256 `caf69d83df6fd4c42a899cfd55c664bca87da01408a5aa4973c4eee6633825d9` |
| HSM witness (`research/hsm-counter/witness.json`, SHA-256 `7ec93660867a367ad79d7af28acb2b0f1035450bcac76d0c1afaa7bcdac93cd5`) | for every saved forward step, setup and reset, `dist(source, display)` equals the saved word length (the Cubistry checker's claim [R], not repeated for this paper; cubacle must reproduce it) |

The print files are AGPL data of Cubistry, the same license as cubacle; they are copied into
`tests/fixtures/fixed_frame/` with their source commit and hash in `NOTICE.md`.

### 10.2 Properties

- Every returned path replays (in cubacle's own facelet model and in the frame model) to a
  state that meets the display.
- `dist` never exceeds the length of any path found by a bounded brute force from the full
  state (depth <= 5 on random sources).
- Fixed frame versus quotient: for the goal "the print's reference state", the fixed-frame
  distance is at least the `PocketSolver` length of the same cube as colors (the quotient
  accepts every rotation of the goal). On the 2x2x2 a whole-cube rotation is reachable by moves:
  `U D'` is a `y` rotation, so for it the quotient says 0 and the fixed frame says 2. That case
  is a test.
- A display and the same display with its cells reordered give identical records.
- Adding an accepted sticker to a cell never increases a distance; removing a cell never
  increases it.

### 10.3 Negative controls

Each must fail with its own message: a print with a duplicate id, a missing facelet, a body
split over two positions, `up` parallel to `n`; a state with a reflected body, a single twisted
corner (unreachable), a sticker moved off its body; a display with two cells on one facelet, a
rule naming an unknown id or image, an `up` that is not a signed axis; an unsatisfiable display
(reports `goalKeys: 0`, never a distance); a request with `formatVersion: 3`, a duplicate key,
`1.0` for an integer. For the checker side, the implementation PR ships corrupted certificates
(a path one move too long, a table byte changed, a histogram off by one, a wrong `key`, a
swapped header frame string) as fixtures that the independent checker must reject; cubacle
itself does not judge them.

### 10.4 Where the tests run

- Default `cargo test`: k <= 4 cases, the counterexample, negative controls, the determinism
  byte test. Budget: under one second of test time in release [E].
- `#[ignore]`, filter `fixed_frame_full`: the full 88-million-key table and its hash, k = 5 and
  k = 6 goldens, the 180-display H/S/M batch. Run on request, or in an opt-in CI job like
  `four-full`, never in the default `rust` job, which already times out on `develop`.
- The independent checker runs in Cubistry, in optional tests that download the pinned release,
  never in cubacle's CI.

## 11. Performance, memory and CI

The default suite gains only the small cases. No table is cached on disk (tables of k <= 7
build in seconds, so the cost of a cache and its integrity hash is not worth it). The 1.5 GB cap
holds with a wide margin: the largest table is 84 MB. Nothing is added to the server in this
plan; a REST endpoint would need a decision on stateless table builds per request (the don't-care
design's open question 6).

## 12. Licensing

cubacle is AGPL-3.0-or-later. Consumers under MIT run the released binary as a separate program
and exchange JSON Lines through pipes or files; they neither link nor embed nor copy cubacle
code, and the tests that do so are optional. That is the owner's decision 5. The certificates
and tables are program output computed from the consumer's own input; this paper assumes [A]
that they carry no AGPL obligation for the consumer, which is the usual reading for program
output but is not legal advice and is the owner's call. The Flix and Julia checkers are written
independently from this specification and carry their own license. Cubistry's print files used as
test fixtures are AGPL like cubacle; their provenance goes into `NOTICE.md`. No dependency
changes license terms: `serde_json` and `serde` (both MIT or Apache-2.0) join the root crate and
`sha2` (MIT or Apache-2.0) moves from a dev-dependency to a dependency (D2).

## 13. Verified versus assumed

Verified for this paper:

- [M] Pose bijection, key counts N_3 = 9,072, N_4 = 136,080, N_5 = 1,632,960 by search, and
  N_7 = 8! x 3^7 = 88,179,840 by a full search; the histograms of section 3.
- [M] All [R] numbers used as goldens: the k = 3 and k = 4 histograms, 378 and 810 premise
  sources, maxima 6, 5 and 6.
- [M] The blank-print counterexample (distance 6, both words replay to blank/H/34) on the real
  print file, and its key, poses, lex-least word and table hash.
- [M] The SHA-256 of the three Cubistry witness files equals the hashes in their reviews.
- [C] The rotation quotient of `PocketSolver` and of the partial targets, the color-only
  `Target`, the identity views of the 3x3x3, the JSON writer without a version field, `sha2` as a
  dev-dependency only, no JSON reader in the library crate (only `crates/server` uses
  `serde_json`), and that the tests read `docs/static/examples/definitions` but not the schema
  files.
- [M] `serde_json` 1.0.154 with a duplicate-key visitor in a scratch crate (offline build): the
  rejections listed in D2.

The scratch scripts: a Python projection search over sticker frames with integer quarter-turn
rotations (about 150 lines), a C full search over the k = 7 key with move tables generated by the
Python model, and a script that computes the example values with the real print. They were run
once and are not part of the repository; the implementation PRs turn their numbers into tests.

Assumed:

- [A] The Cubistry frame convention is the one the consumers will keep (checked in the H/S/M
  verifier and the proof-checkers wording; a future change there is a new frame string).
- [E] All Rust run times and memory figures in section 9.
- [A] Cubistry's displays need at most 24 cells and only the predicates of section 2.5 (exact
  sticker, optional exact `up`); "any rotation of the glyph by 180 degrees" would be two rules,
  which works, but is untested.

## 14. Decisions of the owner

The open questions of the first draft were decided on 2026-10-29. Each decision is recorded as
DECIDED with its reasoning in one or two sentences, and the sections named in it follow it. The
numbers D1 to D7 are used instead of the numbers of section 1.3, which are the decisions that
started this work.

### D1. Subcommand name (DECIDED)

The command is `cubacle fixed-frame` with `batch | distance | table`, as drafted in section 4.
The name says what is new (a frame that does not move), `frame` alone says too little, and a
flag on `solve` would put two frame models into one command, which is the first risk of
section 16.

### D2. Strict JSON reading (DECIDED)

Input is read with `serde_json` plus a duplicate-key check: duplicate keys and unknown fields are
rejected where the schema says so (section 5.3). The check is a few dozen lines on top of a
well-tested parser, against about 300 lines [E] of a reader of our own that would need its own
negative controls.

What this means for dependencies:

- **Crates.** `serde_json` and `serde` (with `default-features = false` and the `std` feature;
  `serde` is needed for the `Visitor` trait) are added to `[dependencies]` of the root crate, and
  `sha2` 0.11.1 moves from `[dev-dependencies]` to `[dependencies]` (it is used today only by
  `tests/synthetic_test.rs` [C]). All three are already in `Cargo.lock` (`serde_json` 1.0.154
  through `crates/server`), so the lock file gains no new package, only new edges. All three are
  MIT or Apache-2.0. No `preserve_order`, `arbitrary_precision` or `float_roundtrip` feature.
- **Rules found [C].** `AGENTS.md` says to keep dependencies on their latest releases and that
  `rust-version` (1.95) is what the `msrv` job measures, so the `msrv` job must stay green with
  the new edges. There is no `deny.toml`, no `cargo-deny` or `cargo-audit` job, and no sentence
  in `AGENTS.md` that requires an approval for a dependency; Dependabot proposes updates weekly
  in one group, and `CODEOWNERS` asks `@wstein` to review everything. The repository does record
  the reasons for a dependency in a decision note (for example the `toml` crate), and the
  PuzlWerk section of `AGENTS.md` says that a name or scope does not authorize a dependency.
- **Approval needed.** D2 is the owner's approval for exactly these edges and no others. The pull
  request that adds them (row 4a of section 15) carries a ChartScribe decision note in `notes/`
  and is merged only after the owner's review; any other crate (a faster hasher, a parallel
  library) needs its own approval.
- **Strictness actually provided [M].** A scratch crate (serde_json 1.0.154 with a duplicate-key
  `Visitor`, built offline) rejected `{"a":1,"a":2}`, `1.0`, `1e0`, `NaN`, a lone surrogate
  `"\ud800"`, trailing characters after the value and nesting deeper than 128, and it hands
  integers beyond `i64` to the visitor, which can then range-check them. What `serde_json` does not
  give is a depth bound of 64 and a 128-byte numeric-token bound: they are a counter in the
  visitor and a bounded line reader (16 MiB) of our own, covered by the negative controls of
  section 10.3, which reuse the names of the strict-only cases of the Cubistry corpus.
- **Placement.** The reader lives in the application layer (`src/fixed_frame_cmd/`), the server
  uses the same function, and the engines never see JSON.

### Still open from the first draft

3. **`solve --json` version 2 by default**, and when; and the key spelling: camelCase
   (`lowerBound`, `toolVersion`, as the existing keys) is proposed instead of `lower_bound`.
4. **Sidecar tables in releases.** Should a release also publish the full-frame table and its
   hash as a reference artifact (84 MB), or only the hash?
5. **Who owns the independent checker**: Cubistry Research (Python) first, the proof-checkers
   (Flix, Julia) later, or both from the start.
6. **Server.** Is a REST endpoint for fixed-frame queries wanted at all?
7. **Release.** Is v0.2.0 the tag for this, and does it wait for the green `rust` job only, or
   for all jobs?

## 15. Implementation plan

Small pull requests against `develop`, each Draft until reviewed; sizes in changed lines
including tests, [E].

| # | Pull request | Size | Depends on |
| --- | --- | ---: | --- |
| 1 | `docs(design)`: this paper and the two draft schemas | docs only | none |
| 2 | `feat(puzzle)`: frame model: print validation, sticker-frame moves, `pose24.v1`, `key.v1`, state input forms; golden poses of the blank counterexample | 450 | 1 |
| 3 | `feat(two)`: fixed-frame goal tables: `FrameGoal`, tracked bodies, multi-source search over keys, lex-least walk, `u8-dense-v1` with SHA-256 (`sha2` to dependencies); k = 3 and k = 4 goldens | 500 | 2 |
| 4 | `feat(cli)`: `fixed-frame batch`, `distance`, `table`; strict JSON reader (D2); records, header, summary; determinism test | 700 | 3 |
| 5 | `feat(two)`: eccentricity with premises, `premiseClosed`, runs, `sample`; sidecars; ignored `fixed_frame_full` tests | 450 | 4 |
| 6 | `feat(report)`: `solve --json-format 2` and REST `formatVersion: 2`; `solve-result.schema.json` | 300 | none (parallel) |
| 7 | `docs`: CLI page `docs/cli/fixed-frame.md` from measured behavior, engine page, notes, NOTICE for fixtures | docs | 4, 5 |
| 8 | Outside cubacle: the Python checker in Cubistry Research, with the corrupted certificates of 10.3 | 400 | 4 |

Files stay under the 500-line planning mark: the frame model, the goal tables, the batch reader
and the record writer are separate modules (`src/puzzle/frame/`, `src/engines/two/fixed/`,
`src/fixed_frame_cmd/`). Engines do not use each other's internals; the fixed-frame tables use
nothing of `PocketSolver` but may share its `Turn` and move types. The tag is cut by the owner
after the CI of `develop` is green (decision 3).

## 16. Risks

- **Two frames in one tool.** A consumer may call `solve --size 2` and take a rotation-quotient
  length for a fixed-frame distance, which is shorter or puts the display on another face. Mitigation:
  separate subcommand, `frame` key in `solve --json` version 2, a sentence on the 2x2 engine page.
- **Convention drift** between cubacle and the consumers (axis signs, turn direction, twist
  digit). Mitigation: the frame string in the header, the checker's self-tests, goldens with real
  prints, and the counterexample replayed by both sides.
- **False independence.** A checker that reuses cubacle code or tables proves nothing. Mitigation:
  the contract forbids it; the checker lives in another repository with its own review.
- **Strict JSON** rests on `serde_json` plus our duplicate-key check (D2). A gap in the check
  would let a duplicate key through; the negative controls of section 10.3 cover it, and the
  independent checkers read the same bytes with their own parsers.
- **CI.** The `rust` job times out on `develop` now; the new default tests must stay small, and
  the heavy ones stay ignored. A release waits for green CI, so this work can be blocked by an
  unrelated job.
- **Large tables in a checker.** Mode L on 88 million keys is slow in CPython; the plan keeps
  Python to k <= 5 and leaves the full table to a compiled checker or to mode R.
- **Scope creep** toward 3x3x3 pictures and center twists: they need other certificates
  (section 2.8) and are a separate design.
