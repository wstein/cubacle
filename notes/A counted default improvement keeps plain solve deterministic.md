---
id: 20261024120700
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Plain 3x3x3 `solve` returned the first answer: 20.71 moves on 100 fixed random cubes and 16.93 on 30 short scrambles (optimal about 9). `--probes 1000` gives 19.39 and 8.83 for 40 to 80 ms mean and 0.25 s at most on random cubes, and the same output on every machine. `--improve -b 100` gives about 19.0 but depends on the machine, and a budget under the first-solution time returns nothing on the superflip (about 1 to 3 s). The owner decided on 2026-10-24: plain 3x3x3 `solve` makes 3000 probes (19.00 moves, 135 to 184 ms mean), `--fast` restores the first answer byte for byte, explicit modes and limits (`--probes`, `--improve`, `--moves`, `--to`, `-m`, `-b`) win, the server is unchanged. The pinned test uses `--fast`.

## Links

- [[Improve modes trade time for moves]]
- [[Bisect over max_moves is unsound under a time budget]]

Source: examples/default_modes.rs, lines 1-10
Source: docs/engines/3x3.md, lines 72-95
