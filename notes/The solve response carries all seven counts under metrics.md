---
id: 20261024235001
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
The solve answer has a `metrics` object with all seven counts (htm, qtm, stm, etm, obtm, btm, atm) in that fixed key order, not the three or four of the CLI line: JSON clients ignore what they do not need, and a fixed shape needs no per-size rule. `compressedMetrics` counts the compressed text when `optimise` is set. The server has no recover route, so only solve changed. The counts come from `cubacle::metrics`, the same code as the CLI, and the server tests compare them with it. No existing test expectation needed a change, because the tests read single fields. The server keeps the default solve options, without the CLI's probe minimum.

## Links

- [[The REST responses always carry the move counts]]
- [[Move metrics count turns of contiguous blocks]]

Source: crates/server/src/counts.rs, lines 1-40
