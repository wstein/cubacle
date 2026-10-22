---
id: 20261022120003
aliases: []
tags: ["type/decision", "status/decided", "category/project"]
---
Branch `main` mirrors the original repository (cs0x7f/min2phase_rust) and stays untouched; `develop` is the working line. Reason: upstream improvements to the algorithm can then be pulled in. Consequence: upstream changes are reviewed and cherry-picked, and `main` is not renamed. Docs publish from `develop` via GitHub Actions (Hugo).

## Links

- [[Cubacle project]]
- [[Notes policy]]

Source: AGENTS.md, lines 15-17
Source: .github/workflows/pages.yml, lines 1-8
Source: docs/contributing.md, lines 11-11
Source: owner decision of 2026-10-21, lines 1-1
