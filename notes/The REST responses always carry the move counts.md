---
id: 20261024200000
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
The owner decided on 2026-10-24 that the solve and recover responses of the REST server always carry the move counts of the metrics line (not only on request). This changes every existing response, so the golden test of the server is updated in its own commit with that reason; solver fixtures stay untouched. Work is queued after the move-metrics merge; the CLI already prints the counts on stderr.

## Links

- [[Crate roles and dependency direction]]

Source: docs/metrics.md, lines 1-40
