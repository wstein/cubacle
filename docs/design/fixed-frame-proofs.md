---
title: "Fixed-frame proofs"
description: "Design: a fixed-frame 2x2x2 mode with sticker identities and display goals, batch JSON Lines, and distance certificates that a checker sharing no code with cubacle can verify."
weight: 45
---

# Fixed-frame proofs

Status: approved design with partial implementation on `develop`. Plan row 6 is implemented:
`solve --json` and `POST /v1/solve` support formats 1 and 2 and report actual-result frame and
proof metadata. See [Solve options](../cli/solve-options.md#--json) for the implemented contract.
The fixed-frame engine, commands, certificates and REST routes remain planned until their
implementation stages pass review and are merged. Neither this design nor the reporting
implementation certifies the withdrawn full-state histogram or its unverified hash.

The owner decided the direction on 2026-10-29 (section 1.3) and the open questions of the first
draft on the same day:
they are recorded as DECIDED D1 to D7 in section 14, and the sections that depend on them
follow the decisions. Sections 17 to 20 were added with them: Compatibility (17), the REST
endpoint (18), releases, table hashes and the v0.2.0 gate (19), and the questions that are still
open (20). Marks used below:

- **[M]**: computed for this paper by throwaway scripts written from scratch (Python and C,
  not committed; see section 13). They share no code with cubacle or Cubistry.
- **[C]**: read from the code of `develop` at `c460ffa`, from the CI branch
  `ci/skip-rust-for-docs-only` at `ec7e3d9` where `ci-result` is named, and from the Cubistry
  `origin/main` at `35db0e1` where the proof checkers are named.
- **[R]**: a number reported by Cubistry Research (branch `research/cube-display-feasibility`,
  commit `8a85fd8`). Every [R] number used for a golden test was reproduced [M].
- **[E]**: estimate, not measured.
- **[A]**: assumption that the implementation or the owner must confirm.

> **Implementation-time audit (2026-10-29).** The full-state histogram reported by the
> uncommitted scratch program is inconsistent with the established rotation-quotient bound
> (section 13.1). Its histogram is withdrawn as a golden, and its associated hash remains
> unverified until checked recomputation. The independently reproduced k = 3 and k = 4
> goldens are unaffected. This does not change decisions D1 to D7 or authorize a release.

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
3. Other repositories pin only release tags with green CI. This work ships in a later tag,
   v0.2.0 (D3, D7), once the release gate of section 19.3 is met.
4. `--json` is extended and **versioned** (new fields, a batch mode in JSON Lines with tables
   built once). The first draft promised that the version 1 bytes stay the default and that
   existing consumers keep working unchanged; D3 (section 14) replaced that promise: format 2
   is the default at once, format 1 stays behind a selector, and the change is breaking
   (section 17).
5. MIT repositories use cubacle only as an external process in optional tests, never linked
   or embedded.

### 1.4 Goals and non-goals

Goals: exact fixed-frame semantics on the 2x2x2; identity-level display goals; exact distances,
eccentricities and runs; certificates; JSON Lines batch; deterministic, byte-reproducible
output.

Non-goals: no change to the search or the text output of `solve` and `solve --to`, and none to
Orbit64 (byte-identical, as the don't-care design requires); the one change to a default output
is the body of `solve --json` and `POST /v1/solve`, which gets the format 2 keys (D3, section 17)
and keeps every solution and length; no print or layout search (that stays in Cubistry); no
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
reachable from the print are 8! x 3^7 = 88,179,840 (the corner invariant and the 24-fold
rotation lift; the historical scratch report is audited in section 13.1). Every state has
a unique full sticker list; the corner twist sum is the only
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
`[id, cubie, [p], [n], [up], image]`, where `image` is null when absent in the print.
An absent image is distinct from the empty image string of a blank; input null is not an image.

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
  `N_8 = N_7 = 88,179,840`. The missing body occupies the one unused position. Its twist is
  derived by the geometric invariant below, not by summing the reference-x twist digits.

**Normative reachability invariant and eighth-pose decoding.** At position `p`, let the signed
outward axes be `X = (p.x,0,0)`, `Y = (0,p.y,0)`, `Z = (0,0,p.z)`. Define the right-handed
cyclic corner frame `C(p)` as `(Y,Z,X)` if `p.x*p.y*p.z = +1`, otherwise `(Y,X,Z)`.
For each body, follow its reference Y-normal sticker through the proper rigid rotation; let
`o(b)` be the index 0, 1 or 2 of its current normal in `C(current position)`. In the print all
`o(b)` are 0. Every legal face turn preserves `sum(o(b)) modulo 3`, and, with distinct
positions and proper rigid bodies, sum 0 is the complete corner reachability condition.
To decode the missing body, enumerate its three proper rotations at the unused position and
choose the unique one making that sum 0; then compute its reference-x twist digit as above.
A checker derives these rotations geometrically, verifies the invariant on the 18 moves and
reference state, and rejects any full pose list whose sum is not 0. Permutation parity is
unrestricted on a 2x2x2. Reference-x twist digits alone are not an additive invariant.

For k = 0, the empty tracked set has one key, 0, and its distance table is the single byte 0.
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

**Premises use the union projection.** Let `U = T(D) union T(P)`, sorted by body id.
Enumerate the reachable keys of `U`, evaluate `P` there, and look up each distance by projecting
to the goal table on `T(D)`. Never evaluate `P` on a goal key that omits one of its bodies.
`sources` and the premise histogram count these union keys, not full-state completions; record
`sourceBodies` and `sourceKeys` to make the denominator explicit. Each key has the same number
of full-state completions: `(8-k)! * 3^(7-k)` for k <= 7, and 1 for k = 8. Thus the maximum is
also the maximum over full physical sources. At k = 0 this factor is 88,179,840. For k = 8 use
the seven-body encoding of section 2.4. A maximum witness carries a full, reachable `from`
state, its union `key`, and a shortest `path`; choose the lowest union key and then the lowest
full key among its completions for deterministic tie-breaking.

An empty display premise means all states. An unsatisfiable premise instead gives `sources: 0`,
`histogram: []`, `eccentricity: null`, and no witness; it must never establish a finite universal
bound by vacuity. An unsatisfiable target gives a goal record with `goalKeys: 0` and no table,
histogram or eccentricity; distance, eccentricity and run requests using it fail `invalid_display`.
Goal-record histogram and eccentricity always describe the unconditional goal table on T(D);
an optional goal premise requests `premiseClosed`, not conditional statistics. Separate
eccentricity records carry the conditional union-source histogram and maximum.
`premiseClosed` means every full state meeting D meets P, checked on the same union projection,
including the cases of empty or unsatisfiable predicates.

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

Default behavior is untouched: `solve --size 2` keeps the rotation quotient and the bytes of its
text output; only its `--json` body gains the format 2 keys (with `frame` derived from the
actual route and target, section 6). A
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
- Historical full fixed-frame report (**withdrawn histogram**, k = 7 reference key):
  0: 1, 1: 18, 2: 243, 3: 2,874,
  4: 28,000, 5: 205,416, 6: 1,168,516, 7: 5,402,628, 8: 20,776,176, 9: 45,391,616,
  10: 15,154,409, 11: 49,943. All 88,179,840 keys reached; maximum 11, the same diameter as
  the rotation quotient. Table SHA-256 for a print whose body `b` sits at position rank `b` (true
  of the Cubistry H/S/M prints [M]):
  `caf69d83df6fd4c42a899cfd55c664bca87da01408a5aa4973c4eee6633825d9`. This table is not
  shipped; D4 still requires publishing a verified hash (section 19). This particular reported
  histogram is contradictory, and the candidate hash is not accepted evidence (section 13.1).

## 4. Command line

One new subcommand; no existing command changes. The name `fixed-frame` and its three
subcommands `batch`, `distance` and `table` are decided (D1, section 14).

```text
cubacle fixed-frame batch    [--tables DIR] [--max-memory MB] [--certificate] < requests.jsonl > records.jsonl
cubacle fixed-frame distance --print PRINT.json --from STATE --display DISPLAY.json [--json]
cubacle fixed-frame table    --print PRINT.json (--display DISPLAY.json [--premise DISPLAY.json] | --full) --out TABLE.bin
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
- `table` writes the sidecar for one goal (and premise) and prints the goal record. `--full`
  takes the print's reference state, all 24 facelets, as the goal: the full-state table of
  section 3, whose hash is published with every release (section 19).
- Errors: text on standard error and exit status 1 for invalid input (print, state, display,
  JSON), 2 for an unsupported format version or puzzle, 3 for internal or I/O errors. In
  `batch` a bad request produces an `error` record and the run goes on, unless `--certificate`
  is set, which stops at the first error with the applicable exit status and writes no summary.
  Standard output may already contain a prefix: it is not a valid certificate. A checker requires
  the final summary, verifies its hash and count, and rejects truncation or trailing records.
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
| `print` | `formatVersion`, `puzzle` (`"2x2x2"`), `printing` (section 2.2) | `header` record; must be the first line; a later `print` line starts a new context and clears all goal bindings and cached tables |
| `goal` | `goalId`, `display`, optional `premise` | `goal` record (counts, histogram, eccentricity, table hash) |
| `distance` | `id`, `from` (a state), `goalId` or `display`, optional `stickers` | `distance` record |
| `eccentricity` | `id`, `goalId`, `premise` (a display), optional `witness` | `eccentricity` record |
| `run` | `id`, `from`, `goals` (list of goal ids, cyclic), `maxSteps` | `run` record (section 7.7) |
| `sample` | `id`, `count`, `seed`, `goalId` | `count` `distance` records from seeded random states (section 8) |

Goal ids are unique within a print context; rebinding an id is `invalid_display`. A distance
request with an inline display first emits a goal record containing that display, then its
distance record. Its generated goal id is `inline.` followed by the first 56 hexadecimal digits
of the SHA-256 of the canonical display bytes; caller-supplied ids beginning with `inline.` are
reserved and rejected. A generated-id collision with a different canonical display is an error,
never an overwrite. Identical inline goals reuse their binding. `distance --json` and the REST
distance endpoint return a single distance record with the inline `display` embedded instead of
`goalId`; they are query answers, not standalone certificates.

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
{"record":"goal","goalId":"blank-H-34","display":{"cells":[{"p":[-1,1,1],"n":[0,0,1],"accept":[{"image":""}]},{"p":[1,1,1],"n":[0,0,1],"accept":[{"image":"H","up":[0,1,0]}]},{"p":[-1,-1,1],"n":[0,0,1],"accept":[{"image":"3","up":[0,1,0]}]},{"p":[1,-1,1],"n":[0,0,1],"accept":[{"image":"4","up":[0,1,0]}]}]},"trackedBodies":[1,3,6,7],"keys":136080,"goalKeys":3,"deadRules":[],"histogram":[3,45,594,6015,39849,79143,10431],"eccentricity":6,"tableSha256":"0a3f73e6d95731fe778d47d5a8bdd092d925a3514f2936f60557899d38d81c1c"}
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

Canonicalization for hashing is explicit. Print fields are `id`, `cubie`, `p`, `n`, `up`,
then `image` only if present; unknown fields are excluded and stickers are sorted by id.
For a display, sort every `ids` list numerically; serialize each rule with `ids` or `image`
first and optional `up` second; deduplicate rules and sort their canonical ASCII bytes
lexicographically. Sort cells lexicographically by the numeric tuple `(p.x,p.y,p.z,n.x,n.y,n.z)`;
write their fields as `p`, `n`, `accept`, under `{"cells":[...]}`. `deadRules` indexes refer to
these canonical cells and rules, not request order. All output displays use this normal form.
Thus cell/rule order changes do not alter table bytes, canonical goal identity or records.

JSON escaping uses `\"` and `\\` for quote and backslash, does not escape `/`, uses
`\u00xx` (lowercase hexadecimal, no short escapes) for every control character, and
`\uxxxx` for every non-ASCII UTF-16 code unit. Code points above U+FFFF use a high/low
surrogate pair; isolated surrogate input is rejected. ASCII printable characters are literal.
The final newline is a single LF, not CRLF, and is included in the chain hash. Optional fields
are omitted unless explicitly defined as nullable; missing `image` remains missing in canonical
print bytes and is never silently converted into an empty string.

## 6. Versioning and the `solve --json` change

This section is the technical part of D3 (section 14). The consequences for consumers are in
section 17; the short form is that **this is a breaking change of the default output**.

- **New formats** carry `format` (`cubacle.fixed-frame`) and `formatVersion` (1). They belong to
  schema family v1 and use `common.schema.json` for `formatVersion`. A later incompatible change
  is format version 2 with a new schema file; cubacle keeps reading version 1 requests for at
  least one release after that. `formatVersion` is a counter per format: the 2 of the solve body
  below has nothing to do with the 1 of the fixed-frame lines or of the pattern search.
- **`solve --json` today** has no version field and no schema file. Its contract is "keys are
  appended, never reordered" (`docs/cli/solve-options.md`, `src/report/mod.rs`) [C]. The command
  and `POST /v1/solve` write the body with one function (`SolveReport::to_json`), and the
  documentation promises equal bytes for equivalent requests [C].
- **Format 2 is the default.** From v0.2.0, `solve --json` and `POST /v1/solve` write format 2.
  It appends these keys after all existing ones, so they come after `elapsedMicros` and after
  `phases` and `patternHint` when those are present. The names are camelCase, like the existing
  keys:

  | Key | Type | Meaning |
  | --- | --- | --- |
  | `formatVersion` | 2 | Marks the extended body. |
  | `toolVersion` | string | The crate version, `0.2.0`. |
  | `gitCommit` | string or null | Set by the release build from the environment; null otherwise. No build script is added. |
  | `frame` | `"any-orientation"` or `"fixed"` | Derived from the actual route and target, not cube size: the ordinary 2x2 table solve uses `"any-orientation"`; exact-state `--within` uses `"fixed"` even on 2x2; accepted rotation-quotiented routes use `"any-orientation"` at any size. New fixed-frame records always use the fixed world frame. |
  | `proven` | Boolean | True only when the actual result proves the emitted path shortest in HTM for its reported frame and target. Never infer this from `--optimal`, `--within` or puzzle size alone; fallback and interrupted results need their own evidence. |
  | `lowerBound` | integer or null | The proven lower bound: equal to `length` when `proven`; the depth searched out when a limit stopped an optimal search; null when nothing is proven. |

  Proof metadata is carried from the actual search result to the report. An interrupted
  `--optimal` returning an incumbent has `proven: false` unless its lower bound meets that
  incumbent. `--within --then-solve` falls back to the normal solver's evidence; the presence
  of `--within` does not prove the fallback optimal. A proof in a slices or blocks alphabet
  does not establish HTM optimality: report `proven: false` and `lowerBound: null` unless a
  separate valid HTM bound exists. Existing routes that refuse `--json` continue to refuse it;
  this format change does not enable new route/option combinations. Tests cover these route
  distinctions, bounded-search interruption, fallback, and quotient versus exact targets.

  Every key of format 1 keeps its name, place, type and meaning. The only difference is the tail:
  the `solve -s "R U" --json` example of `docs/cli/solve-options.md` is, in format 1 (as in
  v0.1.0) and in format 2:

  ```json
  {"kind":"3x3x3","solution":"U' R'","inverse":"R U","length":2,"metrics":{"htm":2,"qtm":2,"stm":2,"etm":2,"obtm":2,"btm":2,"atm":2},"elapsedMicros":0}
  {"kind":"3x3x3","solution":"U' R'","inverse":"R U","length":2,"metrics":{"htm":2,"qtm":2,"stm":2,"etm":2,"obtm":2,"btm":2,"atm":2},"elapsedMicros":0,"formatVersion":2,"toolVersion":"0.2.0","gitCommit":null,"frame":"fixed","proven":false,"lowerBound":null}
  ```

  (The second line is the first with the six keys appended, written by hand for this paper; the
  implementation PR replaces it by recorded output, and `elapsedMicros` is masked to 0 as in
  the existing tests.)
- **Version 1 stays available by an explicit selector.** `solve --json-format 1` and
  `"formatVersion": 1` in a `POST /v1/solve` request write the v0.1.0 body, byte for byte;
  `--json-format 2` and `"formatVersion": 2` name the default. Any other value is an error (exit
  status 1; 400 `bad_input`). `--json-format` without `--json` is an error. There is no
  environment variable and no configuration key that changes the default, and no alias for the
  old behavior: the project is alpha and `AGENTS.md` says to rename without alias or fallback.
  The URL stays `/v1/solve`; the version of the body is negotiated in the request, not in the
  path.
- **Tests.** The recorded literals `tests/solve_json_body.txt` and `tests/solve_json_body_four.txt`
  stay as they are and become the expectation of format 1: the existing tests only gain the
  selector in their arguments. Format 2 gets literals of its own, and one property test states
  the relation above: the body of format 2, with the six keys cut off its end, equals the body of
  format 1 (`elapsedMicros` masked). `AGENTS.md` forbids editing recorded solver output to make
  a change pass; this is the explicit decision it asks for (D3), and no solution or length
  changes.
- **Schema for the existing body.** `solve-result.schema.json` (v1, a draft is in this pull
  request) describes both bodies: the keys of format 1, and the six keys of format 2 as optional
  together, so the contract has a file.
- **Pinning.** Consumers pin a release tag and the SHA-256 of the release tarball, record
  `toolVersion`, and refuse an unknown `formatVersion` (a body without the key is format 1).

## 7. Certificate

### 7.1 Trust model

The checker trusts only itself: its JSON reader, its own integer move engine on sticker frames,
its own implementation of `pose24.v1` and `key.v1`, and SHA-256. It does **not** trust cubacle's
moves, tables, hashes, paths or counts. A certificate is accepted only if every claim in it is
verified; a check it cannot run is a failure, not a skip. A checker must share no code with
cubacle (no port, no generated tables, no linked library); it may read the same specification.

### 7.2 What a certificate is

A certificate is the batch output with `--certificate`: a JSON Lines file with a `header`
record, then the answer records, then one final `summary` record, plus optional sidecar tables. The
header contains the complete print (not only its hash), the frame string, the move order and
the encodings, so the file states the whole problem on its own. The summary holds the record
count and `chainSha256`, the SHA-256 of all preceding lines as bytes (each with its `\n`).
Every goal record includes its canonical `display` and optional `premise`; every eccentricity
record includes its actual `premise`, union-source encoding and optional full-state witness.
Every run includes the cyclic `goals` list, `maxSteps`, and an implicit initial next-goal index
of 0; its step i targets `goals[i modulo goals.length]`. A checker needs no request file.
Later headers start new print contexts and invalidate preceding goal bindings; each record
resolves only against its most recent header. A single summary covers all contexts, including
all headers. Empty output, a missing summary, an error record, an unknown goal binding, a
second summary or any trailing record invalidates a certificate. Producers may emit a prefix
on failure, but never a successful summary; checker acceptance is atomic.

| Claim | Evidence in the certificate | What the checker does |
| --- | --- | --- |
| `dist(s, D) <= d` | `path` of length `d` | Replays it on the full 24-sticker state of `s` with its own engine; checks the end meets `D` and equals `reached`. |
| `dist(s, D) >= d` | goal table of `D` (by hash or sidecar) and `key` | Verifies the table (7.4), computes the key of `s` itself, checks `table[key] == d`. |
| `ecc(D \| P) = e` | goal table, premise `P`, `witness` key and path | Verifies the table; maximum of the goal table over union-source keys meeting `P` equals `e`; the full witness meets `P`, projects to its declared key and has value `e`. |
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
`tableSha256`; the file name carries no meaning beyond that. They are local output of
`--tables`; no release contains one (D4).

### 7.5 Eccentricity and universal bounds

The consumers' universal claims become sets of eccentricity records. For example "after setup
of the letter, every same-letter transition takes at most five" on the preferred intermediate
print is, per target display `D` (180 of them), `ecc(D | P_letter) <= 5`, where the premise
`P_letter` puts the letter upright upper right. The induction step of the Cubistry proof
(every target meets the premise again) is a check of `G(D)` against `P`: the checker verifies
that every union key meeting `D` also meets `P` (on the union of tracked bodies). cubacle reports it as
`premiseClosed: true`; the checker recomputes it.

### 7.6 What is not certified

Print optimality, global route optimality, the counting policy of the Cubistry generators,
intermediate frames of an animation, timing, and anything about other puzzles. A certificate
covers exactly its records.

### 7.7 Runs and period closure

A `run` request gives a start state and a cyclic list of goal ids. cubacle steps with the
lex-least shortest word to the next goal, until the pair (full state, goal index) repeats or
`maxSteps` is reached. The record lists every step (`goalId`, `distance`, `path`) and the
`preperiod` and `period`, or `closed: false`. It also records the complete cyclic `goals`
list and `maxSteps`. The first target is `goals[0]`. At each step boundary the repetition key is
(the complete physical state, index of the next goal); the initial boundary is step 0.
For a closed run, `preperiod + period` equals the number of recorded steps and the first repeat
must occur there; an open run has exactly `maxSteps` steps and no earlier repeat. A closed run
has both period fields; an open run has neither. Zero-move steps are real steps and advance the
goal index. The checker verifies every step against the referenced goal, not just the final state.
cubacle's policy is its tie rule, not Cubistry's
policy 13 or 1, so its runs are not expected to equal Cubistry's runs; the universal bounds of
section 7.5 hold for every shortest policy, which is what the golden tests use.

### 7.8 The independent checkers (D5)

Two checkers, one in Flix and one in Julia, written without cubacle code and without each other's
code. Python is a development aid only: it may generate expected data and help to find the
difference between two verdicts, and no certificate is accepted on the strength of a Python
checker.

**Where they live [C].** The Flix and Julia checkers are in the Cubistry repository
(`/home/claude/cubistry`, read from `origin/main`, commit `35db0e1`), in `proof-checkers/flix/`
(Flix 0.79.0 through the `flixw` wrapper, Java 21; Jackson Core only to tokenize JSON, all type
policy and mathematics in Flix) and `proof-checkers/julia/` (Julia 1.13.3, JSON 1.13.0), under
the contract `proof-checkers/SPEC.md` of Cubistry issue 74 and its shared `INTERFACE.md`. Two
other places were looked at and are not checkers: `/home/claude/cube-solvers` is the Java and
Gradle solver family `wstein/cube-solvers` (engines 2x2 to 5x5 and a replay conformance suite;
no Flix or Julia code) and `/home/claude/wstein/flix-cube-solvers` is a Flix solver package (cube
model, a 2x2 Datalog distance table over a pocket cube that turns three faces, 3x3
coordinates). They
are solvers; they may serve as extra producers of test data, and a checker must not import
either of them.

**What the contract already gives us.** The SPEC fixes the same world frame as section 2.1 (a
quarter turn of R, U, F is -90 degrees about +x, +y, +z), the same wire state, the same strict
JSON policy as section 5.3 (raw UTF-8 bytes, duplicate keys, NaN and `1.0` rejected, 16 MiB, depth
64, 128-byte numeric tokens), a corpus pinned by hash, unknown formats failing closed, a verdict
object with `valid` and `status` (`VALID`, `INVALID`, `INVALID_JSON`, `UNSUPPORTED`, `ERROR`)
and the exit codes 0, 1, 2 and 3. It says that a future Cubacle producer "must declare its own
twist order/direction/reference frame": this paper does, in sections 2.1 and 2.4 and in the
`frame` string of the header. Today both checkers implement the counter certificate (Group 1) and
marked 3x3x3 states (Group 2); a fixed-frame certificate is `UNSUPPORTED` for them until the SPEC
has a Group 3. Strictly speaking the Cubistry coordinator owns that SPEC, so the Group 3 text is a
Cubistry pull request.

**How they consume the certificate.** Each checker gets a new command, `fixedframe FILE
[TABLES_DIR]`, next to `counter` and `state3`. It reads the certificate of section 7.2 (a JSON
Lines file, plus sidecars for mode L) as raw bytes, parses every line strictly, and:

1. rejects an unknown `format` or `formatVersion`;
2. validates the print (section 2.2) and recomputes `printSha256` over the canonical print bytes;
3. builds its own move engine on sticker frames and its own `pose24.v1` and `key.v1`, and
   self-tests them: the 18 moves are permutations of the 24 facelets, `X X'` is the identity, a
   quarter turn has order 4, the reference state has the poses `3 * positionRank`;
4. for each goal record checks the tracked bodies, `goalKeys`, `deadRules` and the table by mode
   R or mode L, then the histogram and the eccentricity;
5. for each distance, eccentricity and run record makes the checks of section 7.2;
6. recomputes `chainSha256` and the record count;
7. prints the verdict object and exits 0, 1, 2 or 3 as the SPEC says.

The mode R search (a breadth-first search over up to 1.6 million keys, 14.7 million at most for
the REST limit) is fast in Julia and acceptable in Flix on the JVM; mode L over the 88-million-key
table is a job for both, not for CPython. The scratch Python searches of this paper took about
20 s for 136,080 keys and about 90 s for 1,632,960 on a loaded machine [M]; that is the reason
Python is not the authority, not a reason to avoid it for development. Expected effort per
checker [E]: 400 to 600 lines plus the corpus entries.

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
| k = 7 or 8 (full) | 1 to 4 s and 84 MB [E], scaled from the 64 ms build with 24 times the states and twice the moves; the C scratch program took 5 min 20 s of CPU on a heavily loaded two-core machine without any tuning [M]. This is the table of D4: generated, hashed, never shipped (section 19.1) |
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
| Full fixed frame from the reference | Recompute the complete table and verify the quotient-ball bounds of section 13.1. The old histogram, diameter claim and candidate SHA-256 `caf69d83df6fd4c42a899cfd55c664bca87da01408a5aa4973c4eee6633825d9` remain unverified; do not freeze them as a fixture. |
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
- A display and the same display with its cells or rules reordered give identical canonical records.
- A premise naming an otherwise untracked body uses the union projection; test empty and
  unsatisfiable premises, k = 0, and all-eight-body union keys.
- A certificate alone reconstructs every target, premise and run phase; reject missing goals,
  truncated output, goal rebinding, a wrong premise, a changed phase and trailing records.
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

- Default `cargo test`: k <= 4 cases, the five golden cases of the release gate (section 19.3),
  the counterexample, negative controls, the determinism byte test. Budget: under one second of
  test time in release [E]. The golden cases also run alone, as the CI job `fixed-frame-golden`
  (`cargo test --release --test fixed_frame_golden_test`), so their result does not wait for the
  slow `rust` job.
- `#[ignore]`, filter `fixed_frame_full`: the full 88-million-key table and its hash against
  `tests/fixtures/fixed_frame/table-hashes.json` (section 19.1), k = 5 and k = 6 goldens, the
  180-display H/S/M batch. Run on request, and in the opt-in CI job `fixed-frame-full`, like
  `four-full`, never in the default `rust` job, which already times out on `develop`.
- The independent checkers (Flix and Julia) run in Cubistry, in optional tests that use the
  pinned release, never in cubacle's CI. They are not part of the release gate (section 19.3);
  they follow the tag and pin it.

## 11. Performance, memory and CI

The default suite gains only the small cases; two small CI jobs are added, `fixed-frame-golden`
and `fixed-frame-full` (section 19.3). No table is cached on disk (tables of k <= 7
build in seconds, so the cost of a cache and its integrity hash is not worth it). The 1.5 GB cap
holds with a wide margin: the largest table is 84 MB. The server gains the endpoints of D6
(section 18); they build their tables per request and keep none, which answers the don't-care
design's open question 6.

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
  N_7 = 8! x 3^7 = 88,179,840 was reported by the scratch full search. The four Cubistry
  reference histograms for 9,072 / 378 and 136,080 / 810 keys were independently reproduced.
  Other new numerical goldens still need implementation checks; the full histogram is withdrawn
  (section 13.1).
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
- [C] The CI jobs and their limits (`ci.yml`; `ci-result` and `changes` only on the branch
  `ci/skip-rust-for-docs-only`), the tag rule of `release.yml` (tag equals the `version` of
  `Cargo.toml`) and its `--generate-notes`, the absence of `deny.toml`, `cargo-deny` and a
  changelog file, the REST server's flags, limits, semaphore and error codes (`docs/api.md`,
  `crates/server/`), and every consumer of the solve body listed in section 17.3.
- [C] In Cubistry `origin/main`: `proof-checkers/SPEC.md`, `INTERFACE.md` and the Flix and Julia
  directories (versions and parsers as named in section 7.8); `/home/claude/cube-solvers` and
  `/home/claude/wstein/flix-cube-solvers` contain solvers and no checker. All read-only.
- [C] A read-only search of `/home/claude/cubelab`, `/home/claude/cubistry` and
  `/home/claude/puzlwerk` found no consumer of `solve --json` or `POST /v1/solve`.

The scratch scripts: a Python projection search over sticker frames with integer quarter-turn
rotations (about 150 lines), a C full search over the k = 7 key with move tables generated by the
Python model, and a script that computes the example values with the real print. They were run
once and are not part of the repository; the implementation PRs turn their numbers into tests.

Assumed:

- [A] The Cubistry frame convention is the one the consumers will keep (checked in the H/S/M
  verifier and the proof-checkers wording; a future change there is a new frame string).
- [E] All Rust run times and memory figures in section 9 and section 19.2, and the sizes of the
  plan in section 15.
- [A] The limits of section 18.3 and the value of `frame` for each engine (section 6).
- [A] Cubistry's displays need at most 24 cells and only the predicates of section 2.5 (exact
  sticker, optional exact `up`); "any rotation of the glyph by 180 degrees" would be two rules,
  which works, but is untested.

### 13.1 Full-table audit: a contradictory reported histogram

The existing pocket-table regression (`src/engines/two/tests.rs`) has 2,644 rotation-quotient
states at distance 11 and 3,671,516 within distance 10. The 24 proper whole-cube rotations act
freely on full physical states and conjugate outer-face HTM moves to outer-face HTM moves.
Normalizing the fixed corner after an opposite-face move gives one of the U/R/F quotient moves.
Thus every fixed-frame word projects to a quotient word of no greater length, and
`|B_fixed(r)| <= 24 * |B_quotient(r)|` for every radius r.

The reported full histogram violates this necessary condition at radius 10:
`88,129,897 > 24 * 3,671,516 = 88,116,384`, an excess of 13,513. Equivalently, at least
`24 * 2,644 = 63,456` full states have distance at least 11, whereas the old report claims
only 49,943 and a diameter of 11. Those claims cannot all hold. An independent reviewer
confirmed this argument in [PR #6](https://github.com/wstein/cubacle/pull/6#issuecomment-6510472080).

The full histogram is therefore withdrawn as acceptance data. This counting argument alone
neither establishes a replacement histogram nor disproves the candidate hash: actual complete
bytes must be regenerated and checked. Do not alter the engine or an existing recorded fixture
to fit the contradictory report. The implementation's ignored full test records actual counts
and the hash, checks reachability and the quotient-ball inequalities, and leaves the immutable
release hash fixture pending measured evidence and review. The v0.2.0 gate is incomplete until
that fixture is verified; small projection goldens remain required and unchanged.

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

### D3. Format 2 is the default of `solve --json` at once (DECIDED)

Format 2 (camelCase keys `formatVersion`, `toolVersion`, `gitCommit`, `frame`, `proven`,
`lowerBound`, as drafted in section 6) is the default of `solve --json` from v0.2.0. The owner
wants `proven` and the frame to be visible to every consumer without a flag that nobody sets, and
the project is alpha, where `AGENTS.md` accepts breaking changes without alias or fallback.

The first draft said that the version 1 bytes stay unchanged. That cannot hold together with this
decision, and this paper does not pretend otherwise: the default body now has six more keys, so
its bytes change. What is kept: every v1 key in its place and meaning, the solutions and lengths,
and the old bytes exactly, behind `--json-format 1` (REST: `"formatVersion": 1`). What is done
about it: existing consumers must be told (release notes, the docs page of `--json`), the change
is marked breaking for the changelog and is the reason the next tag is **v0.2.0** (alpha semver:
a breaking change raises the minor number). The details, the list of consumers found and the
migration are in section 17, the pull request in row 6 of section 15.

### D4. Releases do not ship the full-state distance table (DECIDED)

A release does not contain the 84 MB table of 88,179,840 keys. The table is generated
reproducibly, and only its SHA-256, per frame and print definition, is published: in the release
notes and in a small checked-in file (section 19). A table that every correct implementation
rebuilds in seconds adds nothing as a download except a larger release and a file to trust, while
the hash lets anyone, and the independent checkers, confirm their own table.

### D5. Two independent checkers, in Flix and in Julia (DECIDED)

The consumers of the certificate are two checkers that share no code with cubacle and none with
each other: one in Flix and one in Julia, which already exist as the proof checkers of Cubistry
(section 7.8). Python may stay as a development aid (expected data, differential debugging) and
is not an authority. Two languages and runtimes (the JVM and Julia) make an agreed verdict much
stronger than one checker, and Cubistry already has their contract, build and review process.
The question who owns the checker is closed: Cubistry owns them, cubacle owns the certificate;
the plan is in section 15 (rows C1 to C3).

### D6. A REST endpoint ships with the first fixed-frame release (DECIDED)

`POST /v1/fixed-frame/batch` and `POST /v1/fixed-frame/distance` are part of v0.2.0, designed in
section 18. Request and response are the records of section 5 and the same two schema files,
byte for byte what the command line writes, so the server adds no second format; the tables that
the consumers need (four bodies, 136,080 keys) build in milliseconds [E], which is why no state
is kept between requests.

### D7. The release gate of v0.2.0 (DECIDED)

The tag v0.2.0 waits for all required CI jobs to be green, `ci-result` included, and for the
Cubistry-reference golden tests to be green in CI (section 19.3). The slow `rust` job must be
repaired first; a separate investigation exists, and this paper only records that the gate
depends on it. Other repositories pin only release tags with green CI (1.3.3), so a partial gate
would defeat the point of a tag, and the goldens are the only tie between cubacle's answers and
the numbers of the Cubistry proofs.

The questions that remain open after these decisions are in section 20.

## 15. Implementation plan

Small pull requests against `develop`, each Draft until reviewed; sizes in changed lines
including tests, [E]. Rows 4a and 6 do not depend on the engine and can go first or in parallel.
The decisions D1 to D7 (section 14) are built in as shown in the last column.

| # | Pull request | Size | Depends on | Decision |
| --- | --- | ---: | --- | --- |
| 1 | `docs(design)`: this paper and the three draft schemas (request, record, solve result) | docs only | none | D1 to D7 |
| 2 | `feat(puzzle)`: frame model: print validation, sticker-frame moves, `pose24.v1`, `key.v1`, state input forms; the fixture prints copied from Cubistry with their hashes in `NOTICE.md`; golden poses of the blank counterexample | 450 | 1 | |
| 3 | `feat(two)`: fixed-frame goal tables: `FrameGoal`, tracked bodies, multi-source search over keys, lex-least walk, `u8-dense-v1` with SHA-256; the goldens without premise (three bodies, blank print) in `fixed_frame_golden_test` | 500 | 2, 4a | D2 |
| 4a | `build(deps)` and `feat`: `serde_json`, `serde` (std only), `sha2` as dependencies; the strict reader with duplicate-key check and bounds; negative controls named after the strict-only cases of the Cubistry corpus; a ChartScribe decision note; owner review of the dependency edges | 350 | none | D2 |
| 4b | `feat(cli)`: `fixed-frame batch`, `distance`, `table` (with `--full`); the public `run_batch` and `Limits`; records, header, summary; determinism test | 450 | 3, 4a | D1 |
| 5 | `feat(two)`: eccentricity with premises, `premiseClosed`, runs, `sample`; sidecars; the goldens with premise (378 and 810 sources, counterexample); ignored `fixed_frame_full` tests; the measured time and memory of the full table replace the estimates of section 19.2 | 450 | 4b | D4, D7 |
| 6 | `feat(report)!`: `solve --json-format 1\|2`, format 2 the default for the command and for REST (`formatVersion`), `solve-result.schema.json`; the existing tests gain the selector, format 2 gets new literals, docs and marked examples are regenerated; title with `!` and a `BREAKING CHANGE:` footer, section 17 | 300, plus about 150 of test and doc churn | none (parallel) | D3 |
| 7 | `feat(server)`: `POST /v1/fixed-frame/batch` and `/distance`, the three flags of section 18.3, tests including byte equality with the command, rows in `docs/api.md` and `crates/server/README.md` | 400 | 4b (full ops after 5) | D6 |
| 8 | `ci`: jobs `fixed-frame-golden` and `fixed-frame-full`, both added to `ci-result`; `tests/fixtures/fixed_frame/table-hashes.json` | 100 | 5; the `ci-result` branch merged | D4, D7 |
| 9 | `docs`: CLI page `docs/cli/fixed-frame.md` from measured behavior, engine page, a sentence on the 2x2 engine page about the two frames, decision notes | docs | 4b, 5, 7 | |
| 10 | `chore(release)`: version 0.2.0 in the root crate and in `crates/server`; the release workflow writes the Breaking changes and Fixed-frame tables blocks and attaches `table-hashes.json` | 40 | 6, 7, 8 | D3, D4, D7 |
| C1 | Cubistry: Group 3 of `proof-checkers/SPEC.md`, corpus entries (a golden certificate produced by row 5 as input, expected results from independent Research scripts, the corrupted certificates of section 10.3), coverage ledger | 300 plus data | 5 | D5 |
| C2 | Cubistry: the `fixedframe` command in `proof-checkers/flix/` | 500 | C1 | D5 |
| C3 | Cubistry: the `fixedframe` command in `proof-checkers/julia/` | 400 | C1 | D5 |

Rows C1 to C3 are pull requests in another repository, opened by its owners under its own
process; cubacle does not push there. They follow the tag and pin v0.2.0. Files stay under the
500-line planning mark: the frame model, the goal tables, the batch reader, the record writer
and the REST routes are separate modules (`src/puzzle/frame/`, `src/engines/two/fixed/`,
`src/fixed_frame_cmd/`, `crates/server/src/fixed_frame.rs`). Engines do not use each other's
internals; the fixed-frame tables use nothing of `PocketSolver` but may share its `Turn` and
move types. The tag is cut by the owner when the gate of section 19.3 holds.

## 16. Risks

- **Two frames in one tool.** A consumer may call `solve --size 2` and take a rotation-quotient
  length for a fixed-frame distance, which is shorter or puts the display on another face. Mitigation:
  separate subcommand, `frame` key in `solve --json` version 2, a sentence on the 2x2 engine page.
- **Convention drift** between cubacle and the consumers (axis signs, turn direction, twist
  digit). Mitigation: the frame string in the header, the checker's self-tests, goldens with real
  prints, and the counterexample replayed by both sides.
- **False independence.** A checker that reuses cubacle code or tables proves nothing. Mitigation:
  the contract forbids it; the checkers live in another repository with its own review; there
  are two of them in two languages, and the corpus runs the same cases through both, so a
  disagreement shows at once.
- **Strict JSON** rests on `serde_json` plus our duplicate-key check (D2). A gap in the check
  would let a duplicate key through; the negative controls of section 10.3 cover it, and the
  independent checkers read the same bytes with their own parsers.
- **CI.** The `rust` job times out on `develop` now. A separate investigation repairs it, and
  the release gate depends on that repair (D7): this work cannot be tagged before it, and it can
  be blocked by a job it did not break. The new default tests stay small, the heavy ones stay
  ignored and have their own job, and the golden cases have a small job of their own so that a
  regression in them shows even while `rust` is red.
- **Large tables in a checker.** Mode L on 88 million keys is a compiled-checker job (Julia, Flix
  on the JVM); Python is a development aid only (D5) and is kept to k <= 5.
- **The checkers depend on another repository's SPEC.** A fixed-frame certificate is
  `UNSUPPORTED` for both checkers until Cubistry accepts a Group 3 (section 20). Mitigation:
  the Group 3 text and corpus are the first Cubistry pull request (row C1), and the tag does not
  wait for it.
- **Breaking default output** (D3). Consumers that compare or hash the whole `solve --json` body
  break on v0.2.0, and the search of section 17.3 cannot see every consumer. Mitigation: the
  selector `--json-format 1`, the Breaking changes block of the release notes, v1 bytes kept
  under test, and the v0.2.0 number.
- **A costly endpoint without authentication** (D6). A client can hold every permit for the
  timeout. Mitigation: the limits of section 18.3, local binding by default, the documented
  rule not to expose the server to untrusted networks, as for `solve`.
- **Scope creep** toward 3x3x3 pictures and center twists: they need other certificates
  (section 2.8) and are a separate design.

## 17. Compatibility

**Breaking change.** From v0.2.0 the default body of `cubacle solve --json` and of
`POST /v1/solve` is format 2 (section 6). That changes the bytes of the default output of two
interfaces that v0.1.0 documents (D3). It is marked breaking for the changelog and it is the
reason the next tag is v0.2.0: while the major version is 0, a breaking change raises the minor
number. The release workflow requires the tag to equal the `version` of `Cargo.toml`
(`.github/workflows/release.yml` [C]), so the release pull request (row 10 of section 15) sets
`0.2.0` in the root crate and in `crates/server`.

### 17.1 What changes and what does not

| | v0.1.0 and `--json-format 1` | v0.2.0 default (format 2) |
| --- | --- | --- |
| Keys of the v1 body, their order, types, meaning | as documented | unchanged |
| Keys at the end of the body | `elapsedMicros`, then `phases` and `patternHint` when present | the same, then `formatVersion`, `toolVersion`, `gitCommit`, `frame`, `proven`, `lowerBound` |
| Bytes of the body | the v0.1.0 bytes | v1 bytes with the six keys appended (and `Content-Length` changes) |
| Solutions, lengths, metrics, text output on standard output and error, exit status, error bodies, other endpoints | as before | unchanged |

A consumer that reads the body as JSON and ignores keys it does not know keeps working without a
change. A consumer breaks when it compares or hashes the whole body, checks it with a closed
schema (`additionalProperties: false`), parses it positionally, or diffs recorded output.

### 17.2 What has to change, and for whom

- **Nothing**, for a consumer that ignores unknown keys. It may start to use `proven` and
  `lowerBound`, and should refuse a `formatVersion` it does not know (a body without the key is
  format 1).
- **One argument or one field**, for a consumer that needs the old bytes during a transition:
  `--json-format 1` on the command line, `"formatVersion": 1` in the request body.
- Version 1 is not given an end date. Removing it would be a new breaking change, decided by the
  owner and announced the same way.

How consumers are told: the release notes of v0.2.0 start with a **Breaking changes** block
that names the default change, the selector and the new keys (the release workflow creates its
notes with `--generate-notes`; row 10 adds the block); the pull request of row 6 has a title with
`!` and a `BREAKING CHANGE:` footer in its commit message; `docs/cli/solve-options.md` and
`docs/api.md` get a paragraph "Changed in 0.2.0" with both bodies; the schema file names both
formats.

### 17.3 Consumers found

In this repository (read from the branch of this pull request [C]; searched for `--json`,
`elapsedMicros`, `/v1/solve`, `solve_json_body`, schemas, docs, tests and REST code):

| Where | What | Needed in row 6 |
| --- | --- | --- |
| `src/report/mod.rs`, `src/report/json.rs` | the writer, `SolveReport::to_json` | takes the format; format 1 is today's code |
| `src/answer.rs`, `src/solve_cmd.rs`, `src/partial_cmd.rs`, `src/cli.rs` | print the body for `--json`, refuse it with other layouts, define the option | `--json-format`, its refusal without `--json` |
| `crates/server/src/solve.rs` (`report.to_json()`) | the REST body | read `formatVersion` from the request |
| `tests/solve_json_cli_test.rs` with `tests/solve_json_body.txt`, `tests/solve_json_body_four.txt` | byte tests of the command | add the selector for format 1, add format 2 literals |
| `crates/server/tests/solve_json_test.rs`, `api_test.rs`, `improve_test.rs` | byte and shape tests of the server | same |
| `tests/centers_target_cli_test.rs` | only checks that `--json` is refused with `--to centers` | none |
| `docs/cli/solve-options.md`, `docs/cli/solve.md`, `docs/examples.toml` (`json-fast`, `json-default`), `docs/api.md`, `crates/server/README.md`, `docs/contributing.md` | the contract, the marked examples that `cargo xtask docs` rewrites, the REST bodies | regenerate the examples, update the bodies, add "Changed in 0.2.0" |
| `xtask/src/docs.rs` | masks `elapsedMicros` in examples | none (the key stays) |
| `docs/specs/rest-json.md`, `search-result.schema.json` | the pattern search endpoint, its own `formatVersion` 1 | none |
| `.github/workflows/release.yml` | smoke test greps the text line `moves ` | none |

Outside this repository, read-only searches of `/home/claude/cubelab`, `/home/claude/cubistry`
and `/home/claude/puzlwerk` for `cubacle` (case-insensitive) together with `--json`, `solve`,
`/v1/solve`, `elapsedMicros` and `cubacle-server`:

- **No consumer of the solve body was found.** `cubelab` mentions cubacle only as the producer
  `rust/v1` of the synthetic photo (`cubacle photo`) in its fingerprint schema and page.
  `cubistry` names Cubacle in its proof-checker documents (`proof-checkers/SPEC.md`,
  `README.md`, `RESULTS.md`), in architecture and roadmap documents and in research data; none
  of them calls `cubacle solve` or the server. `puzlwerk` names Cubacle in its README, ADRs and
  `specs/001-core-foundation/contracts/solver.md`, which proposes a protocol of its own
  (`puzlwerk.solver`) and says "No current Cubacle protocol support is asserted".
- The search sees working trees as they are today; it cannot see scripts outside these three
  repositories, other machines, or people who use the v0.1.0 release (the only tag [C]) or the
  server. For those the release notes and the selector are the whole mitigation.

### 17.4 What stays compatible on purpose

The fixed-frame formats are new and have no earlier consumers. The batch and REST records use
their own `formatVersion` 1 (section 5). The endpoints other than `POST /v1/solve` and the
text output of every command are untouched.

## 18. REST endpoint

Decided in D6. The endpoints live in `crates/server` and follow the rules of the existing server
(`docs/api.md`): stateless, compute on blocking threads behind the semaphore, errors in the
shape `{"error":{"code","message"}}`, listener on `127.0.0.1` by default.

### 18.1 Paths and bodies

| Method and path | Request body | Success |
| --- | --- | --- |
| `POST /v1/fixed-frame/batch` (query `certificate=true`, optional) | `Content-Type: application/x-ndjson`: the request lines of section 5, the first one the `print` line ([schema](../static/schema/v1/fixed-frame-request.schema.json)) | 200, `application/x-ndjson`: the record lines of section 5 ([schema](../static/schema/v1/fixed-frame-record.schema.json)), byte for byte what `cubacle fixed-frame batch` writes for the same input; `certificate=true` is `--certificate` |
| `POST /v1/fixed-frame/distance` | `Content-Type: application/json`: one object with `formatVersion`, `puzzle`, `printing`, `from`, `display` and optional `stickers` (`restDistanceRequest` in the request schema) | 200, `application/json`: one `distance` record, what `distance --json` prints |

There is no `table` endpoint and no sidecar: a table of up to 84 MB and files on a server's disk
do not fit a stateless server. The goal record carries `tableSha256`, and a client that wants the
table recomputes it (mode R of section 7.4).

The newline-delimited type is not a "simple" cross-site content type, so, like the JSON-only rule
of the other endpoints, a browser form cannot trigger the work; any other type is a 415
`unsupported_media_type`. The response is built completely before its first byte is sent, so a
status code is always the status of the whole request: there is no partial 200 after a timeout,
and a certificate is never partial.

### 18.2 Errors

- Per-line problems in the default mode are `error` records inside a 200 response, as in the
  command line (section 4); the run goes on.
- With `certificate=true` the first error ends the request: 400 `bad_input`, whose message holds
  the code, the line number and the message of the error record that the command would have
  written. An unsupported `formatVersion` or puzzle in the `print` line (exit status 2 of the
  command) is a 400 `bad_input` in both modes; an internal failure (exit status 3) is 500
  `internal`.
- The existing codes keep their meaning: 413 `payload_too_large`, 415 `unsupported_media_type`,
  429 `busy` with `Retry-After: 1`, 504 `budget_exhausted`, 404 and 405.

### 18.3 Limits

The values are proposals [A], confirmed or changed by the measurements of the server pull
request; every limit is checked before the work it bounds, and a limit that stops one line is an
`error` record with code `over_limit` unless the table says otherwise.

| Limit | Flag and environment variable | Default | Reason |
| --- | --- | --- | --- |
| Body size of the two routes | `--max-fixed-frame-body-bytes`, `CUBACLE_MAX_FIXED_FRAME_BODY_BYTES` | 1048576 | The shared `--max-body-bytes` (16384) holds a print (about 2 KB) and a few goals; a batch of 180 displays is about 100 KB. A route-level limit leaves the other endpoints at 16384. Over it: 413. |
| Lines per request | none, fixed | 2000 | Bounds the number of operations. Over it: 413. |
| Distinct goal tables per request | none, fixed | 256 | Bounds the build work. |
| Tracked bodies per goal or goal/premise union | none, fixed | at most 6 (14,696,640 keys, 14 MB) | The seven- and eight-body table (84 MB, seconds of CPU) is a job for the command line (D4), not for a request. |
| Memory per request | `--max-fixed-frame-memory-mb`, `CUBACLE_MAX_FIXED_FRAME_MEMORY_MB` | 128 | All live allocations: tables, frontiers, parsed input, goal metadata, run state and response buffer. A queue of u32 keys costs 4 bytes per entry, not 1; a scan-per-depth builder may avoid that queue. Check sizes with overflow-safe arithmetic before allocation and account retained tables during eviction/build. With the default concurrency the server needs that many times the number of CPUs at most, which is the place to lower it. |
| Response size | none, fixed | 8 MiB | Bounds the output of one request; 8 MiB is some tens of thousands of distance records [E]. Over it: an `error` record `over_limit` and the request ends. |
| `run.maxSteps` | none, fixed | 10000 | The schema allows more for the command line. |
| `sample.count`, all `sample` lines together | none, fixed | 10000 | Same. |
| Time | `--request-timeout-ms`, `CUBACLE_REQUEST_TIMEOUT_MS` (existing) | 30000 | One deadline for the whole request, checked between lines and at every layer of a table build. Over it: 504 `budget_exhausted`, no body. There is no `budgetMs` field: the answer is either complete or absent, never a function of the clock. |
| Concurrency | `--max-concurrency` (existing) | number of CPUs | One permit per request, as for `solve`; no free permit is a 429 at once. |

### 18.4 No persistent state

Tables are built inside the request and dropped at its end. There is no cache between requests,
no file on disk, no `--tables`, no session; instances can sit behind a load balancer, as
`docs/api.md` promises. The cost is small for the consumers' case (tables of four bodies, a few
milliseconds each [E]), and the batch exists to share tables inside one request. The same body
gives the same bytes, so a cache in front of the server may use the body as its key; the server
does not.

### 18.5 Authentication and exposure

Unchanged from the existing server and stated the same way: there is no authentication, the
listener binds to `127.0.0.1`, and the server must not be exposed to untrusted networks. The
limits above bound one request, not a crowd: a client can hold every permit for the length of the
timeout, exactly as with `solve`. The server has no CORS layer. It logs method, path, status and
latency and never bodies (a print can be private). The server is part of the AGPL program, so
the source-offer duty of the AGPL for network users applies to operators who modify it, as it
does for the existing endpoints; that is not legal advice.

### 18.6 Implementation notes

The batch engine is one public function of the library, `run_batch(input, output, limits,
deadline)`, with a `Limits` value that the command fills generously (`--max-memory`) and the
server fills from the table above. Because both call it, the bytes are equal by construction,
and a test sends the same body to the router and to the command and compares them. The route code
(`crates/server/src/fixed_frame.rs`, about 200 lines [E]) only checks the content type, takes the
permit, runs the function on `spawn_blocking` and maps its outcome to a status. This function and
`Limits` are the only public items of the module (`AGENTS.md`: default visibility is
`pub(crate)`). The pages `docs/api.md` and `crates/server/README.md` get the rows of 18.1 and the
flags of 18.3.

## 19. Releases, table hashes and the v0.2.0 gate

### 19.1 Table hashes instead of tables (D4)

A release of cubacle ships what it ships today (archives, `SHA256SUMS`, build provenance) and no
distance table. For every frame and print definition whose table is meant as a reference, the
release publishes the hash of the table, in two places that must agree:

1. **A small checked-in file**, `tests/fixtures/fixed_frame/table-hashes.json`, read by the
   opt-in test `fixed_frame_full` (section 10.4) and attached to the release as an asset. It is
   data, not output of a solver test, and is added once and changed only with a decision. The
   first version holds one table, the full-state table (the hash and the histogram are the
   measured values of section 3; `prints` is filled by the implementation with the
   `printSha256` of every fixture print that has this layout, which this paper does not state).
   **Historical candidate only:** the example below preserves the withdrawn report for audit.
   It must not be copied into the release fixture until recomputed and corrected (section 13.1):

   ```json
   {"formatVersion":1,"frame":"x=R,y=U,z=F;quarter=-90@outward","poseIndex":"pose24.v1","key":"key.v1","tableEncoding":"u8-dense-v1","tables":[{"goal":"reference","layout":"reference pose of body b is 3b","keyBodies":[0,1,2,3,4,5,6],"keys":88179840,"histogram":[1,18,243,2874,28000,205416,1168516,5402628,20776176,45391616,15154409,49943],"eccentricity":11,"tableSha256":"caf69d83df6fd4c42a899cfd55c664bca87da01408a5aa4973c4eee6633825d9","prints":[]}]}
   ```

2. **The release notes** of the tag, in a block "Fixed-frame tables" written by the release
   workflow from the file (frame string, `key.v1`, `u8-dense-v1`, layout, number of keys, hash),
   next to the Breaking changes block of section 17.2. Once verified, the hash is a function of the problem only
   (section 7.4), so it does not change from release to release unless the frame, the key or the
   encoding changes, and then the format version changes.

### 19.2 How to generate the table, and what it costs

```sh
cubacle fixed-frame table --print tests/fixtures/fixed_frame/blank.print.json --full --out full.u8
sha256sum full.u8
```

`table` prints the goal record on one line, including the complete 24-cell reference `display`.
The line below is a historical, unverified excerpt with that display omitted (not a complete
schema-valid or accepted proof record; section 13.1);
the reproduction must compare the newly computed hash for all 88,179,840 bytes of `full.u8`
with separately verified acceptance data, rather than assume that this old candidate matches:

```json
{"record":"goal","goalId":"reference","trackedBodies":[0,1,2,3,4,5,6,7],"keys":88179840,"goalKeys":1,"deadRules":[],"histogram":[1,18,243,2874,28000,205416,1168516,5402628,20776176,45391616,15154409,49943],"eccentricity":11,"tableSha256":"caf69d83df6fd4c42a899cfd55c664bca87da01408a5aa4973c4eee6633825d9"}
```

(The file name `blank.print.json` is the fixture planned in row 2 of section 15. The displayed
record is the historical scratch report, not a verified cubacle output. Its histogram is
contradictory; the claimed hash comparison above remains a reproduction task, not a result.)

| | Value | Mark |
| --- | --- | --- |
| Time, Rust, release, one thread | 1 to 4 s for the search, about 0.3 s more for the SHA-256 | [E] |
| Peak memory | about 90 MB: the 84 MB table, a frontier by scan per depth | [E] |
| Disk | 84 MB for `full.u8`, deleted after the check | |
| Measured | the throwaway C program, no tuning, heavily loaded two-core machine: 5 min 20 s of CPU | [M] |

The numbers of the first two rows are estimates until the pull request of row 5 measures them on
a quiet machine and replaces them. The opt-in CI job has a 30-minute limit like the other full
jobs, so even the untuned C figure would pass.

Nobody has to download or trust the table: a consumer regenerates it, or runs an independent
checker in mode R (section 7.4), and compares the hash with the notes.

### 19.3 The release gate for v0.2.0 (D7)

The owner cuts the tag (`AGENTS.md`: no tag unless the owner asks) from a commit of `develop` for
which all of the following hold.

1. **All required CI jobs are green, `ci-result` included.** The jobs today are `rust`,
   `five-full`, `four-full`, `msrv`, `docs`, `notes` and `spelling`. The aggregate `ci-result`
   and the classifier job `changes` are on the branch `ci/skip-rust-for-docs-only` (commit
   `ec7e3d9`) and not yet on `develop` [C]; the gate presumes that branch is merged. The two jobs
   of this work, `fixed-frame-golden` and `fixed-frame-full`, are added to the `needs` of
   `ci-result` by row 8 of section 15, and they belong to the set that a documentation-only
   change skips, like `rust` and `four-full`.
2. **The `rust` job is repaired first.** It times out on `develop` today (30-minute limit [C]).
   A separate investigation exists. This paper does not repeat it; the gate only depends on its
   result, and no tag is cut while `rust` is red.
3. **The Cubistry-reference golden tests are green in CI.** The values are the reviews of
   Cubistry reproduced by an independent scratch implementation ([R] and [M], section 3); the
   test names are proposals.

   | Case | Expected | Test |
   | --- | --- | --- |
   | Three corners (letter upper right, tens lower left, units lower right, upright) | 9,072 keys, eccentricity 6, histogram 1, 15, 177, 1397, 4981, 2497, 4 | `golden_three_bodies` |
   | 378 sources with the letter in place | eccentricity 5, histogram 1, 6, 29, 118, 180, 44 | `golden_letter_in_place` |
   | Blank print, blank corner in any twist plus three glyphs | 136,080 keys, 3 goal keys, eccentricity 6, histogram 3, 45, 594, 6015, 39849, 79143, 10431 | `golden_blank_print` |
   | 810 sources with blank and letter in place | eccentricity 6, histogram 3, 9, 27, 99, 273, 363, 36 | `golden_blank_premise` |
   | Counterexample `U D2 B D'` to blank/H/34 | distance 6, key 114083, lex-least shortest word `U B R2 B2 U' L`; `R F' D F2 R' D` replays to the display too | `golden_counterexample` |

   All five are k <= 4 and need well under a second [E], so they are ordinary tests of the
   default suite and also run alone in `fixed-frame-golden`. They are non-slow checks, not an
   opt-in job. The heavy goldens (k = 5 and k = 6, the 180-display batch, the full-table hash of
   section 19.1) run in `fixed-frame-full`, an opt-in job in the sense of `four-full`: it is not
   in the default suite, but it is in `ci-result`, so it must be green for the tag as well, and
   the hash printed in the release notes is checked by a machine.
4. **The release pull request** (row 10) sets version 0.2.0 in both crates, adds the Breaking
   changes block (section 17.2) and the Fixed-frame tables block (section 19.1) to the release
   notes, and is merged before the tag.

The checkers (D5) are not part of the gate: they pin the tag after it exists.

## 20. Open questions

Only what the decisions D1 to D7 leave open.

1. **REST default of `/v1/solve`.** D3 names `solve --json`. This paper makes `POST /v1/solve`
   follow it, because both are written by one function and the documentation promises equal
   bytes (section 6). If the owner wants the server to stay on format 1 until clients have moved,
   it is a one-line default in the request reader, at the price of the two bodies no longer being
   equal.
2. **Group 3 in the Cubistry SPEC.** The Flix and Julia checkers cannot read a fixed-frame
   certificate until Cubistry's coordinator accepts a Group 3 and its corpus (row C1 of
   section 15). That is Cubistry's decision, not cubacle's.
3. **REST limits.** The numbers of section 18.3 are proposals; the server pull request
   measures memory and time per request and may change them.
4. **Output and the AGPL.** Whether certificates and tables carry no AGPL obligation for the
   consumer is assumed (section 12), not decided; it is the owner's call and not legal advice.
