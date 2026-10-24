---
id: 20261024133011
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
The prototype that tries known patterns before the reduction and the library with its search tool were built apart and use two macro sets. They are merged into one module: the registry supplies named patterns and base macros, the same meet-in-the-middle search serves the solve shortcut, the patterns command and a restricted move set on big cubes, and the numbers of the prototype (the owner's 7x7 from 172 to 38 moves, cube in a cube in 15) stay as regression tests.

## Links

- [[Pattern shortcut finds hand-built cubes]]
- [[Patterns are defined by structure and searched]]
- [[Pattern states are projections onto layer sets]]

Source: src/engines/pattern/mod.rs, lines 1-40
Source: src/patterns/mod.rs, lines 1-40
