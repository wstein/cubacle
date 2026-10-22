# Security policy

## Reporting a vulnerability

Please report vulnerabilities privately through GitHub:
[Report a vulnerability](https://github.com/wstein/cubacle/security/advisories/new).
Do not open a public issue for a security problem.

Give the version or commit, the command or request that triggers it, and what you
expected. Reports are answered on a best-effort basis; this is a small project.

## Scope

The REST backend (`crates/server`) is stateless and binds to `127.0.0.1` by
default. Anything that lets a request crash the server, run past its documented
limits (body size, time budget, concurrency) or read or write outside its
cache directory is in scope. Solver answers that are merely long are not.

## Supported versions

Only the current `develop` branch receives fixes.
