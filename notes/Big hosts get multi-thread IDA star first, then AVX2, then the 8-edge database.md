---
id: 20261025160005
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Owner decision of 2026-10-25 for the big hosts, in this order: a multi-thread IDA* in the optimal mode, then the AVX2 move step, and the 8-edge database only after a measurement shows it pays. Table sizes follow the memory the host really has, never the platform, so each step is measured on the same footing.

## Links

- [[Large hosts measure the big tables, the Mac sets the default]]
- [[Table sizes follow the memory the host really has]]
- [[Vector code exists for NEON, AVX2 and AVX-512 with one fallback]]

Source: owner decision of 2026-10-25, lines 1-1
