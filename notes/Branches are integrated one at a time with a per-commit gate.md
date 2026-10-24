---
id: 20261024133002
aliases: []
tags: ["type/process", "status/decided", "category/process"]
---
Finished work arrives on a branch, is rebased onto develop, and every commit is built and checked alone (fmt, clippy with -D warnings, the full workspace test) before develop is fast-forwarded; a failed or conflicting branch stops the queue. Conflicts in notes and docs keep both sides; rewriting still-local commits is allowed to remove files that never belonged in the history (log files, scratch output), because a commit that adds them cannot be repaired by a later one. Nothing is pushed until the owner says so.

## Links

- [[Feature workflow]]
- [[Validate in CI]]

Source: AGENTS.md, lines 9-17
Source: owner decision of 2026-10-24, lines 1-1
