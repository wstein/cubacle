---
id: 20261022140001
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
Reference numbers for the 4x4x4, claimed and measured apart. Claim (README, 2011 laptop): 44.39 moves, 250 ms, 6 to 7 s start, at most 30 MB. Measured on an Apple M2 Pro with Java 21 (the README command `test 2000` prints 45.05 with rotation characters counted, 44.57 without, 128.9 ms mean), 1000 cubes from the reference's own generator, every answer replayed with Facelets::apply (1000 of 1000 solved): mean 44.59 moves without the final rotation (median 45, p90 46, max 47), 47.71 with it as wide turns; 131 ms mean and 112 ms median per cube warm; start 0.81 s; heap 24 MB suffices. The Rust port must match the length (mean at most 44.7, p90 at most 46) and be within 1.5 times the median time; thresholds are proposals for the owner.

## Links

- [[4x4 port of the TPR solver]]
- [[4x4 phases and tables]]
- [[4x4 final rotation]]

Source: docs/design/4x4-port.md, lines 90-117
Source: docs/design/4x4-port.md, lines 359-388
