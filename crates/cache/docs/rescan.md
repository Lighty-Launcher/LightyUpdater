# RescanOrchestrator

Detailed documentation of the automatic and manual rescan system.

## Overview

The `RescanOrchestrator` is the central component that orchestrates monitoring and automatic cache updating. It supports two distinct operating modes and offers manual rescan capabilities.

## Architecture

```mermaid
flowchart TB
    RO[RescanOrchestrator] --> PM[Polling Mode]
    RO --> FW[File Watcher Mode]
    RO --> MR[Manual Rescan]

    PM --> TI[Tokio Interval]
    PM --> SA[Scan All Servers]

    FW --> NW[Notify Watcher]
    FW --> DB[Debouncing]
    FW --> SPC[ServerPathCache]

    MR --> FS[Force Single Server]
    MR --> FAL[Force All Servers]

    SA --> FD[FileDiff]
    DB --> FD
    FS --> FD

    FD --> SCS[Sync Cloud Storage]
    SCS --> PCDN[Purge CDN]
    FD --> PCFAPI[Purge Cloudflare API]
```

## Operating Modes

### Polling Mode

Active when `rescan_interval > 0` in the configuration.

**Principle**:
- Periodic rescan based on a fixed time interval
- Checks all active servers at each iteration
- Uses `tokio::time::interval` for precise timing

**Advantages**:
- Simple and predictable
- Works even if the file system doesn't support notify
- Guarantees regular verification

**Disadvantages**:
- Potential delay between the change and detection
- Regular CPU consumption even without changes

**Configuration**:
```toml
[cache]
rescan_interval = 30  # Rescan every 30 seconds
```

**Sequence diagram**:

```mermaid
sequenceDiagram
    participant Timer
    participant RO as RescanOrchestrator
    participant Config
    participant Scanner

    RO->>Config: Read rescan_interval
    Config-->>RO: 30 seconds

    RO->>Timer: Create interval(30s)
    Timer->>RO: First tick (immediate)

    loop Every 30 seconds
        Timer->>RO: Tick

        RO->>RO: Check paused flag
        alt Not paused
            loop For each enabled server
                RO->>Scanner: scan_server_silent()
                Scanner-->>RO: VersionBuilder
                RO->>RO: update_cache_if_changed()
            end
        end

        Timer->>RO: Next tick (30s later)
    end
```

### File Watcher Mode

Active when `rescan_interval = 0` in the configuration.

**Principle**:
- Real-time file change monitoring
- Uses the `notify` library to receive file system events
- Debouncing to avoid multiple rapid rescans
- O(1) lookup via `ServerPathCache` to identify the affected server

**Advantages**:
- Immediate change detection
- No CPU consumption when there are no changes
- Optimization: rescan only the modified server

**Disadvantages**:
- Requires file system support (inotify on Linux, FSEvents on macOS)
- More complex to debug
- Can generate many events during massive modifications

**Configuration**:
```toml
[cache]
rescan_interval = 0  # Activate file watcher

[hot-reload.files]
enabled = true  # Enable file hot-reload
debounce_ms = 300  # Wait 300ms after the last event
```

**Sequence diagram**:

```mermaid
sequenceDiagram
    participant FS as File System
    participant Notify
    participant RO as RescanOrchestrator
    participant SPC as ServerPathCache
    participant Timer as Debounce Timer
    participant Scanner

    RO->>Notify: Setup watcher
    loop For each enabled server
        RO->>Notify: watch(server_path, Recursive)
    end

    FS->>Notify: File created/modified/removed
    Notify->>RO: Event received

    RO->>RO: Check paused flag
    alt Not paused
        loop For each path in event
            RO->>SPC: find_server(path)
            SPC-->>RO: "survival"
            RO->>RO: Add to pending_servers
        end

        RO->>Timer: Reset to 300ms

        Note over RO,Timer: More events arrive...
        FS->>Notify: Another file modified
        Notify->>RO: Event received
        RO->>Timer: Reset to 300ms again

        Note over Timer: 300ms pass without events
        Timer-->>RO: Debounce expired

        RO->>RO: Build HashMap of servers for O(1) lookup
        loop For each server in pending_servers
            RO->>RO: Lookup server config in HashMap
            RO->>Scanner: scan_server_silent("survival")
            Scanner-->>RO: VersionBuilder
            RO->>RO: update_cache_if_changed()
        end

        RO->>RO: Clear pending_servers
    end
```

**Debouncing algorithm**:

```mermaid
flowchart TD
    Start[Event received] --> CheckPause{Paused?}
    CheckPause -->|Yes| Skip[Skip event]
    CheckPause -->|No| FindServer[Find server via ServerPathCache]

    FindServer --> AddPending[Add to pending_servers]
    AddPending --> CheckTimer{Timer active?}

    CheckTimer -->|Yes| ResetTimer[Reset timer]
    CheckTimer -->|No| StartTimer[Start new timer]

    ResetTimer --> Wait[Wait for events]
    StartTimer --> Wait

    Wait --> MoreEvents{More events?}
    MoreEvents -->|Yes| Start
    MoreEvents -->|No| TimerExpire[Timer expires]

    TimerExpire --> RescanAll[Rescan all pending servers]
    RescanAll --> ClearPending[Clear pending_servers]
    ClearPending --> End[End]

    Skip --> End
```

## Pause and Resume

The RescanOrchestrator can be temporarily paused, particularly during configuration hot-reload.

**Usage**:

```rust
orchestrator.pause();   // Pause rescan
// ... reload configuration ...
orchestrator.resume();  // Resume rescan
```

**Implementation**:

Uses an `AtomicBool` for a thread-safe flag without locking:

```mermaid
sequenceDiagram
    participant Watcher as ConfigWatcher
    participant RO as RescanOrchestrator
    participant Flag as AtomicBool

    Watcher->>RO: pause()
    RO->>Flag: store(true, SeqCst)

    Note over RO: Rescan loop checks flag
    RO->>Flag: load(Relaxed)
    Flag-->>RO: true
    RO->>RO: Skip rescan iteration

    Watcher->>Watcher: Reload config
    Watcher->>RO: resume()
    RO->>Flag: store(false, SeqCst)

    RO->>Flag: load(Relaxed)
    Flag-->>RO: false
    RO->>RO: Continue normal rescanning
```

**Memory Ordering:**
- Store operations use `SeqCst` for strong synchronization
- Load operations use `Relaxed` ordering for simple flag checks

**Importance**: Avoids race conditions during configuration reload by guaranteeing that no rescan occurs during shared config update.

## Change Detection and Synchronization

### update_cache_if_changed Algorithm

```mermaid
flowchart TD
    Start[New VersionBuilder scanned] --> GetOld[Get old version from cache]
    GetOld --> ComputeDiff[FileDiff::compute old vs new]

    ComputeDiff --> CheckChanges{Has changes?}

    CheckChanges -->|No| EmitUnchanged[Emit CacheUnchanged event]
    EmitUnchanged --> End[End]

    CheckChanges -->|Yes| CheckStorage{Storage is remote?}

    CheckStorage -->|Yes| SyncCloud[sync_cloud_storage]
    SyncCloud --> UploadMod[Upload added/modified files in parallel]
    UploadMod --> DeleteRem[Delete removed files in parallel]
    DeleteRem --> CheckNew

    CheckStorage -->|No| CheckNew{Is first scan?}

    CheckNew -->|Yes| FullBuild[build_url_map full]
    CheckNew -->|No| Incremental[apply_to_url_map incremental]

    FullBuild --> UpdateCache[Insert in cache]
    Incremental --> UpdateCache

    UpdateCache --> UpdateTimestamp[Update last_updated]
    UpdateTimestamp --> PurgeCFAPI{Cloudflare API enabled?}

    PurgeCFAPI -->|Yes| CallPurgeAPI[purge_cache API JSON]
    PurgeCFAPI -->|No| EmitEvent

    CallPurgeAPI --> EmitEvent{Is new server?}

    EmitEvent -->|Yes| EmitNew[Emit CacheNew]
    EmitEvent -->|No| EmitUpdated[Emit CacheUpdated with changes]

    EmitNew --> End
    EmitUpdated --> End
```

### Cloud Synchronization

When the storage backend is remote (S3, etc.), the system automatically synchronizes changes:

**Parallelized upload**:

```mermaid
flowchart LR
    Diff[FileDiff] --> Added[Added files]
    Diff --> Modified[Modified files]

    Added --> T1[Task 1: Upload]
    Added --> T2[Task 2: Upload]
    Added --> T3[Task 3: Upload]

    Modified --> T4[Task 4: Upload]
    Modified --> T5[Task 5: Upload]

    T1 --> S3[S3 Storage]
    T2 --> S3
    T3 --> S3
    T4 --> S3
    T5 --> S3

    S3 --> Wait[futures::join_all]
```

**Parallelized delete**:

```mermaid
flowchart LR
    Diff[FileDiff] --> Removed[Removed files]

    Removed --> D1[Task 1: Delete]
    Removed --> D2[Task 2: Delete]
    Removed --> D3[Task 3: Delete]

    D1 --> S3[S3 Storage]
    D2 --> S3
    D3 --> S3

    S3 --> Wait[futures::join_all]
```

**Advantages**:
- Maximum parallelization of uploads/deletes
- No sequential waiting
- Optimization of total synchronization time

## Manual Rescan

### Force Rescan of a Single Server

```rust
orchestrator.force_rescan_server("survival").await?;
```

**Use cases**:
- Manual file correction
- Debugging
- On-demand rescan API
- Configuration hot-reload with server change

**Behavior**:
- Complete server scan
- Complete URL map reconstruction
- Immediate cache update
- No CacheUpdated event emission (manual operation)

### Scan All Servers

```rust
orchestrator.scan_all_servers().await?;
```

**Use cases**:
- System initialization
- Complete cache reconstruction
- Recovery after corruption

**Behavior**:
- Parallel scan of all active servers
- Each server is scanned independently
- Individual failures don't prevent other scans
- Event emission for each scanned server

**Diagram**:

```mermaid
sequenceDiagram
    participant Main
    participant RO as RescanOrchestrator
    participant Scanner
    participant Storage

    Main->>RO: scan_all_servers()

    par Scan server 1
        RO->>Scanner: scan_server(config1)
        Scanner->>Storage: Get URLs
        Storage-->>Scanner: URLs
        Scanner-->>RO: VersionBuilder1
        RO->>RO: Update cache
    and Scan server 2
        RO->>Scanner: scan_server(config2)
        Scanner->>Storage: Get URLs
        Storage-->>Scanner: URLs
        Scanner-->>RO: VersionBuilder2
        RO->>RO: Update cache
    and Scan server 3
        RO->>Scanner: scan_server(config3)
        Scanner->>Storage: Get URLs
        Storage-->>Scanner: URLs
        Scanner-->>RO: VersionBuilder3
        RO->>RO: Update cache
    end

    RO-->>Main: All servers scanned
```

## Optimizations

### ServerPathCache

Cache for efficient lookup of the owner server of a file:

```rust
// Structure: Vec sorted by path length (descending)
// Paths sorted to ensure most specific matches first

// Example:
// [
//   ("/servers/survival-modded", "survival-modded"),  // longest first
//   ("/servers/survival", "survival"),
//   ("/servers/creative", "creative")
// ]

server_path_cache.find_server(&path)
```

**Implementation:**
- Sorted Vec ensures the first matching path is the most specific
- Early termination on first match
- Efficient for typical deployments with few servers

**Impact**: Essential for the file watcher which potentially receives hundreds of events per second. The sorted structure ensures correct matching for nested server paths.

### Silent Scan

`scan_server_silent` vs `scan_server`:
- Silent version: no detailed logging
- Used for frequent automatic rescans
- Reduces noise in logs

### Incremental URL Map

Avoids complete URL map reconstruction:
- First scan: `build_url_map()` - Full map construction
- Rescans: `apply_to_url_map(&diff)` - Updates only changed files

Significant performance gain when few files change.

## Error Handling

### Recoverable Errors

- Temporarily unavailable server: Skip and retry in next cycle
- Missing file: Log warning, continue scan
- Cloud timeout: Log error, local cache remains functional

### Critical Errors

- Uninitialized storage backend: Error propagation
- Complete scan all servers failure: Error event emission

### Error Isolation

Each server is scanned independently:
- Failure of server A doesn't affect server B
- Successful servers are cached even if others fail
