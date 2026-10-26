---
title: "Apply a scramble"
description: "Apply a scramble: groups, commutators, hand units and slices."
weight: 9
---

# Apply a scramble

## Apply a scramble

A scramble is move notation in WCA, SiGN or SSE spelling (see [Move notations](move-notations.md#move-notations)) with the moves separated by whitespace or a middle dot `·` (`R·U·R'·U'`). Besides the canonical forms (`R`, `R2`, `R'`, and wide or inner-layer moves such as `Rw` or `2R`), these whole-token aliases are accepted:

| Written | Read as |
| --- | --- |
| `R1` | `R` |
| `R3` | `R'` |
| `R2'`, `R'2` | `R2` |

Moves must be separated: a run such as `RUR'` and a lone `-` are rejected. A 3x3x3 whose centers a wide or inner-layer move has displaced is solved by the color of its centers (see [Face centers off their faces](solve-options.md#face-centers-off-their-faces)); for a rotation (`x`, `y`, `z`) that is `moves 0`, since every face is already one color.

### Repeated groups

`( ... )` groups moves and `*N` repeats the group, or a single move, right before it N times. Groups nest, and parentheses need no spaces around them; `*N` must follow its group or move directly (`R *2` and `R*0` are errors, as are unbalanced parentheses). An expression may expand to at most 100000 moves. The expression is expanded into plain moves before it is applied, so `(R U)*3` is `R U R U R U`. Six repetitions of the sexy move return a 3x3x3 to the solved state:

```sh
cubacle facelets "(R U R' U')*6"
```

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

After a group, `*N` or a bare `N` repeats it N times and `'` inverts it (the moves in reverse order, each inverted); both can be given, in either order: `(R U)3`, `(R U)'`, `(R U)2'` (inverse of two repeats). After a single move only `*N` repeats (`R2` stays a half turn). Run in this build:

```sh
cubacle facelets "(R U)2'"
cubacle facelets "U' R' U' R'"
```

print the same cube twice.

### Commutators and conjugates

`[A, B]` is the commutator `A B A' B'` and `[A: B]` the conjugate `A B A'`. `A` and `B` are any expressions (groups, rotations, slices, ranges and further brackets, nested up to 100 deep) and may be written with or without spaces; a bracket can take the same suffixes as a group (`[R, U]2`, `[R, U]'`). Each bracket has exactly one `,` or `:`; use a nested bracket for more. Both sides must hold moves. The 100000-move limit counts the expanded moves, so `[(R U)*40000, F]` is refused.

```sh
cubacle facelets "[R U: F]" # is R U F U' R'
cubacle facelets "R U F U' R'"
```

Errors name the character (counted from 0) where the problem starts: `R [U, F, D]` fails at character 7, an unclosed `[` at its own position, and a layer the cube lacks (`2-3r` on a 3x3x3) at the start of the word.

### Hand units

A `·` ends a hand unit: the moves between two dots are done in one grip, one after another (the dot is not a simultaneous turn and changes no move). `( ... )` is a block of units. Two levels are kept for drawing; deeper parentheses are flattened without error. Repetition and inversion repeat the units too (`(R · U)*2` is two blocks of two units). Any notation can be mixed:

```sh
cubacle moves --size 7 "F' · U D · L' U' (ND' NU · R U2 L' · NF NB' · L U2 R') U L · U' D' · F"
```

```text
F' U D L' U' 2D' 2U R U2 L' 2F 2B' L U2 R' U L U' D' F
```

The plain list is the same with or without dots. `moves --icons` draws the units (see [Move icons](../icons.md#hand-units)). `solve --group phases` writes the dots of the solver phases: it is run on the 4x4x4 with

```sh
cubacle solve --size 4 --group phases "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw"
```

```text
B' L' F' R Bw Dw' · B F' R' Dw2 L' Rw2 Fw' · D' L' R' Fw2 U' Bw2 R2 L' F2 Dw2 R U Rw2 · U B U L2 B2 D2 F' U F2 L2 D2 R B2 L F2 R2 U2 L' B' L
```

`--group axis` cuts by axis instead. Consecutive turns about the same axis whose layers do not overlap are one unit, because one grip turns them together (the axial move of the ATM metric, see [Move metrics](../metrics.md)): `U D'`, `R L`, `Rw Lw` on a 4x4x4 or `2R 2L'` on a 5x5x5. A turn that overlaps the layers already in the unit starts the next one (`U D' U` is `U D' · U`), and so does a turn of another axis. A layer range written as two turns, `2-3Rw` (`3Rw R'`), is one block. A rotation joins the unit of its own axis as in ATM, so `R x L` is `R Rw L' · L` (`x` is written `Rw L'`); a rotation of another axis starts a unit. Marks typed in the input are ignored by `moves --group axis`: the units are the ones cut here. Run on a 3x3x3 and a 7x7x7:

```sh
cubacle moves --group axis "U D' R L F B2 R U"
cubacle moves --size 3 --group axis "R x L U"
cubacle moves --size 7 --group axis "R L Rw Lw 2-3Rw 2-3Lw U"
```

```text
U D' · R L · F B2 · R · U
R Rw L' · L · U
R L · Rw Lw · 3Rw R' 3Lw L' · U
```

Without `--icons` `moves --group axis` writes the dotted line (`--group none` needs `--icons`; `--expand` and `--optimize` cannot be combined with it). With `--icons` the units are drawn with brackets as for typed dots (`group 1 of 5: U D'`). `solve --group axis` puts the dots into the answer:

```sh
cubacle solve --size 4 --group axis "R U 2R 2U' Fw 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw"
```

```text
B' · L' · F' · R · Bw · Dw' · B F' · R' · Dw2 · L' Rw2 · Fw' · D' · L' R' · Fw2 · U' · Bw2 · R2 L' · F2 · Dw2 · R · U · Rw2 · U · B · U · L2 · B2 · D2 · F' · U · F2 · L2 · D2 · R · B2 · L · F2 · R2 · U2 · L' · B' · L
```

`--group phases,axis` combines the two. The phase border wins: a unit never crosses it, so the dots are those of `phases` plus those of `axis`, and `U | D'` split across two phases is `U · D'` although `axis` alone writes `U D'` (the 4x4x4 answer above has the same dots both ways; a unit test covers the split). With `--optimize ranges` each unit is compressed on its own. `solve --group axis` has no icons form (`--icons` stays the phase pictures). The dotted answer is read back by `moves` as the same moves (a test checks it for wca, sign and sse). The 2x2x2 and the 3x3x3 (min2phase) report no stage ends: in the runs made for this page the whole solution was one phase and no dot was written. The REST backend returns a flat string; it has no groups.

### Middle slices and whole-cube rotations

| Written | Meaning |
| --- | --- |
| `M`, `E`, `S` | The middle layer of an odd cube, turned like `L`, like `D` and like `F`. On a 3x3x3 `M` is `2L`; on a 7x7x7 it is `4L`. |
| `x`, `y`, `z` | The whole cube turned like `R`, like `U` and like `F`. |

All of them take `'`, `2` and the aliases above, and `*N`. An even cube (2x2x2, 4x4x4, 6x6x6) has no middle layer and refuses `M`, `E` and `S` with an error; `x`, `y` and `z` work on every size. Wide forms such as `Mw` and prefixed forms such as `2M` are not accepted. The expression is expanded into plain layer turns: a rotation becomes a wide turn of all layers but one plus the opposite outer face turned the same way round the axis, so `x` on a 3x3x3 is `Rw L'` (`Rw` is the two layers on the right). The standard identities hold and are tested: `x = R M' L'`, `y = U E' D'`, `z = F S B'`.

A rotation moves the centers, so the state after it is a rotated cube, not a solved-looking one: after `x` a solved 3x3x3 shows `F` on the up face.

```sh
cubacle facelets "x"
```

```text
FFFFFFFFFRRRRRRRRRDDDDDDDDDBBBBBBBBBLLLLLLLLLUUUUUUUUU
```

```sh
cubacle facelets "M"
```

```text
UBUUBUUBURRRRRRRRRFUFFUFFUFDFDDFDDFDLLLLLLLLLBDBBDBBDB
```

`facelets` and `--scramble` read the same notation.

For `convert` and `solve` the text is tried as a cube first (token, facelets, colors), and only when it is none of those as a move expression. So `U U U U R R R R F F F F D D D D L L L L B B B B` is read as the 2x2x2 facelets it spells, whereas `facelets` and `--scramble` always read a scramble: use `--scramble` when the text could be both. `--size` is not compared with the size of a cube given as text. When the text is neither, the error gives both reasons. `convert` and `solve` also take an expression in place of a cube when the text is not a cube; `--size` picks the cube:

```sh
cubacle convert --size 3 "(R U)*6"
```

```text
FULUUBRUURRUFRRBRUDLFFFUFFRDDDDDFDDRLBFLLLLLLBRUDBBBBB
```

`facelets` prints the facelets reached by applying a scramble to a solved cube (`--size` picks the cube, default 3):

```sh
cubacle facelets "R U R' U'"
```

```text
UULUUFUUFRRUBRRURRFFDFFUFFFDDRDDDDDDBLLLLLLLLBRRBBBBBB
```
