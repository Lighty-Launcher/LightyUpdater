# Config Crate

Configuration management system with automatic migration and hot-reload for LightyUpdater.

## Table of Contents

- [Architecture](docs/architecture.md) - Configuration system architecture
- [Migration](docs/migration.md) - Automatic migration system
- [Hot-reload](docs/hot-reload.md) - Hot configuration reloading
- [Errors](docs/errors.md) - Error types documentation
- [Structures](docs/structures.md) - Configuration structures documentation

## Integration

This crate integrates with:
- `lighty_events`: To emit migration and error events
- `lighty_watcher`: For configuration hot-reload
- `toml`: For configuration parsing
- `toml_edit`: For migration without losing comments

## Configuration Format

### Minimal Example

```toml
[server]
host = "0.0.0.0"
port = 8080
base_url = "http://localhost:8080"
base_path = "updater"

[cache]
enabled = true
auto_scan = true
rescan_interval = 30

[[servers]]
name = "survival"
enabled = true
loader = "fabric"
loader_version = "0.16.5"
minecraft_version = "1.21.4"
main_class = "net.fabricmc.loader.impl.launch.knot.KnotClient"
java_version = 21
enable_client = true
enable_libraries = true
enable_mods = true
```

### Complete Configuration

Complete configuration includes:
- HTTP server parameters (timeouts, compression, CORS)
- Cache configuration (intervals, batch sizes, debouncing)
- Storage backend (local or S3)
- Cloudflare integration (cache purging)
- Detailed server list with all options

## Automatic Migration

The system automatically detects:
- Missing fields → Addition with default value
- Deprecated sections → Removal with value migration
- Structure changes → Smart migration
- Old versions → Progressive update

Example:
```toml
# Old config
[cache]
scan_batch_size = 50

# After automatic migration
[cache]
[cache.batch]
client = 50
libraries = 50
mods = 50
natives = 50
assets = 50
```

## Hot-reload

The system supports hot reloading:
- Change detection via file watcher
- Debouncing to avoid multiple reloads
- Rescan pause during reload
- Added/modified server detection
- Automatic rescan of changed servers
- Server path cache update
