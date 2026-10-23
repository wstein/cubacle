---
id: 20261023223000
aliases: ["4x4 stage two wing split", "centres sorted and wings split"]
tags: ["type/evidence", "status/reference", "category/engines"]
---
Phase 2 of the 4x4 engine is not only about centres: it also puts every wing pair into the half-slot split that phase 3 needs (`wings_pairable`), so the stage is named `centres sorted and wings split`. With all centres already correct it still costs about 6.4 moves on average, because a random set of wings is pairable in only about 1 case in 1000. A random cube spends 5.8 moves there, a cube that starts in the phase 3 group 0.9.

## Links

- [[4x4 phases and tables]]
- [[4x4 port of the TPR solver]]

Source: src/engines/four/phase2.rs, lines 205-212
Source: src/engines/four/cube.rs, lines 217-220
