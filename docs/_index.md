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
| 4x4x4 to 7x7x7 | Planned. The notations and the API accept these sizes, but no engine exists yet. |

For details, see [Engines and limits](engines.md).

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
- [Engines and limits](engines.md)
- [Architecture](architecture.md)
- [Contributing](contributing.md)
- [Credits and licence](credits.md)
