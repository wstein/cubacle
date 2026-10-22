---
id: 20261022160004
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
The parallel search cuts the depth-first searches into tasks in the order the serial search visits them, runs them on any number of threads and merges the results in task order, checking the quota after each group. Without a time budget the solution is therefore the same for every CUBACLE_THREADS. With a budget it is not.

## Links

- [[5x5 reduction engine]]

Source: src/reduction/ida.rs, lines 1-14
