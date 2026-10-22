---
id: 20261022120008
aliases: []
tags: ["type/evidence", "status/open", "category/notation"]
---
An Orbit64 token cannot prove that its producer used the same slot numbering as `orbit64-3x3-draft@1`, which uses the standard Kociemba corner order URF UFL ULB UBR DFR DLF DBL DRB and edge order UR UF UL UB DR DF DL DB FR FL BL BR. The decoder therefore has to trust the producer.

## Links

- [[Orbit64 tokens]]
- [[Orbit64 example mismatch]]

Source: src/notation/orbit64.rs, lines 7-11
