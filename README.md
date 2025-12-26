# LightyUpdater

High-performance Minecraft distribution server built with Rust and Axum. Serves game files (client, libraries, mods, natives, assets) via REST API with O(1) file resolution, RAM caching, hot-reload, and automatic scanning.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

![LightyUpdater Banner](docs/img/banner.png)
## Features

- **O(1) File Resolution** - HashMap-based lookup providing instant file path resolution
- **Zero-Copy Serving** - Files cached in RAM using `Bytes` (Arc-based sharing)
- **Hot-Reload** - Configuration and file changes detected automatically
- **Smart Caching** - Configurable LRU eviction with streaming support for large files
- **Auto-Migration** - Configuration automatically updated with new fields
- **Production-Ready** - Async/await, graceful shutdown, configurable compression
- **Modular Architecture** - Clean separation of concerns with 11 specialized crates

---

## Project Structure

```
LightyUpdater/
├── src/                    # Server binary source code
│   ├── main.rs            # Application entry point
│   └── bootstrap/         # Server initialization modules
│       ├── config.rs      # Configuration loading
│       ├── logging.rs     # Tracing setup
│       ├── router.rs      # HTTP routing
│       └── server.rs      # Server lifecycle
│
├── crates/                 # Library crates
│   ├── lib.rs             # Unified facade (lighty crate)
│   ├── Cargo.toml         # Facade manifest
│   │
│   ├── models/            # Domain models
│   │   └── src/
│   │       └── lib.rs     # VersionBuilder, Library, Mod, etc.
│   │
│   ├── events/            # Event bus system
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── bus.rs     # EventBus implementation
│   │       └── models.rs  # AppEvent enum
│   │
│   ├── utils/             # Shared utilities
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── checksum.rs # SHA1 calculation
│   │       └── path.rs     # Path normalization
│   │
│   ├── filesystem/        # File system operations
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models.rs
│   │       └── operations.rs
│   │
│   ├── config/            # Configuration management
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models.rs     # Config structs
│   │       ├── defaults.rs   # Default values
│   │       ├── loader.rs     # TOML loading
│   │       └── migration.rs  # Auto-migration
│   │
│   ├── scanner/           # File scanning
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── server.rs     # ServerScanner
│   │       ├── client.rs     # Client JAR scanning
│   │       ├── libraries.rs  # Maven library scanning
│   │       ├── mods.rs       # Mod scanning
│   │       ├── natives.rs    # Native library scanning
│   │       ├── assets.rs     # Asset scanning
│   │       └── utils/        # JAR utilities
│   │
│   ├── cache/             # Caching layer
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── manager.rs           # CacheManager
│   │       ├── file_cache.rs        # RAM cache (Moka)
│   │       ├── file_manager.rs      # File cache operations
│   │       ├── change_detector.rs   # Diff detection
│   │       └── rescan_orchestrator.rs # Auto-rescan
│   │
│   ├── watcher/           # File watching
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs     # ConfigWatcher
│   │       └── models.rs
│   │
│   └── api/               # HTTP handlers
│       └── src/
│           ├── lib.rs
│           ├── handlers/
│           │   ├── files/       # File serving
│           │   ├── servers.rs   # Server metadata
│           │   ├── rescan.rs    # Force rescan
│           │   └── state.rs     # AppState
│           └── models.rs
│
├── Cargo.toml              # Workspace + server binary manifest
├── Cargo.lock              # Dependency lock file
├── config.toml             # Server configuration
├── LICENSE                 # MIT License
└── README.md
```

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
rescan_interval = 30
max_memory_cache_gb = 0

# Performance
checksum_buffer_size = 8192

# Batch processing
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

### `GET /rescan/{server}`

Force immediate rescan of server files.

**Response:**
```json
{
  "status": "success",
  "message": "Server survival rescanned"
}
```
--
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
3. Handler validates path and resolves file using HashMap (O(1))
4. Cache checked for file in RAM (Moka LRU)
5. If cache miss, file loaded from disk
6. Response streamed back to client (zero-copy if in cache)

### Background Tasks

- **Config Watcher**: Monitors config.toml for changes, triggers reload
- **File Watcher**: Monitors server directories, triggers rescan on changes
- **Auto-Rescan**: Periodic rescan at configurable intervals
- **Cache Eviction**: LRU eviction when memory limit reached

---

## Performance Characteristics

- **O(1) File Resolution**: HashMap-based URL to path mapping
- **Zero-Copy Serving**: Arc-based Bytes sharing, no data duplication
- **Parallel Scanning**: Rayon-based parallel file system traversal
- **Async I/O**: All I/O operations use Tokio async runtime
- **Lock-Free Cache**: DashMap for concurrent access without locks
- **Memory Efficient**: Configurable cache size with LRU eviction

---

## Roadmap

- [x] O(1) file resolution with HashMap
- [x] Zero-copy serving with Bytes
- [x] Configurable compression
- [x] Auto-migration system
- [x] File & config hot-reload
- [x] Modular multicrate architecture
- [ ] Cloudflare R2 integration (CDN + object storage)
- [ ] WebSocket progress streaming 
- [ ] Docker image

---

## License

MIT License - see [LICENSE](LICENSE) file

---

## Acknowledgments

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tokio](https://tokio.rs) - Async runtime
- [Moka](https://github.com/moka-rs/moka) - Caching library
- [Notify](https://github.com/notify-rs/notify) - File watcher

---

**High-performance Minecraft distribution server**
