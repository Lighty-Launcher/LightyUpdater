# Overview

`lighty-filesystem` is the I/O wrapper for everything that needs to
touch the local disk in a predictable way: reading file bytes,
building server paths, MIME detection. Built on `tokio::fs` plus
`mime_guess`.

The HTTP fallback path uses `FileSystem::read_bytes` when the RAM
cache misses; the file-cache bulk loader uses
`FileSystem::build_server_path` to join the base path with a server
name.

## What's inside

| Item | Purpose |
|---|---|
| `FileSystem` | Static methods only — `read`, `read_bytes`, `write`, `get_file_info`, `build_server_path`. |
| `FileInfo { size, mime_type }` | Lightweight metadata returned by `get_file_info`. |
| `FileSystemError` | One enum: `Io`, `NotFound`, `InvalidPath`. |

## Cargo features

None.

## See also

- [`how-to-use.md`](./how-to-use.md)
- [`exports.md`](./exports.md)
- [`operations.md`](./operations.md), [`server-structure.md`](./server-structure.md)
