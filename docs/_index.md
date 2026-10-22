---
title: "cubacle"
description: "A Rubik's cube solver and generator in Rust, from 2x2x2 to 7x7x7."
weight: 1
---

# cubacle

cubacle is a Rubik's cube solver and generator written in Rust. It comes as a command-line tool and as a stateless REST backend (`cubacle-server`) for cube explorers. The name plays on *cubicle*: a small room for working on cubes.

## Status

| Size | Status |
| --- | --- |
| 2x2x2 | Supported. Solutions are optimal in the half-turn metric. |
| 3x3x3 | Supported. Solved with the Kociemba two-phase algorithm. |
| 4x4x4 | Supported by a three-phase reduction. Solutions have about 47 moves, the last few being the rotation to the standard orientation; the tables take 12 MB and are built in a fraction of a second. |
| 5x5x5 | Supported by a five-phase reduction. Solutions have about 73 moves; the first solve needs 237 MB of tables. |
| 6x6x6, 7x7x7 | Planned. The notations and the API accept these sizes, but no engine exists yet. |

For details, see [Engines and limits](engines/_index.md).

## Quick start

Build the tool (see [Install and build](install.md)), then solve a cube:

```sh
cargo build --release
./target/release/cubacle solve A4ib8aejxj4Y
```

The command prints a solution:

```text
B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R
```

Start the REST backend, which listens on `127.0.0.1:8080` by default:

```sh
cargo run --release -p cubacle-server
curl -s localhost:8080/health
```

## Where to go next

- [Install and build](install.md)
- [Command-line guide](cli.md)
- [REST API reference](api.md)
- [Engines and limits](engines/_index.md)
- [Architecture](architecture.md)
- [Contributing](contributing.md)
- [Credits and licence](credits.md)
