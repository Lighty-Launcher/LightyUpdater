# lighty-file-diff

Granular diff between two `VersionBuilder` snapshots.

## Overview

**Version**: 26.5.0
**Part of**: [LightyUpdater](https://github.com/Lighty-Launcher/LightyUpdater)

`FileDiff::compute(server_name, old, new)` walks both snapshots and
returns three buckets — `added`, `modified`, `removed` — covering
the client jar, libraries, mods, natives and assets. Each `FileChange`
carries the file type, remote key, local path and download URL so
callers (rescan, cloud sync, CDN purge) can act without rebuilding
their own diff.

Pure logic crate: no I/O, no async, only depends on `lighty-models`.

## Quick start

```rust
use lighty_file_diff::{FileDiff, FileType};

let diff = FileDiff::compute("survival", Some(&old), &new);

println!("added: {}", diff.added.len());
println!("modified: {}", diff.modified.len());
println!("removed: {}", diff.removed.len());

// Incrementally maintain the URL→path map without a full rebuild
diff.apply_to_url_map(&mut new);
```

## What it provides

- `FileDiff { added, modified, removed }` with categorized changes.
- `FileChange { file_type, remote_key, local_path, url }` per change.
- `FileType { Client, Library, Mod, Native, Asset }`.
- `FileDiff::apply_to_url_map(builder)` to mutate a `VersionBuilder`
  url map incrementally (avoids `build_url_map` rebuilds).

## Related crates

- [`lighty-rescan`](../rescan) — consumes `FileDiff` to drive cache
  refresh, cloud upload, and CDN purge events.
- [`lighty-models`](../models) — defines `VersionBuilder` and the
  underlying entity types.

## Licence

MIT
