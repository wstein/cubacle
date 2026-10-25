---
id: 20261024133005
aliases: []
tags: ["type/decision", "status/decided", "category/process"]
---
The stable toolchain moves from 1.96.1 to 1.99.1 only after the integration queue is empty, so no check runs on two compilers. Before and after the update the solver and the pattern search are timed and every solution is compared byte for byte; new lints are fixed in their own commit. The release notes show no feature we could use, because rust-version is 1.95 (the msrv job measures it; raised from 1.85 on 2026-10-25 on the owner's order, since sysinfo 0.39 requires 1.95) and standard library functions newer than 1.95 are therefore off limits; the possible gain is the LLVM 23 code generation, which has to be measured.

## Links

- [[Validate in CI]]
- [[Crate roles and dependency direction]]

Source: Cargo.toml, lines 1-10
Source: owner decision of 2026-10-24, lines 1-1
Source: owner decision of 2026-10-25, lines 1-1
