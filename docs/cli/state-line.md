---
title: "State line"
description: "The state: and target: lines on standard error, and --state-format."
weight: 14
---

# State line

## State line

A command that starts from a cube or aims at one prints that cube as a line on **standard error**. Standard output does not change, so `cubacle scramble | cubacle solve -` and every other pipe keep working.

```text
state: orbit64v2 AwtB4B1fWQAA
target: orbit64v2 HYKC9_HL1gAA
```

`state:` is the cube the command starts from, `target:` the cube it aims at. `--state-format` picks how the cube is written:

| `--state-format` | The line shows |
| --- | --- |
| `orbit64v2` (default) | The [Orbit64 V2](convert.md#orbit64-v2-tokens) token, color model, framed pose: the whole-cube orientation stays in the token, so it describes the state exactly. `cubacle` reads a color framed token back (5, 12, 27, 42, 66 or 90 characters). A canonical-pose length (4, 11, 26, 65 or 89) is not taken as a token, so a move expression of that length stays a move expression |
| `orbit64` | The same color framed V2 token as `orbit64v2` (`convert --to token`) |
| `facelets` | Canonical URFDLB facelets |
| `color` | W/R/G/Y/O/B sticker letters (`boy`, or the scheme given with `--scheme`; `photo` uses the scheme it detected in color input) |
| `none` | No line |

### Which commands

| Command | Lines |
| --- | --- |
| `solve` | `state:` for the cube to solve; with `--to`, `target:` for the target. With `--now` (the recovery of a mistake, see [Recover](recover.md)) also `now:` for the cube as it is now |
| `moves` | `state:` for the cube the moves start from (the solved cube, or the cube of `--from`); `target:` for the cube after the moves. `--picture` starts from a picture token, which the color formats cannot show: both lines say `unavailable` |
| `scramble` | `state:` for the cube the printed scramble makes |
| `random` | `state:` for the random cube |
| `facelets` | `state:` for the cube the scramble makes |
| `photo` | `state:` for the cube that is photographed |
| `patterns find` | `target:` for the cube the generator builds from the solved cube |

`convert` and `canon` print the state (or its canonical token) as their result on standard output and add no line. `order`, `template` and the rest of `patterns` (`list`, `show`, `identify`, `cycles`, `import`, `update`, `sources`) have no start or target cube.

### A state that cannot be shown

A line that cannot be written says why and the command goes on; the exit code is the one the command has without the line:

```text
target: unavailable: open stickers: the target is a set of states, not one cube
```

This happens for a `--to` target with open stickers (`X`, or a name like `cross`), which stands for many cubes, and for the `--picture` start of `moves`.

### JSON

`solve --json` appends the object `state` after the other keys, with the start cube in all four formats: `orbit64v2`, `orbit64`, `facelets` and `color` (`null` for a format that cannot show the state). With `--results N`, every element of the array carries the same `state` of the start cube. `--state-format` does not change it. The server does not send this object for `/v1/solve`; its other state descriptions carry [`tokenV2`](../api.md#the-tokenv2-field).
