---
id: 20261023120000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
A follower of `solve --icons` on a 7x7x7 drifted (quarter turn done as half turn, skipped moves) and noticed 4 to 53 moves late. The output therefore gives the start cube once as an Orbit64 token and the cube after every phase with moves, as a plain line (`State after phase N as token: TOKEN`). The token is the text `convert --to token` writes, so it can be pasted into a recover command that compares the start cube with the cube in hand. Plain `solve` stays unchanged.

## Links

- [[Orbit64 tokens]]
- [[Icon sheet export for documents]]

Source: owner request of 2026-10-23, lines 1-1
