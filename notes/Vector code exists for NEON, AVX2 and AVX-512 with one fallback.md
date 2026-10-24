---
id: 20261024160001
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Hot loops with byte permutations (optimal search, pattern search, 5x5 and 7x7 tables) get one portable Rust version and optional vector versions for NEON (Apple, ARM), AVX2 and AVX-512 (Intel, AMD). The owner decided on 2026-10-24 that all three are supported. Each vector version is a small unsafe module that is chosen at run time (is_x86_feature_detected on x86, NEON by target on ARM) and checked by a test against the Rust fallback on random input, so the result is identical on every host. Nothing is built with target-cpu=native, and the minimum Rust version stays 1.85, so the intrinsics used must exist there; the std::simd module is unstable and not used. A vector version is only added after a profile shows the loop dominates, and only if it is measurably faster.

## Links

- [[An optimal mode is a measured clean-room design]]

Source: docs/patterns.md, lines 1-10
