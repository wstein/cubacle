---
id: 20261024180003
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Corners only, edges only, centers only, the domino group G1 (on each axis) and the half-turn group are exact from the pieces. G1 is untwisted corners, unflipped edges and the middle-layer edges in the middle layer. The half-turn group needs one more fact: classes of four corners and four edges are kept, and the arrangements of the five classes project to 648 tuples in S3 to the fifth; with the free Klein groups that is 663552 states, which a test confirms by walking the whole group. The groups R,U and R,U,F have no such description, so the test only refutes (a piece they never touch is moved) and otherwise says possible. The restricted engine owns the reach checks but patterns may not depend on engines, and its checks are private and only necessary, so the detector keeps its own and the table search stays the proof.

## Links

- [[A pattern detector reads structure from the cubie state]]
- [[The detector explains patterns and never replaces the search]]

Source: src/patterns/detector/halfturn.rs, lines 1-20
