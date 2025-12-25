# File Distribution System

## Complete Pipeline

```mermaid
flowchart TD
    Request[HTTP GET /{server}/{path}] --> Parse[Parse request path]
    Parse --> Validate[Validate security]

    Validate -->|Invalid| Error400[400 Invalid Path]
    Validate -->|Valid| GetVersion[Get VersionBuilder from cache]

    GetVersion -->|Not found| Error404[404 Server Not Found]
    GetVersion -->|Found| Resolve[Resolve URL to file path]

    Resolve -->|Not resolved| Error404b[404 Not Found]
    Resolve -->|Resolved| TryRAM[Try RAM cache]

    TryRAM -->|Hit| ServeRAM[Serve from RAM<br/>Zero-copy Bytes]
    TryRAM -->|Miss| BuildPath[Build full disk path]

    BuildPath --> CheckExists{File exists?}
    CheckExists -->|No| Error404c[404 Not Found]
    CheckExists -->|Yes| GetSize[Get file size]

    GetSize --> CompareThreshold{Size vs threshold?}

    CompareThreshold -->|< threshold| LoadMemory[Load to memory<br/>tokio::fs::read]
    CompareThreshold -->|>= threshold| StreamFile[Stream file<br/>ReaderStream]

    ServeRAM --> Response200[200 OK + file data]
    LoadMemory --> Response200
    StreamFile --> Response200
```

## Serving Strategies

### RAM Cache (small files)

**Advantages**:
- Minimal latency (< 1ms)
- Zero disk I/O
- Zero-copy with axum Bytes
- Maximum throughput

**Configuration**:
- LRU cache with memory limit
- Automatic eviction
- Pre-loading of frequent files

**Metrics**:
- Hit rate: 70-90% typical
- Latency p50: < 1ms
- Latency p99: < 5ms

### Load to Memory (medium files)

**When**: File < streaming_threshold but not in RAM cache

**Process**:
1. Complete asynchronous file read
2. Load into Vec<u8>
3. Serve with appropriate Content-Type

**Performance**:
- Latency: 5-50ms depending on size
- Memory: File size
- Throughput: Disk limited

### Streaming (large files)

**When**: File >= streaming_threshold

**Process**:
1. Open asynchronous file handle
2. Create ReaderStream
3. Convert to HTTP Body stream
4. Chunks sent progressively

**Advantages**:
- Constant memory (buffer size)
- Support files > RAM
- Automatic backpressure
- No timeout on large files

**Performance**:
- First byte latency: 10-100ms
- Throughput: Network/disk limited
- Memory: ~8KB buffer

## URL to Path Resolution

```mermaid
flowchart LR
    URL["URL requested<br/>http://domain/server/mods/mod.jar"] --> Extract[Extract path<br/>'server/mods/mod.jar']
    Extract --> FullURL[Rebuild full URL<br/>base_url + server + path]
    FullURL --> Lookup[HashMap.get O(1)]

    Lookup -->|Found| ActualPath["Actual path<br/>'mods/mod-1.0.jar'"]
    Lookup -->|Not found| None[None]
```

**Optimization**: Pre-built HashMap with all server URLs for O(1) lookups.

## Security

### Path Traversal Validation

```rust
fn validate_path_component(path: &str) -> Result<(), ApiError> {
    if path.contains("..") => Err(InvalidPath)
    if path.contains('\0') => Err(InvalidPath)
    if Path::new(path).is_absolute() => Err(InvalidPath)
    Ok(())
}
```

### Blocked Examples

- `../../../etc/passwd` (path traversal)
- `/etc/passwd` (absolute path)
- `file\0.jar` (null byte injection)
- `C:\Windows\system32\cmd.exe` (Windows absolute)

## Configuration

```toml
[server]
streaming_threshold_mb = 10

[cache.file]
max_size_mb = 512
max_file_size_mb = 5
```

**Recommendations**:
- `streaming_threshold_mb`: 5-20MB
- `max_size_mb`: 256-2048MB depending on RAM
- `max_file_size_mb`: <= streaming_threshold
