---
id: 20261024160002
aliases: []
tags: ["type/process", "status/open", "category/process"]
---
Intel and AMD hosts with 96 GB of memory are available to the owner from the week after 2026-10-24, for the optimal mode and larger pruning tables. The Apple M2 with 16 GB stays the development machine and sets the default: the table size is chosen by --max-memory, the default fits the small machine, and the large tables (7.3 GiB, perhaps 21 GiB) are measured on the big hosts. Builds and measurements on those hosts are started by the owner or by a CI runner; the agent opens no remote sessions and installs nothing. Results are recorded with CPU model, memory, table size, time and move counts.

## Links

- [[An optimal mode is a measured clean-room design]]

Source: docs/patterns.md, lines 1-10
