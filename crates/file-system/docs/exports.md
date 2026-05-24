# Exports

Public surface of `lighty-file-system`.

## Crate root

```rust
use lighty_file_system::{FileInfo, FileSystem, FileSystemError};
```

## `FileSystem`

```rust
pub struct FileSystem;

impl FileSystem {
    pub fn build_server_path(base_path: &str, server_name: &str) -> PathBuf;

    pub async fn read(path: &Path) -> Result<String, FileSystemError>;
    pub async fn read_bytes(path: &Path) -> Result<Bytes, FileSystemError>;
    pub async fn write(path: &Path, contents: &[u8]) -> Result<(), FileSystemError>;
    pub async fn get_file_info(path: &Path) -> Result<FileInfo, FileSystemError>;
}
```

All `async fn`s sit on top of `tokio::fs`.

## `FileInfo`

```rust
pub struct FileInfo {
    pub size:      u64,
    pub mime_type: String,
}
```

## `FileSystemError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum FileSystemError {
    #[error("I/O error: {0}")]      Io(#[from] std::io::Error),
    #[error("Not found: {0}")]      NotFound(String),
    #[error("Invalid path: {0}")]   InvalidPath(String),
}
```

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`operations.md`](./operations.md), [`server-structure.md`](./server-structure.md)
