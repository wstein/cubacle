---
id: 20261022140004
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
A 4x4x4 has no fixed centres, so the reference solves it up to a whole-cube rotation and prints the rotation. The owner kept the contract that a solution only has layer turns and ends in Facelets::solved: the rotation becomes at most four wide turns (Rw Lw' is x), found by trying the 24 rotations and replaying, which costs 3.1 moves on average (47.47 against 44.34). Both lengths are reported. A second meaning of solved (all faces uniform) was not adopted.

## Links

- [[4x4 versus TPR]]
- [[Colour schemes]]

Source: docs/design/4x4-port.md, lines 183-204
Source: src/puzzle/solver.rs, lines 47-49
Source: src/engines/four/rotation.rs, lines 1-35
