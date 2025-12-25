# Monitoring System Processing Flow

## Overview

This document details the execution flows for each operation of the configuration monitoring system.

## Starting Monitoring

```mermaid
sequenceDiagram
    participant App
    participant CW as ConfigWatcher
    participant FW as FileWatcher
    participant Chan as MPSC Channel
    participant Task as Tokio Task

    App->>CW: start_watching()
    CW->>Chan: Create channel(size)
    CW->>FW: recommended_watcher(callback)

    Note over FW: Setup OS file watcher

    CW->>FW: watch(config_path, NonRecursive)
    FW-->>CW: Ok()

    CW->>Task: spawn(watch_loop)
    Task-->>CW: JoinHandle

    CW-->>App: Ok(handle)

    loop Watch loop
        FW->>Chan: Send event
        Chan->>Task: Receive event
        Task->>Task: Process event
    end
```

## File Modification Detection

```mermaid
sequenceDiagram
    participant Editor
    participant OS as OS File System
    participant FW as FileWatcher
    participant CB as Callback
    participant Chan as Channel

    Editor->>OS: Save config.toml
    OS->>FW: inotify/FSEvents notification

    FW->>CB: Event {kind: Modify}

    alt Event is Modify or Create
        CB->>Chan: blocking_send(())
    else Other event type
        CB->>CB: Ignore event
    end
```

## Event Processing with Debouncing

```mermaid
sequenceDiagram
    participant Chan as Channel
    participant Task as Watch Task
    participant Timer as Tokio Timer
    participant Config

    Chan->>Task: Event received

    Task->>Config: Read debounce_ms
    Config-->>Task: 500ms

    Task->>Timer: sleep(500ms)
    Note over Timer: Debounce period

    alt New event during debounce
        Chan->>Task: Event received
        Note over Task: Event consumed but ignored
    end

    Timer-->>Task: Wake up

    Task->>Task: Check file exists

    alt File exists
        Task->>Config: from_file_with_events()
    else File deleted
        Task->>Task: Log warning + skip
    end
```

## Configuration Comparison

```mermaid
flowchart TD
    Start[New config loaded] --> PauseRescan[pause_rescan]
    PauseRescan --> ReadOld[Read old servers]

    ReadOld --> BuildSets[Build HashSets]
    BuildSets --> CompareNames[Compare server names]

    CompareNames --> CalcAdded[added = new - old]
    CompareNames --> CalcRemoved[removed = old - new]

    BuildSets --> CompareConfigs{Compare each server config}

    CompareConfigs -->|enabled changed| Modified[Add to modified list]
    CompareConfigs -->|loader changed| Modified
    CompareConfigs -->|version changed| Modified
    CompareConfigs -->|other fields changed| Modified
    CompareConfigs -->|no changes| Skip[Skip server]

    CalcAdded --> Actions
    Modified --> Actions
    CalcRemoved --> Actions

    Actions[Execute actions]
```

## Configuration Update

```mermaid
sequenceDiagram
    participant Task as Watch Task
    participant Config as Config Store
    participant CM as CacheManager

    Task->>CM: pause_rescan()
    Note over CM: Stop auto-rescan

    Task->>Config: Read lock - get old data
    Config-->>Task: old_servers, old_configs

    Task->>Task: Detect changes

    Task->>Config: Write lock - EXCLUSIVE
    Task->>Config: *config = new_config
    Config-->>Task: Updated

    Task->>CM: rebuild_server_cache()
    CM-->>Task: Cache rebuilt

    Task->>CM: resume_rescan()
    Note over CM: Resume auto-rescan

    Task->>Task: Log success
```

## Modified Servers Processing

```mermaid
sequenceDiagram
    participant Task as Watch Task
    participant Config as Config Store
    participant CM as CacheManager
    participant Scanner

    loop For each modified server
        Task->>Task: Log "Server config changed"

        Task->>Config: Drop write lock
        Note over Task: Avoid holding lock during I/O

        Task->>CM: force_rescan(server_name)
        CM->>Scanner: Scan server
        Scanner-->>CM: VersionBuilder
        CM->>CM: Update cache
        CM-->>Task: Ok()

        Task->>Config: Re-acquire write lock
    end
```

## Added Servers Processing

```mermaid
sequenceDiagram
    participant Task as Watch Task
    participant Config as Config Store
    participant FS as FileSystem
    participant CM as CacheManager

    loop For each added server
        Task->>Config: Get server config

        alt Server enabled
            Task->>Task: Log "New server detected"

            Task->>Config: Drop write lock

            Task->>FS: ensure_server_structure(base_path, name)

            alt Structure creation success
                FS->>FS: Create client/
                FS->>FS: Create libraries/
                FS->>FS: Create mods/
                FS->>FS: Create natives/
                FS->>FS: Create assets/
                FS-->>Task: Ok(path)

                Task->>CM: force_rescan(server_name)
                CM-->>Task: Ok()
            else Structure creation failed
                FS-->>Task: Err(error)
                Task->>Task: Log error
            end

            Task->>Config: Re-acquire write lock
        else Server disabled
            Task->>Task: Skip disabled server
        end
    end
```

## File Existence Verification

```mermaid
flowchart TD
    Event[File event received] --> Debounce[Wait debounce period]
    Debounce --> Check{File exists?}

    Check -->|Yes| Load[Load configuration]
    Check -->|No| Warn[Log warning]

    Warn --> Continue[Continue watching]
    Load --> Process[Process changes]
    Process --> Continue
```

## Error Handling During Reload

```mermaid
sequenceDiagram
    participant Task as Watch Task
    participant Config
    participant CM as CacheManager
    participant Log

    Task->>Config: from_file_with_events()

    alt Parse success
        Config-->>Task: Ok(new_config)
        Task->>CM: pause_rescan()
        Task->>Task: Apply changes
        Task->>CM: resume_rescan()
    else Parse error
        Config-->>Task: Err(ConfigError)
        Task->>Log: error!("Failed to reload config")
        Task->>Task: Keep old config
        Note over Task: System continues normally
    end
```

## Lock Release Pattern for I/O

```mermaid
sequenceDiagram
    participant Task
    participant Lock as Write Lock
    participant IO as I/O Operation

    Task->>Lock: Acquire write lock
    Note over Lock: Exclusive access

    Task->>Lock: Update config
    Task->>Lock: Update cache structures

    Task->>Lock: drop(write_lock)
    Note over Lock: Lock released

    Task->>IO: File system operations
    Task->>IO: Network operations
    Task->>IO: Cache rescan

    IO-->>Task: Complete

    alt Need more updates
        Task->>Lock: Re-acquire write lock
        Task->>Lock: Continue updates
    end
```

## Complete Reload Cycle

```mermaid
flowchart TD
    Start[File modified] --> Debounce[Debounce wait]
    Debounce --> Exists{File exists?}

    Exists -->|No| Skip[Skip event]
    Exists -->|Yes| Parse[Parse new config]

    Parse -->|Error| LogError[Log error + keep old]
    Parse -->|Success| Pause[Pause rescan]

    Pause --> Compare[Compare configs]
    Compare --> Update[Update config store]
    Update --> Rebuild[Rebuild server cache]
    Rebuild --> Resume[Resume rescan]

    Resume --> HasModified{Modified servers?}
    HasModified -->|Yes| RescanMod[Rescan modified]
    HasModified -->|No| HasAdded{Added servers?}

    RescanMod --> HasAdded

    HasAdded -->|Yes| CreateStruct[Create server structure]
    HasAdded -->|No| Done[Complete]

    CreateStruct --> RescanNew[Rescan new servers]
    RescanNew --> Done

    Skip --> Continue[Continue watching]
    LogError --> Continue
    Done --> Continue
```

## Performance Metrics

```mermaid
graph LR
    Metrics[Collected metrics]

    Metrics --> Time1[Debounce time]
    Metrics --> Time2[Parsing time]
    Metrics --> Time3[Rescan time]
    Metrics --> Time4[Total reload time]

    Metrics --> Count1[Number of reloads]
    Metrics --> Count2[Number of errors]
    Metrics --> Count3[Modified servers]
    Metrics --> Count4[Added servers]
```

Typical times:
- Debounce: 100-500ms (configurable)
- Config parsing: 1-10ms
- Cache rebuild: 5-20ms
- Rescan per server: 100ms-2s
- Total: 200ms-5s depending on number of servers
