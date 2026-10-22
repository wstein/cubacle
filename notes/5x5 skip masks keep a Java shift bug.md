---
id: 20261022160002
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Java genSkipMoves builds its masks with an int shift, so with more than 32 moves (phase 1) moves 32 to 35 land on bits 0 to 3 and move 31 sets the upper bits. The Rust port reproduces this on purpose, because the reduction lengths and the golden transfer depend on it. Fixing it would change solver output and needs an explicit owner decision.

## Links

- [[5x5 reduction engine]]

Source: src/engines/five/mv.rs, lines 52-59
