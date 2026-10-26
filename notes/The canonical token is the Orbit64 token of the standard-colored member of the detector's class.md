---
id: 20261025210200
aliases: []
tags: ["type/design", "status/open", "category/notation"]
---
The spike for a universal pattern identifier takes the detector's canonical form (the least sticker string over the 48 symmetries with colors named by first appearance), renames its colors to the Orbit64 scheme and encodes it, with no mirroring step because the reachable states are closed under mirroring. The reference is flix-orbit64 v0.9.0 (`FORMAT.md`, "Canonical form"); Rust follows it and passes its 93 shared vectors and pinned failures in `tests/canonical_vectors.txt`. An odd cube names the colors by its fixed centers, so the frame digit is 0; an even cube has no centers, so the URF corner is made the solved URF corner and the colors that share no corner with it name the other three faces. The token is invariant and stays in the class, so tokens and classes correspond one to one; on the 2x2x2 all 3 674 160 states give 77 802 tokens, the same as the Burnside count. It is not the numerically smallest token of its class, mirror images share a token, and the identifier depends on the Orbit64 draft slot numbering, so the owner has still to decide whether it ships.

## Links

- [[Orbit64 reads colors so equal-colored pieces need no representative]]
- [[Orbit64 tokens]]
- [[Orbit64 slot numbering caveat]]
- [[The fingerprint is the least reading over every frame and axis order]]
- [[A pattern table is read by structure and every line is replayed]]
- [[A picture list stops at 24 patterns and says how many it cut]]

Source: src/canonical/mod.rs, lines 1-20
Source: src/canonical/mod.rs, lines 79-100
Source: src/canonical/tests.rs, lines 339-384
