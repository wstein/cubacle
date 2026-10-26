---
title: "Recover a mistake"
description: "Rescan the cube after a wrong move and solve from there."
weight: 4
---

# Recover a mistake

## Recover a mistake

**Made a mistake? Do not try to fix it by hand.** Moves made to repair the cube (building centers again, undoing a few turns from memory) only move it further from the printed path, and the search can no longer name the mistake. Stop, rescan the cube and run `cubacle solve --now`, with `--at-move N` for the move you did last before you noticed.

`solve --now` helps a person who followed a printed solution and made a mistake. Give `solve` the original cube (the one the solution was made for, written as always), the cube as it is now with `--now`, and optionally the solution that was handed out with `--solution`. Without `--solution` it derives the solution with the same defaults as `solve`; the solver is deterministic for the same cube, so this gives the printed one. It works for every size from the 2x2x2 to the 7x7x7.

The cube below is the one of the first example after `B L' U2 B2 U2 B2 D2`: the person skipped move 5 (`D'`) and went on to move 8.

```sh
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B'" --now FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL
```

```text
Your cube is off the path of the solution.
Most likely: mistake at move 5, phase 1 (to the G1 subgroup): move 5 (D') was skipped.
You had done 8 of 21 moves when you stopped.
Recover: do 3 moves D2 B2 D', then continue with move 7 (B2). That is 18 moves in all.
Fresh: solve your cube again from where it is, 18 moves.
Recommended: recover, as short as solving fresh and it keeps your printed solution.
Moves now: D2 B2 D'
Then continue with move 7 (B2) of the printed solution.
```

The output says one of these things.

- **The cube is on the path**: `you are after move j of n`, and which move comes next.
- **One or two mistakes explain the cube**: the move and the phase of the solution it is in (named when the solution has more than one phase), the printed move, and what was done instead: the move *skipped*, turned by the *wrong amount* (a quarter instead of a half turn or the reverse), the *wrong way*, in *other layers* (`Fw2` for `3Fw2`), on the *wrong face*, *replaced* by some other move, *done twice*, *swapped* with the next move, or an *extra move* made before it. It also says how many moves were done when the cube was entered. When moves that commute give several equal explanations, the earliest move is shown and the number of the others is printed.
- **One mistake and moves made by hand after it** explain the cube. This is the last resort: it is searched only after every other stage found nothing, so a cube that has a simpler explanation never gets this one. The output then says `then made N stray moves` and `Less certain: this explanation assumes N extra moves after the mistake`; with `--at-move N` the extra moves are taken to follow move N. `-v` shows the stage as `stray moves after the mistake`.
- **The cube is a few moves away from the path**, but no mistake is named.
- **Nothing was found** (three or more mistakes, or moves done by hand after the mistake): the fresh solution is printed.

Tell `solve --now` where you stopped and it searches there first. `--at-move N` says you did move N (counting from 1) and then noticed that the cube no longer matches, so the mistake is at move N or before it and the moves after N are ignored. `--sure-until M` (M below N) adds that you were sure the cube was still right after move M. It is a hint, not a fact, because checking a big cube by eye is unreliable. The search is staged and goes on only when a stage explains nothing:

1. the moves after M (only with `--sure-until`);
2. the moves from the start of the phase that contains move N;
3. the same, from the start of the previous phase;
4. all moves 1 to N;
5. the whole solution, as without `--at-move`.

The phases are the ones `solve -v` reports (for a `--solution` that was not made by `solve`, they are read from the cube). A stop that is a move or two off is tolerated: a skipped move or an extra move shifts the count. `-v` prints which stage answered. On the 7x7x7 attempts of `tests/fixtures/recover_7x7_runs.txt`, the stop alone found the answer in the phase of the stop for four of six explainable cases and one phase earlier for the other two; with a correct `--sure-until` the first stage answered all six.

```sh
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B'" --now FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL --at-move 7 --sure-until 3 -v
```

```text
solution of 21 moves after 41.408208ms
fresh solve 11.25µs
search 277.208µs
stages tried: the window from the move you are sure of; answered by the window from the move you are sure of
Your cube is off the path of the solution.
Most likely: mistake at move 5, phase 1 (to the G1 subgroup): move 5 (D') was skipped.
You had done 8 of 21 moves when you stopped.
Recover: do 3 moves D2 B2 D', then continue with move 7 (B2). That is 18 moves in all.
Fresh: solve your cube again from where it is, 18 moves.
Recommended: recover, as short as solving fresh and it keeps your printed solution.
Moves now: D2 B2 D'
Then continue with move 7 (B2) of the printed solution.
```

(The timings on standard error differ from run to run.)

Then two ways forward are compared: the *recover* (the moves that bring the cube back onto the printed path, followed by the rest of the printed solution) and a *fresh* solution of the cube as it is, which runs beside the search. The recover is either `undo N moves` (the inverse of what was done since the mistake) or a shorter correction found by a search of up to `--depth` moves (default 3) from the cube to any cube of the printed path. The shorter total wins, a tie goes to the recover, and a recover is never recommended when it is longer than the fresh solution. The chosen moves follow `Moves now:` (the way back) or `Fresh solution:` (when solving fresh is recommended), in the notation of `solve`. With `--solution` of the same 3x3x3 and the search limited to undoing, the fresh solution wins:

```sh
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B'" --now FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL --solution "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'" --one-mistake --depth 0
```

```text
Your cube is off the path of the solution.
Most likely: mistake at move 5, phase 1 (to the G1 subgroup): move 5 (D') was skipped.
You had done 8 of 21 moves when you stopped.
Recover: undo 3 moves by turning D2 B2 U2, then continue with move 5 (D'). That is 20 moves in all.
Fresh: solve your cube again from where it is, 18 moves.
Recommended: solve fresh, shorter than the recover (20 moves).
Fresh solution: D2 B2 D' B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'
```

With `--icons` (and `--icon-size`, `--background`, `--columns`, `--views`, `--numbers`, as for a plain `solve`) the mistake is also drawn: the cube as it should be after the last correct move and the cube as it is, side by side, then the rollback and the rest of the printed solution as move pictures, the end state and the moves as one line. See [Recover with icons](../icons.md#recover-with-icons).

On a 4x4x4, move 4 of the solution (`F'`) turned the wrong way:

```sh
cubacle solve --size 4 -s "R U 2R 2U' F 2L' B2 D 2D' R' U2 2F Uw Rw' F2 Lw" --now UDRRDDUUUUUBBLRFDDRUBDUURBRUDFDFDBDLDBFLDLLFRRRLUBBBFBDURLDFDUFULLBRDRFBFBRRRULBFFLBLLRFLFFULLBF
```

```text
Your cube is off the path of the solution.
Most likely: mistake at move 4, phase 1 (centers staged): move 4 (F') was turned the wrong way (F).
You had done 9 of 43 moves when you stopped.
Recover: undo 6 moves by turning B' Uw L2 F Lw' F', then continue with move 4 (F'). That is 46 moves in all.
Fresh: solve your cube again from where it is, 42 moves.
Recommended: solve fresh, shorter than the recover (46 moves).
Fresh solution: R2 D' R U' Fw D2 Fw B2 U2 F2 Dw' D2 F' Dw2 L B' R L' Bw2 Rw2 F' Bw2 Rw2 D2 F2 R2 F' R2 B R2 L2 B L D R' F2 U' R' B R' L D'
```

(plus `preparing the 4x4x4 tables ...` on standard error.) Only if the normal search finds nothing, the 23 other orientations of the whole cube are tried, and a cube that fits one is reported with the turns (`x`, `y`, `z`) that rotate it back: `Your cube is held differently from the original: turn the whole cube (x') to match it.` The moves printed then assume that orientation.

#### Towards a target

When the solution leads to a target and not to the solved cube (see [Solve to a target](solve.md#solve-to-a-target)), add `--to` with the same target to `--now`. The printed solution is then the one `solve --to` gives (or your `--solution`, which must lead the original cube to the target), the mistake is searched along it, and the way back leads to the target. The target is read as for `solve --to`, as moves or as a state, and gets the same refusals (for a 4x4x4 and larger, one color per face or per face and orbit). A target with open stickers (`X`) is refused with `--to: with --now the target must be a complete cube, without open stickers`; a `--solution` that misses the target gets `the solution does not lead the original cube to the target`. Every other option of `--now` keeps its meaning.

```sh
cubacle solve -s "R U2 F' D L2 B R' U' F2 D2 L B'" --to "R2 U2 F2 D" --now UDDDUUDUUFFBLRRBBFLLLBFFLLLDDUDDUUUDBFFLLRFBBRRRFBBRRR
```

```text
Your cube is off the path of the solution.
Most likely: mistake at move 22, phase 2 (solve): the extra move U was made after the last move.
You had done 21 of 21 moves when you stopped.
Recover: undo 1 move by turning U', then continue with the end. That is 1 moves in all.
Fresh: solve your cube again from where it is, 11 moves.
Recommended: recover, 10 moves fewer than solving fresh.
Moves now: U'
Then continue with the end of the printed solution.
```

With `--icons` the pictures show the real cubes, and the end state is the target, not a solved cube. Both cubes are mapped to the problem of the solved cube first (the relative state of each against the target), the search runs there, and the same moves are applied to the real cubes; the tests replay every answer on the real cube and compare it with the target.

These options belong to `solve --now`. All but `--now` itself need it, and `--notation`, `-v`, the cube options and the picture options are the ones of a plain `solve`.

| Option | Meaning |
| --- | --- |
| `--now <CUBE>` | The cube as it is now, an Orbit64 token, facelets or colors (quote it when it has spaces). Switches `solve` to this mode. It cannot be combined with `-m`, `-b`, `--probes`, `--improve`, `--parallel`, `--fast`, `--optimize`, `--slices`, `--moves`, `--all`, `--count`, `--raw`, `--optimal`, `--nodes`, `--max-memory` or `--group`. With `--to` the way leads to the target (see [Towards a target](#towards-a-target)). |
| `--solution <MOVES>` | The solution the person followed, in WCA, SiGN or SSE, groups and ranges allowed (see [Move notations](move-notations.md#move-notations)). Default: the one `solve` prints for the original cube. |
| `--notation <wca\|sign\|sse>` | How the moves in the explanation and in `Moves now:` are written. Default `wca`. |
| `--at-move <N>` | The move you did last before you noticed the mismatch, counting from 1. The mistake is at move N or before; later moves are ignored. |
| `--sure-until <M>` | Needs `--at-move`. The move after which you are sure the cube was right (M before N). A hint: the search widens past it. |
| `--fresh` | Skip the search and print the solution of the current cube. |
| `--depth <N>` | The same flag as the one of `--all` and `--count`, which cannot be used with `--now`. Moves the search for a way back onto the path may use, 0 to 5 (default 3). Measured on a 7x7x7: 3 adds about 20 ms per orientation, 4 makes the whole search take 3 to 5 s, 5 reaches the default time limit and holds about 70 MB. Turned cubes are searched at depth 3 at most. |
| `--strays <N>` | Stray moves after a mistake the last stage allows, 0 to 3 (default 2; one for solutions under 16 moves). 0 leaves the stage out. |
| `--one-mistake` | Look for one mistake only. The default also looks for two. |
| `--thorough` | For two mistakes, try every move instead of it and every extra move. Slow on big cubes (it does not finish within the default time limit on a 7x7x7). |
| `--max-time <MS>` | Time limit of the search in milliseconds, 0 for none (default 8000; with `--within` 60000). The result then says that the search stopped at its time limit. (The flag is the same one a plain `solve` has, with another default.) |
| `-v`, `--verbose` | Print the time of the solve, the fresh solve and the search to standard error. |

The original cube is the one `solve` reads: `--scramble`, `--size` and `--scheme` apply as always. The search treats a move as a permutation of the stickers, so it is the same for every size. Measured on a 7x7x7 (12 cores, with the fresh solve running beside the search): a mistake is found in 30 to 250 ms; a cube that nothing explains takes 1 to 4 s because all 24 orientations are tried; the fresh solve takes 3 to 12 s and the solution of the original cube 4 to 8 s. The mistakes are tried in two levels: one mistake, with every move as a replacement or an extra move, and two mistakes of the likely kinds only (skipped, wrong amount, direction, layers or face, swapped, done twice).
