---
title: "Move icons"
description: "Draw moves as small pictures for the terminal, the web and print."
weight: 4
---

# Move icons

`cubacle icon` draws a move on a cube as a small picture, and `cubacle solve --icons` shows every step of a solution that way. The same code is available as the `cubacle::render` module, which returns SVG, RGBA pixels or Kitty graphics escapes for a frontend or a PDF export.

## What an icon shows

An icon shows the Front, Up and Right faces of the cube in an oblique view, with every sticker of the N×N×N grid (2 to 7) in the real colours of the cube state before the move.

- **Moving layers** keep their colours at full brightness and everything else is darkened to 45 percent. With `--grey`, the moving layers turn grey instead.
- **A red arrow with a white halo** lies on the face where the moving layer is visible, along the whole layer. A "2" badge on the arrow marks a half turn.
- **A curved cyan arrow** over the whole cube marks the rotations `x`, `y` and `z`.
- **The notation** is written beside the cube.

| Move | Arrow lies on | Direction |
| --- | --- | --- |
| `R`, `Rw`, `2R` | the Front face, over the turning column | up |
| `L` | the Front face, left column | down |
| `U` | the Front face, top row | left |
| `D` | the Front face, bottom row | right |
| `F` | the Right face, front column | down |
| `B` | the Right face, back column | up |

A prime (`'`) reverses the direction. The direction is derived from the rotation itself and is tested against `Facelets::apply`; it is not drawn by eye.

### Notation in captions

The caption is the canonical notation of the move. `Rw` and `Rw2` turn the two outer layers, `2R` and `2R2` turn only the second layer, and `3Rw` turns the three outer layers. Lowercase `r` is not accepted. The arrow is centred on the turning layers, so a wide move has its arrow between its layers.

## Draw an icon

```sh
cubacle icon R "U'" Rw2 --cube 4
```

Each move is drawn on its own start cube, the solved cube or the cube from `--from`, as in a move sheet. With `--cumulative`, each move is drawn on the cube left by the moves before it. On a terminal without graphics, the output is text; on a pipe it is plain text without escape sequences:

```text
R ↑
   w w W
  w w W
 w w W
g g G R R R
g g G R R R
g g G R R R
```

Moving stickers are uppercase and the others lowercase.

| Option | Meaning |
| --- | --- |
| `--format auto\|svg\|rgba\|kitty\|text` | The output format. `auto` (the default) uses Kitty graphics on a capable terminal and text otherwise. |
| `--size N` | Icon height in pixels. Default: 32 per cube layer, at least 96 (96 for the 2x2x2 and 3x3x3, 128 for the 4x4x4, 224 for the 7x7x7), so stickers stay readable on larger cubes. The width is 1.5 times the height. |
| `--cube N` | Cube size, 2 to 7, 3 by default. |
| `--out FILE` | Write to a file instead of standard output. |
| `--scheme boy\|jp` | Colour scheme, `boy` by default; six letters in `URFDLB` order also work. |
| `--from SCRAMBLE` | Start from the cube that this scramble produces. |
| `--transparent` | Keep the Kitty background transparent (opaque dark by default). |
| `--cumulative` | Draw each move on the cube left by the moves before it. |
| `--grey` | Grey the moving layers instead of darkening the rest. |

`svg` and `rgba` hold one icon, so they take one move. `rgba` writes `width × height × 4` bytes of straight (not premultiplied) RGBA. Graphics escapes are never written to a standard output that is not a terminal, and raw pixels are never written to a terminal; use `--out FILE` in those cases.

```sh
cubacle icon "Rw'" --cube 5 --format svg --out rw.svg
```

## Solve with icons

```sh
cubacle solve --scramble "R U" --icons
```

Each step prints `step i of n: MOVE` and then the icon of the cube before that step. Without `--icons`, the output of `solve` does not change. The colours come from `--scheme` (`boy` by default).

## Terminal support

`cubacle` writes Kitty graphics when standard output is a terminal and the environment names a terminal that supports the [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/): `KITTY_WINDOW_ID` is set, `TERM` is `xterm-kitty`, or `TERM_PROGRAM` is `ghostty` or `WezTerm`. Other terminals get coloured text, or plain text when `NO_COLOR` is set, `TERM` is `dumb` or standard output is not a terminal.

The Kitty output transmits raw RGBA (`a=T,f=32`) in chunks of at most 4096 base64 bytes, with `m=1` on every chunk but the last. It first erases the terminal rows that the image covers, steps back up, draws the image with `C=1` (the cursor does not move) and then steps down below it, so older text cannot show through and later output does not overlap. The icon has an opaque dark background by default; `--transparent` keeps it transparent. Only the height in rows (`r`) is sent, so the terminal keeps the aspect ratio whatever its cell shape; the row count assumes about 16 pixels per row, which is an estimate and not queried. The encoding is covered by unit tests; the drawing was seen in a Kitty-protocol terminal by the owner.

## Use the module

```rust
use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle::render::{icon, Action, KittyPlacement, Style};

let state = Facelets::solved(PuzzleKind::Cube5);
let action: Action = "2R".parse().expect("a move");
let drawn = icon(&state, &action, &Style::default()).expect("2R exists on a 5x5");
let svg = drawn.to_svg();
let pixels = drawn.to_rgba(128);
let escapes = pixels.to_kitty(&KittyPlacement::default());
assert!(svg.starts_with("<svg") && escapes.starts_with("\x1b_G"));
```

`render` depends only on `puzzle`, does no I/O and keeps no state. The SVG is built from filled polygons with two-decimal numbers, so the same input always gives the same text. The `Style` sets the palette, the shading and a target size in pixels; the size only widens gaps and arrows so that icons at 64 to 128 pixels stay crisp, and the SVG scales freely.

Examples of the SVG output are in `tests/fixtures/icons/`.

## Limits

- PDF export, animation and a web frontend are not part of this module yet.
- Inner slices use the straight arrow; there is no curved arrow for them.
- Geometry and arrow directions are tested for 2×2×2 to 5×5×5 (geometry) and 3×3×3 to 5×5×5 (arrows against `Facelets::apply`). The 6×6×6 and 7×7×7 use the same code and are only tested for shape counts and a stable SVG.
