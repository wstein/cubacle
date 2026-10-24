---
id: 20261024133004
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Speeding up the pattern search follows a fixed order: profile it, then reduce states by cube symmetry and commuting macros, store a 64-bit hash with the move list instead of whole states, spread the expansion over the cores, and only if applying macros still dominates add an aarch64 NEON permutation in one small unsafe module with a plain Rust fallback and a byte-for-byte equality test. A GPU is rejected because the work is random table access, not arithmetic; std::simd is not stable and the minimum Rust version stays 1.85.

## Links

- [[Pattern shortcut finds hand-built cubes]]
- [[Searches are capped at 1.5 GB and run one at a time]]

Source: owner decision of 2026-10-24, lines 1-1
