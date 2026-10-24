---
id: 20261024180005
aliases: []
tags: ["type/question", "status/open", "category/engines"]
---
patterns cycles only prints. Reading our cycle spelling back into a state (so that a typed cycle list builds a cube) is not done: it needs a grammar for slots on every size, a check that the cycles are a legal permutation of one orbit, and a rule for wings whose slots are named by position. The rebuild from the in-memory cycles already exists and is the base for it.

## Links

- [[A pattern detector reads structure from the cubie state]]
- [[The detector explains patterns and never replaces the search]]

Source: src/patterns/detector/cycles.rs, lines 280-330
