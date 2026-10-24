---
id: 20261024130003
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`solve --all`, `--count N` and `--depth D` (3x3x3 only) first find the shortest length with the restricted search, then list every solution of exactly that length by a depth-bounded search with the table as the bound, and with `--depth` of each longer length up to D, shortest first, streaming each solution as found. Of commuting moves one order is listed unless `--raw`; a pair that makes the identity or one letter is never listed, and a solution never passes through the solved cube before its last move. The cancel hook and the time budget are asked at every node, which keeps the server rule that compute never blocks an async thread. A default (`htm`) listing has only a small table as bound, so it is practical for short scrambles; the budget ends a long one with the count found so far. REST (`moves`, `all`, `count` on `/v1/solve`) is a follow-up and the server is untouched.

## Links

- [[Restricted solves search the alphabet's own letters]]

Source: src/engines/restricted/enumerate.rs, lines 1-16
