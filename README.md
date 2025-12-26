# LightyUpdater

High-performance Minecraft distribution server built with Rust and Axum. Serves game files (client, libraries, mods, natives, assets) via REST API with fast file resolution, RAM caching, hot-reload, S3/Cloudflare integration, and automatic scanning.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

![LightyUpdater Banner](docs/img/banner.png)

## Features

### Core Features
- **Fast File Resolution** - HashMap-based lookup providing instant file path resolution
- **Zero-Copy Serving** - Files cached in RAM using `Bytes` (Arc-based sharing)
- **Hot-Reload** - Configuration and file changes detected automatically with debouncing
- **Smart Caching** - Configurable LRU eviction with streaming support for large files
- **Auto-Migration** - Configuration automatically updated with new fields
- **Parallel Scanning** - All server components scanned concurrently for maximum speed

### Storage & CDN
- **S3 & R2 Storage** - Compatible with AWS S3 and Cloudflare R2 for remote file storage
- **Cloudflare Integration** - Automatic cache purging with retry mechanism
- **Local Storage** - Default local filesystem backend with HTTP serving

### Performance Optimizations
- **Optimized Lookups** - HashMap-based server and file comparisons
- **Atomic Operations** - Lock-free operations where possible (Relaxed ordering for flags)
- **Efficient Path Matching** - Sorted path cache for fast server identification
- **Incremental Updates** - Only changed files processed during rescans

### Production Ready
- **Asynchronous Runtime** - Non-blocking I/O with concurrent request handling
- **Graceful Shutdown** - Coordinated task termination and resource cleanup
- **Configurable Compression** - Optional gzip/brotli response compression
- **Modular Architecture** - 11 specialized crates with clear separation of concerns

---

## Configuration

### Complete config.toml

```toml
[server]
# Network
host = "0.0.0.0"
port = 8080
base_url = "http://localhost:8080"
base_path = "updater"

# Performance
tcp_nodelay = true
timeout_secs = 60
max_concurrent_requests = 1000
max_body_size_mb = 100
streaming_threshold_mb = 100
enable_compression = true

# CORS
allowed_origins = ["*"]

[cache]
# Core settings
enabled = true
auto_scan = true
rescan_interval = 0  # 0 = file watcher mode, >0 = polling interval in seconds
max_memory_cache_gb = 0

# Performance
checksum_buffer_size = 8192
config_reload_channel_size = 10

# Batch processing (concurrent file scanning)
[cache.batch]
client = 100
libraries = 100
mods = 100
natives = 100
assets = 100

# Hot-reload configuration
[hot-reload.config]
enabled = true
debounce_ms = 300

[hot-reload.files]
enabled = true
debounce_ms = 300

# Storage backend configuration
[storage]
backend = "local"  # "local" or "s3"

# S3 configuration (if backend = "s3")
[storage.s3]
endpoint = "https://<account-id>.r2.cloudflarestorage.com"
region = "auto"
bucket = "my-bucket"
access_key = "your-access-key"
secret_key = "your-secret-key"
public_url = "https://pub-<hash>.r2.dev"

# CDN cache purging for storage files (optional)
[cdn]
enabled = false
provider = "cloudflare"  # "cloudflare" or "cloudfront"
zone_id = "your-zone-id"
api_token = "your-api-token"

# Cloudflare cache purging for API JSON (optional)
[cloudflare]
enabled = false
zone_id = "your-zone-id"
api_token = "your-api-token"
base_url = "https://api.example.com"

# You can duplicate this [[servers]] section to add multiple servers
[[servers]]
name = "survival"
enabled = true
loader = "fabric"
loader_version = "0.15.7"
minecraft_version = "1.21"
main_class = "net.fabricmc.loader.impl.launch.knot.KnotClient"
java_version = 21
enable_client = true
enable_libraries = true
enable_mods = true
enable_natives = true
enable_assets = true
game_args = ["--width", "1920"]
jvm_args = ["-Xmx4G"]
```

---

## API Endpoints

### `GET /`

List all available servers.

**Response:**
```json
{
  "servers": [
    {
      "name": "survival",
      "loader": "fabric",
      "minecraft_version": "1.21",
      "url": "http://localhost:8080/survival.json"
    }
  ]
}
```

### `GET /{server}.json`

Retrieve server metadata including file URLs and checksums.

**Response:**
```json
{
  "main_class": {
    "main_class": "net.minecraft.client.main.Main"
  },
  "java_version": {
    "major_version": 21
  },
  "client": {
    "url": "http://localhost:8080/survival/client.jar",
    "path": "client.jar",
    "sha1": "abc123...",
    "size": 5527767
  },
  "libraries": [
    {
      "name": "com.mojang:library:1.0",
      "url": "http://localhost:8080/survival/com/mojang/library/1.0/library-1.0.jar",
      "sha1": "def456...",
      "size": 123456
    }
  ],
  "mods": [
    {
      "name": "OptiFine.jar",
      "url": "http://localhost:8080/survival/OptiFine.jar",
      "sha1": "ghi789...",
      "size": 987654
    }
  ]
}
```

### `GET /{server}/{file}`

Download file (zero-copy from RAM or streamed from disk).

**Examples:**
- `/survival/client.jar` → `updater/survival/client/client.jar`
- `/survival/OptiFine.jar` → `updater/survival/mods/OptiFine.jar`
- `/survival/com/mojang/lib.jar` → `updater/survival/libraries/com/mojang/lib.jar`

---
## Architecture

### Crate Dependencies

```
Foundation Layer:
  - models (domain models)
  - events (event bus)
  - utils (checksum, path utilities)

Core Services:
  - filesystem → utils
  - config → filesystem, events

Business Logic:
  - scanner → models, config, utils
  - cache → models, config, events, scanner
  - watcher → config, cache, filesystem

Interface Layer:
  - api → cache, models, config, filesystem

Application:
  - server (binary) → api, cache, config, events, watcher, filesystem

Facade:
  - lighty → all crates (unified re-exports)
```

### Request Flow

1. HTTP request arrives at Axum router
2. Request routed to appropriate handler (api crate)
3. Handler validates path and resolves file using HashMap lookup
4. Cache checked for file in RAM (Moka LRU)
5. If cache miss, file loaded from disk
6. Response streamed back to client (zero-copy if in cache)

### Background Tasks

- **Config Watcher**: Monitors config.toml for changes, triggers hot-reload with debouncing
- **File Watcher**: Monitors server directories, triggers rescan on changes (when `rescan_interval = 0`)
- **Auto-Rescan**: Periodic rescan at configurable intervals (when `rescan_interval > 0`)
- **Cache Eviction**: LRU eviction when memory limit reached
- **Cloud Sync**: Automatic S3/R2 upload/delete on file changes (if enabled)

---

## Performance Characteristics

### Core Performance
- **Fast File Resolution**: HashMap-based URL to path mapping
- **Zero-Copy Serving**: Arc-based Bytes sharing, no data duplication
- **Parallel Scanning**: All components (client, libraries, mods, natives, assets) scanned concurrently
- **Async I/O**: All I/O operations use Tokio async runtime
- **Lock-Free Operations**: DashMap for concurrent access, atomic flags where possible

### Optimizations
- **Efficient Comparisons**: HashMap-based lookups for server and file comparisons
- **Sorted Path Cache**: Server paths sorted by length for accurate prefix matching
- **Incremental Updates**: Only changed files updated in cache
- **Memory Efficient**: Configurable cache size with LRU eviction
- **Relaxed Atomics**: Optimized memory ordering for flag checks
- **Cloudflare Resilience**: 3 retry attempts with exponential backoff, 10s timeout

---

## Roadmap

- [x] Fast file resolution with HashMap
- [x] Zero-copy serving with Bytes
- [x] Configurable compression
- [x] Auto-migration system
- [x] File & config hot-reload
- [x] Modular multicrate architecture
- [x] S3 & R2 storage backend
- [ ] WebSocket progress streaming
- [ ] Docker image

---

## License

MIT License - see [LICENSE](LICENSE) file

---

**High-performance Minecraft distribution server**
