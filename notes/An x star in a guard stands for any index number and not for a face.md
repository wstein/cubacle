---
id: 20261025140105
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
Planned: in a guard `x*` stands for any index number, so unique(a*,b*,c*) repeats the condition for every index present. It is not automatically a face: the index is only a label, so a guard may also compare variables of different faces (cross-face guards are allowed). Keeping the index free of meaning lets the same syntax serve per-face rules and rules across faces.

## Links

- [[A guard combines unique, equal and count with boolean operators]] defines the operators the wildcard appears in.
- [[A schema grid is written with lowercase variables, uppercase colors and don't-care dots]] shows the index numbers on variables.

Source: owner decision of 2026-10-25, lines 1-1
