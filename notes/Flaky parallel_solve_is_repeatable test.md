---
id: 20261022120022
aliases: []
tags: ["type/question", "status/open", "category/open-items"]
---
The former test parallel_solve_is_repeatable was occasionally flaky, suspected to be a shared-best timing race; it is now `parallel_solve_is_valid_on_every_run`, which checks validity, not identical output.

## Links

- [[Open items]]

Source: tests/batch_test.rs, lines 83-99
Source: README.md, lines 116-116
Source: owner decision of 2026-10-21, lines 1-1
