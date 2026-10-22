---
id: 20261022160003
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
The 12 pruning tables of the five phases are exact breadth-first distances, so they are byte-identical to the Java ones for any thread count. The ignored tests compare them with the serialized int arrays that CUBE555_TABLES points at (a 27-byte header, then big-endian ints), and a second ignored test replays the Java reduction of 24 cubes from tests/fixtures/five_transfer.txt.

## Links

- [[5x5 reduction engine]]

Source: src/engines/five/testing.rs, lines 7-11
Source: src/engines/five/reduction.rs, lines 228-236
Source: tests/five_full_test.rs, lines 1-10
