---
title: "Move notations"
description: "WCA, SiGN and SSE move notations."
weight: 12
---

# Move notations

## Move notations

Every command that reads moves (`solve --scramble`, `facelets`, `convert --scramble`, `moves`) reads WCA, SiGN and SSE spellings, token by token and without an option. `solve` (also with `--now`), `scramble` and `moves` write any one with `--notation wca|sign|sse` (default `wca`). The move count and the engines are the same in both: every move counts one. `convert` and `random` print a cube, not moves, so they have no such option.

| What is turned | WCA | SiGN |
| --- | --- | --- |
| The outer layer | `R` | `R` |
| Only layer n (n from 2) | `2R`, `3R` | `2R`, `3R` |
| The two outer layers | `Rw` (or `2Rw`) | `r` (or `2r`) |
| The n outer layers (n from 3) | `3Rw`, `4Rw` | `3r`, `4r` |

A turn ends in nothing (clockwise), `2` or `'`, as before (`3r2`, `u'`). The notations agree on every uppercase spelling and WCA has no lowercase one on big cubes, so no spelling has two meanings and nothing needs a tie-break. If a spelling ever did differ between the two, WCA would win; today none does. Mixed input such as `R u 3Rw 2R` is therefore read as it stands. Output in SiGN writes `u` for the two-layer turn (SiGN omits the 2) and `3u` for three.

#### Layer ranges

SiGN writes the layers a to b of a face as `a-b` before the letter: `2-3r` is the second and third layer from the right, `2-4l2` a half turn of layers 2 to 4 from the left, `1-3r` is `3r`, and `2-2r` is `2R`. A range is expanded when it is read into plain moves, the outer b layers turned together and the outer a-1 layers turned back, so the engines and the move count never see it: `2-3r` is `3Rw R'`, `3-5r` is `5Rw Rw'`, `2-4l2` is `4Lw2 L2`. Layers are counted from the face named, and a is not larger than b. As an extension of this program, which neither source has, the range may be written with `Rw` instead of the lowercase letter: `2-3Rw2` equals `2-3r2`. The tests turn every range of every size from the 2x2x2 to the 7x7x7 and compare it with turning the same layers one by one.

Rejected, with the token in the message: `rw` and `3rw` (a lowercase letter already means a wide turn), `1r` and `0r`, a turn count after the letter that is not 2 or a quote, and ranges that run backwards (`3-2r`), start at 0 (`0-2r`), lack a letter (`2-3`), use a capital without `w` (`2-3R`) or reach a layer the cube lacks. The face letters are `U R F D L B`; the whole-cube rotations `x y z` and the slices `M E S` are read in expressions and by `moves`.

#### Where moves are read and written

Moves are read as WCA, SiGN or SSE, with groups, commutators and ranges, by `solve --scramble`, `convert --scramble` and `facelets`, by `solve --now --solution`, by `moves` (its moves and `--from`), and by the `scramble` field of the REST API. Moves are written in any of the three by `--notation wca|sign|sse` on `solve` (the solution, the `--icons` move lines and the last line; the icon pictures keep WCA letters), `scramble` and `recover` (every move in the explanation and in `Moves now:` and `Fresh solution:`).

```sh
cubacle facelets "r U f'"
cubacle scramble --size 7 -n 6 --notation sign
cubacle solve --size 4 -s "r 2R f' 3u" --notation sign
```

```text
UUUURRURRBBBBBRDDRRDDRDDRFFDLLDLLDBBFFFLFFLFFLLLUUBUUB
2L 2U2 3F' 3U' L U2 2B2 2L'
D' l f2 F u' d
```

The first line is the same cube as `cubacle facelets "Rw U Fw'"`. The scramble is random. A 7x7x7 answer reads `U 3Uw 3Lw' B' 3Rw ...` in WCA and `U 3u 3l' B' 3r ...` in SiGN.

#### SSE (Superset ENG)

SSE is the notation of Werner Randelshofer's cube pages, one page per size (2x2 to 7x7). It is read like the others, token by token, and written with `--notation sse`. Only the notation as the six pages describe it was used (read 2026-10-23); no code from the site. Faces are `U R F L D B`, a turn ends in nothing, `2`, `2'` (the same half turn), `'` or `-` (`R-` is `R'`). Every twist below is expanded into plain moves when it is read, so the engines and the move count never see it.

| Prefix | Twist | Layers turned, counted from the face named (1 is its own layer) | Examples |
| --- | --- | --- | --- |
| none | face | 1 | `R`, `U'`, `F2` |
| `T` | tier | 1 to d, d from 2 (default 2) | `TR`, `T3R'` |
| `N` | numbered layer | layer k (default 2) | `NL`, `N4R` |
| `N` | numbered range | layers a to b | `N2-4R` |
| `V` | void | 2 to d+1, d from 1 (default 2) | `VR`, `V3R` |
| `M` | mid-layer | k layers in the middle (default 1) | `MR`, `M2R` |
| `W` | wide-layer | all but the two outer faces, `M` of n-2 layers | `WR` |
| `S` | slice | 1 to d from the face and from the opposite face (default 1), both the same way round the axis | `SR` (is `R L'`), `S2R` |
| `S` | slice range | a to b from both faces | `S2-4R` |
| `C` | cube rotation | all | `CR`, `CU'` |

Moves may be written together. A word that is not one move is cut after a `'` that is followed by a letter or digit (`R'U'` is `R' U'`), after a `2` that follows a face letter and precedes an SSE prefix letter (`F2NR` is `F2 NR`), and after the face letter of an SSE move that is followed by a capital letter (`NBL` is `NB L`); every piece must be a valid move, otherwise the word is refused with its original error. `F2R` is not cut because it could be `F2 R` or `F 2R`, and `R22` has no cut; both stay errors. A word that already parses is never cut.

What exists on each size follows the six pages, and anything else is refused with the position and the reason:

| Size | Face | `T` | `M` | `W` | `S` (examples) | `N`, `V`, `N` ranges | `C` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2x2x2 | yes | no (a 2-layer tier is the cube) | no | no | no (would turn every layer) | no | yes |
| 3x3x3 | yes | `TR` only | `MR` | no (use `MR`) | `SR` | no | yes |
| 4x4x4 | yes | `TR`, `T3R` | `MR`, `M2R` | `WR` (is `M2R`) | `SR`, `S2-2R` | no | yes |
| 5x5x5 | yes | up to `T4R` | up to `M3R` | `WR` (is `M3R`) | `SR`, `S2R`, `S2-3R` | yes | yes |
| 6x6x6 | yes | up to `T5R` | up to `M4R` | `WR` (is `M4R`) | `S2R`, `S2-4R` | yes | yes |
| 7x7x7 | yes | up to `T6R` | up to `M5R` | `WR` (is `M5R`) | `S3R`, `S2-5R` | yes | yes |

The same moves in the three notations (each row was run with `cubacle moves --size N --notation wca|sign|sse`):

| Size | WCA | SiGN | SSE |
| --- | --- | --- | --- |
| 2x2x2 | `R'` | `R'` | `R'` |
| 3x3x3 | `Rw` | `r` | `TR` |
| 3x3x3 | `2L'` | `2L'` | `ML'` |
| 4x4x4 | `3Rw'` | `3r'` | `T3R'` |
| 4x4x4 | `2R`, `3R` | `2R`, `3R` | `MR`, `ML'` |
| 5x5x5 | `2R`, `3R`, `4R2` | `2R`, `3R`, `4R2` | `NR`, `MR`, `N4R2` |
| 6x6x6 | `3R`, `4R` | `3R`, `4R` | `MR`, `N4R` |
| 7x7x7 | `2R`, `4R` | `2R`, `4R` | `NR`, `MR` |
| 7x7x7 | `2-4Rw` | `2-4r` | `T4R R'` (`moves` writes plain moves; `--optimize ranges` writes `V3R`) |

```sh
cubacle moves --size 7 --notation wca "TR T3R NL N4R V3U MR WF N2-5R'"
cubacle moves --size 5 "SR S2R S2-3R CR"
cubacle moves --size 3 "x M S' E2"
cubacle facelets --size 3 "MR SU CF2 R-"
cubacle facelets --size 3 "2R U D' z2 R'"
cubacle solve --size 4 -s "R U 2R 2U' f 2L' B2 D" --notation sse --optimize ranges
cubacle facelets --size 4 "R N2R"
```

```text
Rw 3Rw 2L 4R 4Uw U' 4R 6Fw F' 5Rw' R
R L' Rw Lw' 4Rw R' 4Rw L'
Rw L' 2L 2F' 2D2
DDLBBBDDLFLFDLDFLFRRDFDBRRDUURFFFUURBUBRRRBUBULLFUBULL
DDLBBBDDLFLFDLDFLFRRDFDBRRDUURFFFUURBUBRRRBUBULLFUBULL
D' B2 ML TF' MU MR' U' R'
error: scramble is not valid move notation (character 2: invalid move "N2R" (N twists exist from the 5x5 on; use M, W or T on smaller cubes))
```

The two `facelets` lines print the same cube, as do the spellings. The `solve` line also prints `moves 11` and `compressed moves 8` on stderr.

How the three notations are told apart, and the decisions that were needed:

- A word is SSE when it starts with one of `T N V M S W C`, has optional numbers, and then a face letter. `M`, `M2`, `S'`, `S2`, `E` and `x y z` have no face letter after the prefix, so they keep their WCA meaning (the middle slices of an odd cube and the whole-cube rotations). `SR` and `MR` were not WCA spellings before, so nothing changed meaning. SSE uses capitals only; SiGN's lowercase letters and ranges that start with a digit (`2-3r`) cannot be mistaken for it.
- `(R U F)3`, `(R U)'`, `(R U)*3` and `[A, B]` are the groups that already existed; SSE's `(R U F)3` is the same.
- `S` takes the layers from both faces as one set, turned the same way round the axis: `SR` is `R L'`, not `R L`. Where the two ends of a slice range meet, the layer is turned once (`S2-3R` on a 5x5 is layers 2 to 4). `S2R` is the layers 1 and 2 of both faces, not layer 2 alone; this is read off the 4x4 page, which lists `S2-2R` but no `S2R`.
- The pages write a twist that "is equivalent" in a shorter form (`S2-4F2` is written `SF2`). Such pairs differ by a rotation of the whole cube, which changes the state, so this program reads each as the layers it names and does not swap them. On the other hand, a range that starts at layer 1 or ends at the last layer, a one-layer range, and the forms the pages call never used (`T1R`, `T2R`, `N2R`, `M1R`, `V2R`) are read as the layers they name, so non-canonical spellings are accepted and written back in the canonical one.
- On a cube with an even number of layers the middle layers closer to the face named are meant: `MR` is layer 2 of a 4x4 and layer 3 of a 6x6 counted from `R`, and `M3R` on a 6x6 is layers 2 to 4. The 4x4 has no `N`, so layer 3 from `R` is written from the other side, `ML'`.
- A void twist `V3R` and a numbered range `N2-4R` name the same layers (both appear in the 7x7 page). The writer keeps the shorter form in this order: a tier (layers from layer 1, or to the last layer from the opposite face, turned the other way), `WR`, a void twist, `MR`/`M2R` in the middle, `N`, then `N2-4R`. So `N2-4R` is read, `V3R` is written.
- Writing is per turn. `R L'` stays two moves; `S` is only read. `--optimize ranges` writes the merged blocks (`VR`, `M2R`, `N3-5R`, `T4L'`).
- The pages also describe permutation and orientation cycles of individual cubies. They are not supported: such text is rejected with the position of the first token that is not a move (`(UFR,URB) R` gives `character 1: invalid move "UFR"`).
- `parse_moves` (the library function with no cube size) reads the twists that do not depend on the size (`T`, `N`, `V`, `-`); `M`, `W`, `S` and `C` need the size and are read by `parse_expression`, which every command uses.

Sources read for these rules (2026-10-23):

- WCA Regulations, Article 12a2 (<https://www.worldcubeassociation.org/regulations/#article-12-notation>): `nRw` moves the n outer layers, `Rw` is `2Rw`, `1Rw` is invalid, `3Rw` is valid from the 4x4x4. The article defines no `2R` and no lowercase big-cube letters (lowercase only for Pyraminx vertices).
- Werner Randelshofer's Superset ENG pages, one per size (<https://www.randelshofer.ch/rubik/pocket/doc/supersetENG_2x2.html>, <https://www.randelshofer.ch/rubik/patterns/doc/supersetENG_3x3.html>, <https://www.randelshofer.ch/rubik/revenge/doc/supersetENG_4x4.html>, <https://www.randelshofer.ch/rubik/professor/doc/supersetENG_5x5.html>, <https://www.randelshofer.ch/rubik/vcube6/doc/supersetENG_6x6.html>, <https://www.randelshofer.ch/rubik/vcube7/doc/supersetENG_7x7.html>): the twists, their defaults and which exist on which size, as in the SSE section above.
- The SiGN description on mzrg.com (<https://mzrg.com/rubik/nota.shtml>): capital letters turn a single layer, lowercase letters several layers; a number before the letter names the layer, or for lowercase the deepest layer of a turn that includes the outer one, with `u` for `2u`; a range such as `2-5r` is written with a dash.

Depth conventions: the sources agree on `2R` (only the second layer) and `3r` (the three outer layers), so there was no choice to make. `3R` (only the third layer) is the same in both. The one choice is `2r`, which mzrg writes `r`: it is accepted on input and written `r`. The speedsolving.com wiki could not be fetched (HTTP 403), and a forum proposal for `r3` was not taken as a convention.
