---
id: 20261025233000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`solve --to T --now N` maps the original cube S and the cube as it is N to their relative states against T, and runs the unchanged recover logic on them. The recover engine assumes the solved cube in one place only (the check that the printed solution ends solved, `src/recover/mod.rs`), and the command checks the fresh solution against it; the conflict was declared by `conflicts_with_all` on `--now` in `src/main.rs`. Relative states are left-invariant, so a move sequence that takes the relative N to solved takes the real N to T; equal pieces are only centres, and a supported target keeps them on one face, so pairing them freely does not change a color. The pictures and the end state use the real cubes, so they end in the target. A target the relative state refuses is refused with the same words as `solve --to`; a target with open stickers is refused because it is a set of states, not one.

## Links

- [[A target is solved as a relative state]]
- [[The recover command is folded into solve --now]]

Source: src/recover_cmd.rs, lines 56-90
