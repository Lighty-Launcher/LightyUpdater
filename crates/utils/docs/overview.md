# Overview

`lighty-utils` is a grab-bag of small, dependency-free helpers used
across the workspace. Three modules: `checksum`, `path`, `errors`.

## What's inside

| Module | Purpose |
|---|---|
| `checksum` | SHA1 helpers — sync and async, in-memory and streaming. |
| `path` | `normalize_path`, `path_to_maven_name` (turns a Maven jar path into the canonical artifact name). |
| `errors` | `UtilsError` — `Io`, `Hash`, simple wrappers. |

The crate has no internal deps.

## Cargo features

None.

## See also

- [`how-to-use.md`](./how-to-use.md)
- [`exports.md`](./exports.md)
- [`checksums.md`](./checksums.md), [`path.md`](./path.md)
