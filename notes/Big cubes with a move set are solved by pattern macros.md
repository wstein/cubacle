---
id: 20261024130002
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
On a 4x4x4 to 7x7x7, `solve --moves SET` and `patterns find --moves SET` keep only the pattern macros whose moves (and those of their inverse) all lie in the set, then use the same meet-in-the-middle search as the known-pattern shortcut. A macro is kept only when its inverse is also present, which is a limit: a set that holds a macro but not its inverse finds less than it could. A set with no macro (`R`) gives an error that says so. A cube that is no product of the kept macros gets "no product of pattern macros ... that does not mean the cube cannot be solved", and a search that reaches the memory cap (`--max-memory`, default 1500 MB) says "nothing found within the cap"; neither is ever called unsolvable. The `Failure` enum is unchanged (the server matches it exhaustively), so these cases are `Exhausted` with a sentence.

## Links

- [[Pattern shortcut finds hand-built cubes]]

Source: src/engines/pattern/solver.rs, lines 1-8
