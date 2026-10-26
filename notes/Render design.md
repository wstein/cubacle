---
id: 20261022130002
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
Move icons live in a new top-level module `render` that depends only on `puzzle`. It never names `notation`: the CLI turns a `ColorScheme` into a `Palette` through six letters, which keeps the module free to become its own crate. It is pure: `icon(&Facelets, &Action, &Style)` builds a list of filled polygons in a 114 x 109.2 canvas (the cube above, the notation below) that four writers consume (an SVG string, an RGBA raster from an own supersampling scanline filler, Kitty graphics escapes, and a text grid for terminals without graphics). Terminal detection and all I/O stay in the CLI (`src/icons.rs`).

## Projection and layers

The view is a deep oblique one: `sx = x - 0.5 z`, `sy = y - 0.42 z` in the axes of `puzzle::locate`, so Front, Up and Right are visible, for any size `n`. The scale is the box size over `2n * 1.4`. A layer moves when `side * position[axis]` equals (single layer) or is at least (wide) `n - 1 - 2(depth - 1)`, the rule `Facelets::apply` uses, so `Rw` is two layers, `2R` one inner layer and `3Rw` three. Moving layers keep their colors and the rest is darkened to 25 percent (dark backdrop) or 40 percent (light backdrop, `Style::backdrop`); `Shading::GreyMoving` greys the moving layer instead. The default is darkening because the real colors of the moving layer matter more than a uniform grey.

## Arrow direction

A move turns by `sense = -side * dir` about the positive axis (`dir` is +1 clockwise, -1 counter-clockwise; half turns draw the clockwise arrow with an upright "2x"). A point `p` moves with velocity `sense * e_axis x p`. The arrow lies on the Front plane for x and y axes and on the Right plane for z (so F and B show on the Right face), along the middle line of the active band and across the full length of the face, pointing along that velocity. It is built in plane coordinates and projected with the stickers, so it shares their shear. Tests compare it with `Facelets::apply`: the stickers that leave the arrow plane must arrive in the direction the arrow points, for N=3, 4 and 5, and the arrow must lie over turning stickers. Whole-cube rotations use a curved arrow. The arrow is soft (round tail, long head with rounded corners, white outline with round joins), has a constant size on the canvas graded by cube size and keeps a 12 percent margin. The notation and the `2x` are Quicksand Bold outlines embedded as polygons, so every writer draws the same letters.

## Deferred

PDF export, animation, a frontend, and curved rotation arrows for inner slices (they use the straight arrow).

## Terminal output

Kitty output erases the covered rows, draws with `C=1` and steps below the image, is transparent unless `--opaque` and sends only `r` so the terminal keeps the aspect ratio. `moves --icons --format ...` draws each move on its own start cube; `--cumulative` replays, as `solve --icons` does. Every drawing keeps a 2 px margin, tested for sizes 2 to 7.

## Links

- [[Crate roles and dependency direction]]

Source: src/render/action.rs, lines 136-147
Source: src/render/arrow.rs, lines 48-93
Source: src/render/arrow.rs, lines 297-340
Source: src/icons.rs, lines 64-82
Source: docs/icons.md, lines 1-10
