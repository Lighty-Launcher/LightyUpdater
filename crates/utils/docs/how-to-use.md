# Using lighty-utils

Pick the helper you need; everything composes.

## 1. SHA1 of a buffer

```rust
use lighty_utils::compute_sha1;

let hex = compute_sha1(b"hello"); // 40-char lowercase hex
```

## 2. SHA1 of a file (sync, small files)

```rust
use lighty_utils::compute_sha1_file_sync;

let hex = compute_sha1_file_sync(&path)?;
```

For files above a few MB, prefer the streaming variant inside
`spawn_blocking`.

## 3. Normalize a path (cross-platform separators)

```rust
use lighty_utils::normalize_path;

let unix = normalize_path(r"survival\mods\iris.jar"); // "survival/mods/iris.jar"
```

Used everywhere a path crosses an HTTP boundary or lands in a key.

## 4. Maven jar path → coordinate

```rust
use lighty_utils::path_to_maven_name;

let name = path_to_maven_name("net/example/lwjgl/3.3.6/lwjgl-3.3.6.jar");
// "net.example:lwjgl:3.3.6"
```

Used by the library scanner to populate `Library.name`.

## Errors at a glance

```rust
pub enum UtilsError {
    Io(io::Error),
    Hash(String),
    InvalidPath(String),
    // ...
}
```

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`checksums.md`](./checksums.md), [`path.md`](./path.md)
