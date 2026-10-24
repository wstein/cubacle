---
id: 20261024133012
aliases: []
tags: ["type/evidence", "status/reference", "category/process"]
---
The occasional failure of the 6x6x6 and 7x7x7 solver tests was not load: normalising a solution could merge a slice turn at the end of one stage with the first turns of the next, so the stage end pointed at a place the cube never was; the stage end now moves forward to the first position where the stickers solved at the engine's end are solved again, and no solution changed. A second failure came from recover tests that carried a wall-clock deadline of 8 seconds, so a loaded machine gave a wrong answer instead of a longer search; those tests now clear the deadline. Reruns passed because the scrambles are random, not because the load had gone.

## Links

- [[Phases come from the solver's stages]]
- [[Flaky parallel_solve_is_repeatable test]]

Source: tests/dwalton_test.rs, lines 30-60
Source: tests/recover_pilot_test.rs, lines 140-150
