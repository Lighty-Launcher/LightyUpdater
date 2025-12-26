# Hot-Reload Mechanism

## Overview

Hot-reload allows modifying the server configuration without restart, with automatic update of all affected components.

## Hot-Reload Architecture

```mermaid
graph TB
    subgraph Detection
        FW[File Watcher]
        Debounce[Debouncer]
    end

    subgraph Validation
        Parser[Config Parser]
        Validator[Config Validator]
    end

    subgraph Synchronization
        Pause[Pause Rescan]
        Lock[Write Lock]
        Resume[Resume Rescan]
    end

    subgraph Updates
        ConfigStore[Config Store]
        CacheRebuild[Cache Rebuild]
        ServerRescan[Server Rescan]
    end

    FW --> Debounce
    Debounce --> Parser
    Parser --> Validator

    Validator --> Pause
    Pause --> Lock
    Lock --> ConfigStore
    ConfigStore --> CacheRebuild
    CacheRebuild --> Resume
    Resume --> ServerRescan
```

## Hot-Reload Phases

### Phase 1: Detection

```mermaid
sequenceDiagram
    participant Editor
    participant OS
    participant Watcher

    Editor->>OS: Write config.toml
    Note over Editor: User saves file

    OS->>Watcher: File system event
    Watcher->>Watcher: Start debounce timer

    Note over Watcher: Wait for stable state

    Watcher->>Watcher: Timer expires
    Watcher->>Watcher: Proceed to validation
```

**Debouncing:**
- Avoids multiple reloads during successive saves
- Configurable delay (default: 500ms)
- Timer reset on each new event

### Phase 2: Validation

```mermaid
sequenceDiagram
    participant Watcher
    participant FileSystem
    participant Parser
    participant Validator

    Watcher->>FileSystem: Check file exists

    alt File exists
        Watcher->>Parser: Parse TOML
        Parser->>Validator: Validate structure

        alt Valid
            Validator-->>Watcher: Ok(Config)
        else Invalid
            Validator-->>Watcher: Err(ConfigError)
            Watcher->>Watcher: Keep old config
        end
    else File deleted
        Watcher->>Watcher: Log warning + skip
    end
```

**Validation including:**
- Correct TOML syntax
- Required fields present
- Correct data types
- Values within acceptable limits
- Logical configuration consistency

### Phase 3: Synchronization

```mermaid
sequenceDiagram
    participant Watcher
    participant CacheManager
    participant RescanOrch as Rescan Orchestrator
    participant Config

    Watcher->>CacheManager: pause_rescan()
    Note over RescanOrch: All auto-rescans paused

    Watcher->>Config: Acquire write lock
    Note over Config: Exclusive access

    Watcher->>Config: Read old configuration
    Watcher->>Watcher: Detect changes
    Watcher->>Config: Write new configuration

    Watcher->>CacheManager: rebuild_server_cache()
    Note over CacheManager: Update path mappings

    Watcher->>Config: Release write lock
    Watcher->>CacheManager: resume_rescan()
    Note over RescanOrch: Auto-rescans resumed
```

**Guarantees:**
- No automatic rescan during update
- Exclusive access to configuration
- Update atomicity
- Guaranteed cache consistency

### Phase 4: Applying Changes

```mermaid
sequenceDiagram
    participant Watcher
    participant FileSystem
    participant CacheManager
    participant Scanner

    alt Modified servers
        loop For each modified
            Watcher->>CacheManager: force_rescan(server)
            CacheManager->>Scanner: Scan server
            Scanner-->>CacheManager: New version
            CacheManager->>CacheManager: Update cache
        end
    end

    alt Added servers
        loop For each added
            Watcher->>FileSystem: Create structure
            FileSystem->>FileSystem: Create directories
            FileSystem-->>Watcher: Structure ready

            Watcher->>CacheManager: force_rescan(server)
            CacheManager->>Scanner: Initial scan
            Scanner-->>CacheManager: Version data
        end
    end
```

## Server Management

### Added Server

```mermaid
graph TD
    Detect[New server detected] --> CheckEnabled{enabled = true?}

    CheckEnabled -->|No| Skip[Skip server]
    CheckEnabled -->|Yes| CreateStruct[Create directory structure]

    CreateStruct --> Dirs[Create subdirectories]
    Dirs --> Client[client/]
    Dirs --> Libs[libraries/]
    Dirs --> Mods[mods/]
    Dirs --> Natives[natives/windows,linux,macos]
    Dirs --> Assets[assets/]

    Client --> Scan[Initial scan]
    Libs --> Scan
    Mods --> Scan
    Natives --> Scan
    Assets --> Scan

    Scan --> Cache[Add to cache]
```

### Modified Server

```mermaid
graph TD
    Detect[Config change detected] --> Compare[Compare old vs new]

    Compare --> Critical{Critical change?}

    Critical -->|Yes| Rescan[Full rescan]
    Critical -->|No| SkipRescan[No rescan needed]

    Rescan --> InvalidateCache[Invalidate cache entry]
    InvalidateCache --> ScanFiles[Scan files]
    ScanFiles --> UpdateCache[Update cache]
    UpdateCache --> Notify[Notify clients]
```

**Critical changes:**
- Modified Minecraft version
- Loader change
- Activated/deactivated components

**Non-critical changes:**
- Name modification (display_name)
- Description change
- Metadata update

### Removed Server

```mermaid
graph TD
    Detect[Server removed from config] --> RemoveCache[Remove from cache]

    RemoveCache --> CleanMemory[Free memory]
    CleanMemory --> UpdatePaths[Update path cache]

    UpdatePaths --> Decision{Delete files?}

    Decision -->|Manual cleanup| Keep[Files remain on disk]
    Decision -->|Auto cleanup| Delete[Delete via API]
```

**Current behavior:**
- Files kept on disk
- Memory cache cleaned
- Path mappings updated
- No automatic deletion

## Race Condition Prevention

### Problem Without Synchronization

```mermaid
sequenceDiagram
    participant AutoRescan
    participant ConfigReload
    participant Cache

    par Concurrent access
        AutoRescan->>Cache: Read config
        Note over AutoRescan: Using old config
        and
        ConfigReload->>Cache: Write new config
        Note over Cache: Config updated
    end

    AutoRescan->>Cache: Scan with old config
    Note over Cache: Inconsistent state!
```

### Solution avec pause/resume

```mermaid
sequenceDiagram
    participant AutoRescan
    participant ConfigReload
    participant PauseFlag
    participant Cache

    ConfigReload->>PauseFlag: Set paused = true

    loop Auto rescan checks
        AutoRescan->>PauseFlag: Check if paused
        PauseFlag-->>AutoRescan: paused = true
        Note over AutoRescan: Wait...
    end

    ConfigReload->>Cache: Update config
    ConfigReload->>PauseFlag: Set paused = false

    AutoRescan->>PauseFlag: Check if paused
    PauseFlag-->>AutoRescan: paused = false
    AutoRescan->>Cache: Continue with new config
```

## System Impact

### During Reload

**Paused operations:**
- Periodic automatic rescan
- File watcher triggered rescan
- Automatic cache update

**Continuing operations:**
- File serving via API
- GET requests to servers
- Read access to existing cache

### After Reload

**Triggered operations:**
- Rescan of modified servers
- Initial scan of added servers
- Path cache reconstruction
- Change event emission

## Performance

### Optimizations

**Minimize lock time:**
```rust
// Acquire lock
let mut config_write = config.write().await;

// Fast operations only
*config_write = new_config;
cache_manager.rebuild_server_cache().await;

// Release immediately
drop(config_write);

// Slow I/O operations without lock
FileSystem::ensure_server_structure().await;
cache_manager.force_rescan().await;
```

**Parallelize rescans:**
```rust
// Rescan multiple servers in parallel
let tasks: Vec<_> = modified_servers
    .iter()
    .map(|server| {
        let cache = cache_manager.clone();
        let name = server.clone();
        tokio::spawn(async move {
            cache.force_rescan(&name).await
        })
    })
    .collect();

// Wait for all
futures::future::join_all(tasks).await;
```

## Observabilite

### Emitted Logs

```rust
// Reload start
tracing::info!("Configuration reloaded successfully from {}", path);

// Modified servers
tracing::info!("Server config changed, rescanning: {}", server_name);

// New servers
tracing::info!("New server detected: {}", server_name);

// Errors
tracing::error!("Failed to reload config: {}", error);
```

### Metrics

- Number of successful reloads
- Number of failed reloads
- Average reload time
- Number of modified servers per reload
- Number of added servers per reload

## Use Cases

### Development Workflow

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Editor
    participant System

    Dev->>Editor: Edit config.toml
    Dev->>Editor: Add new server
    Editor->>System: Save file

    Note over System: Hot-reload triggered

    System->>System: Create directories
    System->>System: Scan server
    System->>System: Update cache

    Dev->>Dev: Test immediately
    Note over Dev: No restart needed
```

### Production Update

```mermaid
sequenceDiagram
    participant Admin
    participant Config
    participant System
    participant Clients

    Admin->>Config: Update minecraft version
    Config->>System: Hot-reload

    System->>System: Rescan server
    System->>System: Update cache

    Clients->>System: GET /server.json
    System-->>Clients: New version data

    Note over Clients: Clients get update automatically
```

## Limitations

**No automatic rollback:**
- If new config invalid, old one remains active
- No config version system
- No automatic snapshot

**No semantic validation:**
- Checks syntax, not business logic
- Doesn't check if files exist
- Doesn't validate Minecraft versions

**Impact on clients:**
- Connected clients are not notified
- Must re-fetch to see changes
- No websocket or SSE for push updates
