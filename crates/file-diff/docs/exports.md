# Exports

Public surface of `lighty-file-diff`.

## Crate root

```rust
use lighty_file_diff::{FileChange, FileDiff, FileType};
```

## Type details

### `FileDiff`

```rust
pub struct FileDiff {
    pub added:    Vec<FileChange>,
    pub modified: Vec<FileChange>,
    pub removed:  Vec<FileChange>,
}

impl FileDiff {
    pub fn compute(
        server_name: &str,
        old: Option<&VersionBuilder>,
        new: &VersionBuilder,
    ) -> Self;

    pub fn apply_to_url_map(&self, builder: &mut VersionBuilder);
}
```

`compute` walks every component of the two snapshots:
- `Client` — `None / Some` transitions plus SHA1 mismatch.
- `Library`, `Mod`, `Asset` — `HashMap`-based pairwise diff keyed by
  path / name / path respectively. Assets compare on `hash`; the rest
  compare on `sha1`.
- `Native` — handled both as a whole (`None / Some`) and per-entry by
  name.

`apply_to_url_map` inserts `(url, relative_path)` for everything in
`added ∪ modified` and removes `url` for everything in `removed`.
Entries with an empty `url` are ignored on both sides.

### `FileChange`

```rust
#[derive(Debug, Clone)]
pub struct FileChange {
    pub file_type:  FileType,
    pub remote_key: String,   // e.g. "survival/mods/iris.jar"
    pub local_path: String,   // e.g. "survival/mods/iris.jar"
    pub url:        String,   // may be empty for libraries/mods without a public URL
}
```

`remote_key` is the path inside the storage backend (S3 / R2 key);
`local_path` is the path under the server's working directory; `url`
is the address a CDN client will purge.

### `FileType`

```rust
#[derive(Debug, Clone)]
pub enum FileType { Client, Library, Mod, Native, Asset }
```

## Cargo features

None — the crate depends only on `lighty-models`.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`../../models/docs/exports.md`](../../models/docs/exports.md) for
  the underlying `VersionBuilder`
