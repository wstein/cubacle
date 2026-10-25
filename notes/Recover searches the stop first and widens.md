---
id: 20261023150000
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
`solve --now --at-move N` names the move the person did last before noticing the mismatch, so the mistake is at move N or before and later moves are ignored; the search runs the existing one-mistake and two-mistake lookups with the mistake positions held to a window (the printed moves used up may differ from N by two, since a skipped or extra move shifts the count). `--sure-until M` is a hint, never a fact, because people check big cubes badly; the search is therefore staged and widens only when a stage explains nothing: after M, the phase containing N, the previous phase, all of 1..N, and the whole solution with the other cube orientations. The phases come from the engine stages that `solve` reports, passed in as plain start positions so `recover` still depends only on `puzzle`. On the real 7x7x7 attempts the phase of the stop answered four of six explainable pairs and the previous phase the other two. People are told not to repair the cube by hand: extra moves are no longer searched, which cost two of the four recorded attempts their explanation.

## Links

- [[Recover searches sticker permutations along the solution]]
- [[Recover advises the shorter way]]

Source: src/recover/stages.rs, lines 1-10
