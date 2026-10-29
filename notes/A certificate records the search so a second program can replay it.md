---
id: 20261029150002
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
`solve --certificate` prints plain `key: value` lines after the moves: the convention version, the start stickers and twist, the target and its twist masks, the moves, the zero-cost frame and final rotations, the end state, the lower bound at the start, the nodes of every depth searched out and a digest of each table. `cubacle verify` replays the moves through the facelet model and the center tracker and checks that the searched-out depths run without a gap from the bound to the answer; it does not rebuild the tables, so shortness is consistent but not re-proved, and an independent checker recomputes the state and the moves from the same lines. A search that stops on a limit prints the record without moves, so the proven depth stays on file.

## Links

- [[A picture cube adds the center twists as a search coordinate]]

Source: src/certificate.rs, lines 1-15
