---
title: "Colour schemes"
description: "Specification: named, custom and file-based colour schemes that map six face roles to letters and display colours."
weight: 20
---

# Colour schemes

Status: specification, not implemented yet. Nothing on this page can be run.

A colour scheme maps the six face roles (U, R, F, D, L, B) to six one-letter codes and gives each code a display colour. It is independent of any generator; the [.gen file](gen-file.md) only uses it to read fixed colours.

## Built-in schemes

| Name | Letters (U R F D L B) | Faces |
| --- | --- | --- |
| `URFDLB` (default) | U R F D L B | Identity: each letter names the face it sits on. |
| `western` (alias `boy`) | W R G Y O B | White up, red right, green front, yellow down, orange left, blue back. |
| `japanese` | W R G B O Y | As western with blue and yellow exchanged: white up, red right, green front, blue down, orange left, yellow back. |

Sources. Western pairs white with yellow, red with orange and blue with green ([Ruwix](https://ruwix.com/the-rubiks-cube/japanese-western-color-schemes/)). The Japanese scheme pairs white with blue, yellow with green and red with orange, and differs from the western one by exchanging yellow and blue ([Speedsolving forum thread on the Japanese colour scheme](https://www.speedsolving.com/threads/japanese-colour-scheme.2537/)). The up and front assignment of the western row is the usual convention for a white-up cube. The sources found do not agree on the sense of rotation of the Japanese scheme, so the Japanese row is derived from the exchange rule above and must be checked against a physical cube before release.

## Custom schemes

A custom scheme is six letters, in the order U R F D L B, for example `ABCDEF`. The letters are the codes of the six faces.

## Display colours

Each code can be given a display colour as `#RGB` or `#RRGGBB`. Hex digits are case-insensitive. Nothing else is accepted: no names, no `rgb(...)`.

```ebnf
scheme-spec = scheme-name | letters ;
scheme-name = "URFDLB" | "western" | "boy" | "japanese" | file-name ;
letters     = letter letter letter letter letter letter ;
letter      = "A".."Z" ;
colour-list = assignment { ws assignment } ;
assignment  = letter "=" colour ;
colour      = "#" hex hex hex [ hex hex hex ] ;
hex         = "0".."9" | "a".."f" | "A".."F" ;
ws          = " " | "," ;
```

`#RGB` expands to `#RRGGBB` by doubling each digit: `#d00` is `#dd0000`.

## Scheme file

A standalone scheme file ends in `.scheme`. It is line-based: `key: value`, and `#` starts a comment.

```text
# japanese.scheme
name: japanese
faces: WRGBOY
colours: W=#fff R=#d00 G=#0a0 B=#00d O=#f80 Y=#ee0
```

```ebnf
scheme-file = { line } ;
line        = ( blank | comment | entry ) newline ;
comment     = "#" { any } ;
entry       = key ":" { " " } value ;
key         = "name" | "faces" | "colours" ;
```

`name` is optional and has the form of a pattern id (lowercase a-z, 0-9, `-`). `faces` is required and holds six letters. `colours` is optional; without it the codes have no display colour and the display uses its own default.

## Using a scheme

`--scheme FILE` takes a file, `--scheme NAME` takes a built-in name, and six letters given inline are a custom scheme. A name is looked up as a built-in first, then as a file.

```text
cubacle solve --scheme japanese ...
cubacle solve --scheme japanese.scheme ...
cubacle solve --scheme ABCDEF ...
```

## Validation

- The six letters must be distinct. A custom letter may not repeat.
- Letters are uppercase A-Z.
- Every letter in `colours` must occur in `faces`.
- A display colour must match the `colour` rule above.
- Two codes with the same display colour give a warning, not an error.

Errors name the line and column and what was expected.

## Where a scheme applies

1. Input parsing of colour codes.
2. The `scheme:` key in a [.gen schema header](gen-file.md): it may name a scheme, give a file, or give six inline letters.
3. Display and move icons, later.

## Defaults

Without the option every command behaves exactly as before, byte for byte. With no scheme header, the scheme is `URFDLB`.
