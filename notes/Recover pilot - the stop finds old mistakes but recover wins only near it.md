---
id: 20261023170000
aliases: []
tags: ["type/evidence", "status/decided", "category/architecture"]
---
The fixed pilot (`examples/recover_pilot`, 60 cases on ten fixed cubes, one run of 113 s) shows two separate things. Naming the mistake works far back: with the stop, 46 of 60 cases named exactly the injected steps (38 of 42 with one mistake, 8 of 12 with two, none of 6 with three) and the kind agreed in 46 of 48 named cases, also at distances 25 to 50 where the answer came from the previous phase or all moves up to the stop. Advising the way back only pays for recent mistakes: the route beat a fresh solve in 16 of 60 cases, 9 of them at distances 2 and 5, and in none at distance 35 or more, because undoing 35 moves costs more than the 40 to 60 moves of a fresh solve. Without the stop the same 17 cases named the mistake as often (17 of 17), at a lower mean time (150 ms against 223 ms), so on this small series the stop did not make the search find more, only choose the window; the series is too small to say it never helps.

## Links

- [[Recover advises the shorter way]]
- [[Recover searches the stop first and widens]]

Source: examples/recover_pilot/main.rs, lines 1-8
