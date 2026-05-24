# Using lighty-file-system

## 1. Build a server path

```rust
use lighty_file_system::FileSystem;

let path = FileSystem::build_server_path(base_path, "survival");
// PathBuf: {base_path}/survival
```

The file-cache bulk loader walks this path; the API handler reads
under it.

## 2. Read bytes (api fallback path)

```rust
let bytes = FileSystem::read_bytes(&path).await?;
```

Returns a `Bytes` so the response body is zero-copy.

## 3. Get metadata without reading the file

```rust
let info = FileSystem::get_file_info(&path).await?;
// info.size, info.mime_type
```

Useful for `HEAD` requests or for emitting `Content-Length` /
`Content-Type` before deciding whether to send a body.

## Errors at a glance

```rust
pub enum FileSystemError {
    Io(io::Error),
    NotFound(String),
    InvalidPath(String),
}
```

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`operations.md`](./operations.md), [`server-structure.md`](./server-structure.md)
