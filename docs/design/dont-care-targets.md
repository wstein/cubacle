---
title: "Don't-care targets"
description: "Concept paper: targets for solve --to where some stickers are unconstrained (cross, F2L, centres only, free parts), per engine, with costs, delivery plan and open decisions."
weight: 40
---

# Don't-care targets

Status: concept, awaiting owner approval. No code has changed. Numbers marked
**[E]** are estimates, not measurements; numbers marked **[M]** were run for this
paper. Only ideas from the study of a GPL sticker solver were used (see the note
"A study of stickersolve shows what to adopt and what not"); no code or text.

## 1. Goal and user stories

`solve --to` today needs a complete target. A person often wants less:

1. **Cross only.** "Get the white cross from this scramble, shortest." The other
   pieces may land anywhere.
2. **F2L.** Cross plus the four first-layer pairs, last layer free.
3. **Centres only (4x4 and up).** Solve the centres, leave edges and corners
   alone. This is the first reduction stage of the 4x4 and 5x5 engines made
   available as a goal.
4. **A pattern with free parts.** "Top face all white, rest don't care", or a
   chessboard on two faces only.
5. **Moved centres (4x4 and up).** Today `--to` refuses them [M]:

   ```
   $ cubacle solve --size 4 --to "r" "R U"
   error: the target has centres out of place; the solvers can only reach a target
   whose centres are on their own faces (...)
   ```

   The cause is that the relative-state method needs every sticker to have its own
   identity, and equal centres have none. A colour target does not need identities:
   "the up face shows white" is a statement about colours. Story 5 is therefore
   solved by the same model as stories 1 to 4.

Non-goals: partial *start* states (the start stays complete, as decided), and
optimal solutions on sizes where the engine is not optimal today.

## 2. Input syntax

Decided (notes "Don't-care stickers are written X or named as a subset" and
"A partial target is a map from slots to allowed colours"): `X` marks a sticker
that does not matter; `?` is accepted on input and `X` is written back; only
uppercase `X` counts because lowercase `x` is the cube rotation; an Orbit64 token
cannot express it. Today both are rejected [M]:

```
$ cubacle solve --to "UUUUUUUUU??" "R U"
error: --to: not a cube (character '?' at 9 is neither a face (URFDLB), ...
```

Forms and how each is told apart from the others (the reader already tries named
targets first, then cube text, then a move expression; see `read_target` in
`src/main.rs` and `named_target` in `src/notation/named.rs`):

| Form | Example (3x3x3) | Parsed as |
|---|---|---|
| Facelets, URFDLB | `UUUUUUUUUXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` | the up face solved, 45 free stickers |
| Colours, scheme letters | `WWWWWWWWWXXX...` | the same, read with `--scheme` |
| Named subset | `cross`, `f2l`, `layer-d`, `centres` | a fixed X-mask over the solved cube |
| Move expression | `R U` | unchanged: applied to the solved cube |
| Mask plus base | `cross+R U` or `f2l@R U` | the subset mask over the cube that the expression gives |

Unambiguity rules:

- `X` is not a face letter (URFDLB) and not a colour letter (WRGYOB), so a string
  with an `X` and otherwise only face or colour letters can never also be a valid
  complete cube. A move expression with a literal `X` is not valid notation either.
  So a text is partial exactly when it has `X`/`?` as a sticker character *and*
  has the length of a cube string. No new flag is needed.
- A string mixing face letters and colour letters is rejected as today (error
  names the first character that fits neither alphabet).
- Names are taken only when the whole text is the name (as `superflip` is). A
  suffix form `name+moves` is a separate, explicit syntax, so `cross` alone means
  "the solved cross, rest free", not "cross of some scramble".
- Length decides the size as today (`--size` or the length of the start).
- Orbit64 tokens stay complete; a partial Orbit64 is an error that says so.

REST: the `target` field of `POST /v1/solve` takes the same text; no new field.
The response gets an optional `"reached"` field (the full state the solution
produced, as facelets) so a client can see what the free stickers became. The
field is absent for complete targets, so existing responses stay byte-identical.

## 3. Model

A target is a map from slot (a sticker position) to the set of allowed colours:
a one-element set for a cared sticker, all six for `X`. This is the decided
representation. A subset of colours smaller than all but larger than one (for
example "white or yellow on U": useful for OLL-style goals) costs nothing extra
and is left for a later step, but the model carries it from the start.

The goal test is a predicate on the state: for every slot, the sticker on it is in
the set. The start is never greyed.

### Why `T^-1 * S` no longer works

The relative-state method (note "A target is solved as a relative state") gives
each sticker the identity of its place on the solved cube, builds `R = T^-1 S` and
solves `R` as an ordinary cube. It needs `T` to be one definite permutation. A
partial target is a *set* of definite targets `{T_1 .. T_n}` (its completions).
Solving to the set means finding `y` with `S*y` in the set; `R_i = T_i^-1 S` differs
per completion and the best `y` is the shortest over all `i`. Enumerating
completions is exact but too many for most masks (with the cross fixed, the other
pieces still have 8! * 3^7 * 8! * 2^8 / 2 completions, about 4.6e14 [M by arithmetic]), so the engines below replace the
reduction to a plain solve by a goal-set search. The fully constrained case still
is the relative-state method, byte for byte (section 8).

### Replacement per engine

| Engine | Complete target today | Partial target |
|---|---|---|
| 2x2x2 | relative state, any orientation | **Exact.** Breadth-first search from the goal *set* (multi-source) over the 3,674,160 states gives exact distances to the nearest goal; the table is built per target, so no stored table changes. |
| 3x3x3 min2phase | relative state, two-phase | Two routes. (a) **Few completions** (up to a threshold, for example 24 for a free whole-cube rotation, or a few free corners' twists): run min2phase once per completion, keep the shortest. Exact up to its usual two-phase caveat. (b) **Many completions** (cross, F2L, a face): the pattern search (`src/engines/pattern`) with projection tables as relaxed bounds, IDA* on the goal predicate. Exact for cross; best-effort with a node limit for F2L. |
| 4x4x4, 5x5x5 | relative state; centres must be home | The engine's own stages are goals: centres only = stage one and two; centres + edges paired = reduction without the 3x3 finish. The stages already stop where a partial target stops. Other masks: best-effort search (below). |
| 6x6x6, 7x7x7 (dwalton) | same | Stage-wise like 4x4/5x5 where the stage is a goal; otherwise refuse with a message naming the supported subsets. |
| Pattern shortcut (4x4 and up) | tries known patterns, keeps the shorter | Only for complete targets that equal a library pattern; for a partial one it is skipped, not made to guess. |

The reduction engines keep reaching the 3x3x3 solver only through the `Solver`
trait with an injected instance (the dependency rule in AGENTS.md); nothing in
this paper changes that.

## 4. Search design

### 4.1 Goal test and heuristic

IDA* tests the slot predicate instead of "equals T". For the heuristic, the
decided design applies: the heuristic is the maximum of several relaxed pattern
tables over the slots that have a cared sticker, and each table is seeded from
*every* allowed assignment so the bound never exceeds the true distance
(admissible). A table is the distance to the nearest allowed state of a projection
(a subset of pieces); with the "layer set projection" idea of the pattern engine
(note "Pattern states are projections onto layer sets") the projection of a
partial target onto the pieces it cares about is again a smaller cube. With nothing
cared there is nothing to look up: the bound is zero, which is correct and slow,
so such a search takes a depth and node limit and says so.

### 4.2 What becomes free

- **Symmetry.** A search symmetry (a whole-cube rotation or reflection) is usable
  only if it maps the *mask* to itself. The cross mask has 8 such symmetries
  (those that fix the cross face) [E, count of the stabiliser]; a mask on one
  face only has the 4-fold rotation plus mirrors. Unconstrained targets keep all
  48. Tables built under a symmetry are keyed by mask class, not shared across
  masks.
- **Orbits.** If a whole orbit is free (all corners, all wings of a 4x4), its
  coordinates drop out of the state, shrinking tables. Centres-only on a 4x4 is
  24 centres in 6 colours: 24! / (4!^6) = 2,308,743,493,056 arrangements [M by
  arithmetic], the number the 4x4 centre stages already face, so the existing
  stage tables apply.
- **Equal centres.** Under a colour goal there is no identity to track, which
  lifts today's centre limit.

### 4.3 Completeness, optimality, what is exact

| Engine / mask | Complete | Shortest | Remark |
|---|---|---|---|
| 2x2x2, any mask | yes | **yes** | multi-source BFS |
| 3x3x3 cross | yes | **yes** | IDA* with cross tables, depth <= 8 in the face-turn metric [E from the known cross bound] |
| 3x3x3 few completions | yes | shortest over completions of the two-phase answers, i.e. not guaranteed optimal | same caveat as plain `solve` |
| 3x3x3 F2L and larger | yes within limit | best-effort | node limit, reports when it stopped |
| 4x4 / 5x5 stage goals | yes | not guaranteed (as today) | the stage solvers are not optimal |
| any with no cared slot | trivial | zero moves | the empty answer is correct |

No path may claim optimality it does not have: the result message says "shortest"
only where the table above says yes, "best found in N nodes" otherwise.

### 4.4 Cost and feasible sizes

- 2x2: building the BFS table is one pass over 3.67 million states; **[E] tens of
  milliseconds in release**, memory about 4 MB per target; not measured.
- 3x3 cross: 190,080 states (12P4 * 2^4) [M by arithmetic]; table build **[E]
  well under a second**.
- 3x3 cross + 1 pair: 12P5 * 2^5 * 24 = 72,990,720 states [M by arithmetic], about
  37 MB at 4 bits: possible as one table, but F2L as a whole is not, so the plan uses cross (edges) plus pair-wise partial
  tables (corner-edge, two-pieces) as separate relaxed bounds, each a few MB.
  Search time for F2L **[E] seconds to minutes**, unmeasured, hence best-effort
  with a node limit.
- 4x4 centres only: the existing centre stage tables (about 12 MB for the 4x4
  engine, from `solve` start-up output [M]) are reused.
- 5x5 and above: only stage goals, using the existing tables and the 5x5 cache.
- Memory: every table respects the 1.5 GB cap (project figure; the 5x5 is already
  above its target, see note "5x5 memory above target"). Per-target tables are
  built under a hard budget; exceeding it falls back to a looser bound, never to
  an allocation failure.

## 5. Interactions

- **Shortest-first policy.** Never lengthen default output. All defaults stay
  byte-identical: no `X` in the target means the old code path, not a general
  predicate that happens to give the same answer. A partial target may find a
  *shorter* answer than a complete target would; it is never a way to get a longer
  one. Where a route gives more than one answer, the shortest wins.
- **Fingertrick ranking.** Only orders solutions of the shortest length, after the
  search (note "Fingertrick ranking only orders solutions of the shortest
  length"). Partial targets have many more equal-length solutions, so ranking
  gains more here, but it comes after enumeration and changes nothing in this
  design.
- **`--moves` and `--all`** (restricted move sets and enumeration of all
  solutions, being built by another agent). The goal predicate is independent of
  the alphabet: a restricted alphabet makes some partial goals unreachable (a
  cross with only `R U`, say), so the error "unreachable" (section 6) must be
  returned by the engine, not assumed. `--all` with a free mask can produce a huge
  list; it needs the stop count that the study already adopted, and a default
  count when the mask has many free slots. This paper adds no flag; the two
  options only need to take a goal predicate rather than a state. Agree the
  interface with that agent before step 3 of the plan.
- **MSRV 1.85.** Nothing here needs newer features: bit sets over six colours fit
  a `u8`; masks are `Vec<u8>` or arrays.
- **Memory cap 1.5 GB.** See section 4.4.
- **Server.** Partial searches are compute-heavy: they run on the blocking pool
  like all solves, never on async threads; the server stays stateless (per request
  tables are built and dropped, or served from a bounded cache keyed by mask
  class, which is a deliberate non-statefulness question for the owner).

## 6. API, CLI and REST changes, errors

CLI: `--to` accepts the forms of section 2. Optional later: `--show-free` prints
the state the free stickers ended in. Without it the output is the moves, as now.

Library: a `Target` type (`slot -> colour set`) in the puzzle layer; `Facelets`
stays complete. A new trait method with a default, so existing engines compile
unchanged:

```
fn solve_to(&self, start: &Facelets, target: &Target, options: &SolveOptions) -> SolveResult
```

The default returns the failure "this engine cannot solve partial targets";
engines override it as they gain support. A complete `Target` calls the existing
`solve`/relative-state code unchanged.

REST: same `target` text; errors use the existing `bad_input` shape.

Error cases (each with a distinct, tested message):

- **Contradictory.** Two constraints that no state can meet: a facelet string
  with the same colour on two stickers of one piece, or a piece with colours no
  piece has (reuses `ImpossiblePiece`). Reported before searching.
- **Unreachable.** Valid-looking constraints the start can never reach: a parity
  or colour-count conflict (for example a cross with one edge in an impossible
  orientation sum when the whole edge orbit is constrained), or the alphabet
  cannot reach it (`--moves`). Reported when the table proves it; for best-effort
  engines "not found within the limit" is a different message from "proved
  unreachable".
- **Ambiguous.** Text that is both a valid named subset and a valid move
  expression, or a mask of the wrong length; resolved by the rule "whole-text name
  wins" already used for `superflip`, with the error naming the choices.
- **Unsupported.** A mask an engine has no stage or table for (6x6 arbitrary
  masks): says which subsets are supported.

## 7. Tests

- **Property test.** For random starts and random masks (random subset of slots,
  random colours drawn from a real completion so the goal is reachable), every
  solution returned makes every constrained slot match when replayed on the start.
  Sizes 2 and 3 in the normal run; 4 under `--release` with a small sample.
- **Equivalence.** A fully constrained mask gives *exactly* the old moves, byte for
  byte, including on the recorded fixtures; fixtures in `tests/fixtures/` are not
  edited.
- **Monotonicity.** Freeing more slots never gives a longer shortest solution
  (checked on 2x2 and the 3x3 cross where the shortest is exact).
- **Exactness.** 2x2 multi-source BFS distance equals the brute-force minimum over
  completions on small masks.
- **Error cases.** One test per case in section 6.
- **Parser.** Round-trips: `X` is written back for `?`; lowercase `x` is not
  accepted as a sticker; Orbit64 with `X` is refused.
- **Server.** `target` with `X` returns the same solution as the library; response
  without `X` is byte-identical to the stored current one.

## 8. Delivery plan

Each step is one atomic commit that builds and passes the three checks. Effort is
in focused working hours, **[E]**.

1. `feat(puzzle): Target type with slot colour sets` and the parser for `X`/`?`
   with the full-constraint conversion to `Facelets`; refusal elsewhere. 4 h.
2. `refactor(cli): read_target returns a Target`; complete targets keep the old
   path; byte-identical tests added first. 3 h.
3. `feat(engines): solve_to on the Solver trait with a default error`. 2 h. (Agree
   the shape with the `--moves`/`--all` work first.)
4. `feat(two): partial targets by multi-source search`. 5 h.
5. `feat(three): partial targets by completion enumeration`. 5 h.
6. `feat(patterns): named subsets cross, layer, centres`. 4 h.
7. `feat(three): cross and F2L partial targets with relaxed tables`. 16 h, with the
   node limit and messages.
8. `feat(four,five): centres-only and colour targets for moved centres`. 12 h per
   engine, including lifting the `CentresMoved` refusal for colour targets only.
9. `feat(server): target with X`, response field `reached`. 3 h.
10. `docs: cli, api, engines pages and notes`. 3 h.

Total **[E]** about 60 h. Steps 1 to 6 give stories 1 (cross on 2x2, partial on
3x3 with few completions) and the syntax; 7 and 8 give stories 2, 3 and 5.

## 9. Risks and open questions

- A relaxed bound that is weak (large free mask) makes IDA* slow; the answer is a
  node limit, not a promise. Real F2L time is unmeasured.
- Per-target table build on every request may be too slow for the server; a cache
  by mask class makes the server stateful in a small way.
- Completion enumeration for the 3x3 is simple but multiplies the cost by the
  number of completions; the threshold needs a measurement.
- On 4x4 and up, centres-only is exact only as far as the stage solvers are; users
  may expect "shortest".
- A colour target on 4x4 is only meaningful if the person's colour scheme is known;
  the existing `--scheme` handling applies, with an unambiguous default.
- The meaning of "cross" on a cube whose centres are not home (after rotation) must
  be defined: by the face the centre shows, or by the position.

## 10. Decisions for the owner

1. **Scope of the first release.** Recommend: 2x2 any mask, 3x3 cross and
   few-completion masks, 4x4 centres only; F2L and 5x5+ in a second release.
2. **Name of the trait method and result field.** Recommend `solve_to` with a
   default error and an optional `reached` field in REST/CLI, absent for complete
   targets.
3. **Colour subsets (more than one allowed colour per slot).** Recommend carry them
   in the model, but do not expose syntax for them in the first release.
4. **Mask-plus-base syntax** (`cross+R U`). Recommend defer; only whole-name
   subsets first.
5. **Optimality claims.** Recommend the message says "shortest" only where proven,
   "best found" otherwise, and never silently falls back.
6. **Server table cache.** Recommend no cache at first (stateless), measure, then
   decide.
7. **Interface with `--moves`/`--all`.** Recommend both take a goal predicate and a
   stop count; settle before step 3.
8. **Moved centres on 4x4+.** Recommend allow them only as colour targets, keep
   refusing complete identity-based targets with moved centres.
