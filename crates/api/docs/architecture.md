# HTTP API Architecture

## Overview

The API is built with Axum and follows a layered architecture with clear separation of concerns.

## Architecture Diagram

```mermaid
graph TB
    Router[Axum Router] --> Middleware[Middleware Stack]
    Middleware --> ServersH[Servers Handler]
    Middleware --> FilesH[Files Handler]

    ServersH --> AppState[AppState]
    FilesH --> Parser[Path Parser]

    Parser --> Validator[Security Validator]
    Validator --> Resolver[URL Resolver]
    Resolver --> CacheServe[RAM Cache Server]
    CacheServe --> DiskServe[Disk Server]
    DiskServe --> FS[File System]

    AppState --> CacheRef[CacheManager Ref]
    AppState --> Config[Base URL/Path]
    CacheRef --> Cache[Cache Manager]
```

## Main Components

### AppState

Shared state structure across all handlers.

**Data:**
- `cache`: Arc<CacheManager> - Reference to cache manager
- `base_url`: Arc<String> - Base URL for generating URLs
- `base_path`: Arc<String> - Root path for files
- `streaming_threshold_bytes`: u64 - Threshold for streaming vs memory loading

### Handlers

**list_servers:**
- Route: GET /servers
- Returns list of all servers with metadata
- Builds ServerInfo for each server

**get_server_metadata:**
- Route: GET /{server}.json
- Returns complete VersionBuilder
- Checks if server is active

**serve_file:**
- Route: GET /{server}/{path}
- Complete file serving pipeline
- RAM cache then disk fallback

### File Serving Pipeline

```mermaid
graph TD
    Request[HTTP Request] --> Parse[Parser]
    Parse --> Validate[Validator]
    Validate --> Resolve[Resolver]
    Resolve --> Cache{In RAM cache?}

    Cache -->|Yes| ServeRAM[Serve from RAM]
    Cache -->|No| CheckDisk[Check file on disk]

    CheckDisk --> Size{File size?}
    Size -->|< threshold| LoadMem[Load to memory]
    Size -->|>= threshold| Stream[Stream from disk]

    ServeRAM --> Response[HTTP Response]
    LoadMem --> Response
    Stream --> Response
```

## Data Models

### ServerListResponse

```rust
{
  "servers": [
    {
      "name": "server1",
      "loader": "forge",
      "minecraft_version": "1.20.1",
      "url": "http://localhost:8080/server1.json",
      "last_update": "2024-01-15T10:30:00Z"
    }
  ]
}
```

### ErrorResponse

```rust
{
  "error": {
    "code": "SERVER_NOT_FOUND",
    "message": "Server 'invalid' not found",
    "available_servers": ["server1", "server2"]
  }
}
```

## Security

### Path Validation

```mermaid
graph TD
    Path[User path input] --> Check1{Contains '..'?}
    Check1 -->|Yes| Reject[Return 400 Bad Request]
    Check1 -->|No| Check2{Contains null bytes?}
    Check2 -->|Yes| Reject
    Check2 -->|No| Check3{Absolute path?}
    Check3 -->|Yes| Reject
    Check3 -->|No| Accept[Accept path]
```

### Path Traversal Protection

The validator checks:
- No `..` sequences (directory traversal)
- No absolute paths
- No null bytes
- Special characters blocked

## Performance

### RAM Cache

**Advantages:**
- Zero disk I/O
- Zero-copy with Bytes

**Limits:**
- Configured memory size
- Automatic LRU eviction
- Only small files (<= threshold)

### Streaming

**When:**
- Files > streaming_threshold_bytes
- Default: 10MB

**Advantages:**
- Constant memory O(buffer_size)
- Large file support
- Automatic backpressure

### Recommended Thresholds

| Use case | Threshold | Rationale |
|----------|-----------|-----------|
| Local dev | 5MB | Abundant memory |
| Production | 10MB | Balance perf/memory |
| Memory constrained | 1MB | Minimize RAM usage |
| High perf | 50MB | Maximize cache hits |
