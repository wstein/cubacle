---
id: 20261022140001
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
Reference numbers for the 4x4x4, claimed and measured apart. Claim (README, 2011 laptop): 44.39 moves, 250 ms, 6 to 7 s start, at most 30 MB. Measured on an Apple M2 Pro with Java 21, 1000 cubes from the reference's own generator, every answer replayed with Facelets::apply: Java mean 44.59 moves without the final rotation (median 45, p90 46, max 47), 47.71 with it as wide turns; 115 ms mean, 99 ms median, 197 ms p90 per cube warm, single thread; start 0.8 s. Rust on the same cubes: 44.34 moves (median 44, p90 46, max 47), 47.47 with the rotation; 70 ms mean, 60 ms median, 115 ms p90 on one thread; tables 12.1 MB built in 0.37 s, peak resident 65 MB.

## Links

- [[4x4 port of the TPR solver]]
- [[4x4 phases and tables]]
- [[4x4 final rotation]]

Source: docs/design/4x4-port.md, lines 90-117
Source: docs/design/4x4-port.md, lines 484-531
Source: tools/bench/run_four.sh, lines 1-26
