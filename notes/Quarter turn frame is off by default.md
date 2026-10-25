---
id: 20261023190000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
The amber frame around a lone quarter turn in the Kitty rows of `solve --icons` and `solve --now --icons` confused more than it helped. It is off by default and kept behind the compile-time cargo feature `quarter-frame` (no CLI option, no environment variable). The text note `<-- quarter turn, not a half turn` stays in every build. If the frame proves more confusing than helpful, remove it for good.

## Links

- [[Solve icons checkpoints are Orbit64 tokens]]

Source: Cargo.toml, lines 1-60
