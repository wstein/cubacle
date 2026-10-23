---
id: 20261023140100
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
`repair` names the mistake, then prices two ways forward and recommends the cheaper: the route back onto the printed path (undo what was done since the mistake, or a shorter correction found by the walk from the cube onto the path) plus the printed moves left, against a fresh solution of the cube as it is. The fresh solve runs beside the search; a tie goes to the repair because it keeps the printed solution in use; a repair is never recommended when it is longer. On the thirteen pairs of consecutive real 7x7x7 attempts the fresh solution usually won: undoing a mistake made early costs more than solving again, and the repair won only where a short correction existed (a wrong amount with two stray moves, a skipped move near the end). The mistake is still worth naming, so the answer shows both.

## Links

- [[Repair searches sticker permutations along the solution]]

Source: src/repair/plan.rs, lines 1-60
