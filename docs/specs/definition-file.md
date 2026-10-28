---
title: "The definition file"
description: "Specification: the definition file that defines a pattern by a schema grid and a guard, and the grammar of both."
weight: 10
---

# The definition file

Status: specification, not implemented yet. Nothing on this page can be run.

## Purpose

A definition file defines a family of patterns by a schema grid (which stickers are equal) and a guard (a condition on the colors). It is given as a command argument; there is no fixed directory.

```text
cubacle patterns find concentric-squares.cubacle
```

By default the command only checks the definition and counts what fits it. Searching for a move sequence happens only with the explicit flag `--solve`.

```text
cubacle patterns find concentric-squares.cubacle --solve
```

## Principle

The definition file says what a pattern is: its name, grid, guard, symmetry, order and orbits. The name must mean the same pattern everywhere, in `identify` and in the library. Run options say how much to compute (`--limit`, `--count`, `--sample`, `--seed`, `--max-states`, `--max-time`, `--max-memory`, `--solve`, `--icons`). They never change the pattern.

## Layout

The file is line-based: `key: value`. `#` starts a comment that runs to the end of the line. A line that starts with white space continues the value of the previous key.

```text
# concentric squares on a 5x5x5
pattern-name: concentric-squares
pattern-title: Concentric squares
cube-size: 5
sticker-grid:
  a b b b a b b c b b b c U c b b b c b b a b b b a
color-constraints: unique(a*, b*, c*)
```

```ebnf
file         = { line } ;
line         = ( blank | comment | entry | continuation ) newline ;
blank        = { " " | tab } ;
comment      = { " " | tab } "#" { any } ;
entry        = key ":" { " " } value ;
continuation = ( " " | tab ) { " " | tab } text ;
key          = "pattern-name" | "pattern-title" | "pattern-aliases" | "cube-size"
             | "color-scheme" | "color-letters" | "sticker-grid"
             | "color-constraints" | "move-alphabet" | "search-limits" | "pattern-tags"
             | "required-symmetry" | "required-antisymmetry" | "sequence-order"
             | "movable-orbits" | "fixed-orbits" | "format-version" ;
value        = text { newline continuation } ;
```

## Keys

| Key | Required | Meaning |
| --- | --- | --- |
| `pattern-name` | yes | Pattern id: lowercase `a-z`, `0-9` and `-`. Defines the pattern name. |
| `pattern-title` | no | Display title. |
| `pattern-aliases` | no | Other ids, separated by commas. |
| `cube-size` | yes | Cube size, or a list of sizes. |
| `color-scheme` | no | Color scheme for fixed colors; see [Color schemes](color-schemes.md). Default `URFDLB`. |
| `color-letters` | no | Display colors of an in-place six-letter `color-scheme`; same value grammar as in a scheme file. |
| `sticker-grid` | yes | The schema grid. |
| `color-constraints` | no | The guard. Without it every assignment of the variables is allowed. |
| `move-alphabet` | no | Moves used by `--solve`: `outer`, `slices`, `blocks` or a list of moves. |
| `search-limits` | no | Default limits: `state-limit`, `max-states`, `max-time-ms`, `max-memory-mb`, `max-depth`. |
| `pattern-tags` | no | Tags, separated by commas. |
| `format-version` | no | Format version, `1`. Default `1`. |
| `required-symmetry` | no | The pattern has at least this symmetry type. |
| `required-antisymmetry` | no | The same for the extension in which the symmetry gives the inverse cube. |
| `sequence-order` | no | Order of the generating move sequence. |
| `movable-orbits` | no | Only these orbits may change; the rest stays solved. |
| `fixed-orbits` | no | Orbits that must stay solved. |

### Symmetry, order and orbits

`required-symmetry` names a Schoenflies type, for example `D4h`. The cube symmetry group has 48 elements; its subgroups fall into 33 conjugacy classes, which the implementation derives by computation. This page does not list them, and the count is not verified here. `required-antisymmetry` uses the same names for the 96-element extension of that group, in which applying the symmetry gives the inverse cube.

`sequence-order` is the order of the move sequence that generates the pattern: a number, a range such as `3..6`, or a set such as `{2, 3, 6}`.

`movable-orbits` and `fixed-orbits` list orbit names as the implementation names them, for example corners, edges, wings, midges, centers, x-centers and t-centers. `fixed-orbits: X` is shorthand for `movable-orbits:` naming every other orbit.

```ebnf
symmetry-value = schoenflies ;
schoenflies    = upper { letter | digit } ;
order-value    = number | number ".." number | "{" number { "," number } "}" ;
orbits-value   = orbit-name { "," orbit-name } ;
orbit-name     = lower { lower | "-" } ;
```

Error messages, each with line and column:

- `unknown symmetry type "X"`
- `unknown orbit "X"`
- `order range "a..b": a is greater than b`
- `order must be a positive number, a range or a set`
- `"movable-orbits" and "fixed-orbits" name the same orbit "X"`

```ebnf
id       = lower-or-digit { lower-or-digit | "-" } ;
move-alphabet = "outer" | "slices" | "blocks" | move { ws move } ;
```

## Schema grid

The grid lists the stickers of the faces, face by face, each row by row.

- A token made of a lowercase letter and an optional number, such as `a` or `b1`, is a placeholder that may take any color. The same token means the same color.
- A single uppercase letter is a fixed color, read through the scheme. A center is written as the letter of its face (U R F D L B): the fixed color of that face. A center token may also be a lowercase variable, like any other sticker. In a one-face grid, the center letter stands for the letter of the face the grid is applied to: the U letter of the scheme at the center position is replaced by the letter of each face, other letters are kept. An even size has no center.
- `.` and `?` are don't care.
- Separators are `,` and space; `/` separates faces, and so do a newline and a blank line.
- The number of stickers per face is checked against the size: N times N for size N.

```ebnf
schema      = face { face-sep face } ;
face        = token { sep token } ;
token       = placeholder | fixed | dontcare ;
placeholder = lower [ digits ] ;
fixed       = upper ;
dontcare    = "." | "?" ;
sep         = "," | " " ;
face-sep    = "/" | newline | blank-line ;
lower       = "a".."z" ;
upper       = "A".."U" ;
digits      = digit { digit } ;
```

The face index (the position of the face in the schema) is appended to every placeholder: `a` on the first face is the variable `a1`, on the second face `a2`.

One face. A grid of exactly N times N tokens is one face and stands for all six: the same pattern on every face, the index of the face appended to each placeholder (`a1` to `a6`), the center letter as above. Then `/` and newlines only break rows, so `a b a / b . b / a b a` is one 3x3 face. A grid with six faces writes its placeholders as they are to be named (`a1`, `b2`), without an index appended.

Differences: the layout example used `color-scheme: western`, whose letters are W R G Y O B, so its `U` was an unknown color; the example now uses the default scheme. The grammar above lets `/` separate faces, but the shipped one-face examples break the rows of one face with `/`. The reader decides by the count: N times N tokens or fewer in all is one face and `/` breaks rows; more is six faces. A number in a token of a one-face grid is kept and the face index follows it (`b1` on the second face is `b12`).

## Guard

```ebnf
guard      = or-expr ;
or-expr    = and-expr { ( "||" | "OR" | "or" ) and-expr } ;
and-expr   = not-expr { ( "&&" | "AND" | "and" ) not-expr } ;
not-expr   = ( "!" | "NOT" ) not-expr | comparison ;
comparison = term [ cmp-op term ] ;
cmp-op     = "==" | "!=" | "<" | "<=" | ">" | ">=" ;
term       = call | number | "(" guard ")" ;
func       = "unique" | "equal" | "count" ;
call       = func "(" var { "," var } ")"
           | ( "symmetric" | "antisymmetric" ) "(" schoenflies ")"
           | "order" "(" [ order-value ] ")" ;
var        = lower [ digits | "*" ] ;
number     = digit { digit } ;
```

Precedence, highest first: `!` or `NOT`; comparison; `&&`, `AND` or `and`; `||`, `OR` or `or`.

- `unique(...)` holds when all named variables have different colors.
- `equal(...)` holds when they all have the same color.
- `count(...)` is the number of distinct colors among the matching variables. It is a number, to be compared.

Pattern functions, which take no variables:

- `symmetric(TYPE)` holds when the pattern has at least the symmetry type TYPE.
- `antisymmetric(TYPE)` holds the same for the antisymmetry extension.
- `order(...)` holds when the order of the generating sequence is in the given number, range or set.

`required-symmetry: X` is shorthand for `color-constraints: symmetric(X)`. Both may be combined with `&&`, for example `color-constraints: unique(a*, b*, c*) && symmetric(D4h)`. The functions `symmetric(TYPE)`, `antisymmetric(TYPE)` and `order(...)` keep their names: they are functions, not keys.

A `*` in a variable name stands for the same index in all arguments of the term, for every index that occurs. It is not automatically a face: the index is only a label, so a guard may also compare variables of different faces.

## Limits and options of the generator

| Option | Default | Meaning |
| --- | --- | --- |
| `--limit N` | 20 | Print at most N states. When more exist, stderr says `limit reached: N of M`. |
| `--count` | off | Count only, exactly; nothing is listed. |
| `--sample N` | off | Draw N states evenly or randomly; `--seed` fixes the draw. |
| `--max-states N` | 50 000 000 | Hard cap on the enumeration. Over it: error, exit code 3, with the counted number if it is known. |
| `--max-time MS` | 120 000 | Time budget in milliseconds. |
| `--max-memory MB` | 1500 | Memory budget in megabytes. |

Exit codes: 0 ok; 1 input error; 2 impossible or nothing found; 3 limit reached.

Rules:

- No limit ever stops silently. Stderr says which limit was hit and how far the run got.
- Stdout keeps what was found so far and ends with a marker that the output is incomplete.
- The `search-limits:` key of the definition file sets the defaults; the command line overrides it.
- When neither `--limit` nor `--count` is given, the generator counts before it lists.
- REST uses the names `stateLimit`, `countOnly`, `sampleSize`, `sampleSeed` and `budgetMs`, and the answer has the field `searchComplete: true|false`.
- The search for moves runs only behind `--solve`, with its own `--max-depth` and the `move-alphabet` key.

Order and symmetry are first filters for counting, listing and checking. Searching move sequences for them is `--solve` only, and comes later.

## REST

Planned: `POST /v1/patterns/search`. The body carries the file text in the field `patternDefinitionText` and the run options in the object `runOptions` (`stateLimit`, `countOnly`, `sampleSize`, `sampleSeed`, `budgetMs`). The server is stateless and stores nothing. The answer has the field `searchComplete`. The parser is the same as for the command line, and errors carry the same text with line and column.

## Errors

The parser reports the line, the column and what was expected, for example `line 7, column 12: expected ")" or ","`. It reports at least these cases:

- a missing required key, or an unknown key;
- a bad id;
- a sticker count per face that does not match the size;
- a variable in the guard that is absent from the schema;
- an unknown color letter;
- an unbalanced parenthesis.

## JSON form

The text form and a JSON document are two serializations of one canonical model: the typed definition (pattern name, title, aliases, tags, cube size, color scheme, color letters, sticker grid, color constraints, required symmetry and antisymmetry, sequence order, movable and fixed orbits, move alphabet, search limits). The run options are a separate object, not part of the definition. The text parser and the JSON reader both produce exactly this model, so converting from one form to the other loses nothing except comments and layout.

- Status: `cubacle template check` reads the text form only; the JSON reader is not implemented, and the `.json` example files only show the form (`template check` on one says `expected "key: value"`).
- The JSON text is read with a JSON parser library (`serde_json`). There is no hand-written JSON parser.
- The result is checked against the JSON Schema with a validation library (the `jsonschema` crate). Errors carry a JSON pointer path, for example `/schema/2/3`, in the place where the text form gives line and column.
- The schema is the contract. It is written by hand and split into files with shared definitions. The Rust types are checked against it by conformance tests: every example must validate and round-trip. Schemas are not generated from the parser output.
- Rules the schema cannot express (the sticker count per face, distinct scheme letters, the guard grammar, known symmetry types) are checked by the reader after validation and reported with the same path.

The schema files are published under `https://wstein.github.io/cubacle/schema/v1/`:

| File | Describes |
| --- | --- |
| `common.schema.json` | Shared definitions: names, tokens, face grid, guard string, color scheme, orbits, limits, error object. |
| `pattern-definition.schema.json` | The definition (this page). |
| `color-scheme.schema.json` | A color scheme, see [Color schemes](color-schemes.md). |
| `run-options.schema.json` | The run options. |
| `search-request.schema.json`, `search-result.schema.json` | The REST contract, see [REST and JSON](rest-json.md). |

Key mapping, stated once: every JSON key is the camelCase form of the kebab-case text key (`pattern-name` is `patternName`, `sticker-grid` is `stickerGrid`, `max-time-ms` is `maxTimeMs`). There are no aliases and no other names. The JSON object adds `formatVersion` (always 1, text key `format-version`) and `$schema`. Differences of form:

- `stickerGrid` is a list of faces in the order U R F D L B; a face is a list of rows; a row is a list of tokens. One face means the same face pattern on all six. The path `/stickerGrid/2/3` is face 2, row 3.
- `colorConstraints` is the guard as the same string as in the text form.
- `sequenceOrder` is a number, a set `[3, 6]` or a range `{"min": 3, "max": 6}`.
- `colorScheme` is a name or six letters (see [Color schemes](color-schemes.md)). The display colors of six letters are `colorLetters`, an object from letter to color in JSON and a `color-letters:` line in the text form, with the same value grammar as in a scheme file.
- `movableOrbits`, `fixedOrbits`, `patternAliases` and `patternTags` are lists of strings.

The example of the worked example above in both forms (the `.cubacle` file spells out all six faces with their indexes, the JSON file has the one-face shorthand; `cubacle template check` reads the one-face form of the text file too; files `concentric-squares.cubacle` and `concentric-squares.json` in `docs/static/examples/definitions/`, published at `https://wstein.github.io/cubacle/examples/definitions/`):

```json
{
  "$schema": "https://wstein.github.io/cubacle/schema/v1/pattern-definition.schema.json",
  "formatVersion": 1,
  "patternName": "concentric-squares",
  "patternTitle": "Concentric squares",
  "cubeSize": 5,
  "stickerGrid": [
    [
      ["a", "b", "b", "b", "a"],
      ["b", "b", "c", "b", "b"],
      ["b", "c", "U", "c", "b"],
      ["b", "b", "c", "b", "b"],
      ["a", "b", "b", "b", "a"]
    ]
  ],
  "colorConstraints": "unique(a*, b*, c*)"
}
```

Further examples in the same directory: `checkerboard-free-center` (a 3x3 grid with a don't-care), `ring-symmetric` (with `required-symmetry: D4h`, the same as `symmetric(D4h)` in the guard; `template check` reads the key and does not read the call yet) and `custom-scheme` (a color scheme object with `#RGB` and `#RRGGBB` colors). All JSON examples start with `$schema`, so editors can offer completion and validation. The example files were validated against the schemas with the Python package `jsonschema` (Draft 2020-12).

## Result lines

Found sequences are stored as result lines of the form `name | size | metric | status | length | moves`, as in `src/patterns/generators.txt`. The `pattern-name` key of the definition file is the key of those lines.

## Worked example

A 5x5x5 where every face has the same ring structure and a center that carries the letter of its face (U, R, F, D, L or B). Per face, in the order of the face index:

```text
a b b b a
b b c b b
b c U c b
b b c b b
a b b b a
```

The grid above is the U face; the other faces carry their own letter in the center. With the face index, the variables of the first face are `a1`, `b1` and `c1`. The guard

```text
unique(a*, b*, c*)
```

requires, for every face, that the three ring colors differ.

## Measured facts

Two throw-away spike branches (`spike/template` and `spike/template-count`) explored this schema. Counts only.

For the 5x5x5 example above there are 393 120 target assignments with the center included in `unique`, and 15 321 600 without it. Of these, the second spike (commit `d6a9751`) found these reachable color states:

| Reading | Assignments | Reachable color states | Share |
| --- | --- | --- | --- |
| Center included in `unique` | 393 120 | 2 640 | 0.67 % |
| Center not included | 15 321 600 | 28 032 | 0.18 % |

These state counts are to be re-confirmed after the mirrored-corner fix of the 5x5x5 engine.

The only reasons for rejection are non-symmetry of `a` and `b`, mirrored corners, and corner/middle-edge parity. Corner twist and edge flip never reject. The result is the same for both readings of `c`.

See also: [Color schemes](color-schemes.md).
