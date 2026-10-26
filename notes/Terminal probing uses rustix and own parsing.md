---
id: 20261023120001
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
The CLI asks the terminal for its pixel width and background color with its own parser (`src/terminal/parse.rs`, no tty needed in tests) and `rustix` (termios and `TIOCGWINSZ`, no C code, MIT or Apache-2.0, already in the lock file through the server). `terminal-colorsaurus` was left out because it covers only the color and not the size, and its tmux handling is more than this tool uses; the probe is skipped inside `tmux` and `screen`. `libc` would add unsafe code for the same calls.

## Links

- [[Render design]]
- [[Licensing]]

Source: src/terminal/mod.rs, lines 1-12
Source: Cargo.toml, lines 33-34
