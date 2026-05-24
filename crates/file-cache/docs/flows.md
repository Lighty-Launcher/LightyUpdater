# Flows

## Bulk load at boot

```mermaid
sequenceDiagram
    participant Init as CacheManager::initialize
    participant Mgr as FileCacheManager
    participant Pool as spawn_blocking + rayon
    participant Cache as Moka

    Init->>Mgr: load_all_servers
    loop per enabled server
        Mgr->>Mgr: walkdir + filter
        Mgr->>Pool: par_iter + from_file_sync
        Pool-->>Mgr: list of FileCache
        loop per file
            Mgr->>Cache: insert
        end
    end
```

`cache.hash_concurrency` servers are walked at a time via
`buffer_unordered`; inside each, rayon parallelises the hashing.

## Refresh on diff

```mermaid
sequenceDiagram
    participant Rescan as update_cache_if_changed
    participant FC as FileCacheManager
    participant FS as filesystem

    loop per change in removed
        Rescan->>FC: invalidate_file
    end
    loop per change in added or modified
        Rescan->>FC: refresh_file_from_disk
        FC->>FS: exists?
        alt exists
            FC->>FS: read + sha1 + mime
            FC->>FC: insert
        else missing
            FC->>FC: invalidate_file
        end
    end
```

Order: invalidate then refresh, so requests in flight never serve
stale bytes.

## Server-prefix invalidation

```mermaid
flowchart TD
    Remove[CacheManager::remove_server] --> Inv[invalidate_server]
    Inv --> Iter[Cache::iter]
    Iter --> Match{starts_with prefix}
    Match -->|yes| Drop[invalidate key]
    Match -->|no| Skip
```

Moka has no prefix-delete; the function collects matching keys then
invalidates one by one.

## See also

- [overview.md](./overview.md)
- [../../rescan/docs/flows.md](../../rescan/docs/flows.md)
