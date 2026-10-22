---
id: 20261022140007
aliases: []
tags: ["type/question", "status/open", "category/open-items"]
---
A module engines/reduction would hold code written once for the 4x4 and the 5x5 ports: combinatorics, prune-table builders, a bounded candidate list, ordered parallel helpers and the 3x3x3 hand-off now only in five. It holds no ported code, so it needs no third-party notice. The 5x5 move vocabulary and 3-cycle library are not extracted any more, because the port does not use 3-cycle libraries. To be aligned with the author of the 5x5 port through the coordinator.

## Links

- [[4x4 port of the TPR solver]]
- [[Crate roles and dependency direction]]
- [[Reduction engines and the 3x3 solver]]

Source: docs/design/4x4-port.md, lines 327-358
Source: ARCHITECTURE.md, lines 67-79
