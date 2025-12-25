# Flux de traitement

## Flux 1: Scan complet d'un serveur

```mermaid
sequenceDiagram
    participant Cache as CacheManager
    participant SS as ServerScanner
    participant Storage
    participant Config

    Cache->>SS: scan_server(config, storage, base_path)
    SS->>SS: validate_server_path()
    alt Server not found
        SS-->>Cache: Error: ServerFolderNotFound
    end

    SS->>Config: Read server config
    Config-->>SS: ServerConfig

    par Scan Client
        SS->>SS: scan_client()
    and Scan Libraries
        SS->>SS: scan_libraries()
    and Scan Mods
        SS->>SS: scan_mods()
    and Scan Natives
        SS->>SS: scan_natives()
    and Scan Assets
        SS->>SS: scan_assets()
    end

    SS->>SS: Build VersionBuilder
    SS->>SS: build_url_map()
    SS-->>Cache: VersionBuilder
```

## Flux 2: JarScanner parallelise

```mermaid
sequenceDiagram
    participant LS as LibraryScanner
    participant JS as JarScanner
    participant WD as WalkDir
    participant Sem as Semaphore
    participant Tasks as Parallel Tasks
    participant Hash as SHA1 Computer

    LS->>JS: scan(mapper, buffer_size)
    JS->>WD: Collect all .jar paths
    WD-->>JS: Vec<PathBuf>

    JS->>Sem: Create(batch_size=100)

    loop For each file
        JS->>Tasks: Spawn task
        Tasks->>Sem: Acquire permit
        alt Permit available
            Sem-->>Tasks: Grant permit
            Tasks->>Hash: compute_sha1_with_size()
            Hash-->>Tasks: (SHA1, size)
            Tasks->>Tasks: Build Library
            Tasks->>Sem: Release permit
        else All permits used
            Tasks->>Tasks: Wait for permit
        end
    end

    Tasks-->>JS: Vec<Result<Library>>
    JS->>JS: Filter errors
    JS-->>LS: Vec<Library>
```

## Flux 3: Calcul de hash asynchrone

```mermaid
graph TD
    Start[File path] --> Open[tokio::fs::File::open]
    Open --> CreateHasher[Create SHA1 Hasher]
    CreateHasher --> CreateBuffer[Allocate buffer buffer_size]

    CreateBuffer --> Read[Read chunk async]
    Read --> CheckData{Data read?}

    CheckData -->|Yes| Update[Update hasher]
    Update --> AddSize[Increment total size]
    AddSize --> Read

    CheckData -->|No EOF| Finalize[Finalize hasher]
    Finalize --> ToHex[Convert to hex string]
    ToHex --> Return[Return SHA1 hex, size]
```

## Flux 4: Scan des assets recursif

```mermaid
sequenceDiagram
    participant AS as AssetScanner
    participant WD as WalkDir
    participant Sem as Semaphore(100)
    participant Stream as futures::stream
    participant Hash

    AS->>WD: Traverse assets/ recursively
    WD-->>AS: Vec<PathBuf> (all files)

    AS->>Sem: Create semaphore
    AS->>Stream: stream::iter(paths)

    loop For each asset in parallel
        Stream->>Sem: Acquire permit
        Sem-->>Stream: Permit granted
        Stream->>Hash: Compute SHA1
        Hash-->>Stream: SHA1 + size
        Stream->>Stream: Build Asset
        Stream->>Sem: Release permit
    end

    Stream->>Stream: buffer_unordered(100)
    Stream->>Stream: Collect results
    Stream-->>AS: Vec<Asset>
```

## Flux 5: Scan des natives multi-OS

```mermaid
graph TD
    Start[scan_natives] --> CheckDir{natives/ exists?}
    CheckDir -->|No| ReturnEmpty[Return Vec]

    CheckDir -->|Yes| InitVec[Vec all_natives]

    InitVec --> Windows[Scan natives/windows/]
    InitVec --> Linux[Scan natives/linux/]
    InitVec --> MacOS[Scan natives/macos/]

    Windows --> WinParallel[scan_files_parallel<br/>with os=windows]
    Linux --> LinuxParallel[scan_files_parallel<br/>with os=linux]
    MacOS --> MacParallel[scan_files_parallel<br/>with os=macos]

    WinParallel --> WinMap[Map: natives:windows:file]
    LinuxParallel --> LinuxMap[Map: natives:linux:file]
    MacParallel --> MacMap[Map: natives:macos:file]

    WinMap --> Extend1[Extend all_natives]
    LinuxMap --> Extend2[Extend all_natives]
    MacMap --> Extend3[Extend all_natives]

    Extend1 --> Return[Return all_natives]
    Extend2 --> Return
    Extend3 --> Return
    ReturnEmpty --> End[End]
    Return --> End
```

## Flux 6: Construction de VersionBuilder

```mermaid
sequenceDiagram
    participant SS as ServerScanner
    participant VB as VersionBuilder
    participant Config

    SS->>Config: Get server config
    Config-->>SS: ServerConfig

    SS->>VB: Create with main_class
    SS->>VB: Set java_version
    SS->>VB: Set arguments (game, jvm)

    alt enable_client
        SS->>SS: scan_client()
        SS->>VB: Set client
    end

    alt enable_libraries
        SS->>SS: scan_libraries()
        SS->>VB: Set libraries
    end

    alt enable_mods
        SS->>SS: scan_mods()
        SS->>VB: Set mods
    end

    alt enable_natives
        SS->>SS: scan_natives()
        SS->>VB: Set natives
    end

    alt enable_assets
        SS->>SS: scan_assets()
        SS->>VB: Set assets
    end

    SS->>VB: build_url_map()
    VB->>VB: Create url_to_path_map
    VB-->>SS: Complete VersionBuilder
```

## Metriques de performance

### Complexite temporelle

**Sequential**:
- Scan de N fichiers: O(N * T) ou T = temps de hash

**Parallel avec batch_size B**:
- Scan de N fichiers: O(N/B * T) theorique
- En pratique: Limite par I/O disque

### Exemples de temps de scan

**Configuration**:
- batch_size = 100
- buffer_size = 8192

**Resultats typiques**:
- Client (1 file): < 100ms
- Libraries (200 files): 2-5s
- Mods (50 files): 1-2s
- Natives (30 files): 500ms-1s
- Assets (5000 files): 10-20s

**Total**: 15-30s pour un serveur complet
