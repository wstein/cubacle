---
title: "Install and build"
description: "Install and build cubacle."
weight: 2
---

# Install and build

cubacle is built from source. No packaged releases are documented yet.

## Requirements

- A Rust toolchain, version 1.82 or later (the `rust-version` declared in `Cargo.toml`).

## Build

Clone the repository and build in release mode. Release mode matters: the solver tables and search are far slower in a debug build.

```sh
git clone https://github.com/wstein/cubacle.git
cd cubacle
cargo build --release
```

This builds the command-line tool at `target/release/cubacle`. The release profile uses fat link-time optimization, so the build takes longer than a default one.

## Build the server

The server is a separate workspace member. Build it with:

```sh
cargo build --release -p cubacle-server
```

The binary is `target/release/cubacle-server`. See the [REST API reference](api.md) for its flags.

## Check the installation

```sh
./target/release/cubacle --version
./target/release/cubacle facelets "R U R' U'"
```

The second command prints the facelets of a cube after the scramble:

```text
UULUUFUUFRRUBRRURRFFDFFUFFFDDRDDDDDDBLLLLLLLLBRRBBBBBB
```

## Run the tests

A plain `cargo test` at the repository root does not run the server tests. Use the workspace form instead:

```sh
cargo test --workspace --release
```
