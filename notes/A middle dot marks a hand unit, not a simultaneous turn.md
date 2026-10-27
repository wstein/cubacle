---
id: 20261024130000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
A `·` in a move list marks a hand unit (a grip group): the moves between two dots are done in one grip, one after the other. It is not a simultaneous turn and it changes no move. The notation reader keeps the structure next to the flat list: marks are recorded only for `parse_grouped`, so `parse_expression` returns the same moves as before. `( ... )` is a block of units (two levels are kept, deeper parentheses are flattened); repetition and inversion carry the marks along. `moves --icons` draws units as close icons under a thin numbered bracket and blocks under a thicker one; rows break between units. `solve --group phases` writes ` · ` between the solver phases it reports; engines without stage ends (the 2x2x2 and the 3x3x3 here) write none. The `--group` flag of `moves` (dots or none) and of `solve` (phases or none) are separate options, because `solve` shares its picture options with `recover`. Follow-ups: REST stays a flat string for now; JSON groups and merged two-corner icons for opposite pairs (`--group axis`) are later work.

## Links

- [[Move input rules are WCA and SiGN]]
- [[An inverse solve mirrors the phases and starts from the end state]]

Source: src/notation/expr/grouped.rs, lines 1-60
