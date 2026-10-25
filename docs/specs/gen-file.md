---
title: "The .gen definition file"
description: "Specification: the .gen file that defines a pattern by a schema grid and a guard, and the grammar of both."
weight: 10
---

# The .gen definition file

Status: specification, not implemented yet. Nothing on this page can be run.

## Purpose

A `.gen` file defines a family of patterns by a schema grid (which stickers are equal) and a guard (a condition on the colours). It is given as a command argument; there is no fixed directory.

```text
cubacle patterns find concentric-squares.gen
```

By default the command only checks the definition and counts what fits it. Searching for a move sequence happens only with the explicit flag `--solve`.

```text
cubacle patterns find concentric-squares.gen --solve
```

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
key          = "name" | "title" | "aliases" | "size" | "scheme" | "schema"
             | "where" | "alphabet" | "limits" | "tags" ;
value        = text { newline continuation } ;
```

## Keys

| Key | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Pattern id: lowercase `a-z`, `0-9` and `-`. Defines the pattern name. |
| `title` | no | Display title. |
| `aliases` | no | Other ids, separated by commas. |
| `size` | yes | Cube size, or a list of sizes. |
| `scheme` | no | Colour scheme for fixed colours; see [Colour schemes](colour-schemes.md). Default `URFDLB`. |
| `schema` | yes | The schema grid. |
| `where` | no | The guard. Without it every assignment of the variables is allowed. |
| `alphabet` | no | Moves used by `--solve`: `outer`, `slices`, `blocks` or a list of moves. |
| `limits` | no | Search limits for `--solve`. |
| `tags` | no | Tags, separated by commas. |

```ebnf
id       = lower-or-digit { lower-or-digit | "-" } ;
alphabet = "outer" | "slices" | "blocks" | move { ws move } ;
```

## Schema grid

The grid lists the stickers of the faces, face by face, each row by row.

- A token made of a lowercase letter and an optional number, such as `a` or `b1`, is a placeholder that may take any colour. The same token means the same colour.
- A single uppercase letter is a fixed colour, read through the scheme.
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
call       = func "(" var { "," var } ")" ;
func       = "unique" | "equal" | "count" ;
var        = lower [ digits | "*" ] ;
number     = digit { digit } ;
```

Precedence, highest first: `!` or `NOT`; comparison; `&&`, `AND` or `and`; `||`, `OR` or `or`.

- `unique(...)` holds when all named variables have different colours.
- `equal(...)` holds when they all have the same colour.
- `count(...)` is the number of distinct colours among the matching variables. It is a number, to be compared.

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

## Errors

The parser reports the line, the column and what was expected, for example `line 7, column 12: expected ")" or ","`. It reports at least these cases:

- a missing required key, or an unknown key;
- a bad id;
- a sticker count per face that does not match the size;
- a variable in the guard that is absent from the schema;
- an unknown colour letter;
- an unbalanced parenthesis.

## Result lines

Found sequences are stored as result lines of the form `name | size | metric | status | length | moves`, as in `src/patterns/generators.txt`. The `name` key of the `.gen` file is the key of those lines.

## Worked example

A 5x5x5 where every face has the same ring structure and a fixed colour `Z` in the middle. Per face, in the order of the face index:

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

requires, for every face, that the three ring colours differ.

## Measured facts

Two throw-away spike branches (`spike/template` and `spike/template-count`) explored this schema. Counts only.

For the 5x5x5 example above there are 393 120 target assignments with the centre `Z` included in `unique`, and 15 321 600 without it. Of these, the second spike (commit `d6a9751`) found these reachable colour states:

| Reading | Assignments | Reachable colour states | Share |
| --- | --- | --- | --- |
| With `Z` in `unique` | 393 120 | 2 640 | 0.67 % |
| Without `Z` | 15 321 600 | 28 032 | 0.18 % |

These state counts are to be re-confirmed after the mirrored-corner fix of the 5x5x5 engine.

The only reasons for rejection are non-symmetry of `a` and `b`, mirrored corners, and corner/middle-edge parity. Corner twist and edge flip never reject. The result is the same for both readings of `c`.

See also: [Colour schemes](colour-schemes.md).
