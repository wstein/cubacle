---
id: 20261025185100
aliases: []
tags: ["type/design", "status/decided", "category/notation"]
---
`cubacle order` defines the order of a move sequence on the stickers: the sequence is a permutation of the sticker places, and the order is the least common multiple of its cycle lengths. This needs no cubie model, so it is the same on the 2x2x2 to the 7x7x7 and exact for any expression the parser accepts. On the 3x3x3 it equals the cubie view, where a twisted or flipped piece multiplies its cycle length by 3 or 2; a test compares the two. On the 4x4x4 and up stickers are told apart where colors are not, so a turned center counts and the order can be a multiple of the repetition count at which the colors look solved. `-v` splits the cycles by orbit (corners, edges, wings, centers) and the orbit orders have the whole order as their lcm.

## Links

- [[Cycles use our own spelling and the order comes from the cycle type]]
- [[A symmetry is a sticker permutation plus a colour map]]
- [[Move input rules are WCA and SiGN]]

Source: src/puzzle/order.rs, lines 1-7
