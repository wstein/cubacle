---
id: 20261024101500
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
The environment alone missed Kitty terminals such as herdr, so `--graphics auto|kitty|ansi|plain` and `CUBACLE_GRAPHICS` override the choice (flag, then variable, then auto; an explicit value is used even when output is not a terminal), and `auto` asks the terminal when the variables would give colored text. The question is the neutral `i=31,q=1` plus the specification's `a=q` query and a DA1 sentinel; any `i=31` reply, error included, counts, since kitty and herdr both answer the neutral form with `EINVAL`. It is skipped in `tmux` and `screen`, and its I/O sits behind a trait so the parser and the decision are tested without a tty.

## Links

- [[Terminal probing uses rustix and own parsing]]

Source: src/terminal/kitty.rs, lines 1-60
