---
id: 20261024140103
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
The config is read with the `toml` crate 1.1.8 (MIT or Apache-2.0, rust-version 1.85, below our MSRV 1.95) with default features off and only `std`, `parse` and `serde` (the light serde_core, needed for `toml::Table`) on, into a plain `toml::Table`. No derive is used, so the dependency is the parser and its spans: an error reports the file, the line from the byte span and the message, and unknown sections or keys are errors so a typo cannot silently drop a file from the list. Only the binary uses it; the library and the dependency rules are unchanged.

## Links

- [[Pattern files are listed in cubacle.toml and the default is the built-in library]]

Source: owner request of 2026-10-24, lines 1-1
