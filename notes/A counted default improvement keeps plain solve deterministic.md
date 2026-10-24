---
id: 20261024120700
aliases: []
tags: ["type/decision", "status/open", "category/engines"]
---
Plain 3x3x3 `solve` returns the first answer: 20.71 moves on 100 fixed random cubes and 16.93 on 30 short scrambles (optimal about 9). `--probes 1000` gives 19.39 and 8.83 for 40 to 80 ms mean and 0.25 s at most on random cubes, and the same output on every machine. `--improve -b 100` gives about 19.0 but depends on the machine, and a budget under the first-solution time returns nothing on the superflip (about 1 to 3 s). So a default, if one is chosen, should be counted (about 1000 probes) with `--fast` restoring today's output. It is not implemented: `tests/cli_test.rs` records the exact first solution of plain `solve`, and the REST server must keep its behaviour until the owner agrees.

## Links

- [[Improve modes trade time for moves]]
- [[Bisect over max_moves is unsound under a time budget]]

Source: examples/default_modes.rs, lines 1-10
Source: docs/engines/3x3.md, lines 72-95
