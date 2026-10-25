---
id: 20261022130001
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
FiveSolver accepts only a 5x5x5 whose six face centers are in place and an injected solver of kind 3x3x3 (otherwise InvalidInput or Internal); a cube whose face centers moved is Failure::CentersOffFace, not InvalidInput, because it is reachable; the command line and the server rename the colors after the centers before they call it, so only a solve to a target gets this far. `maxMoves` bounds the whole solution; when the time budget runs out the solve fails with BudgetExhausted, and without a budget the 3x3x3 stage is cut off after 10 s as Exhausted, and the message says so if the tables were still being built. An internal impossibility (no way found, moves that do not fit) is Failure::Internal, HTTP 500.

## Links

- [[5x5 reduction engine]]

Source: src/engines/five/solver.rs, lines 215-318
Source: docs/engines/5x5.md, lines 74-79
