---
id: 20261023180000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
Moves are read as WCA and SiGN everywhere (solve --scramble, convert, facelets, solve --now --solution, icon, REST `scramble`) by one reader, `src/notation/expr/`. It expands everything into plain moves, so the engines, the metric and the move limit (100000) are unchanged.

Rules and where they come from (read 2026-10-23):

- WCA Regulations Article 12a1/12a2/12a4 (<https://www.worldcubeassociation.org/regulations/#article-12-notation>): `nRw` turns n outer layers with 1 < n < N, `Rw` is `2Rw`; x, y, z are cube rotations. The article has no layer ranges, no groups, no brackets for NxNxN and no `M E S`; those are SiGN or community notation. The hybrid range `2-3Rw` is therefore our extension.
- SiGN, mzrg.com (<https://mzrg.com/rubik/nota.shtml>): lowercase letters turn several layers, a number before the letter names the deepest layer, a range `a-b` is written with a dash (`2-5r`), `'` after a parenthesised group inverts it, a number repeats it, `[P, Q]` is P Q P' Q' and `[P: Q]` is P Q P'.
- Ruwix (<https://ruwix.com/the-rubiks-cube/notation/advanced/>): `2F` is the second layer alone, a lowercase letter the two outer layers, `(X)2` = X X (a reader comment, not the page text). It defines no ranges or commutators.
- The speedsolving.com wiki answered 403 and was not used.

Decisions: the layers a..b are `b` outer layers wide, then the inverse of `a-1` outer layers; `a-1 = 1` is the plain face move, so `2-3r` is `3Rw R'` (not `3Rw Rw'`, which would leave the outer layer turned and layer 3 only). A wide turn turns every layer the same way as the face, so there is no per-face case; tests apply every range on sizes 2 to 7 and compare with single-layer turns. `(...)N` is a repeat (not a half turn), `(...)'` inverts, `*N` stays; `R2` keeps its meaning for single moves. A bracket has exactly one `,` or `:`; nesting is for more. Errors carry a character position.

## Links

- [[SiGN notation as an output option]]
- [[Input notations]]

Source: owner task of 2026-10-23, lines 1-1
Source: src/notation/range.rs, lines 1-8
Source: src/notation/expr/mod.rs, lines 1-60
Source: docs/cli.md, lines 380-420
