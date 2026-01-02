# Scan System Architecture

## Overview

The scan system is designed as a modular architecture with specialized components for each file type. The focus is on parallelization and performance.

## Architecture Diagram

```mermaid
graph TB
    SS[ServerScanner] --> CS[ClientScanner]
    SS --> LS[LibraryScanner]
    SS --> MS[ModScanner]
    SS --> NS[NativeScanner]
    SS --> AS[AssetScanner]

    LS --> JS[JarScanner]
    MS --> JS
    NS --> FP[scan_files_parallel]
    AS --> FP

    JS --> Hash[SHA1 Computer]
    FP --> Hash
    JS --> Path[Path Normalizer]
    FP --> Path
    LS --> Maven[Maven Converter]

    CS --> Storage[Storage Backend]
    LS --> Storage
    MS --> Storage
    NS --> Storage
    AS --> Storage

    SS --> Models[VersionBuilder]
```

## Main Components

### ServerScanner

Main entry point that orchestrates the complete scan of a server.

**Responsibilities**:
- Server structure validation
- Specialized scanner coordination
- VersionBuilder construction
- URL map generation

**Methods**:
- `scan_server`: Scan with detailed logging
- `scan_server_silent`: Scan without logging (for frequent rescans)
- `validate_server_path`: Verify server existence
- `build_version_metadata`: Build metadata with parallel component scanning

**Parallelization**:
The `build_version_metadata` method uses `tokio::join!` to scan all components (client, libraries, mods, natives, assets) concurrently. This allows multiple I/O operations to execute simultaneously, significantly reducing total scan time.

### JarScanner

Reusable generic scanner for JAR files with parallelization.

**Features**:
- Generic over return type `T`
- Customizable mapping function
- Concurrency control via Semaphore
- Uses futures::stream for parallelization

**Structure**:
```rust
pub struct JarScanner {
    pub base_dir: PathBuf,
    pub server: String,
    pub storage: Arc<dyn StorageBackend>,
    pub batch_size: usize,
}
```

**Algorithm**:
1. Synchronous collection of JAR file paths
2. Semaphore creation for concurrency control
3. File streaming with `buffer_unordered`
4. Parallel hash computation
5. Result mapping via provided function
6. Error filtering and collection

### scan_files_parallel

Utility function for parallel file scanning with customizable filter.

**Generic parameters**:
- `T`: Return type
- `Filter`: File filtering function
- `Mapper`: FileInfo → T transformation function

**Usage**: Natives and other non-JAR files requiring parallel processing.

## Specialized scanners

### ClientScanner

Scans the single client JAR file.

**Algorithm**:
```mermaid
graph TD
    Start[Start scan_client] --> CheckDir{client/ exists?}
    CheckDir -->|No| ReturnNone[Return None]
    CheckDir -->|Yes| ReadDir[Read directory]

    ReadDir --> FindJar{Find .jar file?}
    FindJar -->|No| ReturnNone
    FindJar -->|Yes| GetName[Extract filename]

    GetName --> ComputeHash[Compute SHA1 & size]
    ComputeHash --> GenURL[Generate URL from storage]
    GenURL --> BuildClient[Build Client struct]
    BuildClient --> ReturnSome[Return Some Client]

    ReturnNone --> End[End]
    ReturnSome --> End
```

**Characteristics**:
- No parallelization (single file)
- Takes the first .jar found
- Returns Option<Client>

### LibraryScanner

Scans libraries with Maven notation conversion.

**Process**:
```mermaid
sequenceDiagram
    participant LS as LibraryScanner
    participant JS as JarScanner
    participant Maven as path_to_maven_name
    participant Storage

    LS->>JS: new(libraries_dir, server, storage, batch_size)
    LS->>JS: scan(mapper, buffer_size)

    JS->>JS: Collect all .jar paths
    loop For each jar in parallel
        JS->>JS: Compute SHA1
        JS->>Maven: Convert path to Maven name
        Maven-->>JS: "com.example:library:1.0.0"
        JS->>Storage: get_url(remote_key)
        Storage-->>JS: URL
        JS->>JS: Build Library struct
    end

    JS-->>LS: Vec<Library>
```

**Maven conversion**:
- Input: `com/example/library/1.0.0/library-1.0.0.jar`
- Output: `com.example:library:1.0.0`

### ModScanner

Scans mods with simple structure.

**Features**:
- Uses JarScanner as base
- No name conversion (keeps filename)
- Flat structure (no subdirectories)

**Mapping**:
```rust
Mod {
    name: info.file_name,  // "optifine.jar"
    url: Some(info.url),
    path: Some(info.url_path),
    sha1: Some(info.sha1),
    size: Some(info.size),
}
```

### NativeScanner

Scans natives with multi-OS organization.

**Organization**:
```
natives/
├── windows/
│   └── lwjgl-natives-windows.jar
├── linux/
│   └── lwjgl-natives-linux.jar
└── macos/
    └── lwjgl-natives-macos.jar
```

**Algorithm**:
```mermaid
graph TD
    Start[Start scan_natives] --> CheckDir{natives/ exists?}
    CheckDir -->|No| ReturnEmpty[Return empty Vec]
    CheckDir -->|Yes| InitVec[Initialize all_natives Vec]

    InitVec --> IterOS[Iterate OS types]
    IterOS --> Windows[Scan windows/]
    IterOS --> Linux[Scan linux/]
    IterOS --> MacOS[Scan macos/]

    Windows --> ScanParallel1[scan_files_parallel]
    Linux --> ScanParallel2[scan_files_parallel]
    MacOS --> ScanParallel3[scan_files_parallel]

    ScanParallel1 --> Mapper1[Map with OS tag]
    ScanParallel2 --> Mapper2[Map with OS tag]
    ScanParallel3 --> Mapper3[Map with OS tag]

    Mapper1 --> Extend1[Extend all_natives]
    Mapper2 --> Extend2[Extend all_natives]
    Mapper3 --> Extend3[Extend all_natives]

    Extend1 --> Return[Return all_natives]
    Extend2 --> Return
    Extend3 --> Return
    ReturnEmpty --> End[End]
    Return --> End
```

**Name format**:
```rust
name: format!("natives:{}:{}", os, file_name)
// Example: "natives:windows:lwjgl-natives-windows.jar"
```

### AssetScanner

Recursive scan of all assets.

**Characteristics**:
- Recursive scan of entire tree
- Can generate thousands of files
- Uses walkdir to traverse directories
- Massive parallelization with semaphore

**Typical structure**:
```
assets/
├── minecraft/
│   ├── textures/
│   │   ├── block/
│   │   │   ├── stone.png
│   │   │   └── dirt.png
│   │   └── item/
│   │       └── diamond.png
│   └── sounds/
│       └── ambient/
│           └── cave.ogg
└── custom/
    └── logo.png
```

## Parallelization

### Concurrency architecture

```mermaid
graph TB
    Collect[Collect file paths<br/>sync] --> Sem[Semaphore<br/>max = batch_size]

    Sem --> T1[Task 1: Hash file 1]
    Sem --> T2[Task 2: Hash file 2]
    Sem --> T3[Task 3: Hash file 3]
    Sem --> Tn[Task N: Hash file N]

    T1 --> Stream[futures::stream]
    T2 --> Stream
    T3 --> Stream
    Tn --> Stream

    Stream --> Buffer[buffer_unordered]
    Buffer --> Collect2[Collect results]
```

### Concurrency control

**Semaphore**:
- Limits the number of concurrent tasks
- Prevents CPU/memory overload
- Configured by batch_size

**buffer_unordered**:
- Executes up to N futures simultaneously
- Collects results in completion order
- Optimizes parallelization

### Flow example

```mermaid
sequenceDiagram
    participant Main
    participant Sem as Semaphore(100)
    participant Task1
    participant Task2
    participant Task100
    participant Task101

    Main->>Sem: Create with capacity 100

    par Task 1-100 start immediately
        Main->>Task1: Spawn
        Task1->>Sem: Acquire permit
        Sem-->>Task1: Permit 1
        Main->>Task2: Spawn
        Task2->>Sem: Acquire permit
        Sem-->>Task2: Permit 2
        Main->>Task100: Spawn
        Task100->>Sem: Acquire permit
        Sem-->>Task100: Permit 100
    end

    Main->>Task101: Spawn
    Task101->>Sem: Acquire permit (wait)

    Task1->>Task1: Compute hash
    Task1->>Sem: Release permit
    Sem-->>Task101: Permit 1 (reused)

    Task101->>Task101: Compute hash
```

## Asynchronous hash computation

### Problem

SHA1 computation is CPU-intensive and could block the async runtime.

### Solution

Using tokio for asynchronous computation with buffer:

```mermaid
graph TD
    Start[File path] --> OpenFile[tokio::fs::File::open]
    OpenFile --> CreateBuf[Create buffer buffer_size]
    CreateBuf --> LoopRead{Read chunk}

    LoopRead -->|Has data| UpdateHash[Update SHA1 hasher]
    UpdateHash --> LoopRead
    LoopRead -->|EOF| Finalize[Finalize hash]

    Finalize --> Return[Return SHA1 hex + size]
```

**Advantages**:
- No runtime blocking
- Configurable buffer to optimize I/O
- Size computation at the same time

**Configuration**:
```toml
[cache]
checksum_buffer_size = 8192  # 8KB buffer
```

## Integration with Storage

### URL generation

```mermaid
sequenceDiagram
    participant Scanner
    participant Storage as StorageBackend
    participant Config

    Scanner->>Scanner: Compute remote_key
    Note over Scanner: format!("{}/{}", server, url_path)

    Scanner->>Storage: get_url(remote_key)

    alt Local storage
        Storage->>Config: Get base_url
        Config-->>Storage: "http://localhost:8080"
        Storage->>Storage: Concatenate base_url + key
        Storage-->>Scanner: "http://localhost:8080/files/survival/mods/optifine.jar"
    else S3 storage
        Storage->>Config: Get public_url
        Config-->>Storage: "https://cdn.example.com"
        Storage->>Storage: Concatenate public_url + bucket_prefix + key
        Storage-->>Scanner: "https://cdn.example.com/servers/survival/mods/optifine.jar"
    end
```

### Remote key format

```
{server_name}/{category}/{relative_path}

Examples:
- survival/client.jar
- survival/libraries/com/example/lib/1.0.0/lib-1.0.0.jar
- survival/mods/optifine.jar
- survival/natives/windows/lwjgl-natives-windows.jar
- survival/assets/minecraft/textures/block/stone.png
```

## Error handling

### Filtering strategy

Individual errors do not block the complete scan:

```rust
let results: Vec<Result<T>> = stream::iter(paths)
    .map(|path| async { /* scan */ })
    .buffer_unordered(batch_size)
    .collect()
    .await;

// Filter errors
Ok(results.into_iter().filter_map(|r| r.ok()).collect())
```

**Impact**: A corrupted file does not prevent scanning other files.

### Critical error propagation

Some errors are critical:
- Server folder does not exist
- Insufficient permissions
- Storage backend inaccessible

These errors are propagated via `Result<VersionBuilder>`.

## Optimizations

### Eager path collection

```rust
// Good: Sync collection then async processing
let paths: Vec<PathBuf> = WalkDir::new(&dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| is_jar_file(e.path()))
    .map(|e| e.path().to_path_buf())
    .collect();

let results = stream::iter(paths)
    .map(|path| async { /* process */ })
    .buffer_unordered(batch_size)
    .collect()
    .await;
```

**Advantage**: Separates sync I/O operations (walkdir) from async operations (hash).

### Arc reuse

```rust
let storage = Arc::clone(&self.storage);
let mapper = Arc::new(mapper);

// Lightweight clone for each task
let storage = Arc::clone(&storage);
let mapper = Arc::clone(&mapper);
```

**Advantage**: No copying of large structures.

### Path normalization

Converting Windows paths to Unix format:
```rust
// Windows: libraries\com\example\lib.jar
// Unix:    libraries/com/example/lib.jar
normalize_path(path)  // Always "/"
```

**Importance**: Consistent URLs regardless of OS.
