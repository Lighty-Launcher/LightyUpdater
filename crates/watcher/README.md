# Watcher Crate

Configuration monitoring system with hot-reloading and intelligent server change detection.

## Table of Contents

- [Architecture](docs/architecture.md) - Monitoring system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and event flow
- [Errors](docs/errors.md) - Error types documentation
- [Change Detection](docs/change-detection.md) - Configuration modification detection algorithms
- [Hot-reload](docs/hot-reload.md) - Hot-reload mechanism

## Integration

This crate integrates with:
- `lighty_config`: To load and validate the new configuration
- `lighty_cache`: To trigger rescans and manage the cache
- `lighty_filesystem`: To create folder structures for new servers
- `notify`: For system-level file monitoring
