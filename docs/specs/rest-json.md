---
title: "REST and JSON"
description: "Specification: the request and response contract of POST /v1/patterns/search and the detection of .gen and .json files on the command line."
weight: 30
---

# REST and JSON

Status: specification, not implemented yet. Nothing on this page can be run.

The JSON form is described with the [.gen definition file](gen-file.md); colour schemes with [Colour schemes](colour-schemes.md). This page gives the contract of the planned search endpoint and the command line rules for choosing a form.

## POST /v1/patterns/search

The server is stateless and stores nothing. The compute work does not run on async threads.

Request body, described by `https://wstein.github.io/cubacle/schema/v1/search-request.schema.json`: `version` (1), exactly one of `definition` (the definition object) or `definitionText` (the text of a `.gen` file), and optional `options` (`limit`, `count`, `sample`, `seed`, `budgetMs`, `maxStates`, `maxMemoryMb`, `solve`, `maxDepth`; see `run-options.schema.json`).

```json
{
  "version": 1,
  "definitionText": "name: checkerboard\nsize: 3\nschema: a b a / b a b / a b a\n",
  "options": { "count": true }
}
```

Answer, described by `https://wstein.github.io/cubacle/schema/v1/search-result.schema.json`: `version`, `complete` (`true` or `false`), `count`, `states` (each with `facelets` and, if asked, `tokens` or `moves`), `limits` (which limit stopped the run, how many states were returned and how many exist) and `error`.

```json
{
  "version": 1,
  "complete": false,
  "count": 2640,
  "states": [{ "facelets": "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB" }],
  "limits": { "reached": "limit", "returned": 1, "total": 2640 }
}
```

`complete` is `false` whenever a limit stopped the run; no limit stops silently. What was found so far stays in `states`.

Errors use the field `error` with `message`, optionally `expected`, and the position: `line` and `column` for `definitionText`, `path` (a JSON pointer such as `/schema/2/3`) for `definition`. The text of the message is the same as on the command line.

## Command line

- A file argument ending in `.gen` is read as text, one ending in `.json` as JSON.
- `-` reads standard input; `--format gen|json` says which form it is.
- `cubacle gen convert FILE --to json|gen` writes the other form. Comments and layout of a `.gen` file are not kept.

```text
cubacle patterns find concentric-squares.json
cubacle patterns find - --format json < concentric-squares.json
cubacle gen convert concentric-squares.gen --to json
```
