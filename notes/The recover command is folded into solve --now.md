---
id: 20261025090000
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
The owner dropped the `recover` command and moved it into `solve`: `solve CUBE --now STATE` is the old `recover` with the same output (Should be / Is now, Stickers that differ, Rollback, Continue, End state, Moves), and `--solution`, `--at-move`, `--sure-until`, `--fresh`, `--strays`, `--one-mistake`, `--thorough` and `--max-time` came with it. Two commands read the same cube, took the same picture and notation options and printed phases the same way, so one command with a mode removes a second option list to keep in step. There is no alias: `cubacle recover` is an unknown subcommand. `--depth` already existed in `solve` for `--all` and `--count`; with `--now` it keeps its old recover meaning (moves of the way back, 0 to 5, default 3), and the options that change how a plain solve searches are refused with `--now`. The library module `recover` is unchanged, so the engine notes still apply.

## Links

- [[Recover searches sticker permutations along the solution]]
- [[Recover advises the shorter way]]

Source: src/recover_cmd.rs, lines 1-20
