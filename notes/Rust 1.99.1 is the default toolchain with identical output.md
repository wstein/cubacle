---
id: 20261024200003
aliases: []
tags: ["type/process", "status/done", "category/process"]
---
On 2026-10-24 the default toolchain moved from 1.96.1 to 1.99.1 after a light measurement in CPU time on a loaded machine: solutions byte-identical on 3x3, 4x4, 5x5 and 7x7, formatting, lints and tests clean, speed within noise (3x3 and 4x4 up to 6 percent faster, 7x7 4 percent slower, to be measured again on a quiet machine). The repository pins no toolchain and rust-version was 1.85 then and is 1.95 since 2026-10-25, which the MSRV job checks.

## Links

- [[The Rust toolchain is raised after the queue, the minimum version is 1.95]]

Source: Cargo.toml, lines 1-10
