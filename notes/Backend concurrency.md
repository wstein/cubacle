---
id: 20261022120014
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
Solving runs on dedicated threads, never on async threads, so the async runtime is not blocked. Concurrency is bounded and overload answers 429 with Retry-After. Engines are prepared once at startup.

## Links

- [[Backend stack and protocol]]

Source: project history of cubacle, lines 1-1
