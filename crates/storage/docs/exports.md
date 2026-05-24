# Exports

Public surface of `lighty-storage`.

## Crate root

```rust
use lighty_storage::{LocalBackend, StorageBackend, StorageError};

#[cfg(feature = "s3")]
use lighty_storage::S3Backend;
```

## `StorageBackend` trait

```rust
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<(), StorageError>;
    async fn delete_file(&self, remote_key: &str)                    -> Result<(), StorageError>;
    async fn read_bytes(&self, remote_key: &str)                     -> Result<Bytes, StorageError>;
    fn url(&self, remote_key: &str) -> String;
    fn is_remote(&self) -> bool;
}
```

The minimum set every consumer relies on. Specific impls may add
typed helpers, but the trait is what `lighty-cache`, `lighty-rescan`
and `lighty-api` import.

## `LocalBackend`

```rust
pub struct LocalBackend { /* base_url + base_path */ }

impl LocalBackend {
    pub fn new(base_url: String, base_path: PathBuf) -> Self;
}
```

`is_remote() == false`. Reads/writes through `tokio::fs`.

## `S3Backend` (feature `s3`)

```rust
# #[cfg(feature = "s3")]
pub struct S3Backend { /* client + bucket + prefix + public_url */ }

# #[cfg(feature = "s3")]
impl S3Backend {
    pub async fn new(
        endpoint_url: String,
        region:       String,
        access_key_id: String,
        secret_access_key: String,
        bucket_name:  String,
        public_url:   String,
        bucket_prefix: String,
    ) -> Result<Self, StorageError>;
}
```

`is_remote() == true`. Backed by `aws-sdk-s3`; works against AWS S3
or any compatible API (Cloudflare R2 tested).

## `StorageError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")] Io(#[from] std::io::Error),
    #[error("Not found: {0}")] NotFound(String),
    // see errors.md for the s3 variants
}
```

## Cargo features

| Feature | Effect |
|---|---|
| `s3` | Pulls AWS SDK and exposes `S3Backend`. |

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`local.md`](./local.md), [`s3.md`](./s3.md)
- [`architecture.md`](./architecture.md)
