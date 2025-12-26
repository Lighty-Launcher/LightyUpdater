# Configuration Change Detection

## Overview

The detection system precisely identifies which servers have been added, modified, or removed during a configuration reload.

## Detection Algorithm

### Step 1: Building Sets

```mermaid
graph LR
    OldConfig[Old Config] --> ExtractOld[Extract server names]
    NewConfig[New Config] --> ExtractNew[Extract server names]

    ExtractOld --> SetOld[HashSet old_servers]
    ExtractNew --> SetNew[HashSet new_servers]

    SetOld --> Operations
    SetNew --> Operations

    Operations[Set Operations]
```

### Step 2: Added/Removed Servers Detection

```mermaid
graph TD
    SetNew[new_servers] --> Diff1[Difference]
    SetOld[old_servers] --> Diff1

    Diff1 --> Added["added = new - old<br/>(servers in new but not in old)"]
    Diff1 --> Removed["removed = old - new<br/>(servers in old but not in new)"]
```

### Step 3: Modified Servers Detection

```mermaid
graph TD
    NewServers[new_config.servers] --> Loop[For each server]
    OldConfigs[old_configs] --> Loop

    Loop --> Find{Find in old configs?}

    Find -->|Not found| Skip[Skip - it's a new server]
    Find -->|Found| Compare[Compare configurations]

    Compare --> Changed{Config changed?}

    Changed -->|Yes| AddToModified[Add to modified list]
    Changed -->|No| SkipUnchanged[Skip unchanged]
```

## Monitored Fields

### Critical Fields Triggering Rescan

Changes to these fields require a complete server rescan:

**Server identity:**
- `loader`: Change from Forge to Fabric, etc.
- `loader_version`: New loader version
- `minecraft_version`: Minecraft version change

**Java configuration:**
- `main_class`: Different main class
- `java_version`: Modified required Java version

**Component activation:**
- `enable_client`: Activate/deactivate client JAR
- `enable_libraries`: Activate/deactivate libraries
- `enable_mods`: Activate/deactivate mods
- `enable_natives`: Activate/deactivate natives
- `enable_assets`: Activate/deactivate assets

**Arguments:**
- `game_args`: Modified game arguments
- `jvm_args`: Modified JVM arguments

**Status:**
- `enabled`: Server activation/deactivation

### Comparison Implementation

```rust
fn server_config_changed(old: &ServerConfig, new: &ServerConfig) -> bool {
    old.enabled != new.enabled
        || old.loader != new.loader
        || old.loader_version != new.loader_version
        || old.minecraft_version != new.minecraft_version
        || old.main_class != new.main_class
        || old.java_version != new.java_version
        || old.enable_client != new.enable_client
        || old.enable_libraries != new.enable_libraries
        || old.enable_mods != new.enable_mods
        || old.enable_natives != new.enable_natives
        || old.enable_assets != new.enable_assets
        || old.game_args != new.game_args
        || old.jvm_args != new.jvm_args
}
```

## Change Scenarios

### Scenario 1: Adding a New Server

```mermaid
sequenceDiagram
    participant Old as Old Config
    participant New as New Config
    participant Detector
    participant Actions

    Old->>Detector: servers = ["server1", "server2"]
    New->>Detector: servers = ["server1", "server2", "server3"]

    Detector->>Detector: added = ["server3"]
    Detector->>Detector: modified = []
    Detector->>Detector: removed = []

    Detector->>Actions: Create structure for server3
    Detector->>Actions: Force rescan server3
```

### Scenario 2: Modifying an Existing Server

```mermaid
sequenceDiagram
    participant Old as Old Config
    participant New as New Config
    participant Detector
    participant Actions

    Old->>Detector: server1: {loader: "forge", version: "1.19.2"}
    New->>Detector: server1: {loader: "forge", version: "1.20.1"}

    Detector->>Detector: added = []
    Detector->>Detector: modified = ["server1"]
    Detector->>Detector: removed = []

    Detector->>Actions: Force rescan server1
```

### Scenario 3: Removing a Server

```mermaid
sequenceDiagram
    participant Old as Old Config
    participant New as New Config
    participant Detector
    participant Actions

    Old->>Detector: servers = ["server1", "server2", "server3"]
    New->>Detector: servers = ["server1", "server2"]

    Detector->>Detector: added = []
    Detector->>Detector: modified = []
    Detector->>Detector: removed = ["server3"]

    Note over Actions: No automatic cleanup<br/>Files remain on disk
```

### Scenario 4: Multiple Changes

```mermaid
sequenceDiagram
    participant Old as Old Config
    participant New as New Config
    participant Detector
    participant Actions

    Old->>Detector: ["server1", "server2", "server3"]
    New->>Detector: ["server1:modified", "server2", "server4"]

    Detector->>Detector: added = ["server4"]
    Detector->>Detector: modified = ["server1"]
    Detector->>Detector: removed = ["server3"]

    par Parallel actions
        Detector->>Actions: Create structure server4
        Detector->>Actions: Rescan server4
        and
        Detector->>Actions: Rescan server1
    end
```

## Optimizations

### Using HashSet

**Advantages:**
- O(1) lookups instead of O(n)
- Efficient set operations
- Fast comparisons

### Lazy Comparison

Configuration comparison is only done for servers existing in both configs:

```rust
// New server: no comparison necessary
if !old_servers.contains(&server.name) {
    continue;
}

// Existing server: comparison necessary
if let Some(old_server) = old_configs.iter().find(|s| s.name == server.name) {
    if server_config_changed(old_server, &server) {
        modified_servers.push(server.name.clone());
    }
}
```

### Rescan Batching

Rescans can be batched to optimize performance:

```mermaid
graph LR
    Modified[Modified servers] --> Batch[Group by priority]
    Batch --> High[High priority]
    Batch --> Low[Low priority]

    High --> Immediate[Rescan immediately]
    Low --> Delayed[Rescan in background]
```

## Special Cases

### Disabled to Active Server

```rust
old.enabled = false
new.enabled = true
// Result: modified = true, rescan triggered
```

Action: Complete rescan like for a new server.

### Active to Disabled Server

```rust
old.enabled = true
new.enabled = false
// Result: modified = true, but rescan skipped
```

Action: No rescan, server is simply ignored.

### Name Change Only

```rust
old: name = "server1"
new: name = "server1_renamed"
// Result: removed = ["server1"], added = ["server1_renamed"]
```

Action: Treated as removal + addition. server1 cache is lost.

### Simultaneous Multiple Modifications

If multiple fields change simultaneously, only one rescan is triggered:

```rust
// Multiple changes
old.loader != new.loader
old.minecraft_version != new.minecraft_version
old.enable_mods != new.enable_mods

// Result: single rescan, not three
```

## Cache Integration

### ServerPathCache Update

```mermaid
sequenceDiagram
    participant Detector
    participant Config as Config Store
    participant SPC as ServerPathCache

    Detector->>Config: Config updated
    Detector->>SPC: rebuild_server_cache()

    SPC->>SPC: Clear old mappings
    SPC->>Config: Read all servers
    SPC->>SPC: Build new path mappings
    SPC-->>Detector: Cache rebuilt
```

### Version Cache Invalidation

```mermaid
graph TD
    Modified[Modified servers detected] --> Loop[For each modified server]

    Loop --> Invalidate[Invalidate version cache]
    Loop --> Rescan[Trigger rescan]

    Rescan --> Scan[Scanner scans files]
    Scan --> NewVersion[New VersionBuilder]
    NewVersion --> UpdateCache[Update cache]
```

## Logging and Observability

### Generated Logs

```rust
// New server
tracing::info!("New server detected: {}", server_name);

// Modified server
tracing::info!("Server config changed, rescanning: {}", server_name);

// Removed server
// No log (silent removal)
```

### Collected Metrics

- Number of servers added per reload
- Number of servers modified per reload
- Number of servers removed per reload
- Change detection time
- False positive rate (change detected but identical)
