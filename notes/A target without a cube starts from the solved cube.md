---
id: 20261025231500
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
When `solve --to TARGET` has no cube and no `--scramble`, the start is the solved cube of the target's size, so the answer is the route from solved to the target: how to build the pattern. A target that is a cube state gives its own size (an explicit `--size` that disagrees is an error); a move expression or a pattern name uses `--size`, default 3. One line on stderr says so. Without `--to` the missing cube stays an error, now with a hint. `--now` conflicts with `--to` already and needs a real cube.

## Links

- [[A target is solved as a relative state]]

Source: owner decision of 2026-10-25, lines 1-1
