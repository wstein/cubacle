---
id: 20261027100000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
`solve --inverse` solves as always, then shows the inverse: the pictures start from the cube the solution ends in (the solved cube, or the `--to` target) and the target line is the start cube. The phase list is reversed with the ranges mirrored and named `undo phase N: <name>`, as the center fix-up stages are; `--every` and `--optimize ranges` are applied to the inverted moves, so the cuts and the ranges are those of what is shown. The counts on standard error stay those of the solution (the inverse of a sequence has the same length and metrics). `--now`, `--all` and `--count` print other things than one solution and are refused. `canon` and `convert` get no `--inverse`: they print a state, and writing a state as moves is a solve, which `solve --inverse` already does. The REST `inverse` field is `MoveSequence::inverse` written in the notation of `solution`.

## Links

- [[A middle dot marks a hand unit, not a simultaneous turn]]

Source: src/inverse.rs, lines 1-64
