---
id: 20261024140102
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
`cubacle.toml` has a `[patterns]` section with `files`, `cache_dir` and `auto_fetch`, found as `--config`, then `CUBACLE_CONFIG`, `./cubacle.toml`, then the global `~/.config/cubacle/config.toml` (`$XDG_CONFIG_HOME/cubacle/config.toml`). On the first run without any of them the global file is created from the embedded `cubacle.example.toml` (one stderr line; a read-only location is said and skipped), so the list starts as the built-in library alone. Third-party data is never bundled and never fetched by default: the example (and so the created file) has the built-in entry active and the six CubeTwister raw URLs (2xPocket to 7xVCube7) commented out, with a reminder that the data belongs to its author and the user must check his terms. The binary has no HTTP client and the owner ruled out both a heavy HTTP stack and shelling out, so URLs are only resolved to a path in the cache directory and the user downloads with `curl -o`; `auto_fetch` is read and ignored. A missing or broken entry is a note in `patterns sources`, not an error, so one bad file does not hide the library.

## Links

- [[A pattern table is read by structure and every line is replayed]]
- [[The toml crate reads the config without serde derive]]

Source: owner request of 2026-10-24, lines 1-1
