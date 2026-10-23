---
title: "Move icons"
description: "Draw moves as small pictures for the terminal, the web and print."
weight: 4
---

# Move icons

A move icon (a "minicube": a small cube picture with an arrow, also called an arrow diagram) shows one move on the cube. `cubacle icon` draws a move as such a picture, and `cubacle solve --icons` shows every step of a solution that way. The same code is available as the `cubacle::render` module, which returns SVG, RGBA pixels or Kitty graphics escapes for a frontend or a PDF export.

## What an icon shows

An icon shows the Front, Up and Right faces of the cube in a deep oblique view, with every flat, bright sticker of the N×N×N grid (2 to 7) in the real colours of the cube state before the move.

- **Moving layers** keep their colours at full brightness and everything else is darkened: to 25 percent on a dark background and to 40 percent on a light one (see `--background`). With `--grey`, the moving layers turn grey instead.
- **A soft violet arrow with a white outline** lies on the face where the moving layer is visible, along the layer with a margin of 12 percent of the face width at both ends. It has a round tail and a head about twice as long as wide with rounded corners. Its size stays the same on the canvas for a given cube size: 4.0 canvas units wide on the 2x2x2, narrowing to 2.8 on the 7x7x7. On the Right face it is foreshortened like the face.
- **An upright violet "2x"** with a white outline, the same height as the notation, marks a half turn. It sits right of a vertical arrow and above a horizontal one.
- **A curved violet arrow** over the whole cube marks the rotations `x`, `y` and `z`.
- **The notation** is written below the cube in Quicksand Bold, always in the same size and on the same baseline. It is light on a dark background and dark on a light one.

| Move | Arrow lies on | Direction |
| --- | --- | --- |
| `R`, `Rw`, `2R` | the Front face, over the turning column | up |
| `L` | the Front face, left column | down |
| `U` | the Front face, top row | left |
| `D` | the Front face, bottom row | right |
| `F` | the Right face, front column | down |
| `B` | the Right face, back column | up |

A prime (`'`) reverses the direction. The direction is derived from the rotation itself and is tested against `Facelets::apply`; it is not drawn by eye.

### Notation

The notation below the cube is the canonical notation of the move. `Rw` and `Rw2` turn the two outer layers, `2R` and `2R2` turn only the second layer, and `3Rw` turns the three outer layers. Lowercase `r` is not accepted. The arrow is centred on the turning layers, so a wide move has its arrow between its layers.

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
| `--size N` | Icon height in pixels. Default: 32 per cube layer, at least 96 (96 for the 2x2x2 and 3x3x3, 128 for the 4x4x4, 224 for the 7x7x7), so stickers stay readable on larger cubes. The canvas is 114 by 109.2 units, so the width is about 1.04 times the height (100 by 96 pixels by default). |
| `--cube N` | Cube size, 2 to 7, 3 by default. |
| `--out FILE` | Write to a file instead of standard output. |
| `--scheme boy\|jp` | Colour scheme, `boy` by default; six letters in `URFDLB` order also work. |
| `--from SCRAMBLE` | Start from the cube that this scramble produces. |
| `--background dark\|light` | What the icon is seen on, `dark` by default. It sets the dimming (25 or 40 percent) and the colour of the notation (light or dark). |
| `--opaque` | Paint a solid background of the `--background` colour (`#20242b` for dark, white for light) behind SVG, RGBA and Kitty output. The default is transparent, so the picture takes the colours of the terminal or page. |
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

On a Kitty-graphics terminal the icons come in rows, as many side by side as fit the width of the terminal, and each row has a line above it such as `steps 9-16 of 128`. A row is one image, with 8 pixels between the icons, and the background stays transparent. The last line is the plain move string. Without graphics (a pipe, `NO_COLOR`, `TERM=dumb` or another terminal), each step prints `step i of n: MOVE` and then its text picture, as before. Without `--icons`, the output of `solve` does not change. The colours come from `--scheme` (`boy` by default).

| Option | Meaning |
| --- | --- |
| `--icon-size PIXELS` | Icon height; all icons have the same size. Default: 32 per cube layer, at least 96. |
| `--columns N` | Icons per row. Default: as many as fit. Wide rows are cut off by the terminal, so a large `N` is your own choice. |
| `--background dark\|light` | What the icons are seen on. Default: asked from the terminal, dark when it does not answer. |

### How the terminal is asked

Before drawing, `solve --icons` asks the terminal for two things, only when standard input and output are a terminal with Kitty graphics:

- **Width.** `TIOCGWINSZ` gives the columns and, in most terminals, the size in pixels. When the pixel size is 0, the reply to `CSI 14 t` (text area in pixels) is used, then `CSI 16 t` (cell size) times the columns, and last `columns * 8` pixels (an assumed cell width; `COLUMNS` or 80 for the columns). The same sources give the cell height that moves the cursor below a row, with 16 pixels assumed.
- **Background.** The reply to `OSC 11` (`ESC ] 11 ; ? ESC \`, answered as `rgb:RRRR/GGGG/BBBB`) is judged by its brightness (0.2126 R + 0.7152 G + 0.0722 B above 0.5 is light). Without `--background` and without a reply the background is dark. With `--background`, the colour is not asked.

The queries go through `/dev/tty` in a short raw mode, and the saved settings are restored on every path, also on an error. A DA1 query (`CSI c`) is sent last: a terminal that does not know the other queries still answers it, so the program does not wait. If nothing answers, the wait is 100 ms.

Inside `tmux` and `screen` (`TMUX` or `STY` is set, or `TERM` starts with `screen` or `tmux`) nothing is asked, because they would swallow the queries and the images need a passthrough that is not implemented. The defaults apply there: the width from `TIOCGWINSZ` or `columns * 8`, and a dark background.

## Terminal support

`cubacle` writes Kitty graphics when standard output is a terminal and the environment names a terminal that supports the [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/): `KITTY_WINDOW_ID` is set, `TERM` is `xterm-kitty`, or `TERM_PROGRAM` is `ghostty` or `WezTerm`. Other terminals get coloured text, or plain text when `NO_COLOR` is set, `TERM` is `dumb` or standard output is not a terminal.

The Kitty output transmits raw RGBA (`a=T,f=32`) in chunks of at most 4096 base64 bytes, with `m=1` on every chunk but the last. It first erases the terminal rows that the image covers, steps back up, draws the image with `C=1` (the cursor does not move) and then steps down below it, so older text cannot show through and later output does not overlap. The icon background is transparent by default, so it takes the colours of a terminal and does not drift against other backgrounds. `--opaque` paints a solid backdrop of the `--background` colour instead (`icon` only). In `solve --icons` the image is drawn at its own pixel size, without `r`, so a row is as wide as computed; the number of rows to erase is its height divided by the cell height, rounded up. For `icon`, only the height in rows (`r`) is sent, from an assumed 16 pixels per row. See [How the terminal is asked](#how-the-terminal-is-asked) for the width and the background. The encoding is covered by unit tests; the drawing was seen in a Kitty-protocol terminal by the owner.

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

`render` depends only on `puzzle`, does no I/O and keeps no state. The SVG is built from filled polygons with two-decimal numbers, so the same input always gives the same text. The `Style` sets the palette, the shading, the `Backdrop` (dark or light), an optional solid background and a target size in pixels; the size only widens gaps and arrows so that icons at 64 to 128 pixels stay crisp, and the SVG scales freely.

The notation and the `2x` are glyph outlines of Quicksand Bold, converted to polygons and embedded in the program, so SVG, pixels and Kitty graphics look the same and no font is needed. The font is under the SIL Open Font License; see `NOTICE.md` in the repository.

Examples of the SVG output are in `tests/fixtures/icons/`.

## Limits

- PDF export, animation and a web frontend are not part of this module yet.
- Inner slices use the straight arrow; there is no curved arrow for them.
- Geometry and arrow directions are tested for 2×2×2 to 5×5×5 (geometry) and 3×3×3 to 5×5×5 (arrows against `Facelets::apply`). The 6×6×6 and 7×7×7 use the same code and are only tested for shape counts and a stable SVG.
