---
id: 20261024235002
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
`--group axis` cuts the moves into hand units: consecutive turns about one axis whose layers do not overlap are one unit (`U D'`, `R L`, `Rw Lw`), the grip of the axial move of ATM. The cut works on the expanded moves, so a layer range written as two turns (`3Rw R'`) is one block and a rotation, written `Rw L'`, joins the unit of its own axis. Marks typed in the input are ignored by this mode. With `solve --group phases,axis` the phase border always ends a unit, so the axis dots only refine the phases. Overlap ends a unit where ATM would not (`U D' U` is two units, ATM counts one), because one grip cannot turn a layer twice.

## Links

- [[Move metrics count turns of contiguous blocks]]

Source: src/notation/axis.rs, lines 1-120
