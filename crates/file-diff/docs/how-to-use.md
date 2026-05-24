# Using lighty-file-diff

One short example per use case. The crate is sync, no setup required.

## 1. Detect changes between two snapshots

```rust
use lighty_file_diff::FileDiff;

let diff = FileDiff::compute("survival", Some(&old), &new);

println!(
    "{} added, {} modified, {} removed",
    diff.added.len(),
    diff.modified.len(),
    diff.removed.len(),
);
```

`old` is `Option<&VersionBuilder>` — pass `None` for the first scan
and every file present in `new` lands in `diff.added`.

## 2. Filter by file type

`FileChange.file_type` carries the family (`Client`, `Library`,
`Mod`, `Native`, `Asset`). Useful when only some are worth uploading
or purging.

```rust
use lighty_file_diff::{FileDiff, FileType};

let only_mods: Vec<_> = diff
    .added
    .iter()
    .chain(diff.modified.iter())
    .filter(|change| matches!(change.file_type, FileType::Mod))
    .collect();
```

## 3. Materialise the URLs to purge

Each `FileChange` carries `url`. Empty strings mean "no public URL"
(typical for libraries without a `url` field in the manifest), so
filter them out before calling a CDN.

```rust
let purge_urls: Vec<String> = diff
    .added
    .iter()
    .chain(diff.modified.iter())
    .chain(diff.removed.iter())
    .map(|change| change.url.clone())
    .filter(|url| !url.is_empty())
    .collect();
```

## 4. Keep the URL map in sync

After computing a diff and inserting the new `VersionBuilder` into the
cache, replay the diff against its `url_to_path_map` so HTTP handlers
keep resolving by URL without an expensive `build_url_map` rebuild.

```rust
let diff = FileDiff::compute("survival", Some(&old), &new_builder);

// new_builder.url_to_path_map starts empty after deserialization
diff.apply_to_url_map(&mut new_builder);
```

Behaviour: `added` and `modified` entries are inserted; `removed`
entries are deleted; empty `url`s are ignored on both sides.

## 5. First-scan shortcut

The first time a server is seen, there is no `old` to compare against.
`compute` handles this:

```rust
let first = FileDiff::compute("survival", None, &new_builder);
// first.modified.is_empty() == true
// first.removed.is_empty() == true
// first.added.len() == every file in new_builder
```

## Errors at a glance

`lighty-file-diff` has no error type. The functions either return a
diff or, in the case of `apply_to_url_map`, just mutate the builder.

## See also

- [`overview.md`](./overview.md) — what the crate computes and why
- [`exports.md`](./exports.md) — full public surface
- [`../../rescan/docs/flows.md`](../../rescan/docs/flows.md) — how the
  rescan orchestrator wires `FileDiff::compute` together with the
  file cache, cloud upload and CDN purge events
