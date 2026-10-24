---
id: 20261024184502
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The move step of the optimal search (12 edge bytes, 8 corner bytes, and the same for the inverse cube) is a byte-table lookup, so `engines/optimal/vector.rs` has a NEON version (`vqtbl2q_u8`, `vqtbl1q_u8`) on aarch64 and the portable loop elsewhere, chosen at compile time. It is the only `unsafe` code of the engine, a test compares it with the portable step on random cubes, and every intrinsic used is stable in Rust 1.85. For 110.5 million nodes on the same cubes the search took 10.4 to 11.8 s of CPU with NEON and 16.9 to 19.5 s without, so it stays. The next step is AVX2 on the owner's big hosts: `vector::turn(cube, tables, move)` is the whole interface, so an x86 version (`vpshufb` on 16-byte halves, chosen with `is_x86_feature_detected!`) replaces that one function.

## Links

- [[Vector code exists for NEON, AVX2 and AVX-512 with one fallback]]
- [[The optimal mode bounds its search by corner and edge databases]]

Source: docs/engines/3x3.md, lines 133-133
