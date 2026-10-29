---
id: 20261024140102
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
`cubacle.toml` has a `[patterns]` section with `files`, `cache_dir` and `auto_fetch`, found as `--config`, then `CUBACLE_CONFIG`, `./cubacle.toml`, then the global `~/.config/cubacle/config.toml` (`$XDG_CONFIG_HOME/cubacle/config.toml`). On the first run without any of them the global file is created from the embedded `cubacle.example.toml` (one stderr line; a read-only location is said and skipped), so the list starts as the built-in library alone. The built-in library is always loaded (owner decision of 2026-10-29): the list only adds sources, a config cannot drop it, and a `"builtin"` entry is accepted and redundant. Third-party data is never bundled and nothing is downloaded by default: the example (and so the created file) has the built-in entry active and the six CubeTwister raw URLs (2xPocket to 7xVCube7) commented out, with a reminder that the data belongs to its author and the user must check his terms. Downloading happens only for sources the user enabled in the config (uncommenting an entry is the consent), through `cubacle patterns update`; `auto_fetch` defaults to false and only the user sets it. A missing or broken entry is a note in `patterns sources`, not an error, so one bad file does not hide the library.

## Links

- [[Pattern sources are downloaded by cubacle itself from the user's config]]
- [[A pattern table is read by structure and every line is replayed]]
- [[The toml crate reads the config without serde derive]]

Source: owner request of 2026-10-24, lines 1-1
