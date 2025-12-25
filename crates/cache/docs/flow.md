# Processing Flow

This document describes the main processing flows of the cache system with detailed sequence diagrams.

## Flow 1: Startup and Initialization

```mermaid
sequenceDiagram
    participant Main
    participant CM as CacheManager
    participant FCM as FileCacheManager
    participant SPC as ServerPathCache
    participant RO as RescanOrchestrator
    participant Config
    participant Events

    Main->>CM: new(config, events, storage, cloudflare)
    CM->>Config: read config
    Config-->>CM: settings

    CM->>FCM: new(max_cache_gb)
    FCM-->>CM: FileCacheManager

    CM->>SPC: new()
    SPC-->>CM: ServerPathCache
    CM->>SPC: rebuild(servers, base_path)

    CM->>RO: new(cache, config, events, storage, cloudflare)
    RO-->>CM: RescanOrchestrator

    Main->>CM: initialize()
    alt auto_scan enabled
        CM->>Events: emit InitialScanStarted
        CM->>RO: scan_all_servers()
        loop For each enabled server
            RO->>RO: Scan server
            RO->>CM: Update cache
        end
        CM->>FCM: load_all_servers()
    end

    Main->>CM: start_auto_rescan()
    CM->>RO: run_rescan_loop()
    alt rescan_interval == 0
        RO->>Events: emit ContinuousScanEnabled
        RO->>RO: run_file_watcher_loop()
    else rescan_interval > 0
        RO->>Events: emit AutoScanEnabled
        RO->>RO: run polling loop
    end
```

## Flow 2: Automatic Rescan (Polling Mode)

```mermaid
sequenceDiagram
    participant RO as RescanOrchestrator
    participant Timer as Tokio Timer
    participant Scanner
    participant FD as FileDiff
    participant Storage
    participant Cache
    participant Events

    RO->>Timer: interval(rescan_interval)
    Timer->>RO: tick

    loop For each enabled server
        RO->>RO: Check if paused
        alt Not paused
            RO->>Scanner: scan_server_silent()
            Scanner-->>RO: new VersionBuilder

            RO->>Cache: get old version
            Cache-->>RO: old VersionBuilder

            RO->>FD: compute(old, new)
            FD->>FD: diff_client()
            FD->>FD: diff_libraries()
            FD->>FD: diff_mods()
            FD->>FD: diff_natives()
            FD->>FD: diff_assets()
            FD-->>RO: FileDiff

            alt Has changes
                RO->>Storage: sync_cloud_storage()
                par Upload added/modified
                    Storage->>Storage: upload_file()
                and Delete removed
                    Storage->>Storage: delete_file()
                end

                alt First scan
                    RO->>RO: build_url_map()
                else Incremental update
                    RO->>FD: apply_to_url_map()
                end

                RO->>Cache: insert(server, version)
                RO->>Events: emit CacheUpdated
            else No changes
                RO->>Events: emit CacheUnchanged
            end
        end
    end

    Timer->>RO: tick (next interval)
```

## Flow 3: Automatic Rescan (File Watcher Mode)

```mermaid
sequenceDiagram
    participant FS as File System
    participant Watcher as Notify Watcher
    participant RO as RescanOrchestrator
    participant SPC as ServerPathCache
    participant Debounce as Debounce Timer
    participant Scanner

    FS->>Watcher: File modified
    Watcher->>RO: Event received

    RO->>RO: Check if paused
    alt Not paused
        loop For each path in event
            RO->>SPC: find_server(path)
            SPC-->>RO: server_name
            RO->>RO: Add to pending_servers
        end

        RO->>Debounce: Reset timer

        Debounce-->>RO: Timer expired

        loop For each pending server
            RO->>Scanner: scan_server_silent()
            Scanner-->>RO: VersionBuilder
            RO->>RO: update_cache_if_changed()
        end

        RO->>RO: Clear pending_servers
    end
```

## Flow 4: Granular Change Detection (FileDiff)

```mermaid
graph TD
    Start[Start: compute diff] --> CheckOld{Old version exists?}

    CheckOld -->|No| AddAll[add_all_files: Mark all as added]
    CheckOld -->|Yes| DiffTypes[Diff each file type]

    DiffTypes --> DiffClient[diff_client]
    DiffTypes --> DiffLibs[diff_libraries]
    DiffTypes --> DiffMods[diff_mods]
    DiffTypes --> DiffNatives[diff_natives]
    DiffTypes --> DiffAssets[diff_assets]

    subgraph diff_libraries_similar_for_mods_assets
        DiffLibs --> BuildMaps[Build old_map & new_map]
        BuildMaps --> IterNew[Iterate new_map]
        IterNew --> InOld{Exists in old?}
        InOld -->|Yes| CheckSHA1{SHA1 changed?}
        InOld -->|No| AddToAdded[Add to 'added']
        CheckSHA1 -->|Yes| AddToModified[Add to 'modified']
        CheckSHA1 -->|No| Skip[Skip - unchanged]
        BuildMaps --> IterOld[Iterate old_map]
        IterOld --> InNew{Exists in new?}
        InNew -->|No| AddToRemoved[Add to 'removed']
        InNew -->|Yes| SkipOld[Skip]
    end

    DiffClient --> Return[Return FileDiff]
    DiffLibs --> Return
    DiffMods --> Return
    DiffNatives --> Return
    DiffAssets --> Return
    AddAll --> Return
```

## Flow 5: Cloud Synchronization

```mermaid
sequenceDiagram
    participant RO as RescanOrchestrator
    participant FD as FileDiff
    participant Storage as Storage Backend
    participant S3

    RO->>FD: Get diff result
    FD-->>RO: added, modified, removed

    par Upload added files
        loop For each added file
            RO->>Storage: upload_file(local_path, remote_key)
            Storage->>S3: PUT object
            S3-->>Storage: Success
        end
    and Upload modified files
        loop For each modified file
            RO->>Storage: upload_file(local_path, remote_key)
            Storage->>S3: PUT object
            S3-->>Storage: Success
        end
    end

    par Delete removed files
        loop For each removed file
            RO->>Storage: delete_file(remote_key)
            Storage->>S3: DELETE object
            S3-->>Storage: Success
        end
    end

    RO->>RO: Log sync complete
```

## Flow 6: Configuration Hot-reload

```mermaid
sequenceDiagram
    participant Watcher as ConfigWatcher
    participant CM as CacheManager
    participant RO as RescanOrchestrator
    participant SPC as ServerPathCache
    participant Config
    participant Scanner

    Watcher->>Watcher: Config file modified
    Watcher->>Watcher: Debounce delay

    Watcher->>CM: pause_rescan()
    CM->>RO: pause()
    RO->>RO: Set paused flag

    Watcher->>Config: Load new config
    Config-->>Watcher: New Config

    Watcher->>Watcher: Detect changes
    Watcher->>Watcher: Identify: added, modified servers

    Watcher->>Config: Update shared config

    Watcher->>CM: rebuild_server_cache()
    CM->>SPC: rebuild(servers, base_path)

    Watcher->>CM: resume_rescan()
    CM->>RO: resume()
    RO->>RO: Clear paused flag

    loop For each modified server
        Watcher->>CM: force_rescan(server)
        CM->>RO: force_rescan_server(server)
        RO->>Scanner: scan_server()
        Scanner-->>RO: VersionBuilder
        RO->>CM: Update cache
    end

    loop For each added server
        Watcher->>Watcher: Create server structure
        Watcher->>CM: force_rescan(server)
    end
```

## Flow 7: Incremental URL Map Update

```mermaid
graph TD
    Start[FileDiff computed] --> CheckNew{Is first scan?}

    CheckNew -->|Yes| FullBuild[builder.build_url_map]
    CheckNew -->|No| Incremental[Apply incremental update]

    FullBuild --> BuildLoop[Iterate all files]
    BuildLoop --> AddMapping[Add URL → path mapping]
    AddMapping --> Done[Done]

    Incremental --> IterAdded[Iterate added files]
    IterAdded --> AddURL1[add_url_mapping]

    Incremental --> IterModified[Iterate modified files]
    IterModified --> AddURL2[add_url_mapping]

    Incremental --> IterRemoved[Iterate removed files]
    IterRemoved --> RemoveURL[remove_url_mapping]

    AddURL1 --> Done
    AddURL2 --> Done
    RemoveURL --> Done
```

## Flow 8: File LRU Cache

```mermaid
sequenceDiagram
    participant API
    participant CM as CacheManager
    participant FCM as FileCacheManager
    participant MokaCache as Moka LRU Cache
    participant Storage

    API->>CM: get_file(server, path)
    CM->>FCM: get_file(server, path)
    FCM->>MokaCache: get(key)

    alt File in cache
        MokaCache-->>FCM: FileCache
        FCM-->>CM: FileCache
        CM-->>API: FileCache
    else Cache miss
        MokaCache-->>FCM: None
        FCM->>Storage: read_file(path)
        Storage-->>FCM: File data
        FCM->>FCM: Compute SHA1, MIME type
        FCM->>MokaCache: insert(key, FileCache)
        alt Cache full
            MokaCache->>MokaCache: Evict LRU entry
        end
        FCM-->>CM: FileCache
        CM-->>API: FileCache
    end
```

## Flow 9: Graceful Shutdown

```mermaid
sequenceDiagram
    participant Main
    participant CM as CacheManager
    participant Broadcast
    participant Tasks
    participant FCM as FileCacheManager

    Main->>CM: shutdown()
    CM->>Broadcast: send shutdown signal
    Broadcast-->>Tasks: Receive signal

    loop For each task
        Tasks->>Tasks: Stop gracefully
    end

    CM->>CM: Wait for all tasks
    Tasks-->>CM: Task completed

    CM->>FCM: shutdown()
    FCM->>FCM: Wait for file cache tasks

    CM-->>Main: Shutdown complete
```

## Performance Metrics

### Time Complexity

- **FileDiff computation**: O(n) where n = total number of files
- **ServerPathCache lookup**: O(1) amortized
- **Version cache get/insert**: O(1) with DashMap
- **File cache get/insert**: O(1) with Moka
- **URL map update**: O(k) where k = number of changes

### Implemented Optimizations

1. **HashMap for FileDiff**: Avoids O(n²) comparisons
2. **Incremental URL map**: Avoids O(n) reconstruction
3. **Parallel file upload**: Reduces total synchronization time
4. **Debouncing**: Avoids unnecessary rescans
5. **Lock-free DashMap**: Maximizes concurrency
