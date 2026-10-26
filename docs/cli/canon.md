---
title: "Canonical token"
description: "The canonical Orbit64 token of a cube: one identifier for a cube and its turned, mirrored and recolored images."
weight: 10
---

# Canonical token

`cubacle canon CUBE` prints the canonical Orbit64 token of a cube state: one identifier for the pattern the cube shows. A pattern is a color state, so the identifier must not depend on how the cube is turned, mirrored or colored. The token is the same for a cube, its rotated and mirrored images, and any renaming of its six colors. How the token is made is in the [concept paper](../design/canonical-form.md).

`canon` reads a cube like [`convert`](convert.md): an Orbit64 token, URFDLB facelets, W/R/G/Y/O/B colors, or a move expression for a cube of `--size`, or `-s` with a scramble; see [Input notations](input-notations.md). Sizes are 2x2x2 to 7x7x7. By default the output is one line with the token alone, so scripts can use it:

```sh
cubacle canon -s "R U"
cubacle canon -s "L' U'"
cubacle canon C0RkZMybxEmw
cubacle canon --size 7 -s "R U Rw"
```

```text
C0RkZMybxEmw
C0RkZMybxEmw
C0RkZMybxEmw
Dki5KH1XWMQR-MYl-jyNBoh54zmGNjlON6_sDBROhBhzl8DJ7cw2FaslOb-HhRAA5OvOUzsxSlROfgJt0Cx98MDpAA
```

`L' U'` is the mirror image of `R U`, and the token `C0RkZMybxEmw` is its own canonical token (reading it back gives it back). The solved cube has the token `AAAAAAAAAAAA` on the 3x3x3 and `AAAAA` on the 2x2x2.

## Schema and size

The token depends on the slot numbering of Orbit64, which is still a draft. A stored identifier therefore carries the name of that convention, `orbit64-Nxn-draft@1` (N is the size). `-v` prints the token with the schema name and the size; `--format json` writes one JSON object with the same data:

```sh
cubacle canon -v -s "R U"
cubacle canon --format json --size 4 -s "Rw U"
```

```text
canonical: C0RkZMybxEmw
schema: orbit64-3x3-draft@1
size: 3x3x3
{"canonical":{"token":"AC6Z137Oxa2xaYfThBJmjjR64oQ","schema":"orbit64-4x4-draft@1"},"size":4}
```

The token is an identifier, not a search key: do not look patterns up by it in a file name, and do not read order into it (mirror images share a token, and tokens do not sort like the detector's order).

## Errors

A text that is no cube, or stickers that no turns reach in any coloring (a twisted corner, a parity), print `error: ...` on standard error and exit with 1:

```sh
cubacle canon UUUUUUUURFRRRRRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

```text
error: this is not a cube state (the corners are invalid: one corner is twisted on its own)
```

Unlike `solve`, `canon` accepts a recolored cube whose centers are not those of a standard cube, because renaming the colors is what it is for. `patterns identify` prints the same token as its `canonical:` line, and the REST server answers it as the field `canonical` (see [REST API reference](../api.md#the-canonical-field)).
