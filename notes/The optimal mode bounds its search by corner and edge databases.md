---
id: 20261024184500
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The optimal 3x3x3 engine (`engines::optimal`, `solve --optimal`) searches with IDA* from the lower bound upwards and prunes with the largest of three 4-bit pattern databases, each looked up for the cube and for its inverse. Small: all corners (42 MiB) and two disjoint sets of six edges (20 MiB each), 83 MiB, 49 s of CPU to build. Large: the corners and two overlapping sets of seven edges (244 MiB each), 529 MiB, 328 s of CPU to build; it is chosen when `--max-memory` (default 1500) allows 555 MB. The tables are cached like the 5x5x5 ones under the ids optimal-*. The default engine runs first with its 3000-probe improvement and is the upper bound and the fallback, so the answer is never longer; a search that ends with nothing shorter proves that answer shortest. A budget (`-b`, `--nodes`) that ends first prints the default answer and says "not proven shortest" with the lower bound reached.

## Links

- [[An optimal mode is a measured clean-room design]]
- [[Shorter solutions beat compute time]]
- [[5x5 table cache]]
- [[Searches are capped at 1.5 GB and run one at a time]]

Source: docs/engines/3x3.md, lines 98-110
