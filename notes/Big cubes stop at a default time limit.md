---
id: 20261025200000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The owner decided on 2026-10-25 that a solve of the 4x4x4 to 7x7x7 stops after 120 s unless `--max-time MS` says otherwise (0 for none), with `error: time limit of N s reached on the NxNxN (raise it with --max-time MS)`, nothing on standard output and exit status 3. The default is set once, in `DEFAULT_MAX_TIME_MS` of `src/main.rs`; the engines only see `SolveOptions::time_budget`. Measured on a loaded machine the slowest of three random solves took 18 s (7x7x7), so the limit is about seven times that. The 6x6x6 and 7x7x7 engine checks the clock inside the center searches (every 4096 nodes, in `reduction::ida`) and between the wing stages, which get what is left; reading or building tables is not counted. A face center off its face is a separate `Failure::CentersOffFace`, status 1, because such a cube is reachable.

## Links

- [[5x5 parity and limits]]
- [[Cold start and prepare]]

Source: src/main.rs, lines 28-52
Source: src/engines/dwalton/solver.rs, lines 60-190
Source: owner decision of 2026-10-25, lines 1-1
