---
title: "Architecture"
description: "Roles and design of cubacle."
weight: 6
---

# Architecture

This page describes the design by role. The source layout and the dependency rules are in [ARCHITECTURE.md](https://github.com/wstein/cubacle/blob/develop/ARCHITECTURE.md).

## Roles

cubacle has five roles.

| Role | Responsibility |
| --- | --- |
| Puzzle | Describes a cube of any size from 2x2x2 to 7x7x7 and the moves on it. Defines the solver trait that every engine implements. |
| Notation | Reads and writes cube states: Orbit64 tokens, facelets, colours, schemes and move sequences. |
| Render | Draws a move on a cube state as an icon: SVG, pixels, Kitty graphics or text. Depends only on puzzle and does no I/O. |
| Engines | Solve one cube size each: the 2x2x2 table solver, the 3x3x3 two-phase solver and the 5x5x5 reduction solver, with the other sizes planned. |
| Apps | The command-line tool and the REST backend. They parse input through notation, pick an engine and print the result. |

## Dependency direction

Dependencies point one way: apps depend on engines and notation, and engines and notation depend on puzzle. Puzzle depends on none of them.

## Reduction engines

The 5x5x5 engine, and the planned engines for the other large sizes, reduce the cube to a 3x3x3. They do not depend on the 3x3x3 engine directly. They depend on it only through the solver trait, and the 3x3x3 solver is injected when the engine is built. This keeps the engines independent and lets tests substitute a solver; the 5x5x5 tests do.

## Stateless backend

The REST backend holds no per-client state. It builds engine tables once at startup and then answers each request on its own. For now it binds to the local machine only. See the [REST API reference](api.md).

## Planned workspace split

Today the library and the command-line tool live in one crate, and the server is a second workspace member. After the remaining features are done, the roles above are planned to become separate workspace crates.
