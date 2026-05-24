# Using lighty-storage

## 1. Build a local backend

```rust
use lighty_storage::{LocalBackend, StorageBackend};
use std::sync::Arc;

let backend: Arc<dyn StorageBackend> = Arc::new(LocalBackend::new(
    "http://localhost:8080".to_string(),
    std::path::PathBuf::from("./servers"),
));
```

Files are read from `./servers/{server}/{path}`, and `url(key)`
returns `http://localhost:8080/{key}`.

## 2. Build an S3 / R2 backend

```rust
# #[cfg(feature = "s3")]
use lighty_storage::S3Backend;

# #[cfg(feature = "s3")]
let backend = S3Backend::new(
    "https://r2.example.com".to_string(), // endpoint_url
    "auto".to_string(),                    // region
    "access-key".to_string(),
    "secret-key".to_string(),
    "lighty-prod".to_string(),             // bucket_name
    "https://cdn.example.com".to_string(), // public_url
    "".to_string(),                        // bucket_prefix
).await?;
```

Requires the `s3` Cargo feature (and `cmake` on the build host for
`aws-lc-rs`).

## 3. Upload a file (rescan does this for you)

```rust
backend.upload_file(&local_path, &remote_key).await?;
```

`local_path` is the absolute path on disk; `remote_key` is the path
inside the backend (e.g. `survival/mods/iris.jar`).

## 4. Delete a file

```rust
backend.delete_file(&remote_key).await?;
```

Idempotent — deleting a missing key returns `Ok(())`.

## 5. Read bytes (api fallback path)

```rust
let bytes = backend.read_bytes(&remote_key).await?;
```

Used by `lighty-api` when the file isn't in the RAM cache. For local
files this is `tokio::fs::read`; for S3 it streams the object body
into a `Bytes`.

## 6. Branch on remoteness

```rust
if backend.is_remote() {
    // upload / delete / purge CDN
} else {
    // just serve from disk
}
```

`lighty-rescan::update_cache_if_changed` uses exactly this check
before triggering cloud sync.

## Errors at a glance

```rust
pub enum StorageError {
    Io(io::Error),
    NotFound(String),
    #[cfg(feature = "s3")] S3(/* aws-sdk-s3 error variant */),
    // ...
}
```

Surfaced on `RescanError::Storage` via `#[from]`.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`local.md`](./local.md), [`s3.md`](./s3.md)
