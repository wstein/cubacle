---
title: "REST and JSON"
description: "Specification: the request and response contract of POST /v1/patterns/search and the detection of .cubacle and .json files on the command line."
weight: 30
---

# REST and JSON

Status: specification, not implemented yet. Nothing on this page can be run.

The JSON form is described with the [definition file](definition-file.md); color schemes with [Color schemes](color-schemes.md). This page gives the contract of the planned search endpoint and the command line rules for choosing a form.

## POST /v1/patterns/search

The server is stateless and stores nothing. The compute work does not run on async threads.

Request body, described by `https://wstein.github.io/cubacle/schema/v1/search-request.schema.json`: `formatVersion` (1), exactly one of `patternDefinition` (the definition object) or `patternDefinitionText` (the text of a definition file), and optional `runOptions` (`stateLimit`, `countOnly`, `sampleSize`, `sampleSeed`, `budgetMs`, `maxStates`, `maxMemoryMb`, `solve`, `maxDepth`; see `run-options.schema.json`). JSON keys are the camelCase form of the kebab-case text keys, see [the definition file](definition-file.md).

```json
{
  "formatVersion": 1,
  "patternDefinitionText": "pattern-name: checkerboard\ncube-size: 3\nsticker-grid: a b a / b a b / a b a\n",
  "runOptions": { "countOnly": true }
}
```

Answer, described by `https://wstein.github.io/cubacle/schema/v1/search-result.schema.json`: `formatVersion`, `searchComplete` (`true` or `false`), `matchCount`, `matchingStates` (each with `facelets` and, if asked, `tokens` or `moves`), `limits` (which limit stopped the run, how many states were returned and how many exist) and `error`.

```json
{
  "formatVersion": 1,
  "searchComplete": false,
  "matchCount": 2640,
  "matchingStates": [{ "facelets": "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB" }],
  "limits": { "reached": "stateLimit", "returned": 1, "total": 2640 }
}
```

`searchComplete` is `false` whenever a limit stopped the run; no limit stops silently. What was found so far stays in `matchingStates`.

Errors use the field `error` with `message`, optionally `expected`, and the position: `line` and `column` for `patternDefinitionText`, `path` (a JSON pointer such as `/stickerGrid/2/3`) for `patternDefinition`. The text of the message is the same as on the command line.

## Command line

- A file argument ending in `.cubacle` is read as text, one ending in `.json` as JSON. The double extension `.cubacle.json` is supported and recommended for JSON definition files; it is optional, and the file is read as JSON.
- `-` reads standard input; `--format cubacle|json` says which form it is.
- `cubacle definition convert FILE --to json|cubacle` writes the other form. Comments and layout of a definition file are not kept.

```text
cubacle patterns find concentric-squares.json
cubacle patterns find - --format json < concentric-squares.json
cubacle definition convert concentric-squares.cubacle --to json
```
