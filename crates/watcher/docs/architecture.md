# Configuration Monitoring System Architecture

## Overview

The monitoring system is designed to detect and react to configuration file changes in real-time, with a robust race condition prevention mechanism.

## Diagramme d'architecture

```mermaid
graph TB
    ConfigFile[config.toml] --> FW[File Watcher<br/>notify::RecommendedWatcher]
    FW --> Channel[MPSC Channel]
    Channel --> CW[ConfigWatcher]
    CW --> Detector[Change Detector]
    Detector --> Comparator[Config Comparator]
    Comparator --> CacheManager[Cache Manager]
    Comparator --> FileSystem[File System]
    Comparator --> ConfigStore[Config Store<br/>Arc RwLock Config]
    CW --> Events[Event Bus]
```

## Main Components

### ConfigWatcher

Central structure managing configuration file monitoring.

**Data:**
- `config`: Arc<RwLock<Config>> - Shared read/write configuration
- `config_path`: String - Path to configuration file to monitor
- `cache_manager`: Arc<CacheManager> - Reference to cache manager

**Responsibilities:**
- Initialize file watcher with notify
- Handle communication channel for events
- Debounce multiple changes
- Coordinate configuration updates
- Trigger necessary rescans

### File Watcher (notify)

System component monitoring file modifications.

**Type:** `notify::RecommendedWatcher`
- Uses inotify on Linux
- Uses FSEvents on macOS
- Uses ReadDirectoryChangesW on Windows

**Captured events:**
- `EventKind::Modify`: File modified
- `EventKind::Create`: File created

**Mode:** `RecursiveMode::NonRecursive`
- Monitors only the specific file
- No recursive folder monitoring

### MPSC Channel

Asynchronous channel for communication between watcher and handler.

**Configuration:**
- Configurable capacity via `config.cache.config_reload_channel_size`
- Default: 10 events
- Type: `tokio::sync::mpsc::channel`

**Flow:**
- Producer: File watcher callback
- Consumer: ConfigWatcher main loop

### Change Detector

Configuration change detection logic.

**Comparisons:**
- Added servers detection
- Modified servers detection
- Removed servers detection

**Algorithm:**

```mermaid
graph TD
    Start[Configuration change detected] --> LoadNew[Load new config]
    LoadNew --> GetOld[Get old server list]
    GetOld --> GetOldConfigs[Get old server configs]

    GetOldConfigs --> BuildMap[Build HashMap of old configs]
    BuildMap --> CompareServers[Compare server names]
    CompareServers --> Added[Detect added servers]
    CompareServers --> Removed[Detect removed servers]

    BuildMap --> CompareConfigs[Compare server configs via HashMap]
    CompareConfigs --> Modified[Detect modified servers]

    Added --> Actions
    Modified --> Actions
    Removed --> Actions

    Actions[Execute actions]
```

**Performance:**
Server config comparison uses a HashMap for fast lookups when checking modifications.

## Data Flow

### Initialization

```mermaid
sequenceDiagram
    participant App
    participant CW as ConfigWatcher
    participant Config
    participant CM as CacheManager

    App->>Config: Load initial config
    Config-->>App: Config instance
    App->>CW: new(config, path, cache_manager)
    CW->>CW: Store references
    CW-->>App: ConfigWatcher instance

    App->>CW: start_watching()
    CW->>CW: spawn async task
    CW->>CW: Setup file watcher
    CW-->>App: JoinHandle
```

### Change Detection and Processing

```mermaid
sequenceDiagram
    participant FS as File System
    participant FW as File Watcher
    participant Chan as Channel
    participant CW as ConfigWatcher
    participant CM as CacheManager
    participant Config as Config Store

    FS->>FW: File modified event
    FW->>Chan: Send notification
    Chan->>CW: Receive event

    CW->>CW: Debounce wait
    Note over CW: Wait hot_reload.config.debounce_ms

    CW->>CW: Check file exists
    CW->>Config: Load new config from file

    alt Config valid
        Config-->>CW: New Config

        CW->>CM: pause_rescan()
        Note over CM: Prevent race conditions

        CW->>Config: Read old servers
        CW->>Config: Read old configs
        CW->>CW: Detect changes

        CW->>Config: Write new config
        CW->>CM: rebuild_server_cache()
        CW->>CM: resume_rescan()

        alt Has modified servers
            loop For each modified server
                CW->>CM: force_rescan(server)
            end
        end

        alt Has added servers
            loop For each added server
                CW->>FS: ensure_server_structure()
                CW->>CM: force_rescan(server)
            end
        end
    else Config invalid
        Config-->>CW: Error
        CW->>CW: Log error
    end
```

## Configuration Change Detection

### Server Comparison

```mermaid
graph TD
    OldServers[Old server names] --> SetOld[HashSet old_servers]
    NewServers[New server names] --> SetNew[HashSet new_servers]

    SetNew --> Diff1[new - old]
    SetOld --> Diff2[old - new]

    Diff1 --> Added[Added servers]
    Diff2 --> Removed[Removed servers]

    SetNew --> Intersect[new ∩ old]
    SetOld --> Intersect
    Intersect --> Existing[Existing servers]

    Existing --> Compare[Compare configs]
    Compare --> Modified[Modified servers]
```

### Monitored Fields for Modifications

The system checks the following fields to detect significant modifications:

**Server configuration fields:**
- `enabled`: Server activation/deactivation
- `loader`: Loader type (Forge, Fabric, etc.)
- `loader_version`: Loader version
- `minecraft_version`: Minecraft version
- `main_class`: Java main class
- `java_version`: Required Java version
- `enable_client`: Client JAR activation
- `enable_libraries`: Libraries activation
- `enable_mods`: Mods activation
- `enable_natives`: Natives activation
- `enable_assets`: Assets activation
- `game_args`: Game arguments
- `jvm_args`: JVM arguments

**Comparison algorithm:**

```rust
fn server_config_changed(old: &ServerConfig, new: &ServerConfig) -> bool {
    old.enabled != new.enabled
        || old.loader != new.loader
        || old.loader_version != new.loader_version
        || old.minecraft_version != new.minecraft_version
        // ... other fields
}
```

## Race Condition Prevention

### Problem

```mermaid
sequenceDiagram
    participant Rescan as Auto Rescan
    participant Config as Config Reload
    participant Cache as Cache

    Note over Rescan,Cache: POTENTIAL RACE CONDITION

    par Without synchronization
        Rescan->>Cache: Read old config
        and
        Config->>Cache: Write new config
    end

    Rescan->>Cache: Scan with mixed old/new data
    Note over Cache: Possible corruption!
```

### Solution: Pause/Resume

```mermaid
sequenceDiagram
    participant Config as Config Reload
    participant CM as CacheManager
    participant Rescan as Auto Rescan

    Config->>CM: pause_rescan()
    Note over Rescan: Rescan paused

    Config->>CM: Read old config
    Config->>CM: Write new config
    Config->>CM: rebuild_server_cache()

    Config->>CM: resume_rescan()
    Note over Rescan: Rescan resumed

    Rescan->>CM: Continue with new config
```

**Mechanism:**
- CacheManager uses an atomic flag to pause/resume
- Automatic rescans check this flag before executing
- Config reload is protected by an exclusive write lock
- Guaranteed sequence: pause → update → resume

## Debouncing

### Problem Without Debouncing

```mermaid
sequenceDiagram
    participant Editor
    participant FS as File System
    participant Watcher

    Editor->>FS: Save (temporary write)
    FS->>Watcher: Modify event #1
    Watcher->>Watcher: Reload config

    Editor->>FS: Save (final write)
    FS->>Watcher: Modify event #2
    Watcher->>Watcher: Reload config

    Editor->>FS: Update metadata
    FS->>Watcher: Modify event #3
    Watcher->>Watcher: Reload config

    Note over Watcher: 3 reloads for 1 save!
```

### Solution With Debouncing

```mermaid
sequenceDiagram
    participant Editor
    participant FS as File System
    participant Watcher
    participant Timer

    Editor->>FS: Save (temporary)
    FS->>Watcher: Event #1
    Watcher->>Timer: Start debounce timer

    Editor->>FS: Save (final)
    FS->>Watcher: Event #2
    Watcher->>Timer: Reset debounce timer

    Editor->>FS: Update metadata
    FS->>Watcher: Event #3
    Watcher->>Timer: Reset debounce timer

    Note over Timer: Wait hot_reload.config.debounce_ms

    Timer->>Watcher: Timer expired
    Watcher->>Watcher: Reload config (once)
```

**Configuration:**
- Configurable delay via `config.hot_reload.config.debounce_ms`
- Default: 300ms
- Enable/disable via `config.hot_reload.config.enabled`
- Each event resets the timer
- Reload only executes after delay with no new events

## Concurrency Management

### Used Locks

```mermaid
graph TB
    R1[Get current config] --> ReadLock[Read Lock]
    R2[Check server existence] --> ReadLock
    R3[Read server list] --> ReadLock
    ReadLock --> RwLock[Arc RwLock Config]

    W1[Update config] --> WriteLock[Write Lock - EXCLUSIVE]
    W2[Rebuild cache] --> WriteLock
    W3[Force rescan] --> WriteLock
    WriteLock --> RwLock
```

**Strategy:**
- Multiple simultaneous reads possible
- Exclusive write blocks all reads
- Lock released between I/O operations to avoid deadlocks
- Pattern: acquire → read/write → drop → I/O → re-acquire

### Lock Release Pattern

```rust
// Acquire lock
let mut config_write = config.write().await;

// Config operations
*config_write = new_config;
cache_manager.rebuild_server_cache().await;

// Explicit release before I/O
drop(config_write);

// I/O operations without lock
FileSystem::ensure_server_structure(&path, &folder).await;
cache_manager.force_rescan(&server).await;

// Re-acquire if necessary
config_write = config.write().await;
```

## Optimizations

### Channel Sizing

Channel size based on:
- Config file modification frequency
- Processing time per event
- Available memory

**Recommendations:**
- Development: 10 (default)
- Stable production: 5
- Frequent hot-reload: 20

### HashSet for Comparison

Using HashSet for efficient comparisons:
- Converting server lists to HashSet
- Fast difference and intersection operations
- Efficient server list comparisons

### Lazy Evaluation

Expensive operations only execute when necessary:
- No rescan if no server modified
- No folder creation if server disabled
- File validation before full reload

## Extensibility

### Adding New Change Types

To monitor other aspects of the configuration:

1. Add fields in `server_config_changed`
2. Implement comparison logic
3. Trigger appropriate actions
4. Document the behavior

### Multiple File Support

The system can be extended to monitor multiple files:
- One watcher per file
- Change aggregation
- Centralized coordination
