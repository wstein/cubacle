---
id: 20261022160005
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
Each table is one file named by id and format version in CUBACLE_CACHE_DIR, else XDG_CACHE_HOME/cubacle, else ~/.cache/cubacle. A file has a header with magic, version, word count and checksum, and is read in blocks. A missing, damaged or foreign file is rebuilt; writing goes to a temporary name and is renamed; failing to write is not an error but is counted, and the CLI warns. Tests that solve a 5x5x5 point the cache at the build directory.

## Links

- [[5x5 reduction engine]]

Source: src/reduction/cache.rs, lines 1-8
Source: src/reduction/cache.rs, lines 77-107
Source: tests/cli_test.rs, lines 6-12
