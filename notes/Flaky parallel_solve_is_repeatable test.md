---
id: 20261022120022
aliases: []
tags: ["type/question", "status/open", "category/open-items"]
---
The former test parallel_solve_is_repeatable was occasionally flaky, suspected to be a shared-best timing race; it is now `parallel_solve_is_valid_on_every_run`, which checks validity, not identical output.

## Links

- [[Open items]]

Source: project history of cubacle, lines 1-1
