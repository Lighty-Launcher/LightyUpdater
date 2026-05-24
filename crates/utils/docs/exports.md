# Exports

Public surface of `lighty-utils`.

## Crate root

```rust
use lighty_utils::{
    // checksum
    compute_sha1,
    compute_sha1_file_sync,
    compute_sha1_file_async,
    // path
    normalize_path,
    path_to_maven_name,
    // errors
    UtilsError,
};
```

## Module: `checksum`

```rust
pub fn compute_sha1(data: &[u8]) -> String;                          // 40-char hex
pub fn compute_sha1_file_sync(path: &Path) -> Result<String, UtilsError>;
pub async fn compute_sha1_file_async(path: &Path) -> Result<String, UtilsError>;
```

The async variant streams the file through a fixed-size buffer; the
sync variant reads the whole file into memory.

## Module: `path`

```rust
pub fn normalize_path(path: &str) -> String;
pub fn path_to_maven_name(path: &str) -> String;
```

`normalize_path` swaps `\` for `/`. `path_to_maven_name` reverses
`group/artifact/version/jar`-style layouts into `group:artifact:version`.

## `UtilsError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum UtilsError {
    #[error("I/O error: {0}")]      Io(#[from] std::io::Error),
    #[error("Hash error: {0}")]     Hash(String),
    #[error("Invalid path: {0}")]   InvalidPath(String),
}
```

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`checksums.md`](./checksums.md), [`path.md`](./path.md)
