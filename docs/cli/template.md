---
title: "Check a template file"
description: "Check a template file of target stickers."
weight: 6
---

# Check a template file

## Check a template file

`cubacle template check FILE.cubacle` reads a template definition, a sticker grid that says which stickers share a color plus optional constraints on those colors, and reports whether it parses. It does not search and does not count; the file is an argument, there is no fixed directory.

The file is line-based, `key: value`. `#` starts a comment, a line that starts with white space continues the previous value, a blank line ends it. The keys are `pattern-name`, `cube-size` and `sticker-grid` (required) and `pattern-title`, `pattern-aliases`, `color-scheme`, `color-constraints`, `move-alphabet`, `search-limits` and `pattern-tags`; the keys for symmetry, order and orbits of the specification are accepted and not evaluated yet. The sticker grid lists all six faces in `URFDLB` order, `N*N` tokens each: a lowercase letter with an optional number (`a`, `b1`) is a placeholder named by the whole token, an uppercase letter is a fixed color read through the `color-scheme` (default `URFDLB`), `.` and `?` are don't care. `/` or a newline ends a face. A grid of one face (`N*N` tokens in all) stands for all six faces, see below. The color constraints combine `unique(...)`, `equal(...)` and `count(...)` with comparisons, `&&`, `||` and `!` (also `and`, `or`, `not`); `a*` stands for the same index in all arguments of one term. `docs/specs/definition-file.md` has the grammar.

```text
# concentric-squares.cubacle, a 5x5x5: per face corners a, edges b, crosses c
pattern-name: concentric-squares
pattern-title: Concentric squares
cube-size: 5
sticker-grid:
  a1 b1 b1 b1 a1 b1 b1 c1 b1 b1 b1 c1 U c1 b1 b1 b1 c1 b1 b1 a1 b1 b1 b1 a1 /
  a2 b2 b2 b2 a2 b2 b2 c2 b2 b2 b2 c2 R c2 b2 b2 b2 c2 b2 b2 a2 b2 b2 b2 a2 /
  a3 b3 b3 b3 a3 b3 b3 c3 b3 b3 b3 c3 F c3 b3 b3 b3 c3 b3 b3 a3 b3 b3 b3 a3 /
  a4 b4 b4 b4 a4 b4 b4 c4 b4 b4 b4 c4 D c4 b4 b4 b4 c4 b4 b4 a4 b4 b4 b4 a4 /
  a5 b5 b5 b5 a5 b5 b5 c5 b5 b5 b5 c5 L c5 b5 b5 b5 c5 b5 b5 a5 b5 b5 b5 a5 /
  a6 b6 b6 b6 a6 b6 b6 c6 b6 b6 b6 c6 B c6 b6 b6 b6 c6 b6 b6 a6 b6 b6 b6 a6
color-constraints: unique(a*, b*, c*)
```

```sh
cubacle template check concentric-squares.cubacle
```

```text
ok: concentric-squares, size 5, 18 variables, guard ok
```

The file is in `docs/static/examples/definitions/` (published at `https://wstein.github.io/cubacle/examples/definitions/`).

### A grid of one face

A symmetric definition needs only one face. When the grid has `N*N` tokens in all, they are the U face, the same pattern on all six faces; then `/` and newlines only break rows. A placeholder gets the number of its face appended (`a` is `a1` on U, `a2` on R, and so on up to `a6` on B), so `a*` in the constraints names the six of them. On a cube of odd size the center letter `U` stands for the letter of the face it is on. A number already in a token is kept and the face number comes after it (`b1` on R is `b12`).

```text
# checkerboard-free-center.cubacle
pattern-name: checkerboard-free-center
pattern-title: Checkerboard with free centers
cube-size: 3
sticker-grid: a b a / b . b / a b a
color-constraints: unique(a*, b*)
```

```sh
cubacle template check checkerboard-free-center.cubacle
```

```text
ok: checkerboard-free-center, size 3, 12 variables, guard ok
```

Fewer or more stickers than one face and not six faces are an error with the position, for example `error: short.cubacle:3:34: the grid of one face has 8 stickers, expected 9 (3x3)`.

Every `.cubacle` file in that directory is read (`checkerboard-free-center`, `concentric-squares`, `custom-scheme`, `ring-symmetric`), except `bad.cubacle`, which fails on purpose, see below. `ring-symmetric.cubacle` gives its symmetry with the key `required-symmetry: D4h`; the call `symmetric(D4h)` in `color-constraints` is not read yet.

`template check` reads the text form only. The `.json` files in the directory show the same definitions in the JSON form of the specification; `cubacle template check` does not read them yet and says `expected "key: value"`.

A file without `color-constraints` ends in `no guard`. A problem is reported as `error: file:line:column: message` on stderr with exit status 1, the position counted in the file. `bad.cubacle` in the same directory is the 3x3x3 grid of a cross with the constraint `unique(a*, b*, c*) && equal(a*, d*)`; the constraint names `d*`, which the grid does not have:

```sh
cubacle template check bad.cubacle
```

```text
error: bad.cubacle:11:52: no variable of the schema matches "d*"
```
