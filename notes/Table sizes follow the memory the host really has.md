---
id: 20261025101500
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Owner decision of 2026-10-25: the table set and every memory default (the optimal mode's Small and Large sets, later the 8-edge database) follow the physical and available memory the host reports at run time, never the platform or a fixed constant, so there is no cfg(target_os) or architecture based size. The reading is done by the sysinfo crate (version 0.39, rust-version 1.95, default features off, only `system`), which covers macOS, Linux and Windows without hand-written unsafe and without shelling out; its rust-version is why the minimum is 1.95. `src/ram.rs` holds it. Without `--max-memory` the optimal mode budgets half of the available memory (the total when none is reported), at most 16000 MB; an explicit value wins; when the memory cannot be read the Small set is used and stderr says so. The first stderr line names the tables, the memory seen and the budget, and a test injects the memory value so it needs no platform.

## Links

- [[The optimal mode bounds its search by corner and edge databases]]
- [[Large hosts measure the big tables, the Mac sets the default]]

Source: docs/cli.md, lines 158-186
