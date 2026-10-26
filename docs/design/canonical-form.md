---
title: "Canonical form"
description: "Concept paper: one canonical Orbit64 token per pattern, invariant under the 48 cube symmetries and any renaming of colors, for 2x2x2 to 7x7x7."
weight: 50
---

# Canonical form

Status: in the library, in `cubacle patterns identify` (the `canonical:` line), in
`cubacle canon` ([Canonical token](../cli/canon.md)) and in the REST field `canonical` of
`/v1/convert` and `/v1/random` ([REST API](../api.md#the-canonical-field)). The code is
`src/canonical/mod.rs`, its tests `src/canonical/tests.rs`. Numbers
marked **[M]** were measured for this paper (release build, one Apple Silicon
core, `nice`); every command below was run.

The reference implementation of the canonical form is flix-orbit64 (tag v0.9.0, Apache-2.0,
`FORMAT.md`, sections "Canonical form" and "Facelet geometry"); the rule text lives
there and this Rust code follows it. This paper explains the rule and records the
Rust measurements. The shared conformance suite is `tests/canonical_vectors.txt`
(17 token vectors, 76 facelet vectors and the pinned failures of the reference),
checked by `canonical::tests::every_reference_vector_gives_the_reference_token`
and `every_reference_failure_is_an_error`; Rust and the reference agree on all of them.

## 1. Goal

A pattern is a color state: pieces of one color are interchangeable. Its
identifier must therefore be the same for a cube, its rotated and mirrored image,
and a recolored copy. The identifier chosen is an Orbit64 state token (widths 5,
12, 27, 43, 66 and 90 characters for the 2x2x2 to the 7x7x7), because it is exact,
short, ordered and convertible back to a cube.

`cubacle::canonical::canonical(&cube) -> Result<Token, CanonError>` returns it; `canonical_text` reads an Orbit64 token,
facelets (`URFDLB`) or colors (`WRGYOB`); `canonical_moves` takes a move
sequence on the solved cube. `Token::state()` gives the cube back. The code is its
own module because `patterns` may not name `notation` (`tests/architecture_test.rs`);
`canonical` sits above both.

## 2. Definition

Let `S` be the sticker string of a cube (`6 n^2` colors, `U R F D L B` = 0 to 5).

1. **Orbit minimum.** `C(S)` is the least string among the 48 position
   symmetries of `S` (rotations and mirrors), each with its colors renamed in
   order of first appearance. The order is lexicographic on the numbers 0 to 5.
   This is the form the detector already uses for exact library recognition
   (`patterns/detector/canon.rs`); `C(S) = C(S')` exactly when `S'` is `S`
   turned, mirrored and recolored in any way.
2. **Standard colors.** `C(S)` has arbitrary color names. They are replaced so
   that the cube is in the Orbit64 scheme:
   - odd cube: the color in the center of face `f` becomes `f`, so the six fixed
     centers are home and the frame digit of the token is 0;
   - even cube: there are no centers, so the three colors of the corner in the
     URF slot become `U R F` in turning order and the three colors that share no
     corner with them (the opposite colors) become `D L B`.
3. **Token.** The result is encoded by `orbit64::encode`. A state that is no
   cube (twisted corner, parity) is an error. There is no mirroring step and no
   second attempt: the reachable states are closed under mirroring, so a mirror
   image renames to a cube as well (the reference proved the retry unreachable;
   the Rust spike had it and it never fired in the tests).

The token is a function of `C(S)` only, so it is invariant. It is injective on
classes because step 2 only recolors, which keeps the state in
its class: `C(decode(token)) = C(S)`, checked in the tests. The solved cube of
every size is the token of `A` repeated (`AAAAA`, `AAAAAAAAAAAA`, ...).

The token is **not** the numerically smallest token of its class, and the order
of step 1 is on stickers, not on tokens. That keeps the cost at 48 images and
one encoding (section 6). Choosing the smallest token would need an encoding per
candidate: 48 images times 24 standard colorings on an even cube.

## 3. Equal-colored pieces

From the 4x4x4 on, a color state belongs to many piece states. Orbit64 already
reads colors, not pieces, so no representative has to be picked:

- **Wings.** Orbit64 reads a wing from the ordered pair of colors in its slot
  (reference sticker first), one table of 24 wings per depth, so wings are not
  ambiguous and a wing of one depth never trades with another. This agrees with
  the note "Equal pieces are matched fixed points first".
- **Centers.** An orbit of 24 centers holds each color four times and is stored
  as five combination ranks of colors. Swapping two same-colored centers does not
  change the token.
- **Midges, corners.** Distinct by their colors.

The ambiguity therefore lives only in piece-level representations: sticker
permutations that follow pieces through moves, and `cycles_of_state`, which pairs
equal pieces by index. They are not needed to make a token.

Checked **[M]** with two kinds of pairs: the center-only swaps of
`tests/same_color_centers_test.rs` (4x4x4 to 6x6x6), and half-turn words found by
a random search that end in the same colors on different pieces (4x4x4 to 7x7x7,
for example `2R2 2F2` and `2F2 2L2 2R2 2B2 2L2 2B2`). Their pieces differ
(sticker permutations), their colors and tokens are equal.

## 4. Odd and even cubes

- **Odd (3, 5, 7).** The six fixed centers name the colors. Whole-cube rotations
  are the 24 frames of Orbit64; the canonical token has frame 0. A cube with its
  centers swapped (reachable by slice turns of the 3x3x3) is recolored into a
  standard cube first, which is why slice scrambles work.
- **Even (2, 4, 6).** No centers, no frame digit: a rotated solved cube has 24
  different Orbit64 tokens. The URF corner rule picks one of the 24 colorings.
  The 2x2x2 has no wings or centers at all; the same rule applies.

## 5. Results

Run with `cargo test --release --lib canonical::tests` (all pass).

| Property | Result **[M]** |
| --- | --- |
| Idempotent: `canonical(token.state()) = token`, and the state is in standard form | 40 (2x2x2, 3x3x3), 12 (4x4x4, 5x5x5) and 6 (6x6x6, 7x7x7) random states |
| Same token under 48 symmetries x 3 color renamings (identity and two random) | the same states, 144 variants each |
| Different color states, different tokens (token <-> detector form is one to one, and `C(decode(token)) = C(S)`) | 4 005 states 3x3x3 (3 684 classes), 4 001 on 2x2x2 (3 177), 1 507 on 4x4x4 (1 434), 1 509 on 5x5x5 (1 454), 409 on 6x6x6 (400), 406 on 7x7x7 (400), library patterns included |
| Every library pattern, every size it has | token, and `detect` finds it again from the token's cube |
| Round trip text, facelets, colors, Orbit64 token, move sequence | same token |
| Piece states of one color state | 4x4x4 to 7x7x7, one token |

Exhaustive on the 2x2x2: `cargo test --release --lib burnside -- --ignored
--nocapture` enumerates all 3 674 160 states, takes **[M]** 30 to 50 s, and finds
77 802 tokens. Burnside's count from the symmetry group order of each state,
`sum(order) / 48`, is also 77 802.

Token widths and time per call (`cargo test --release --lib
canonical::tests::numbers -- --ignored --nocapture`, 200 random 40-move cubes per
size):

| Size | Width | Time per call |
| --- | --- | --- |
| 2x2x2 | 5 | 6.6 us |
| 3x3x3 | 12 | 11.9 us |
| 4x4x4 | 27 | 21 us |
| 5x5x5 | 43 | 29 us |
| 6x6x6 | 66 | 41 us |
| 7x7x7 | 90 | 53 us |

The 37 library entries (every pattern in every size it has) take 2.5 ms together.

## 6. Complexity

`O(48 * 6 n^2)` for the orbit minimum (48 sticker permutations, cached per
size), `O(n^2)` for the renaming, and one Orbit64 encoding (its wing and center
loops are small constants times `n^2`), once or twice. Measured above.

## 7. Limits and open problems

- **Mirror images are one class.** A chiral pattern and its mirror image have the
  same token, because the 48 symmetries include the mirrors. Telling them apart
  would need an order over the 24 rotations only; not tried.
- **Stability.** The token depends on the Orbit64 slot convention
  `orbit64-Nxn-draft@1` (see "Orbit64 slot numbering caveat"), on the sticker
  order `URFDLB`, on the facelet geometry and on the URF corner. Any of
  them changing renames every identifier; a stored identifier should carry the
  convention name.
- **Unreachable states have no token.** Only reachable cubes are encoded; the
  error may be less exact than
  the usual `check` message for a recolored input.
- **Partial states.** A target with don't-care stickers is not a cube state and
  has no token.
- **Sizes above 7** are outside the Orbit64 format.
- **The order is not the token order**, so tokens cannot be sorted into the
  detector's order; sorting by token is stable, though.
- **Do not hash the detector form instead.** It is not a cube and has no
  inverse.

## 8. Recommendation

| Use | Rating (1 to 5) | Reason |
| --- | --- | --- |
| Library use: store the token with every pattern and look patterns up by it | 5 | Same discrimination as the detector form, short, printable and convertible back; the detector keeps its in-memory form (2.5 ms for the library). |
| REST: a `canonical` field on the state descriptions | 4 | Cheap (under 60 us) and stateless. Built on `/v1/convert` and `/v1/random`. |
| CLI `cubacle canon <cube>` | 4 | One token per line, any notation; useful in scripts. Built. |
| Use inside searches (transposition keys) | 2 | The search state space is not quotiented by colors; 20 to 60 us per call is slow there. |
| Identifier for chiral patterns | 2 | See "Mirror images" above. |
