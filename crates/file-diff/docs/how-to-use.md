# Using lighty-file-diff

## Compute a diff

```rust
use lighty_file_diff::FileDiff;

let diff = FileDiff::compute("survival", Some(&old), &new);
println!("{} / {} / {}", diff.added.len(), diff.modified.len(), diff.removed.len());
```

`old = None` for first scan: everything lands in `added`.

## Filter by file type

```rust
use lighty_file_diff::FileType;

let only_mods: Vec<_> = diff
    .modified
    .iter()
    .filter(|change| matches!(change.file_type, FileType::Mod))
    .collect();
```

## Collect URLs to purge

```rust
let urls: Vec<String> = diff
    .added.iter().chain(diff.modified.iter()).chain(diff.removed.iter())
    .map(|change| change.url.clone())
    .filter(|url| !url.is_empty())
    .collect();
```

## Keep the url map in sync

```rust
let diff = FileDiff::compute("survival", Some(&old), &new_builder);
diff.apply_to_url_map(&mut new_builder);
```

Adds entries from `added`+`modified`, removes entries from `removed`,
skips empty URLs.

## Errors

None — the crate has no error type.

## See also

- [overview.md](./overview.md)
- [exports.md](./exports.md)
