---
id: 20261024133005
aliases: []
tags: ["type/decision", "status/decided", "category/process"]
---
The stable toolchain moves from 1.96.1 to 1.99.1 only after the integration queue is empty, so no check runs on two compilers. Before and after the update the solver and the pattern search are timed and every solution is compared byte for byte; new lints are fixed in their own commit. The release notes show no feature we could use, because rust-version stays at 1.85 (the msrv job measures it) and newer standard library functions are therefore off limits; the possible gain is the LLVM 23 code generation, which has to be measured.

## Links

- [[Validate in CI]]
- [[Crate roles and dependency direction]]

Source: Cargo.toml, lines 1-10
Source: owner decision of 2026-10-24, lines 1-1
