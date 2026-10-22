---
id: 20261022140004
aliases: []
tags: ["type/question", "status/open", "category/open-items"]
---
A 4x4x4 has no fixed centres, so the reference solves it up to a whole-cube rotation and prints the rotation. The plan keeps the contract that a solution only has layer turns and ends in Facelets::solved: the rotation becomes at most four wide turns, measured at 3.1 moves on average. The open question is whether to accept a second meaning of solved (all faces uniform) and save them.

## Links

- [[4x4 versus TPR]]
- [[Colour schemes]]

Source: docs/design/4x4-port.md, lines 183-204
Source: src/puzzle/solver.rs, lines 47-49
