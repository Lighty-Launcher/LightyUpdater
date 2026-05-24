# Flows

## Bulk load at boot

`CacheManager::initialize` calls `load_all_servers` once, after the
initial scan has produced `VersionBuilder`s. The bulk load is what
makes the first user request fast.

```mermaid
sequenceDiagram
    participant Init as CacheManager::initialize
    participant Mgr as FileCacheManager
    participant Walk as walkdir
    participant Pool as spawn_blocking + rayon
    participant Cache as Moka cache

    Init->>Mgr: load_all_servers(servers, base_path, concurrency)
    loop per enabled server
        Mgr->>Walk: WalkDir(server_path)
        Walk-->>Mgr: filtered file list (jar / json / assets/*)
        Mgr->>Pool: par_iter().filter_map(FileCache::from_file_sync)
        Pool-->>Mgr: Vec<(relative_path, FileCache)>
        loop per file
            Mgr->>Cache: insert(key, value)
        end
    end
    Mgr-->>Init: Ok (logs success / failure counts)
```

Server-level parallelism is governed by `cache.hash_concurrency`:
`load_all_servers` uses `stream::buffer_unordered(N)` to walk that
many servers at a time. Inside each server, file hashing is parallel
via rayon.

## Refresh-on-diff

Whenever `lighty-rescan` produces a non-empty `FileDiff`, the
orchestrator calls into the file cache to keep RAM consistent with
disk before any HTTP response can stale.

```mermaid
sequenceDiagram
    participant RU as rescan::update_cache_if_changed
    participant FC as FileCacheManager
    participant FS as filesystem

    Note over RU: diff already computed
    loop per change in diff.removed
        RU->>FC: invalidate_file(server, relative)
        FC->>FC: Moka::invalidate(key)
    end
    loop per change in diff.added ∪ diff.modified
        RU->>FC: refresh_file_from_disk(server, relative, full_path)
        FC->>FS: full_path.exists()?
        alt exists
            FC->>FS: read sync + sha1 + mime
            FC->>FC: Moka::insert(key, FileCache)
        else missing
            FC->>FC: invalidate_file(...)
        end
    end
```

The intent: a HTTP client that requests `survival/mods/iris.jar`
seconds after a mod swap gets the new bytes, not the stale ones.

## Server-prefix invalidation

When the operator removes a server from `config.toml` and saves, the
watcher triggers `CacheManager::remove_server`, which calls
`invalidate_server` here.

```mermaid
flowchart TD
    Watcher[config watcher detects rename/removal] --> Remove[CacheManager::remove_server]
    Remove --> Inv[FileCacheManager::invalidate_server]
    Inv --> Iter[Cache::iter]
    Iter --> Match{key.starts_with prefix?}
    Match -->|yes| Collect[push Arc to drop list]
    Match -->|no| Skip
    Collect --> Drop[Cache::invalidate per key]
```

Moka's iterator yields `Arc<str>` keys; the function clones the ones
that start with `"{server}/"` into a `Vec` and then invalidates them
one by one (Moka has no batch invalidate by prefix).

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`exports.md`](./exports.md)
- [`../../rescan/docs/flows.md`](../../rescan/docs/flows.md)
