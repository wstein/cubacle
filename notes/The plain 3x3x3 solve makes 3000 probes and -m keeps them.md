---
id: 20261024200002
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The owner chose 3000 probes as the default of plain solve on the 3x3x3: 18.9 moves on average on 30 fixed cubes against 20.6 for the first answer, about 90 ms process time, never longer than the first answer, deterministic. A fast option returns the first answer byte for byte. The move limit -m caps the probes and does not switch them off; a time budget -b, a target, a move set and explicit probe or improve options do. Time-based improvement was rejected as the default because it fails on hard cubes such as the superflip and varies between runs. The server keeps the first answer until the owner agrees.

## Links

- [[Shorter solutions beat compute time]]

Source: docs/engines/3x3.md, lines 1-120
