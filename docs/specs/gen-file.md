---
title: "The .gen definition file"
description: "Specification: the .gen file that defines a pattern by a schema grid and a guard, and the grammar of both."
weight: 10
---

# The .gen definition file

Status: specification, not implemented yet. Nothing on this page can be run.

## Purpose

A `.gen` file defines a family of patterns by a schema grid (which stickers are equal) and a guard (a condition on the colors). It is given as a command argument; there is no fixed directory.

```text
cubacle patterns find concentric-squares.gen
```

By default the command only checks the definition and counts what fits it. Searching for a move sequence happens only with the explicit flag `--solve`.

```text
cubacle patterns find concentric-squares.gen --solve
```

## Principle

The definition file says what a pattern is: its name, grid, guard, symmetry, order and orbits. The name must mean the same pattern everywhere, in `identify` and in the library. Run options say how much to compute (`--limit`, `--count`, `--sample`, `--seed`, `--max-states`, `--max-time`, `--max-memory`, `--solve`, `--icons`). They never change the pattern.

## Layout

The file is line-based: `key: value`. `#` starts a comment that runs to the end of the line. A line that starts with white space continues the value of the previous key.

```text
# concentric squares on a 5x5x5
name: concentric-squares
title: Concentric squares
size: 5
scheme: western
schema:
  a b b b a / b b c b b / b c Z c b / b b c b b / a b b b a
where: unique(a*, b*, c*)
```

```ebnf
file         = { line } ;
line         = ( blank | comment | entry | continuation ) newline ;
blank        = { " " | tab } ;
comment      = { " " | tab } "#" { any } ;
entry        = key ":" { " " } value ;
continuation = ( " " | tab ) { " " | tab } text ;
key          = "name" | "title" | "aliases" | "size" | "scheme" | "colors" | "schema"
             | "where" | "alphabet" | "limits" | "tags"
             | "symmetry" | "antisymmetry" | "order" | "orbits" | "fixed" ;
value        = text { newline continuation } ;
```

## Keys

| Key | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Pattern id: lowercase `a-z`, `0-9` and `-`. Defines the pattern name. |
| `title` | no | Display title. |
| `aliases` | no | Other ids, separated by commas. |
| `size` | yes | Cube size, or a list of sizes. |
| `scheme` | no | Color scheme for fixed colors; see [Color schemes](color-schemes.md). Default `URFDLB`. |
| `colors` | no | Display colors of an in-place six-letter `scheme`; same value grammar as in a scheme file. |
| `schema` | yes | The schema grid. |
| `where` | no | The guard. Without it every assignment of the variables is allowed. |
| `alphabet` | no | Moves used by `--solve`: `outer`, `slices`, `blocks` or a list of moves. |
| `limits` | no | Search limits for `--solve`. |
| `tags` | no | Tags, separated by commas. |
| `symmetry` | no | The pattern has at least this symmetry type. |
| `antisymmetry` | no | The same for the extension in which the symmetry gives the inverse cube. |
| `order` | no | Order of the generating move sequence. |
| `orbits` | no | Only these orbits may change; the rest stays solved. |
| `fixed` | no | Orbits that must stay solved. |

### Symmetry, order and orbits

`symmetry` names a Schoenflies type, for example `D4h`. The cube symmetry group has 48 elements; its subgroups fall into 33 conjugacy classes, which the implementation derives by computation. This page does not list them, and the count is not verified here. `antisymmetry` uses the same names for the 96-element extension of that group, in which applying the symmetry gives the inverse cube.

`order` is the order of the move sequence that generates the pattern: a number, a range such as `3..6`, or a set such as `{2, 3, 6}`.

`orbits` and `fixed` list orbit names as the implementation names them, for example corners, edges, wings, midges, centers, x-centers and t-centers. `fixed: X` is shorthand for `orbits:` naming every other orbit.

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
- `"orbits" and "fixed" name the same orbit "X"`

```ebnf
id       = lower-or-digit { lower-or-digit | "-" } ;
alphabet = "outer" | "slices" | "blocks" | move { ws move } ;
```

## Schema grid

The grid lists the stickers of the faces, face by face, each row by row.

- A token made of a lowercase letter and an optional number, such as `a` or `b1`, is a placeholder that may take any color. The same token means the same color.
- A single uppercase letter is a fixed color, read through the scheme.
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
upper       = "A".."Z" ;
digits      = digit { digit } ;
```

The face index (the position of the face in the schema) is appended to every placeholder: `a` on the first face is the variable `a1`, on the second face `a2`.

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

`symmetry: X` is shorthand for `where: symmetric(X)`. Both may be combined with `&&`, for example `where: unique(a*, b*, c*) && symmetric(D4h)`.

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
- The `limits:` key of the `.gen` file sets the defaults; the command line overrides it.
- When neither `--limit` nor `--count` is given, the generator counts before it lists.
- REST uses the names `limit`, `count`, `sample` and `budgetMs`, and the answer has the field `complete: true|false`.
- The search for moves runs only behind `--solve`, with its own `--max-depth` and the `alphabet` key.

Order and symmetry are first filters for counting, listing and checking. Searching move sequences for them is `--solve` only, and comes later.

## REST

Planned: `POST /v1/patterns/search`. The body carries the file text in the field `definition` and the run options as the fields `limit`, `count`, `sample`, `seed` and `budgetMs`. The server is stateless and stores nothing. The answer has the field `complete`. The parser is the same as for the command line, and errors carry the same text with line and column.

## Errors

The parser reports the line, the column and what was expected, for example `line 7, column 12: expected ")" or ","`. It reports at least these cases:

- a missing required key, or an unknown key;
- a bad id;
- a sticker count per face that does not match the size;
- a variable in the guard that is absent from the schema;
- an unknown color letter;
- an unbalanced parenthesis.

## JSON form

The `.gen` text and a JSON document are two serializations of one canonical model: the typed definition (name, size, scheme, grid, guard, symmetry, antisymmetry, order, orbits, fixed, limits, tags). The run options are a separate object, not part of the definition. The text parser and the JSON reader both produce exactly this model, so converting from one form to the other loses nothing except comments and layout.

- The JSON text is read with a JSON parser library (`serde_json`). There is no hand-written JSON parser.
- The result is checked against the JSON Schema with a validation library (the `jsonschema` crate). Errors carry a JSON pointer path, for example `/schema/2/3`, in the place where the text form gives line and column.
- The schema is the contract. It is written by hand and split into files with shared definitions. The Rust types are checked against it by conformance tests: every example must validate and round-trip. Schemas are not generated from the parser output.
- Rules the schema cannot express (the sticker count per face, distinct scheme letters, the guard grammar, known symmetry types) are checked by the reader after validation and reported with the same path.

The schema files are published under `https://wstein.github.io/cubacle/schema/v1/`:

| File | Describes |
| --- | --- |
| `common.schema.json` | Shared definitions: names, tokens, face grid, guard string, color scheme, orbits, limits, error object. |
| `gen-definition.schema.json` | The definition (this page). |
| `color-scheme.schema.json` | A color scheme, see [Color schemes](color-schemes.md). |
| `run-options.schema.json` | The run options. |
| `search-request.schema.json`, `search-result.schema.json` | The REST contract, see [REST and JSON](rest-json.md). |

Mapping of keys: the JSON object uses the same names as the text keys (`name`, `title`, `aliases`, `size`, `scheme`, `schema`, `where`, `alphabet`, `limits`, `tags`, `symmetry`, `antisymmetry`, `order`, `orbits`, `fixed`) and adds `version` (always 1) and `$schema`. Differences of form:

- `schema` is a list of faces in the order U R F D L B; a face is a list of rows; a row is a list of tokens. One face means the same face pattern on all six. The path `/schema/2/3` is face 2, row 3.
- `where` is the guard as the same string as in the text form.
- `order` is a number, a set `[3, 6]` or a range `{"min": 3, "max": 6}`.
- `scheme` is a name, six letters, or an object (see [Color schemes](color-schemes.md)). In the text form an in-place scheme is `scheme: ABCDEF` plus a `colors:` line with the same value grammar as in a scheme file.
- `orbits`, `fixed`, `aliases` and `tags` are lists of strings.

The example of the worked example above in both forms (files `concentric-squares.gen` and `concentric-squares.json` in `docs/static/examples/gen/`, published at `https://wstein.github.io/cubacle/examples/gen/`):

```json
{
  "$schema": "https://wstein.github.io/cubacle/schema/v1/gen-definition.schema.json",
  "version": 1,
  "name": "concentric-squares",
  "title": "Concentric squares",
  "size": 5,
  "scheme": "western",
  "schema": [
    [
      ["a", "b", "b", "b", "a"],
      ["b", "b", "c", "b", "b"],
      ["b", "c", "Z", "c", "b"],
      ["b", "b", "c", "b", "b"],
      ["a", "b", "b", "b", "a"]
    ]
  ],
  "where": "unique(a*, b*, c*)"
}
```

Further examples in the same directory: `checkerboard-free-center` (a 3x3 grid with a don't-care), `ring-symmetric` (with `symmetric(D4h)`) and `custom-scheme` (a color scheme object with `#RGB` and `#RRGGBB` colors). All JSON examples start with `$schema`, so editors can offer completion and validation. The example files were validated against the schemas with the Python package `jsonschema` (Draft 2020-12).

## Result lines

Found sequences are stored as result lines of the form `name | size | metric | status | length | moves`, as in `src/patterns/generators.txt`. The `name` key of the `.gen` file is the key of those lines.

## Worked example

A 5x5x5 where every face has the same ring structure and a fixed color `Z` in the middle. Per face, in the order of the face index:

```text
a b b b a
b b c b b
b c Z c b
b b c b b
a b b b a
```

With the face index, the variables of the first face are `a1`, `b1` and `c1`. The guard

```text
unique(a*, b*, c*)
```

requires, for every face, that the three ring colors differ.

## Measured facts

Two throw-away spike branches (`spike/template` and `spike/template-count`) explored this schema. Counts only.

For the 5x5x5 example above there are 393 120 target assignments with the center `Z` included in `unique`, and 15 321 600 without it. Of these, the second spike (commit `d6a9751`) found these reachable color states:

| Reading | Assignments | Reachable color states | Share |
| --- | --- | --- | --- |
| With `Z` in `unique` | 393 120 | 2 640 | 0.67 % |
| Without `Z` | 15 321 600 | 28 032 | 0.18 % |

These state counts are to be re-confirmed after the mirrored-corner fix of the 5x5x5 engine.

The only reasons for rejection are non-symmetry of `a` and `b`, mirrored corners, and corner/middle-edge parity. Corner twist and edge flip never reject. The result is the same for both readings of `c`.

See also: [Color schemes](color-schemes.md).
